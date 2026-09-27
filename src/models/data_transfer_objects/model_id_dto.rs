use std::fmt;

use serde::{Deserialize, Serialize};

/// The name a request sends as `model`: a gateway model or an alias, e.g.
/// `zdev` or `anthropic/claude-opus-5.5`. Serialized as the plain string.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ModelIdDTO(pub String);

impl From<String> for ModelIdDTO {
    fn from(id: String) -> Self {
        Self(id)
    }
}

impl fmt::Display for ModelIdDTO {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}
