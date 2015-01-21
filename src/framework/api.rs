use collections;
use serialize::json;

use queryst;

use backend;
use server::header;
use errors::{self, Error};

use framework::{self, ApiHandler};
use framework::app;
use framework::nesting::{self, Nesting, Node};
use framework::media;
use framework::formatters;
use framework::path;

#[allow(dead_code)]
#[allow(missing_copy_implementations)]
pub enum Versioning {
    Path,
    AcceptHeader(&'static str),
    Param(&'static str)
}

pub struct Api {
    pub version: Option<String>,
    pub versioning: Option<Versioning>,
    pub prefix: String,
    pub handlers: framework::ApiHandlers,
    before: framework::Callbacks,
    before_validation: framework::Callbacks,
    after_validation: framework::Callbacks,
    after: framework::Callbacks,
    error_formatters: framework::ErrorFormatters,
    default_error_formatters: framework::ErrorFormatters
}

unsafe impl Send for Api {}

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
            default_error_formatters: vec![formatters::create_default_error_formatter()]
        }
    }

    pub fn build<F>(builder: F) -> Api where F: FnOnce(&mut Api) {
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

    pub fn error_formatter<F>(&mut self, formatter: F) 
    where F: Fn(&Box<Error + 'static>, &media::Media) -> Option<backend::Response> + Send+Sync {
        self.error_formatters.push(Box::new(formatter));
    }

    fn handle_error(&self, err: &Box<Error>, media: &media::Media) -> Option<backend::Response>  {
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

    fn extract_media(&self, req: &backend::Request) -> Option<media::Media> {
        let header = req.headers().get::<header::Accept>();
        match header {
            Some(&header::Accept(ref mimes)) if !mimes.is_empty() => {
                // TODO: Allow only several mime types
                Some(media::Media::from_mime(&mimes[0].item))
            },
            _ => Some(media::Media::default())
        }
    }

    fn parse_query(query_str: &str, params: &mut json::Object) -> backend::HandleSuccessResult {
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
                return Err(Box::new(errors::QueryString) as Box<Error>);
            }
        }

        Ok(())
    }

    fn parse_json_body(req: &mut backend::Request, params: &mut json::Object) -> backend::HandleSuccessResult {

        let utf8_string_body = match String::from_utf8(req.body().clone()) {
            Ok(e) => e,
            Err(_) => return Err(Box::new(
                errors::Body::new("Invalid UTF-8 sequence".to_string())
            ) as Box<Error>),
        };

        if utf8_string_body.len() > 0 {
          let maybe_json_body = utf8_string_body.parse::<json::Json>();
            match maybe_json_body {
                Some(json_body) => {
                    for (key, value) in json_body.as_object().unwrap().iter() {
                        if !params.contains_key(key) {
                            params.insert(key.to_string(), value.clone());
                        }
                    }
                },
                None => return Err(Box::new(errors::Body::new(format!("Invalid JSON"))) as Box<Error>)
            }  
        }

        Ok(())
    }

    fn parse_request(req: &mut backend::Request, params: &mut json::Object) -> backend::HandleSuccessResult {
        // extend params with query-string params if any
        if req.url().query().is_some() {
            try!(Api::parse_query(req.url().query().as_ref().unwrap().as_slice(), params));   
        }

        // extend params with json-encoded body params if any
        if req.is_json_body() {
            try!(Api::parse_json_body(req, params));
        }

        Ok(())
    }

    #[allow(unused_variables)]
    pub fn call(&self, rest_path: &str, req: &mut backend::Request, app: &app::Application) 
    -> backend::HandleResult<backend::Response> {
        let mut params = collections::BTreeMap::new();
        try!(Api::parse_request(req, &mut params));
        
        match self.api_call(rest_path, &mut params, req, &mut framework::CallInfo::new(app))  {
            Ok(resp) => Ok(resp),
            Err(err) => {
                if err.downcast::<errors::NotMatch>().is_none() {
                    // FIXME: Here we extract mime second time (first time in `api_call`), 
                    //        maybe we can do something better?
                    match self.handle_error(&err, &self.extract_media(req).unwrap_or_else(|| media::Media::default())) {
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

impl_nesting!(Api);

impl framework::ApiHandler for Api {
    fn api_call<'a>(&'a self, rest_path: &str, params: &mut json::Object, 
                    req: &mut backend::Request, info: &mut framework::CallInfo<'a>) 
    -> backend::HandleResult<backend::Response> {
        
        // Check prefix
        let mut rest_path = if self.prefix.len() > 0 {
            if rest_path.starts_with(self.prefix.as_slice()) {
                path::normalize(rest_path.slice_from(self.prefix.len()))
            } else {
               return Err(Box::new(errors::NotMatch) as Box<Error>)
            }
        } else {
            rest_path
        };

        let mut media: Option<media::Media> = None;

        // Check version
        if self.version.is_some() {
            let version = self.version.as_ref().unwrap();
            let versioning = self.versioning.as_ref().unwrap();

            match versioning {
                &Versioning::Path => {
                    if rest_path.starts_with(version.as_slice()) {
                        rest_path = path::normalize(rest_path.slice_from(version.len()))
                    } else {
                       return Err(Box::new(errors::NotMatch) as Box<Error>)
                    }
                },
                &Versioning::Param(ref param_name) => {
                    match params.get(*param_name) {
                        Some(obj) if obj.is_string() && obj.as_string().unwrap() == version.as_slice() => (),
                        _ => return Err(Box::new(errors::NotMatch) as Box<Error>)
                    }
                },
                &Versioning::AcceptHeader(ref vendor) => {
                    let header = req.headers().get::<header::Accept>();
                    match header {
                        Some(&header::Accept(ref quals)) => {
                            let mut matched_media: Option<media::Media> = None;
                            for qual in quals.iter() {
                                match media::Media::from_vendor(&qual.item) {
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
                                return Err(Box::new(errors::NotMatch) as Box<Error>)
                            } else {
                                media = matched_media;
                            }
                        },
                        None => return Err(Box::new(errors::NotMatch) as Box<Error>)
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
                None => return Err(Box::new(errors::NotAcceptable) as Box<Error>)
            }
        }

        self.push_node(info);
        self.call_handlers(rest_path, params, req, info)
    }
}