use serde::{Deserialize, Serialize};

use super::model_capabilities_dto::ModelCapabilitiesDTO;
use super::model_id_dto::ModelIdDTO;
use super::model_limits_dto::ModelLimitsDTO;
use super::model_mode_dto::ModelModeDTO;
use super::model_pricing_dto::ModelPricingDTO;

/// A model from Zeldoc.ai's model catalog (`GET /v1/zeldoc/models`), aliases
/// such as `zdev` included.
#[derive(Debug, Deserialize, Serialize)]
pub struct ModelDTO {
    pub id: ModelIdDTO,
    pub mode: Option<ModelModeDTO>,
    pub limits: ModelLimitsDTO,
    pub pricing: ModelPricingDTO,
    pub capabilities: ModelCapabilitiesDTO,
}

#[cfg(test)]
mod tests {
    use crate::models::test_catalog;

    #[test]
    fn json_keeps_the_catalog_shape() {
        let json = serde_json::to_value(&test_catalog::models()[0]).unwrap();
        assert_eq!(
            json,
            serde_json::json!({
                "id": "zdev",
                "mode": "chat",
                "limits": {"context": 1000000, "output": 131072},
                "pricing": {"input": "0", "output": "0", "cache_read": "0", "cache_write": null},
                "capabilities": {
                    "tool_calls": true, "reasoning": true, "vision": true,
                    "pdf_input": false, "prompt_caching": false,
                    "reasoning_efforts": ["low", "high", "max"]
                }
            })
        );
    }
}
