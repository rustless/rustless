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
extern crate error;
extern crate cookie;

extern crate iron;
extern crate typemap;
extern crate plugin;

extern crate collections;
extern crate valico;
extern crate queryst;

pub use valico::Builder as Valico;
pub use backend::{Request, SimpleRequest, Response};
pub use backend::{Handler, HandleResult, HandleSuccessResult};
pub use framework::{
    Endpoint, Client, Api, Application, Namespace, Nesting, Media, Versioning
};

#[macro_export]
macro_rules! edp_handler {
    ($edp:ident, |$client:ident, $params:ident| $blk:block) => ({
        #[allow(dead_code)]
        #[allow(unused_mut)]
        fn endpoint_handler<'a>(mut $client: Client<'a>, $params: &JsonObject) -> HandleResult<Client<'a>> {
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

pub trait Extensible for Sized? {
    fn ext(&self) -> &::typemap::TypeMap;
    fn ext_mut(&mut self) -> &mut ::typemap::TypeMap;
}

macro_rules! impl_extensible(
    ($t:ty) => (
        impl Extensible for $t {
            fn ext(&self) -> &::typemap::TypeMap { &self.ext }
            fn ext_mut(&mut self) -> &mut ::typemap::TypeMap { &mut self.ext }
        }
    )
)

pub mod errors;
pub mod server;
pub mod backend;
pub mod framework;
pub mod batteries;