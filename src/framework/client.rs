use serialize::json;
use typemap;
use std::env;
use std::path::Path;

use backend;
use errors::{self, Error};
use framework::app;
use framework::endpoint;
use framework::media;
use server::status;
use server::mime;
use server::header;

pub struct Client<'a> {
    pub app: &'a app::Application,
    pub endpoint: &'a endpoint::Endpoint,
    pub request: &'a mut (backend::Request + 'a),
    pub media: &'a media::Media,
    pub ext: typemap::TypeMap,
    pub response: backend::Response
}

pub type ClientResult<'a> = backend::HandleResult<Client<'a>>;

impl<'a> Client<'a> {

    pub fn new<'r>(app: &'a app::Application, endpoint: &'a endpoint::Endpoint,
               request: &'a mut (backend::Request + 'r), media: &'a media::Media) -> Client<'a> {
        Client {
            app: app,
            endpoint: endpoint,
            request: request,
            media: media,
            ext: typemap::TypeMap::new(),
            response: backend::Response::new(status::StatusCode::Ok)
        }
    }

    //
    // Work with status
    //

    pub fn status(&mut self) -> status::StatusCode {
        self.response.status
    }

    pub fn set_status(&mut self, status: status::StatusCode) {
        self.response.status = status;
    }

    pub fn unauthorized(&mut self) {
        self.response.status = status::StatusCode::Unauthorized;
    }

    pub fn forbidden(&mut self) {
        self.response.status = status::StatusCode::Forbidden;
    }

    pub fn not_found(&mut self) {
        self.response.status = status::StatusCode::NotFound;
    }

    pub fn internal_server_error(&mut self) {
        self.response.status = status::StatusCode::InternalServerError;
    }

    pub fn not_implemented(&mut self) {
        self.response.status = status::StatusCode::NotImplemented;
    }

    //

    pub fn set_header<H: header::Header + header::HeaderFormat>(&mut self, header: H) {
        self.response.set_header(header);
    }

    pub fn set_json_content_type(&mut self) {
        self.set_header(header::ContentType(
            mime::Mime(mime::TopLevel::Application, mime::SubLevel::Json, vec![])
        ));
    }

    pub fn set_content_type(&mut self, mime: mime::Mime) {
        self.set_header(header::ContentType(mime));
    }

    pub fn error<T: Error>(self, error: T) -> ClientResult<'a> {
        Err(error_response!(error))
    }

    pub fn json(mut self, result: &json::Json) -> ClientResult<'a> {
        self.set_json_content_type();
        self.response.push_string(result.to_string());

        Ok(self)
    }

    pub fn text(mut self, result: String) -> ClientResult<'a> {
        self.response.push_string(result);

        Ok(self)
    }

    pub fn file(mut self, path: &Path) -> ClientResult<'a> {
        let absolute_path = match env::current_dir().map(|curr_dir| curr_dir.join(path)) {
            Ok(path) => path,
            Err(err) => {
                return Err(error_response!(errors::File(err)));
            }
        };

        match self.response.push_file(&absolute_path) {
            Ok(()) => Ok(self),
            Err(err) => {
                return Err(error_response!(errors::File(err)));
            }
        }
    }

    pub fn empty(self) -> ClientResult<'a> {
        Ok(self)
    }

    pub fn redirect(mut self, to: &str) -> ClientResult<'a> {
        self.set_status(status::StatusCode::Found);
        self.set_header(header::Location(to.to_string()));

        Ok(self)
    }

    pub fn permanent_redirect(mut self, to: &str) -> ClientResult<'a> {
        self.set_status(status::StatusCode::MovedPermanently);
        self.set_header(header::Location(to.to_string()));

        Ok(self)
    }

    pub fn move_response(self) -> backend::Response {
        self.response
    }

    pub fn ext(&self) -> &typemap::TypeMap {
        &self.ext
    }

    pub fn ext_mut(&mut self) -> &mut typemap::TypeMap {
        &mut self.ext
    }

}

impl<'a> ::Extensible for Client<'a> {
    fn ext(&self) -> &::typemap::TypeMap { &self.ext }
    fn ext_mut(&mut self) -> &mut ::typemap::TypeMap { &mut self.ext }
}