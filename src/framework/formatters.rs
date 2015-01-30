
use serialize::json::{ToJson, as_pretty_json};

use errors::{self, Error};
use framework;
use framework::media;
use backend;
use server::status;


pub fn create_default_error_formatter() -> framework::ErrorFormatter  {
    let closure = |&: err: &Box<errors::Error>, media: &media::Media| -> Option<backend::Response> {
        if err.is::<errors::Validation>() {
            let err = err.downcast::<errors::Validation>().unwrap();
            match media.format {
                media::Format::JsonFormat => Some(backend::Response::from_json(
                    status::StatusCode::BadRequest, 
                    &err.reason.to_json()
                )),
                // TODO: Make formatter for a mere mortals
                _ => Some(
                    backend::Response::from_string(
                        status::StatusCode::BadRequest, 
                        as_pretty_json(&err.reason.to_json()).indent(4u32).to_string()
                    )
                )  
            }
        } else if err.is::<errors::NotMatch>() {
            Some(backend::Response::new(
                    status::StatusCode::NotFound
            ))  
        } else {
            Some(backend::Response::new(
                status::StatusCode::InternalServerError
            ))
        }
    };

    Box::new(closure)
}

