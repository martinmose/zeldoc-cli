use std::io;
use std::path::PathBuf;

use thiserror::Error;

use super::data_transfer_objects::profile_name_dto::ProfileNameDTO;
use crate::constants::{api, environment, files};

#[derive(Debug, Error)]
pub enum CredentialsError {
    #[error(
        "no API key found. Run `zeldoc auth login`, or set {}. Need a key? See {}",
        environment::API_KEY,
        api::API_KEY_DOCS_URL
    )]
    NoApiKey,
    #[error(
        "no API key is saved as profile `{name}`. Run `zeldoc auth login --profile {name}`, \
         or `zeldoc auth list` to see the saved profiles"
    )]
    ProfileNotSaved { name: ProfileNameDTO },
    #[error(
        "{} picks profile `{name}`, but no API key is saved under that name. \
         Run `zeldoc auth login` in this folder to save one",
        .pin.display()
    )]
    PinnedProfileNotSaved { name: ProfileNameDTO, pin: PathBuf },
    #[error(
        "API keys are saved, but none is the default. Run `zeldoc auth use <profile>`, \
         or add a {} file to this folder",
        files::PROFILE_PIN
    )]
    NoActiveProfile,
    #[error("{} does not name a valid profile: {reason}", .path.display())]
    InvalidPin { path: PathBuf, reason: String },
    #[error(
        "could not find a config directory; set {}",
        environment::CONFIG_DIRECTORY
    )]
    NoConfigDirectory,
    #[error("could not read {}", .path.display())]
    Read {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("{} is not a valid credentials file", .path.display())]
    Malformed {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
    #[error("could not encode the credentials file")]
    Encode(#[source] serde_json::Error),
    #[error("could not write {}", .path.display())]
    Write {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("could not delete {}", .path.display())]
    Delete {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
}
