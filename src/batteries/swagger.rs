use std::ascii::AsciiExt;
use serialize::json::{self, ToJson};
use jsonway::{self, MutableJson};
use framework::{self, Nesting};
use server::mime;
use server::header::common::access_control::allow_origin;

#[derive(Copy)]
#[allow(dead_code)]
pub enum Scheme {
    Http,
    Https,
    Ws,
    Wss,
}

impl ToString for Scheme {
    fn to_string(&self) -> String {
        match self {
            &Scheme::Http => "http",
            &Scheme::Https => "https",
            &Scheme::Ws => "ws",
            &Scheme::Wss => "wss"
        }.to_string()
    }
}

#[allow(dead_code)]
#[derive(Default)]
pub struct Contact {
    pub name: String,
    pub url: Option<String>,
    pub email: Option<String>
}

#[allow(dead_code)]
#[derive(Default)]
pub struct Spec {
    pub info: Info,
    pub host: Option<String>,
    pub base_path: Option<String>,
    pub schemes: Option<Vec<Scheme>>,
    pub consumes: Option<Vec<mime::Mime>>,
    pub produces: Option<Vec<mime::Mime>>,
}

#[allow(dead_code)]
#[derive(Default)]
pub struct Info {
    pub title: String,
    pub description: Option<String>,
    pub terms_of_service: Option<String>,
    pub contact: Option<Contact>,
    pub license: Option<License>,
    pub version: Option<String>
}

#[allow(dead_code)]
#[derive(Default)]
pub struct License {
    pub name: String,
    pub url: String
}

#[derive(Copy)]
pub struct SwaggerSpecKey;
impl ::typemap::Key for SwaggerSpecKey {
    type Value = json::Json;
}

pub fn enable(app: &mut framework::Application, spec: Spec) {
    let spec = build_spec(app, spec);
    app.ext.insert::<SwaggerSpecKey>(spec);
}

#[allow(unused_variables)]
pub fn build_spec(app: &framework::Application, spec: Spec) -> json::Json {
    jsonway::JsonWay::object(|&: json| {
        // Required. Specifies the Swagger Specification version being used. 
        // It can be used by the Swagger UI and other clients to interpret the API listing. 
        // The value MUST be "2.0".
        json.set("swagger", "2.0".to_string());

        // Required. Provides metadata about the API. The metadata can be used by the clients if needed.
        json.object("info", |info| {
            // Required. The title of the application.
            info.set("title", spec.info.title.clone());

            if spec.info.description.is_some() {
                // A short description of the application. GFM syntax can be used for rich text representation.
                info.set("description", spec.info.description.as_ref().unwrap().clone());
            }

            if spec.info.terms_of_service.is_some() {
                // The Terms of Service for the API.
                info.set("termsOfService", spec.info.terms_of_service.as_ref().unwrap().clone());  
            }

            if spec.info.contact.is_some() {
                let contact_spec = spec.info.contact.as_ref().unwrap();
                // The contact information for the exposed API.
                info.object("contact", |contact| {
                    // The identifying name of the contact person/organization.
                    contact.set("name", contact_spec.name.clone());

                    if contact_spec.url.is_some() {
                        // The URL pointing to the contact information. MUST be in the format of a URL.
                        contact.set("url", contact_spec.url.as_ref().unwrap().clone());   
                    }
                    
                    if contact_spec.email.is_some() {
                        // The email address of the contact person/organization. MUST be in the format of an email address.
                        contact.set("email", contact_spec.email.as_ref().unwrap().clone());
                    }
                });
            }

            if spec.info.license.is_some() {
                let license_spec = spec.info.license.as_ref().unwrap();

                info.object("license", |license| {
                    // The license name used for the API.
                    license.set("name", license_spec.name.clone());
                    // An URL to the license used for the API. MUST be in the format of a URL.
                    license.set("url", license_spec.url.clone());    
                });
            }

            // Required. Provides the version of the application API (not to be confused by the specification version).
            info.set("version", spec.info.version.clone()
                .or(app.root_api.version.clone())
                .unwrap_or_else(|| "0.0.0".to_string()));
        });

        if spec.host.is_some() {
            // The host (name or ip) serving the API. This MUST be the host only and does not 
            // include the scheme nor sub-paths. It MAY include a port. If the host is not included, 
            // the host serving the documentation is to be used (including the port). 
            // The host does not support path templating.
            json.set("host", spec.host.as_ref().unwrap().clone());   
        }

        // The base path on which the API is served, which is relative to the host. 
        // If it is not included, the API is served directly under the host. 
        // The value MUST start with a leading slash (/). The basePath does not support path
        // templating.
        json.set("basePath", spec.base_path.clone().unwrap_or_else(|| {
            let mut base_path = "/".to_string() + app.root_api.prefix.as_slice();
            if app.root_api.versioning.is_some() {
                match app.root_api.versioning.as_ref().unwrap() {
                    &framework::Versioning::Path if app.root_api.version.is_some() => {
                        if base_path.len() > 1 {
                            base_path.push_str("/")
                        }
                        base_path.push_str(app.root_api.version.as_ref().unwrap().as_slice());
                    },
                    _ => ()
                }
            }

            base_path
        }));

        // The transfer protocol of the API. Values MUST be from the list: 
        // "http", "https", "ws", "wss". If the schemes is not included, the default 
        // scheme to be used is the one used to access the specification.
        if spec.schemes.is_some() {
            let schemes_spec = spec.schemes.as_ref().unwrap();
            json.array("schemes", |schemes| {
                for scheme in schemes_spec.iter() {
                    schemes.push(scheme.to_string())
                }
            })
        }

        if spec.consumes.is_some() {
            let consumes_spec = spec.consumes.as_ref().unwrap();
            // A list of MIME types the APIs can consume. This is global to all APIs but can be 
            // overridden on specific API calls. Value MUST be as described under Mime Types.
            json.array("consumes", |consumes| {
                for mime in consumes_spec.iter() {
                    consumes.push(mime.to_string())
                }
            }); 
        }

        if spec.produces.is_some() {
            let produces_spec = spec.produces.as_ref().unwrap();
            // A list of MIME types the APIs can produce. This is global to all APIs but can 
            // be overridden on specific API calls. Value MUST be as described under Mime Types.
            json.array("produces", |produces| {
                for mime in produces_spec.iter() {
                    produces.push(mime.to_string())
                }
            }); 
        }

        // Required. The available paths and operations for the API.
        json.object("paths", |paths| {
            fill_paths("", paths, &app.root_api.handlers);
        });

        // TODO Implement the rest of the spec

        // // An object to hold data types produced and consumed by operations.
        // json.object("definitions", |definitions| {

        // });

        // // An object to hold parameters that can be used across operations. 
        // // This property does not define global parameters for all operations.
        // json.object("parameters", |parameters| {

        // });

        // // An object to hold responses that can be used across operations. 
        // // This property does not define global responses for all operations.
        // json.object("responses", |responses| {

        // });

        // // Security scheme definitions that can be used across the specification.
        // json.object("securityDefinitions", |security_definitions| {

        // });

        // // A declaration of which security schemes are applied for the API as a whole. 
        // // The list of values describes alternative security schemes that can be used 
        // // (that is, there is a logical OR between the security requirements). 
        // // Individual operations can override this definition.
        // json.array("security", |security| {

        // });

        // // A list of tags used by the specification with additional metadata. 
        // // The order of the tags can be used to reflect on their order by the parsing tools. 
        // // Not all tags that are used by the Operation Object must be declared. 
        // // The tags that are not declared may be organized randomly or based on the tools' logic. 
        // // Each tag name in the list MUST be unique.
        // json.array("tags", |tags| {

        // });

        // // Additional external documentation.
        // json.object("externalDocs", |external_docs| {

        // })  

    }).unwrap()
}

#[allow(unused_variables)]
pub fn create_api(path: &str) -> framework::Api {
    framework::Api::build(|: api| {
        api.namespace(path, |: docs| {
            docs.get("", |: endpoint| {
                endpoint.summary("Get Swagger 2.0 specification of this API");
                endpoint.handle(|&: mut client, _params| {
                    client.set_header(allow_origin::AccessControlAllowOrigin::AllowStar);
                    let swagger_spec = client.app.ext.get::<SwaggerSpecKey>();
                    if swagger_spec.is_some() {
                        client.json(swagger_spec.unwrap())
                    } else {
                        client.empty()
                    }
                })
            })
        })
    })
}

#[allow(unused_variables)]
fn fill_paths(current_path: &str, paths: &mut jsonway::ObjectBuilder, handlers: &framework::ApiHandlers) {
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

                if endpoint.summary.is_some() {
                    // A short summary of what the operation does. For maximum readability in the swagger-ui, 
                    // this field SHOULD be less than 120 characters.
                    def.set("summary", endpoint.summary.as_ref().unwrap().clone());  
                }

                if endpoint.desc.is_some() {
                    // A verbose explanation of the operation behavior. 
                    // GFM syntax can be used for rich text representation.
                    def.set("description",  endpoint.desc.as_ref().unwrap().clone());
                }

                // Required. The list of possible responses as they are returned from executing this operation.
                def.object("responses",  |responses| {
                    responses.object("200", |default| {
                        // Required. A short description of the response. 
                        // GFM syntax can be used for rich text representation.
                        default.set("description", "Default response".to_string());

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

                // TODO Implement the rest of the Swagger 2.0 spec

                // // External Documentation Object   
                // // Additional external documentation for this operation.
                // def.object("externalDocs", |external_docs| {
                //     // A short description of the target documentation. 
                //     // GFM syntax can be used for rich text representation.
                //     external_docs.set("description", "Description".to_string()); 
                //     //  Required. The URL for the target documentation. Value MUST be in the format of a URL.
                //     external_docs.set("url", "http://google.com".to_string());
                // });

                // // A friendly name for the operation. The id MUST be unique among all operations described 
                // // in the API. Tools and libraries MAY use the operation id to uniquely identify an operation.
                // def.set("operationId", "OP".to_string());

                // // A list of MIME types the operation can consume. 
                // // This overrides the [consumes](#swaggerConsumes) definition at the Swagger Object. 
                // // An empty value MAY be used to clear the global definition. 
                // // Value MUST be as described under Mime Types.
                // def.array("consumes", |consumes| {});    

                // // A list of MIME types the operation can produce. 
                // // This overrides the [produces](#swaggerProduces) definition at the Swagger Object. 
                // // An empty value MAY be used to clear the global definition. 
                // // Value MUST be as described under Mime Types.
                // def.array("produces", |produces| {});

                // // A list of parameters that are applicable for this operation. 
                // // If a parameter is already defined at the Path Item, the new definition will override it, 
                // // but can never remove it. The list MUST NOT include duplicated parameters. 
                // // A unique parameter is defined by a combination of a name and location. 
                // // The list can use the Reference Object to link to parameters that are 
                // // defined at the Swagger Object's parameters. There can be one "body" parameter at most.
                // def.array("parameters", |parameters| {});

                // // The transfer protocol for the operation. Values MUST be from the list: "http", "https", 
                // // "ws", "wss". The value overrides the Swagger Object schemes definition.
                // def.array("schemes", |schemes| {});

                // // Declares this operation to be deprecated. Usage of the declared operation should be refrained. 
                // // Default value is false.
                // def.set("deprecated", false);

                // // A declaration of which security schemes are applied for this operation. 
                // // The list of values describes alternative security schemes that can be used 
                // // (that is, there is a logical OR between the security requirements). 
                // // This definition overrides any declared top-level security. 
                // // To remove a top-level security declaration, an empty array can be used.
                // def.array("security", |security| {});
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