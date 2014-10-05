use serialize::Decodable;
use hyper::method::{Method};
use hyper::status;

use collections::treemap::TreeMap;
use serialize::json;
use serialize::json::{Json, JsonObject};
use serialize::json::ToJson;

use request::Request;
use response::Response;
use path::{Path};
use middleware::{Handler, HandleResult, SimpleError, NotMatchError, Error, ErrorRefExt};

use api::{ApiHandler};

pub type ApiHandlers = Vec<Box<ApiHandler + Send + Sync>>;

pub struct Namespace {
    handlers: ApiHandlers,
    path: Path  
}

pub trait NamespaceBehavior {

    fn handlers<'a>(&'a self) -> &'a ApiHandlers;
    fn handlers_mut<'a>(&'a mut self) -> &'a mut ApiHandlers;

    fn mount(&mut self, edp: Box<ApiHandler + Send + Sync>) {
        self.handlers_mut().push(edp)
    }

    fn call_handlers(&self, rest_path: &str, params: &mut JsonObject, req: &mut Request) -> HandleResult<Response> {
        for handler in self.handlers().iter() {
            match handler.call(rest_path, params, req) {
                Ok(response) => return Ok(response),
                Err(err) => {
                    match err.downcast::<NotMatchError>() {
                        Some(_) => (),
                        None => return Err(err),
                    }
                }
            };
        }

        Err(NotMatchError.abstract())
    }

}

impl NamespaceBehavior for Namespace {
    fn handlers<'a>(&'a self) -> &'a ApiHandlers { &self.handlers }
    fn handlers_mut<'a>(&'a mut self) -> &'a mut ApiHandlers { &mut self.handlers }
}

impl Namespace {
    pub fn new(path: &'static str) -> Namespace {
        Namespace {
            handlers: vec![],
            path: Path::parse(path, false).unwrap()
        }
    }
}

impl ApiHandler for Namespace {
    fn call(&self, rest_path: &str, params: &mut JsonObject, req: &mut Request) -> HandleResult<Response> {

        let rest_path: &str = match self.path.is_match(rest_path) {
            Some(captures) =>  {
                let captured_length = captures.at(0).len();
                self.path.apply_captures(params, captures);
                rest_path.slice_from(captured_length)
            },
            None => return Err(NotMatchError.abstract())
        };

        self.call_handlers(rest_path, params, req)
    }
}