use std::old_io;
use std::fmt;
use std::old_io::net::ip;
use typemap;

use {Extensible};

use server::method;
use server::header;
use super::request;
use backend::{Request, Url, AsUrl, WrapUrl};

#[allow(dead_code)]
pub struct SimpleRequest {
    pub url: Url,
    pub ext: typemap::TypeMap,
    pub remote_addr: ip::SocketAddr,
    pub headers: header::Headers,
    pub method: method::Method,
    pub body: Box<Reader + 'static>
}

impl<'a> Request for SimpleRequest {

    fn url(&self) -> &AsUrl {
        return &self.url;
    }

    fn remote_addr(&self) -> &ip::SocketAddr {
        return &self.remote_addr;
    }

    fn headers(&self) -> &header::Headers {
        return &self.headers;
    }

    fn method(&self) -> &method::Method {
        return &self.method;
    }

    fn body(&self) -> &request::Body {
        return &self.body;
    }

    fn body_mut(&mut self) -> &mut request::Body {
        return &mut self.body;
    }

}

#[allow(dead_code)]
impl SimpleRequest {

    pub fn new(method: method::Method, url: ::url::Url) -> SimpleRequest {
        SimpleRequest {
            url: url.wrap_url(),
            method: method,
            ext: typemap::TypeMap::new(),
            remote_addr: "127.0.0.1:8000".parse().unwrap(),
            headers: header::Headers::new(),
            body: Box::new(old_io::MemReader::new(vec![]))
        }
    }

    pub fn build<F>(method: method::Method, url: ::url::Url, builder: F) -> SimpleRequest
    where F: Fn(&mut SimpleRequest) {
        let mut srq = SimpleRequest::new(method, url);
        builder(&mut srq);

        srq
    }

    pub fn set_remote_addr(&mut self, addr: ip::SocketAddr) {
        self.remote_addr = addr;
    }

    pub fn set_remote_str(&mut self, addr: &str) {
        self.remote_addr = addr.parse().unwrap();
    }

    pub fn headers_mut(&mut self) -> &mut header::Headers {
        return &mut self.headers;
    }

    pub fn push_string(&mut self, body: String) {
        self.body = Box::new(old_io::MemReader::new(body.into_bytes()));
    }

    pub fn push_file(&mut self, path: &Path) -> old_io::IoResult<()> {
        self.body = Box::new(try!(old_io::File::open(path)));

        Ok(())
    }

}

impl_extensible!(SimpleRequest);

impl fmt::Debug for SimpleRequest {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        try!(writeln!(f, "SimpleRequest ->"));
        try!(writeln!(f, "  url: {}", self.url));
        try!(writeln!(f, "  method: {}", self.method()));
        try!(writeln!(f, "  path: {:?}", self.url.path()));
        try!(writeln!(f, "  query: {:?}", self.url.query));
        try!(writeln!(f, "  remote_addr: {}", self.remote_addr()));
        Ok(())
    }
}