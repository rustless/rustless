## What is Rustless?

[![Build Status](https://travis-ci.org/rustless/rustless.svg?branch=master)](https://travis-ci.org/rustless/rustless)

Rustless is a REST-like API micro-framework for Rust. It's designed to provide a simple DSL to easily develop RESTful APIs. It has built-in support for common conventions, including multiple formats, subdomain/prefix restriction, content negotiation, versioning and much more.

Rustless in a port of [Grape] library from Ruby world. Based on [hyper] - an HTTP library for Rust.

Like Rust itself, Rustless is still in the early stages of development, so don't be surprised if APIs change and things break. If something's not working properly, file an issue or submit a pull request! 

[Grape]: https://github.com/intridea/grape
[hyper]: https://github.com/hyperium/hyper

```toml
# Cargo.toml
[dependencies.rustless]
git = "https://github.com/rustless/rustless"
```

[API docs](http://rustless.org/rustless/doc/rustless)

## Basic Usage

Below is a simple example showing some of the more common features of Rustless.

~~~rust
#![feature(phase)]

#[phase(plugin)]
extern crate rustless;
extern crate rustless;
extern crate hyper;
extern crate serialize;

use std::io::net::ip::Ipv4Addr;
use serialize::json::{JsonObject, ToJson};
use rustless::{
    Server, Application, Valico, Api, Client, Nesting, 
    HandleResult, HandleSuccessResult, AcceptHeaderVersioning
};

fn main() {

    let api = box Api::build(|api| {
        // Specify API version
        api.version("v1", AcceptHeaderVersioning("chat"));
        api.prefix("api");

        // Create API for chats
        let chats_api = box Api::build(|chats_api| {

            chats_api.after(callback!(|client, _params| {
                client.set_status(hyper::status::NotFound);
                Ok(())
            }));

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

                    // Set-up handler for endpoint, note that we return
                    // of macro invocation.
                    edp_handler!(endpoint, |client, params| {
                        client.json(&params.to_json())
                    })
                });

            });
        });

        api.mount(chats_api);
    });

    let mut app = Application::new();
    app.mount(api);

    let server: Server = Server;
    server.listen(
        app,
        Ipv4Addr(127, 0, 0, 1),
        3000
    );

    println!("Rustless server started!");
}
~~~

## Mounting

In Rustless you can use three core entities to build your RESTful app: `Api`, `Namespace` and `Endpoint`. 

* Api can mount Api, Namespace and Endpoint
* Namespace can mount Api, Namespace and Endpoint

~~~rust
Api::build(|api| {

    // Api inside Api example
    api.mount(box Api::build(|nested_api| {

        // Endpoint definition
        nested_api.get("nested_info", |endpoint| {
            // endpoint.params(|params| {});
            // endpoint.desc("Some description");

            // Endpoint handler
            edp_handler!(endpoint, |client, _params| {
                client.text("Some usefull info".to_string())
            })
        });

    }))

    // The namespace method has a number of aliases, including: group, 
    // resource, resources, and segment. Use whichever reads the best 
    // for your API.
    api.namespace("ns1", |ns1| {
        ns1.group("ns2", |ns2| {
            ns2.resource("ns3", |ns3| {
                ns3.resources("ns4", |ns4| {
                    ns4.segment("ns5", |ns5| {
                        // ...
                    );
                })
            })
        })
    })
})
~~~

## Parameter Validation and Coercion

You can define validations and coercion options for your parameters using a DSL block inside `Endpoint` and `Namespace` definition. See [Valico] for more info about things you can do.

~~~rust 
api.get("users/:user_id/messages/:message_id", |endpoint| {
    endpoint.params(|params| {
        params.req_typed("user_id", Valico::u64());
        params.req_typed("message_id", Valico::u64());
    });

    // ...
})
~~~

[Valico]: https://github.com/rustless/valico

## Query strings

Rustless is intergated with [queryst] to allow smart query-string parsing 
end decoding (even with nesting, like `foo[0][a]=a&foo[0][b]=b&foo[1][a]=aa&foo[1][b]=bb`). See [queryst] for more info.

[queryst]: https://github.com/rustless/queryst

## API versioning

There are three strategies in which clients can reach your API's endpoints: 

* PathVersioning
* AcceptHeaderVersioning
* ParamVersioning

### Path versioning strategy

~~~rust
api.version("v1", PathVersioning);
~~~

Using this versioning strategy, clients should pass the desired version in the URL.

    curl -H http://localhost:3000/v1/chats/

### Header versioning strategy

~~~rust
api.version("v1", AcceptHeaderVersioning("chat"));
~~~

Using this versioning strategy, clients should pass the desired version in the HTTP `Accept` head.

    curl -H Accept:application/vnd.chat.v1+json http://localhost:3000/chats

Accept version format is the same as Github (uses)[https://developer.github.com/v3/media/].

### Param versioning strategy

~~~rust
api.version("v1", ParamVersioning("ver"));
~~~

Using this versioning strategy, clients should pass the desired version as a request parameter in the URL query.

    curl -H http://localhost:9292/statuses/public_timeline?ver=v1

## Respond with custom HTTP Status Code

By default Rustless returns a 200 status code for `GET`-Requests and 201 for `POST`-Requests. You can use `status` and `set_status` to query and set the actual HTTP Status Code

~~~rust
client.set_status(NotFound);
~~~

## Use parameters

Request parameters are available through the `params: JsonObject` inside `Endpoint` handlers and all callbacks. This includes `GET`, `POST` and `PUT` parameters, along with any named parameters you specify in your route strings.

The request:

~~~
curl -d '{"text": "hello from echo"}' 'http://localhost:3000/echo' -H Content-Type:application/json -v
~~~

The Rustless endpoint:

~~~rust
api.post("", |endpoint| {
    edp_handler!(endpoint, |client, params| {
        client.json(params)
    })
});
~~~

In the case of conflict between either of:

* route string parameters
* `GET`, `POST` and `PUT` parameters
* the contents of the request body on `POST` and `PUT`

route string parameters will have precedence.

## Redirecting

You can redirect to a new url temporarily (302) or permanently (301).

~~~rust
client.redirect("http://google.com");
~~~

~~~rust
client.redirect_permanent("http://google.com");
~~~

## Error firing

You can abort the execution of an API method by raising errors with `error`.

Define your error like this:

~~~rust
use rustless::errors::{Error, ErrorRefExt};

#[deriving(Show)]
pub struct UnauthorizedError;

impl Error for UnauthorizedError {
    fn name(&self) -> &'static str {
        return "UnauthorizedError";
    }
}
~~~

And then throw:

~~~rust
client.error(UnauthorizedError);
~~~

## Error handling

By default Rustless wil respond all errors with status::InternalServerError.

Rustless can be told to rescue specific errors and return them in the custom API format.

~~~rust
format_error!(api, UnauthorizedError, |_err, _media| {
    Some(Response::from_string(status::Unauthorized, "Please provide correct `token` parameter".to_string()))
});
~~~

Also Rustless can be told to rescue all errors:

~~~rust
format_error!(api, all, |_err, _media| {
    Some(Response::from_string(status::InternalServerError, "Not enough mana!".to_string()))
});
~~~

## Before and After callbacks

Blocks can be executed before or after every API call, using `before`, `after`,
`before_validation` and `after_validation`.

Before and after callbacks execute in the following order:

1. `before`
2. `before_validation`
3. _validations_
4. `after_validation`
5. _the API call_
6. `after`

Steps 4, 5 and 6 only happen if validation succeeds.

The block applies to every API call within and below the current nesting level.

## Secure API example

~~~rust
Api::build(|api| {
    api.prefix("api");

    format_error!(api, UnauthorizedError, |_err, _media| {
        Some(Response::from_string(status::Unauthorized, "Please provide correct `token` parameter".to_string()))
    });

    api.namespace("admin", |admin_ns| {

        admin_ns.params(|params| {
            params.req_typed("token", Valico::string())
        });

        // Using after_validation callback to check token
        admin_ns.after_validation(callback!(|_client, params| {
            
            match params.find(&"token".to_string()) {
                // We can unwrap() safely because token in validated already
                Some(token) => if token.as_string().unwrap().as_slice() == "password1" { return Ok(()) },
                None => ()
            }

            // Fire error from callback is token is wrong
            return Err(UnauthorizedError.erase())

        }));

        // This `/api/admin/server_status` endpoint is secure now
        admin_ns.get("server_status", |endpoint| {
            edp_handler!(endpoint, |client, _params| {
                client.text("Everything is OK".to_string())  
            })
        });
    })
})
~~~

## JSON responses

Rustless includes (JsonWay)[https://github.com/rustless/jsonway] library to offer both complex JSON building DSL and configurable serializers for your objects. See (API docs)[http://rustless.org/jsonway/doc/jsonway/] for details.

Also feel free to use any other serialization library you want.

