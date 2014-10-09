use std::io::{IoResult, File, MemReader};

use server_backend::header::Headers;
use server_backend::status;
use server_backend::status::StatusCode;
use anymap::AnyMap;

pub struct Response {
    pub status: StatusCode,
    pub headers: Headers,
    pub body: Option<Box<Reader + Send>>,
    pub ext: AnyMap
}

impl Response {

    pub fn new(status: StatusCode) -> Response {
        Response {
            status: status,
            headers: Headers::new(),
            body: None,
            ext: AnyMap::new()
        }
    }

    #[allow(dead_code)]
    pub fn from_reader(status: StatusCode, body: Box<Reader + Send>) -> Response {
        Response {
            status: status,
            headers: Headers::new(),
            body: Some(body),
            ext: AnyMap::new()
        }
    }

    pub fn from_string(status: StatusCode, body: String) -> Response {
        let mut response = Response::new(status);
        response.push_string(body);
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
        let mut response = Response::new(status::Ok);
        try!(response.push_file(path));
        Ok(response)
    }

    pub fn ext(&self) -> &AnyMap {
        &self.ext
    }

    pub fn ext_mut(&mut self) -> &mut AnyMap {
        &mut self.ext
    }

}