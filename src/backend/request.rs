use std::fmt;
use std::io::net::ip;
use url;

use framework::media;

use server::method;
use server::header;

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
        self.headers().get::<header::ContentType>().map_or(false, |ct| media::is_json(&ct.0))
    }

    fn is_urlencoded_body(&self) -> bool {
        self.headers().get::<header::ContentType>().map_or(false, |ct| media::is_urlencoded(&ct.0))
    }


    fn is_form_data_body(&self) -> bool {
        self.headers().get::<header::ContentType>().map_or(false, |ct| media::is_form_data(&ct.0))
    }
}
