// #![deny(warnings)]
#![deny(bad_style)]

#[macro_use]
extern crate rustless;
extern crate hyper;
extern crate url;
extern crate valico;
extern crate jsonway;
extern crate serde;
extern crate serde_json;

#[macro_export]
macro_rules! sr {
    ($method:ident, $url:expr) => {
        ::rustless::SimpleRequest::new(::rustless::server::method::Method::$method, ::url::Url::parse($url).unwrap())
    };
    ($method:ident, $url:expr, $blk:expr) => {
        ::rustless::SimpleRequest::build(::rustless::server::method::Method::$method, ::url::Url::parse($url).unwrap(), $blk)
    };
}

#[macro_export]
macro_rules! call_app {
    ($app:ident, $method:ident, $url:expr) => {
        $app.call(&mut sr!($method, $url))
    };
    ($app:ident, $method:ident, $url:expr, $blk:expr) => {
        $app.call(&mut sr!($method, $url, $blk))
    };
}

#[macro_export]
macro_rules! resp_body {
    ($resp:ident) => {
        {
            let mut vec = Vec::new();
            
            $resp.body.unwrap().write_body(&mut ::rustless::ResponseBody::new(&mut vec)).expect("Can't write");
            String::from_utf8(vec).unwrap()
        }
    }
}

#[macro_export]
macro_rules! mime {
    ($mime:expr) => ($mime.parse().unwrap())
}

macro_rules! app {
    ($builder:expr) => ({
        let app = ::rustless::Application::new(::rustless::Api::build($builder));
        app
    })
}

macro_rules! edp_stub_handler {
    ($endpoint:ident) => ({
        $endpoint.handle(|client, _params| {
            client.text("Some usefull info".to_string())
        })
    })
}

macro_rules! edp_stub {
    ($api:ident) => ({
        $api.get("info", |endpoint| {
            edp_stub_handler!(endpoint)
        });
    })
}

mod versioning;
mod namespace;
mod params;
mod prefix;
mod redirect;
mod callbacks;
mod serializers;
