use serde::{Deserialize, Serialize};

/// A key's value for a field: a boolean, a text, or a select field's option
/// key. A value of a kind the CLI does not know yet is kept as `Unknown`
/// instead of failing the whole response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum KeyFieldValueDTO {
    Boolean(bool),
    Text(String),
    Unknown(serde_json::Value),
}
