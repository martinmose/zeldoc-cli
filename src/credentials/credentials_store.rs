//! Where the CLI finds its API key: ZELDOC_API_KEY first, then the file
//! written by `zeldoc auth login`.
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

    pub fn load(&self) -> Result<Option<ApiKeySecretDTO>, CredentialsError> {
        let contents = match fs::read_to_string(&self.path) {
            Ok(contents) => contents,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
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
        Ok(Some(file.api_key).filter(|api_key| !api_key.is_empty()))
    }

    /// Write the key so only the current user can read it. The file is
    /// written next to its final name and renamed into place, so an
    /// interrupted save never leaves a half-written or wider-permission file.
    pub fn save(&self, api_key: &ApiKeySecretDTO) -> Result<(), CredentialsError> {
        let directory = self
            .path
            .parent()
            .filter(|directory| !directory.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        create_private_directory(directory).map_err(|source| CredentialsError::Write {
            path: directory.to_path_buf(),
            source,
        })?;

        let contents = serde_json::to_string_pretty(&CredentialsFileDTO {
            api_key: api_key.clone(),
        })
        .map_err(CredentialsError::Encode)?;
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

/// The key the CLI uses: ZELDOC_API_KEY when set, otherwise the saved key.
pub fn resolve_api_key() -> Result<Option<ResolvedApiKey>, CredentialsError> {
    if let Some(secret) = api_key_from_environment() {
        return Ok(Some(ResolvedApiKey {
            secret,
            source: ApiKeySource::Environment,
        }));
    }
    let store = CredentialsStore::at_default_location()?;
    Ok(store.load()?.map(|secret| ResolvedApiKey {
        secret,
        source: ApiKeySource::File(store.path),
    }))
}

/// As `resolve_api_key`, but having no key is an error that says how to get one.
pub fn require_api_key() -> Result<ResolvedApiKey, CredentialsError> {
    resolve_api_key()?.ok_or(CredentialsError::NoApiKey)
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

    fn secret(key: &str) -> ApiKeySecretDTO {
        ApiKeySecretDTO::new(key.to_string())
    }

    #[test]
    fn save_then_load_round_trips() {
        let directory = tempfile::tempdir().unwrap();
        let store = CredentialsStore::at(directory.path().join("zeldoc").join(FILE_NAME));

        assert_eq!(store.load().unwrap(), None);
        store.save(&secret("sk-first")).unwrap();
        assert_eq!(store.load().unwrap(), Some(secret("sk-first")));
        store.save(&secret("sk-second")).unwrap();
        assert_eq!(store.load().unwrap(), Some(secret("sk-second")));
    }

    #[cfg(unix)]
    #[test]
    fn saved_key_is_readable_only_by_the_owner() {
        use std::os::unix::fs::PermissionsExt;

        let directory = tempfile::tempdir().unwrap();
        let config_directory = directory.path().join("zeldoc");
        let store = CredentialsStore::at(config_directory.join(FILE_NAME));
        store.save(&secret("sk-secret")).unwrap();

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
    fn remove_reports_whether_a_key_was_saved() {
        let directory = tempfile::tempdir().unwrap();
        let store = CredentialsStore::at(directory.path().join(FILE_NAME));

        assert!(!store.remove().unwrap());
        store.save(&secret("sk-secret")).unwrap();
        assert!(store.remove().unwrap());
        assert_eq!(store.load().unwrap(), None);
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
