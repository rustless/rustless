use serialize::json::{Json};
use typemap::TypeMap;
use std::os;

use backend::{Request, Response};
use errors::{Error, FileError};
use backend::{HandleResult};
use framework::api::{Application};
use framework::endpoint::Endpoint;
use framework::media::Media;
use server::status::StatusCode;
use server::mime::{Mime, TopLevel, SubLevel};
use server::header::{Header, HeaderFormat};
use server::header::common::{ContentType, Location};
use {Extensible};

pub struct Client<'a> {
    pub app: &'a Application,
    pub endpoint: &'a Endpoint,
    pub request: &'a mut Request,
    pub media: &'a Media,
    pub ext: TypeMap,
    pub response: Response
}

pub type ClientResult<'a> = HandleResult<Client<'a>>;

impl<'a> Client<'a> {

    pub fn new(app: &'a Application, endpoint: &'a Endpoint, request: &'a mut Request, media: &'a Media) -> Client<'a> {
        Client {
            app: app,
            endpoint: endpoint,
            request: request,
            media: media,
            ext: TypeMap::new(),
            response: Response::new(StatusCode::Ok)
        }
    }

    //
    // Work with status
    //

    pub fn status(&mut self) -> StatusCode {
        self.response.status
    }

    pub fn set_status(&mut self, status: StatusCode) {
        self.response.status = status;
    }

    pub fn unauthorized(&mut self) {
        self.response.status = StatusCode::Unauthorized;
    }

    pub fn forbidden(&mut self) {
        self.response.status = StatusCode::Forbidden;
    }

    pub fn not_found(&mut self) {
        self.response.status = StatusCode::NotFound;
    }

    pub fn internal_server_error(&mut self) {
        self.response.status = StatusCode::InternalServerError;
    }

    pub fn not_implemented(&mut self) {
        self.response.status = StatusCode::NotImplemented;
    }

    //

    pub fn set_header<H: Header + HeaderFormat>(&mut self, header: H) {
        self.response.set_header(header);
    }

    pub fn set_json_content_type(&mut self) {
        self.set_header(ContentType(Mime(TopLevel::Application, SubLevel::Json, vec![])));
    }

    pub fn set_content_type(&mut self, mime: Mime) {
        self.set_header(ContentType(mime));
    }

    pub fn error<T: Error>(self, error: T) -> ClientResult<'a> {
        Err(box error as Box<Error>)
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
        let absolute_path = match os::make_absolute(path) {
            Ok(path) => path,
            Err(err) => {
                return Err(box FileError(err) as Box<Error>);
            }
        };

        match self.response.push_file(&absolute_path) {
            Ok(()) => Ok(self),
            Err(err) => {
                return Err(box FileError(err) as Box<Error>);
            }
        } 
    }

    pub fn empty(self) -> ClientResult<'a> {
        Ok(self)
    }

    pub fn redirect(mut self, to: &str) -> ClientResult<'a> {
        self.set_status(StatusCode::Found);
        self.set_header(Location(to.to_string()));

        Ok(self)
    }

    pub fn permanent_redirect(mut self, to: &str) -> ClientResult<'a> {
        self.set_status(StatusCode::MovedPermanently);
        self.set_header(Location(to.to_string()));

        Ok(self)
    }

    pub fn move_response(self) -> Response {
        self.response
    }

    pub fn ext(&self) -> &TypeMap {
        &self.ext
    }

    pub fn ext_mut(&mut self) -> &mut TypeMap {
        &mut self.ext
    }
    
}

impl<'a> Extensible for Client<'a> {
    fn ext(&self) -> &::typemap::TypeMap { &self.ext }
    fn ext_mut(&mut self) -> &mut ::typemap::TypeMap { &mut self.ext }
}