use std::ascii::AsciiExt;
use serialize::json::{ToJson};
use jsonway::{self, MutableJson};
use framework::{self, Nesting};
use server::header::common::access_control::allow_origin;

#[allow(unused_variables)]
pub fn fill_paths(current_path: &str, paths: &mut jsonway::ObjectBuilder, handlers: &framework::ApiHandlers) {
    for handler in handlers.iter() {
        if handler.is::<framework::Api>() {
            let mut path = current_path.to_string();

            let api = handler.downcast_ref::<framework::Api>().unwrap();
            if api.prefix.len() > 0 {
                path.push_str((api.prefix).as_slice());
            }
            if api.versioning.is_some() {
                match api.versioning.as_ref().unwrap() {
                    &framework::Versioning::Path if api.version.is_some() => {
                        path.push_str((api.version.as_ref().unwrap().to_string() + "/").as_slice())
                    },
                    _ => ()
                }
            }
            fill_paths(path.as_slice(), paths, &api.handlers);

        } else if handler.is::<framework::Namespace>() {
            let mut path = current_path.to_string();
            let namespace = handler.downcast_ref::<framework::Namespace>().unwrap();
            path.push_str(("/".to_string() + namespace.path.path.as_slice()).as_slice());
            fill_paths(path.as_slice(), paths, &namespace.handlers);
        
        } else if handler.is::<framework::Endpoint>() {
            let mut path = current_path.to_string();
            let endpoint = handler.downcast_ref::<framework::Endpoint>().unwrap();

            if endpoint.path.path.len() > 0 {
                path.push_str(("/".to_string() + endpoint.path.path.as_slice()).as_slice());
            }

            let definition = jsonway::JsonWay::object(|def| {
                // A list of tags for API documentation control. Tags can be used for logical grouping 
                // of operations by resources or any other qualifier.
                // def.array("tags", |tags| { });

                // A short summary of what the operation does. For maximum readability in the swagger-ui, 
                // this field SHOULD be less than 120 characters.
                def.set("summary", "Summary".to_string());

                // A verbose explanation of the operation behavior. 
                // GFM syntax can be used for rich text representation.
                def.set("description",  "Description".to_string());

                // External Documentation Object   
                // Additional external documentation for this operation.
                def.object("externalDocs", |external_docs| {
                    // A short description of the target documentation. 
                    // GFM syntax can be used for rich text representation.
                    external_docs.set("description", "Description".to_string()); 
                    //  Required. The URL for the target documentation. Value MUST be in the format of a URL.
                    external_docs.set("url", "http://google.com".to_string());
                });

                // A friendly name for the operation. The id MUST be unique among all operations described 
                // in the API. Tools and libraries MAY use the operation id to uniquely identify an operation.
                def.set("operationId", "OP".to_string());

                // A list of MIME types the operation can consume. 
                // This overrides the [consumes](#swaggerConsumes) definition at the Swagger Object. 
                // An empty value MAY be used to clear the global definition. 
                // Value MUST be as described under Mime Types.
                def.array("consumes", |consumes| {});    

                // A list of MIME types the operation can produce. 
                // This overrides the [produces](#swaggerProduces) definition at the Swagger Object. 
                // An empty value MAY be used to clear the global definition. 
                // Value MUST be as described under Mime Types.
                def.array("produces", |produces| {});

                // A list of parameters that are applicable for this operation. 
                // If a parameter is already defined at the Path Item, the new definition will override it, 
                // but can never remove it. The list MUST NOT include duplicated parameters. 
                // A unique parameter is defined by a combination of a name and location. 
                // The list can use the Reference Object to link to parameters that are 
                // defined at the Swagger Object's parameters. There can be one "body" parameter at most.
                def.array("parameters", |parameters| {});

                // Required. The list of possible responses as they are returned from executing this operation.
                def.object("responses",  |responses| {
                    responses.object("200", |default| {
                        // Required. A short description of the response. 
                        // GFM syntax can be used for rich text representation.
                        default.set("description", "Description".to_string());

                        // A definition of the response structure. It can be a primitive, an array or an object. 
                        // If this field does not exist, it means no content is returned as part of the response. 
                        // As an extension to the Schema Object, its root type value may also be "file". 
                        // This SHOULD be accompanied by a relevant produces mime-type.
                        default.object("schema", |schema| {});

                        // A list of headers that are sent with the response.
                        default.object("headers", |headers| {});

                        // An example of the response message.
                        default.object("examples", |examples| {});
                    })
                });

                // The transfer protocol for the operation. Values MUST be from the list: "http", "https", 
                // "ws", "wss". The value overrides the Swagger Object schemes definition.
                def.array("schemes", |schemes| {});

                // Declares this operation to be deprecated. Usage of the declared operation should be refrained. 
                // Default value is false.
                def.set("deprecated", false);

                // A declaration of which security schemes are applied for this operation. 
                // The list of values describes alternative security schemes that can be used 
                // (that is, there is a logical OR between the security requirements). 
                // This definition overrides any declared top-level security. 
                // To remove a top-level security declaration, an empty array can be used.
                def.array("security", |security| {});
            });

            let method = format!("{:?}", endpoint.method).to_ascii_lowercase();
            let present = {
                let maybe_path_obj = paths.object.get_mut(path.as_slice());
                if maybe_path_obj.is_some() {
                    let path_obj = maybe_path_obj.unwrap().as_object_mut().unwrap();
                    path_obj.insert(method.to_string(), definition.to_json());
                    true
                } else {
                    false
                }
            };
            
            if !present {
                paths.object(path.as_slice(), move |: path_item| {
                    path_item.set(method, definition);
                })
            }

        }
    }
}

#[allow(dead_code)]
#[allow(unused_variables)]
pub fn create_swagger_api(path: &str) -> framework::Api {
    framework::Api::build(|api| {
        api.namespace(path, |docs| {
            docs.options("", |endpoint| {
                endpoint.handle(|&: mut client, _params| {
                    client.set_header(allow_origin::AccessControlAllowOrigin::AllowStar);
                    client.empty()
                })
            });
            docs.get("", |endpoint| {
                endpoint.handle(|&: mut client, _params| {
                    client.set_header(allow_origin::AccessControlAllowOrigin::AllowStar);
                    let swagger_data = &jsonway::JsonWay::object(|json| {
                        // Required. Specifies the Swagger Specification version being used. 
                        // It can be used by the Swagger UI and other clients to interpret the API listing. 
                        // The value MUST be "2.0".
                        json.set("swagger", "2.0".to_string());

                        // Required. Provides metadata about the API. The metadata can be used by the clients if needed.
                        json.object("info", |info| {
                            // Required. The title of the application.
                            info.set("title", "Rustless API".to_string());

                            // A short description of the application. GFM syntax can be used for rich text representation.
                            info.set("description", "Rustless API description".to_string());

                            // The Terms of Service for the API.
                            info.set("termsOfService", "Terms of service".to_string()); 

                            // The contact information for the exposed API.
                            info.object("contact", |contact| {
                                // The identifying name of the contact person/organization.
                                contact.set("name", "API owner".to_string());
                                // The URL pointing to the contact information. MUST be in the format of a URL.
                                contact.set("url", "http://localhost".to_string());
                                // The email address of the contact person/organization. MUST be in the format of an email address.
                                contact.set("email", "admin@example.com".to_string());  
                            });

                            info.object("license", |license| {
                                // The license name used for the API.
                                license.set("name", "MIT".to_string());

                                // An URL to the license used for the API. MUST be in the format of a URL.
                                license.set("url", "http://opensource.org/licenses/MIT".to_string());
                            });

                            // Required Provides the version of the application API (not to be confused by the specification version).
                            info.set("version", "1.0".to_string());
                        });

                        // The host (name or ip) serving the API. This MUST be the host only and does not 
                        // include the scheme nor sub-paths. It MAY include a port. If the host is not included, 
                        // the host serving the documentation is to be used (including the port). 
                        // The host does not support path templating.
                        json.set("host", "localhost:4000".to_string());

                        // The base path on which the API is served, which is relative to the host. 
                        // If it is not included, the API is served directly under the host. 
                        // The value MUST start with a leading slash (/). The basePath does not support path
                        // templating.
                        json.set("basePath", "/".to_string());

                        // The transfer protocol of the API. Values MUST be from the list: 
                        // "http", "https", "ws", "wss". If the schemes is not included, the default 
                        // scheme to be used is the one used to access the specification.
                        json.array("schemes", |schemes| {
                            schemes.push("http".to_string())
                        });

                        // A list of MIME types the APIs can consume. This is global to all APIs but can be 
                        // overridden on specific API calls. Value MUST be as described under Mime Types.
                        json.array("consumes", |consumes| {
                            consumes.push("application/json".to_string())
                        });   

                        // A list of MIME types the APIs can produce. This is global to all APIs but can 
                        // be overridden on specific API calls. Value MUST be as described under Mime Types.
                        json.array("produces", |produces| {
                            produces.push("application/json".to_string())
                        });  

                        // Required. The available paths and operations for the API.
                        json.object("paths", |paths| {
                            fill_paths("", paths, &client.app.root_api.handlers);
                        });

                         // An object to hold data types produced and consumed by operations.
                        json.object("definitions", |definitions| {

                        });

                        // An object to hold parameters that can be used across operations. 
                        // This property does not define global parameters for all operations.
                        json.object("parameters", |parameters| {

                        });

                        // An object to hold responses that can be used across operations. 
                        // This property does not define global responses for all operations.
                        json.object("responses", |responses| {

                        });

                        // Security scheme definitions that can be used across the specification.
                        json.object("securityDefinitions", |security_definitions| {

                        });

                        // A declaration of which security schemes are applied for the API as a whole. 
                        // The list of values describes alternative security schemes that can be used 
                        // (that is, there is a logical OR between the security requirements). 
                        // Individual operations can override this definition.
                        json.array("security", |security| {

                        });

                        // A list of tags used by the specification with additional metadata. 
                        // The order of the tags can be used to reflect on their order by the parsing tools. 
                        // Not all tags that are used by the Operation Object must be declared. 
                        // The tags that are not declared may be organized randomly or based on the tools' logic. 
                        // Each tag name in the list MUST be unique.
                        json.array("tags", |tags| {

                        });

                        // Additional external documentation.
                        json.object("externalDocs", |external_docs| {

                        })   
                    }).unwrap();
                    client.json(swagger_data)
                })
            })
        })
    })
}