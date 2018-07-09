
[![Gitter](https://badges.gitter.im/Join%20Chat.svg)](https://gitter.im/rustless/rustless?utm_source=badge&utm_medium=badge&utm_campaign=pr-badge)

## Table of Contents

- [What is Rustless?](#what-is-rustless)
- [Usage warning](#usage-warning)
- [Basic Usage](#basic-usage)
- [Complex example](#complex-example)
- [Mounting](#mounting)
- [Parameters validation and coercion](#parameters-validation-and-coercion)
- [Use JSON Schema](#use-json-schema)
- [Query strings](#query-strings)
- [API versioning](#api-versioning)
- [Respond with custom HTTP Status Code](#respond-with-custom-http-status-code)
- [Use parameters](#use-parameters)
- [Redirecting](#redirecting)
- [Errors firing](#errors-firing)
- [Errors handling](#errors-handling)
- [Before and After callbacks](#before-and-after-callbacks)
- [Secure API example](#secure-api-example)
- [JSON responses](#json-responses)
- [Swagger 2.0 support](#swagger-20)
- [Integration with PostgreSQL](#integration-with-postgresql)
- [Integration with Deuterium ORM](#integration-with-deuterium-orm)

## What is Rustless?

[![Build Status](https://travis-ci.org/rustless/rustless.svg?branch=master)](https://travis-ci.org/rustless/rustless)

Rustless is a REST-like API micro-framework for Rust. It's designed to provide a simple DSL to easily develop RESTful APIs on top of the [Iron](https://github.com/iron/iron) web framework. It has built-in support for common conventions, including multiple formats, subdomain/prefix restriction, content negotiation, versioning and much more.

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

## See also

* [Valico](https://github.com/rustless/valico) - Rust JSON validator and coercer. See [Api docs](http://rustless.org/valico/doc/valico).
* [Queryst](https://github.com/rustless/queryst) - Rust query string parser with nesting support. See [Api docs](http://rustless.org/queryst/doc/queryst).
* [JsonWay](https://github.com/rustless/jsonway) - JSON building DSL and configurable serializers for Rust. See [Api docs](http://rustless.org/jsonway/doc/jsonway).

## Usage warning

Rustless is based on Iron, which is based on Hyper, which is **synchronous**. Hyper has a lot of limitations right now, and can't handle many simultaneous connections, especially with keep-alive. So it is **highly recommended** to use light asynchronous web server such as **Nginx** as a reverse proxy server with Rustless.

## Basic Usage

Below is a simple example showing some of the more common features of Rustless.

~~~rust
extern crate rustless;
extern crate hyper;
extern crate iron;
extern crate rustc_serialize as serialize;
extern crate valico;

use valico::json_dsl;
use rustless::server::status::StatusCode;
use rustless::{
    Application, Api, Nesting, Versioning
};
use rustless::json::ToJson;

fn main() {

    let api = Api::build(|api| {
        // Specify API version
        api.version("v1", Versioning::AcceptHeader("chat"));
        api.prefix("api");

        // Create API for chats
        api.mount(Api::build(|chats_api| {

            chats_api.after(|client, _params| {
                client.set_status(StatusCode::NotFound);
                Ok(())
            });

            // Add namespace
            chats_api.namespace("chats/:id", |chat_ns| {

                // Valico settings for this namespace
                chat_ns.params(|params| {
                    params.req_typed("id", json_dsl::u64())
                });

                // Create endpoint for POST /chats/:id/users/:user_id
                chat_ns.post("users/:user_id", |endpoint| {

                    // Add description
                    endpoint.desc("Update user");

                    // Valico settings for endpoint params
                    endpoint.params(|params| {
                        params.req_typed("user_id", json_dsl::u64());
                        params.req_typed("id", json_dsl::string())
                    });

                    endpoint.handle(|client, params| {
                        client.json(&params.to_json())
                    })
                });

            });
        }));
    });

    let app = Application::new(api);

    iron::Iron::new(app).http("0.0.0.0:4000").unwrap();
    println!("On 4000");

    println!("Rustless server started!");
}
~~~

To easily build the example, you can set your Cargo.toml file approximately as follows:

~~~
[package]
name = "rustless-example"
version = "0.1.0"

[dependencies]
rustless = "0.10.0"
hyper = "0.10.5"
rustc-serialize = "0.3"
valico = "1"

[dependencies.iron]
version = "*"
~~~

## Complex example

If you want to see how you can write some complex application using Rustless, please see the [example](https://github.com/rustless/rustless-example).

In the example, please note the following aspects:

* Complex nested API with versioning.
* CRUD operations with rust-postgres.
* Swagger 2.0 intergration.
* JSON Schema validations.
* Error reporting.
* Serializers.
* File structure.
* Integration with [docopt](https://github.com/docopt/docopt.rs).
* Integration with [deuterium-orm](https://github.com/deuterium-orm/deuterium-orm). Database migrations.

## Mounting

In Rustless you can use three core entities to build your RESTful app: `Api`, `Namespace` and `Endpoint`.

* Api can mount Api, Namespace and Endpoint
* Namespace can mount Api, Namespace and Endpoint

~~~rust
Api::build(|api| {

    // Api inside Api example
    api.mount(Api::build(|nested_api| {

        // Endpoint definition
        nested_api.get("nested_info", |endpoint| {
            // endpoint.params(|params| {});
            // endpoint.desc("Some description");

            // Endpoint handler
            endpoint.handle(|client, _params| {
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

## Parameters validation and coercion

You can define validations and coercion options for your parameters using a DSL block inside `Endpoint` and `Namespace` definition. See [Valico] for more info about what you can do.

~~~rust
api.get("users/:user_id/messages/:message_id", |endpoint| {
    endpoint.params(|params| {
        params.req_typed("user_id", Valico::u64());
        params.req_typed("message_id", Valico::u64());
    });

    // ...
})
~~~

## Use JSON Schema

Also you can use JSON Schema (IETF's draft v4) to validate your parameters. To use schemes in your application you need to use the following setup:

~~~rust
use valico::json_schema;
use rustless::batteries::schemes;

let scope = json_schema::Scope::new();

// ... You can insert some external schemes here ...

schemes::enable_schemes(&mut app, scope).unwrap();
~~~

See [Valico] for more info about JSON Scheme usage inside DSL blocks.

[Valico]: https://github.com/rustless/valico

## Query strings

Rustless is intergated with [queryst] to allow smart query-string parsing
end decoding (even with nesting, like `foo[0][a]=a&foo[0][b]=b&foo[1][a]=aa&foo[1][b]=bb`). See [queryst] for more info.

[queryst]: https://github.com/rustless/queryst

## API versioning

There are three strategies in which clients can reach your API's endpoints:

* Path
* AcceptHeader
* Param

### Path versioning strategy

~~~rust
api.version("v1", Path);
~~~

Using this versioning strategy, clients should pass the desired version in the URL.

    curl -H http://localhost:3000/v1/chats/

### Header versioning strategy

~~~rust
api.version("v1", AcceptHeader("chat"));
~~~

Using this versioning strategy, clients should pass the desired version in the HTTP `Accept` head.

    curl -H Accept:application/vnd.chat.v1+json http://localhost:3000/chats

Accept version format is the same as Github (uses)[https://developer.github.com/v3/media/].

### Param versioning strategy

~~~rust
api.version("v1", Param("ver"));
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
    endpoint.handle(|client, params| {
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

## Errors firing

You can abort the execution of an API method by raising errors with `error`.

Define your error like this:

~~~rust
use rustless::errors::{Error, ErrorRefExt};

#[deriving(Show)]
pub struct UnauthorizedError;

impl std::error::Error for UnauthorizedError {
    fn description(&self) -> &str {
        return "UnauthorizedError";
    }
}
~~~

And then throw:

~~~rust
client.error(UnauthorizedError);
~~~

## Errors handling

By default Rustless wil respond all errors with status::InternalServerError.

Rustless can be told to rescue specific errors and return them in the custom API format.

~~~rust
api.error_formatter(|err, _media| {
    match err.downcast::<UnauthorizedError>() {
        Some(_) => {
            return Some(Response::from_string(StatusCode::Unauthorized, "Please provide correct `token` parameter".to_string()))
        },
        None => None
    }
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
})
~~~

## JSON responses

Rustless includes [JsonWay](https://github.com/rustless/jsonway) library to offer both complex JSON building DSL and configurable serializers for your objects. See [API docs](http://rustless.org/jsonway/doc/jsonway/) for details.

Also feel free to use any other serialization library you want.

## Swagger 2.0

Rustless has a basic implementation of Swagger 2.0 specification. It is not fully complete and in future we need to implement:

* JSON Schema support (when some appropriate JSON Schema library will appear);
* Security parts of the specification;

But now you can already use Swagger 2.0:

```rust
let mut app = rustless::Application::new(rustless::Api::build(|api| {
    // ...

    api.mount(swagger::create_api("api-docs"));

    // ...
}))

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
    host: "localhost:4000".to_string(),
    ..std::default::Default::default()
});
```

After that you can use `/api-docs` path in Swagger UI to render your API structure.

## Integration with PostgreSQL

We have an annotated example of such integration in [rustless-example](https://github.com/rustless/rustless-example). Please try it and feel free to say your opinion.

## Integration with Deuterium ORM

TODO: Example
