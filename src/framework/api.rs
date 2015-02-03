use serialize::json;

use backend;
use server::mime;
use server::header;
use errors::{self, Error};

use framework::{self, ApiHandler};
use framework::nesting::{self, Nesting, Node};
use framework::media;
use framework::path;

#[allow(dead_code)]
#[allow(missing_copy_implementations)]
#[derive(Clone)]
pub enum Versioning {
    Path,
    AcceptHeader(&'static str),
    Param(&'static str)
}

#[derive(Clone)]
pub struct Version {
    pub version: String,
    pub versioning: Versioning,
}

pub struct Api {
    pub version: Option<Version>,
    pub prefix: Option<String>,
    pub handlers: framework::ApiHandlers,
    before: framework::Callbacks,
    before_validation: framework::Callbacks,
    after_validation: framework::Callbacks,
    after: framework::Callbacks,
    error_formatters: framework::ErrorFormatters,
    consumes: Option<Vec<mime::Mime>>,
    produces: Option<Vec<mime::Mime>>,
}

unsafe impl Send for Api {}

impl Api {

    pub fn new() -> Api {
        Api {
            version: None,
            prefix: None,
            handlers: vec![],
            before: vec![],
            before_validation: vec![],
            after_validation: vec![],
            after: vec![],
            error_formatters: vec![],
            consumes: None,
            produces: None,
        }
    }

    pub fn build<F>(builder: F) -> Api where F: FnOnce(&mut Api) {
        let mut api = Api::new();
        builder(&mut api);

        return api;
    }

    pub fn version(&mut self, version: &str, versioning: Versioning) {
        self.version = Some(Version {
            version: version.to_string(),
            versioning: versioning
        });
    }

    pub fn prefix(&mut self, prefix: &str) {
        self.prefix = Some(prefix.to_string());
    }

    pub fn consumes(&mut self, mimes: Vec<mime::Mime>) {
        self.consumes = Some(mimes);
    }

    pub fn produces(&mut self, mimes: Vec<mime::Mime>) {
        self.produces = Some(mimes);
    }

    pub fn error_formatter<F>(&mut self, formatter: F) 
    where F: Fn(&Box<Error + 'static>, &media::Media) -> Option<backend::Response> + Send+Sync {
        self.error_formatters.push(Box::new(formatter));
    }

    fn handle_error(&self, err: &Box<Error>, media: &media::Media) -> Option<backend::Response>  {
        for err_formatter in self.error_formatters.iter() {
            match err_formatter(err, media) {
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
}

impl_nesting!(Api);

impl framework::ApiHandler for Api {
    fn api_call<'a, 'r>(&'a self, 
        rest_path: &str, 
        params: &mut json::Json, 
        req: &'r mut (backend::Request + 'r), 
        info: &mut framework::CallInfo<'a>) -> backend::HandleResult<backend::Response> {

        // Check prefix
        let mut rest_path = match self.prefix.as_ref() {
            Some(prefix) => {
                if rest_path.starts_with(prefix.as_slice()) {
                    path::normalize(&rest_path[(prefix.len())..])
                } else {
                   return Err(error_response!(errors::NotMatch))
                }
            },
            None => rest_path
        };

        let mut media: Option<media::Media> = None;

        // Check version
        if self.version.is_some() {
            let version_struct = self.version.as_ref().unwrap();
            let ref version = version_struct.version;
            let ref versioning = version_struct.versioning;

            match versioning {
                &Versioning::Path => {
                    if rest_path.starts_with(version.as_slice()) {
                        rest_path = path::normalize(&rest_path[(version.len())..])
                    } else {
                       return Err(error_response!(errors::NotMatch))
                    }
                },
                &Versioning::Param(ref param_name) => {
                    match params.find(param_name) {
                        Some(obj) if obj.is_string() && obj.as_string().unwrap() == version.as_slice() => (),
                        _ => return Err(error_response!(errors::NotMatch))
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
                                return Err(error_response!(errors::NotMatch))
                            } else {
                                media = matched_media;
                            }
                        },
                        None => return Err(error_response!(errors::NotMatch))
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
                None => return Err(error_response!(errors::NotAcceptable))
            }
        }

        self.push_node(info);
        self.call_handlers(rest_path, params, req, info).map_err(|err_resp| {
            if err_resp.response.is_some() {
                err_resp
            } else {
                let resp = self.handle_error(&err_resp.error, &self.extract_media(req).unwrap_or_else(|| media::Media::default()));
                errors::ErrorResponse { 
                    error: err_resp.error,
                    response: resp
                }
            }
        })
    }
}