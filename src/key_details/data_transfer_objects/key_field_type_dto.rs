use std::fmt;

use serde::{Deserialize, Serialize};

/// What a key field holds. A type the CLI does not know yet is kept as
/// `Unknown` instead of failing the whole response.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KeyFieldTypeDTO {
    Text,
    Boolean,
    Select,
    #[serde(untagged)]
    Unknown(String),
}

impl fmt::Display for KeyFieldTypeDTO {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Text => "text",
            Self::Boolean => "boolean",
            Self::Select => "select",
            Self::Unknown(field_type) => field_type,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_and_unknown_types_round_trip() {
        for (json, field_type) in [
            (r#""text""#, KeyFieldTypeDTO::Text),
            (r#""boolean""#, KeyFieldTypeDTO::Boolean),
            (r#""select""#, KeyFieldTypeDTO::Select),
            (r#""date""#, KeyFieldTypeDTO::Unknown("date".to_string())),
        ] {
            assert_eq!(
                serde_json::from_str::<KeyFieldTypeDTO>(json).unwrap(),
                field_type
            );
            assert_eq!(serde_json::to_string(&field_type).unwrap(), json);
        }
    }
}
