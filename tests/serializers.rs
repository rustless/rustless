use serialize::json;
use std::io::Read;
use std::str;

use jsonway;

use rustless::server::header;
use rustless::server::status;
use rustless::server::mime;
use rustless::{Nesting};

#[test]
fn it_serializes_json_properly() {

    let app = app!(|api| {
        api.prefix("api");

        api.get("status", |endpoint| {
            endpoint.handle(|client, params| {
                client.json(&jsonway::object(|json| {
                    json.set("uptime", "Ok".to_string());
                    json.set("echo_params", params.clone());
                }).unwrap())
            })
        })
    });

    let mut response = call_app!(app, Get, "http://127.0.0.1:3000/api/status").ok().unwrap();
    assert_eq!(response.status, status::StatusCode::Ok);

    {
        let &header::ContentType(ref mime_type): &header::ContentType = response.headers.get().unwrap();
        assert_eq!(*mime_type, mime::Mime(mime::TopLevel::Application, mime::SubLevel::Json, vec![]));
    }

    let mut bytes = Vec::new();
    response.read_to_end(&mut bytes).unwrap();
    let body: json::Json = str::from_utf8(&bytes).unwrap().parse().unwrap();

    assert!(body.find("uptime").is_some());
    assert!(body.find("echo_params").is_some());
}