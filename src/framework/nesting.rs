use serialize::json::Object;

use backend::{Request, Response};
use server::method::Method::{Get, Post, Put, Delete};
use backend::{HandleResult};
use errors::{NotMatchError};

use errors::Error;

use framework::endpoint::{Endpoint, EndpointHandlerPresent};
use framework::{ApiHandler, ApiHandlers, Callback, Callbacks, CallInfo};
use framework::namespace::Namespace;

pub trait Nesting {

    fn get_handlers<'a>(&'a self) -> &'a ApiHandlers;
    fn get_handlers_mut<'a>(&'a mut self) -> &'a mut ApiHandlers;

    fn get_before<'a>(&'a self) -> &'a Callbacks;
    fn get_before_mut<'a>(&'a mut self) -> &'a mut Callbacks;

    fn get_before_validation<'a>(&'a self) -> &'a Callbacks;
    fn get_before_validation_mut<'a>(&'a mut self) -> &'a mut Callbacks;

    fn get_after_validation<'a>(&'a self) -> &'a Callbacks;
    fn get_after_validation_mut<'a>(&'a mut self) -> &'a mut Callbacks;

    fn get_after<'a>(&'a self) -> &'a Callbacks;
    fn get_after_mut<'a>(&'a mut self) -> &'a mut Callbacks;

    fn push_callbacks<'a>(&'a self, info: &mut CallInfo<'a>) {
        for cb in self.get_before().iter() { info.before.push(cb); }
        for cb in self.get_before_validation().iter() { info.before_validation.push(cb); }
        for cb in self.get_after_validation().iter() { info.after_validation.push(cb); }
        for cb in self.get_after().iter() { info.after.push(cb); }
    }

    fn mount(&mut self, edp: Box<ApiHandler + Send + Sync>) {
        self.get_handlers_mut().push(edp)
    }

    /* 
     * Namespace aliases
     */

    fn namespace<F>(&mut self, path: &str, builder: F) where F: Fn(&mut Namespace) {
        self.mount(Box::new(Namespace::build(path, builder)));
    }
    fn group<F>(&mut self, path: &str, builder: F) where F: Fn(&mut Namespace) {
        self.mount(Box::new(Namespace::build(path, builder)));
    }
    fn resource<F>(&mut self, path: &str, builder: F) where F: Fn(&mut Namespace) {
        self.mount(Box::new(Namespace::build(path, builder)));
    }
    fn resources<F>(&mut self, path: &str, builder: F) where F: Fn(&mut Namespace) {
        self.mount(Box::new(Namespace::build(path, builder)));
    }
    fn segment<F>(&mut self, path: &str, builder: F) where F: Fn(&mut Namespace) {
        self.mount(Box::new(Namespace::build(path, builder)));
    }

    /* 
     * Endpoints
     */

    fn get<F>(&mut self, path: &str, builder: F) where F: Fn(&mut Endpoint) -> EndpointHandlerPresent {
        self.mount(Box::new(Endpoint::build(Get, path, builder)));
    }    
    fn post<F>(&mut self, path: &str, builder: F) where F: Fn(&mut Endpoint) -> EndpointHandlerPresent {
        self.mount(Box::new(Endpoint::build(Post, path, builder)));
    }    
    fn put<F>(&mut self, path: &str, builder: F) where F: Fn(&mut Endpoint) -> EndpointHandlerPresent {
        self.mount(Box::new(Endpoint::build(Put, path, builder)));
    }    
    fn delete<F>(&mut self, path: &str, builder: F) where F: Fn(&mut Endpoint) -> EndpointHandlerPresent {
        self.mount(Box::new(Endpoint::build(Delete, path, builder)));
    }

    fn before(&mut self, callback: Callback) { self.get_before_mut().push(callback); }
    fn before_validation(&mut self, callback: Callback) { self.get_before_validation_mut().push(callback); }
    fn after(&mut self, callback: Callback) { self.get_after_mut().push(callback); }
    fn after_validation(&mut self, callback: Callback) { self.get_after_validation_mut().push(callback); }

    fn call_handlers(&self, rest_path: &str, params: &mut Object, req: &mut Request, info: &mut CallInfo) -> HandleResult<Response> {
        for handler in self.get_handlers().iter() {
            match handler.api_call(rest_path, params, req, info) {
                Ok(response) => return Ok(response),
                Err(err) => {
                    match err.downcast::<NotMatchError>() {
                        Some(_) => (),
                        None => return Err(err),
                    }
                }
            };
        }

        Err(Box::new(NotMatchError) as Box<Error>)
    }

}