#![crate_name = "rustless"]
#![crate_type = "rlib"]
#![feature(plugin)]
#![feature(collections)]
#![feature(core)]
#![feature(io)]
#![feature(std_misc)]
#![feature(env)]
#![feature(path)]
// #![deny(warnings)]
#![deny(bad_style)]

#[plugin]
extern crate regex_macros;
extern crate regex;
extern crate hyper;
extern crate "rustc-serialize" as serialize;
extern crate url;
extern crate error;
extern crate cookie;

extern crate iron;
extern crate typemap;
extern crate plugin;

extern crate collections;
extern crate valico;
extern crate queryst;
extern crate jsonway;

#[macro_use] #[no_link]
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