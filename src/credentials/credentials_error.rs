use std::io;
use std::path::PathBuf;

use thiserror::Error;

use crate::constants::{api, environment};

#[derive(Debug, Error)]
pub enum CredentialsError {
    #[error(
        "no API key found. Run `zeldoc auth login`, or set {}. Need a key? See {}",
        environment::API_KEY,
        api::API_KEY_DOCS_URL
    )]
    NoApiKey,
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
