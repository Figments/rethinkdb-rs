# ReQL Driver

This is a [ReQL] driver written in [Rust].

[![travis-badge][]][travis] [![cratesio-badge][]][cratesio] [![docsrs-badge][]][docsrs]

## Example

```rust
extern crate reql;
extern crate reql_types;
extern crate futures;

use futures::Stream;
use reql_types::ServerStatus;
use reql::{Config, Client, Document, Run};

fn main() -> reql::Result<()> {
    // Create a new ReQL client
    let r = Client::new();

    // Create a connection pool
    let conn = r.connect(Config::default())?;

    // Run the query
    let stati = r.db("rethinkdb")
        .table("server_status")
        .run::<ServerStatus>(conn)?;

    // Process the results
    match stati.wait().next().unwrap() {
        // The server returned the response we were expecting
        Ok(Some(Document::Expected(status))) => {
            println!("{:?}", status);
        }
        // We got a response alright, but it wasn't the one we were
        // expecting plus it's not an error either, otherwise it would
        // have been returned as such (This simply means that the response
        // we got couldn't be serialised into the type we were expecting)
        Ok(Some(Document::Unexpected(status))) => {
            println!("unexpected response from server: {:?}", status);
        }
        // This is impossible in this particular example since there
        // needs to be at least one server available to give this
        // response otherwise we would have run into an error for
        // failing to connect
        Ok(None) => {
            println!("got no documents in the database");
        }
        // Oops! We ran into an error
        Err(error) => {
            println!("error: {}", error);
        }
    }

    Ok(())
}
```

## Tests

The tests currently only tests a small subset of features in the driver against a live rethink database.

Running the tests requires that you have rethinkdb installed and in your path, and that the test are run
one at a time since they would otherwise destroy each other.

    cargo test --lib -- --test-threads=1

The tests run per file, in alphabetical order, and tests features gradually and most are depending on
previous that previous tests pass

## License

Licensed under either of
* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you shall be dual licensed as above, without any
additional terms or conditions.

[ReQL]: https://rethinkdb.com/api
[Rust]: https://rust-lang.org
[travis-badge]: https://travis-ci.org/RethinkDB/rethinkdb-rs.svg?branch=master
[travis]: https://travis-ci.org/RethinkDB/rethinkdb-rs
[cratesio-badge]: https://img.shields.io/crates/v/reql.svg
[cratesio]: https://crates.io/crates/reql
[docsrs-badge]: https://docs.rs/reql/badge.svg
[docsrs]: https://docs.rs/reql
