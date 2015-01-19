use serialize::json::{Object};

use valico::Builder as ValicoBuilder;

use server::method::{Method};
use backend::{Request, Response};
use errors::{NotMatchError, Error, ValidationError};
use backend::{HandleResult, HandleSuccessResult};
use framework::path::{Path};
use framework::{
    ApiHandler, Client, CallInfo, Callback
};

pub type EndpointHandler = Box<for<'a> Fn(Client<'a>, &Object) -> HandleResult<Client<'a>> + 'static + Sync>;

pub enum EndpointHandlerPresent {
    HandlerPresent
}

pub type EndpointBuilder = Fn(&mut Endpoint) -> EndpointHandlerPresent + 'static;

pub struct Endpoint {
    pub method: Method,
    pub path: Path,
    pub desc: Option<String>,
    pub coercer: Option<ValicoBuilder>,
    handler: Option<EndpointHandler>,
}

unsafe impl Send for Endpoint {}

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

    pub fn build<F>(method: Method, path: &str, builder: F) -> Endpoint 
    where F: Fn(&mut Endpoint) -> EndpointHandlerPresent {
        let mut endpoint = Endpoint::new(method, path);
        builder(&mut endpoint);

        endpoint
    }

    pub fn desc(&mut self, desc: &str) {
        self.desc = Some(desc.to_string());
    }

    pub fn params<F>(&mut self, builder: F) where F: Fn(&mut ValicoBuilder) + 'static {
        self.coercer = Some(ValicoBuilder::build(builder));
    }

    pub fn handle<F>(&mut self, handler: F) -> EndpointHandlerPresent
    where F: for<'a> Fn(Client<'a>, &Object) -> HandleResult<Client<'a>> + Sync+Send {
        self.handler = Some(Box::new(handler));
        EndpointHandlerPresent::HandlerPresent
    }

    fn validate(&self, params: &mut Object) -> HandleResult<()> {
        // Validate namespace params with valico
        if self.coercer.is_some() {
            // validate and coerce params
            let coercer = self.coercer.as_ref().unwrap();
            match coercer.process(params) {
                Ok(()) => Ok(()),
                Err(err) => return Err(Box::new(ValidationError{ reason: err }) as Box<Error>)
            }   
        } else {
            Ok(())
        }
    }

    pub fn call_decode(&self, params: &mut Object, req: &mut Request, info: &mut CallInfo) -> HandleResult<Response> {
        
        let mut client = Client::new(info.app, self, req, &info.media);

        try!(Endpoint::call_callbacks(&info.before, &mut client, params));
        try!(Endpoint::call_callbacks(&info.before_validation, &mut client, params));
        try!(self.validate(params));
        try!(Endpoint::call_callbacks(&info.after_validation, &mut client, params));

        let handler = self.handler.as_ref();
        let mut client = try!((handler.unwrap())(client, params));
            
        try!(Endpoint::call_callbacks(&info.after, &mut client, params));

        Ok(client.move_response())
    }

    fn call_callbacks(cbs: &Vec<Callback>, client: &mut Client, params: &mut Object) -> HandleSuccessResult {
        for cb in cbs.iter() {
            try!((*cb)(client, params));
        }

        Ok(())
    }

}

impl ApiHandler for Endpoint {
    fn api_call(&self, rest_path: &str, params: &mut Object, req: &mut Request, info: &mut CallInfo) -> HandleResult<Response> {

        // Method guard
        if req.method() != &self.method {
            return Err(Box::new(NotMatchError) as Box<Error>)
        }

        match self.path.is_match(rest_path) {
            Some(captures) =>  {
                self.path.apply_captures(params, captures);
                self.call_decode(params, req, info)
            },
            None => Err(Box::new(NotMatchError) as Box<Error>)
        }

    }
}