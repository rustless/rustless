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
  Rustless, Builder, Application, Api, Endpoint, EndpointInstance, 
  Namespace, NamespaceBehavior, Get, Post
};

fn main() {

    // Create API with "1" version
    let mut chat_api = box Api::new("1");
    
    // Crate namespace "users" that will accept parameter "id" in URL
    let mut chat_namespace = box Namespace::new("users/:id");

    // This is the handler function to provide response, we will use it later
    fn process<'a>(endpoint: EndpointInstance<'a>, params: &Json) -> EndpointInstance<'a> {
        endpoint.json(params)
    }

    // Create Endpoint to respond to "POST /users/:id/add_friend" URL.
    chat_namespace.mount(box Endpoint::new(
        Post,
        "add_friend",
        "Add friend to user",
        |params| { 
            // Rustless is intergated with Valico library to provide
            // parameters validation and coercion.
            params.req_typed("friend_id", Valico::u64())
        },
        process
    ));

    chat_api.mount(chat_namespace);
  
    let mut builder = Builder::new();
    builder.mount(chat_api);
  
    // Start server stuff
    let rustless: Rustless = rustless::Rustless;
    rustless.listen(
      builder.get_app(),
      Ipv4Addr(127, 0, 0, 1),
      3000
    );
  
    println!("Server started!");
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

