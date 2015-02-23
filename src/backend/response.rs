use std::old_io;
use serialize::json;

use server::header;
use server::mime;
use server::status;
use typemap;

pub struct Response {
    pub status: status::StatusCode,
    pub headers: header::Headers,
    pub body: Option<Box<old_io::Reader + Send>>,
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
    pub fn from_reader(status: status::StatusCode, body: Box<old_io::Reader + Send>) -> Response {
        Response {
            status: status,
            headers: header::Headers::new(),
            body: Some(body),
            ext: typemap::TypeMap::new()
        }
    }

    pub fn from_string(status: status::StatusCode, body: String) -> Response {
        let mut response = Response::new(status);
        response.push_string(body);
        response
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
        response.push_string(body.to_string());
        response
    }

    pub fn push_string(&mut self, body: String) {
        self.body = Some(Box::new(old_io::MemReader::new(body.into_bytes())) as Box<old_io::Reader + Send>)
    }

    pub fn push_file(&mut self, path: &Path) -> old_io::IoResult<()> {
        let reader = Box::new(try!(old_io::File::open(path)));
        self.body = Some(reader as Box<old_io::Reader + Send>);

        Ok(())
    }

    #[allow(dead_code)]
    pub fn from_file(path: &Path) -> old_io::IoResult<Response> {
        let mut response = Response::new(status::StatusCode::Ok);
        try!(response.push_file(path));
        Ok(response)
    }

}

impl_extensible!(Response);

impl Reader for Response {
    fn read(&mut self, buf: &mut [u8]) -> old_io::IoResult<usize> {
        match self.body {
            Some(ref mut reader) => reader.read(buf),
            None => Ok(0usize)
        }
    }
}