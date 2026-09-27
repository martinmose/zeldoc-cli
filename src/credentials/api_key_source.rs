use std::path::PathBuf;

/// Where the API key the CLI uses came from.
pub enum ApiKeySource {
    /// The ZELDOC_API_KEY environment variable.
    Environment,
    /// The credentials file written by `zeldoc auth login`.
    File(PathBuf),
}
