use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// The key's monthly spend limit and what counts against it this calendar
/// month (UTC), whatever period the report is for. Requests are refused once
/// `spent_usd` reaches `limit_usd`.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MonthlyLimitDTO {
    pub limit_usd: Decimal,
    pub spent_usd: Decimal,
}
