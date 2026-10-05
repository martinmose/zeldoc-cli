use serde::{Deserialize, Serialize};

use super::usage_totals_dto::UsageTotalsDTO;
use crate::models::data_transfer_objects::model_id_dto::ModelIdDTO;

/// One model's share of the key's usage, under the name the requests used.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ModelUsageDTO {
    pub id: ModelIdDTO,
    #[serde(flatten)]
    pub usage: UsageTotalsDTO,
}
