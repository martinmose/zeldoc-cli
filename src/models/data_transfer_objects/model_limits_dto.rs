use serde::{Deserialize, Serialize};

/// Token limits of a model; null where the gateway does not know them.
#[derive(Debug, Deserialize, Serialize)]
pub struct ModelLimitsDTO {
    /// Input tokens (the context window).
    pub context: Option<u64>,
    /// Output tokens per response.
    pub output: Option<u64>,
}
