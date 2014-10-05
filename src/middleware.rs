
use request::Request;
use response::Response;

use std::fmt::Show;
pub use error::{Error, ErrorRefExt};
use hyper::status;

#[deriving(Show)]
pub struct SimpleError {
    pub name: &'static str
}

impl Error for SimpleError {
    fn name(&self) -> &'static str {
        return self.name;
    }
}

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


pub type HandleError = Box<Error>;
pub type HandleResult<'a, T> = Result<T, HandleError>;
pub type HandleSuccessResult<'a> = HandleResult<'a, ()>;

pub trait BeforeMiddleware: Send + Sync {
    fn before(&self, &mut Request) -> HandleSuccessResult;
}

pub trait AfterMiddleware: Send + Sync {
    fn after(&self, &mut Request, &mut Response) -> HandleSuccessResult;
}

pub trait CatchMiddleware: Send + Sync {
    fn rescue(&self, &mut Request, err: &HandleError) -> HandleResult<Option<Response>>;
}

pub trait Handler: Send + Sync {
    fn call(&self, &mut Request) -> HandleResult<Response>;
}

#[deriving(Send)]
pub struct Application {
    before: Vec<Box<BeforeMiddleware + Send + Sync>>,
    after: Vec<Box<AfterMiddleware + Send + Sync>>,
    catch: Vec<Box<CatchMiddleware + Send + Sync>>,
    handlers: Vec<Box<Handler + Send + Sync>>
}

impl Application {

    pub fn call(&self, req: &mut Request) -> HandleResult<Response> {

        for mdw in self.before.iter() {
            match mdw.before(req) {
                Err(err) => {
                    match self.handle_error(req, err) {
                        Some(response) => return Ok(response),
                        None => ()
                    }
                },
                Ok(()) => ()
            }
        }

        let mut response: Option<Response> = None;

        for handler in self.handlers.iter() {
            match handler.call(req) {
                Ok(resp) => response = Some(resp),
                Err(err) => match err.downcast::<NotMatchError>() {
                    Some(_) => (),
                    None => match self.handle_error(req, err) {
                        Some(response) => return Ok(response),
                        None => ()
                    }
                }
            }
        }

        let mut exact_response = match response {
            Some(resp) => resp,
            None => return Ok(self.handle_error(req, NotFoundError.abstract()).unwrap())
        };

        for mdw in self.after.iter() {
            match mdw.after(req, &mut exact_response) {
                Err(err) => match self.handle_error(req, err) {
                    Some(response) => return Ok(response),
                    None => ()
                },
                Ok(()) => ()
            }
        }

        Ok(exact_response)

    }

    pub fn handle_error(&self, req: &mut Request, err: HandleError) -> Option<Response> {
        for catcher in self.catch.iter() {
            match catcher.rescue(req, &err) {
                Ok(maybe_response) => {
                    match maybe_response {
                        Some(resp) => return Some(resp),
                        None => return None
                    }
                },
                Err(some) => ()
            }
        }

        let error_message = format!("{}", err);
        Some(Response::from_string(status::InternalServerError, error_message))
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
                handlers: vec![]
            }
        }
    }

    pub fn mount(&mut self, handler: Box<Handler + Send + Sync>) {
        self.app.handlers.push(handler);
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

