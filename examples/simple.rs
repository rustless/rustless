#![allow(unstable)]

#[macro_use]
extern crate rustless;

extern crate iron;
extern crate url;
extern crate "rustc-serialize" as serialize;
extern crate valico;
extern crate cookie;

use std::error::Error as StdError;
use iron::{Iron, Chain, ChainBuilder};
use cookie::Cookie;

use valico::Builder as Valico;
use rustless::server::status::{StatusCode};
use rustless::errors::{Error};
use rustless::batteries::cookie::CookieExt;
use rustless::{
    Application, Api, Nesting, Versioning,
    Response
};

#[derive(Show, Copy)]
pub struct UnauthorizedError;

impl StdError for UnauthorizedError {
    fn description(&self) -> &str {
        return "UnauthorizedError";
    }
}

fn main() {

    let app = Application::new(Api::build(|api| {
        api.prefix("api");
        api.version("v1", Versioning::Path);

        api.error_formatter(|err, _media| {
            match err.downcast::<UnauthorizedError>() {
                Some(_) => {
                    return Some(Response::from_string(StatusCode::Unauthorized, "Please provide correct `token` parameter".to_string()))
                },
                None => None
            }
        });

        api.namespace("admin", |admin_ns| {

            admin_ns.params(|params| {
                params.req_typed("token", Valico::string())
            });

            // Using after_validation callback to check token
            admin_ns.after_validation(|&: _client, params| {

                match params.get("token") {
                    // We can unwrap() safely because token in validated already
                    Some(token) => if token.as_string().unwrap().as_slice() == "password1" { return Ok(()) },
                    None => ()
                }

                // Fire error from callback is token is wrong
                return Err(Box::new(UnauthorizedError) as Box<Error>)

            });

            // This `/api/admin/server_status` endpoint is secure now
            admin_ns.get("server_status", |endpoint| {
                endpoint.handle(|client, _params| {
                    {
                        let cookies = client.request.cookies();
                        let signed_cookies = cookies.signed();

                        let user_cookie = Cookie::new("session".to_string(), "verified".to_string());
                        signed_cookies.add(user_cookie);
                    }

                    client.text("Everything is OK".to_string())  
                })
            });
        })
    }));

    

    let mut chain = ChainBuilder::new(app);
    chain.link(::rustless::batteries::cookie::new("secretsecretsecretsecretsecretsecretsecret".as_bytes()));

    Iron::new(chain).listen("localhost:4000").unwrap();
    println!("On 4000");

}