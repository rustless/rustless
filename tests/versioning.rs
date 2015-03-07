use rustless::server::status;
use rustless::server::header;
use rustless::{self, Nesting};

#[test]
fn it_pass_accept_header_versioning() {

    let app = app!(|api| {
        api.version("v1", rustless::Versioning::AcceptHeader("infoapi"));
        edp_stub!(api);
    });

    let err_resp = call_app!(app, Get, "http://127.0.0.1:3000/info").err().unwrap();
    // not found because accept-header is not present
    assert_eq!(err_resp.response.status, status::StatusCode::NotFound);

    let response = call_app!(app, Get, "http://127.0.0.1:3000/info", |rq| {
        rq.headers_mut().set(
            header::Accept( vec![mime!("application/vnd.infoapi.v1+json")] )
        );
    }).ok().unwrap();

    assert_eq!(response.status, status::StatusCode::Ok);
}

#[test]
fn it_pass_path_versioning() {
    let app = app!(|api| {
        api.version("v1", rustless::Versioning::Path);
        edp_stub!(api);
    });

    let err_resp = call_app!(app, Get, "http://127.0.0.1:3000/info").err().unwrap();
    // not found because version is not present in path
    assert_eq!(err_resp.response.status, status::StatusCode::NotFound);

    let response = call_app!(app, Get, "http://127.0.0.1:3000/v1/info").ok().unwrap();
    assert_eq!(response.status, status::StatusCode::Ok);
}

#[test]
fn it_pass_param_versioning() {
    let app = app!(|api| {
        api.version("v1", rustless::Versioning::Param("v"));
        edp_stub!(api);
    });

    let err_resp = call_app!(app, Get, "http://127.0.0.1:3000/info").err().unwrap();
    // not found because version is not present in param
    assert_eq!(err_resp.response.status, status::StatusCode::NotFound);

    let response = call_app!(app, Get, "http://127.0.0.1:3000/info?v=v1").ok().unwrap();
    assert_eq!(response.status, status::StatusCode::Ok);
}

#[test]
fn it_pass_nesting_param_versioning() {
    let app = app!(|api| {
        api.version("v1", rustless::Versioning::Param("v"));
        edp_stub!(api);

        api.mount(rustless::Api::build(|nested_api| {
            nested_api.version("v2", rustless::Versioning::Param("nested_ver"));

            nested_api.get("nested_info", |endpoint| {
                endpoint.handle(|client, _params| {
                    client.text("Some usefull info".to_string())
                })
            });
        }))
    });

    let response = call_app!(app, Get, "http://127.0.0.1:3000/info?v=v1").ok().unwrap();
    assert_eq!(response.status, status::StatusCode::Ok);

    let err_resp = call_app!(app, Get, "http://127.0.0.1:3000/nested_info?v=v1").err().unwrap();
    // not found because nested_info param in not present
    assert_eq!(err_resp.response.status, status::StatusCode::NotFound);

    let response = call_app!(app, Get, "http://127.0.0.1:3000/nested_info?v=v1&nested_ver=v2").ok().unwrap();
    assert_eq!(response.status, status::StatusCode::Ok);
}

#[test]
fn it_pass_nesting_path_versioning() {
    let app = app!(|api| {
        api.version("v1", rustless::Versioning::Path);
        edp_stub!(api);

        api.mount(rustless::Api::build(|nested_api| {
            nested_api.version("v2", rustless::Versioning::Path);

            nested_api.get("nested_info", |endpoint| {
                endpoint.handle(|client, _params| {
                    client.text("Some usefull info".to_string())
                })
            });
        }))
    });

    let response = call_app!(app, Get, "http://127.0.0.1:3000/v1/info").ok().unwrap();
    assert_eq!(response.status, status::StatusCode::Ok);

    let err_resp = call_app!(app, Get, "http://127.0.0.1:3000/v1/nested_info").err().unwrap();
    // not found because v2 in not present in path
    assert_eq!(err_resp.response.status, status::StatusCode::NotFound);

    let response = call_app!(app, Get, "http://127.0.0.1:3000/v1/v2/nested_info").ok().unwrap();
    assert_eq!(response.status, status::StatusCode::Ok);
}

#[test]
fn it_pass_nesting_crazy_mixed_versioning_never_do_this() {
    let app = app!(|api| {
        api.version("v1", rustless::Versioning::AcceptHeader("infoapi"));
        edp_stub!(api);

        api.mount(rustless::Api::build(|nested_api| {
            nested_api.version("v2", rustless::Versioning::Path);

            nested_api.mount(rustless::Api::build(|nested_nested_api| {
                nested_nested_api.version("v3", rustless::Versioning::Param("ver"));
                edp_stub!(nested_nested_api);
            }))
        }))
    });

    let response = call_app!(app, Get, "http://127.0.0.1:3000/v2/info?ver=v3", |rq| {
        rq.headers_mut().set(
            header::Accept( vec![mime!("application/vnd.infoapi.v1+json")] )
        );
    }).ok().unwrap();

    assert_eq!(response.status, status::StatusCode::Ok);

    let err_resp = call_app!(app, Get, "http://127.0.0.1:3000/v2/nested_nested_info", |rq| {
        rq.headers_mut().set(
            header::Accept( vec![mime!("application/vnd.infoapi.v1+json")] )
        );
    }).err().unwrap();

    assert_eq!(err_resp.response.status, status::StatusCode::NotFound);
}

#[test]
fn it_pass_without_versioning() {
    let app = app!(|api| {
        edp_stub!(api);
    });

    let response = call_app!(app, Get, "http://127.0.0.1:3000/info").ok().unwrap();
    assert_eq!(response.status, status::StatusCode::Ok);
}