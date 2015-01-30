use std::collections;
use serialize::json;
use typemap;
use queryst;

use super::{ApiHandler};
use super::api;
use super::super::backend;
use super::super::errors;
use super::super::server::status;

pub struct Application {
    pub ext: typemap::TypeMap,
    pub root_api: api::Api 
}

unsafe impl Send for Application {}

impl Application {
    pub fn new(root_api: api::Api) -> Application {
        Application {
            root_api: root_api,
            ext: typemap::TypeMap::new()
        }
    }

    fn call_internal<'a>(&self, req: &'a mut (backend::Request + 'a)) -> backend::HandleResult<backend::Response> {
        let mut params = collections::BTreeMap::new();
        let parse_result = parse_request(req, &mut params);

        parse_result.and_then(|_| {
            self.root_api.api_call((req.url().path().connect("/").as_slice()).as_slice(), &mut params, req, &mut super::CallInfo::new(self))
        })
    }

    pub fn call<'a>(&self, req: &'a mut (backend::Request + 'a)) -> backend::HandleResultStrict<backend::Response> {
        self.call_internal(req)
            .map_err(|error_response| {
                if error_response.response.is_some() {
                    let errors::ErrorResponse{error, response} = error_response;
                    errors::StrictErrorResponse{
                        error: error,
                        response: response.unwrap()
                    }
                } else {
                    let errors::ErrorResponse{error, ..} = error_response;

                    // Simple default error responses for common errors
                    let response = if error.is::<errors::NotMatch>() {
                        backend::Response::new(
                            status::StatusCode::NotFound
                        )
                    } else if error.is::<errors::Validation>() {
                        backend::Response::new(
                            status::StatusCode::BadRequest
                        )
                    } else {
                        backend::Response::new(
                            status::StatusCode::InternalServerError
                        )
                    };

                    errors::StrictErrorResponse {
                        error: error,
                        response: response
                    }
                } 
            })
    }
}

fn parse_query(query_str: &str, params: &mut json::Object) -> backend::HandleSuccessResult {
    let maybe_query_params = queryst::parse(query_str);
    match maybe_query_params {
        Ok(query_params) => {
            for (key, value) in query_params.as_object().unwrap().iter() {
                if !params.contains_key(key) {
                    params.insert(key.to_string(), value.clone());
                }
            }
        }, 
        Err(_) => {
            return Err(error_response!(errors::QueryString));
        }
    }

    Ok(())
}

fn parse_utf8(req: &mut backend::Request) -> backend::HandleResult<String> {
    // FIXME https://github.com/rustless/rustless/issues/19
    //       We need to implement some common Iron middleware/plugin 
    //       and use it instead of `.read_to_end()`.
    match req.body_mut().read_to_end() {
        Ok(bytes) => {
             match String::from_utf8(bytes) {
                Ok(e) => Ok(e),
                Err(_) => Err(error_response!(
                    errors::Body::new("Invalid UTF-8 sequence".to_string())
                )),
            }
        },
        Err(_) => Err(error_response!(
            errors::Body::new("Invalid request body".to_string())
        )),
    }
}

fn parse_json_body(req: &mut backend::Request, params: &mut json::Object) -> backend::HandleSuccessResult {

    let utf8_string_body = try!(parse_utf8(req));

    if utf8_string_body.len() > 0 {
      let maybe_json_body = utf8_string_body.parse::<json::Json>();
        match maybe_json_body {
            Some(json_body) => {
                for (key, value) in json_body.as_object().unwrap().iter() {
                    if !params.contains_key(key) {
                        params.insert(key.to_string(), value.clone());
                    }
                }
            },
            None => return Err(error_response!(errors::Body::new(format!("Invalid JSON"))))
        }  
    }

    Ok(())
}

fn parse_urlencoded_body(req: &mut backend::Request, params: &mut json::Object) -> backend::HandleSuccessResult {
    let utf8_string_body = try!(parse_utf8(req));

    if utf8_string_body.len() > 0 {
        let maybe_json_body = queryst::parse(utf8_string_body.as_slice());
        match maybe_json_body {
            Ok(json_body) => {
                for (key, value) in json_body.as_object().unwrap().iter() {
                    if !params.contains_key(key) {
                        params.insert(key.to_string(), value.clone());
                    }
                }
            },
            Err(_) => return Err(error_response!(errors::Body::new(format!("Invalid encoded data"))))
        }  
    }

    Ok(())
}

fn parse_request(req: &mut backend::Request, params: &mut json::Object) -> backend::HandleSuccessResult {
    // extend params with query-string params if any
    if req.url().query().is_some() {
        try!(parse_query(req.url().query().as_ref().unwrap().as_slice(), params));   
    }

    // extend params with json-encoded body params if any
    if req.is_json_body() {
        try!(parse_json_body(req, params));
    } else if req.is_urlencoded_body() {
        try!(parse_urlencoded_body(req, params));
    }

    Ok(())
}