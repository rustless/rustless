use framework;
use framework::namespace;
use framework::endpoint;
use framework::client;
use json::{JsonValue};
use backend;
use server::method;
use errors;

pub trait Node {
    fn get_handlers<'a>(&'a self) -> &'a framework::ApiHandlers;
    fn get_handlers_mut<'a>(&'a mut self) -> &'a mut framework::ApiHandlers;

    fn get_before<'a>(&'a self) -> &'a framework::Callbacks;
    fn get_before_mut<'a>(&'a mut self) -> &'a mut framework::Callbacks;

    fn get_before_validation<'a>(&'a self) -> &'a framework::Callbacks;
    fn get_before_validation_mut<'a>(&'a mut self) -> &'a mut framework::Callbacks;

    fn get_after_validation<'a>(&'a self) -> &'a framework::Callbacks;
    fn get_after_validation_mut<'a>(&'a mut self) -> &'a mut framework::Callbacks;

    fn get_after<'a>(&'a self) -> &'a framework::Callbacks;
    fn get_after_mut<'a>(&'a mut self) -> &'a mut framework::Callbacks;

    fn push_node<'a>(&'a self, _info: &mut framework::CallInfo<'a>);
}

#[macro_export]
macro_rules! impl_nesting {
    ($t:ident) => (
        impl nesting::Node for $t {
            fn get_handlers<'a>(&'a self) -> &'a ::framework::ApiHandlers { &self.handlers }
            fn get_handlers_mut<'a>(&'a mut self) -> &'a mut ::framework::ApiHandlers { &mut self.handlers }

            fn get_before<'a>(&'a self) -> &'a ::framework::Callbacks { &self.before }
            fn get_before_mut<'a>(&'a mut self) -> &'a mut ::framework::Callbacks { &mut self.before }

            fn get_before_validation<'a>(&'a self) -> &'a ::framework::Callbacks { &self.before_validation }
            fn get_before_validation_mut<'a>(&'a mut self) -> &'a mut ::framework::Callbacks { &mut self.before_validation }

            fn get_after_validation<'a>(&'a self) -> &'a ::framework::Callbacks { &self.after_validation }
            fn get_after_validation_mut<'a>(&'a mut self) -> &'a mut ::framework::Callbacks { &mut self.after_validation }

            fn get_after<'a>(&'a self) -> &'a ::framework::Callbacks { &self.after }
            fn get_after_mut<'a>(&'a mut self) -> &'a mut ::framework::Callbacks { &mut self.after }

            fn push_node<'a>(&'a self, _info: &mut ::framework::CallInfo<'a>) {
                _info.parents.push(self);
            }
        }

        impl ::framework::nesting::Nesting for $t {}
    )
}

pub trait Nesting: Node {

    fn mount<H>(&mut self, edp: H) where H: framework::ApiHandler + Send+Sync {
        self.get_handlers_mut().push(Box::new(edp))
    }

    /*
     * namespace::Namespace aliases
     */

    fn namespace<F>(&mut self, path: &str, builder: F) where F: FnOnce(&mut namespace::Namespace) {
        self.mount(namespace::Namespace::build(path, builder));
    }
    fn group<F>(&mut self, path: &str, builder: F) where F: FnOnce(&mut namespace::Namespace) {
        self.mount(namespace::Namespace::build(path, builder));
    }
    fn resource<F>(&mut self, path: &str, builder: F) where F: FnOnce(&mut namespace::Namespace) {
        self.mount(namespace::Namespace::build(path, builder));
    }
    fn resources<F>(&mut self, path: &str, builder: F) where F: FnOnce(&mut namespace::Namespace) {
        self.mount(namespace::Namespace::build(path, builder));
    }
    fn segment<F>(&mut self, path: &str, builder: F) where F: FnOnce(&mut namespace::Namespace) {
        self.mount(namespace::Namespace::build(path, builder));
    }

    /*
     * endpoint::Endpoints
     */

    fn get<F>(&mut self, path: &str, builder: F) where F: FnOnce(&mut endpoint::Endpoint)
    -> endpoint::EndpointHandlerPresent {
        self.mount(endpoint::Endpoint::build(method::Method::Get, path, builder));
    }
    fn post<F>(&mut self, path: &str, builder: F) where F: FnOnce(&mut endpoint::Endpoint)
    -> endpoint::EndpointHandlerPresent {
        self.mount(endpoint::Endpoint::build(method::Method::Post, path, builder));
    }
    fn put<F>(&mut self, path: &str, builder: F) where F: FnOnce(&mut endpoint::Endpoint)
    -> endpoint::EndpointHandlerPresent {
        self.mount(endpoint::Endpoint::build(method::Method::Put, path, builder));
    }
    fn delete<F>(&mut self, path: &str, builder: F) where F: FnOnce(&mut endpoint::Endpoint)
    -> endpoint::EndpointHandlerPresent {
        self.mount(endpoint::Endpoint::build(method::Method::Delete, path, builder));
    }
    fn options<F>(&mut self, path: &str, builder: F) where F: FnOnce(&mut endpoint::Endpoint)
    -> endpoint::EndpointHandlerPresent {
        self.mount(endpoint::Endpoint::build(method::Method::Options, path, builder));
    }
    fn head<F>(&mut self, path: &str, builder: F) where F: FnOnce(&mut endpoint::Endpoint)
    -> endpoint::EndpointHandlerPresent {
        self.mount(endpoint::Endpoint::build(method::Method::Head, path, builder));
    }
    fn patch<F>(&mut self, path: &str, builder: F) where F: FnOnce(&mut endpoint::Endpoint)
    -> endpoint::EndpointHandlerPresent {
        self.mount(endpoint::Endpoint::build(method::Method::Patch, path, builder));
    }

    fn before<F: 'static>(&mut self, callback: F) where F: for<'a> Fn(&'a mut client::Client, &JsonValue)
    -> backend::HandleSuccessResult + Send+Sync {
        self.get_before_mut().push(Box::new(callback));
    }
    fn before_validation<F: 'static>(&mut self, callback: F) where F: for<'a> Fn(&'a mut client::Client, &JsonValue)
    -> backend::HandleSuccessResult + Send+Sync {
        self.get_before_validation_mut().push(Box::new(callback));
    }
    fn after<F: 'static>(&mut self, callback: F) where F: for<'a> Fn(&'a mut client::Client, &JsonValue)
    -> backend::HandleSuccessResult + Send+Sync {
        self.get_after_mut().push(Box::new(callback));
    }
    fn after_validation<F: 'static>(&mut self, callback: F) where F: for<'a> Fn(&'a mut client::Client, &JsonValue)
    -> backend::HandleSuccessResult + Send+Sync {
        self.get_after_validation_mut().push(Box::new(callback));
    }

    fn call_handlers<'a, 'r>(&'a self, rest_path: &str, params: &mut JsonValue, req: &'r mut (backend::Request + 'r),
                         info: &mut framework::CallInfo<'a>) -> backend::HandleResult<backend::Response> {

        for handler in self.get_handlers().iter() {
            match handler.api_call(rest_path, params, req, info) {
                Ok(response) => return Ok(response),
                Err(error_response) => {
                    if !errors::Error::is::<errors::NotMatch>(&*error_response.error) {
                        return Err(error_response)
                    }
                }
            };
        }

        Err(error_response!(errors::NotMatch))
    }

}