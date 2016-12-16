pub mod connection;
pub mod db_create;
pub mod db;
pub mod table_create;
pub mod table;
pub mod uuid;
pub mod get;
pub mod get_all;
pub mod changes;
pub mod map;
pub mod get_field;
pub mod rem;
pub mod run;

use std::string::String as StdString;

use errors::Error;
use types::{self, Command as Cmd};
use ql2::proto::{Term, Term_TermType as TermType};
use serde_json::value::ToJson;

include!(concat!(env!("OUT_DIR"), "/opts.rs"));

#[derive(Debug)]
pub struct Client<T, O> {
    cmd: Command<T, O>,
    errors: Option<Vec<Error>>,
}

/// Convenient type for use with maps
pub type Arg = Client<types::Object, ()>;

#[allow(non_upper_case_globals)]
pub const r: Client<(), ()> = Client {
    cmd: Command((), None),
    errors: None,
};

#[derive(Debug, Clone)]
pub struct Command<T, O>(T, Option<O>);

fn make_cmd<A, T, O, PT, PO>(typ: TermType,
                                 args: Option<Vec<A>>,
                                 opts: Option<O>,
                                 cmd: Option<Command<PT, PO>>,
                                 errors: Option<Vec<Error>>)
-> Client<T, O>
where A: types::DataType,
T: types::DataType,
O: ToJson + Clone,
PT: types::DataType,
PO: ToJson + Clone
{
    let prev_cmd = match cmd {
        Some(cmd) => {
            let cmd: Term = cmd.into();
            Some(cmd)
        },
        None => None,
    };
    let mut dt = Cmd::new(typ, prev_cmd);
    if let Some(args) = args {
        for arg in args {
            dt.with_args(arg.into());
        }
    }
    Client {
        cmd: Command(dt.into(), opts),
        errors: errors,
    }
}

impl<T, O> From<Command<T, O>> for Term
    where T: types::DataType,
          O: ToJson + Clone
{
    fn from(t: Command<T, O>) -> Term {
        let term: Term = t.0.into();
        let mut cmd: Cmd = term.into();
        if let Some(opt) = t.1 {
            let obj = types::Object::from(opt);
            cmd.with_opts(obj);
        }
        cmd.into()
    }
}

impl<T, O> From<Client<T, O>> for Term
    where T: types::DataType,
          O: ToJson + Clone
{
    fn from(t: Client<T, O>) -> Term {
        t.cmd.into()
    }
}

impl<T> From<T> for Client<T, ()> {
    fn from(t: T) -> Client<T, ()>
    {
        Client {
            cmd: Command(t, None as Option<()>),
            errors: None,
        }
    }
}

impl<T, O> Command<T, O>
    where O: Clone,
{
    fn opts(&self) -> O {
        let msg = "Command options is not set. This is a bug in the driver.";
        self.1.clone().expect(msg)
    }
}

#[derive(Debug, Clone)]
pub struct RunOpts {
    read_mode: Option<ReadMode>,
    time_format: Format,
    profile: bool,
    durability: Durability,
    group_format: Format,
    db: Option<Command<types::Db, ()>>,
    array_limit: u64,
    binary_format: Format,
    min_batch_rows: u32,
    max_batch_rows: u64,
    max_batch_bytes: u64,
    max_batch_seconds: f32,
    first_batch_scaledown_factor: u64,
}

impl Default for RunOpts {
    fn default() -> RunOpts {
        RunOpts {
            read_mode: Some(ReadMode::Single),
            time_format: Format::Native,
            profile: false,
            durability: Durability::Hard,
            group_format: Format::Native,
            db: None,
            array_limit: 100_000,
            binary_format: Format::Native,
            min_batch_rows: 8,
            // 2^53 is the biggest integer that RethinkDB supports
            max_batch_rows: 2u64.pow(53),
            max_batch_bytes: 1_000_000,
            max_batch_seconds: 0.5,
            first_batch_scaledown_factor: 4,
        }
    }
}
