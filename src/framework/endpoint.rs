use serialize::json;
use serialize::json::{JsonObject};

use valico::Builder as ValicoBuilder;
use query;

use server_backend::method::{Method};
use server::{Request, Response};
use errors::{NotMatchError, Error, QueryStringDecodeError, ValidationError, BodyDecodeError};
use middleware::{HandleResult, HandleSuccessResult};
use framework::path::{Path};
use framework::{
    ApiHandler, ValicoBuildHandler, Client, CallInfo, Callback
};

pub type EndpointHandler = fn<'a>(Client<'a>, &JsonObject) -> HandleResult<Client<'a>>;

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

    fn validate(&self, params: &mut JsonObject) -> HandleResult<()> {
        // Validate namespace params with valico
        if self.coercer.is_some() {
            // validate and coerce params
            let coercer = self.coercer.as_ref().unwrap();
            match coercer.process(params) {
                Ok(()) => Ok(()),
                Err(err) => return Err(ValidationError{ reason: err }.erase())
            }   
        } else {
            Ok(())
        }
    }

    pub fn call_decode(&self, params: &mut JsonObject, req: &mut Request, info: &mut CallInfo) -> HandleResult<Response> {
        
        let mut client = Client::new(self, req, &info.media);

        try!(Endpoint::call_callbacks(&info.before, &mut client, params));
        try!(Endpoint::parse_request(client.request, params));
        try!(Endpoint::call_callbacks(&info.before_validation, &mut client, params));
        try!(self.validate(params));
        try!(Endpoint::call_callbacks(&info.after_validation, &mut client, params));

        let ref handler = self.handler.unwrap();
        let mut client = try!((*handler)(client, params));
            
        try!(Endpoint::call_callbacks(&info.after, &mut client, params));

        Ok(client.move_response())
    }

    fn parse_query(query_str: &str, params: &mut JsonObject) -> HandleSuccessResult {
        let maybe_query_params = query::parse(query_str);
        match maybe_query_params {
            Ok(query_params) => {
                for (key, value) in query_params.as_object().unwrap().iter() {
                    if !params.contains_key(key) {
                        params.insert(key.to_string(), value.clone());
                    }
                }
            }, 
            Err(_) => {
                return Err(QueryStringDecodeError.erase());
            }
        }

        Ok(())
    }

    fn parse_json_body(req: &mut Request, params: &mut JsonObject) -> HandleSuccessResult {
        let maybe_body = req.read_to_end();
        
        let utf8_string_body = {
            match maybe_body {
                Ok(body) => {
                    match String::from_utf8(body) {
                        Ok(e) => e,
                        Err(_) => return Err(BodyDecodeError::new("Invalid UTF-8 sequence".to_string()).erase()),
                    }
                },
                Err(err) => return Err(BodyDecodeError::new(format!("{}", err)).erase())
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
                Err(err) => return Err(BodyDecodeError::new(format!("{}", err)).erase())
            }  
        }

        Ok(())
    }

    fn call_callbacks(cbs: &Vec<Callback>, client: &mut Client, params: &mut JsonObject) -> HandleSuccessResult {
        for cb in cbs.iter() {
            try!((*cb)(client, params));
        }

        Ok(())
    }

    fn parse_request(req: &mut Request, params: &mut JsonObject) -> HandleSuccessResult {
        // extend params with query-string params if any
        if req.url().query.is_some() {
            try!(Endpoint::parse_query(req.url().query.as_ref().unwrap().as_slice(), params));   
        }

        // extend params with json-encoded body params if any
        if req.is_json_body() {
            try!(Endpoint::parse_json_body(req, params));
        }

        Ok(())
    }

}

impl ApiHandler for Endpoint {
    fn api_call(&self, rest_path: &str, params: &mut JsonObject, req: &mut Request, info: &mut CallInfo) -> HandleResult<Response> {

        // Method guard
        if req.method() != &self.method {
            return Err(NotMatchError.erase())
        }

        match self.path.is_match(rest_path) {
            Some(captures) =>  {
                self.path.apply_captures(params, captures);
                self.call_decode(params, req, info)
            },
            None => Err(NotMatchError.erase())
        }

    }
}