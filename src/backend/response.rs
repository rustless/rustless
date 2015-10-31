use serialize::json;

use server::header;
use server::mime;
use server::status;
use typemap;

pub use iron::response::WriteBody;
pub use iron::response::ResponseBody;

pub struct Response {
    pub status: status::StatusCode,
    pub headers: header::Headers,
    pub body: Option<Box<WriteBody + Send>>,
    pub ext: typemap::TypeMap
}

impl Response {

    pub fn new(status: status::StatusCode) -> Response {
        Response {
            status: status,
            headers: header::Headers::new(),
            body: None,
            ext: typemap::TypeMap::new()
        }
    }

    #[allow(dead_code)]
    pub fn from(status: status::StatusCode, body: Box<WriteBody + Send>) -> Response {
        Response {
            status: status,
            headers: header::Headers::new(),
            body: Some(body),
            ext: typemap::TypeMap::new()
        }
    }

    pub fn set_header<H: header::Header + header::HeaderFormat>(&mut self, header: H) {
        self.headers.set(header);
    }

    pub fn set_json_content_type(&mut self) {
        self.set_header(header::ContentType(
            mime::Mime(mime::TopLevel::Application, mime::SubLevel::Json, vec![])
        ));
    }

    pub fn from_json(status: status::StatusCode, body: &json::Json) -> Response {
        let mut response = Response::new(status);
        response.set_json_content_type();
        response.replace_body(Box::new(body.to_string()));
        response
    }

    pub fn replace_body(&mut self, body: Box<WriteBody + Send>) {
        self.body = Some(body)
    }
}

impl_extensible!(Response);