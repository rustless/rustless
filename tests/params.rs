use url::Url;
use serialize::json::{JsonObject};
use rustless::server_backend::method::{Get};
use rustless::server_backend::status;
use rustless::{
    Application, Api, Client, Valico, Nesting, HandleResult, SimpleRequest
};

use rustless::errors::{Error};

#[test]
fn it_validates_endpoint_simple_path_params() {

    let app = app!(|api| {
        api.prefix("api");

        api.get("users/:user_id/messages/:message_id", |endpoint| {
            edp_stub_handler!(endpoint)
        })
    });

    let response = call_app!(app, Get, "http://127.0.0.1:3000/api/users").unwrap();
    assert_eq!(response.status, status::NotFound);

    let response = call_app!(app, Get, "http://127.0.0.1:3000/api/users/").unwrap();
    assert_eq!(response.status, status::NotFound);

    let response = call_app!(app, Get, "http://127.0.0.1:3000/api/users/Skywalker/messages/100").unwrap();
    assert_eq!(response.status, status::Ok);

    let response = call_app!(app, Get, "http://127.0.0.1:3000/api/users/100/messages/12").unwrap();
    assert_eq!(response.status, status::Ok);

}

#[test]
fn it_validates_typed_endpoint_path_params() {

    let app = app!(|api| {
        api.prefix("api");

        api.get("users/:user_id/messages/:message_id", |endpoint| {
            endpoint.params(|params| {
                params.req_typed("user_id", Valico::u64());
                params.req_typed("message_id", Valico::u64());
            });

            edp_stub_handler!(endpoint)
        })
    });

    let response = call_app!(app, Get, "http://127.0.0.1:3000/api/users/Skywalker/messages/100").unwrap();
    assert_eq!(response.status, status::BadRequest);    

    let response = call_app!(app, Get, "http://127.0.0.1:3000/api/users/100/messages/Skywalker").unwrap();
    assert_eq!(response.status, status::BadRequest);

    let response = call_app!(app, Get, "http://127.0.0.1:3000/api/users/100/messages/12").unwrap();
    assert_eq!(response.status, status::Ok);

}