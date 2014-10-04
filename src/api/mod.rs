
use serialize::Decodable;
use http::method::{Method};
use http::status;

use collections::treemap::TreeMap;
use serialize::json;
use serialize::json::{Json, JsonObject};
use serialize::json::ToJson;

use request::Request;
use response::Response;
use path::{Path};
use middleware::{Handler, HandleResult, SimpleError, NotMatchError, Error, ErrorRefExt};

pub use self::endpoint::Endpoint;
pub use self::namespace::Namespace;

mod endpoint;
mod namespace;

pub trait ApiHandler {
    fn call(&self, &str, &mut JsonObject, &mut Request) -> HandleResult<Response>;
}

#[deriving(Send)]
pub struct Api {
    pub version: String,
    handlers: Vec<Box<ApiHandler + Send + Sync>>
}

impl Api {

    pub fn new(version: &str) -> Api {
        Api {
            version: version.to_string(),
            handlers: vec![]
        }
    }

    pub fn mount(&mut self, edp: Box<ApiHandler + Send + Sync>) {
        self.handlers.push(edp)
    }
    
}

impl Handler for Api {
    fn call(&self, req: &mut Request) -> HandleResult<Response> {

        let path = req.url.serialize_path().unwrap_or(String::new());

        for handler in self.handlers.iter() {
            match handler.call(path.as_slice(), &mut TreeMap::new(), req) {
                Ok(response) => return Ok(response),
                Err(err) => {
                    match err.downcast::<NotMatchError>() {
                        Some(_) => (),
                        None => return Err(err),
                    }
                }
            };
        }

        Err(NotMatchError.abstract())
    }
}

// #[test]
// fn params_decode() {

//     use http::method::{Get};
    
//     #[deriving(Decodable)]
//     struct Params {
//         user_id: String,
//         user_type: Option<String>
//     };

//     let endpoint: Endpoint = Endpoint::new(
//         "Test endpoint", 
//         Get,
//         "test",
//         // |params: Params| -> String {
//         //     assert_eq!(params.user_id.as_slice(), "test");
//         //     assert!(
//         //         match params.user_type {
//         //             Some(String) => false,
//         //             Nothing => true
//         //         }
//         //     )

//         //     "Result".to_string()
//         // }
//     );

//     // assert_eq!(endpoint.process("{\"user_id\": \"test\"}").as_slice(), "Result");

// }