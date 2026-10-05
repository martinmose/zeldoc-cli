use serde::{Deserialize, Serialize};

use super::zdev_plan_dto::ZDevPlanDTO;

/// The ZDev allowance of the person the key belongs to, this calendar month
/// (UTC), counted over all of their keys.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ZDevAllowanceDTO {
    pub plan: ZDevPlanDTO,
    /// Counted tokens the plan includes per month; `None` for Max (fair use).
    pub included_tokens: Option<u64>,
    /// Counted tokens used this month: new input, output, and cache reads at
    /// a tenth. Updated every few minutes.
    pub used_tokens: u64,
    /// The allowance is used up and ZDev is paused until the 1st.
    pub paused: bool,
    /// ZDev continues past the allowance, billed as overage.
    pub overage: bool,
}
