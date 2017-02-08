mod with_args;

use std::net::SocketAddr;

use Result;
use reql_io::r2d2;
use reql_io::tokio_core::reactor::Handle;
use errors::Error;

pub struct Response<T>(T);
pub struct Connection;

#[derive(Debug, Clone)]
pub struct Opts {
    addresses: Vec<SocketAddr>,
    db: &'static str,
    user: &'static str,
    password: &'static str,
    retries: u8,
    tls: Option<TlsCfg>,
}

#[derive(Debug, Clone)]
struct TlsCfg {
    ca_certs: &'static str,
}

pub type Config = r2d2::Config<Connection, Error>;

/// Create a new connection to the database server
pub trait Connect {
    type Connection;

    fn connect(&self, args: Vec<(Config, Opts)>, handle: &Handle) -> Result<Self::Connection>;
}

/// Run the query
pub trait Run : Connect {
    fn run<T>(&self, conn: &Self::Connection) -> Response<T>;
}
