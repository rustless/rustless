use url::{Url};

use std::io::{Reader, IoResult};
use std::fmt::{Show, Formatter, FormatError};
use std::io::net::ip::SocketAddr;
use anymap::AnyMap;

use server_backend::method::Method;
use server_backend::header;
use server_backend::header::Headers;
use server_backend::server::Request as RawRequest;
use server_backend::mime::{Mime, Application, Json};
use server_backend::uri;

pub trait Request: Reader + Show + Send {
    fn remote_addr(&self) -> &SocketAddr;
    fn headers(&self) -> &Headers;
    fn method(&self) -> &Method;
    fn is_json_body(&self) -> bool;
    fn ext(&self) -> &AnyMap;
    fn ext_mut(&mut self) -> &mut AnyMap;
    fn url(&self) -> &Url;
}

#[deriving(Send)]
pub struct ServerRequest {
    pub url: Url,
    pub ext: AnyMap,
    raw: RawRequest
}

impl Request for ServerRequest {

    fn url(&self) -> &Url {
        return &self.url;    
    }

    fn remote_addr(&self) -> &SocketAddr {
        return &self.raw.remote_addr;
    }

    fn headers(&self) -> &Headers {
        return &self.raw.headers;
    }

    fn method(&self) -> &Method {
        return &self.raw.method;
    }

    fn is_json_body(&self) -> bool {
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

    fn ext(&self) -> &AnyMap {
        &self.ext
    }

    fn ext_mut(&mut self) -> &mut AnyMap {
        &mut self.ext
    }
}

impl ServerRequest {
    pub fn new(url: Url, req: RawRequest) -> ServerRequest {
        ServerRequest {
            url: url,
            raw: req,
            ext: AnyMap::new(),
        }
    }

    pub fn wrap(req: RawRequest) -> Result<ServerRequest, String> {
        
        let url = match req.uri {
            uri::AbsolutePath(ref path) => {
                match req.headers.get::<header::common::Host>() {
                    Some(host) => format!("http://{}{}", host.0, path),
                    None => return Err("No HOST header specified in request".to_string())
                }
            },
            uri::AbsoluteUri(ref uri) => format!("{}", uri),
            _ => return Err("Unsupported request URI".to_string())
        };

        println!("Url is {}", url);

        let parsed_url = match Url::parse(url.as_slice()) {
            Ok(url) => url,
            Err(parse_error) => return Err(format!("{}", parse_error))
        };

        Ok(ServerRequest::new(parsed_url, req))

    }
}

impl Reader for ServerRequest {
    fn read(&mut self, buf: &mut [u8]) -> IoResult<uint> {
        self.raw.read(buf)
    }
}

impl Show for ServerRequest {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FormatError> {
        try!(writeln!(f, "ServerRequest ->"));
        try!(writeln!(f, "  url: {}", self.url));
        try!(writeln!(f, "  method: {}", self.method()));
        try!(writeln!(f, "  path: {}", self.url.path()));
        try!(writeln!(f, "  query: {}", self.url.query));
        try!(writeln!(f, "  remote_addr: {}", self.remote_addr()));
        Ok(())
    }
}