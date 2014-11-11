
use anymap::AnyMap;
use server::{Request, Response};
use server_backend::status;
use errors::{Error, ErrorRefExt, NotMatchError, NotFoundError};
use {Extensible};

pub type HandleError = Box<Error>;
pub type HandleResult<T> = Result<T, HandleError>;
pub type HandleSuccessResult = HandleResult<()>;

pub trait BeforeMiddleware: Send + Sync {
    fn before(&self, &mut Request) -> HandleResult<Option<Response>>;
}

pub trait AfterMiddleware: Send + Sync {
    fn after(&self, &mut Request, &mut Response) -> HandleResult<Option<Response>>;
}

pub trait CatchMiddleware: Send + Sync {
    fn rescue(&self, &mut Request, err: &HandleError) -> HandleResult<Option<Response>>;
}

pub trait Handler: Send + Sync {
    fn call(&self, &str, &mut Request, &Application) -> HandleResult<Response>;
}

#[deriving(Send)]
pub struct Application {
    before: Vec<Box<BeforeMiddleware + Send + Sync>>,
    after: Vec<Box<AfterMiddleware + Send + Sync>>,
    catch: Vec<Box<CatchMiddleware + Send + Sync>>,
    handlers: Vec<Box<Handler + Send + Sync>>,
    pub ext: AnyMap
}

impl Application {

    pub fn new() -> Application {
        Application {
            before: vec![],
            after: vec![],
            catch: vec![],
            handlers: vec![],
            ext: AnyMap::new()
        }
    }

    pub fn call(&self, req: &mut Request) -> HandleResult<Response> {

        for mdw in self.before.iter() {
            match mdw.before(req) {
                Err(err) => {
                    match self.handle_error(req, err) {
                        Some(response) => return Ok(response),
                        None => ()
                    }
                },
                Ok(Some(response)) => return Ok(response),
                Ok(None) => ()
            }
        }

        let mut response: Option<Response> = None;
        let path = req.url().serialize_path().unwrap_or(String::new());

        for handler in self.handlers.iter() {
            match handler.call(path.as_slice(), req, self) {
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
            None => return Ok(self.handle_error(req, box NotFoundError).unwrap())
        };

        for mdw in self.after.iter() {
            match mdw.after(req, &mut exact_response) {
                Err(err) => match self.handle_error(req, err) {
                    Some(response) => return Ok(response),
                    None => ()
                },
                Ok(Some(response)) => return Ok(response),
                Ok(None) => ()
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
                Err(_) => ()
            }
        }

        if err.downcast::<NotFoundError>().is_some() {
            Some(Response::from_string(status::NotFound, "".to_string()))  
        } else {
            Some(Response::from_string(status::InternalServerError, format!("{}", err)))    
        }
    }

    pub fn mount(&mut self, handler: Box<Handler + Send + Sync>) {
        self.handlers.push(handler);
    }

    pub fn before<B>(&mut self, middleware: B) where B: BeforeMiddleware {
        self.before.push(box middleware as Box<BeforeMiddleware + Send + Sync>);
    }

    pub fn after<A>(&mut self, middleware: A) where A: AfterMiddleware {
        self.after.push(box middleware as Box<AfterMiddleware + Send + Sync>);
    }

    pub fn using<B, A>(&mut self, middlewares: (B, A)) where B: BeforeMiddleware, A: AfterMiddleware {
        let (before, after) = middlewares;
        self.before.push(box before as Box<BeforeMiddleware + Send + Sync>);
        self.after.push(box after as Box<AfterMiddleware + Send + Sync>);
    }

    pub fn catch<C>(&mut self, middleware: C) where C: CatchMiddleware {
        self.catch.push(box middleware as Box<CatchMiddleware + Send + Sync>);
    }
}

impl_extensible!(Application)

