use rustless::server::status;
use rustless::{self, Nesting};

#[test]
fn it_allows_prefix() {

    let app = app!(|api| {
        api.prefix("api");
        edp_stub!(api);
    });

    let response = call_app!(app, Get, "http://127.0.0.1:3000/info").unwrap();
    // not found because prefix is not present
    assert_eq!(response.status, status::StatusCode::NotFound);

    let response = call_app!(app, Get, "http://127.0.0.1:3000/api/info").unwrap();
    assert_eq!(response.status, status::StatusCode::Ok);
}

#[test]
fn it_allows_nested_prefix() {

    let app = app!(|api| {
        api.prefix("api");
        api.mount(rustless::Api::build(|nested_api| {
            nested_api.prefix("nested_api");
            edp_stub!(nested_api);
        }))
    });

    let response = call_app!(app, Get, "http://127.0.0.1:3000/info").unwrap();
    assert_eq!(response.status, status::StatusCode::NotFound);

    let response = call_app!(app, Get, "http://127.0.0.1:3000/api/info").unwrap();
    assert_eq!(response.status, status::StatusCode::NotFound);

    let response = call_app!(app, Get, "http://127.0.0.1:3000/api/nested_api/info").unwrap();
    assert_eq!(response.status, status::StatusCode::Ok);
}

#[test]
fn it_allows_prefix_with_path_versioning() {

    let app = app!(|api| {
        api.prefix("api");
        api.version("v1", rustless::Versioning::Path);
        api.mount(rustless::Api::build(|nested_api| {
            nested_api.prefix("nested_api");
            edp_stub!(nested_api);
        }))
    });

    let response = call_app!(app, Get, "http://127.0.0.1:3000/info").unwrap();
    assert_eq!(response.status, status::StatusCode::NotFound);

    let response = call_app!(app, Get, "http://127.0.0.1:3000/api/info").unwrap();
    assert_eq!(response.status, status::StatusCode::NotFound);

    let response = call_app!(app, Get, "http://127.0.0.1:3000/api/v1/info").unwrap();
    assert_eq!(response.status, status::StatusCode::NotFound);
}