#![feature(core)]

#[macro_use]
extern crate rustless;

extern crate iron;
extern crate url;
extern crate "rustc-serialize" as serialize;
extern crate valico;
extern crate cookie;

use std::fmt;
use std::error;
use std::error::Error as StdError;
use valico::json_dsl;

use rustless::server::status;
use rustless::errors::{Error};
use rustless::batteries::swagger;
use rustless::batteries::cookie::CookieExt;
use rustless::{Nesting};

#[derive(Debug, Copy)]
pub struct UnauthorizedError;

impl error::Error for UnauthorizedError {
    fn description(&self) -> &str {
        return "UnauthorizedError";
    }
}

impl fmt::Display for UnauthorizedError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        self.description().fmt(formatter)
    }
}

fn main() {

    let mut app = rustless::Application::new(rustless::Api::build(|api| {
        api.prefix("api");
        api.version("v1", rustless::Versioning::Path);
        
        api.mount(swagger::create_api("api-docs"));

        api.error_formatter(|err, _media| {
            match err.downcast::<UnauthorizedError>() {
                Some(_) => {
                    return Some(rustless::Response::from_string(
                        status::StatusCode::Unauthorized, 
                        "Please provide correct `token` parameter".to_string()
                    ))
                },
                None => None
            }
        });

        api.post("greet/:name", |endpoint| {
            endpoint.summary("Sends greeting");
            endpoint.desc("Use this to talk to yourself");
            endpoint.params(|params| {
                params.req_typed("name", json_dsl::string());
                params.req_typed("greeting", json_dsl::string());
            });
            endpoint.handle(|client, params| {
                client.text(
                    format!("{}, {}", 
                        params.find("greeting").unwrap().to_string(),
                        params.find("name").unwrap().to_string())
                )
            })
        });

        api.get("echo", |endpoint| {
            endpoint.summary("Sends back what it gets");
            endpoint.desc("Use this to talk to yourself");
            endpoint.handle(|client, params| {
                client.json(params)
            })
        });

        api.namespace("admin", |admin_ns| {

            admin_ns.params(|params| {
                params.req_typed("token", json_dsl::string())
            });

            // Using after_validation callback to check token
            admin_ns.after_validation(|&: _client, params| {

                match params.find("token") {
                    // We can unwrap() safely because token in validated already
                    Some(token) => if token.as_string().unwrap().as_slice() == "password1" { return Ok(()) },
                    None => ()
                }

                // Fire error from callback is token is wrong
                return Err(rustless::ErrorResponse{
                    error: Box::new(UnauthorizedError) as Box<Error>,
                    response: None
                })

            });

            // This `/api/admin/server_status` endpoint is secure now
            admin_ns.get("server_status", |endpoint| {
                endpoint.summary("Get server status");
                endpoint.desc("Use this API to receive some useful information about the state of our server");
                endpoint.handle(|client, _params| {
                    {
                        let cookies = client.request.cookies();
                        let signed_cookies = cookies.signed();

                        let user_cookie = cookie::Cookie::new("session".to_string(), "verified".to_string());
                        signed_cookies.add(user_cookie);
                    }

                    client.text("Everything is OK".to_string())  
                })
            });
        })
    }));

    swagger::enable(&mut app, swagger::Spec {
        info: swagger::Info {
            title: "Example API".to_string(),
            description: Some("Simple API to demonstration".to_string()),
            contact: Some(swagger::Contact {
                name: "Stanislav Panferov".to_string(),
                url: Some("http://panferov.me".to_string()),
                ..std::default::Default::default()
            }),
            license: Some(swagger::License {
                name: "MIT".to_string(),
                url: "http://opensource.org/licenses/MIT".to_string()
            }),
            ..std::default::Default::default()
        },
        ..std::default::Default::default()
    });

    let mut chain = iron::Chain::new(app);
    chain.link(::rustless::batteries::cookie::new("secretsecretsecretsecretsecretsecretsecret".as_bytes()));

    iron::Iron::new(chain).listen("localhost:4000").unwrap();
    println!("On 4000");

}