use serialize::json::JsonObject;

use server::{Request, Response};
use server_backend::method::{Get, Post, Put, Delete};
use middleware::{HandleResult};
use errors::{NotMatchError, Error, ErrorRefExt};

use framework::endpoint::{Endpoint, EndpointBuilder};
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

    fn push_callbacks(&self, info: &mut CallInfo) {
        for cb in self.get_before().iter() { info.before.push(*cb); }
        for cb in self.get_before_validation().iter() { info.before_validation.push(*cb); }
        for cb in self.get_after_validation().iter() { info.after_validation.push(*cb); }
        for cb in self.get_after().iter() { info.after.push(*cb); }
    }

    fn mount(&mut self, edp: Box<ApiHandler + Send + Sync>) {
        self.get_handlers_mut().push(edp)
    }

    /* 
     * Namespace aliases
     */

    fn namespace(&mut self, path: &str, builder: |&mut Namespace|) {
        self.mount(box Namespace::build(path, builder));
    }
    fn group(&mut self, path: &str, builder: |&mut Namespace|) {
        self.mount(box Namespace::build(path, builder));
    }
    fn resource(&mut self, path: &str, builder: |&mut Namespace|) {
        self.mount(box Namespace::build(path, builder));
    }
    fn resources(&mut self, path: &str, builder: |&mut Namespace|) {
        self.mount(box Namespace::build(path, builder));
    }
    fn segment(&mut self, path: &str, builder: |&mut Namespace|) {
        self.mount(box Namespace::build(path, builder));
    }

    /* 
     * Endpoints
     */

    fn get(&mut self, path: &str, builder: EndpointBuilder) {
        self.mount(box Endpoint::build(Get, path, builder));
    }    
    fn post(&mut self, path: &str, builder: EndpointBuilder) {
        self.mount(box Endpoint::build(Post, path, builder));
    }    
    fn put(&mut self, path: &str, builder: EndpointBuilder) {
        self.mount(box Endpoint::build(Put, path, builder));
    }    
    fn delete(&mut self, path: &str, builder: EndpointBuilder) {
        self.mount(box Endpoint::build(Delete, path, builder));
    }

    fn before(&mut self, callback: Callback) { self.get_before_mut().push(callback); }
    fn before_validation(&mut self, callback: Callback) { self.get_before_validation_mut().push(callback); }
    fn after(&mut self, callback: Callback) { self.get_after_mut().push(callback); }
    fn after_validation(&mut self, callback: Callback) { self.get_after_validation_mut().push(callback); }

    fn call_handlers(&self, rest_path: &str, params: &mut JsonObject, req: &mut Request, info: &mut CallInfo) -> HandleResult<Response> {
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

        Err(NotMatchError.abstract())
    }

}