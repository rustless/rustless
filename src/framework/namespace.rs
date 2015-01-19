use serialize::json::{Object};

use valico::Builder as ValicoBuilder;

use backend::{Request, Response};
use errors::{NotMatchError, ValidationError, Error};
use backend::{HandleResult};

use framework::path::{Path};
use framework::nesting::{self, Nesting, Node};
use framework::{
    ApiHandler, Callbacks, ApiHandlers, CallInfo
};

pub struct Namespace {
    handlers: ApiHandlers,
    path: Path,
    coercer: Option<ValicoBuilder>,
    before: Callbacks,
    before_validation: Callbacks,
    after_validation: Callbacks,
    after: Callbacks
}

impl_nesting!(Namespace);

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

    pub fn params<F>(&mut self, builder: F) where F: Fn(&mut ValicoBuilder) {
        self.coercer = Some(ValicoBuilder::build(builder));
    }

    pub fn build<F>(path: &str, builder: F) -> Namespace where F: Fn(&mut Namespace) {
        let mut namespace = Namespace::new(path);
        builder(&mut namespace);

        return namespace;
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
}

impl ApiHandler for Namespace {
    fn api_call<'a>(&'a self, rest_path: &str, params: &mut Object, req: &mut Request, info: &mut CallInfo<'a>) -> HandleResult<Response> {

        let rest_path: &str = match self.path.is_match(rest_path) {
            Some(captures) =>  {
                let captured_length = captures.at(0).map_or(0, |c| c.len());
                self.path.apply_captures(params, captures);
                rest_path.slice_from(captured_length)
            },
            None => return Err(Box::new(NotMatchError) as Box<Error>)
        };

        try!(self.validate(params));

        self.push_node(info);
        self.call_handlers(rest_path, params, req, info)
    }
}