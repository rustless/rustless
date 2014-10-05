use collections::treemap::TreeMap;
use serialize::json;
use serialize::json::{Json, JsonObject};
use serialize::json::ToJson;
use serialize::Decodable;
use std::str;

use anymap::AnyMap;
use hyper::method::{Method};
use hyper::status;
use hyper::header;
use valico::Builder as ValicoBuilder;
use query;

use request::Request;
use response::Response;
use path::{Path};
use middleware::{Handler, HandleResult, SimpleError, NotMatchError, Error, ErrorRefExt};
use api::{ApiHandler};

pub type EndpointHandler = fn<'a>(EndpointInstance<'a>, &Json) -> EndpointInstance<'a>;
pub type ValicoBuildHandler<'a> = |&mut ValicoBuilder|:'a;

#[deriving(Show)]
pub struct QueryStringDecodeError;

impl Error for QueryStringDecodeError {
    fn name(&self) -> &'static str {
        return "QueryStringDecodeError";
    }
}

#[deriving(Show)]
pub struct ValidationError {
    reason: JsonObject
}

impl Error for ValidationError {
    fn name(&self) -> &'static str {
        return "ValidationError";
    }
}

#[deriving(Show)]
pub struct BodyDecodeError {
    reason: String
}

impl BodyDecodeError {
    pub fn new(reason: String) -> BodyDecodeError {
        return BodyDecodeError {
            reason: reason
        }
    }
}

impl Error for BodyDecodeError {
    fn name(&self) -> &'static str {
        return "BodyDecodeError";
    }

    fn description(&self) -> Option<&str> {
        return Some(self.reason.as_slice())
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

    pub fn new(method: Method, path: &str, desc: &str, params: ValicoBuildHandler, handler: EndpointHandler) -> Endpoint {
        Endpoint {
            desc: desc.to_string(),
            method: method,
            path: Path::parse(path, true).unwrap(),
            coercer: ValicoBuilder::build(params),
            handler: handler
        }
    }

    pub fn process<'a>(&'a self, params: &mut JsonObject, req: &'a mut Request) -> EndpointInstance<'a> {
        let ref handler = self.handler;

        let mut endpoint_response = EndpointInstance::new(self, req);

        // fixme not efficient
        (*handler)(endpoint_response, &params.to_json())
    }

    pub fn call_decode(&self, params: &mut JsonObject, req: &mut Request) -> HandleResult<Response> {
        
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

        // extend params with json-encoded body params if any
        if req.is_json_body() {
            let maybe_body = req.read_to_end();
        
            let utf8_string_body = {
                match maybe_body {
                    Ok(body) => {
                        match String::from_utf8(body) {
                            Ok(e) => e,
                            Err(_) => return Err(BodyDecodeError::new("Invalid UTF-8 sequence".to_string()).abstract()),
                        }
                    },
                    Err(err) => return Err(BodyDecodeError::new(format!("{}", err)).abstract())
                }
            };

            if (utf8_string_body.len() > 0) {
              let maybe_json_body = json::from_str(utf8_string_body.as_slice());
                match maybe_json_body {
                    Ok(json_body) => {
                        for (key, value) in json_body.as_object().unwrap().iter() {
                            if !params.contains_key(key) {
                                params.insert(key.to_string(), value.clone());
                            }
                        }
                    },
                    Err(err) => return Err(BodyDecodeError::new(format!("{}", err)).abstract())
                }  
            }
        }

        // validate and coerce params
        match self.coercer.process(params) {
            Ok(()) => (),
            Err(err) => return Err(ValidationError{ reason: err }.abstract())
        }

        return Ok(self.process(params, req).move_response())
    }

}

impl ApiHandler for Endpoint {
    fn call(&self, rest_path: &str, params: &mut JsonObject, req: &mut Request) -> HandleResult<Response> {

        match self.path.is_match(rest_path) {
            Some(captures) =>  {
                self.path.apply_captures(params, captures);
                self.call_decode(params, req)
            },
            None => Err(NotMatchError.abstract())
        }

    }
}

pub struct EndpointInstance<'a> {
    pub endpoint: &'a Endpoint,
    pub request: &'a Request,
    pub ext: AnyMap,
    pub response: Response
}

impl<'a> EndpointInstance<'a> {

    pub fn new(endpoint: &'a Endpoint, request: &'a Request) -> EndpointInstance<'a> {
        EndpointInstance {
            endpoint: endpoint,
            request: request,
            ext: AnyMap::new(),
            response: Response::new(status::Ok)
        }
    }

    pub fn redirect(mut self, to: &str) -> EndpointInstance<'a> {
        self.response.status = status::Found;
        self.response.headers.set(header::common::Location(to.to_string()));

        self
    }

    pub fn permanent_redirect(mut self, to: &str) -> EndpointInstance<'a> {
        self.response.status = status::MovedPermanently;
        self.response.headers.set(header::common::Location(to.to_string()));

        self
    }

    pub fn move_response(self) -> Response {
        self.response
    }
    
}