#[macro_use]
extern crate reql_derive;
extern crate ql2;
extern crate protobuf;
extern crate serde_json;

#[cfg(test)]
mod tests;

macro_rules! commands {
    ($($cmd:ident),* $(,)*) => {
        $(
            mod $cmd;
            pub use self::$cmd::*;
        )*
    }
}

macro_rules! command {
    ( $(#[$attr:meta])* ) => {
        #[derive(Command)]
        $(#[$attr])*
        struct _Dummy;
    }
}

pub mod commands;
mod types;
mod args;

use ql2::proto::Term;

#[must_use = "command results are moved from one command to another so you must either catch a command's result using a let binding or chain the command all the way through"]
pub struct Command {
    term: Option<Term>,
    idx: u32,
}

#[allow(non_upper_case_globals)]
pub const r: Command = Command{ term: None, idx: 0 };

pub trait IntoArg {
    fn into_arg(self) -> Vec<Term>;
}
