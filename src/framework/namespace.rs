use serialize::json;
use valico::json_dsl;
use valico::json_schema;

use backend;
use errors;

use framework;
use framework::path;
use framework::nesting::{self, Nesting, Node};

use batteries::json_schema as json_schema_battery;

pub struct Namespace {
    pub handlers: framework::ApiHandlers,
    pub path: path::Path,
    pub coercer: Option<json_dsl::Builder>,
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

    pub fn params<F>(&mut self, builder: F) where F: FnOnce(&mut json_dsl::Builder) {
        self.coercer = Some(json_dsl::Builder::build(builder));
    }

    pub fn build<F>(path: &str, builder: F) -> Namespace where F: FnOnce(&mut Namespace) {
        let mut namespace = Namespace::new(path);
        builder(&mut namespace);

        return namespace;
    }

    fn validate(&self, params: &mut json::Json, scope: Option<&json_schema::Scope>) -> backend::HandleResult<()> {
        // Validate namespace params with valico
        if self.coercer.is_some() {
            // validate and coerce params
            let coercer = self.coercer.as_ref().unwrap();
            let state = coercer.process(params, &scope);

            if state.is_strictly_valid() {
                Ok(())
            } else {
                if state.missing.len() > 0 {
                    warn!("There are some missing JSON schemes: {:?}", state.missing);
                }
                Err(error_response!(errors::Validation{ reason: state.errors }))
            }
        } else {
            Ok(())
        }
    }
}

impl framework::ApiHandler for Namespace {
    fn api_call<'a, 'r>(&'a self, rest_path: &str, params: &mut json::Json, req: &'r mut (backend::Request + 'r), 
                    info: &mut framework::CallInfo<'a>) -> backend::HandleResult<backend::Response> {

        let rest_path: &str = match self.path.is_match(rest_path) {
            Some(captures) =>  {
                let captured_length = captures.at(0).map_or(0, |c| c.len());
                self.path.apply_captures(params, captures);
                path::normalize(&rest_path[(captured_length)..])
            },
            None => return Err(error_response!(errors::NotMatch))
        };

        try!(self.validate(params, info.app.ext.get::<json_schema_battery::JsonSchemaScope>()));

        self.push_node(info);
        self.call_handlers(rest_path, params, req, info)
    }
}