use serialize::json::{JsonObject};

use valico::Builder as ValicoBuilder;

use server::{Request, Response};
use middleware::{HandleResult, HandleSuccessResult};

pub use self::api::{Api, PathVersioning, AcceptHeaderVersioning, ParamVersioning};
pub use self::endpoint::{Endpoint, EndpointBuilder};
pub use self::client::Client;
pub use self::nesting::Nesting;
pub use self::namespace::{Namespace};
pub use self::media::Media;

mod nesting;
mod api;
mod errors;
mod endpoint;
mod namespace;
mod client;
mod media;
mod path;

pub type ValicoBuildHandler<'a> = |&mut ValicoBuilder|:'a;

pub trait ApiHandler {
    fn api_call(&self, &str, &mut JsonObject, &mut Request, &mut CallInfo) -> HandleResult<Response>;
}

pub type ApiHandlers = Vec<Box<ApiHandler + Send + Sync>>;

pub type Callback = fn<'a>(&'a mut Client, &JsonObject) -> HandleSuccessResult;

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



