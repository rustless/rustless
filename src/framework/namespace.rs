use serialize::json::{JsonObject};

use valico::Builder as ValicoBuilder;

use server::{Request, Response};
use errors::{NotMatchError, ValidationError, Error};
use middleware::{HandleResult};

use framework::path::{Path};
use framework::nesting::Nesting;
use framework::{
    ApiHandler, ValicoBuildHandler,
    Callback, ApiHandlers, CallInfo
};

pub struct Namespace {
    handlers: ApiHandlers,
    path: Path,
    coercer: Option<ValicoBuilder>,
    before: Vec<Callback>,
    before_validation: Vec<Callback>,
    after_validation: Vec<Callback>,
    after: Vec<Callback>
}

impl Nesting for Namespace {
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