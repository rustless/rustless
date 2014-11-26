use url::{Url};

use std::io::{Reader, IoResult};
use std::fmt::{Show, Formatter};
use std::io::{MemReader};
use std::io::net::ip::SocketAddr;
use anymap::AnyMap;
use {Extensible};

use server_backend::method::Method;
use server_backend::header;
use server_backend::header::Headers;
use server_backend::server::Request as RawRequest;
use server_backend::mime::{Mime, Application, Json};
use server_backend::uri;

pub trait Request: Reader + Show + Send + Extensible {
    fn remote_addr(&self) -> &SocketAddr;
    fn headers(&self) -> &Headers;
    fn method(&self) -> &Method;
    fn url(&self) -> &Url;

    fn is_json_body(&self) -> bool {
        let content_type = self.headers().get::<header::common::ContentType>(); 
        if content_type.is_some() {
            match content_type.unwrap().0 {
                Mime(Application, Json, _) => true,
                _ => false
            }
        } else {
            false
        }
    }
}

#[deriving(Send)]
pub struct ServerRequest {
    url: Url,
    remote_addr: SocketAddr,
    headers: Headers,
    method: Method,
    body: Vec<u8>,
    pub ext: AnyMap
}

impl Request for ServerRequest {

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
}

impl_extensible!(ServerRequest)

impl ServerRequest {
    pub fn new(url: Url, remote_addr: SocketAddr, headers: Headers, method: Method, body: Vec<u8>) -> ServerRequest {
        ServerRequest {
            url: url,
            remote_addr: remote_addr,
            headers: headers,
            method: method,
            body: body,
            ext: AnyMap::new(),
        }
    }

    pub fn wrap(mut req: RawRequest) -> Result<ServerRequest, String> {
        
        let url = match req.uri {
            uri::AbsolutePath(ref path) => {
                match req.headers.get::<header::common::Host>() {
                    Some(host) => format!("http://{}{}", host.hostname, path),
                    None => return Err("No HOST header specified in request".to_string())
                }
            },
            uri::AbsoluteUri(ref uri) => format!("{}", uri),
            _ => return Err("Unsupported request URI".to_string())
        };

        let parsed_url = match Url::parse(url.as_slice()) {
            Ok(url) => url,
            Err(parse_error) => return Err(format!("{}", parse_error))
        };

        let body = match req.read_to_end() {
            Ok(body) => body,
            Err(e) => return Err(format!("Couldn't read request body: {}", e))
        };

        Ok(ServerRequest::new(parsed_url, req.remote_addr.clone(), req.headers.clone(), req.method.clone(), body))

    }
}

impl Reader for ServerRequest {
    fn read(&mut self, buf: &mut [u8]) -> IoResult<uint> {
        MemReader::new(self.body.clone()).read(buf)
    }
}

impl Show for ServerRequest {
    fn fmt(&self, f: &mut Formatter) -> Result<(), ::std::fmt::Error> {
        try!(writeln!(f, "ServerRequest ->"));
        try!(writeln!(f, "  url: {}", self.url));
        try!(writeln!(f, "  method: {}", self.method()));
        try!(writeln!(f, "  path: {}", self.url.path()));
        try!(writeln!(f, "  query: {}", self.url.query));
        try!(writeln!(f, "  remote_addr: {}", self.remote_addr()));
        Ok(())
    }
}