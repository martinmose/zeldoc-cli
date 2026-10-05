use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Requests, tokens and cost over a period. The cost is in USD, as the
/// dashboard's key usage shows it, sent as a decimal string; Zeldoc's own
/// models are covered by the subscription and cost 0.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UsageTotalsDTO {
    pub cost_usd: Decimal,
    pub requests: u64,
    pub input_tokens: u64,
    pub output_tokens: u64,
    /// Input tokens read from the prompt cache. Part of `input_tokens`, not
    /// added to it.
    pub cache_read_tokens: u64,
    /// Input tokens written to the prompt cache. Part of `input_tokens`, not
    /// added to it.
    pub cache_write_tokens: u64,
}
