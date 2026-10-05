use std::fmt;

use clap::ValueEnum;
use serde::Serialize;

/// The period to report usage for. Zeldoc.ai turns it into dates in UTC, so
/// "today" starts at midnight UTC, not at local midnight.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, ValueEnum, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum UsagePeriod {
    /// Today so far
    Today,
    /// The last 7 days, today included
    Week,
    /// This calendar month so far
    #[default]
    Month,
    /// The previous calendar month
    LastMonth,
}

impl UsagePeriod {
    /// The value of the `period` query parameter.
    pub fn as_query_value(self) -> &'static str {
        match self {
            Self::Today => "today",
            Self::Week => "week",
            Self::Month => "month",
            Self::LastMonth => "last_month",
        }
    }

    /// How the period is named in text output.
    pub fn label(self) -> &'static str {
        match self {
            Self::Today => "today",
            Self::Week => "last 7 days",
            Self::Month => "this month",
            Self::LastMonth => "last month",
        }
    }
}

impl fmt::Display for UsagePeriod {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_query_value())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn query_values_match_the_serialized_names() {
        for period in UsagePeriod::value_variants() {
            assert_eq!(
                serde_json::to_string(period).unwrap(),
                format!("\"{}\"", period.as_query_value())
            );
        }
    }

    #[test]
    fn command_line_names_are_kebab_case() {
        let period = UsagePeriod::from_str("last-month", false).unwrap();
        assert_eq!(period, UsagePeriod::LastMonth);
    }
}
