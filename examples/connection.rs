extern crate slog_term;
#[macro_use] extern crate slog;
extern crate tokio_core;
extern crate futures;
#[macro_use] extern crate reql;

use reql::{Client, Run};
use reql::structs::ClusterConfig;
use slog::DrainExt;
use tokio_core::reactor::Core;
use futures::stream::Stream;

fn main() {
    // Build an output drain
    let drain = slog_term::streamer().async().compact().build();

    // Setup a logger
    let logger = slog::Logger::root(drain.fuse(), o!());

    // Create a new ReQL client with the logger
    let r = Client::new().with_logger(logger);

    // Create an even loop
    let core = Core::new().unwrap();

    // Create a connection pool
    let conn = r.connect(args!(core.handle(), {servers: ["localhost"]})).unwrap();
    
    // Run the query
    let cfg = r.db("rethinkdb").table("cluster_config").run::<ClusterConfig>(conn).unwrap();

    // Process results
    for cfg in cfg.wait() {
        println!("{:?}", cfg);
    }
}
