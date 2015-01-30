use std::str;
use valico;
use rustless::server::status;
use rustless::{Nesting};

#[test]
fn it_validates_endpoint_simple_path_params() {

    let app = app!(|api| {
        api.prefix("api");

        api.get("users/:user_id/messages/:message_id", |endpoint| {
            edp_stub_handler!(endpoint)
        })
    });

    let err_resp = call_app!(app, Get, "http://127.0.0.1:3000/api/users").err().unwrap();
    assert_eq!(err_resp.response.status, status::StatusCode::NotFound);

    let err_resp = call_app!(app, Get, "http://127.0.0.1:3000/api/users/").err().unwrap();
    assert_eq!(err_resp.response.status, status::StatusCode::NotFound);

    let response = call_app!(app, Get, "http://127.0.0.1:3000/api/users/Skywalker/messages/100").ok().unwrap();
    assert_eq!(response.status, status::StatusCode::Ok);

    let response = call_app!(app, Get, "http://127.0.0.1:3000/api/users/100/messages/12").ok().unwrap();
    assert_eq!(response.status, status::StatusCode::Ok);

}

#[test]
fn it_validates_typed_endpoint_path_params() {

    let app = app!(|api| {
        api.prefix("api");

        api.get("users/:user_id/messages/:message_id", |endpoint| {
            endpoint.params(|params| {
                params.req_typed("user_id", valico::u64());
                params.req_typed("message_id", valico::u64());
            });

            edp_stub_handler!(endpoint)
        })
    });

    let err_resp = call_app!(app, Get, "http://127.0.0.1:3000/api/users/Skywalker/messages/100").err().unwrap();
    assert_eq!(err_resp.response.status, status::StatusCode::BadRequest);    

    let err_resp = call_app!(app, Get, "http://127.0.0.1:3000/api/users/100/messages/Skywalker").err().unwrap();
    assert_eq!(err_resp.response.status, status::StatusCode::BadRequest);

    let response = call_app!(app, Get, "http://127.0.0.1:3000/api/users/100/messages/12").ok().unwrap();
    assert_eq!(response.status, status::StatusCode::Ok);

}

#[test]
fn it_validates_query_params() {

    let app = app!(|api| {
        api.prefix("api");

        api.get("users/:user_id", |endpoint| {
            endpoint.params(|params| {
                params.req_typed("user_id", valico::u64());
                params.req("profile", |profile| {
                    profile.allow_values(&["simple".to_string(), "full".to_string()])
                })
            });

            edp_stub_handler!(endpoint)
        })
    });

    let err_resp = call_app!(app, Get, "http://127.0.0.1:3000/api/users/100").err().unwrap();
    assert_eq!(err_resp.response.status, status::StatusCode::BadRequest);    

    let err_resp = call_app!(app, Get, "http://127.0.0.1:3000/api/users/100?profile=1").err().unwrap();
    assert_eq!(err_resp.response.status, status::StatusCode::BadRequest);    

    let err_resp = call_app!(app, Get, "http://127.0.0.1:3000/api/users/100?profile=fulll").err().unwrap();
    assert_eq!(err_resp.response.status, status::StatusCode::BadRequest);        

    let response = call_app!(app, Get, "http://127.0.0.1:3000/api/users/100?profile=full").ok().unwrap();
    assert_eq!(response.status, status::StatusCode::Ok);    

    let response = call_app!(app, Get, "http://127.0.0.1:3000/api/users/100?profile=simple").ok().unwrap();
    assert_eq!(response.status, status::StatusCode::Ok);   
     
}

#[test]
fn it_validates_common_namespace_params() {

    let app = app!(|api| {
        api.prefix("api");

        api.resources("users/:user_id", |users| {
            users.params(|params| {
                // one parameter goes from path and one from query-string or body
                params.req_typed("user_id", valico::u64());
                params.req_typed("ext", valico::string());
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

    let err_resp = call_app!(app, Get, "http://127.0.0.1:3000/api/users/100").err().unwrap();
    assert_eq!(err_resp.response.status, status::StatusCode::BadRequest);    

    let err_resp = call_app!(app, Get, "http://127.0.0.1:3000/api/users/100/profile/full").err().unwrap();
    // missed `ext` param
    assert_eq!(err_resp.response.status, status::StatusCode::BadRequest);  

    let mut response = call_app!(app, Get, "http://127.0.0.1:3000/api/users/100/profile/full?ext=some").ok().unwrap();
        println!("{}", resp_body!(response));
    assert_eq!(response.status, status::StatusCode::Ok);   
}