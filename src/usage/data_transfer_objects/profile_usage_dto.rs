use serde::Serialize;

use super::usage_report_dto::UsageReportDTO;
use crate::credentials::data_transfer_objects::profile_name_dto::ProfileNameDTO;

/// One saved profile's usage as `zeldoc usage --all --json` prints it: the
/// report, or why there is none.
#[derive(Debug, Serialize)]
pub struct ProfileUsageDTO {
    pub profile: ProfileNameDTO,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub report: Option<UsageReportDTO>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}
