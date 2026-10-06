use serde::{Deserialize, Serialize};

use super::key_field_dto::KeyFieldDTO;

/// The API key as its organization labelled it (`GET /v1/zeldoc/key`).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct KeyDetailsDTO {
    /// The key's name, as the organization's admins gave it.
    pub key_name: Option<String>,
    /// Every key field of the organization, in display order, with this
    /// key's value.
    pub fields: Vec<KeyFieldDTO>,
}
