#![crate_name = "rustless"]
#![comment = "REST-like API micro-framework for Rust"]
#![license = "MIT"]
#![crate_type = "rlib"]
#![deny(warnings)]
#![deny(bad_style)]
#![feature(macro_rules, phase, tuple_indexing)]
#[phase(plugin)]

extern crate regex_macros;
extern crate regex;
extern crate hyper;
extern crate serialize;
extern crate url;
extern crate anymap;
extern crate error;
extern crate cookie;

extern crate collections;
extern crate valico;
extern crate queryst;

pub use common::{Cookies, Static};
pub use valico::Builder as Valico;
pub use server::{Server, Request, SimpleRequest, Response};
pub use middleware::{Application, HandleResult, HandleSuccessResult};
pub use framework::{
    Endpoint, Client, Api, Namespace, Nesting, Media,
    PathVersioning, AcceptHeaderVersioning, ParamVersioning
};

#[macro_export]
macro_rules! edp_handler {
    ($edp:ident, |$client:ident, $params:ident| $blk:block) => ({
        #[allow(dead_code)]
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

#[macro_export]
macro_rules! format_error (
    ($api:ident, $t:ty, |$err:ident, $media:ident| $blk:block) => ({
        #[allow(dead_code)]
        fn error_formatter(err: &Box<Error>, $media: &Media) -> Option<Response> { 
            match err.downcast::<$t>() {
                Some($err) => {
                    $blk
                },
                None => None
            }
        }

        $api.error_formatter(error_formatter);
    });
    ($api:ident, all, |$err:ident, $media:ident| $blk:block) => ({
        #[allow(dead_code)]
        fn error_formatter($err: &Box<Error>, $media: &Media) -> Option<Response> { 
            $blk
        }

        $api.error_formatter(error_formatter);
    });
)

pub trait Extensible {
    fn ext(&self) -> &::anymap::AnyMap;
    fn ext_mut(&mut self) -> &mut ::anymap::AnyMap;
}

macro_rules! impl_extensible(
    ($t:ty) => (
        impl Extensible for $t {
            fn ext(&self) -> &::anymap::AnyMap { &self.ext }
            fn ext_mut(&mut self) -> &mut ::anymap::AnyMap { &mut self.ext }
        }
    )
)

pub mod errors;
pub mod server_backend;
mod middleware;
mod server;
pub mod framework;
pub mod common;