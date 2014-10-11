
use serialize::json::ToJson;

use errors::{Error, ErrorRefExt, ValidationError};
use framework::media;
use framework::media::Media;
use server::Response;
use server_backend::status::BadRequest;

pub fn validation_error_formatter(err: &Box<Error>, media: &Media) -> Option<Response> {
    match err.downcast::<ValidationError>() {
        Some(err) => {
            match media.format {
                media::JsonFormat => Some(Response::from_json(BadRequest, &err.reason.to_json())),
                // TODO: Make formatter for a mere mortals
                _ => Some(Response::from_string(BadRequest, err.reason.to_json().to_pretty_str()))  
            }
        },
        None => None
    }
}