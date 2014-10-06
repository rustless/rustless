
use collections::treemap::TreeMap;
use serialize::json::{JsonObject};

use valico::Builder as ValicoBuilder;

use request::Request;
use response::Response;
use middleware::{Handler, HandleResult, Error};

pub use self::endpoint::{Endpoint, EndpointBuilder, EndpointInstance};
pub use self::namespace::{Namespace, NamespaceBehavior, ApiHandlers};

mod endpoint;
mod namespace;

pub type ValicoBuildHandler<'a> = |&mut ValicoBuilder|:'a;

#[deriving(Show)]
pub struct QueryStringDecodeError;

impl Error for QueryStringDecodeError {
    fn name(&self) -> &'static str {
        return "QueryStringDecodeError";
    }
}

#[deriving(Show)]
pub struct ValidationError {
    reason: JsonObject
}

impl Error for ValidationError {
    fn name(&self) -> &'static str {
        return "ValidationError";
    }
}

#[deriving(Show)]
pub struct BodyDecodeError {
    reason: String
}

impl BodyDecodeError {
    pub fn new(reason: String) -> BodyDecodeError {
        return BodyDecodeError {
            reason: reason
        }
    }
}

impl Error for BodyDecodeError {
    fn name(&self) -> &'static str {
        return "BodyDecodeError";
    }

    fn description(&self) -> Option<&str> {
        return Some(self.reason.as_slice())
    }
}


pub trait ApiHandler {
    fn call(&self, &str, &mut JsonObject, &mut Request) -> HandleResult<Response>;
}

#[deriving(Send)]
pub struct Api {
    pub version: String,
    handlers: ApiHandlers
}

impl Api {

    pub fn new() -> Api {
        Api {
            version: "v1".to_string(),
            handlers: vec![]
        }
    }

    pub fn build(builder: |&mut Api|) -> Api {
        let mut api = Api::new();
        builder(&mut api);

        return api;
    }

    pub fn version(&mut self, version: &str) {
        self.version = version.to_string();
    }
    
}

impl NamespaceBehavior for Api {
    fn handlers<'a>(&'a self) -> &'a ApiHandlers { &self.handlers }
    fn handlers_mut<'a>(&'a mut self) -> &'a mut ApiHandlers { &mut self.handlers }
}

impl Handler for Api {
    fn call(&self, req: &mut Request) -> HandleResult<Response> {
        let path = req.url.serialize_path().unwrap_or(String::new());
        self.call_handlers(path.as_slice(), &mut TreeMap::new(), req)
    }
}