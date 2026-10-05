//! A `.zeldoc-profile` file pins a folder, and every folder below it, to a
//! profile: in a customer's repository, the CLI then uses that customer's key
//! without switching. The file holds only the profile name, never a key, so
//! it can be committed; everyone saves their own key under that name.

use std::fs;
use std::path::{Path, PathBuf};

use super::credentials_error::CredentialsError;
use super::data_transfer_objects::profile_name_dto::ProfileNameDTO;
use crate::constants::files;

/// A pin file and the profile it names.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfilePin {
    pub name: ProfileNameDTO,
    pub path: PathBuf,
}

impl ProfilePin {
    /// The pin file in `directory` or the nearest folder above it.
    pub fn find_from(directory: &Path) -> Result<Option<Self>, CredentialsError> {
        for folder in directory.ancestors() {
            let path = folder.join(files::PROFILE_PIN);
            // Folders that cannot be read are passed over, as if they had no pin.
            if !path.is_file() {
                continue;
            }
            let bytes = fs::read(&path).map_err(|source| CredentialsError::Read {
                path: path.clone(),
                source,
            })?;
            let Some(contents) = decode(&bytes) else {
                return Err(CredentialsError::InvalidPin {
                    path,
                    reason: "it is not UTF-8 or UTF-16 text".to_string(),
                });
            };
            return Self::parse(path, &contents).map(Some);
        }
        Ok(None)
    }

    /// Pin `directory` to `name`, replacing a pin file already there.
    pub fn write(directory: &Path, name: &ProfileNameDTO) -> Result<Self, CredentialsError> {
        let path = directory.join(files::PROFILE_PIN);
        fs::write(&path, format!("{name}\n")).map_err(|source| CredentialsError::Write {
            path: path.clone(),
            source,
        })?;
        Ok(Self {
            name: name.clone(),
            path,
        })
    }

    /// The first line that is not empty or a `#` comment is the name.
    fn parse(path: PathBuf, contents: &str) -> Result<Self, CredentialsError> {
        let line = contents
            .lines()
            .map(str::trim)
            .find(|line| !line.is_empty() && !line.starts_with('#'))
            .unwrap_or_default();
        match line.parse() {
            Ok(name) => Ok(Self { name, path }),
            Err(reason) => Err(CredentialsError::InvalidPin { path, reason }),
        }
    }
}

/// The file as text. Besides UTF-8, with or without a byte order mark,
/// accepts the UTF-16 that Windows PowerShell 5.1 writes for
/// `echo acme > .zeldoc-profile`.
fn decode(bytes: &[u8]) -> Option<String> {
    if let Some(utf16) = bytes.strip_prefix(&[0xFF, 0xFE]) {
        let units: Vec<u16> = utf16
            .chunks_exact(2)
            .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
            .collect();
        return String::from_utf16(&units).ok();
    }
    let utf8 = bytes.strip_prefix(&[0xEF, 0xBB, 0xBF]).unwrap_or(bytes);
    String::from_utf8(utf8.to_vec()).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nearest_pin_above_the_directory_wins() {
        let root = tempfile::tempdir().unwrap();
        let nested = root.path().join("app").join("src");
        fs::create_dir_all(&nested).unwrap();
        fs::write(root.path().join(files::PROFILE_PIN), "globex\n").unwrap();
        fs::write(
            root.path().join("app").join(files::PROFILE_PIN),
            "# Zeldoc profile\n\nacme\n",
        )
        .unwrap();

        let pin = ProfilePin::find_from(&nested).unwrap().unwrap();
        assert_eq!(pin.name.as_str(), "acme");
        assert_eq!(pin.path, root.path().join("app").join(files::PROFILE_PIN));
    }

    #[test]
    fn no_pin_anywhere_is_none() {
        let root = tempfile::tempdir().unwrap();
        // Pins above the temporary directory would make this test depend on
        // the machine, so only folders inside it are checked.
        let found = ProfilePin::find_from(root.path())
            .unwrap()
            .filter(|pin| pin.path.starts_with(root.path()));
        assert_eq!(found, None);
    }

    #[test]
    fn written_pin_is_found_again() {
        let root = tempfile::tempdir().unwrap();
        let written = ProfilePin::write(root.path(), &"acme".parse().unwrap()).unwrap();
        assert_eq!(ProfilePin::find_from(root.path()).unwrap(), Some(written));
    }

    #[test]
    fn pins_written_by_windows_tools_are_read() {
        let utf16_with_crlf: Vec<u8> = [0xFF, 0xFE]
            .into_iter()
            .chain("acme\r\n".encode_utf16().flat_map(u16::to_le_bytes))
            .collect();
        let utf8_with_bom = b"\xEF\xBB\xBFacme\r\n".to_vec();
        for contents in [utf16_with_crlf, utf8_with_bom] {
            let root = tempfile::tempdir().unwrap();
            fs::write(root.path().join(files::PROFILE_PIN), contents).unwrap();
            let pin = ProfilePin::find_from(root.path()).unwrap().unwrap();
            assert_eq!(pin.name.as_str(), "acme");
        }
    }

    #[test]
    fn pin_with_an_invalid_name_is_an_error() {
        let root = tempfile::tempdir().unwrap();
        fs::write(root.path().join(files::PROFILE_PIN), "acme corp\n").unwrap();
        assert!(matches!(
            ProfilePin::find_from(root.path()),
            Err(CredentialsError::InvalidPin { .. })
        ));
    }
}
