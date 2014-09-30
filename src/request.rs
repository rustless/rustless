use url::{Url};

use std::fmt::{Show, Formatter, FormatError};
use std::io::net::ip::SocketAddr;
use http::headers::request::HeaderCollection;
use http::server::request::{AbsoluteUri, AbsolutePath};
use http::method::Method;
use http::server::request::Request as HttpRequest;
use anymap::AnyMap;

pub struct Request {
    pub url: Url,
    pub remote_addr: Option<SocketAddr>,
    pub headers: HeaderCollection,
    pub body: String,
    pub method: Method,
    pub ext: AnyMap
}

impl Show for Request {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FormatError> {
        try!(writeln!(f, "Request ->"));
        try!(writeln!(f, "  url: {}", self.url));
        try!(writeln!(f, "  method: {}", self.method));
        try!(writeln!(f, "  path: {}", self.url.path()));
        try!(writeln!(f, "  query: {}", self.url.query));
        try!(writeln!(f, "  remote_addr: {}", self.remote_addr));
        try!(writeln!(f, "  body: {}", self.body));
        Ok(())
    }
}

impl Request {
    pub fn wrap(req: HttpRequest) -> Result<Request, String> {
        
        let url = match req.request_uri {
            
            AbsoluteUri(url) => format!("{}", url),
            AbsolutePath(path) => {
                match req.headers.host {
                    Some(ref host) => format!("http://{}{}", host, path),
                    None => return Err("No HOST header specified in request".to_string())
                }
            },

            _ => return Err("Unsupported request URI".to_string())

        };

        let parsed_url = match Url::parse(url.as_slice()) {
            Ok(url) => url,
            Err(parse_error) => return Err(format!("{}", parse_error))
        };

        Ok(Request {
            url: parsed_url,
            remote_addr: req.remote_addr,
            headers: req.headers,
            body: req.body,
            method: req.method,
            ext: AnyMap::new()
        })

    }
}