use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::api_key_secret_dto::ApiKeySecretDTO;
use super::profile_dto::ProfileDTO;
use super::profile_name_dto::ProfileNameDTO;

/// The file `zeldoc auth login` writes: the saved API keys by profile name,
/// and the profile used where nothing else picks one.
///
/// ```json
/// {"active_profile": "acme", "profiles": {"acme": {"api_key": "sk-..."}}}
/// ```
///
/// Versions before profiles wrote `{"api_key": "sk-..."}`; such a file is read
/// as one profile named `default`, and written in the new form on the next
/// change.
#[derive(Default, Serialize, Deserialize)]
pub struct CredentialsFileDTO {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_profile: Option<ProfileNameDTO>,
    #[serde(default)]
    pub profiles: BTreeMap<ProfileNameDTO, ProfileDTO>,
    /// The single key of the old format; only ever read.
    #[serde(default, skip_serializing)]
    api_key: Option<ApiKeySecretDTO>,
}

impl CredentialsFileDTO {
    /// Move a key in the old format into a `default` profile, drop profiles
    /// with an empty key, and make the only profile the default when there is
    /// no default.
    pub fn upgraded(mut self) -> Self {
        if let Some(api_key) = self.api_key.take().filter(|api_key| !api_key.is_empty())
            && self.profiles.is_empty()
        {
            let name = ProfileNameDTO::default_name();
            self.profiles.insert(name.clone(), ProfileDTO { api_key });
            self.active_profile.get_or_insert(name);
        }
        self.profiles
            .retain(|_, profile| !profile.api_key.is_empty());
        if self
            .active_profile
            .as_ref()
            .is_some_and(|active| !self.profiles.contains_key(active))
        {
            self.active_profile = None;
        }
        if self.active_profile.is_none() && self.profiles.len() == 1 {
            self.active_profile = self.profiles.keys().next().cloned();
        }
        self
    }

    pub fn api_key(&self, name: &ProfileNameDTO) -> Option<&ApiKeySecretDTO> {
        self.profiles.get(name).map(|profile| &profile.api_key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn name(name: &str) -> ProfileNameDTO {
        name.parse().unwrap()
    }

    #[test]
    fn old_single_key_file_becomes_the_active_default_profile() {
        let file: CredentialsFileDTO = serde_json::from_str(r#"{"api_key":"sk-old"}"#).unwrap();
        let file = file.upgraded();
        assert_eq!(file.active_profile, Some(name("default")));
        assert_eq!(file.api_key(&name("default")).unwrap().expose(), "sk-old");
        assert_eq!(
            serde_json::to_string(&file).unwrap(),
            r#"{"active_profile":"default","profiles":{"default":{"api_key":"sk-old"}}}"#
        );
    }

    #[test]
    fn active_profile_that_is_not_saved_is_dropped() {
        let file: CredentialsFileDTO = serde_json::from_str(
            r#"{"active_profile":"gone","profiles":{"acme":{"api_key":"sk-a"},"globex":{"api_key":"sk-g"},"empty":{"api_key":""}}}"#,
        )
        .unwrap();
        let file = file.upgraded();
        assert_eq!(file.active_profile, None);
        assert_eq!(
            file.profiles.keys().collect::<Vec<_>>(),
            [&name("acme"), &name("globex")]
        );
    }

    #[test]
    fn only_profile_becomes_the_default() {
        let file: CredentialsFileDTO =
            serde_json::from_str(r#"{"profiles":{"acme":{"api_key":"sk-a"}}}"#).unwrap();
        assert_eq!(file.upgraded().active_profile, Some(name("acme")));
    }
}
