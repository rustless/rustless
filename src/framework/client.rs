use serialize::json::{Json};
use anymap::AnyMap;
use std::os;

use server::{Request, Response};
use errors::{Error, FileError};
use middleware::{HandleResult};
use framework::endpoint::Endpoint;
use framework::media::Media;
use server_backend::status;
use server_backend::mime;
use server_backend::header::Header;
use server_backend::header::common::{ContentType, Location};

pub struct Client<'a> {
    pub endpoint: &'a Endpoint,
    pub request: &'a mut Request,
    pub media: &'a Media,
    pub ext: AnyMap,
    pub response: Response
}

pub type ClientResult<'a> = HandleResult<Client<'a>>;

impl<'a> Client<'a> {

    pub fn new(endpoint: &'a Endpoint, request: &'a mut Request, media: &'a Media) -> Client<'a> {
        Client {
            endpoint: endpoint,
            request: request,
            media: media,
            ext: AnyMap::new(),
            response: Response::new(status::Ok)
        }
    }

    pub fn status(&mut self) -> status::StatusCode {
        self.response.status
    }

    pub fn set_status(&mut self, status: status::StatusCode) {
        self.response.status = status;
    }

    pub fn set_header<H: Header>(&mut self, header: H) {
        self.response.set_header(header);
    }

    pub fn set_json_content_type(&mut self) {
        self.set_header(ContentType(mime::Mime(mime::Application, mime::Json, vec![])));
    }

    pub fn set_content_type(&mut self, mime: mime::Mime) {
        self.set_header(ContentType(mime));
    }

    pub fn error<T: Error>(self, error: T) -> ClientResult<'a> {
        Err(error.erase())
    }

    pub fn json(mut self, result: &Json) -> ClientResult<'a> {
        self.set_json_content_type();
        self.response.push_string(result.to_string());

        Ok(self)
    }

    pub fn text(mut self, result: String) -> ClientResult<'a> {
        self.response.push_string(result);

        Ok(self)
    }

    pub fn file(mut self, path: &Path) -> ClientResult<'a> {
       match self.response.push_file(&os::make_absolute(path)) {
            Ok(()) => Ok(self),
            Err(err) => {
                return Err(FileError(err).erase());
            }
        } 
    }

    pub fn redirect(mut self, to: &str) -> ClientResult<'a> {
        self.set_status(status::Found);
        self.set_header(Location(to.to_string()));

        Ok(self)
    }

    pub fn permanent_redirect(mut self, to: &str) -> ClientResult<'a> {
        self.set_status(status::MovedPermanently);
        self.set_header(Location(to.to_string()));

        Ok(self)
    }

    pub fn move_response(self) -> Response {
        self.response
    }

    pub fn ext(&self) -> &AnyMap {
        &self.ext
    }

    pub fn ext_mut(&mut self) -> &mut AnyMap {
        &mut self.ext
    }
    
}