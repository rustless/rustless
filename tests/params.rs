use url::Url;
use std::str;
use rustless::server::method::Method::{Get};
use rustless::server::status::StatusCode;
use rustless::{
    Application, Api, Client, Valico, Nesting, SimpleRequest
};

#[test]
fn it_validates_endpoint_simple_path_params() {

    let app = app!(|api| {
        api.prefix("api");

        api.get("users/:user_id/messages/:message_id", |endpoint| {
            edp_stub_handler!(endpoint)
        })
    });

    let response = call_app!(app, Get, "http://127.0.0.1:3000/api/users").unwrap();
    assert_eq!(response.status, StatusCode::NotFound);

    let response = call_app!(app, Get, "http://127.0.0.1:3000/api/users/").unwrap();
    assert_eq!(response.status, StatusCode::NotFound);

    let response = call_app!(app, Get, "http://127.0.0.1:3000/api/users/Skywalker/messages/100").unwrap();
    assert_eq!(response.status, StatusCode::Ok);

    let response = call_app!(app, Get, "http://127.0.0.1:3000/api/users/100/messages/12").unwrap();
    assert_eq!(response.status, StatusCode::Ok);

}

#[test]
fn it_validates_typed_endpoint_path_params() {

    let app = app!(|api| {
        api.prefix("api");

        api.get("users/:user_id/messages/:message_id", |endpoint| {
            endpoint.params(|params| {
                params.req_typed("user_id", Valico::u64());
                params.req_typed("message_id", Valico::u64());
            });

            edp_stub_handler!(endpoint)
        })
    });

    let response = call_app!(app, Get, "http://127.0.0.1:3000/api/users/Skywalker/messages/100").unwrap();
    assert_eq!(response.status, StatusCode::BadRequest);    

    let response = call_app!(app, Get, "http://127.0.0.1:3000/api/users/100/messages/Skywalker").unwrap();
    assert_eq!(response.status, StatusCode::BadRequest);

    let response = call_app!(app, Get, "http://127.0.0.1:3000/api/users/100/messages/12").unwrap();
    assert_eq!(response.status, StatusCode::Ok);

}

#[test]
fn it_validates_query_params() {

    let app = app!(|api| {
        api.prefix("api");

        api.get("users/:user_id", |endpoint| {
            endpoint.params(|params| {
                params.req_typed("user_id", Valico::u64());
                params.req("profile", |profile| {
                    profile.allow_values(&["simple".to_string(), "full".to_string()])
                })
            });

            edp_stub_handler!(endpoint)
        })
    });

    let response = call_app!(app, Get, "http://127.0.0.1:3000/api/users/100").unwrap();
    assert_eq!(response.status, StatusCode::BadRequest);    

    let response = call_app!(app, Get, "http://127.0.0.1:3000/api/users/100?profile=1").unwrap();
    assert_eq!(response.status, StatusCode::BadRequest);    

    let response = call_app!(app, Get, "http://127.0.0.1:3000/api/users/100?profile=fulll").unwrap();
    assert_eq!(response.status, StatusCode::BadRequest);        

    let response = call_app!(app, Get, "http://127.0.0.1:3000/api/users/100?profile=full").unwrap();
    assert_eq!(response.status, StatusCode::Ok);    

    let response = call_app!(app, Get, "http://127.0.0.1:3000/api/users/100?profile=simple").unwrap();
    assert_eq!(response.status, StatusCode::Ok);   
     
}

#[test]
fn it_validates_common_namespace_params() {

    let app = app!(|api| {
        api.prefix("api");

        api.resources("users/:user_id", |users| {
            users.params(|params| {
                // one parameter goes from path and one from query-string or body
                params.req_typed("user_id", Valico::u64());
                params.req_typed("ext", Valico::string());
            });

            users.get("profile/:profile", |endpoint| {
                endpoint.params(|params| {
                    params.req("profile", |profile| {
                        profile.allow_values(&["simple".to_string(), "full".to_string()])
                    })
                });

                edp_stub_handler!(endpoint)
            })
        })
    });

    let response = call_app!(app, Get, "http://127.0.0.1:3000/api/users/100").unwrap();
    assert_eq!(response.status, StatusCode::BadRequest);    

    let response = call_app!(app, Get, "http://127.0.0.1:3000/api/users/100/profile/full").unwrap();
    // missed `ext` param
    assert_eq!(response.status, StatusCode::BadRequest);  

    let mut response = call_app!(app, Get, "http://127.0.0.1:3000/api/users/100/profile/full?ext=some").unwrap();
        println!("{}", resp_body!(response));
    assert_eq!(response.status, StatusCode::Ok);   
}