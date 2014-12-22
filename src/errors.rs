use serialize::json::Object;
use std::io::IoError;
pub use error::{Error};
use std::error::Error as StdError;

#[deriving(Show, Copy)]
pub struct NotMatchError;

impl StdError for NotMatchError {
    fn description(&self) -> &str {
        return "NotMatchError";
    }
}

#[deriving(Show, Copy)]
pub struct NotFoundError;

impl StdError for NotFoundError {
    fn description(&self) -> &str {
        return "NotFoundError";
    }
}

#[deriving(Show, Copy)]
pub struct QueryStringDecodeError;

impl StdError for QueryStringDecodeError {
    fn description(&self) -> &str {
        return "QueryStringDecodeError";
    }
}

#[deriving(Show)]
pub struct ValidationError {
    pub reason: Object
}

impl StdError for ValidationError {
    fn description(&self) -> &str {
        return "ValidationError";
    }
}

#[deriving(Show)]
pub struct BodyDecodeError {
    pub reason: String
}

impl BodyDecodeError {
    pub fn new(reason: String) -> BodyDecodeError {
        return BodyDecodeError {
            reason: reason
        }
    }
}

impl StdError for BodyDecodeError {
    fn description(&self) -> &str {
        return "BodyDecodeError";
    }
}

#[deriving(Show)]
pub struct FileError(pub IoError);

impl StdError for FileError {
    fn description(&self) -> &str {
        let &FileError(ref error) = self;
        error.desc
    }
}

#[deriving(Show, Copy)]
pub struct NotAcceptableError;
impl StdError for NotAcceptableError {
    fn description(&self) -> &str {
        "NotAcceptableError"
    }
}
