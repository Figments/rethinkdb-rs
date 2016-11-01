//! Prelude

pub use command::Response;
pub use futures::Future;
pub use futures::stream::Stream;
pub use serde_json::{Value, from_str, to_string};

pub trait ResponseExt {
    fn consume(self) {
        for _ in self.wait() { }
    }
}
