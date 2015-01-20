#![crate_name = "rustless"]
#![crate_type = "rlib"]
#![feature(plugin)]
#![allow(unstable)]
#![deny(warnings)]
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

pub use backend::{Request, SimpleRequest, Response};
pub use backend::{Handler, HandleResult, HandleSuccessResult};
pub use framework::{
    Endpoint, Client, Api, Application, Namespace, Nesting, Media, Versioning
};

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

pub mod errors;
pub mod server;
pub mod backend;
pub mod framework;
pub mod batteries;