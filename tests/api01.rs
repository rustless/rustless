
use url::Url;
// use std::str;
use std::path::Path;
use serialize::json::{JsonObject, ToJson};
use rustless::server_backend::method::{Post};
use rustless::server_backend::status;
use rustless::server_backend::header::common::Accept;
use rustless::{
    Application, Valico, Api, Client, Nesting, Media,
    HandleResult, AcceptHeaderVersioning, Static, SimpleRequest, Response
};

use rustless::errors::{Error, ErrorRefExt, ValidationError};

fn create_app() -> Application {

    let api = box Api::build(|api| {
        // Specify API version
        api.version("v1", AcceptHeaderVersioning("chat"));
        api.prefix("api");

        rescue_from!(api, ValidationError, |err, _client| {
            Some(Response::from_json(status::BadRequest, &err.reason.to_json()))
        })

        // Create API for chats
        let chats_api = box Api::build(|chats_api| {

            // Add namespace
            chats_api.namespace("chats/:id", |chat_ns| {
                
                // Valico settings for this namespace
                chat_ns.params(|params| { 
                    params.req_typed("id", Valico::u64())
                });

                // Create endpoint for POST /chats/:id/users/:user_id
                chat_ns.post("users/:user_id", |endpoint| {
                
                    // Add description
                    endpoint.desc("Update user");

                    // Valico settings for endpoint params
                    endpoint.params(|params| { 
                        params.req_typed("user_id", Valico::u64());
                        params.req_typed("name", Valico::string())
                    });

                    // Set-up handler for endpoint, note that we return
                    // of macro invocation.
                    edp_handler!(endpoint, |client, params| {
                        client.json(&params.to_json())
                    })
                });

            });
        });

        api.mount(chats_api);
    });

    let mut app = Application::new();

    app.mount(box Static::new(Path::new(".")));
    app.mount(api);

    app

}

macro_rules! sr {
    ($edp:ident, $url:expr) => {
        SimpleRequest::new($edp, Url::parse($url).unwrap())
    };
    ($edp:ident, $url:expr, $blk:expr) => {
        SimpleRequest::build($edp, Url::parse($url).unwrap(), $blk)
    };
}

macro_rules! call_app {
    ($app:ident, $edp:ident, $url:expr) => {
        $app.call(&mut sr!($edp, $url))
    };    
    ($app:ident, $edp:ident, $url:expr, $blk:expr) => {
        $app.call(&mut sr!($edp, $url, $blk))
    };
}

macro_rules! resp_body (
    ($resp:ident) => (str::from_utf8($resp.read_to_end().unwrap().as_slice()).unwrap())
)

macro_rules! mime(
    ($mime:expr) => (from_str($mime).unwrap())
)

#[test]
#[allow(unused_variable)]
fn test_api() {
    let app = create_app();

    let response = call_app!(app, Post, "http://127.0.0.1:3000/api/chats/100/users/15").unwrap();
    // not found because accept-header not present
    assert_eq!(response.status, status::NotFound);

    let response = call_app!(app, Post, "http://127.0.0.1:3000/api/chats/100/users/15", |rq| {
        rq.headers_mut().set(
            Accept( vec![mime!("application/vnd.chat.v1+json")] )
        );
    }).unwrap();

    // bad request because `name` parameter is required
    assert_eq!(response.status, status::BadRequest);

    let response = call_app!(app, Post, "http://127.0.0.1:3000/api/chats/100/users/15?name=Anakin%20Skywalker", |rq| {
        rq.headers_mut().set(
            Accept( vec![mime!("application/vnd.chat.v1+json")] )
        );
    }).unwrap();

    assert_eq!(response.status, status::Ok);
}