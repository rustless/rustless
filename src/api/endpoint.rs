use serialize::json;
use serialize::json::{Json, JsonObject};
use serialize::json::ToJson;

use hyper::method::{Method};
use valico::Builder as ValicoBuilder;
use query;

use request::Request;
use response::Response;
use path::{Path};
use middleware::{HandleResult, NotMatchError, Error};
use api::{
    ApiHandler, QueryStringDecodeError, ValidationError, 
    BodyDecodeError, ValicoBuildHandler, Client
};

pub type EndpointHandler = fn<'a>(Client<'a>, &Json) -> Client<'a>;

pub enum EndpointHandlerPresent {
    HandlerPresent
}

pub type EndpointBuilder = |&mut Endpoint|: 'static -> EndpointHandlerPresent;

#[deriving(Send)]
pub struct Endpoint {
    pub method: Method,
    pub path: Path,
    pub desc: Option<String>,
    pub coercer: Option<ValicoBuilder>,
    handler: Option<EndpointHandler>,
}

impl Endpoint {

    pub fn new(method: Method, path: &str) -> Endpoint {
        Endpoint {
            method: method,
            path: Path::parse(path, true).unwrap(),
            desc: None,
            coercer: None,
            handler: None
        }
    }

    pub fn build(method: Method, path: &str, builder: EndpointBuilder) -> Endpoint {
        let mut endpoint = Endpoint::new(method, path);
        builder(&mut endpoint);

        endpoint
    }

    pub fn desc(&mut self, desc: &str) {
        self.desc = Some(desc.to_string());
    }

    pub fn params(&mut self, builder: ValicoBuildHandler) {
        self.coercer = Some(ValicoBuilder::build(builder));
    }

    pub fn handle(&mut self, handler: EndpointHandler) -> EndpointHandlerPresent {
        self.handler = Some(handler);
        HandlerPresent
    }

    pub fn process<'a>(&'a self, params: &mut JsonObject, req: &'a mut Request) -> Client<'a> {
        let ref handler = self.handler.unwrap();

        let endpoint_response = Client::new(self, req);

        // fixme not efficient
        (*handler)(endpoint_response, &params.to_json())
    }

    fn validate(&self, params: &mut JsonObject) -> HandleResult<()> {
        // Validate namespace params with valico
        if self.coercer.is_some() {
            // validate and coerce params
            let coercer = self.coercer.as_ref().unwrap();
            match coercer.process(params) {
                Ok(()) => Ok(()),
                Err(err) => return Err(ValidationError{ reason: err }.abstract())
            }   
        } else {
            Ok(())
        }
    }

    pub fn call_decode(&self, params: &mut JsonObject, req: &mut Request) -> HandleResult<Response> {
        
        // extend params with query-string params if any
        if req.url.query.is_some() {
            let maybe_query_params = query::parse(req.url.query.as_ref().unwrap().as_slice());
            match maybe_query_params {
                Ok(query_params) => {
                    for (key, value) in query_params.as_object().unwrap().iter() {
                        if !params.contains_key(key) {
                            params.insert(key.to_string(), value.clone());
                        }
                    }
                }, 
                Err(_) => {
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

            if utf8_string_body.len() > 0 {
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

        try!(self.validate(params));

        return Ok(self.process(params, req).move_response())
    }

}

impl ApiHandler for Endpoint {
    fn api_call(&self, rest_path: &str, params: &mut JsonObject, req: &mut Request) -> HandleResult<Response> {

        match self.path.is_match(rest_path) {
            Some(captures) =>  {
                self.path.apply_captures(params, captures);
                self.call_decode(params, req)
            },
            None => Err(NotMatchError.abstract())
        }

    }
}