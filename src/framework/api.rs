use collections::treemap::TreeMap;
use serialize::json::{JsonObject};

use server::{Request, Response};
use server_backend::header::common::Accept;
use middleware::{Handler, HandleResult, Error, NotMatchError};

use framework::nesting::Nesting;
use framework::media::Media;
use framework::{ApiHandler, ApiHandlers, Callback, CallInfo};

#[allow(dead_code)]
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
    handlers: ApiHandlers,
    before: Vec<Callback>,
    before_validation: Vec<Callback>,
    after_validation: Vec<Callback>,
    after: Vec<Callback>
}

impl Api {

    pub fn new() -> Api {
        Api {
            version: None,
            versioning: None,
            prefix: "".to_string(),
            handlers: vec![],
            before: vec![],
            before_validation: vec![],
            after_validation: vec![],
            after: vec![]
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

impl Nesting for Api {
    fn get_handlers<'a>(&'a self) -> &'a ApiHandlers { &self.handlers }
    fn get_handlers_mut<'a>(&'a mut self) -> &'a mut ApiHandlers { &mut self.handlers }

    fn get_before<'a>(&'a self) -> &'a Vec<Callback> { &self.before }
    fn get_before_mut<'a>(&'a mut self) -> &'a mut Vec<Callback> { &mut self.before }

    fn get_before_validation<'a>(&'a self) -> &'a Vec<Callback> { &self.before_validation }
    fn get_before_validation_mut<'a>(&'a mut self) -> &'a mut Vec<Callback> { &mut self.before_validation }

    fn get_after_validation<'a>(&'a self) -> &'a Vec<Callback> { &self.after_validation }
    fn get_after_validation_mut<'a>(&'a mut self) -> &'a mut Vec<Callback> { &mut self.after_validation }

    fn get_after<'a>(&'a self) -> &'a Vec<Callback> { &self.after }
    fn get_after_mut<'a>(&'a mut self) -> &'a mut Vec<Callback> { &mut self.after }
}

impl ApiHandler for Api {
    fn api_call(&self, rest_path: &str, params: &mut JsonObject, req: &mut Request, info: &mut CallInfo) -> HandleResult<Response> {
        
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

        self.push_callbacks(info);
        self.call_handlers(rest_path, params, req, info)
    }
}

impl Handler for Api {
    fn call(&self, rest_path: &str, req: &mut Request) -> HandleResult<Response> {
        self.api_call(rest_path, &mut TreeMap::new(), req, &mut CallInfo::new())
    }
}