Rustless
======

## What is Rustless?

Rustless is a REST-like API micro-framework for Rust. It's designed to provide a simple DSL to easily develop RESTful APIs. It has built-in support for common conventions, including multiple formats, subdomain/prefix restriction, content negotiation, versioning and much more.

Rustless in a port of [Grape] library from Ruby world and is still mostly **in progress** (that mean that API and features in
**experimental** in Rust's terms). Based on [hyper] - an HTTP library for Rust.

[Grape]: https://github.com/intridea/grape
[hyper]: https://github.com/hyperium/hyper

## Basic Usage

Below is a simple example showing some of the more common features of Rustless.

~~~rust
extern crate rustless;
extern crate serialize;

use std::io::net::ip::Ipv4Addr;
use serialize::json::Json;
use rustless::{
    Rustless, Builder, Valico, Api, EndpointInstance, NamespaceBehavior
};

fn main() {

    // Create API for chats
    let mut chats_api = box Api::build(|chats_api| {
        // Specify API version
        chats_api.version("v1");

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

                // Function to handle requests
                fn handler<'a>(endpoint: EndpointInstance<'a>, params: &Json) -> EndpointInstance<'a> {
                    // Respond with JSON with correct content-type
                    endpoint.json(params)
                }

                // Set-up handler for endpoint
                endpoint.handle(handler)
            });

        });
    });


    let mut builder = Builder::new();
    builder.mount(chats_api);

    let rustless: Rustless = Rustless;
    rustless.listen(
        builder.get_app(),
        Ipv4Addr(127, 0, 0, 1),
        3000
    );

    println!("Rustless server started!");
}
~~~

## Parameter Validation and Coercion

You can define validations and coercion options for your parameters using a DSL block in 
Endpoint and Namespace definition. See [Valico] for more info.

[Valico]: https://github.com/rustless/valico

## Query strings

Rustless is intergated with [rust-query] to allow smart query-string parsing 
(e.g. like `foo[0][a]=a&foo[0][b]=b&foo[1][a]=aa&foo[1][b]=bb`). See [rust-query] for more info.

[rust-query]: https://github.com/rustless/rust-query

