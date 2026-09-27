use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// What the key's organization pays in USD per 1 million tokens. The API sends
/// decimal strings, kept as `Decimal` so no precision is lost.
#[derive(Debug, Deserialize, Serialize)]
pub struct ModelPricingDTO {
    pub input: Option<Decimal>,
    pub output: Option<Decimal>,
    pub cache_read: Option<Decimal>,
    pub cache_write: Option<Decimal>,
}
