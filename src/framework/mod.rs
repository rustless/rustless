use backend;
use errors;
use json::{JsonValue};

pub use self::api_handler::{ApiHandler, ApiHandlers};
pub use self::app::{Application};
pub use self::api::{Api, Versioning, Version};
pub use self::endpoint::{Endpoint, EndpointHandler, EndpointBuilder};
pub use self::client::Client;
pub use self::nesting::{Nesting, Node};
pub use self::namespace::{Namespace};
pub use self::media::Media;
pub use self::path::Path;

#[macro_use]
pub mod nesting;
pub mod api_handler;
pub mod api;
pub mod endpoint;
pub mod namespace;
pub mod client;
pub mod media;
pub mod path;
pub mod app;

pub struct CallInfo<'a> {
    pub media: media::Media,
    pub parents: Vec<&'a (nesting::Node + 'static)>,
    pub app: &'a app::Application
}

pub type Callback = Box<for<'a> Fn(&'a mut client::Client, &JsonValue) -> backend::HandleSuccessResult + 'static + Sync + Send>;
pub type Callbacks = Vec<Callback>;

pub type ErrorFormatter = Box<Fn(&errors::Error, &media::Media) -> Option<backend::Response> + 'static + Sync + Send>;
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



