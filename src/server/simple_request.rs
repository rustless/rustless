use url::{Url};

use std::io::{Reader, IoResult};
use std::fmt::{Show, Formatter, FormatError};
use std::io::net::ip::SocketAddr;
use anymap::AnyMap;

use server_backend::method::Method;
use server_backend::header::Headers;

use server::{Request};

#[deriving(Send)]
#[allow(dead_code)]
pub struct SimpleRequest {
    pub url: Url,
    pub ext: AnyMap,
    pub remote_addr: SocketAddr,
    pub headers: Headers,
    pub method: Method
}

impl Request for SimpleRequest {

    fn url(&self) -> &Url {
        return &self.url;    
    }

    fn remote_addr(&self) -> &SocketAddr {
        return &self.remote_addr;
    }

    fn headers(&self) -> &Headers {
        return &self.headers;
    }

    fn method(&self) -> &Method {
        return &self.method;
    }

    fn ext(&self) -> &AnyMap {
        &self.ext
    }

    fn ext_mut(&mut self) -> &mut AnyMap {
        &mut self.ext
    }
}

impl SimpleRequest {
    
}

impl Reader for SimpleRequest {
    fn read(&mut self, buf: &mut [u8]) -> IoResult<uint> {
        self.read(buf)
    }
}

impl Show for SimpleRequest {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FormatError> {
        try!(writeln!(f, "SimpleRequest ->"));
        try!(writeln!(f, "  url: {}", self.url));
        try!(writeln!(f, "  method: {}", self.method()));
        try!(writeln!(f, "  path: {}", self.url.path()));
        try!(writeln!(f, "  query: {}", self.url.query));
        try!(writeln!(f, "  remote_addr: {}", self.remote_addr()));
        Ok(())
    }
}