pub use iron::Url;
pub use url::Url as GenericUrl;
use url::Host;

use plugin::Extensible as PluginExtensible;

use iron::{IronResult};
use iron::Request as IronRequest;
use iron::Response as IronResponse;

pub use iron::Handler;
use framework::Application;
use backend::{Request, AsUrl};

use std::io::net::ip::SocketAddr;
use server::method::Method;
use server::header::Headers;

use {Extensible};

pub type HandleResult<T> = IronResult<T>;
pub type HandleSuccessResult = IronResult<()>;

pub trait WrapUrl {
    fn wrap_url(self) -> Url;
}

impl WrapUrl for GenericUrl {
    fn wrap_url(self) -> Url {
        Url::from_generic_url(self).unwrap()
    }
}

impl AsUrl for Url {
    fn scheme(&self) -> &str { self.scheme.as_slice() }
    fn host(&self) -> &Host { &self.host }
    fn port(&self) -> &u16 { &self.port }
    fn path(&self) -> &Vec<String> { &self.path }
    fn username(&self) -> &Option<String> { &self.username }
    fn password(&self) -> &Option<String> { &self.password }
    fn query(&self) -> &Option<String> { &self.query }
    fn fragment(&self) -> &Option<String> { &self.fragment }
}

impl Request for IronRequest {
    fn remote_addr(&self) -> &SocketAddr { &self.remote_addr }
    fn headers(&self) -> &Headers { &self.headers }
    fn method(&self) -> &Method { &self.method }
    fn url(&self) -> &AsUrl { &self.url }
    fn body(&self) -> &Vec<u8> { &self.body }
}

impl Extensible for IronRequest {
    fn ext(&self) -> &::typemap::TypeMap { self.extensions() }
    fn ext_mut(&mut self) -> &mut ::typemap::TypeMap { self.extensions_mut() }
}

impl Handler for Application {
    fn call(&self, req: &mut IronRequest) -> IronResult<IronResponse> {
        self.call_with_not_found(req).map(|resp| {
            IronResponse {
                status: Some(resp.status),
                headers: resp.headers,
                body: resp.body,
                extensions: resp.ext
            }
        })
    }
}
