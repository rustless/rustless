use std::io::{Reader, IoResult, File, MemReader};
use serialize::json::Json;

use server::header::{Headers, Header, HeaderFormat};
use server::header::common::{ContentType};
use server::mime::{Mime, TopLevel, SubLevel};
use server::status::StatusCode;
use typemap::TypeMap;
use {Extensible};

pub struct Response {
    pub status: StatusCode,
    pub headers: Headers,
    pub body: Option<Box<Reader + Send>>,
    pub ext: TypeMap
}

impl Response {

    pub fn new(status: StatusCode) -> Response {
        Response {
            status: status,
            headers: Headers::new(),
            body: None,
            ext: TypeMap::new()
        }
    }

    #[allow(dead_code)]
    pub fn from_reader(status: StatusCode, body: Box<Reader + Send>) -> Response {
        Response {
            status: status,
            headers: Headers::new(),
            body: Some(body),
            ext: TypeMap::new()
        }
    }

    pub fn from_string(status: StatusCode, body: String) -> Response {
        let mut response = Response::new(status);
        response.push_string(body);
        response
    }

    pub fn set_header<H: Header + HeaderFormat>(&mut self, header: H) {
        self.headers.set(header);
    }

    pub fn set_json_content_type(&mut self) {
        self.set_header(ContentType(Mime(TopLevel::Application, SubLevel::Json, vec![])));
    }

    pub fn from_json(status: StatusCode, body: &Json) -> Response {
        let mut response = Response::new(status);
        response.set_json_content_type();
        response.push_string(body.to_string());
        response
    }

    pub fn push_string(&mut self, body: String) {
        self.body = Some(box MemReader::new(body.into_bytes()) as Box<Reader + Send>)
    }

    pub fn push_file(&mut self, path: &Path) -> IoResult<()> {
        let reader = box try!(File::open(path));
        self.body = Some(reader as Box<Reader + Send>);

        Ok(())
    }

    #[allow(dead_code)]
    pub fn from_file(path: &Path) -> IoResult<Response> {
        let mut response = Response::new(StatusCode::Ok);
        try!(response.push_file(path));
        Ok(response)
    }

    pub fn ext(&self) -> &TypeMap {
        &self.ext
    }

    pub fn ext_mut(&mut self) -> &mut TypeMap {
        &mut self.ext
    }

}

impl_extensible!(Response)

impl Reader for Response {
    fn read(&mut self, buf: &mut [u8]) -> IoResult<uint> {
        match self.body {
            Some(ref mut reader) => reader.read(buf),
            None => Ok(0u)
        }
    }
}