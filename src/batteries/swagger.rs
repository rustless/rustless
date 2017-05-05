use std::ascii::AsciiExt;
use valico::json_dsl;
use std::collections;
use jsonway::{self};

use json::{self, JsonValue, ToJson};
use framework::{self, Nesting};
use server::mime;
use server::header;
use server::method;

#[allow(dead_code)]
/// The transfer protocol for the operation. Values MUST be from the list: "http", "https", "ws", "wss".
/// The value overrides the Swagger Object schemes definition.
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
/// Contact information for the exposed API.
pub struct Contact {
    pub name: String,
    pub url: Option<String>,
    pub email: Option<String>
}

#[allow(dead_code)]
#[derive(Default)]
/// This is the root document object for the API specification.
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
/// The object provides metadata about the API. The metadata can be used by the clients
/// if needed, and can be presented in the Swagger-UI for convenience.
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
/// License information for the exposed API.
pub struct License {
    pub name: String,
    pub url: String
}

#[allow(dead_code)]
#[derive(Clone)]
/// The location of the parameter. Possible values are
/// "query", "header", "path", "formData" or "body".
enum Place {
    Query,
    Header,
    Path,
    FormData,
    Body
}

impl ToString for Place {
    fn to_string(&self) -> String {
        match self {
            &Place::Query => "query",
            &Place::Header => "header",
            &Place::Path => "path",
            &Place::FormData => "formData",
            &Place::Body => "body"
        }.to_string()
    }
}

#[allow(dead_code)]
#[derive(Clone)]
/// The type of the parameter. Since the parameter is not located at the request body,
/// it is limited to simple types (that is, not an object).
/// The value MUST be one of "string", "number", "integer", "boolean", "array" or "file".
enum ParamType {
    String,
    Number,
    Integer,
    Boolean,
    Array,
    File
}

impl ToString for ParamType {
    fn to_string(&self) -> String {
        match self {
            &ParamType::String => "string",
            &ParamType::Number => "number",
            &ParamType::Integer => "integer",
            &ParamType::Boolean => "boolean",
            &ParamType::Array => "array",
            &ParamType::File => "file"
        }.to_string()
    }
}

#[allow(dead_code)]
#[derive(Clone)]
/// The internal type of the array.
/// The value MUST be one of "string", "number", "integer", "boolean", or "array".
/// Files and models are not allowed.
enum ItemType {
    String,
    Number,
    Integer,
    Boolean,
    Array
}

impl ToString for ItemType {
    fn to_string(&self) -> String {
        match self {
            &ItemType::String => "string",
            &ItemType::Number => "number",
            &ItemType::Integer => "integer",
            &ItemType::Boolean => "boolean",
            &ItemType::Array => "array"
        }.to_string()
    }
}

#[allow(dead_code)]
#[derive(Clone)]
/// An limited subset of JSON-Schema's items object.
/// It is used by parameter definitions that are not located in "body".
struct ItemParams {
    pub type_: ItemType,
    pub format: Option<String>,
    pub items: Option<Vec<ItemParams>>
}

#[allow(dead_code)]
#[derive(Clone)]
/// Describes a single operation parameter extension fields.
enum ParamExt {
    BodyParam(json::Object),
    NormalParam {
        type_: ParamType,
        format: Option<String>,
        items: Option<Vec<ItemParams>>,
    }
}

#[allow(dead_code)]
#[derive(Clone)]
/// Describes a single operation parameter.
struct Param {
    pub name: String,
    pub place: Place,
    pub description: Option<String>,
    pub required: bool,
    pub ext: ParamExt
}

impl ToJson for Param {
    fn to_json(&self) -> JsonValue {
        jsonway::object(|param| {
            param.set("name", self.name.clone());
            param.set("in", self.place.to_string());
            if self.description.is_some() {
                param.set("description", self.description.clone().unwrap());
            }
            param.set("required", self.required);
            match &self.ext {
                &ParamExt::BodyParam(ref schema) => param.set("schema", schema.clone()),
                &ParamExt::NormalParam{ref type_, ref format, ..} => {
                    param.set("type", type_.to_string());
                    if format.is_some() {
                        param.set("format", format.clone().unwrap())
                    }
                    // TODO items
                }
            }
        }).to_json()
    }
}

pub struct SwaggerSpecKey;
impl ::typemap::Key for SwaggerSpecKey {
    type Value = JsonValue;
}

pub fn enable(app: &mut framework::Application, spec: Spec) {
    let spec = build_spec(app, spec);
    app.ext.insert::<SwaggerSpecKey>(spec);
}

#[allow(unused_variables)]
/// Build the basic Swagger 2.0 object
pub fn build_spec(app: &framework::Application, spec: Spec) -> JsonValue {
    jsonway::object(|json| {
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
                .or(app.root_api.version.clone().map(|v| v.version))
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
            // Here we are generating the `basePath` from prefix and version
            // (if versioning strategy is Path)

            let mut base_path = "/".to_string();
            if app.root_api.prefix.is_some() {
                base_path.push_str(&app.root_api.prefix.as_ref().unwrap());
            }
            if app.root_api.version.is_some() {
                match app.root_api.version.as_ref().unwrap() {
                    &framework::Version{ref version, versioning: framework::Versioning::Path}  => {
                        if base_path.len() > 1 {
                            base_path.push_str("/")
                        }
                        base_path.push_str(&version);
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
            fill_paths(WalkContext {
                path: "",
                params: vec![]
            }, paths, &app.root_api.handlers);
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
/// Create an API to handle doc requests
pub fn create_api(path: &str) -> framework::Api {
    framework::Api::build(|api| {
        api.namespace(path, |docs| {
            docs.get("", |endpoint| {
                endpoint.summary("Get Swagger 2.0 specification of this API");
                endpoint.handle(|mut client, _params| {
                    client.set_header(header::AccessControlAllowOrigin::Any);
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

#[allow(dead_code)]
struct WalkContext<'a> {
    pub path: &'a str,
    pub params: Vec<Param>
}

/// Walks through the tree and collects the info about Endpoints
fn fill_paths<'a>(mut context: WalkContext<'a>, paths: &mut jsonway::ObjectBuilder, handlers: &framework::ApiHandlers) {
    for handler_ in handlers.iter() {
        let handler = &**handler_ as &framework::ApiHandler;
        if handler.is::<framework::Api>() {
            let mut path = context.path.to_string();

            let api = handler.downcast::<framework::Api>().unwrap();
            if api.prefix.is_some() {
                path.push_str(&api.prefix.as_ref().unwrap());
            }

            if api.version.is_some() {
                match api.version.as_ref().unwrap() {
                    &framework::Version{ref version, versioning: framework::Versioning::Path} => {
                        path.push_str(&version);
                        path.push_str("/");
                    },
                    _ => ()
                }
            }

            fill_paths(WalkContext{
                path: &path,
                params: context.params.clone()
            }, paths, &api.handlers);

        } else if handler.is::<framework::Namespace>() {

            let mut path = context.path.to_string();
            let namespace = handler.downcast::<framework::Namespace>().unwrap();
            path.push_str(&("/".to_string() + &encode_path_string(&namespace.path)));

            let mut params = context.params.clone();
            params.extend(extract_params(&namespace.coercer, &namespace.path));

            fill_paths(WalkContext{
                path: &path,
                params: params,
            }, paths, &namespace.handlers);

        } else if handler.is::<framework::Endpoint>() {
            let mut path = context.path.to_string();
            let endpoint = handler.downcast::<framework::Endpoint>().unwrap();

            if endpoint.path.path.len() > 0 {
                path.push_str(&("/".to_string() + &encode_path_string(&endpoint.path)));
            }

            let definition = build_endpoint_definition(endpoint, &mut context);

            let method = format!("{:?}", endpoint.method).to_ascii_lowercase();
            let exists = {
                let maybe_path_obj = paths.object.get_mut(&path);
                if maybe_path_obj.is_some() {
                    let path_obj = maybe_path_obj.unwrap().as_object_mut().unwrap();
                    path_obj.insert(method.to_string(), definition.to_json());
                    true
                } else {
                    false
                }
            };

            if !exists {
                paths.object(path.to_string(), |path_item| {
                    path_item.set(method.clone(), definition.to_json());
                    path_item.array("parameters", |parameters| {
                        for param in context.params.iter() {
                            parameters.push(param.to_json())
                        }
                    })
                });
            }
        }
    }
}

#[allow(unused_variables)]
/// Creates Endpoint definition according to Swagger 2.0 specification
fn build_endpoint_definition(endpoint: &framework::Endpoint, context: &mut WalkContext) -> JsonValue {
    jsonway::object(|def| {
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

        if endpoint.consumes.is_some() {
            // A list of MIME types the operation can consume.
            // This overrides the [consumes](#swaggerConsumes) definition at the Swagger Object.
            // An empty value MAY be used to clear the global definition.
            // Value MUST be as described under Mime Types.
            def.array("consumes", |consumes| {
                let consumes_spec = endpoint.consumes.as_ref().unwrap();
                for mime in consumes_spec.iter() {
                    consumes.push(mime.to_string())
                }
            });
        }

        if endpoint.produces.is_some() {
            // A list of MIME types the operation can produce.
            // This overrides the [produces](#swaggerProduces) definition at the Swagger Object.
            // An empty value MAY be used to clear the global definition.
            // Value MUST be as described under Mime Types.
            def.array("produces", |produces| {
                let produces_spec = endpoint.produces.as_ref().unwrap();
                for mime in produces_spec.iter() {
                    produces.push(mime.to_string())
                }
            });
        }

        // A list of parameters that are applicable for this operation.
        // If a parameter is already defined at the Path Item, the new definition will override it,
        // but can never remove it. The list MUST NOT include duplicated parameters.
        // A unique parameter is defined by a combination of a name and location.
        // The list can use the Reference Object to link to parameters that are
        // defined at the Swagger Object's parameters. There can be one "body" parameter at most.
        def.array("parameters", |parameters| {
            let params = extract_params(&endpoint.coercer, &endpoint.path);
            let mut final_params = vec![];
            for param in context.params.iter() {
                final_params.push(param.clone())
            }
            for param in params.iter() {
                final_params.push(param.clone())
            }

            match endpoint.method {
                method::Method::Post |
                method::Method::Put |
                method::Method::Patch => {
                    for param in final_params.iter_mut() {
                        match param.place {
                            Place::Query => param.place = Place::FormData,
                            _ => ()
                        }
                    }
                },
                _ => ()
            };

            parameters.map(final_params.iter(), |param| param.to_json());
        });

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
    }).unwrap()
}

/// Encodes path string to Swagger 2.0 format (e.g. '/user/:user_id' becomes '/user/{user_id}')
fn encode_path_string(path: &framework::Path) -> String {
    let ref original_path = path.path;
    return framework::path::MATCHER.replace_all(&original_path, "{$1}");
}

/// Converts `valico::Param` into Swagger's ParamType
fn param_type(param: &json_dsl::Param) -> ParamType {
    match &param.coercer {
        &Some(ref coercer) => {
            match coercer.get_primitive_type() {
                json_dsl::PrimitiveType::String => ParamType::String,
                json_dsl::PrimitiveType::I64 => ParamType::Integer,
                json_dsl::PrimitiveType::F64 => ParamType::Number,
                json_dsl::PrimitiveType::Array => ParamType::Array,
                json_dsl::PrimitiveType::Boolean => ParamType::Boolean,
                json_dsl::PrimitiveType::File => ParamType::File,
                _ => ParamType::String
            }
        },
        &None => ParamType::String
    }
}

/// Crate Swagger's `Param` from `valico::Param`
fn build_param_from_coercer(param: &json_dsl::Param, required: bool) -> Param {
    let swagger_param = Param {
        name: param.name.clone(),
        place: Place::Query,
        description: param.description.clone(),
        required: required,
        ext: ParamExt::NormalParam {
            type_: param_type(param),
            format: None,
            items: None
        }
    };

    swagger_param
}

/// Translates information from `coercer` and `path` to list of Swagger's Param objects
fn extract_params(coercer: &Option<json_dsl::Builder>, path: &framework::Path) -> Vec<Param> {
    let mut params = collections::BTreeMap::new();

    if coercer.is_some() {
        let coercer = coercer.as_ref().unwrap();
        for param in coercer.get_required().iter() {
            params.insert(param.name.clone(), build_param_from_coercer(param, true));
        }
        for param in coercer.get_optional().iter() {
            params.insert(param.name.clone(), build_param_from_coercer(param, false));
        }
    }

    for param_name in path.params.iter() {
        let exists = {
            let mut existing_param = params.get_mut(param_name);
            if existing_param.is_some() {
                let param = existing_param.as_mut().unwrap();
                param.place = Place::Path;
                param.required = true;
                true
            } else {
                false
            }
        };

        if !exists {
            let param = Param {
                name: param_name.clone(),
                place: Place::Path,
                description: None,
                required: true,
                ext: ParamExt::NormalParam {
                    type_: ParamType::String,
                    format: None,
                    items: None
                }
            };

            params.insert(param_name.clone(), param);
        }
    }

    params.into_iter().map(|(_key, value)| value).collect::<Vec<Param>>()
}
