use url::{Url};

use std::io::{File, MemReader, Reader, IoResult};
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
    pub method: Method,
    pub body: Option<Box<Reader + Send>>
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

#[allow(dead_code)]
impl SimpleRequest {
    
    pub fn new(method: Method, url: Url) -> SimpleRequest {
        SimpleRequest {
            url: url,
            method: method,
            ext: AnyMap::new(),
            remote_addr: from_str("127.0.0.1:8000").unwrap(),
            headers: Headers::new(),
            body: None
        }
    }

    pub fn build(method: Method, url: Url, builder: |&mut SimpleRequest|) -> SimpleRequest {
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
        self.body = Some(box MemReader::new(body.into_bytes()) as Box<Reader + Send>)
    }

    pub fn push_file(&mut self, path: &Path) -> IoResult<()> {
        let reader = box try!(File::open(path));
        self.body = Some(reader as Box<Reader + Send>);

        Ok(())
    }

}

impl Reader for SimpleRequest {
    fn read(&mut self, buf: &mut [u8]) -> IoResult<uint> {
        match self.body {
            Some(ref mut reader) => reader.read(buf),
            None => Ok(0u)
        }
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