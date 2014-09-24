
use request::Request;
use response::Response;
use endpoint::Endpoint;

use std::fmt::Show;
use std::any::{Any, AnyRefExt};

trait Error: Show {
	fn name(&self) -> &'static str;
    fn description(&self) -> Option<&str> { None }
}

#[deriving(Show)]
pub struct SimpleError {
    pub name: &'static str
}

impl Error for SimpleError {
    fn name(&self) -> &'static str {
    	return self.name;
    }
}

pub type HandleResult<'a, T> = Result<T, Box<Any + 'a>>;
pub type HandleSuccessResult<'a> = HandleResult<'a, ()>;

pub trait BeforeMiddleware: Send + Sync {
    fn before(&self, &mut Request) -> HandleSuccessResult;
}

pub trait AfterMiddleware: Send + Sync {
    fn after(&self, &mut Request, &mut Response) -> HandleSuccessResult;
}

pub trait CatchMiddleware: Send + Sync {
    fn catch(&self, &mut Request) -> Result<(), HandleSuccessResult>;
}

pub trait Handler: Send + Sync {
	fn call(&self, &mut Request) -> HandleResult<Response>;
}

#[deriving(Send)]
pub struct Application {
    before: Vec<Box<BeforeMiddleware + Send + Sync>>,
    after: Vec<Box<AfterMiddleware + Send + Sync>>,
    catch: Vec<Box<CatchMiddleware + Send + Sync>>,
    endpoints: Vec<Box<Handler + Send + Sync>>
}

impl Application {
	pub fn call(&self, req: &mut Request) -> HandleResult<Response> {

		for mdw in self.before.iter() {
			match mdw.before(req) {
				Err(some) => self.handle_error(),
				Ok(()) => ()
			}
		}

		let mut response: Option<Response> = None;

		for edp in self.endpoints.iter() {
			match edp.call(req) {
				Ok(resp) => response = Some(resp),
				Err(some) => self.handle_error(),
			}
		}

		let mut exact_response = response.unwrap();

		for mdw in self.after.iter() {
			match mdw.after(req, &mut exact_response) {
				Err(some) => self.handle_error(),
				Ok(()) => ()
			}
		}

		Ok(exact_response)

	}

	pub fn handle_error(&self) {

	}
}

pub struct Builder {
	app: Application
}

trait AfterMiddlewareSupport {
	fn using(&mut self, middleware: Box<AfterMiddleware + Send + Sync>);
}

trait BeforeMiddlewareSupport {
	fn using(&mut self, middleware: Box<BeforeMiddleware + Send + Sync>);
}

trait RunHandlerSupport {
	fn run(&mut self, middleware: Box<Handler + Send + Sync>);
}

impl Builder {

	pub fn get_app(self) -> Application {
		self.app
	}

	pub fn new() -> Builder {
		Builder {
			app: Application {
				before: vec![],
				after: vec![],
				catch: vec![],
				endpoints: vec![]
			}
		}
	}
}

impl AfterMiddlewareSupport for Builder {
	fn using(&mut self, middleware: Box<AfterMiddleware + Send + Sync>) {
		self.app.after.push(middleware);
	}
}

impl BeforeMiddlewareSupport for Builder {
	fn using(&mut self, middleware: Box<BeforeMiddleware + Send + Sync>) {
		self.app.before.push(middleware);
	}
}

impl RunHandlerSupport for Builder {
	fn run(&mut self, handler: Box<Handler + Send + Sync>) {
		self.app.endpoints.push(handler);
	}
}
