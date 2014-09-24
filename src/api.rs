
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

pub trait ApiHandler: Send + Sync {
	fn call(&self, &str, &mut JsonObject, &mut Request) -> HandleResult<Response>;
}

#[deriving(Send)]
pub struct Endpoint<T> {
	pub desc: &'static str,
	pub path: Path,
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

	pub fn new(desc: &'static str, method: Method, path: &'static str, handler: |&params: T|:'static + Sync + Send -> String) -> Endpoint<T> {
		Endpoint {
			desc: desc,
			method: method,
			path: Path::parse(path, true).unwrap(),
			handler: handler
		}
	}
}

impl<T: Decodable<json::Decoder, json::DecoderError>> ApiHandler for Endpoint<T> {
	fn call(&self, rest_path: &str, params: &mut JsonObject, req: &mut Request) -> HandleResult<Response> {
		match self.path.is_match(rest_path) {
			Some(captures) =>  {
				return Ok(Response::from_string(status::Ok, "MATCH ENDPOINT!!".to_string()))
			},
			None => return Err(NotMatchError.abstract())
		};
	}
}

pub struct Namespace {
	handlers: Vec<Box<ApiHandler + Send + Sync>>,
	path: Path	
}

impl Namespace {
	pub fn new(path: &'static str) -> Namespace {
		Namespace {
			handlers: vec![],
			path: Path::parse(path, false).unwrap()
		}
	}

	pub fn mount(&mut self, edp: Box<ApiHandler + Send + Sync>) {
		self.handlers.push(edp)
	}
}

impl ApiHandler for Namespace {
	fn call(&self, rest_path: &str, params: &mut JsonObject, req: &mut Request) -> HandleResult<Response> {

		let rest_path: &str = match self.path.is_match(rest_path) {
			Some(captures) =>  {
				for param in self.path.params.iter() {
					params.insert(param.clone(), captures.name(param.as_slice()).to_string().to_json());
				}

				println!("{}", params.to_json().to_pretty_str());
				rest_path.slice_from(captures.at(0).len())
			},
			None => return Err(NotMatchError.abstract())
		};

		for handler in self.handlers.iter() {
			match handler.call(rest_path, params, req) {
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
	handlers: Vec<Box<ApiHandler + Send + Sync>>
}

impl Api {

	pub fn new(version: &'static str) -> Api {
		Api {
			version: version,
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

#[test]
fn params_decode() {

	use http::method::{Get};
	
	#[deriving(Decodable)]
	struct Params {
		user_id: String,
		user_type: Option<String>
	};

	let endpoint: Endpoint<Params> = Endpoint::new(
		"Test endpoint", 
		Get,
		"test",
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