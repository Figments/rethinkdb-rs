//! Primitives for Connecting to RethinkDB Server

use ql2::proto;
use std::net::TcpStream;
use std::io::Write;
use byteorder::{WriteBytesExt, LittleEndian};
use bufstream::BufStream;
use std::io::BufRead;
use std::str;
use r2d2::{self, Pool, Config as PoolConfig};
use errors::*;
use super::Result;
use commands::Query;
use super::session::Client;
use super::serde_json;
use super::types::{ServerInfo, AuthRequest, AuthResponse, AuthConfirmation};
use scram::ClientFirst;

/// Options
#[derive(Debug, Clone)]
pub struct ConnectOpts {
    pub servers: Vec<&'static str>,
    pub db: &'static str,
    pub user: &'static str,
    pub password: &'static str,
    pub ssl: Option<SslCfg>,
    server: Option<&'static str>,
}

#[derive(Debug, Clone)]
pub struct SslCfg {
    pub ca_certs: &'static str,
}

impl Default for ConnectOpts {
    fn default() -> ConnectOpts {
        ConnectOpts {
            servers: vec!["localhost:28015"],
            db: "test",
            user: "admin",
            password: "",
            ssl: None,
            server: None,
        }
    }
}

/// A connection to a RethinkDB database.
#[derive(Debug)]
pub struct Connection {
    pub stream   : TcpStream,
    pub token : u64,
    pub broken: bool,
}

impl ConnectOpts {
    /// Sets servers
    pub fn set_servers(mut self, s: Vec<&'static str>) -> Self {
        self.servers = s;
        self
    }
    /// Sets database
    pub fn set_db(mut self, d: &'static str) -> Self {
        self.db = d;
        self
    }
    /// Sets username
    pub fn set_user(mut self, u: &'static str) -> Self {
        self.user = u;
        self
    }
    /// Sets password
    pub fn set_password(mut self, p: &'static str) -> Self {
        self.password = p;
        self
    }

    /// Creates a connection pool
    pub fn connect(self) -> Result<()> {
        let logger = try!(Client::logger().read());
        trace!(logger, "Calling r.connect()");
        // If pool is already set we do nothing
        if try!(Client::pool().read()).is_some() {
            info!(logger, "A connection pool is already initialised. We will use that one instead...");
            return Ok(());
        }
        // Otherwise we set it
        let mut opts = self;
        opts.server = Some(opts.servers[0]);
        let manager = ConnectionManager::new(opts);
        let config = PoolConfig::builder()
            // If we are under load and our pool runs out of connections
            // we are doomed so we set a very high number of maximum
            // connections that can be opened
            .pool_size(1000000)
            // To counter the high number of open connections we set
            // a reasonable number of minimum connections we want to
            // keep when we are idle.
            .min_idle(Some(10))
            .build();
        let new_pool = try!(Pool::new(config, manager));
        try!(Client::set_pool(new_pool));
        info!(logger, "A connection pool has been initialised...");
        Ok(())
    }
}

impl Connection {
    pub fn new(opts: &ConnectOpts) -> Result<Connection> {
        let logger = try!(Client::logger().read());
        trace!(logger, "Calling Connection::new()");
        let server = match opts.server {
            Some(server) => server,
            None => return Err(From::from(ConnectionError::Other(String::from("No server selected.")))),
        };
        let mut conn = Connection{
            stream  : try!(TcpStream::connect(server)),
            token: 0,
            broken: false,
        };

        let _ = try!(conn.handshake(opts));
        Ok(conn)
    }

    fn handshake(&mut self, opts: &ConnectOpts) -> Result<()> {
        let logger = try!(Client::logger().read());
        let null_str = b"\0"[0];
        // Process: When you first open a connection, send the magic number
        // for the version of the protobuf you're targeting (in the [Version]
        // enum).  This should **NOT** be sent as a protobuf; just send the
        // little-endian 32-bit integer over the wire raw.  This number should
        // only be sent once per connection.
        let _ = try!(self.stream.write_u32::<LittleEndian>(proto::VersionDummy_Version::V1_0 as u32));
        {
            // The server will then respond with a NULL-terminated string response.
            // "SUCCESS" indicates that the connection has been accepted. Any other
            // response indicates an error, and the response string should describe
            // the error.
            let mut resp = Vec::new();
            let mut buf = BufStream::new(&self.stream);
            let _ = try!(buf.read_until(null_str, &mut resp));

            let _ = resp.pop();

            if resp.is_empty() {
                let msg = String::from("unable to connect for an unknown reason");
                crit!(logger, "{}", msg);
                return Err(From::from(ConnectionError::Other(msg)));
            };

            let resp = try!(str::from_utf8(&resp));
            // If it's not a JSON object it's an error
            if !resp.starts_with("{") {
                crit!(logger, "{}", resp);
                return Err(From::from(ConnectionError::Other(resp.to_string())));
            };
            let info: ServerInfo = match serde_json::from_str(&resp) {
                Ok(res) => res,
                Err(err) => {
                    crit!(logger, "{}", err);
                    return Err(From::from(err));
                },
            };

            if !info.success {
                return Err(From::from(ConnectionError::Other(resp.to_string())));
            };
        }

        let scram = ClientFirst::new(opts.user, opts.password, None).unwrap();
        let (scram, client_first) = scram.client_first();

        let ar = AuthRequest{
            protocol_version: 0,
            authentication_method: String::from("SCRAM-SHA-256"),
            authentication: client_first,
        };
        let mut msg = match serde_json::to_vec(&ar) {
            Ok(res) => res,
            Err(err) => {
                crit!(logger, "{}", err);
                return Err(From::from(err));
            },
        };
        msg.push(null_str);

        // The magic number shall be followed by an authorization key.  The
        // first 4 bytes are the length of the key to be sent as a little-endian
        // 32-bit integer, followed by the key string.  Even if there is no key,
        // an empty string should be sent (length 0 and no data).
        let info: AuthResponse;
        let _ = try!(self.stream.write_all(&msg[..]));
        {
            // The server will then respond with a NULL-terminated string response.
            // "SUCCESS" indicates that the connection has been accepted. Any other
            // response indicates an error, and the response string should describe
            // the error.
            let mut resp = Vec::new();
            let mut buf = BufStream::new(&self.stream);
            let _ = try!(buf.read_until(null_str, &mut resp));

            let _ = resp.pop();

            if resp.is_empty() {
                let msg = String::from("unable to connect for an unknown reason");
                crit!(logger, "{}", msg);
                return Err(From::from(ConnectionError::Other(msg)));
            };

            let resp = try!(str::from_utf8(&resp));
            // If it's not a JSON object it's an error
            if !resp.starts_with("{") {
                crit!(logger, "{}", resp);
                return Err(From::from(ConnectionError::Other(resp.to_string())));
            };
            info  = match serde_json::from_str(&resp) {
                Ok(res) => res,
                Err(err) => {
                    crit!(logger, "{}", err);
                    return Err(From::from(err));
                },
            };

            if !info.success {
                let mut err = resp.to_string();
                if let Some(e) = info.error {
                    err = e;
                }
                // If error code is between 10 and 20, this is an auth error
                if let Some(10 ... 20) = info.error_code {
                    return Err(From::from(DriverError::Auth(err)));
                } else {
                    return Err(From::from(ConnectionError::Other(err)));
                }
            };
        }

        if let Some(auth) = info.authentication {
            let scram = scram.handle_server_first(&auth).unwrap();
            let (scram, client_final) = scram.client_final();
            let auth = AuthConfirmation {
                authentication: client_final,
            };
            let mut msg = match serde_json::to_vec(&auth) {
                Ok(res) => res,
                Err(err) => {
                    crit!(logger, "{}", err);
                    return Err(From::from(err));
                },
            };
            msg.push(null_str);
            let _ = try!(self.stream.write_all(&msg[..]));

            let mut resp = Vec::new();
            let mut buf = BufStream::new(&self.stream);
            let _ = try!(buf.read_until(null_str, &mut resp));

            let _ = resp.pop();

            if resp.is_empty() {
                let msg = String::from("unable to connect for an unknown reason");
                crit!(logger, "{}", msg);
                return Err(From::from(ConnectionError::Other(msg)));
            };

            let resp = try!(str::from_utf8(&resp));
            // If it's not a JSON object it's an error
            if !resp.starts_with("{") {
                crit!(logger, "{}", resp);
                return Err(From::from(ConnectionError::Other(resp.to_string())));
            };
            let info: AuthResponse  = match serde_json::from_str(&resp) {
                Ok(res) => res,
                Err(err) => {
                    crit!(logger, "{}", err);
                    return Err(From::from(err));
                },
            };

            if !info.success {
                let mut err = resp.to_string();
                if let Some(e) = info.error {
                    err = e;
                }
                // If error code is between 10 and 20, this is an auth error
                if let Some(10 ... 20) = info.error_code {
                    return Err(From::from(DriverError::Auth(err)));
                } else {
                    return Err(From::from(ConnectionError::Other(err)));
                }
            };
            if let Some(auth) = info.authentication {
                let _ = scram.handle_server_final(&auth).unwrap();
            }
        }
        let _ = try!(self.stream.flush());

        Ok(())
    }
}

pub struct ConnectionManager(ConnectOpts);

impl ConnectionManager {
    pub fn new(opts: ConnectOpts) -> Self {
        ConnectionManager(opts)
    }
}

impl r2d2::ManageConnection for ConnectionManager {
    type Connection = Connection;
    type Error = Error;

    fn connect(&self) -> Result<Connection> {
        Connection::new(&self.0)
    }

    fn is_valid(&self, mut conn: &mut Connection) -> Result<()> {
        let logger = try!(Client::logger().read());
        conn.token += 1;
        let query = Query::wrap(
            proto::Query_QueryType::START,
            Some(String::from("1")),
            None);
        try!(Query::write(&query, &mut conn));
        let resp = try!(Query::read(&mut conn));
        let resp = try!(str::from_utf8(&resp));
        if resp != r#"{"t":1,"r":[1]}"# {
            warn!(logger, "Got {} from server instead of the expected `is_valid()` response.", resp);
            return Err(
                From::from(
                    ConnectionError::Other(
                        String::from("Unexpected response from server."))));
        }
        Ok(())
    }

    fn has_broken(&self, conn: &mut Connection) -> bool {
        if conn.broken {
            return true;
        }
        match conn.stream.take_error() {
            Ok(error) => if error.is_some() { return true; },
            Err(_) => { return true; },
        }
        false
    }
}
