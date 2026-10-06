use serde::{Deserialize, Serialize};

use super::key_field_key_dto::KeyFieldKeyDTO;
use super::key_field_type_dto::KeyFieldTypeDTO;
use super::key_field_value_dto::KeyFieldValueDTO;

/// One key field of the organization and the key's value for it.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct KeyFieldDTO {
    pub key: KeyFieldKeyDTO,
    /// The field's name, e.g. "Team".
    pub display_name: String,
    #[serde(rename = "type")]
    pub field_type: KeyFieldTypeDTO,
    /// None when the key has no value for the field.
    pub value: Option<KeyFieldValueDTO>,
    /// The chosen option's label for a select field, e.g. "Backend".
    pub value_label: Option<String>,
}
