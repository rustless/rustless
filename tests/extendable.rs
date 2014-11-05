use url::Url;
use serialize::json::{Json, JsonObject};
use jsonway::JsonWay;
use serialize::json::from_str;
use std::str::from_utf8;
use rustless::server_backend::method::{Get};
use rustless::server_backend::status;
use rustless::{
    Application, Api, Client, Nesting, HandleResult, 
    SimpleRequest
};

struct TestExt(String);

#[test]
fn it_pass_without_versioning() {
    let mut app = app!(|api| {
        api.get("plugin_value", |endpoint| {
            edp_handler!(endpoint, |client, _params| {
                let ref ext_value = client.app.ext.find::<TestExt>().unwrap().0;
                client.json(&JsonWay::object(|json| {
                    json.set("value", ext_value.clone());
                }).unwrap())
            })
        })
    });

    app.ext.insert(TestExt("Ok".to_string()));

    let mut response = call_app!(app, Get, "http://127.0.0.1:3000/plugin_value").unwrap();
    assert_eq!(response.status, status::Ok);

    let body: Json = from_str(from_utf8(response.read_to_end().unwrap().as_slice()).unwrap()).unwrap();
    assert_eq!(body.find(&"value".to_string()).unwrap().as_string().unwrap(), "Ok");
}