use std::fmt;

use serde::{Deserialize, Serialize};

/// A ZDev plan. A plan the CLI does not know yet is kept as `Unknown` instead
/// of failing the whole report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ZDevPlanDTO {
    Go,
    Pro,
    Max,
    #[serde(untagged)]
    Unknown(String),
}

impl fmt::Display for ZDevPlanDTO {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Go => "Go",
            Self::Pro => "Pro",
            Self::Max => "Max",
            Self::Unknown(plan) => plan,
        })
    }
}
