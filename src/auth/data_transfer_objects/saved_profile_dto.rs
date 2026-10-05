use serde::Serialize;

use crate::credentials::data_transfer_objects::profile_name_dto::ProfileNameDTO;

/// A saved profile as `zeldoc auth list --json` prints it. The key is masked.
#[derive(Debug, Serialize)]
pub struct SavedProfileDTO {
    pub profile: ProfileNameDTO,
    pub key: String,
    /// Set with `zeldoc auth use`.
    pub default: bool,
    /// Named by the `.zeldoc-profile` file for the current folder.
    pub pinned_here: bool,
}
