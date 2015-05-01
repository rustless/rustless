use std::error;
use std::error::Error as StdError;
use std::fmt;
use valico::json_dsl;
use rustless::{self, Nesting};
use rustless::server::status;
use rustless::errors::{Error};

#[derive(Debug)]
pub struct UnauthorizedError;

impl error::Error for UnauthorizedError {
    fn description(&self) -> &'static str {
        return "Unauthorized";
    }
}

impl fmt::Display for UnauthorizedError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        self.description().fmt(formatter)
    }
}

#[test]
fn it_invokes_callbacks() {

    let app = app!(|api| {
        api.prefix("api");

        api.error_formatter(|err, _media| {
            if err.is::<UnauthorizedError>() {
                Some(rustless::Response::from_string(status::StatusCode::Unauthorized, "Please provide correct `token` parameter".to_string()))
            } else {
                None
            }
        });

        api.namespace("admin", |admin_ns| {

            admin_ns.params(|params| {
                params.req_typed("token", json_dsl::string())
            });

            // Using after_validation callback to check token
            admin_ns.after_validation(|_client, params| {
                match params.find(&"token".to_string()) {
                    // We can.unwrap() safely because token in validated already
                    Some(token) => if token.as_string().unwrap() == "password1" { return Ok(()) },
                    None => ()
                }

                // Fire error from callback is token is wrong
                return Err(rustless::ErrorResponse{
                    error: Box::new(UnauthorizedError) as Box<Error + Send>,
                    response: None
                })
            });

            // This `/api/admin/server_status` endpoint is secure now
            admin_ns.get("server_status", |endpoint| {

                endpoint.handle(|client, _params| {
                    client.text("Everything is OK".to_string())
                })
            });
        })
    });

    let err_resp = call_app!(app, Get, "http://127.0.0.1:3000/api/admin/server_status").err().unwrap();
    assert_eq!(err_resp.response.status, status::StatusCode::BadRequest);

    let err_resp = call_app!(app, Get, "http://127.0.0.1:3000/api/admin/server_status?token=wrong%20token").err().unwrap();
    assert_eq!(err_resp.response.status, status::StatusCode::Unauthorized);

    let response = call_app!(app, Get, "http://127.0.0.1:3000/api/admin/server_status?token=password1").ok().unwrap();
    assert_eq!(response.status, status::StatusCode::Ok);

}
