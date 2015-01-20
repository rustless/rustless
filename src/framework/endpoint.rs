use serialize::json;

use valico;

use server::method;
use backend;
use errors;
use framework;
use framework::path;

pub type EndpointHandler = Box<for<'a> Fn(framework::Client<'a>, &json::Object) -> backend::HandleResult<framework::Client<'a>> + 'static + Sync>;

#[allow(missing_copy_implementations)]
pub enum EndpointHandlerPresent {
    HandlerPresent
}

pub type EndpointBuilder = Fn(&mut Endpoint) -> EndpointHandlerPresent + 'static;

pub struct Endpoint {
    pub method: method::Method,
    pub path: path::Path,
    pub desc: Option<String>,
    pub coercer: Option<valico::Builder>,
    handler: Option<EndpointHandler>,
}

unsafe impl Send for Endpoint {}

impl Endpoint {

    pub fn new(method: method::Method, path: &str) -> Endpoint {
        Endpoint {
            method: method,
            path: path::Path::parse(path, true).unwrap(),
            desc: None,
            coercer: None,
            handler: None
        }
    }

    pub fn build<F>(method: method::Method, path: &str, builder: F) -> Endpoint 
    where F: Fn(&mut Endpoint) -> EndpointHandlerPresent {
        let mut endpoint = Endpoint::new(method, path);
        builder(&mut endpoint);

        endpoint
    }

    pub fn desc(&mut self, desc: &str) {
        self.desc = Some(desc.to_string());
    }

    pub fn params<F>(&mut self, builder: F) where F: Fn(&mut valico::Builder) + 'static {
        self.coercer = Some(valico::Builder::build(builder));
    }

    pub fn handle<F>(&mut self, handler: F) -> EndpointHandlerPresent
    where F: for<'a> Fn(framework::Client<'a>, &json::Object) -> backend::HandleResult<framework::Client<'a>> + Sync+Send {
        self.handler = Some(Box::new(handler));
        EndpointHandlerPresent::HandlerPresent
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

    pub fn call_decode(&self, params: &mut json::Object, req: &mut backend::Request, 
                       info: &mut framework::CallInfo) -> backend::HandleResult<backend::Response> {
        
        let mut client = framework::Client::new(info.app, self, req, &info.media);

        for parent in info.parents.iter() {
            try!(Endpoint::call_callbacks(parent.get_before(), &mut client, params));
        }

        for parent in info.parents.iter() {
            try!(Endpoint::call_callbacks(parent.get_before_validation(), &mut client, params));
        }

        try!(self.validate(params));

        for parent in info.parents.iter() {
            try!(Endpoint::call_callbacks(parent.get_after_validation(), &mut client, params));
        }

        let handler = self.handler.as_ref();
        let mut client = try!((handler.unwrap())(client, params));

        for parent in info.parents.iter() {
            try!(Endpoint::call_callbacks(parent.get_after(), &mut client, params));
        }

        Ok(client.move_response())
    }

    fn call_callbacks(cbs: &Vec<framework::Callback>, client: &mut framework::Client, params: &mut json::Object) 
    -> backend::HandleSuccessResult {
        for cb in cbs.iter() {
            try!(cb(client, params));
        }

        Ok(())
    }

}

impl framework::ApiHandler for Endpoint {
    fn api_call(&self, rest_path: &str, params: &mut json::Object, req: &mut backend::Request, 
                info: &mut framework::CallInfo) -> backend::HandleResult<backend::Response> {

        // method::Method guard
        if req.method() != &self.method {
            return Err(Box::new(errors::NotMatch) as Box<errors::Error>)
        }

        match self.path.is_match(rest_path) {
            Some(captures) =>  {
                self.path.apply_captures(params, captures);
                self.call_decode(params, req, info)
            },
            None => Err(Box::new(errors::NotMatch) as Box<errors::Error>)
        }

    }
}