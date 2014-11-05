use url::Url;
use serialize::json::{JsonObject, Json, ToJson};
use serialize::json::from_str;
use std::str::from_utf8;

use jsonway::JsonWay;

use rustless::server_backend::header::common::ContentType;
use rustless::server_backend::method::{Get};
use rustless::server_backend::status;
use rustless::server_backend::mime;
use rustless::{
    Application, Api, Client, Nesting, HandleResult, SimpleRequest
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
    assert_eq!(response.status, status::Ok);

    {
        let &ContentType(ref mime_type): &ContentType = response.headers.get().unwrap();
        assert_eq!(*mime_type, mime::Mime(mime::Application, mime::Json, vec![]));    
    }

    let body: Json = from_str(from_utf8(response.read_to_end().unwrap().as_slice()).unwrap()).unwrap();

    assert!(body.find(&"uptime".to_string()).is_some());
    assert!(body.find(&"echo_params".to_string()).is_some());
}