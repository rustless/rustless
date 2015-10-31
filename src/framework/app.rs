use std::collections;
use serialize::json;
use typemap;
use queryst;
use valico::MutableJson;

use super::{ApiHandler};
use super::api;
use super::super::backend;
use super::super::errors;
use super::super::server::status;

pub struct Application {
    pub ext: typemap::TypeMap,
    pub root_api: api::Api,
}

unsafe impl Send for Application {}
unsafe impl Sync for Application {}

impl Application {
    pub fn new(root_api: api::Api) -> Application {
        Application {
            root_api: root_api,
            ext: typemap::TypeMap::new()
        }
    }

    fn call_internal<'a>(&self, req: &'a mut (backend::Request + 'a)) -> backend::HandleResult<backend::Response> {
        let mut params = json::Json::Object(collections::BTreeMap::new());
        let parse_result = parse_request(req, &mut params);

        parse_result.and_then(|_| {
            self.root_api.api_call(&(req.url().path().join("/")), &mut params, req, &mut super::CallInfo::new(self))
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
                    let response = if (&*error as &errors::Error).is::<errors::NotMatch>() {
                        backend::Response::new(
                            status::StatusCode::NotFound
                        )
                    } else if (&*error as &errors::Error).is::<errors::Validation>() {
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

impl super::super::Extensible for Application {
    fn ext(&self) -> &::typemap::TypeMap { &self.ext }
    fn ext_mut(&mut self) -> &mut ::typemap::TypeMap { &mut self.ext }
}

fn parse_query(query_str: &str, params: &mut json::Json) -> backend::HandleSuccessResult {
    let maybe_query_params = queryst::parse(query_str);
    match maybe_query_params {
        Ok(query_params) => {
            let params = params.as_object_mut().expect("Params must be an object");
            for (key, value) in query_params.as_object().expect("Query params must be an object").iter() {
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

fn parse_json_body(req: &mut backend::Request, params: &mut json::Json) -> backend::HandleSuccessResult {
    let maybe_body = try!(req.read_to_end().map_err(|err| error_response_boxed!(err)))
        .unwrap_or(String::new());

    if maybe_body.len() > 0 {
      let maybe_json_body = maybe_body.parse::<json::Json>();
        match maybe_json_body {
            Ok(json_body) => {
                let params = params.as_object_mut().expect("Params must be object");
                if json_body.is_object() {
                    for (key, value) in json_body.as_object().unwrap().iter() {
                        if !params.contains_key(key) {
                            params.insert(key.to_string(), value.clone());
                        }
                    }
                } else {
                    params.insert("body".to_string(), json_body);
                }
            },
            Err(e) => return Err(error_response!(errors::Body::new(format!("Invalid JSON: {}", e))))
        }
    }

    Ok(())
}

fn parse_urlencoded_body(req: &mut backend::Request, params: &mut json::Json) -> backend::HandleSuccessResult {
    let maybe_body = try!(req.read_to_end().map_err(|err| error_response_boxed!(err)))
        .unwrap_or(String::new());

    if maybe_body.len() > 0 {
        let maybe_json_body = queryst::parse(&maybe_body);
        match maybe_json_body {
            Ok(json_body) => {
                let params = params.as_object_mut().expect("Params must be object");
                if json_body.is_object() {
                    for (key, value) in json_body.as_object().unwrap().iter() {
                        if !params.contains_key(key) {
                            params.insert(key.to_string(), value.clone());
                        }
                    }
                } else {
                    params.insert("body".to_string(), json_body);
                }
            },
            Err(_) => return Err(error_response!(errors::Body::new(format!("Invalid encoded data"))))
        }
    }

    Ok(())
}

fn parse_request(req: &mut backend::Request, params: &mut json::Json) -> backend::HandleSuccessResult {
    // extend params with query-string params if any
    if req.url().query().is_some() {
        try!(parse_query(&req.url().query().as_ref().unwrap(), params));
    }

    // extend params with json-encoded body params if any
    if req.is_json_body() {
        try!(parse_json_body(req, params));
    } else if req.is_urlencoded_body() {
        try!(parse_urlencoded_body(req, params));
    }

    Ok(())
}