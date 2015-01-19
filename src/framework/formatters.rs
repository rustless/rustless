
use serialize::json::{ToJson, as_pretty_json};

use errors::{Error, ValidationError};
use framework::media;
use framework::media::Media;
use backend::{Response};
use server::status::StatusCode::BadRequest;


pub fn create_default_error_formatter() -> Box<Fn(&Box<::errors::Error + 'static>, &Media) -> Option<Response> + 'static + Send+Sync>  {
    let closure = |&: err: &Box<::errors::Error + 'static>, media: &Media| -> Option<Response> {
        match err.downcast::<ValidationError>() {
            Some(err) => {
                match media.format {
                    media::Format::JsonFormat => Some(Response::from_json(BadRequest, &err.reason.to_json())),
                    // TODO: Make formatter for a mere mortals
                    _ => Some(Response::from_string(BadRequest, as_pretty_json(&err.reason.to_json()).indent(4u32).to_string()))  
                }
            },
            None => None
        }
    };

    Box::new(closure)
}

