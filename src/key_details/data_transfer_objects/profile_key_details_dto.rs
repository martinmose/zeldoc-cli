use serde::Serialize;

use super::key_details_dto::KeyDetailsDTO;
use crate::credentials::data_transfer_objects::profile_name_dto::ProfileNameDTO;

/// One saved profile's key as `zeldoc auth fields --all --json` prints it:
/// the key's details, or why there are none.
#[derive(Debug, Serialize)]
pub struct ProfileKeyDetailsDTO {
    pub profile: ProfileNameDTO,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<KeyDetailsDTO>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}
