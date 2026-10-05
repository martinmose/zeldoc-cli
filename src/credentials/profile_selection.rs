use super::profile_pin::ProfilePin;

/// What picked the profile whose key the CLI uses.
pub enum ProfileSelection {
    /// `--profile` or ZELDOC_PROFILE.
    Requested,
    /// A `.zeldoc-profile` file in the current folder or above it.
    Pin(ProfilePin),
    /// The profile set with `zeldoc auth use`.
    Active,
}
