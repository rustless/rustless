use rustless;
use rustless::errors;
use rustless::prelude::*;
use rustless::server::status;
use jsonway;
use rustc_serialize::json::ToJson;

mod tweets;

pub fn root() -> rustless::Api {
    rustless::Api::build(|api| {
        api.prefix("api");
        api.version("v1", rustless::Versioning::Path);

        // Add error formatter to send validation errors back to the client
        api.error_formatter(|error, _media| {
            if error.is::<errors::Validation>() {
                let val_err = error.downcast::<errors::Validation>().unwrap();
                return Some(rustless::Response::from_json(status::StatusCode::BadRequest, &jsonway::object(|json| {
                    json.set_json("errors", val_err.reason.to_json())
                }).unwrap()))
            }

            None
        });

        api.mount(tweets::tweets("tweets"));
    })
}
