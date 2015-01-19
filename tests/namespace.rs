use url::Url;
use rustless::server::method::Method::{Get};
use rustless::server::status::StatusCode;
use rustless::{
    Application, Api, Nesting, SimpleRequest
};

#[test]
fn it_allows_to_create_namespace() {

    let app = app!(|api| {
        api.prefix("api");

        api.namespace("ns1", |ns| edp_stub!(ns));
        api.group("ns2", |ns| edp_stub!(ns));
        api.resource("ns3", |ns| edp_stub!(ns));
        api.resources("ns4", |ns| edp_stub!(ns));
        api.segment("ns5", |ns| edp_stub!(ns));

    });

    let response = call_app!(app, Get, "http://127.0.0.1:3000/api/ns1/info").unwrap();
    assert_eq!(response.status, StatusCode::Ok);

    let response = call_app!(app, Get, "http://127.0.0.1:3000/api/ns2/info").unwrap();
    assert_eq!(response.status, StatusCode::Ok);

    let response = call_app!(app, Get, "http://127.0.0.1:3000/api/ns3/info").unwrap();
    assert_eq!(response.status, StatusCode::Ok);

    let response = call_app!(app, Get, "http://127.0.0.1:3000/api/ns4/info").unwrap();
    assert_eq!(response.status, StatusCode::Ok);

    let response = call_app!(app, Get, "http://127.0.0.1:3000/api/ns5/info").unwrap();
    assert_eq!(response.status, StatusCode::Ok);

}

#[test]
fn it_allows_nested_namespaces() {

    let app = app!(|api| {
        api.prefix("api");

        api.namespace("ns1", |ns1| {
            ns1.group("ns2", |ns2| {
                ns2.resource("ns3", |ns3| {
                    ns3.resources("ns4", |ns4| {
                        ns4.segment("ns5", |ns5| edp_stub!(ns5));
                    })
                })
            })
        })

    });

    let response = call_app!(app, Get, "http://127.0.0.1:3000/api/ns1/ns2/ns3/ns4/ns5/info").unwrap();
    assert_eq!(response.status, StatusCode::Ok);

}