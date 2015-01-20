use serialize::json;
use std::io;
pub use error::{Error};
use std;

#[derive(Show, Copy)]
pub struct NotMatch;

impl std::error::Error for NotMatch {
    fn description(&self) -> &str {
        return "NotMatch";
    }
}

#[derive(Show, Copy)]
pub struct NotFound;

impl std::error::Error for NotFound {
    fn description(&self) -> &str {
        return "NotFound";
    }
}

#[derive(Show, Copy)]
pub struct QueryString;

impl std::error::Error for QueryString {
    fn description(&self) -> &str {
        return "QueryString";
    }
}

#[derive(Show)]
pub struct Validation {
    pub reason: json::Object
}

impl std::error::Error for Validation {
    fn description(&self) -> &str {
        return "Validation";
    }
}

#[derive(Show)]
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

impl std::error::Error for Body {
    fn description(&self) -> &str {
        return "Body";
    }
}

#[derive(Show)]
pub struct File(pub io::IoError);

impl std::error::Error for File {
    fn description(&self) -> &str {
        let &File(ref error) = self;
        error.desc
    }
}

#[derive(Show, Copy)]
pub struct NotAcceptable;
impl std::error::Error for NotAcceptable {
    fn description(&self) -> &str {
        "NotAcceptable"
    }
}
