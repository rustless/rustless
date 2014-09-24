
use serialize::json;
use serialize::Decodable;
use http::method::{Method, Get, Post};

use request::Request;
use response::Response;
use route::Route;
use middleware::{Handler, HandleResult, SimpleError, NotMatchError, Error, ErrorRefExt};

#[deriving(Send)]
pub struct Endpoint<T> {
	pub desc: &'static str,
	pub route: Route,
	pub method: Method,
	handler: |&params: T|:'static + Sync + Send -> String,
}

impl<T: Decodable<json::Decoder, json::DecoderError>> Endpoint<T> {

	pub fn decode(from: &str) -> T {
		json::decode(from).unwrap()
	}

	pub fn process(self, params_body: &str) -> String {
		let params = Endpoint::decode(params_body);
		let handler = self.handler;
		handler(params)
	}

	pub fn new(desc: &'static str, method: Method, handler: |&params: T|:'static + Sync + Send -> String) -> Endpoint<T> {
		Endpoint {
			desc: desc,
			method: method,
			route: Route {
				matcher: || { Err("Not implemented".to_string()) }
			},
			handler: handler
		}
	}
}

impl<T: Decodable<json::Decoder, json::DecoderError>> Handler for Endpoint<T> {
	fn call(&self, req: &mut Request) -> HandleResult<Response> {

		if false {
			Err(NotMatchError.abstract())
		} else {
			Err(NotMatchError.abstract())
		}
		
	}
}

pub struct Namespace {
	handlers: Vec<Box<Handler + Send + Sync>>	
}

impl Namespace {
	pub fn new() -> Namespace {
		Namespace {
			handlers: vec![]
		}
	}

	pub fn mount_handler(&mut self, edp: Box<Handler + Send + Sync>) {
		self.handlers.push(edp)
	}
}

impl Handler for Namespace {
	fn call(&self, req: &mut Request) -> HandleResult<Response> {
		for handler in self.handlers.iter() {
			match handler.call(req) {
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


#[deriving(Send)]
pub struct Api {
	pub version: &'static str,
	handlers: Vec<Box<Handler + Send + Sync>>
}

impl Api {

	pub fn new(version: &'static str) -> Api {
		Api {
			version: version,
			handlers: vec![]
		}
	}

	pub fn mount_namespace(&mut self, ns: Box<Namespace>) {
		self.handlers.push(ns as Box<Handler + Send + Sync>)
	}

	pub fn mount_handler(&mut self, edp: Box<Handler + Send + Sync>) {
		self.handlers.push(edp)
	}
	
}

impl Handler for Api {
	fn call(&self, req: &mut Request) -> HandleResult<Response> {

		for handler in self.handlers.iter() {
			match handler.call(req) {
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

#[test]
fn params_decode() {
	
	#[deriving(Decodable)]
	struct Params {
		user_id: String,
		user_type: Option<String>
	};

	let endpoint: Endpoint<Params> = Endpoint::new(
		"Test endpoint", 
		Get,
		|params: Params| -> String {
			assert_eq!(params.user_id.as_slice(), "test");
			assert!(
				match params.user_type {
					Some(String) => false,
					Nothing => true
				}
			)

			"Result".to_string()
		}
	);

	assert_eq!(endpoint.process("{\"user_id\": \"test\"}").as_slice(), "Result");

}