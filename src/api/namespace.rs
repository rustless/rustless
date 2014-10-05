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

pub struct Namespace {
    handlers: Vec<Box<ApiHandler + Send + Sync>>,
    path: Path  
}

impl Namespace {
    pub fn new(path: &'static str) -> Namespace {
        Namespace {
            handlers: vec![],
            path: Path::parse(path, false).unwrap()
        }
    }

    pub fn mount(&mut self, edp: Box<ApiHandler + Send + Sync>) {
        self.handlers.push(edp)
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

        for handler in self.handlers.iter() {
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