use serialize::json::{JsonObject};

use valico::Builder as ValicoBuilder;

use server_backend::method::{Method};
use server::{Request, Response};
use errors::{NotMatchError, Error, ValidationError};
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
                Err(err) => return Err(box ValidationError{ reason: err } as Box<Error>)
            }   
        } else {
            Ok(())
        }
    }

    pub fn call_decode(&self, params: &mut JsonObject, req: &mut Request, info: &mut CallInfo) -> HandleResult<Response> {
        
        let mut client = Client::new(info.app, self, req, &info.media);

        try!(Endpoint::call_callbacks(&info.before, &mut client, params));
        try!(Endpoint::call_callbacks(&info.before_validation, &mut client, params));
        try!(self.validate(params));
        try!(Endpoint::call_callbacks(&info.after_validation, &mut client, params));

        let ref handler = self.handler.unwrap();
        let mut client = try!((*handler)(client, params));
            
        try!(Endpoint::call_callbacks(&info.after, &mut client, params));

        Ok(client.move_response())
    }

    fn call_callbacks(cbs: &Vec<Callback>, client: &mut Client, params: &mut JsonObject) -> HandleSuccessResult {
        for cb in cbs.iter() {
            try!((*cb)(client, params));
        }

        Ok(())
    }

}

impl ApiHandler for Endpoint {
    fn api_call(&self, rest_path: &str, params: &mut JsonObject, req: &mut Request, info: &mut CallInfo) -> HandleResult<Response> {

        // Method guard
        if req.method() != &self.method {
            return Err(box NotMatchError as Box<Error>)
        }

        match self.path.is_match(rest_path) {
            Some(captures) =>  {
                self.path.apply_captures(params, captures);
                self.call_decode(params, req, info)
            },
            None => Err(box NotMatchError as Box<Error>)
        }

    }
}