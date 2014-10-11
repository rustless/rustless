
use serialize::json::ToJson;

use errors::{Error, ErrorRefExt, ValidationError};
use framework::media::Media;
use server::Response;
use server_backend::status::BadRequest;

pub fn validation_error_formatter(err: &Box<Error>, _media: &Media) -> Option<Response> {
    match err.downcast::<ValidationError>() {
        Some(err) => {
             // TODO respond with media
            Some(Response::from_json(BadRequest, &err.reason.to_json()))   
        },
        None => None
    }
}