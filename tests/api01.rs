

use std::path::Path;
use serialize::json::{JsonObject, ToJson};
use rustless::{
    Application, Valico, Api, Client, Nesting, 
    HandleResult, AcceptHeaderVersioning, Static
};

fn create_app() -> Application {

    let api = box Api::build(|api| {
        // Specify API version
        api.version("v1", AcceptHeaderVersioning("chat"));
        api.prefix("api");

        // Create API for chats
        let chats_api = box Api::build(|chats_api| {

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

    app.mount(box Static::new(Path::new(".")));
    app.mount(api);

    app

}

#[test]
#[allow(unused_variable)]
fn test_api() {

    let app = create_app();

}