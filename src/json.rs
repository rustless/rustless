pub use serde_json::{Value as JsonValue, to_value, to_string};
pub use serde_json::value::{ToJson};
use std::collections::{BTreeMap};

pub type Object = BTreeMap<String, JsonValue>;
pub type Array = Vec<JsonValue>;
