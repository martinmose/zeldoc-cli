use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

/// The days a report covers, both included, in UTC.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UsageDateRangeDTO {
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
}
