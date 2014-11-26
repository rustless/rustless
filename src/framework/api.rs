use collections::tree_map::TreeMap;
use serialize::json;
use serialize::json::{JsonObject};

use queryst;

use server::{Request, Response};
use server_backend::header::common::Accept;
use errors::{Error, ErrorRefExt, NotMatchError, NotAcceptableError, QueryStringDecodeError, BodyDecodeError};
use middleware::{Application, Handler, HandleResult, HandleSuccessResult};

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

    fn parse_query(query_str: &str, params: &mut JsonObject) -> HandleSuccessResult {
        let maybe_query_params = queryst::parse(query_str);
        match maybe_query_params {
            Ok(query_params) => {
                for (key, value) in query_params.as_object().unwrap().iter() {
                    if !params.contains_key(key) {
                        params.insert(key.to_string(), value.clone());
                    }
                }
            }, 
            Err(_) => {
                return Err(box QueryStringDecodeError as Box<Error>);
            }
        }

        Ok(())
    }

    fn parse_json_body(req: &mut Request, params: &mut JsonObject) -> HandleSuccessResult {
        let maybe_body = req.read_to_end();
        
        let utf8_string_body = {
            match maybe_body {
                Ok(body) => {
                    match String::from_utf8(body) {
                        Ok(e) => e,
                        Err(_) => return Err(box BodyDecodeError::new("Invalid UTF-8 sequence".to_string()) as Box<Error>),
                    }
                },
                Err(err) => return Err(box BodyDecodeError::new(format!("{}", err)) as Box<Error>)
            }
        };

        if utf8_string_body.len() > 0 {
          let maybe_json_body = json::from_str(utf8_string_body.as_slice());
            match maybe_json_body {
                Ok(json_body) => {
                    for (key, value) in json_body.as_object().unwrap().iter() {
                        if !params.contains_key(key) {
                            params.insert(key.to_string(), value.clone());
                        }
                    }
                },
                Err(err) => return Err(box BodyDecodeError::new(format!("{}", err)) as Box<Error>)
            }  
        }

        Ok(())
    }

    fn parse_request(req: &mut Request, params: &mut JsonObject) -> HandleSuccessResult {
        // extend params with query-string params if any
        if req.url().query.is_some() {
            try!(Api::parse_query(req.url().query.as_ref().unwrap().as_slice(), params));   
        }

        // extend params with json-encoded body params if any
        if req.is_json_body() {
            try!(Api::parse_json_body(req, params));
        }

        Ok(())
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
               return Err(box NotMatchError as Box<Error>) 
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
                &Versioning::PathVersioning => {
                    if rest_path.slice_from(1).starts_with(version.as_slice()) {
                        rest_path = rest_path.slice_from(version.len() + 1)
                    } else {
                       return Err(box NotMatchError as Box<Error>) 
                    }
                },
                &Versioning::ParamVersioning(ref param_name) => {
                    match req.url().query_pairs() {
                        Some(query_pairs) => {
                            if !query_pairs.iter().any(|&(ref key, ref val)| key.as_slice() == *param_name && val == version) {
                                return Err(box NotMatchError as Box<Error>) 
                            }    
                        },
                        None => return Err(box NotMatchError as Box<Error>)
                    }
                },
                &Versioning::AcceptHeaderVersioning(ref vendor) => {
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
                                return Err(box NotMatchError as Box<Error>)
                            } else {
                                media = matched_media;
                            }
                        },
                        None => return Err(box NotMatchError as Box<Error>)
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
                None => return Err(box NotAcceptableError as Box<Error>)
            }
        }

        self.push_callbacks(info);
        self.call_handlers(rest_path, params, req, info)
    }
}

impl Handler for Api {
    fn call(&self, rest_path: &str, req: &mut Request, app: &Application) -> HandleResult<Response> {

        let mut params = TreeMap::new();
        try!(Api::parse_request(req, &mut params));
        
        match self.api_call(rest_path, &mut params, req, &mut CallInfo::new(app))  {
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