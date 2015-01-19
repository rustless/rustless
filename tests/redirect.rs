use url::Url;
use rustless::server::method::Method::{Post};
use rustless::server::header::common::{Location};
use rustless::server::status::StatusCode;
use rustless::{
    Application, Api, Nesting, SimpleRequest
};

#[test]
fn it_allows_redirect() {

    let app = app!(|api| {
        api.prefix("api");
        api.post("redirect_me/:href", |endpoint| {
            endpoint.handle(|client, params| {
                client.redirect(params.get(&"href".to_string()).unwrap().as_string().unwrap())
            })
        });
    });

    let response = call_app!(app, Post, "http://127.0.0.1:3000/api/redirect_me/google.com").unwrap();
    assert_eq!(response.status, StatusCode::Found);
    let &Location(ref location) = response.headers.get::<Location>().unwrap();
    assert_eq!(location.as_slice(), "google.com")

}

#[test]
fn it_allows_permanent_redirect() {

    let app = app!(|api| {
        api.prefix("api");
        api.post("redirect_me/:href", |endpoint| {
            endpoint.handle(|client, params| {
                client.permanent_redirect(params.get(&"href".to_string()).unwrap().as_string().unwrap())
            })
        });
    });

    let response = call_app!(app, Post, "http://127.0.0.1:3000/api/redirect_me/google.com").unwrap();
    assert_eq!(response.status, StatusCode::MovedPermanently);
    let &Location(ref location) = response.headers.get::<Location>().unwrap();
    assert_eq!(location.as_slice(), "google.com")

}