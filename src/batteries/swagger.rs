
use jsonway;
use framework::{self, Nesting};
use server::header::common::access_control::allow_origin;

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
                    client.json(&jsonway::JsonWay::object(|json| {
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
                        json.array("security", |responses| {

                        });

                        // Additional external documentation.
                        json.object("externalDocs", |external_docs| {

                        })   
                    }).unwrap())
                })
            })
        })
    })
}