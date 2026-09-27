use serde::{Deserialize, Serialize};

/// One search result. `url` and `date` stay strings: they come from many
/// engines, and one malformed value must not fail the whole response.
#[derive(Debug, Deserialize, Serialize)]
pub struct SearchResultDTO {
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub url: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub snippet: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub date: Option<String>,
}
