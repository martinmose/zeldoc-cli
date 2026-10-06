use std::env;
use std::time::Duration;

use axoupdater::{AxoUpdater, Version};
use tokio::runtime::{Builder, Runtime};

use super::update_error::UpdateError;
use super::update_outcome::UpdateOutcome;

/// The name the installer records its install receipt under.
const APP_NAME: &str = "zeldoc";

pub trait UpdateService {
    /// The running version.
    fn current_version(&self) -> &Version;
    /// Whether this executable is the copy the installer installed, the only
    /// one `update` can replace. Reads local files only.
    fn can_update(&self) -> bool;
    /// Whether a newer release than the running version exists.
    fn check(&self) -> Result<UpdateOutcome, UpdateError>;
    /// Installs the latest release in place of the running version, when it
    /// is newer.
    fn update(&self) -> Result<UpdateOutcome, UpdateError>;
}

/// Updates through the release's own shell or PowerShell installer, found
/// from the receipt the installer wrote when zeldoc was first installed.
pub struct UpdateServiceImpl {
    current: Version,
    /// Limits each request to GitHub; none when the user waits for an update.
    timeout: Option<Duration>,
}

impl UpdateServiceImpl {
    pub fn new() -> Self {
        Self {
            current: env!("CARGO_PKG_VERSION")
                .parse()
                .expect("the crate version is a valid version"),
            timeout: None,
        }
    }

    /// For checks in the background, which must not hold a command up.
    pub fn with_timeout(timeout: Duration) -> Self {
        Self {
            timeout: Some(timeout),
            ..Self::new()
        }
    }

    /// An updater for this executable, or an error when the installer did not
    /// install it.
    fn updater(&self) -> Result<AxoUpdater, UpdateError> {
        let mut updater = AxoUpdater::new_for(APP_NAME);
        updater.load_receipt()?;
        if !updater.check_receipt_is_for_this_executable()? {
            return Err(UpdateError::OtherCopyInstalled {
                executable: env::current_exe().unwrap_or_default(),
                installer_directory: updater.install_prefix_root()?.into_std_path_buf(),
            });
        }
        updater.set_current_version(self.current.clone())?;
        if let Some(timeout) = self.timeout {
            let client = reqwest::Client::builder()
                .timeout(timeout)
                .build()
                .map_err(UpdateError::HttpClient)?;
            updater.set_client(client);
        }
        // Our own summary goes to stdout; the installer's errors still show.
        updater.disable_installer_stdout();
        Ok(updater)
    }
}

impl Default for UpdateServiceImpl {
    fn default() -> Self {
        Self::new()
    }
}

impl UpdateService for UpdateServiceImpl {
    fn current_version(&self) -> &Version {
        &self.current
    }

    fn can_update(&self) -> bool {
        self.updater().is_ok()
    }

    fn check(&self) -> Result<UpdateOutcome, UpdateError> {
        let mut updater = self.updater()?;
        let latest = runtime()?.block_on(updater.query_new_version())?.cloned();
        Ok(match latest {
            Some(latest) if latest > self.current => UpdateOutcome::Available {
                current: self.current.clone(),
                latest,
            },
            _ => UpdateOutcome::UpToDate {
                current: self.current.clone(),
            },
        })
    }

    fn update(&self) -> Result<UpdateOutcome, UpdateError> {
        let mut updater = self.updater()?;
        Ok(match runtime()?.block_on(updater.run())? {
            Some(result) => UpdateOutcome::Updated {
                previous: self.current.clone(),
                installed: result.new_version,
            },
            None => UpdateOutcome::UpToDate {
                current: self.current.clone(),
            },
        })
    }
}

fn runtime() -> Result<Runtime, UpdateError> {
    Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(UpdateError::Runtime)
}
