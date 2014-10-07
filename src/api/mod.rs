
use collections::treemap::TreeMap;
use serialize::json::{JsonObject};

use valico::Builder as ValicoBuilder;

use hyper;
use hyper::header::common::{Accept};
use request::Request;
use response::Response;
use middleware::{Handler, HandleResult, Error, NotMatchError};

pub use self::endpoint::{Endpoint, EndpointBuilder};
pub use self::client::Client;
pub use self::namespace::{Namespace, NS, ApiHandlers};
pub use self::media::Media;

mod endpoint;
mod namespace;
mod client;
mod media;

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
    fn api_call(&self, &str, &mut JsonObject, &mut Request) -> HandleResult<Response>;
}

pub enum Versioning {
    PathVersioning,
    AcceptHeaderVersioning(&'static str),
    ParamVersioning(&'static str)
}

#[deriving(Send)]
pub struct Api {
    pub version: Option<String>,
    pub versioning: Option<Versioning>,
    pub prefix: String,
    handlers: ApiHandlers
}

impl Api {

    pub fn new() -> Api {
        Api {
            version: None,
            versioning: None,
            prefix: "".to_string(),
            handlers: vec![]
        }
    }

    pub fn build(builder: |&mut Api|) -> Api {
        let mut api = Api::new();
        builder(&mut api);

        return api;
    }

    pub fn version(&mut self, version: &str, versioning: Versioning) {
        self.version = Some(version.to_string());
        self.versioning = Some(versioning);
    }

    pub fn prefix(&mut self, prefix: &str) {
        self.prefix = prefix.to_string();
    }
    
}

impl NS for Api {
    fn handlers<'a>(&'a self) -> &'a ApiHandlers { &self.handlers }
    fn handlers_mut<'a>(&'a mut self) -> &'a mut ApiHandlers { &mut self.handlers }
}

impl ApiHandler for Api {
    fn api_call(&self, rest_path: &str, params: &mut JsonObject, req: &mut Request) -> HandleResult<Response> {
        self.call(rest_path, req)
    }
}

impl Handler for Api {
    fn call(&self, rest_path: &str, req: &mut Request) -> HandleResult<Response> {
        
        // Check prefix
        let mut rest_path = if self.prefix.len() > 0 {
            if rest_path.slice_from(1).starts_with(self.prefix.as_slice()) {
                rest_path.slice_from(self.prefix.len() + 1)
            } else {
               return Err(NotMatchError.abstract()) 
            }
        } else {
            rest_path
        };

        // Check version
        if self.version.is_some() {
            let version = self.version.as_ref().unwrap();
            let versioning = self.versioning.as_ref().unwrap();

            match versioning {
                &PathVersioning => {
                    if rest_path.slice_from(1).starts_with(version.as_slice()) {
                        rest_path = rest_path.slice_from(version.len() + 1)
                    } else {
                       return Err(NotMatchError.abstract()) 
                    }
                },
                &ParamVersioning(ref param_name) => {
                    match req.url.query_pairs() {
                        Some(query_pairs) => {
                            if !query_pairs.iter().any(|&(ref key, ref val)| key.as_slice() == *param_name && val == version) {
                                return Err(NotMatchError.abstract()) 
                            }    
                        },
                        None => return Err(NotMatchError.abstract())
                    }
                },
                &AcceptHeaderVersioning(ref vendor) => {
                    let header = req.headers().get::<Accept>();
                    match header {
                        Some(&Accept(ref mimes)) => {
                            let mut matched_media: Option<Media> = None;
                            for mime in mimes.iter() {
                                match Media::from_mime(mime) {
                                    Some(media) => {
                                        if media.vendor.as_slice() == *vendor && 
                                           media.version.is_some() && 
                                           media.version.as_ref().unwrap() == version {
                                            matched_media = Some(media);
                                            break;
                                        }
                                    }, 
                                    None => ()
                                }
                            }

                            if matched_media.is_none() {
                                return Err(NotMatchError.abstract())
                            }
                        },
                        None => return Err(NotMatchError.abstract())
                    }
                }
            }
        }

        self.call_handlers(rest_path, &mut TreeMap::new(), req)
    }
}