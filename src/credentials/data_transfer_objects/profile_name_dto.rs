use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

/// Longest profile name accepted.
const MAX_LENGTH: usize = 64;

/// The name a saved API key is stored under, e.g. `acme`: 1 to 64 ASCII
/// letters, digits, `-`, `_` or `.`, starting with a letter or digit.
/// Serialized as the plain string.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct ProfileNameDTO(String);

impl ProfileNameDTO {
    /// The name `zeldoc auth login` uses when none is given, and the name a
    /// key saved by a CLI without profiles is read as.
    pub fn default_name() -> Self {
        Self("default".to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl FromStr for ProfileNameDTO {
    type Err = String;

    fn from_str(name: &str) -> Result<Self, Self::Err> {
        let starts_well = name
            .chars()
            .next()
            .is_some_and(|first| first.is_ascii_alphanumeric());
        let only_allowed = name
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || "-_.".contains(character));
        if !starts_well || !only_allowed || name.len() > MAX_LENGTH {
            return Err(format!(
                "`{name}` is not a valid profile name: use 1 to {MAX_LENGTH} letters, digits, \
                 `-`, `_` or `.`, starting with a letter or digit"
            ));
        }
        Ok(Self(name.to_string()))
    }
}

impl TryFrom<String> for ProfileNameDTO {
    type Error = String;

    fn try_from(name: String) -> Result<Self, Self::Error> {
        name.parse()
    }
}

impl From<ProfileNameDTO> for String {
    fn from(name: ProfileNameDTO) -> Self {
        name.0
    }
}

impl fmt::Display for ProfileNameDTO {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_simple_names() {
        for name in ["acme", "Acme-2", "globex_eu", "client.prod", "7eleven"] {
            assert_eq!(name.parse::<ProfileNameDTO>().unwrap().as_str(), name);
        }
    }

    #[test]
    fn rejects_names_that_could_confuse_a_shell_or_a_path() {
        let too_long = "a".repeat(MAX_LENGTH + 1);
        for name in [
            "",
            "-acme",
            ".hidden",
            "acme corp",
            "../acme",
            "acme/eu",
            &too_long,
        ] {
            assert!(name.parse::<ProfileNameDTO>().is_err(), "{name:?}");
        }
    }

    #[test]
    fn invalid_names_fail_deserialization() {
        assert!(serde_json::from_str::<ProfileNameDTO>("\"acme corp\"").is_err());
    }
}
