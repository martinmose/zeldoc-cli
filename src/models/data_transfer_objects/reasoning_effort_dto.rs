use std::fmt;

use serde::{Deserialize, Serialize};

/// A `reasoning_effort` value. A value the CLI does not know yet is kept as
/// `Unknown` instead of failing the whole catalog.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReasoningEffortDTO {
    None,
    Minimal,
    Low,
    Medium,
    High,
    Xhigh,
    Max,
    #[serde(untagged)]
    Unknown(String),
}

impl fmt::Display for ReasoningEffortDTO {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::None => "none",
            Self::Minimal => "minimal",
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::Xhigh => "xhigh",
            Self::Max => "max",
            Self::Unknown(effort) => effort,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn efforts_round_trip_as_their_api_names() {
        let json = r#"["none","low","xhigh","max","ultra"]"#;
        let efforts: Vec<ReasoningEffortDTO> = serde_json::from_str(json).unwrap();
        assert_eq!(
            efforts,
            [
                ReasoningEffortDTO::None,
                ReasoningEffortDTO::Low,
                ReasoningEffortDTO::Xhigh,
                ReasoningEffortDTO::Max,
                ReasoningEffortDTO::Unknown("ultra".to_string()),
            ]
        );
        assert_eq!(serde_json::to_string(&efforts).unwrap(), json);
    }
}
