use serialize::json;

use backend;
use errors;

pub use self::app::{Application};
pub use self::api::{Api, Versioning};
pub use self::endpoint::{Endpoint, EndpointBuilder};
pub use self::client::Client;
pub use self::nesting::{Nesting, Node};
pub use self::namespace::{Namespace};
pub use self::media::Media;

#[macro_use]
pub mod nesting;
pub mod api;
pub mod endpoint;
pub mod namespace;
pub mod client;
pub mod media;
pub mod path;
pub mod formatters;
pub mod app;

pub struct CallInfo<'a> {
    pub media: media::Media,
    pub parents: Vec<&'a (nesting::Node + 'static)>,
    pub app: &'a app::Application
}

pub trait ApiHandler {
    fn api_call<'a>(&'a self, &str, &mut json::Object, &mut backend::Request, &mut CallInfo<'a>) -> backend::HandleResult<backend::Response>;
}

pub type ApiHandlers = Vec<Box<ApiHandler + Send + Sync>>;

pub type Callback = Box<for<'a> Fn(&'a mut client::Client, &json::Object) -> backend::HandleSuccessResult + 'static + Sync + Send>;
pub type Callbacks = Vec<Callback>;

pub type ErrorFormatter = Box<Fn(&Box<errors::Error + 'static>, &media::Media) -> Option<backend::Response> + 'static + Sync + Send>;
pub type ErrorFormatters = Vec<ErrorFormatter>;

impl<'a> CallInfo<'a> {
    pub fn new(app: &'a Application) -> CallInfo<'a> {
        CallInfo {
            media: Media::default(),
            parents: vec![],
            app: app
        }
    }
}



