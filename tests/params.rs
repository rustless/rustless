use valico::json_dsl;
use valico::json_schema;
use rustless::server::status;
use rustless::batteries::schemes;
use rustless::{Nesting};

#[test]
fn it_urldecodes_path_params() {

    let app = app!(|api| {
        api.prefix("api");

        api.get("users/:user_id/messages/:message_id", |endpoint| {
            endpoint.params(|params| {
                params.req("user_id", |user_id| {
                    user_id.allow_values(&["100/200".to_string()])
                })
            });

            endpoint.handle(|client, params| {
                client.text(format!("{}", params.find("message_id").and_then(|obj| { obj.as_str() }).unwrap()))
            })
        })
    });

    let response = call_app!(app, Get, "http://127.0.0.1:3000/api/users/100%2F200/messages/a%2Fb%3F").ok().unwrap();
    assert_eq!(response.status, status::StatusCode::Ok);
    assert_eq!(resp_body!(response), "a/b?");

}

#[test]
fn it_urldecodes_query_params() {

    let app = app!(|api| {
        api.prefix("api");

        api.get("users", |endpoint| {
            endpoint.params(|params| {
                params.req("user_id", |user_id| {
                    user_id.allow_values(&["100&200".to_string()])
                })
            });

            endpoint.handle(|client, params| {
                client.text(format!("{}", params.find("message_id").and_then(|obj| { obj.as_str() }).unwrap()))
            })
        })
    });

    let response = call_app!(app, Get, "http://127.0.0.1:3000/api/users?user_id=100%26200&message_id=a%26b%3F").ok().unwrap();
    assert_eq!(response.status, status::StatusCode::Ok);
    assert_eq!(resp_body!(response), "a&b?");

}

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
                params.req_typed("user_id", json_dsl::u64());
                params.req_typed("message_id", json_dsl::u64());
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
                params.req_typed("user_id", json_dsl::u64());
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
                params.req_typed("user_id", json_dsl::u64());
                params.req_typed("ext", json_dsl::string());
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

    let response = call_app!(app, Get, "http://127.0.0.1:3000/api/users/100/profile/full?ext=some").ok().unwrap();
        println!("{}", resp_body!(response));
    assert_eq!(response.status, status::StatusCode::Ok);
}

#[test]
fn it_validates_params_with_json_schema() {

    let mut app = app!(|api| {
        api.prefix("api");

        api.resources("users/:user_id", |users| {
            users.params(|params| {
                // one parameter goes from path and one from query-string or body
                params.req("user_id", |user_id| {
                    user_id.coerce(json_dsl::u64());
                    user_id.schema(|schema| {
                        schema.maximum(1000f64, false);
                    })
                });

                params.schema(|schema| {
                    schema.max_properties(1);
                });
            });

            users.get("profile/:profile", |endpoint| {
                endpoint.params(|params| {
                    params.req("profile", |profile| {
                        profile.schema(|schema| {
                            schema.enum_(|values| {
                                values.push("full".to_string());
                                values.push("short".to_string());
                            })
                        })
                    })
                });

                edp_stub_handler!(endpoint)
            })
        })
    });

    schemes::enable_schemes(&mut app, json_schema::Scope::new()).unwrap();

    let response = call_app!(app, Get, "http://127.0.0.1:3000/api/users/100/profile/full").ok().unwrap();
    assert_eq!(response.status, status::StatusCode::Ok);

    let err_resp = call_app!(app, Get, "http://127.0.0.1:3000/api/users/1001/profile/full").err().unwrap();
    assert_eq!(err_resp.response.status, status::StatusCode::BadRequest);

    let err_resp = call_app!(app, Get, "http://127.0.0.1:3000/api/users/1000/profile/wrong").err().unwrap();
    assert_eq!(err_resp.response.status, status::StatusCode::BadRequest);

    let err_resp = call_app!(app, Get, "http://127.0.0.1:3000/api/users/1000/profile/full?one_more=1").err().unwrap();
    assert_eq!(err_resp.response.status, status::StatusCode::BadRequest);

}