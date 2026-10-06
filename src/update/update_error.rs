use std::path::PathBuf;

use axoupdater::AxoupdateError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum UpdateError {
    /// No install receipt: the CLI was built with `cargo install`, installed
    /// by a package manager or unpacked from an archive by hand.
    #[error(
        "this copy of zeldoc was not installed with the zeldoc installer, so it cannot update \
         itself; update it the way you installed it, or run the install command from the README"
    )]
    NotInstalledByInstaller,
    /// The installer installed zeldoc elsewhere, and this is another copy.
    #[error(
        "this copy of zeldoc ({}) is not the one the installer put in {}; update it the way you \
         installed it",
        .executable.display(),
        .installer_directory.display()
    )]
    OtherCopyInstalled {
        executable: PathBuf,
        installer_directory: PathBuf,
    },
    #[error("could not set up the update")]
    Runtime(#[source] std::io::Error),
    #[error("could not create the HTTP client")]
    HttpClient(#[source] reqwest::Error),
    #[error("could not update zeldoc")]
    Updater(#[source] Box<AxoupdateError>),
}

impl From<AxoupdateError> for UpdateError {
    fn from(error: AxoupdateError) -> Self {
        match error {
            AxoupdateError::NoReceipt { .. } => Self::NotInstalledByInstaller,
            error => Self::Updater(Box::new(error)),
        }
    }
}
