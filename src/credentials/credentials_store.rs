//! Where the CLI finds its API key, in this order:
//!
//! 1. the saved profile named by `--profile` or ZELDOC_PROFILE;
//! 2. the saved profile named by a `.zeldoc-profile` file in the current folder
//!    or a folder above it;
//! 3. ZELDOC_API_KEY;
//! 4. the saved profile set with `zeldoc auth use`.
//!
//! A pin file wins over ZELDOC_API_KEY because the README suggests exporting
//! that variable in every shell; if it won, pins would never apply. On a
//! machine with no saved keys (CI, containers) pins are ignored.
//!
//! The file is plain JSON readable only by the current user (mode 0600 in a
//! 0700 directory on Unix; Windows keeps the per-user ACL of %APPDATA%). That
//! is the same protection ~/.codex/auth.json and gh's fallback storage get.

use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use super::api_key_source::ApiKeySource;
use super::credentials_error::CredentialsError;
use super::data_transfer_objects::api_key_secret_dto::ApiKeySecretDTO;
use super::data_transfer_objects::credentials_file_dto::CredentialsFileDTO;
use super::data_transfer_objects::profile_name_dto::ProfileNameDTO;
use super::profile_pin::ProfilePin;
use super::profile_selection::ProfileSelection;
use super::resolved_api_key::ResolvedApiKey;
use crate::constants::environment;

const FILE_NAME: &str = "credentials.json";

/// The credentials file at one path.
pub struct CredentialsStore {
    path: PathBuf,
}

impl CredentialsStore {
    /// $ZELDOC_CONFIG_DIR/credentials.json, or credentials.json in a `zeldoc`
    /// folder in the platform config directory.
    pub fn at_default_location() -> Result<Self, CredentialsError> {
        let directory =
            match env::var_os(environment::CONFIG_DIRECTORY).filter(|dir| !dir.is_empty()) {
                Some(directory) => PathBuf::from(directory),
                None => dirs::config_dir()
                    .ok_or(CredentialsError::NoConfigDirectory)?
                    .join("zeldoc"),
            };
        Ok(Self::at(directory.join(FILE_NAME)))
    }

    pub fn at(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// The saved profiles; none when there is no file yet.
    pub fn load(&self) -> Result<CredentialsFileDTO, CredentialsError> {
        let contents = match fs::read_to_string(&self.path) {
            Ok(contents) => contents,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                return Ok(CredentialsFileDTO::default());
            }
            Err(source) => {
                return Err(CredentialsError::Read {
                    path: self.path.clone(),
                    source,
                });
            }
        };
        let file: CredentialsFileDTO =
            serde_json::from_str(&contents).map_err(|source| CredentialsError::Malformed {
                path: self.path.clone(),
                source,
            })?;
        Ok(file.upgraded())
    }

    /// Write the profiles so only the current user can read them; with no
    /// profiles left, delete the file. The file is written next to its final
    /// name and renamed into place, so an interrupted save never leaves a
    /// half-written or wider-permission file.
    pub fn save(&self, file: &CredentialsFileDTO) -> Result<(), CredentialsError> {
        if file.profiles.is_empty() {
            return self.remove().map(|_| ());
        }
        let directory = self
            .path
            .parent()
            .filter(|directory| !directory.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        create_private_directory(directory).map_err(|source| CredentialsError::Write {
            path: directory.to_path_buf(),
            source,
        })?;

        let contents = serde_json::to_string_pretty(file).map_err(CredentialsError::Encode)?;
        let temporary = directory.join(format!(".{FILE_NAME}.tmp"));
        write_private_file(&temporary, contents.as_bytes()).map_err(|source| {
            CredentialsError::Write {
                path: temporary.clone(),
                source,
            }
        })?;
        fs::rename(&temporary, &self.path).map_err(|source| CredentialsError::Write {
            path: self.path.clone(),
            source,
        })
    }

    /// Delete the credentials file. Returns whether there was one.
    pub fn remove(&self) -> Result<bool, CredentialsError> {
        remove_if_present(&self.path).map_err(|source| CredentialsError::Delete {
            path: self.path.clone(),
            source,
        })
    }
}

/// The key the CLI uses, picked in the order described at the top of this
/// file. `requested` is the profile from `--profile` or ZELDOC_PROFILE.
pub fn resolve_api_key(
    requested: Option<&ProfileNameDTO>,
) -> Result<Option<ResolvedApiKey>, CredentialsError> {
    let file = CredentialsStore::at_default_location()?.load()?;
    let pin = find_pin()?;
    select_api_key(&file, requested, pin, api_key_from_environment())
}

/// As `resolve_api_key`, but having no key is an error that says how to get one.
pub fn require_api_key(
    requested: Option<&ProfileNameDTO>,
) -> Result<ResolvedApiKey, CredentialsError> {
    resolve_api_key(requested)?.ok_or(CredentialsError::NoApiKey)
}

/// The pin file for the current folder, if any.
pub fn find_pin() -> Result<Option<ProfilePin>, CredentialsError> {
    match env::current_dir() {
        Ok(directory) => ProfilePin::find_from(&directory),
        Err(_) => Ok(None),
    }
}

fn select_api_key(
    file: &CredentialsFileDTO,
    requested: Option<&ProfileNameDTO>,
    pin: Option<ProfilePin>,
    environment_key: Option<ApiKeySecretDTO>,
) -> Result<Option<ResolvedApiKey>, CredentialsError> {
    if let Some(name) = requested {
        let secret = file
            .api_key(name)
            .ok_or_else(|| CredentialsError::ProfileNotSaved { name: name.clone() })?;
        return Ok(Some(profile_key(name, secret, ProfileSelection::Requested)));
    }
    if let Some(pin) = pin
        && !file.profiles.is_empty()
    {
        let Some(secret) = file.api_key(&pin.name) else {
            return Err(CredentialsError::PinnedProfileNotSaved {
                name: pin.name,
                pin: pin.path,
            });
        };
        let name = pin.name.clone();
        return Ok(Some(profile_key(&name, secret, ProfileSelection::Pin(pin))));
    }
    if let Some(secret) = environment_key {
        return Ok(Some(ResolvedApiKey {
            secret,
            source: ApiKeySource::Environment,
        }));
    }
    match &file.active_profile {
        Some(name) => Ok(file
            .api_key(name)
            .map(|secret| profile_key(name, secret, ProfileSelection::Active))),
        None if file.profiles.is_empty() => Ok(None),
        None => Err(CredentialsError::NoActiveProfile),
    }
}

fn profile_key(
    name: &ProfileNameDTO,
    secret: &ApiKeySecretDTO,
    selection: ProfileSelection,
) -> ResolvedApiKey {
    ResolvedApiKey {
        secret: secret.clone(),
        source: ApiKeySource::Profile {
            name: name.clone(),
            selection,
        },
    }
}

pub fn api_key_from_environment() -> Option<ApiKeySecretDTO> {
    env::var(environment::API_KEY)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .map(ApiKeySecretDTO::new)
}

fn write_private_file(path: &Path, contents: &[u8]) -> io::Result<()> {
    remove_if_present(path)?;
    let mut file = private_file_options().open(path)?;
    file.write_all(contents)?;
    file.write_all(b"\n")?;
    file.sync_all()
}

fn remove_if_present(path: &Path) -> io::Result<bool> {
    match fs::remove_file(path) {
        Ok(()) => Ok(true),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error),
    }
}

#[cfg(unix)]
fn create_private_directory(directory: &Path) -> io::Result<()> {
    use std::os::unix::fs::DirBuilderExt;
    fs::DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(directory)
}

#[cfg(not(unix))]
fn create_private_directory(directory: &Path) -> io::Result<()> {
    fs::create_dir_all(directory)
}

fn private_file_options() -> fs::OpenOptions {
    let mut options = fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    options
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::credentials::data_transfer_objects::profile_dto::ProfileDTO;

    fn secret(key: &str) -> ApiKeySecretDTO {
        ApiKeySecretDTO::new(key.to_string())
    }

    fn name(name: &str) -> ProfileNameDTO {
        name.parse().unwrap()
    }

    /// Profiles `acme` (sk-acme) and `globex` (sk-globex), `globex` active.
    fn two_profiles() -> CredentialsFileDTO {
        let mut file = CredentialsFileDTO::default();
        for profile in ["acme", "globex"] {
            file.profiles.insert(
                name(profile),
                ProfileDTO {
                    api_key: secret(&format!("sk-{profile}")),
                },
            );
        }
        file.active_profile = Some(name("globex"));
        file
    }

    fn pin(profile: &str) -> Option<ProfilePin> {
        Some(ProfilePin {
            name: name(profile),
            path: PathBuf::from("/work/acme-app/.zeldoc-profile"),
        })
    }

    fn selected(
        file: &CredentialsFileDTO,
        requested: Option<&str>,
        pin: Option<ProfilePin>,
        environment_key: Option<&str>,
    ) -> Result<Option<String>, CredentialsError> {
        let requested = requested.map(name);
        Ok(
            select_api_key(file, requested.as_ref(), pin, environment_key.map(secret))?
                .map(|resolved| resolved.secret.expose().to_string()),
        )
    }

    #[test]
    fn requested_profile_wins_over_everything() {
        let key = selected(&two_profiles(), Some("acme"), pin("globex"), Some("sk-env"));
        assert_eq!(key.unwrap().as_deref(), Some("sk-acme"));
    }

    #[test]
    fn pin_wins_over_the_environment_and_the_active_profile() {
        let key = selected(&two_profiles(), None, pin("acme"), Some("sk-env"));
        assert_eq!(key.unwrap().as_deref(), Some("sk-acme"));
    }

    #[test]
    fn environment_wins_over_the_active_profile() {
        let key = selected(&two_profiles(), None, None, Some("sk-env"));
        assert_eq!(key.unwrap().as_deref(), Some("sk-env"));
    }

    #[test]
    fn active_profile_is_the_fallback() {
        let key = selected(&two_profiles(), None, None, None);
        assert_eq!(key.unwrap().as_deref(), Some("sk-globex"));
    }

    #[test]
    fn unsaved_requested_or_pinned_profile_is_an_error_not_another_key() {
        assert!(matches!(
            selected(&two_profiles(), Some("initech"), None, Some("sk-env")),
            Err(CredentialsError::ProfileNotSaved { .. })
        ));
        assert!(matches!(
            selected(&two_profiles(), None, pin("initech"), Some("sk-env")),
            Err(CredentialsError::PinnedProfileNotSaved { .. })
        ));
    }

    #[test]
    fn pins_are_ignored_where_no_keys_are_saved() {
        let empty = CredentialsFileDTO::default();
        let key = selected(&empty, None, pin("acme"), Some("sk-env"));
        assert_eq!(key.unwrap().as_deref(), Some("sk-env"));
        assert_eq!(selected(&empty, None, pin("acme"), None).unwrap(), None);
    }

    #[test]
    fn saved_keys_without_a_default_ask_for_one() {
        let mut file = two_profiles();
        file.active_profile = None;
        assert!(matches!(
            selected(&file, None, None, None),
            Err(CredentialsError::NoActiveProfile)
        ));
    }

    #[test]
    fn save_then_load_round_trips() {
        let directory = tempfile::tempdir().unwrap();
        let store = CredentialsStore::at(directory.path().join("zeldoc").join(FILE_NAME));

        assert!(store.load().unwrap().profiles.is_empty());
        store.save(&two_profiles()).unwrap();
        let loaded = store.load().unwrap();
        assert_eq!(loaded.active_profile, Some(name("globex")));
        assert_eq!(loaded.api_key(&name("acme")), Some(&secret("sk-acme")));
    }

    #[test]
    fn saving_no_profiles_deletes_the_file() {
        let directory = tempfile::tempdir().unwrap();
        let store = CredentialsStore::at(directory.path().join(FILE_NAME));
        store.save(&two_profiles()).unwrap();
        store.save(&CredentialsFileDTO::default()).unwrap();
        assert!(!store.path().exists());
    }

    #[test]
    fn file_written_before_profiles_still_loads() {
        let directory = tempfile::tempdir().unwrap();
        let store = CredentialsStore::at(directory.path().join(FILE_NAME));
        fs::write(store.path(), r#"{"api_key": "sk-old"}"#).unwrap();
        let key = selected(&store.load().unwrap(), None, None, None);
        assert_eq!(key.unwrap().as_deref(), Some("sk-old"));
    }

    #[cfg(unix)]
    #[test]
    fn saved_keys_are_readable_only_by_the_owner() {
        use std::os::unix::fs::PermissionsExt;

        let directory = tempfile::tempdir().unwrap();
        let config_directory = directory.path().join("zeldoc");
        let store = CredentialsStore::at(config_directory.join(FILE_NAME));
        store.save(&two_profiles()).unwrap();

        let file_mode = fs::metadata(store.path()).unwrap().permissions().mode() & 0o777;
        let directory_mode = fs::metadata(&config_directory)
            .unwrap()
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(file_mode, 0o600);
        assert_eq!(directory_mode, 0o700);
    }

    #[test]
    fn load_rejects_a_malformed_file() {
        let directory = tempfile::tempdir().unwrap();
        let store = CredentialsStore::at(directory.path().join(FILE_NAME));
        fs::write(store.path(), "not json").unwrap();

        assert!(matches!(
            store.load(),
            Err(CredentialsError::Malformed { .. })
        ));
    }
}
