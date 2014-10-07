#![crate_name = "rustless"]
#![comment = "REST-like API micro-framework for Rust"]
#![license = "MIT"]
#![crate_type = "rlib"]
#![feature(macro_rules, phase, tuple_indexing)]
#[phase(plugin)]

extern crate regex_macros;
extern crate regex;
extern crate hyper;
extern crate serialize;
extern crate url;
extern crate anymap;
extern crate error;
extern crate collections;
extern crate valico;
extern crate query;

pub use rustless::{Rustless};
pub use request::{Request};
pub use api::{
    Endpoint, Client, Api, Namespace, NS, Versioning, PathVersioning,
    AcceptHeaderVersioning, ParamVersioning
};
pub use middleware::{Application, HandleResult, HandleSuccessResult};
pub use hyper::method::{Method, Get, Post};
pub use valico::Builder as Valico;

#[macro_export]
macro_rules! edp_handler {
    ($edp:ident, |$client:ident, $params:ident| $blk:block) => ({
        fn endpoint_handler<'a>($client: Client<'a>, $params: &Json) -> HandleResult<Client<'a>> {
            $blk
        }

        $edp.handle(endpoint_handler)
    })
}

#[macro_export]
macro_rules! callback {
    (|$client:ident| $blk:block) => ({
        fn callback<'a>($client: &mut Client<'a>) -> HandleSuccessResult {
            $blk
        }

        callback
    })
}

mod listener;
mod rustless;
mod request;
mod path;
mod middleware;
mod response;
mod api;