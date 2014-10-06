use url::{Url};

use std::io::{Reader, IoResult};
use std::fmt::{Show, Formatter, FormatError};
use std::io::net::ip::SocketAddr;
use anymap::AnyMap;

use hyper;
use hyper::method::Method;
use hyper::header;
use hyper::header::Headers;
use hyper::server::Request as HyperRequest;
use hyper::mime::{Mime, Application, Json};

pub struct Request {
    pub url: Url,
    pub ext: AnyMap,
    raw: HyperRequest
}

impl Show for Request {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FormatError> {
        try!(writeln!(f, "Request ->"));
        try!(writeln!(f, "  url: {}", self.url));
        try!(writeln!(f, "  method: {}", self.method()));
        try!(writeln!(f, "  path: {}", self.url.path()));
        try!(writeln!(f, "  query: {}", self.url.query));
        try!(writeln!(f, "  remote_addr: {}", self.remote_addr()));
        Ok(())
    }
}

impl Request {
    pub fn new(url: Url, req: HyperRequest) -> Request {
        Request {
            url: url,
            raw: req,
            ext: AnyMap::new(),
        }
    }

    pub fn remote_addr(&self) -> &SocketAddr {
        return &self.raw.remote_addr;
    }

    pub fn headers(&self) -> &Headers {
        return &self.raw.headers;
    }

    pub fn method(&self) -> &Method {
        return &self.raw.method;
    }

    pub fn wrap(req: HyperRequest) -> Result<Request, String> {
        
        let url = match req.uri {
            hyper::uri::AbsolutePath(ref path) => {
                match req.headers.get::<header::common::Host>() {
                    Some(host) => format!("http://{}{}", host.0, path),
                    None => return Err("No HOST header specified in request".to_string())
                }
            },
            hyper::uri::AbsoluteUri(ref uri) => format!("{}", uri),
            _ => return Err("Unsupported request URI".to_string())
        };

        println!("Url is {}", url);

        let parsed_url = match Url::parse(url.as_slice()) {
            Ok(url) => url,
            Err(parse_error) => return Err(format!("{}", parse_error))
        };

        Ok(Request::new(parsed_url, req))

    }

    pub fn is_json_body(&self) -> bool {
        let content_type = self.headers().get::<header::common::ContentType>(); 
        if content_type.is_some() {
            println!("ContentType: {}", content_type.unwrap().0);
            match content_type.unwrap().0 {
                Mime(Application, Json, _) => true,
                _ => false
            }
        } else {
            false
        }
    }
}

impl Reader for Request {
    fn read(&mut self, buf: &mut [u8]) -> IoResult<uint> {
        self.raw.read(buf)
    }
}