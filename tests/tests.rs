#![feature(phase)]
#![feature(macro_rules)]

#![deny(warnings)]
#![deny(bad_style)]

#[phase(plugin)]
extern crate rustless;
extern crate rustless;
extern crate hyper;
extern crate serialize;
extern crate url;
extern crate jsonway;

#[macro_export]
macro_rules! sr {
    ($edp:ident, $url:expr) => {
        SimpleRequest::new($edp, Url::parse($url).unwrap())
    };
    ($edp:ident, $url:expr, $blk:expr) => {
        SimpleRequest::build($edp, Url::parse($url).unwrap(), $blk)
    };
}

#[macro_export]
macro_rules! call_app {
    ($app:ident, $edp:ident, $url:expr) => {
        $app.call_with_not_found(&mut sr!($edp, $url))
    };    
    ($app:ident, $edp:ident, $url:expr, $blk:expr) => {
        $app.call_with_not_found(&mut sr!($edp, $url, $blk))
    };
}

#[macro_export]
macro_rules! resp_body (
    ($resp:ident) => (str::from_utf8($resp.read_to_end().unwrap().as_slice()).unwrap())
)

#[macro_export]
macro_rules! mime(
    ($mime:expr) => (from_str($mime).unwrap())
)

macro_rules! app(
    ($builder:expr) => ({
        let app = Application::new(Api::build($builder));
        app
    })
)

macro_rules! edp_stub_handler(
    ($endpoint:ident) => ({
        edp_handler!($endpoint, |client, _params| {
            client.text("Some usefull info".to_string())
        })
    })
)

macro_rules! edp_stub(
    ($api:ident) => ({
        $api.get("info", |endpoint| {
            edp_stub_handler!(endpoint)
        });    
    })
)

mod versioning;
mod namespace;
mod params;
mod prefix;
mod redirect;
mod callbacks;
mod serializers;
