use url;
use std::old_io::net::ip;
use plugin::{Extensible};
pub use iron::{Url, Handler};

use iron::{self};

use backend::{self};
use framework::Application;

use server::method;
use server::header;

use super::request;
use super::response;
use super::super::errors;

pub struct ErrorResponse {
    pub error: Box<errors::Error>,
    pub response: response::Response
}

pub type HandleExtendedResult<T> = Result<T, ErrorResponse>;
pub type HandleResult<T> = Result<T, Box<errors::Error>>;
pub type HandleSuccessResult = HandleResult<()>;

impl<'a> request::Body for iron::request::Body<'a> { }

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

impl<'a> backend::Request for iron::Request<'a> {
    fn remote_addr(&self) -> &ip::SocketAddr { &self.remote_addr }
    fn headers(&self) -> &header::Headers { &self.headers }
    fn method(&self) -> &method::Method { &self.method }
    fn url(&self) -> &backend::AsUrl { &self.url }
    fn body(&self) -> &request::Body { &self.body }
    fn body_mut(&mut self) -> &mut request::Body { &mut self.body }
}

impl<'a>  ::Extensible for iron::Request<'a> {
    fn ext(&self) -> &::typemap::TypeMap { self.extensions() }
    fn ext_mut(&mut self) -> &mut ::typemap::TypeMap { self.extensions_mut() }
}

impl Handler for Application {
    fn handle<'a>(&self, req: &mut iron::Request<'a>) -> iron::IronResult<iron::Response> {
        self.call(req)
            .map(|resp| {
                iron::Response {
                    status: Some(resp.status),
                    headers: resp.headers,
                    body: resp.body,
                    extensions: resp.ext
                }
            })
            .map_err(|err_resp| {
                let ErrorResponse{error, response} = err_resp;
                iron::IronError {
                    error: error,
                    response: iron::Response {
                        status: Some(response.status),
                        headers: response.headers,
                        body: response.body,
                        extensions: response.ext
                    }
                }
            })
    }
}
