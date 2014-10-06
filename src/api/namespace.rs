use serialize::json::{JsonObject};

use hyper::method::{Get, Post, Put, Delete};
use valico::Builder as ValicoBuilder;

use request::Request;
use response::Response;
use path::{Path};
use middleware::{Handler, HandleResult, NotMatchError, Error, ErrorRefExt};

use api::{ApiHandler, Endpoint, EndpointBuilder, ValidationError, ValicoBuildHandler};

pub type ApiHandlers = Vec<Box<ApiHandler + Send + Sync>>;

pub trait NS {

    fn handlers<'a>(&'a self) -> &'a ApiHandlers;
    fn handlers_mut<'a>(&'a mut self) -> &'a mut ApiHandlers;

    fn mount(&mut self, edp: Box<ApiHandler + Send + Sync>) {
        self.handlers_mut().push(edp)
    }

    /* 
     * Namespace aliases
     */

    fn namespace(&mut self, path: &str, builder: |&mut Namespace|) {
        self.mount(box Namespace::build(path, builder));
    }
    fn group(&mut self, path: &str, builder: |&mut Namespace|) {
        self.mount(box Namespace::build(path, builder));
    }
    fn resource(&mut self, path: &str, builder: |&mut Namespace|) {
        self.mount(box Namespace::build(path, builder));
    }
    fn resources(&mut self, path: &str, builder: |&mut Namespace|) {
        self.mount(box Namespace::build(path, builder));
    }
    fn segment(&mut self, path: &str, builder: |&mut Namespace|) {
        self.mount(box Namespace::build(path, builder));
    }

    /* 
     * Endpoints
     */

    fn get(&mut self, path: &str, builder: EndpointBuilder) {
        self.mount(box Endpoint::build(Get, path, builder));
    }    
    fn post(&mut self, path: &str, builder: EndpointBuilder) {
        self.mount(box Endpoint::build(Post, path, builder));
    }    
    fn put(&mut self, path: &str, builder: EndpointBuilder) {
        self.mount(box Endpoint::build(Put, path, builder));
    }    
    fn delete(&mut self, path: &str, builder: EndpointBuilder) {
        self.mount(box Endpoint::build(Delete, path, builder));
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

pub struct Namespace {
    handlers: ApiHandlers,
    path: Path,
    coercer: Option<ValicoBuilder>
}

impl NS for Namespace {
    fn handlers<'a>(&'a self) -> &'a ApiHandlers { &self.handlers }
    fn handlers_mut<'a>(&'a mut self) -> &'a mut ApiHandlers { &mut self.handlers }
}

impl Namespace {
    
    pub fn new(path: &str) -> Namespace {
        Namespace {
            handlers: vec![],
            path: Path::parse(path, false).unwrap(),
            coercer: None
        }
    }

    pub fn params(&mut self, builder: ValicoBuildHandler) {
        self.coercer = Some(ValicoBuilder::build(builder));
    }

    pub fn build(path: &str, builder: |&mut Namespace|) -> Namespace {
        let mut namespace = Namespace::new(path);
        builder(&mut namespace);

        return namespace;
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

        try!(self.validate(params));

        self.call_handlers(rest_path, params, req)
    }
}