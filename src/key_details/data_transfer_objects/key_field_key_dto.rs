use std::fmt;

use serde::{Deserialize, Serialize};

/// The stable identifier of a key field, e.g. `team`. Serialized as the plain
/// string.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct KeyFieldKeyDTO(pub String);

impl From<String> for KeyFieldKeyDTO {
    fn from(key: String) -> Self {
        Self(key)
    }
}

impl fmt::Display for KeyFieldKeyDTO {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}
