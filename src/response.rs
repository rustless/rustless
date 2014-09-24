use std::io::{IoResult, File, MemReader};
use http::headers::response::HeaderCollection;
use anymap::AnyMap;
use std::path::BytesContainer;
use http::status;
use http::status::Status;
use http::method;
use http::server::response::ResponseWriter;
use http::headers::content_type::MediaType;

pub struct Response {
    pub status: Status,
    pub headers: HeaderCollection,
    pub body: Option<Box<Reader + Send>>,
    pub ext: AnyMap
}

impl Response {

    pub fn from_reader(status: status::Status, body: Box<Reader + Send>) -> Response {
        Response {
            status: status,
            headers: HeaderCollection::new(),
            body: Some(body),
            ext: AnyMap::new()
        }
    }

    pub fn from_string(status: status::Status, body: String) -> Response {
        Response::from_reader(status, box MemReader::new(body.into_bytes()) as Box<Reader + Send>)
    }

    pub fn from_file(path: &Path) -> IoResult<Response> {
        let file = try!(File::open(path));
        let mut response = Response::from_reader(
            status::Ok,
            box file as Box<Reader + Send>
        );
        // TODO: content_type
        Ok(response)
    }

    pub fn write(self, res: &mut ResponseWriter) {
        res.status = self.status;
        res.headers = self.headers;

        match self.body {
            Some(mut reader) => {
                match reader.read_to_end() {
                    Ok(content) => {
                        // Set content length and type
                        res.headers.content_length = Some(content.len());
                        res.write(content.as_slice())
                    },
                    Err(e) => Err(e)
                }.map_err(|e| {
                    println!("Error occured while writing body: {}", e);
                    res.status = status::InternalServerError;
                    res.write(b"Internal Server Error")
                       .map_err(|e| println!("Error writing error message: {}", e));
                });
            },

            _ => ()
        }
    }

}