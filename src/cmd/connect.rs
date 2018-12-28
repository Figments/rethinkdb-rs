use std::{
    str,
    net::SocketAddr,
    sync::atomic::{AtomicBool, AtomicU64, Ordering::SeqCst},
};

use crate::{
    r, Result, error,
    opt::Connect,
};
use futures::prelude::*;
use romio::TcpStream;
use serde::{Serialize, Deserialize};
use scram::client::{ScramClient, ServerFirst, ServerFinal};
use crossbeam_skiplist::SkipMap;

const NULL_BYTE: u8 = b'\0';
const BUF_SIZE: usize = 512;

pub(crate) type RequestId = u64;
pub(crate) type Controller = SkipMap<RequestId, ()>;

#[derive(Debug)]
pub struct Connection {
    stream: TcpStream,
    broken: AtomicBool,
    counter: AtomicU64,
    controller: Controller,
}

enum Version {
    V1_0 = 0x34c2bdc3, // Users and permissions
}

#[derive(Serialize, Deserialize, Debug)]
struct ServerInfo {
    success: bool,
    min_protocol_version: usize,
    max_protocol_version: usize,
    server_version: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct AuthRequest {
    protocol_version: i32,
    authentication_method: String,
    authentication: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct AuthResponse {
    success: bool,
    authentication: Option<String>,
    error_code: Option<usize>,
    error: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct AuthConfirmation {
    authentication: String,
}

struct HandShake {
    // this should be enough for the handshake messages
    buf: [u8; BUF_SIZE],
    conn: Connection,
}

impl r {
    pub async fn connect<'a>(self, opt: Connect<'a>) -> Result<Connection> {
        let addr = SocketAddr::new(opt.host, opt.port);
        let stream = await!(TcpStream::connect(&addr))?;
        let conn = Connection {
            stream,
            broken: AtomicBool::new(false),
            // Start counting from 1 because we want to use 0 to detect when the
            // token wraps over. If we allow tokens to be reused, the client may
            // return data not meant for that particular connection which may be
            // a security risk. This effectively means that a connection may be
            // used by `run` only up to `usize::max_value()` times.
            counter: AtomicU64::new(1),
            controller: SkipMap::new(),
        };
        let handshake = HandShake { conn, buf: [0u8; BUF_SIZE] };
        await!(handshake.greet(opt))
    }
}

impl Connection {
    pub(crate) fn stream(&self) -> &TcpStream {
        &self.stream
    }

    pub(crate) fn mark_broken(&self) {
        self.broken.store(true, SeqCst);
    }

    pub fn broken(&self) -> bool {
        self.broken.load(SeqCst)
    }

    pub(crate) fn token(&self) -> Result<RequestId> {
        let id = self.counter.fetch_add(1, SeqCst);
        if id == 0 {
            self.mark_broken();
            return Err(error::Driver::TokenOverflow)?;
        }
        Ok(id)
    }

    pub(crate) fn controller(&self) -> &Controller {
        &self.controller
    }
}

impl HandShake {
    // Performs the actual handshake
    //
    // This method optimises message exchange as suggested in the RethinkDB
    // documentation by sending message 3 right after message 1, without waiting
    // for message 2 first.
    async fn greet<'a>(mut self, opt: Connect<'a>) -> Result<Connection> {
        // Send the version we support
        let version = (Version::V1_0 as u32).to_le_bytes();
        await!(self.conn.stream.write_all(&version))?; // message 1

        // Send client first message
        let scram = ScramClient::new(opt.user, opt.password, None)?;
        let (scram, msg) = client_first(scram)?;
        await!(self.conn.stream.write_all(&msg))?; // message 3

        // Receive supported versions
        await!(self.conn.stream.read(&mut self.buf))?; // message 2
        let (len, info) = self.read_buf(0);
        ServerInfo::from_slice(info)?;

        // Receive server first message
        let offset = len + 1;
        let resp = if offset < BUF_SIZE && self.buf[offset] != NULL_BYTE {
            self.read_buf(offset).1
        } else {
            await!(self.conn.stream.read(&mut self.buf))?; // message 4
            self.read_buf(0).1
        };
        let info = AuthResponse::from_slice(resp)?;
        let auth = match info.authentication {
            Some(auth) => auth,
            None => {
                let msg = String::from("server did not send authentication info");
                return Err(error::Driver::Other(msg))?;
            }
        };

        // Send client final message
        let (scram, msg) = client_final(scram, &auth)?;
        await!(self.conn.stream.write_all(&msg))?; // message 5

        // Receive server final message
        await!(self.conn.stream.read(&mut self.buf))?; // message 6
        server_final(scram, self.read_buf(0).1)?;

        await!(self.conn.stream.flush())?;
        Ok(self.conn)
    }

    fn read_buf(&self, offset: usize) -> (usize, &[u8]) {
        let len = (&self.buf[offset..]).iter()
            .take_while(|x| **x != NULL_BYTE)
            .count();
        let max = offset + len;
        (max, &self.buf[offset..max])
    }
}

impl ServerInfo {
    fn from_slice(resp: &[u8]) -> Result<Self> {
        match serde_json::from_slice::<ServerInfo>(resp) {
            Ok(info) => {
                if !info.success {
                    return Err(error::Runtime::Internal(resp.to_owned()))?;
                }
                Ok(info)
            }
            Err(_) => {
                let msg = str::from_utf8(resp)?;
                Err(error::Driver::Other(msg.to_owned()))?
            }
        }
    }
}

impl AuthResponse {
    fn from_slice(resp: &[u8]) -> Result<Self> {
        match serde_json::from_slice::<AuthResponse>(resp) {
            Ok(info) => {
                if !info.success {
                    // If error code is between 10 and 20, this is an auth error
                    if let Some(10...20) = info.error_code {
                        if let Some(msg) = info.error {
                            return Err(error::Driver::Auth(msg))?;
                        }
                    }
                    return Err(error::Runtime::Internal(resp.to_owned()))?;
                }
                Ok(info)
            }
            Err(_) => {
                let msg = str::from_utf8(resp)?;
                Err(error::Driver::Other(msg.to_owned()))?
            }
        }
    }
}

fn client_first<'a>(scram: ScramClient<'a>) -> Result<(ServerFirst<'a>, Vec<u8>)> {
    let (scram, client_first) = scram.client_first();
    let ar = AuthRequest {
        protocol_version: 0,
        authentication_method: String::from("SCRAM-SHA-256"),
        authentication: client_first,
    };
    let mut msg = serde_json::to_vec(&ar)?;
    msg.push(NULL_BYTE);
    Ok((scram, msg))
}

fn client_final<'a>(scram: ServerFirst<'a>, auth: &str) -> Result<(ServerFinal, Vec<u8>)> {
    let scram = scram.handle_server_first(auth)?;
    let (scram, client_final) = scram.client_final();
    let conf = AuthConfirmation { authentication: client_final };
    let mut msg = serde_json::to_vec(&conf)?;
    msg.push(NULL_BYTE);
    Ok((scram, msg))
}

fn server_final(scram: ServerFinal, resp: &[u8]) -> Result<()> {
    let info = AuthResponse::from_slice(resp)?;
    if let Some(auth) = info.authentication {
        if let Err(err) = scram.handle_server_final(&auth) {
            return Err(err)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::r;
    use futures::executor::block_on;

    #[test]
    fn driver_can_connect() {
        block_on(r.connect(Default::default())).unwrap();
    }
}
