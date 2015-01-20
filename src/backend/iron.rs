use url;
use std::io::net::ip;
use plugin::{Extensible};
pub use iron::{Url, Handler};

use iron::{self};

use backend::{self};
use framework::Application;

use server::method;
use server::header;

pub type HandleResult<T> = iron::IronResult<T>;
pub type HandleSuccessResult = iron::IronResult<()>;

pub trait WrapUrl {
    fn wrap_url(self) -> Url;
}

impl WrapUrl for url::Url {
    fn wrap_url(self) -> Url {
        Url::from_generic_url(self).unwrap()
    }
}

impl backend::AsUrl for Url {
    fn scheme(&self) -> &str { self.scheme.as_slice() }
    fn host(&self) -> &url::Host { &self.host }
    fn port(&self) -> &u16 { &self.port }
    fn path(&self) -> &Vec<String> { &self.path }
    fn username(&self) -> &Option<String> { &self.username }
    fn password(&self) -> &Option<String> { &self.password }
    fn query(&self) -> &Option<String> { &self.query }
    fn fragment(&self) -> &Option<String> { &self.fragment }
}

impl backend::Request for iron::Request {
    fn remote_addr(&self) -> &ip::SocketAddr { &self.remote_addr }
    fn headers(&self) -> &header::Headers { &self.headers }
    fn method(&self) -> &method::Method { &self.method }
    fn url(&self) -> &backend::AsUrl { &self.url }
    fn body(&self) -> &Vec<u8> { &self.body }
}

impl ::Extensible for iron::Request {
    fn ext(&self) -> &::typemap::TypeMap { self.extensions() }
    fn ext_mut(&mut self) -> &mut ::typemap::TypeMap { self.extensions_mut() }
}

impl Handler for Application {
    fn call(&self, req: &mut iron::Request) -> iron::IronResult<iron::Response> {
        self.call_with_not_found(req).map(|resp| {
            iron::Response {
                status: Some(resp.status),
                headers: resp.headers,
                body: resp.body,
                extensions: resp.ext
            }
        })
    }
}
