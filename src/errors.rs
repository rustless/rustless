use serialize::json::JsonObject;
use std::io::IoError;
pub use error::{Error, ErrorRefExt};

#[deriving(Show)]
pub struct NotMatchError;

impl Error for NotMatchError {
    fn name(&self) -> &'static str {
        return "NotMatchError";
    }
}

#[deriving(Show)]
pub struct NotFoundError;

impl Error for NotFoundError {
    fn name(&self) -> &'static str {
        return "NotFoundError";
    }
}

#[deriving(Show)]
pub struct QueryStringDecodeError;

impl Error for QueryStringDecodeError {
    fn name(&self) -> &'static str {
        return "QueryStringDecodeError";
    }
}

#[deriving(Show)]
pub struct ValidationError {
    pub reason: JsonObject
}

impl Error for ValidationError {
    fn name(&self) -> &'static str {
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

impl Error for BodyDecodeError {
    fn name(&self) -> &'static str {
        return "BodyDecodeError";
    }

    fn description(&self) -> Option<&str> {
        return Some(self.reason.as_slice())
    }
}

#[deriving(Show)]
pub struct FileError(pub IoError);

impl Error for FileError {
    fn name(&self) -> &'static str {
        let &FileError(ref error) = self;
        error.desc
    }
}

#[deriving(Show)]
pub struct NotAcceptableError;
impl Error for NotAcceptableError {
    fn name(&self) -> &'static str {
        "NotAcceptableError"
    }
}
