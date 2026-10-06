use axoupdater::Version;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// The last check for a newer release, cached so GitHub is asked at most once
/// a day.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct UpdateCheckDTO {
    pub checked_at: DateTime<Utc>,
    /// The latest release found, if a check has ever succeeded.
    #[serde(
        default,
        serialize_with = "serialize_version",
        deserialize_with = "deserialize_version"
    )]
    pub latest_version: Option<Version>,
}

// axoupdater's `Version` has no serde support; it is stored as its text.
fn serialize_version<S: Serializer>(
    version: &Option<Version>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    version
        .as_ref()
        .map(ToString::to_string)
        .serialize(serializer)
}

fn deserialize_version<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<Version>, D::Error> {
    Option::<String>::deserialize(deserializer)?
        .map(|text| text.parse().map_err(serde::de::Error::custom))
        .transpose()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_through_json() {
        let check = UpdateCheckDTO {
            checked_at: "2026-10-06T10:00:00Z".parse().unwrap(),
            latest_version: Some("0.3.0".parse().unwrap()),
        };
        let json = serde_json::to_string(&check).unwrap();
        assert_eq!(
            json,
            r#"{"checked_at":"2026-10-06T10:00:00Z","latest_version":"0.3.0"}"#
        );
        assert_eq!(
            serde_json::from_str::<UpdateCheckDTO>(&json).unwrap(),
            check
        );
    }

    #[test]
    fn a_check_that_never_succeeded_has_no_version() {
        let check: UpdateCheckDTO =
            serde_json::from_str(r#"{"checked_at":"2026-10-06T10:00:00Z"}"#).unwrap();
        assert_eq!(check.latest_version, None);
    }
}
