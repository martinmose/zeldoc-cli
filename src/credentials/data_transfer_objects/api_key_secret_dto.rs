use std::fmt;

use serde::{Deserialize, Serialize};

/// A Zeldoc.ai API key (`sk-...`).
///
/// `Debug` is redacted so the key cannot end up in a log line or an error;
/// `masked` shows which key it is without revealing it, and `expose` is for
/// the places that must send or print it.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApiKeySecretDTO(String);

impl ApiKeySecretDTO {
    pub fn new(key: String) -> Self {
        Self(key)
    }

    pub fn expose(&self) -> &str {
        &self.0
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// The key with all but its prefix and last four characters hidden, e.g.
    /// `sk-…a1b2`.
    pub fn masked(&self) -> String {
        let prefix = if self.0.starts_with("sk-") { "sk-" } else { "" };
        let characters: Vec<char> = self.0.chars().collect();
        if characters.len() < 12 {
            return format!("{prefix}…");
        }
        let last_four: String = characters[characters.len() - 4..].iter().collect();
        format!("{prefix}…{last_four}")
    }
}

impl fmt::Debug for ApiKeySecretDTO {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("ApiKeySecretDTO(<redacted>)")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn masked_shows_only_the_prefix_and_last_four() {
        let masked = |key: &str| ApiKeySecretDTO::new(key.to_string()).masked();
        assert_eq!(masked("sk-abcdefghijklmnop"), "sk-…mnop");
        assert_eq!(masked("abcdefghijklmnop"), "…mnop");
        assert_eq!(masked("sk-short"), "sk-…");
    }

    #[test]
    fn debug_never_shows_the_key() {
        let api_key = ApiKeySecretDTO::new("sk-abcdefghijklmnop".to_string());
        assert_eq!(format!("{api_key:?}"), "ApiKeySecretDTO(<redacted>)");
    }
}
