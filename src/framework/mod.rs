use serialize::json::{Object};

use backend::{Request, Response};
use backend::{HandleResult, HandleSuccessResult};
use errors::{Error};

pub use self::api::{Application, Api, Versioning};
pub use self::endpoint::{Endpoint, EndpointBuilder};
pub use self::client::Client;
pub use self::nesting::{Nesting, Node};
pub use self::namespace::{Namespace};
pub use self::media::Media;

#[macro_use]
mod nesting;
mod api;
mod endpoint;
mod namespace;
mod client;
mod media;
mod path;
mod formatters;

pub trait ApiHandler {
    fn api_call<'a>(&'a self, &str, &mut Object, &mut Request, &mut CallInfo<'a>) -> HandleResult<Response>;
}

pub type ApiHandlers = Vec<Box<ApiHandler + Send + Sync>>;

pub type Callback = Box<for<'a> Fn(&'a mut Client, &Object) -> HandleSuccessResult + 'static + Sync + Send>;
pub type ErrorFormatter = Box<Fn(&Box<Error + 'static>, &Media) -> Option<Response> + 'static + Sync + Send>;

pub type Callbacks = Vec<Callback>;
pub type ErrorFormatters = Vec<ErrorFormatter>;

pub struct CallInfo<'a> {
    pub media: Media,
    pub parents: Vec<&'a (Node + 'static)>,
    pub app: &'a Application
}

impl<'a> CallInfo<'a> {
    pub fn new(app: &'a Application) -> CallInfo<'a> {
        CallInfo {
            media: Media::default(),
            parents: vec![],
            app: app
        }
    }
}



