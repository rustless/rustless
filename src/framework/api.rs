use collections::treemap::TreeMap;
use serialize::json::{JsonObject};

use server::{Request, Response};
use server_backend::header::common::Accept;
use errors::{Error, ErrorRefExt, NotMatchError, NotAcceptableError};
use middleware::{Handler, HandleResult};

use framework::nesting::Nesting;
use framework::media::Media;
use framework::{ApiHandler, ApiHandlers, Callbacks, CallInfo, ErrorFormatters, ErrorFormatter};
use framework::formatters;

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
    before: Callbacks,
    before_validation: Callbacks,
    after_validation: Callbacks,
    after: Callbacks,
    error_formatters: ErrorFormatters,
    default_error_formatters: ErrorFormatters
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
            after: vec![],
            error_formatters: vec![],
            default_error_formatters: vec![formatters::validation_error_formatter]
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

    pub fn error_formatter(&mut self, formatter: ErrorFormatter) {
        self.error_formatters.push(formatter);
    }

    fn handle_error(&self, err: &Box<Error>, media: &Media) -> Option<Response>  {
        for err_formatter in self.error_formatters.iter() {
            match (*err_formatter)(err, media) {
                Some(resp) => return Some(resp),
                None => ()
            }
        }

        for err_formatter in self.default_error_formatters.iter() {
            match (*err_formatter)(err, media) {
                Some(resp) => return Some(resp),
                None => ()
            }
        }

        None
    }

    fn extract_media(&self, req: &Request) -> Option<Media> {
        let header = req.headers().get::<Accept>();
        match header {
            Some(&Accept(ref mimes)) if !mimes.is_empty() => {
                // TODO: Allow only several mime types
                Some(Media::from_mime(&mimes[0]))
            },
            _ => Some(Media::default())
        }
    }
    
}

impl Nesting for Api {
    fn get_handlers<'a>(&'a self) -> &'a ApiHandlers { &self.handlers }
    fn get_handlers_mut<'a>(&'a mut self) -> &'a mut ApiHandlers { &mut self.handlers }

    fn get_before<'a>(&'a self) -> &'a Callbacks { &self.before }
    fn get_before_mut<'a>(&'a mut self) -> &'a mut Callbacks { &mut self.before }

    fn get_before_validation<'a>(&'a self) -> &'a Callbacks { &self.before_validation }
    fn get_before_validation_mut<'a>(&'a mut self) -> &'a mut Callbacks { &mut self.before_validation }

    fn get_after_validation<'a>(&'a self) -> &'a Callbacks { &self.after_validation }
    fn get_after_validation_mut<'a>(&'a mut self) -> &'a mut Callbacks { &mut self.after_validation }

    fn get_after<'a>(&'a self) -> &'a Callbacks { &self.after }
    fn get_after_mut<'a>(&'a mut self) -> &'a mut Callbacks { &mut self.after }
}

impl ApiHandler for Api {
    fn api_call(&self, rest_path: &str, params: &mut JsonObject, req: &mut Request, info: &mut CallInfo) -> HandleResult<Response> {
        
        // Check prefix
        let mut rest_path = if self.prefix.len() > 0 {
            if rest_path.slice_from(1).starts_with(self.prefix.as_slice()) {
                rest_path.slice_from(self.prefix.len() + 1)
            } else {
               return Err(NotMatchError.erase()) 
            }
        } else {
            rest_path
        };

        let mut media: Option<Media> = None;

        // Check version
        if self.version.is_some() {
            let version = self.version.as_ref().unwrap();
            let versioning = self.versioning.as_ref().unwrap();

            match versioning {
                &PathVersioning => {
                    if rest_path.slice_from(1).starts_with(version.as_slice()) {
                        rest_path = rest_path.slice_from(version.len() + 1)
                    } else {
                       return Err(NotMatchError.erase()) 
                    }
                },
                &ParamVersioning(ref param_name) => {
                    match req.url().query_pairs() {
                        Some(query_pairs) => {
                            if !query_pairs.iter().any(|&(ref key, ref val)| key.as_slice() == *param_name && val == version) {
                                return Err(NotMatchError.erase()) 
                            }    
                        },
                        None => return Err(NotMatchError.erase())
                    }
                },
                &AcceptHeaderVersioning(ref vendor) => {
                    let header = req.headers().get::<Accept>();
                    match header {
                        Some(&Accept(ref mimes)) => {
                            let mut matched_media: Option<Media> = None;
                            for mime in mimes.iter() {
                                match Media::from_vendor(mime) {
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
                                return Err(NotMatchError.erase())
                            } else {
                                media = matched_media;
                            }
                        },
                        None => return Err(NotMatchError.erase())
                    }
                }
            }
        }

        // Check accept media type
        if media.is_none() {
            match self.extract_media(req) {
                Some(media) => {
                    info.media = media
                },
                None => return Err(NotAcceptableError.erase())
            }
        }

        self.push_callbacks(info);
        self.call_handlers(rest_path, params, req, info)
    }
}

impl Handler for Api {
    fn call(&self, rest_path: &str, req: &mut Request) -> HandleResult<Response> {
        match self.api_call(rest_path, &mut TreeMap::new(), req, &mut CallInfo::new())  {
            Ok(resp) => Ok(resp),
            Err(err) => {
                if err.downcast::<NotMatchError>().is_none() {
                    // FIXME: Here we extract mime second time (first time in `api_call`), 
                    //        maybe we can do something better?
                    match self.handle_error(&err, &self.extract_media(req).unwrap_or_else(|| Media::default())) {
                        Some(resp) => Ok(resp),
                        None => Err(err)
                    }
                } else {
                    Err(err)
                }
            }
        }
    }
}