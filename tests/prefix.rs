use url::Url;
use serialize::json::{JsonObject};
use rustless::server_backend::method::{Get};
use rustless::server_backend::status;
use rustless::{
    Application, Api, Client, Nesting, HandleResult, SimpleRequest, PathVersioning
};

#[test]
fn it_allows_prefix() {

    let app = app!(|api| {
        api.prefix("api");
        edp_stub!(api);
    });

    let response = call_app!(app, Get, "http://127.0.0.1:3000/info").unwrap();
    // not found because prefix is not present
    assert_eq!(response.status, status::NotFound);

    let response = call_app!(app, Get, "http://127.0.0.1:3000/api/info").unwrap();
    assert_eq!(response.status, status::Ok);
}

#[test]
fn it_allows_nested_prefix() {

    let app = app!(|api| {
        api.prefix("api");
        api.mount(box Api::build(|nested_api| {
            nested_api.prefix("nested_api");
            edp_stub!(nested_api);
        }))
    });

    let response = call_app!(app, Get, "http://127.0.0.1:3000/info").unwrap();
    assert_eq!(response.status, status::NotFound);

    let response = call_app!(app, Get, "http://127.0.0.1:3000/api/info").unwrap();
    assert_eq!(response.status, status::NotFound);

    let response = call_app!(app, Get, "http://127.0.0.1:3000/api/nested_api/info").unwrap();
    assert_eq!(response.status, status::Ok);
}

#[test]
fn it_allows_prefix_with_path_versioning() {

    let app = app!(|api| {
        api.prefix("api");
        api.version("v1", PathVersioning);
        api.mount(box Api::build(|nested_api| {
            nested_api.prefix("nested_api");
            edp_stub!(nested_api);
        }))
    });

    let response = call_app!(app, Get, "http://127.0.0.1:3000/info").unwrap();
    assert_eq!(response.status, status::NotFound);

    let response = call_app!(app, Get, "http://127.0.0.1:3000/api/info").unwrap();
    assert_eq!(response.status, status::NotFound);

    let response = call_app!(app, Get, "http://127.0.0.1:3000/api/v1/info").unwrap();
    assert_eq!(response.status, status::NotFound);
}