//! The errors returned by this driver

mod impls;

use std::{io, str};

use serde_json::error as js;

/// The most generic error message in ReQL
#[derive(Debug)]
pub enum Error {
    Compile(Vec<u8>),
    Runtime(Runtime),
    Driver(Driver),
}

/// The parent class of all runtime errors
///
/// All errors on the server unrelated to compilation. Programs may use this to catch any runtime
/// error, but the server will always return a more specific error class.
#[derive(Debug)]
pub enum Runtime {
    /// The query contains a logical impossibility, such as adding a number to a string.
    QueryLogic(Vec<u8>),
    NonExistence(Vec<u8>),
    ResourceLimit(Vec<u8>),
    User(Vec<u8>),
    Internal(Vec<u8>),
    Timeout(Vec<u8>),
    Availability(Availability),
    Permission(Vec<u8>),
}

/// A server in the cluster is unavailable
///
/// The parent class of `OpFailedError` and `OpIndeterminateError`. Programs may use this
/// to catch any availability error, but the server will always return one of this class’s
/// children.
#[derive(Debug)]
pub enum Availability {
    OpFailed(Vec<u8>),
    OpIndeterminate(Vec<u8>),
}

/// An error has occurred within the driver
///
/// This may be a driver bug, or it may be an unfulfillable command, such as an unserializable
/// query.
#[derive(Debug)]
pub enum Driver {
    Auth(String),
    Utf8(str::Utf8Error),
    Scram(scram::Error),
    Io(io::Error),
    Json(js::Error),
    ConnectionBroken,
    // The connection token has exhausted all possible IDs
    TokenOverflow,
    Other(String),
}
