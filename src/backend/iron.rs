use url;
use std::net;
use plugin::{Extensible, Pluggable};
pub use iron::{Url, Handler};

use iron::{self};
use bodyparser;

use backend::{self};
use super::super::framework;

use server::method;
use server::header;
use super::super::errors;
use super::request;

pub type HandleResultStrict<T> = Result<T, errors::StrictErrorResponse>;
pub type HandleResult<T> = Result<T, errors::ErrorResponse>;
pub type HandleSuccessResult = HandleResult<()>;

impl<'a, 'b> request::Body for iron::request::Body<'a, 'b> { }

pub trait WrapUrl {
    fn wrap_url(self) -> Url;
}

impl WrapUrl for url::Url {
    fn wrap_url(self) -> Url {
        Url::from_generic_url(self).unwrap()
    }
}

impl backend::AsUrl for Url {
    fn scheme(&self) -> &str { &self.scheme }
    fn host(&self) -> &url::Host { &self.host }
    fn port(&self) -> &u16 { &self.port }
    fn path(&self) -> &Vec<String> { &self.path }
    fn username(&self) -> &Option<String> { &self.username }
    fn password(&self) -> &Option<String> { &self.password }
    fn query(&self) -> &Option<String> { &self.query }
    fn fragment(&self) -> &Option<String> { &self.fragment }
}

impl<'a, 'b> backend::Request for iron::Request<'a, 'b> {
    fn remote_addr(&self) -> &net::SocketAddr { &self.remote_addr }
    fn headers(&self) -> &header::Headers { &self.headers }
    fn method(&self) -> &method::Method { &self.method }
    fn url(&self) -> &backend::AsUrl { &self.url }
    fn body(&self) -> &request::Body { &self.body }
    fn body_mut(&mut self) -> &mut request::Body { &mut self.body }
    fn read_to_end(&mut self) -> Result<Option<String>, Box<errors::Error + Send>> {
        self.get::<bodyparser::Raw>().map_err(|err| Box::new(err) as Box<errors::Error + Send>)
    }
}

impl<'a, 'b> ::Extensible for iron::Request<'a, 'b> {
    fn ext(&self) -> &::typemap::TypeMap { self.extensions() }
    fn ext_mut(&mut self) -> &mut ::typemap::TypeMap { self.extensions_mut() }
}

impl Handler for framework::Application {
    fn handle<'a, 'b>(&self, req: &mut iron::Request<'a, 'b>) -> iron::IronResult<iron::Response> {
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
                let errors::StrictErrorResponse{error, response} = err_resp;
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
