use std::io::{File, Reader, IoResult};
use std::fmt::{Show, Formatter, Error};
use std::io::net::ip::SocketAddr;
use typemap::TypeMap;

use {Extensible};

use server::method::Method;
use server::header::Headers;
use backend::{Request, Url, AsUrl, WrapUrl};

#[deriving(Send)]
#[allow(dead_code)]
pub struct SimpleRequest {
    pub url: Url,
    pub ext: TypeMap,
    pub remote_addr: SocketAddr,
    pub headers: Headers,
    pub method: Method,
    pub body: Vec<u8>
}

impl Request for SimpleRequest {

    fn url(&self) -> &AsUrl {
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

    fn body(&self) -> &Vec<u8> {
        return &self.body;
    }

}

#[allow(dead_code)]
impl SimpleRequest {
    
    pub fn new(method: Method, url: ::url::Url) -> SimpleRequest {
        SimpleRequest {
            url: url.wrap_url(),
            method: method,
            ext: TypeMap::new(),
            remote_addr: from_str("127.0.0.1:8000").unwrap(),
            headers: Headers::new(),
            body: vec![]
        }
    }

    pub fn build(method: Method, url: ::url::Url, builder: |&mut SimpleRequest|) -> SimpleRequest {
        let mut srq = SimpleRequest::new(method, url);
        builder(&mut srq);

        srq
    }

    pub fn set_remote_addr(&mut self, addr: SocketAddr) {
        self.remote_addr = addr;
    }

    pub fn set_remote_str(&mut self, addr: &str) {
        self.remote_addr = from_str(addr).unwrap();
    }

    pub fn headers_mut(&mut self) -> &mut Headers {
        return &mut self.headers;
    }

    pub fn push_string(&mut self, body: String) {
        self.body = body.into_bytes()
    }

    pub fn push_file(&mut self, path: &Path) -> IoResult<()> {
        let mut reader = box try!(File::open(path));
        self.body = try!(reader.read_to_end());

        Ok(())
    }

}

impl_extensible!(SimpleRequest)

impl Show for SimpleRequest {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        try!(writeln!(f, "SimpleRequest ->"));
        try!(writeln!(f, "  url: {}", self.url));
        try!(writeln!(f, "  method: {}", self.method()));
        try!(writeln!(f, "  path: {}", self.url.path()));
        try!(writeln!(f, "  query: {}", self.url.query));
        try!(writeln!(f, "  remote_addr: {}", self.remote_addr()));
        Ok(())
    }
}