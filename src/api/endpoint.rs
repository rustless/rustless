use serialize::Decodable;
use hyper::method::{Method};
use hyper::status;

use collections::treemap::TreeMap;
use serialize::json;
use serialize::json::{Json, JsonObject};
use serialize::json::ToJson;
use valico::Builder as ValicoBuilder;

use request::Request;
use response::Response;
use path::{Path};
use middleware::{Handler, HandleResult, SimpleError, NotMatchError, Error, ErrorRefExt};

use api::{ApiHandler};

pub type EndpointHandler = |Json|:'static + Send + Sync -> String;
pub type ValicoBuildHandler<'a> = |&mut ValicoBuilder|:'a;

#[deriving(Send)]
pub struct Endpoint {
    pub desc: String,
    pub path: Path,
    pub method: Method,
    pub coercer: ValicoBuilder,
    handler: EndpointHandler,
}

impl Endpoint {

    pub fn process(self, params_body: &str) -> String {
        // let params = Endpoint::decode(params_body);
        // let handler = self.handler;
        // handler(params)
        "".to_string()
    }

    pub fn new(method: Method, path: &str, desc: &str, params: ValicoBuildHandler, handler: EndpointHandler) -> Endpoint {
        Endpoint {
            desc: desc.to_string(),
            method: method,
            path: Path::parse(path, true).unwrap(),
            coercer: ValicoBuilder::build(params),
            handler: handler
        }
    }
}

impl ApiHandler for Endpoint {
    fn call(&self, rest_path: &str, params: &mut JsonObject, req: &mut Request) -> HandleResult<Response> {
        match self.path.is_match(rest_path) {
            Some(captures) =>  {
                return Ok(Response::from_string(status::Ok, "MATCH ENDPOINT!!".to_string()))
            },
            None => return Err(NotMatchError.abstract())
        };
    }
}