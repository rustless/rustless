pub use self::simple_request::{SimpleRequest};
pub use self::request::{Request, AsUrl};
pub use self::response::{Response};

pub use self::iron::{
    Url, 
    Handler, 
    HandleResult, 
    HandleSuccessResult, 
    HandleExtendedResult,
    WrapUrl,
    ErrorResponse
};

pub mod request;
mod simple_request;
mod response;
mod iron;