use valico;
use typemap;

pub struct JsonSchemaScope;

impl typemap::Key for JsonSchemaScope {
    type Value = valico::json_schema::Scope;
}