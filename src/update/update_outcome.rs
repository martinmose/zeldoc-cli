use axoupdater::Version;

/// What `zeldoc update` found or did.
#[derive(Debug, PartialEq)]
pub enum UpdateOutcome {
    /// The running version is the latest release.
    UpToDate { current: Version },
    /// A newer release exists; only reported by `--check`.
    Available { current: Version, latest: Version },
    /// The latest release was installed in place of the running version.
    Updated {
        previous: Version,
        installed: Version,
    },
}

impl UpdateOutcome {
    /// The latest release, as far as this outcome knows.
    pub fn latest_version(&self) -> &Version {
        match self {
            Self::UpToDate { current } => current,
            Self::Available { latest, .. } => latest,
            Self::Updated { installed, .. } => installed,
        }
    }
}
