use url::Url;
use serialize::json::{JsonObject};
use rustless::server_backend::method::{Post};
use rustless::server_backend::header::common::{Location};
use rustless::server_backend::status;
use rustless::{
    Application, Api, Client, Nesting, HandleResult, SimpleRequest
};

#[test]
fn it_allows_redirect() {

    let app = app!(|api| {
        api.prefix("api");
        api.post("redirect_me/:href", |endpoint| {
            edp_handler!(endpoint, |client, params| {
                client.redirect(params.find(&"href".to_string()).unwrap().as_string().unwrap())
            })
        });
    });

    let response = call_app!(app, Post, "http://127.0.0.1:3000/api/redirect_me/google.com").unwrap();
    assert_eq!(response.status, status::Found);
    let &Location(ref location) = response.headers.get::<Location>().unwrap();
    assert_eq!(location.as_slice(), "google.com")

}

#[test]
fn it_allows_permanent_redirect() {

    let app = app!(|api| {
        api.prefix("api");
        api.post("redirect_me/:href", |endpoint| {
            edp_handler!(endpoint, |client, params| {
                client.permanent_redirect(params.find(&"href".to_string()).unwrap().as_string().unwrap())
            })
        });
    });

    let response = call_app!(app, Post, "http://127.0.0.1:3000/api/redirect_me/google.com").unwrap();
    assert_eq!(response.status, status::MovedPermanently);
    let &Location(ref location) = response.headers.get::<Location>().unwrap();
    assert_eq!(location.as_slice(), "google.com")

}