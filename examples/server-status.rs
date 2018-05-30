#![feature(proc_macro)]
#![feature(proc_macro_non_items)]

extern crate futures;
extern crate reql;
extern crate reql_derive;
extern crate reql_types;
#[macro_use]
extern crate slog;
extern crate slog_term;

use reql::{Config, Client, Document, Run};
use reql_types::{Change, ServerStatus};
use reql_derive::args;
use futures::Stream;
use slog::Drain;

fn main() -> reql::Result<()> {
    // Build an output drain
    let plain = slog_term::PlainSyncDecorator::new(std::io::stdout());
    let drain = slog_term::FullFormat::new(plain).build();

    // Setup a logger
    let logger = slog::Logger::root(drain.fuse(), o!());

    // Create a new ReQL client with the logger
    let r = Client::new().with_logger(logger);

    // Create a connection pool
    let conn = r.connect(Config::default())?;

    // Run the query
    let stati = r.db("rethinkdb")
        .table("server_status")
        .changes()
        .with_args(args!({include_initial: true}))
        .run::<Change<ServerStatus, ServerStatus>>(conn)?;

    // Process results
    for res in stati.wait() {
        match res {
            Ok(Some(Document::Expected(change))) => {
                println!("{:?}", change);
            }
            res => {
                println!("unexpected response from server: {:?}", res);
            }
        }
    }

    Ok(())
}
