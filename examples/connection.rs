extern crate slog_term;
#[macro_use] extern crate slog;
#[macro_use] extern crate reql;

use slog::DrainExt;
use reql::Client;

fn main() {
    // Build an output drain
    let drain = slog_term::streamer().compact().build();

    // Setup a logger
    let logger = slog::Logger::root(drain.fuse(), o!());

    // Create a new ReQL client with the logger
    let r = Client::new().with_logger(logger);

    // Create a connection pool
    let _pool = r.connect(args!()).unwrap();
}
