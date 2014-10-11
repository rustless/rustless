use url::Url;
use serialize::json::{JsonObject};
use rustless::server_backend::method::{Get};
use rustless::server_backend::status;
use rustless::server_backend::header::common::Accept;
use rustless::{
    Application, Api, Client, Nesting, HandleResult, AcceptHeaderVersioning, 
    PathVersioning, ParamVersioning, SimpleRequest
};

use rustless::errors::{Error};

#[test]
fn it_pass_accept_header_versioning() {

    let app = app!(|api| {
        api.version("v1", AcceptHeaderVersioning("infoapi"));
        edp_stub!(api);
    });

    let response = call_app!(app, Get, "http://127.0.0.1:3000/info").unwrap();
    // not found because accept-header is not present
    assert_eq!(response.status, status::NotFound);

    let response = call_app!(app, Get, "http://127.0.0.1:3000/info", |rq| {
        rq.headers_mut().set(
            Accept( vec![mime!("application/vnd.infoapi.v1+json")] )
        );
    }).unwrap();

    assert_eq!(response.status, status::Ok);
}

#[test]
fn it_pass_path_versioning() {
    let app = app!(|api| {
        api.version("v1", PathVersioning);
        edp_stub!(api);
    });

    let response = call_app!(app, Get, "http://127.0.0.1:3000/info").unwrap();
    // not found because version is not present in path
    assert_eq!(response.status, status::NotFound);

    let response = call_app!(app, Get, "http://127.0.0.1:3000/v1/info").unwrap();
    assert_eq!(response.status, status::Ok);
}

#[test]
fn it_pass_param_versioning() {
    let app = app!(|api| {
        api.version("v1", ParamVersioning("v"));
        edp_stub!(api);
    });

    let response = call_app!(app, Get, "http://127.0.0.1:3000/info").unwrap();
    // not found because version is not present in param
    assert_eq!(response.status, status::NotFound);

    let response = call_app!(app, Get, "http://127.0.0.1:3000/info?v=v1").unwrap();
    assert_eq!(response.status, status::Ok);
}

#[test]
fn it_pass_nesting_param_versioning() {
    let app = app!(|api| {
        api.version("v1", ParamVersioning("v"));
        edp_stub!(api);

        api.mount(box Api::build(|nested_api| {
            nested_api.version("v2", ParamVersioning("nested_ver"));

            nested_api.get("nested_info", |endpoint| {
                edp_handler!(endpoint, |client, _params| {
                    client.text("Some usefull info".to_string())
                })
            });
        }))
    });

    let response = call_app!(app, Get, "http://127.0.0.1:3000/info?v=v1").unwrap();
    assert_eq!(response.status, status::Ok);

    let response = call_app!(app, Get, "http://127.0.0.1:3000/info/nested_info?v=v1").unwrap();
    // not found because nested_info param in not present
    assert_eq!(response.status, status::NotFound);

    let response = call_app!(app, Get, "http://127.0.0.1:3000/nested_info?v=v1&nested_ver=v2").unwrap();
    assert_eq!(response.status, status::Ok);
}

#[test]
fn it_pass_nesting_path_versioning() {
    let app = app!(|api| {
        api.version("v1", PathVersioning);
        edp_stub!(api);

        api.mount(box Api::build(|nested_api| {
            nested_api.version("v2", PathVersioning);

            nested_api.get("nested_info", |endpoint| {
                edp_handler!(endpoint, |client, _params| {
                    client.text("Some usefull info".to_string())
                })
            });
        }))
    });

    let response = call_app!(app, Get, "http://127.0.0.1:3000/v1/info").unwrap();
    assert_eq!(response.status, status::Ok);

    let response = call_app!(app, Get, "http://127.0.0.1:3000/v1/nested_info").unwrap();
    // not found because v2 in not present in path
    assert_eq!(response.status, status::NotFound);

    let response = call_app!(app, Get, "http://127.0.0.1:3000/v1/v2/nested_info").unwrap();
    assert_eq!(response.status, status::Ok);
}

#[test]
fn it_pass_nesting_crazy_mixed_versioning_never_do_this() {
    let app = app!(|api| {
        api.version("v1", AcceptHeaderVersioning("infoapi"));
        edp_stub!(api);

        api.mount(box Api::build(|nested_api| {
            nested_api.version("v2", PathVersioning);

            nested_api.mount(box Api::build(|nested_nested_api| {
                nested_nested_api.version("v3", ParamVersioning("ver"));
                edp_stub!(nested_nested_api);
            }))
        }))
    });

    let response = call_app!(app, Get, "http://127.0.0.1:3000/v2/info?ver=v3", |rq| {
        rq.headers_mut().set(
            Accept( vec![mime!("application/vnd.infoapi.v1+json")] )
        );
    }).unwrap();

    assert_eq!(response.status, status::Ok);

    let response = call_app!(app, Get, "http://127.0.0.1:3000/v2/nested_nested_info", |rq| {
        rq.headers_mut().set(
            Accept( vec![mime!("application/vnd.infoapi.v1+json")] )
        );
    }).unwrap();

    assert_eq!(response.status, status::NotFound);
}

#[test]
fn it_pass_without_versioning() {
    let app = app!(|api| {
        edp_stub!(api);
    });

    let response = call_app!(app, Get, "http://127.0.0.1:3000/info").unwrap();
    assert_eq!(response.status, status::Ok);
}