use serde::Serialize;

use super::time_range::TimeRange;

/// The body of `POST /v1/search/zeldoc-search`. `engines` and `categories`
/// are comma-separated, as the endpoint expects.
#[derive(Debug, Serialize)]
pub struct SearchParameters {
    pub query: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_range: Option<TimeRange>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub categories: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub engines: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unset_filters_are_omitted() {
        let parameters = SearchParameters {
            query: "podman rootless".to_string(),
            time_range: None,
            language: None,
            categories: None,
            engines: None,
        };
        assert_eq!(
            serde_json::to_value(&parameters).unwrap(),
            serde_json::json!({"query": "podman rootless"})
        );
    }

    #[test]
    fn set_filters_use_the_endpoint_names() {
        let parameters = SearchParameters {
            query: "axum".to_string(),
            time_range: Some(TimeRange::Week),
            language: Some("de".to_string()),
            categories: Some("it".to_string()),
            engines: Some("github,crates.io".to_string()),
        };
        assert_eq!(
            serde_json::to_value(&parameters).unwrap(),
            serde_json::json!({
                "query": "axum",
                "time_range": "week",
                "language": "de",
                "categories": "it",
                "engines": "github,crates.io"
            })
        );
    }
}
