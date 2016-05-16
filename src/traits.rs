pub trait R {
    type Connection;
    fn connect<T: ConnectOpts>(&self, opts: T) -> Self::Connection;
}

pub trait ConnectOpts {}

pub trait Connector {
    type Connection;
    fn close(&self, noreply_wait: bool);
    fn reconnect(&self, noreply_wait: bool) -> Self::Connection;
    // use is a reserved keyword in Rust
    fn use_db(&self, db_name: &str) -> Self::Connection;
}
