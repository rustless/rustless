use serialize::json;
use valico;

use backend;
use errors;

use framework;
use framework::path;
use framework::nesting::{self, Nesting, Node};

pub struct Namespace {
    pub handlers: framework::ApiHandlers,
    pub path: path::Path,
    pub coercer: Option<valico::Builder>,
    before: framework::Callbacks,
    before_validation: framework::Callbacks,
    after_validation: framework::Callbacks,
    after: framework::Callbacks
}

impl_nesting!(Namespace);

impl Namespace {
    
    pub fn new(path: &str) -> Namespace {
        Namespace {
            handlers: vec![],
            path: path::Path::parse(path, false).unwrap(),
            coercer: None,
            before: vec![],
            before_validation: vec![],
            after_validation: vec![],
            after: vec![]
        }
    }

    pub fn params<F>(&mut self, builder: F) where F: Fn(&mut valico::Builder) {
        self.coercer = Some(valico::Builder::build(builder));
    }

    pub fn build<F>(path: &str, builder: F) -> Namespace where F: Fn(&mut Namespace) {
        let mut namespace = Namespace::new(path);
        builder(&mut namespace);

        return namespace;
    }

    fn validate(&self, params: &mut json::Object) -> backend::HandleResult<()> {
        // Validate namespace params with valico
        if self.coercer.is_some() {
            // validate and coerce params
            let coercer = self.coercer.as_ref().unwrap();
            match coercer.process(params) {
                Ok(()) => Ok(()),
                Err(err) => return Err(Box::new(errors::Validation{ reason: err }) as Box<errors::Error>)
            }   
        } else {
            Ok(())
        }
    }
}

impl framework::ApiHandler for Namespace {
    fn api_call<'a>(&'a self, rest_path: &str, params: &mut json::Object, req: &mut backend::Request, 
                    info: &mut framework::CallInfo<'a>) -> backend::HandleResult<backend::Response> {

        let rest_path: &str = match self.path.is_match(rest_path) {
            Some(captures) =>  {
                let captured_length = captures.at(0).map_or(0, |c| c.len());
                self.path.apply_captures(params, captures);
                path::normalize(rest_path.slice_from(captured_length))
            },
            None => return Err(Box::new(errors::NotMatch) as Box<errors::Error>)
        };

        try!(self.validate(params));

        self.push_node(info);
        self.call_handlers(rest_path, params, req, info)
    }
}