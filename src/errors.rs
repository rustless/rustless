use std::io;
pub use error::{Error};
use std::error::Error as StdError;
use valico;

use super::backend;

pub struct ErrorResponse {
    pub error: Box<Error + Send>,
    pub response: Option<backend::Response>
}

pub struct StrictErrorResponse {
    pub error: Box<Error + Send>,
    pub response: backend::Response
}

macro_rules! error_response{
    ($error:expr) => ($crate::errors::ErrorResponse{
        error: Box::new($error) as Box<$crate::errors::Error + Send>,
        response: None
    })
}

macro_rules! error_response_boxed{
    ($error:expr) => ($crate::errors::ErrorResponse{
        error: $error,
        response: None
    })
}


macro_rules! impl_basic_err {
    ($err:ty, $code:expr) => {
        impl ::std::error::Error for $err {
            fn description(&self) -> &str {
                $code
            }
        }

        impl ::std::fmt::Display for $err {
            fn fmt(&self, formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                self.description().fmt(formatter)
            }
        }
    }
}

#[derive(Debug)]
pub struct NotMatch;
impl_basic_err!(NotMatch, "NotMatch");

#[derive(Debug)]
pub struct NotFound;
impl_basic_err!(NotFound, "NotFound");

#[derive(Debug)]
pub struct QueryString;
impl_basic_err!(QueryString, "QueryString");

#[derive(Debug)]
pub struct Validation {
    pub reason: valico::ValicoErrors
}
impl_basic_err!(Validation, "Validation");

#[derive(Debug)]
pub struct Body {
    pub reason: String
}

impl Body {
    pub fn new(reason: String) -> Body {
        return Body {
            reason: reason
        }
    }
}
impl_basic_err!(Body, "Body");

#[derive(Debug)]
pub struct File(pub io::Error);
impl_basic_err!(File, "File");

#[derive(Debug)]
pub struct NotAcceptable;
impl_basic_err!(NotAcceptable, "NotAcceptable");
