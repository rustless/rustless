#![feature(phase)]

#![deny(warnings)]
#![deny(bad_style)]

#[phase(plugin)]
extern crate rustless;
extern crate rustless;
extern crate hyper;
extern crate serialize;
extern crate url;

mod request;
mod api01;
