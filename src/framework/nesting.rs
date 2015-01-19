use serialize::json::Object;

use backend::{Request, Response};
use server::method::Method::{Get, Post, Put, Delete};
use backend::{HandleResult, HandleSuccessResult};
use errors::{NotMatchError};

use errors::Error;

use framework::endpoint::{Endpoint, EndpointHandlerPresent};
use framework::{ApiHandler, ApiHandlers, Callbacks, CallInfo};
use framework::namespace::Namespace;
use framework::client::{Client};

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

    fn push_callbacks(&self, _info: &mut CallInfo) {
        // for cb in self.get_before().iter() { info.before.push(cb); }
        // for cb in self.get_before_validation().iter() { info.before_validation.push(cb); }
        // for cb in self.get_after_validation().iter() { info.after_validation.push(cb); }
        // for cb in self.get_after().iter() { info.after.push(cb); }
    }

    fn mount<H>(&mut self, edp: H) where H: ApiHandler + Send+Sync {
        self.get_handlers_mut().push(Box::new(edp))
    }

    /* 
     * Namespace aliases
     */

    fn namespace<F>(&mut self, path: &str, builder: F) where F: Fn(&mut Namespace) {
        self.mount(Namespace::build(path, builder));
    }
    fn group<F>(&mut self, path: &str, builder: F) where F: Fn(&mut Namespace) {
        self.mount(Namespace::build(path, builder));
    }
    fn resource<F>(&mut self, path: &str, builder: F) where F: Fn(&mut Namespace) {
        self.mount(Namespace::build(path, builder));
    }
    fn resources<F>(&mut self, path: &str, builder: F) where F: Fn(&mut Namespace) {
        self.mount(Namespace::build(path, builder));
    }
    fn segment<F>(&mut self, path: &str, builder: F) where F: Fn(&mut Namespace) {
        self.mount(Namespace::build(path, builder));
    }

    /* 
     * Endpoints
     */

    fn get<F>(&mut self, path: &str, builder: F) where F: Fn(&mut Endpoint) -> EndpointHandlerPresent {
        self.mount(Endpoint::build(Get, path, builder));
    }    
    fn post<F>(&mut self, path: &str, builder: F) where F: Fn(&mut Endpoint) -> EndpointHandlerPresent {
        self.mount(Endpoint::build(Post, path, builder));
    }    
    fn put<F>(&mut self, path: &str, builder: F) where F: Fn(&mut Endpoint) -> EndpointHandlerPresent {
        self.mount(Endpoint::build(Put, path, builder));
    }    
    fn delete<F>(&mut self, path: &str, builder: F) where F: Fn(&mut Endpoint) -> EndpointHandlerPresent {
        self.mount(Endpoint::build(Delete, path, builder));
    }

    fn before<F>(&mut self, callback: F) where F: for<'a> Fn(&'a mut Client, &Object) -> HandleSuccessResult + Send+Sync { 
        self.get_before_mut().push(Box::new(callback)); 
    }
    fn before_validation<F>(&mut self, callback: F) where F: for<'a> Fn(&'a mut Client, &Object) -> HandleSuccessResult + Send+Sync { 
        self.get_before_validation_mut().push(Box::new(callback)); 
    }
    fn after<F>(&mut self, callback: F) where F: for<'a> Fn(&'a mut Client, &Object) -> HandleSuccessResult + Send+Sync { 
        self.get_after_mut().push(Box::new(callback));
    }
    fn after_validation<F>(&mut self, callback: F) where F: for<'a> Fn(&'a mut Client, &Object) -> HandleSuccessResult + Send+Sync { 
        self.get_after_validation_mut().push(Box::new(callback)); 
    }

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