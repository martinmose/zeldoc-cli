use serde::{Deserialize, Serialize};

use super::reasoning_effort_dto::ReasoningEffortDTO;

/// What a model accepts beyond plain text.
#[derive(Debug, Deserialize, Serialize)]
pub struct ModelCapabilitiesDTO {
    pub tool_calls: bool,
    pub reasoning: bool,
    /// Image input.
    pub vision: bool,
    pub pdf_input: bool,
    pub prompt_caching: bool,
    /// The `reasoning_effort` values that change something; empty when the
    /// model has none to pick from.
    pub reasoning_efforts: Vec<ReasoningEffortDTO>,
}
