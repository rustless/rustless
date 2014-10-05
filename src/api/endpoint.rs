use collections::treemap::TreeMap;
use serialize::json;
use serialize::json::{Json, JsonObject};
use serialize::json::ToJson;
use serialize::Decodable;

use hyper::method::{Method};
use hyper::status;
use valico::Builder as ValicoBuilder;
use query;

use request::Request;
use response::Response;
use path::{Path};
use middleware::{Handler, HandleResult, SimpleError, NotMatchError, Error, ErrorRefExt};
use api::{ApiHandler};

pub type EndpointHandler = fn(&Json) -> String;
pub type ValicoBuildHandler<'a> = |&mut ValicoBuilder|:'a;

#[deriving(Show)]
pub struct QueryStringDecodeError;

impl Error for QueryStringDecodeError {
    fn name(&self) -> &'static str {
        return "QueryStringDecodeError";
    }
}

#[deriving(Send)]
pub struct Endpoint {
    pub desc: String,
    pub path: Path,
    pub method: Method,
    pub coercer: ValicoBuilder,
    handler: EndpointHandler,
}

impl Endpoint {

    pub fn process(&self, params: &JsonObject) -> String {
        // let params = Endpoint::decode(params_body);
        let ref handler = self.handler;
        // fixme not efficient
        (*handler)(&params.to_json())
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
                self.path.apply_captures(params, captures);

                // extend params with query-string params if any
                if req.url.query.is_some() {
                    let mut maybe_query_params = query::parse(req.url.query.as_ref().unwrap().as_slice());
                    match maybe_query_params {
                        Ok(query_params) => {
                            for (key, value) in query_params.as_object().unwrap().iter() {
                                if !params.contains_key(key) {
                                    params.insert(key.to_string(), value.clone());
                                }
                            }
                        }, 
                        Err(err) => {
                            return Err(QueryStringDecodeError.abstract());
                        }
                    }
                }

                return Ok(Response::from_string(status::Ok, self.process(params)))
            },
            None => return Err(NotMatchError.abstract())
        };

    }
}