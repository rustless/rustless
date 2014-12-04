use std::fmt::{Show};
use std::io::net::ip::SocketAddr;
use {Extensible};
use url::Host;

use server::method::Method;
use server::header;
use server::header::Headers;
use server::mime::{Mime, TopLevel, SubLevel};

pub trait AsUrl {
    fn scheme(&self) -> &str;
    fn host(&self) -> &Host;
    fn port(&self) -> &u16;
    fn path(&self) -> &Vec<String>;
    fn username(&self) -> &Option<String>;
    fn password(&self) -> &Option<String>;
    fn query(&self) -> &Option<String>;
    fn fragment(&self) -> &Option<String>;
}

pub trait Request: Show + Send + Extensible {
    fn remote_addr(&self) -> &SocketAddr;
    fn headers(&self) -> &Headers;
    fn method(&self) -> &Method;
    fn url(&self) -> &AsUrl;
    fn body(&self) -> &Vec<u8>;

    fn is_json_body(&self) -> bool {
        let content_type = self.headers().get::<header::common::ContentType>(); 
        if content_type.is_some() {
            match content_type.unwrap().0 {
                Mime(TopLevel::Application, SubLevel::Json, _) => true,
                _ => false
            }
        } else {
            false
        }
    }
}
