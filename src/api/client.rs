
use api::Endpoint;
use request::Request;
use response::Response;

use serialize::json::{Json};

use anymap::AnyMap;
use hyper::status;
use hyper::mime;
use hyper::header::Header;
use hyper::header::common::{ContentType, Location};

pub struct Client<'a> {
    pub endpoint: &'a Endpoint,
    pub request: &'a Request,
    pub ext: AnyMap,
    pub response: Response
}

impl<'a> Client<'a> {

    pub fn new(endpoint: &'a Endpoint, request: &'a Request) -> Client<'a> {
        Client {
            endpoint: endpoint,
            request: request,
            ext: AnyMap::new(),
            response: Response::new(status::Ok)
        }
    }

    pub fn set_status(&mut self, status: status::StatusCode) {
        self.response.status = status;
    }

    pub fn set_header<H: Header>(&mut self, header: H) {
        self.response.headers.set(header);
    }

    pub fn set_json_content_type(&mut self) {
        let application_json: mime::Mime = from_str("application/json").unwrap();
        self.set_header(ContentType(application_json));
    }

    pub fn json(mut self, result: &Json) -> Client<'a> {
        self.set_json_content_type();
        self.response.push_string(result.to_string());

        self
    }

    pub fn text(mut self, result: String) -> Client<'a> {
        self.response.push_string(result);

        self
    }

    pub fn redirect(mut self, to: &str) -> Client<'a> {
        self.set_status(status::Found);
        self.set_header(Location(to.to_string()));

        self
    }

    pub fn permanent_redirect(mut self, to: &str) -> Client<'a> {
        self.set_status(status::MovedPermanently);
        self.set_header(Location(to.to_string()));

        self
    }

    pub fn move_response(self) -> Response {
        self.response
    }
    
}