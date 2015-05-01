use valico::json_schema;
use typemap;

use super::super::framework;

pub struct SchemesScope;

impl typemap::Key for SchemesScope {
    type Value = json_schema::Scope;
}

fn build_schemes(handlers: &mut framework::ApiHandlers, scope: &mut json_schema::Scope) -> Result<(), json_schema::SchemaError> {
    for handler_ in handlers.iter_mut() {
        let mut handler = &mut **handler_ as &mut framework::ApiHandler;
        if handler.is::<framework::Api>() {
            let api = handler.downcast_mut::<framework::Api>().unwrap();
            try!(build_schemes(&mut api.handlers, scope))
        } else if handler.is::<framework::Namespace>() {
            let namespace = handler.downcast_mut::<framework::Namespace>().unwrap();
            if namespace.coercer.is_some() {
                let coercer = namespace.coercer.as_mut().unwrap();
                try!(coercer.build_schemes(scope));
            }
            try!(build_schemes(&mut namespace.handlers, scope));
        } else if handler.is::<framework::Endpoint>() {
            let endpoint = handler.downcast_mut::<framework::Endpoint>().unwrap();
            if endpoint.coercer.is_some() {
                let coercer = endpoint.coercer.as_mut().unwrap();
                try!(coercer.build_schemes(scope));
            }
        }
    }

    Ok(())
}

pub fn enable_schemes(app: &mut framework::Application, mut scope: json_schema::Scope) -> Result<(), json_schema::SchemaError> {
    try!(build_schemes(&mut app.root_api.handlers, &mut scope));
    app.ext.insert::<SchemesScope>(scope);
    Ok(())
}