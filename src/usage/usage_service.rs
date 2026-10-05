use super::data_transfer_objects::usage_report_dto::UsageReportDTO;
use super::request_parameters::usage_period::UsagePeriod;
use crate::api_request_error::ApiRequestError;
use crate::api_request_handler::ApiRequestHandler;
use crate::constants::api;

pub trait UsageService {
    /// What the API key itself used in `period`.
    fn report(&self, period: UsagePeriod) -> Result<UsageReportDTO, ApiRequestError>;
}

pub struct UsageServiceImpl {
    request_handler: ApiRequestHandler,
}

impl UsageServiceImpl {
    pub fn new(request_handler: ApiRequestHandler) -> Self {
        Self { request_handler }
    }
}

impl UsageService for UsageServiceImpl {
    fn report(&self, period: UsagePeriod) -> Result<UsageReportDTO, ApiRequestError> {
        self.request_handler.get(&report_path(period))
    }
}

fn report_path(period: UsagePeriod) -> String {
    format!("{}?period={}", api::USAGE_PATH, period.as_query_value())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn report_path_carries_the_period() {
        assert_eq!(
            report_path(UsagePeriod::LastMonth),
            "/zeldoc/usage?period=last_month"
        );
    }
}
