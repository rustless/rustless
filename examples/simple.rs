#![feature(phase)]

#[phase(plugin)]
extern crate rustless;
extern crate rustless;

extern crate iron;
extern crate url;
extern crate serialize;
extern crate valico;
extern crate cookie;

use iron::{Iron, Chain, ChainBuilder};
use cookie::Cookie;

use valico::Builder as Valico;
use rustless::server::status::{StatusCode};
use rustless::errors::{Error, ErrorRefExt};
use rustless::batteries::cookie::CookieExt;
use rustless::{
    Application, Api, Client, Nesting, Versioning,
    Response
};

#[deriving(Show, Copy)]
pub struct UnauthorizedError;

impl Error for UnauthorizedError {
    fn name(&self) -> &'static str {
        return "UnauthorizedError";
    }
}

fn main() {

    let app = Application::new(Api::build(|api| {
        api.prefix("api");
        api.version("v1", Versioning::Path);

        format_error!(api, UnauthorizedError, |_err, _media| {
            Some(Response::from_string(StatusCode::Unauthorized, "Please provide correct `token` parameter".to_string()))
        });

        api.namespace("admin", |admin_ns| {

            admin_ns.params(|params| {
                params.req_typed("token", Valico::string())
            });

            // Using after_validation callback to check token
            admin_ns.after_validation(callback!(|_client, params| {

                match params.get("token") {
                    // We can unwrap() safely because token in validated already
                    Some(token) => if token.as_string().unwrap().as_slice() == "password1" { return Ok(()) },
                    None => ()
                }

                // Fire error from callback is token is wrong
                return Err(box UnauthorizedError as Box<Error>)

            }));

            // This `/api/admin/server_status` endpoint is secure now
            admin_ns.get("server_status", |endpoint| {
                edp_handler!(endpoint, |client, _params| {
                    {
                        let cookies = client.request.cookies();
                        let signed_cookies = cookies.signed();

                        let mut user_cookie = Cookie::new("session".to_string(), "verified".to_string());
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