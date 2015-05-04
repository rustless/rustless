#![crate_name = "rustless"]
#![crate_type = "rlib"]
// #![deny(warnings)]
#![deny(bad_style)]

extern crate regex;
extern crate hyper;
extern crate rustc_serialize as serialize;
extern crate url;
extern crate error;
extern crate cookie;

extern crate iron;
extern crate typemap;
extern crate plugin;
extern crate bodyparser;

extern crate valico;
extern crate queryst;
extern crate jsonway;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate log;

#[macro_use]
extern crate mopa;

pub use backend::{Request, SimpleRequest, Response, Handler, HandleResult, HandleSuccessResult};
pub use errors::{ErrorResponse};
pub use framework::{
    Endpoint, Client, Api, Application, Namespace, Nesting, Media, Versioning
};

pub mod prelude {
    pub use {Nesting, Extensible};
}

pub trait Extensible {
    fn ext(&self) -> &::typemap::TypeMap;
    fn ext_mut(&mut self) -> &mut ::typemap::TypeMap;
}

macro_rules! impl_extensible {
    ($t:ty) => (
        impl $crate::Extensible for $t {
            fn ext(&self) -> &::typemap::TypeMap { &self.ext }
            fn ext_mut(&mut self) -> &mut ::typemap::TypeMap { &mut self.ext }
        }
    )
}

#[macro_use] pub mod errors;
#[macro_use] pub mod backend;
pub mod server;
pub mod framework;
pub mod batteries;