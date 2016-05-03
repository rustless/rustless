use std::fmt;
use std::fs::File;
use std::io;
use std::net;
use std::path::Path;

use typemap;

use {Extensible};
use server::method;
use server::header;
use super::request;
use super::super::errors;
use backend::{Request, Url, AsUrl, WrapUrl};

#[allow(dead_code)]
pub struct SimpleRequest {
    pub url: Url,
    pub ext: typemap::TypeMap,
    pub remote_addr: net::SocketAddr,
    pub headers: header::Headers,
    pub method: method::Method,
    pub body: Box<io::Read + 'static>
}

impl<'a> Request for SimpleRequest {

    fn url(&self) -> &AsUrl {
        return &self.url;
    }

    fn remote_addr(&self) -> &net::SocketAddr {
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

    fn read_to_end(&mut self) -> Result<Option<String>, Box<errors::Error + Send>> {
        let mut bytes = Vec::new();
        self.body.read_to_end(&mut bytes).unwrap();
        String::from_utf8(bytes)
            .map(|body| Some(body))
            .map_err(|err| Box::new(err) as Box<errors::Error + Send>)
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
            body: Box::new(io::Cursor::new(vec![]))
        }
    }

    pub fn build<F>(method: method::Method, url: ::url::Url, builder: F) -> SimpleRequest
    where F: Fn(&mut SimpleRequest) {
        let mut srq = SimpleRequest::new(method, url);
        builder(&mut srq);

        srq
    }

    pub fn set_remote_addr(&mut self, addr: net::SocketAddr) {
        self.remote_addr = addr;
    }

    pub fn set_remote_str(&mut self, addr: &str) {
        self.remote_addr = addr.parse().unwrap();
    }

    pub fn headers_mut(&mut self) -> &mut header::Headers {
        return &mut self.headers;
    }

    pub fn push_string(&mut self, body: String) {
        self.body = Box::new(io::Cursor::new(body.into_bytes()));
    }

    pub fn push_file(&mut self, path: &Path) -> io::Result<()> {
        self.body = Box::new(try!(File::open(path)));

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
        try!(writeln!(f, "  query: {:?}", self.url.query()));
        try!(writeln!(f, "  remote_addr: {}", self.remote_addr()));
        Ok(())
    }
}