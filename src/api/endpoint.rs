use serialize::json;
use serialize::json::{Json, JsonObject};
use serialize::json::ToJson;

use anymap::AnyMap;
use hyper::method::{Method};
use hyper::status;
use hyper::mime;
use hyper::header::Header;
use hyper::header::common::{ContentType, Location};
use valico::Builder as ValicoBuilder;
use query;

use request::Request;
use response::Response;
use path::{Path};
use middleware::{HandleResult, NotMatchError, Error};
use api::{
    ApiHandler, QueryStringDecodeError, ValidationError, 
    BodyDecodeError, ValicoBuildHandler
};

pub type EndpointHandler = fn<'a>(EndpointInstance<'a>, &Json) -> EndpointInstance<'a>;

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

    pub fn process<'a>(&'a self, params: &mut JsonObject, req: &'a mut Request) -> EndpointInstance<'a> {
        let ref handler = self.handler.unwrap();

        let endpoint_response = EndpointInstance::new(self, req);

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

    pub fn json(mut self, result: &Json) -> EndpointInstance<'a> {
        self.set_json_content_type();
        self.response.push_string(result.to_string());

        self
    }

    pub fn text(mut self, result: String) -> EndpointInstance<'a> {
        self.response.push_string(result);

        self
    }

    pub fn redirect(mut self, to: &str) -> EndpointInstance<'a> {
        self.set_status(status::Found);
        self.set_header(Location(to.to_string()));

        self
    }

    pub fn permanent_redirect(mut self, to: &str) -> EndpointInstance<'a> {
        self.set_status(status::MovedPermanently);
        self.set_header(Location(to.to_string()));

        self
    }

    pub fn move_response(self) -> Response {
        self.response
    }
    
}