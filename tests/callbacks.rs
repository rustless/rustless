use url::Url;
use serialize::json::{JsonObject};
use rustless::server_backend::method::{Get};
use rustless::server_backend::status;
use rustless::errors::{Error, ErrorRefExt};
use rustless::{
    Application, Api, Client, Media, Nesting, HandleResult, HandleSuccessResult, SimpleRequest, Response
};

#[deriving(Show)]
pub struct UnauthorizedError;

impl Error for UnauthorizedError {
    fn name(&self) -> &'static str {
        return "Unauthorized";
    }
}

#[test]
fn it_allows_to_create_namespace() {

    let app = app!(|api| {
        api.prefix("api");

        rescue_from!(api, UnauthorizedError, |_err, _media| {
            Some(Response::from_string(status::Unauthorized, "Please provide correct `token` parameter".to_string()))
        });

        api.namespace("admin", |admin_ns| {
            admin_ns.before(callback!(|_client, params| {
                match params.find(&"token".to_string()) {
                    Some(token) => if token.as_string().unwrap_or("").as_slice() == "password1" { return Ok(()) },
                    None => ()
                }

                return Err(UnauthorizedError.erase())
            }));

            // This endpoint is secure
            edp_stub!(admin_ns)
        })
    });

    let response = call_app!(app, Get, "http://127.0.0.1:3000/api/admin/info").unwrap();
    assert_eq!(response.status, status::Unauthorized);

    let response = call_app!(app, Get, "http://127.0.0.1:3000/api/admin/info?token=wrong%20token").unwrap();
    assert_eq!(response.status, status::Unauthorized);

    let response = call_app!(app, Get, "http://127.0.0.1:3000/api/admin/info?token=password1").unwrap();
    assert_eq!(response.status, status::Ok);

}
