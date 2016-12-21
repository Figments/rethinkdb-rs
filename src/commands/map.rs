#![allow(dead_code)]

use ql2::types;
use ql2::proto::{Term, Term_TermType as TermType};
use ::Client;
use super::Arg;
use serde_json::value::ToJson;

impl<O> Client<types::Table, O>
    where O: ToJson + Clone
{
    pub fn map<T>(self, arg: T) -> Client<types::Stream, ()>
        where T: Into<MapArg<types::Stream>>
    {
        let arg: Vec<types::Stream> = arg.into().into();
        super::make_cmd(TermType::MAP, Some(arg), None, Some(self.cmd), self.errors)
    }
}

impl<O> Client<types::Stream, O>
    where O: ToJson + Clone
{
    pub fn map<T>(self, arg: T) -> Client<types::Stream, ()>
        where T: Into<MapArg<types::Stream>>
    {
        let arg: Vec<types::Stream> = arg.into().into();
        super::make_cmd(TermType::MAP, Some(arg), None, Some(self.cmd), self.errors)
    }
}

impl<O> Client<types::StreamSelection, O>
    where O: ToJson + Clone
{
    pub fn map<T>(self, arg: T) -> Client<types::Stream, ()>
        where T: Into<MapArg<types::Stream>>
    {
        let arg: Vec<types::Stream> = arg.into().into();
        super::make_cmd(TermType::MAP, Some(arg), None, Some(self.cmd), self.errors)
    }
}

impl<O> Client<types::Array, O>
    where O: ToJson + Clone
{
    pub fn map<T>(self, arg: T) -> Client<types::Array, ()>
        where T: Into<types::Array>
    {
        super::make_cmd(TermType::MAP, Some(vec![arg.into()]), None, Some(self.cmd), self.errors)
    }
}

pub struct MapArg<T>(Vec<T>);

impl From<MapArg<types::Stream>> for Vec<types::Stream> {
    fn from(t: MapArg<types::Stream>) -> Vec<types::Stream> {
        t.0
    }
}

impl<F, T, O> From<F> for MapArg<types::Stream>
    where T: types::DataType,
          O: ToJson + Clone,
          F: Fn(Arg) -> Client<T, O>
{
    fn from(t: F) -> MapArg<types::Stream> {
        let res = t(var!());
        let term = func!(res.into());
        MapArg(vec![term.into()])
    }
}

pub trait Stream: types::DataType {}

impl Stream for types::Table {}

impl<F, CT, CO, T, O> From<(Client<CT, CO>, F)> for MapArg<types::Stream>
    where CT: Stream,
          CO: ToJson + Clone,
          T: types::DataType,
          O: ToJson + Clone,
          F: Fn(Arg, Arg) -> Client<T, O>
{
    fn from(t: (Client<CT, CO>, F)) -> MapArg<types::Stream> {
        let arg: Term = t.0.into();
        let res = t.1(var!(), var!());
        let func = func!(res.into());
        MapArg(vec![arg.into(), func.into()])
    }
}

impl<F, CT, CO, T, O> From<(Client<CT, CO>, Client<CT, CO>, F)> for MapArg<types::Stream>
    where CT: Stream,
          CO: ToJson + Clone,
          T: types::DataType,
          O: ToJson + Clone,
          F: Fn(Arg, Arg, Arg) -> Client<T, O>
{
    fn from(t: (Client<CT, CO>, Client<CT, CO>, F)) -> MapArg<types::Stream> {
        let arg0: Term = t.0.into();
        let arg1: Term = t.1.into();
        let res = t.2(var!(), var!(), var!());
        let func = func!(res.into());
        MapArg(vec![arg0.into(), arg1.into(), func.into()])
    }
}
