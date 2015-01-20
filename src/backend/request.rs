use std::fmt;
use std::io::net::ip;
use url;

use server::method;
use server::header;
use server::mime;

pub trait AsUrl {
    fn scheme(&self) -> &str;
    fn host(&self) -> &url::Host;
    fn port(&self) -> &u16;
    fn path(&self) -> &Vec<String>;
    fn username(&self) -> &Option<String>;
    fn password(&self) -> &Option<String>;
    fn query(&self) -> &Option<String>;
    fn fragment(&self) -> &Option<String>;
}

pub trait Request: fmt::Show + Send + ::Extensible {
    fn remote_addr(&self) -> &ip::SocketAddr;
    fn headers(&self) -> &header::Headers;
    fn method(&self) -> &method::Method;
    fn url(&self) -> &AsUrl;
    fn body(&self) -> &Vec<u8>;

    fn is_json_body(&self) -> bool {
        let content_type = self.headers().get::<header::ContentType>(); 
        if content_type.is_some() {
            match content_type.unwrap().0 {
                mime::Mime(mime::TopLevel::Application, mime::SubLevel::Json, _) => true,
                _ => false
            }
        } else {
            false
        }
    }
}
