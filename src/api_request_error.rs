use reqwest::StatusCode;
use thiserror::Error;

use crate::constants::api;

/// Longest raw response body quoted in an error when it has no JSON message.
const MAX_ERROR_BODY_CHARACTERS: usize = 300;

#[derive(Debug, Error)]
pub enum ApiRequestError {
    #[error("could not create the HTTP client")]
    ClientSetup(#[source] reqwest::Error),
    #[error("could not reach {}", api::BASE_URL)]
    Network(#[source] reqwest::Error),
    #[error("{}", describe_failure(.status, .message))]
    FailedWithApiResponse {
        status: StatusCode,
        message: Option<String>,
    },
    #[error("Zeldoc.ai sent a response the CLI could not read")]
    Deserialization(#[source] reqwest::Error),
}

impl ApiRequestError {
    /// Zeldoc.ai rejected the API key: missing, unknown, expired or blocked.
    pub fn is_auth_failure(&self) -> bool {
        matches!(
            self,
            Self::FailedWithApiResponse { status, .. } if *status == StatusCode::UNAUTHORIZED
        )
    }

    pub(crate) fn from_response_body(status: StatusCode, body: &str) -> Self {
        Self::FailedWithApiResponse {
            status,
            message: message_from_body(body),
        }
    }
}

fn describe_failure(status: &StatusCode, message: &Option<String>) -> String {
    let mut description = format!("Zeldoc.ai returned {status}");
    if let Some(message) = message {
        description.push_str(&format!(": {message}"));
    }
    if *status == StatusCode::UNAUTHORIZED {
        description.push_str(" (check the key with `zeldoc auth status`)");
    }
    description
}

/// The human-readable part of an error body: an `error.message`, an `error`
/// string or a `detail` string, whichever the body has, or the start of the
/// raw body.
fn message_from_body(body: &str) -> Option<String> {
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(body) {
        let message = ["/error/message", "/error", "/detail"]
            .iter()
            .find_map(|pointer| json.pointer(pointer)?.as_str());
        if let Some(message) = message {
            return Some(message.to_string());
        }
    }
    let body = body.trim();
    if body.is_empty() {
        return None;
    }
    let mut excerpt: String = body.chars().take(MAX_ERROR_BODY_CHARACTERS).collect();
    if excerpt.len() < body.len() {
        excerpt.push('…');
    }
    Some(excerpt)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn message_from_body_reads_nested_error_messages() {
        let body = r#"{"error":{"message":"Authentication failed. Please check your API key.","type":"auth_error","param":"None","code":"401"}}"#;
        assert_eq!(
            message_from_body(body).as_deref(),
            Some("Authentication failed. Please check your API key.")
        );
    }

    #[test]
    fn message_from_body_reads_error_strings() {
        assert_eq!(
            message_from_body(r#"{"error":"Invalid API key"}"#).as_deref(),
            Some("Invalid API key")
        );
    }

    #[test]
    fn message_from_body_reads_detail_errors() {
        assert_eq!(
            message_from_body(r#"{"detail":"Not found"}"#).as_deref(),
            Some("Not found")
        );
    }

    #[test]
    fn message_from_body_falls_back_to_the_raw_body() {
        assert_eq!(
            message_from_body("  Bad Gateway\n").as_deref(),
            Some("Bad Gateway")
        );
        assert_eq!(message_from_body(""), None);
    }

    #[test]
    fn message_from_body_truncates_long_bodies() {
        let body = "x".repeat(MAX_ERROR_BODY_CHARACTERS + 50);
        let message = message_from_body(&body).unwrap();
        assert_eq!(message.chars().count(), MAX_ERROR_BODY_CHARACTERS + 1);
        assert!(message.ends_with('…'));
    }

    #[test]
    fn auth_failures_point_at_auth_status() {
        let error = ApiRequestError::from_response_body(
            StatusCode::UNAUTHORIZED,
            r#"{"error":"Invalid API key"}"#,
        );
        assert!(error.is_auth_failure());
        assert_eq!(
            error.to_string(),
            "Zeldoc.ai returned 401 Unauthorized: Invalid API key \
             (check the key with `zeldoc auth status`)"
        );
    }
}
