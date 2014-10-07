use serialize::json::{JsonObject};

use hyper::method::{Get, Post, Put, Delete};
use valico::Builder as ValicoBuilder;

use request::Request;
use response::Response;
use path::{Path};
use middleware::{Handler, HandleResult, HandleSuccessResult, NotMatchError, Error, ErrorRefExt};

use api::{ApiHandler, Client, Endpoint, EndpointBuilder, ValidationError, ValicoBuildHandler};

pub type ApiHandlers = Vec<Box<ApiHandler + Send + Sync>>;
pub type Callback = fn<'a>(&'a mut Client) -> HandleSuccessResult;

pub struct CallInfo {
    pub before: Vec<Callback>,
    pub before_validation: Vec<Callback>,
    pub after_validation: Vec<Callback>,
    pub after: Vec<Callback>
}

impl CallInfo {
    pub fn new() -> CallInfo {
        CallInfo {
            before: vec![],
            before_validation: vec![],
            after_validation: vec![],
            after: vec![]
        }
    }
}

pub trait NS {

    fn get_handlers<'a>(&'a self) -> &'a ApiHandlers;
    fn get_handlers_mut<'a>(&'a mut self) -> &'a mut ApiHandlers;

    fn get_before<'a>(&'a self) -> &'a Vec<Callback>;
    fn get_before_mut<'a>(&'a mut self) -> &'a mut Vec<Callback>;

    fn get_before_validation<'a>(&'a self) -> &'a Vec<Callback>;
    fn get_before_validation_mut<'a>(&'a mut self) -> &'a mut Vec<Callback>;

    fn get_after_validation<'a>(&'a self) -> &'a Vec<Callback>;
    fn get_after_validation_mut<'a>(&'a mut self) -> &'a mut Vec<Callback>;

    fn get_after<'a>(&'a self) -> &'a Vec<Callback>;
    fn get_after_mut<'a>(&'a mut self) -> &'a mut Vec<Callback>;

    fn push_callbacks(&self, info: &mut CallInfo) {
        for cb in self.get_before().iter() { info.before.push(*cb); }
        for cb in self.get_before_validation().iter() { info.before_validation.push(*cb); }
        for cb in self.get_after_validation().iter() { info.after_validation.push(*cb); }
        for cb in self.get_after().iter() { info.after.push(*cb); }
    }

    fn mount(&mut self, edp: Box<ApiHandler + Send + Sync>) {
        self.get_handlers_mut().push(edp)
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

    fn before(&mut self, callback: Callback) { self.get_before_mut().push(callback); }
    fn before_validation(&mut self, callback: Callback) { self.get_before_validation_mut().push(callback); }
    fn after(&mut self, callback: Callback) { self.get_after_mut().push(callback); }
    fn after_validation(&mut self, callback: Callback) { self.get_after_validation_mut().push(callback); }

    fn call_handlers(&self, rest_path: &str, params: &mut JsonObject, req: &mut Request, info: &mut CallInfo) -> HandleResult<Response> {
        for handler in self.get_handlers().iter() {
            match handler.api_call(rest_path, params, req, info) {
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
    coercer: Option<ValicoBuilder>,
    before: Vec<Callback>,
    before_validation: Vec<Callback>,
    after_validation: Vec<Callback>,
    after: Vec<Callback>
}

impl NS for Namespace {
    fn get_handlers<'a>(&'a self) -> &'a ApiHandlers { &self.handlers }
    fn get_handlers_mut<'a>(&'a mut self) -> &'a mut ApiHandlers { &mut self.handlers }

    fn get_before<'a>(&'a self) -> &'a Vec<Callback> { &self.before }
    fn get_before_mut<'a>(&'a mut self) -> &'a mut Vec<Callback> { &mut self.before }

    fn get_before_validation<'a>(&'a self) -> &'a Vec<Callback> { &self.before_validation }
    fn get_before_validation_mut<'a>(&'a mut self) -> &'a mut Vec<Callback> { &mut self.before_validation }

    fn get_after_validation<'a>(&'a self) -> &'a Vec<Callback> { &self.after_validation }
    fn get_after_validation_mut<'a>(&'a mut self) -> &'a mut Vec<Callback> { &mut self.after_validation }

    fn get_after<'a>(&'a self) -> &'a Vec<Callback> { &self.after }
    fn get_after_mut<'a>(&'a mut self) -> &'a mut Vec<Callback> { &mut self.after }
}

impl Namespace {
    
    pub fn new(path: &str) -> Namespace {
        Namespace {
            handlers: vec![],
            path: Path::parse(path, false).unwrap(),
            coercer: None,
            before: vec![],
            before_validation: vec![],
            after_validation: vec![],
            after: vec![]
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
    fn api_call(&self, rest_path: &str, params: &mut JsonObject, req: &mut Request, info: &mut CallInfo) -> HandleResult<Response> {

        let rest_path: &str = match self.path.is_match(rest_path) {
            Some(captures) =>  {
                let captured_length = captures.at(0).len();
                self.path.apply_captures(params, captures);
                rest_path.slice_from(captured_length)
            },
            None => return Err(NotMatchError.abstract())
        };

        try!(self.validate(params));

        self.push_callbacks(info);
        self.call_handlers(rest_path, params, req, info)
    }
}