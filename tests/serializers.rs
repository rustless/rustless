use url::Url;
use serialize::json::{Json, ToJson};
use serialize::json::from_str;
use std::str::from_utf8;

use jsonway::JsonWay;

use rustless::server::header::common::ContentType;
use rustless::server::method::Method::{Get};
use rustless::server::status::StatusCode;
use rustless::server::mime;
use rustless::{
    Application, Api, Client, Nesting, SimpleRequest
};

#[test]
fn it_serializes_json_properly() {

    let app = app!(|api| {
        api.prefix("api");

        api.get("status", |endpoint| {
            edp_handler!(endpoint, |client, params| {
                client.json(&JsonWay::object(|json| {
                    json.set("uptime", "Ok".to_string());
                    json.set("echo_params", params.to_json());
                }).unwrap())
            })
        })
    });

    let mut response = call_app!(app, Get, "http://127.0.0.1:3000/api/status").unwrap();
    assert_eq!(response.status, StatusCode::Ok);

    {
        let &ContentType(ref mime_type): &ContentType = response.headers.get().unwrap();
        assert_eq!(*mime_type, mime::Mime(mime::TopLevel::Application, mime::SubLevel::Json, vec![]));    
    }

    let body: Json = from_str(from_utf8(response.read_to_end().unwrap().as_slice()).unwrap()).unwrap();

    assert!(body.find("uptime").is_some());
    assert!(body.find("echo_params").is_some());
}