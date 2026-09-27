use clap::ValueEnum;
use serde::Serialize;

/// The period results should come from; engine-dependent, not guaranteed.
#[derive(Debug, Clone, Copy, ValueEnum, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum TimeRange {
    Day,
    Week,
    Month,
    Year,
}
