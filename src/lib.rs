#![crate_name = "rustless"]
#![comment = "REST-like API micro-framework for Rust"]
#![license = "MIT"]
#![crate_type = "rlib"]
#![deny(warnings)]
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

pub use valico::Builder as Valico;
pub use server::{Server, Request, Response};
pub use middleware::{Application, HandleResult, HandleSuccessResult};
pub use framework::{
    Endpoint, Client, Api, Namespace, Nesting, 
    PathVersioning, AcceptHeaderVersioning, ParamVersioning
};

#[macro_export]
macro_rules! edp_handler {
    ($edp:ident, |$client:ident, $params:ident| $blk:block) => ({
        fn endpoint_handler<'a>($client: Client<'a>, $params: &JsonObject) -> HandleResult<Client<'a>> {
            $blk
        }

        $edp.handle(endpoint_handler)
    })
}

#[macro_export]
macro_rules! callback {
    (|$client:ident, $params:ident| $blk:block) => ({
        fn callback<'a>($client: &mut Client<'a>, $params: &JsonObject) -> HandleSuccessResult {
            $blk
        }

        callback
    })
}

mod server_backend;
mod rustless;
mod middleware;
mod server;
mod framework;