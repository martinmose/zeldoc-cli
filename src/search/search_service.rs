use super::data_transfer_objects::search_response_dto::SearchResponseDTO;
use super::data_transfer_objects::search_result_dto::SearchResultDTO;
use super::request_parameters::search_parameters::SearchParameters;
use crate::api_request_error::ApiRequestError;
use crate::api_request_handler::ApiRequestHandler;
use crate::constants::api;

pub trait SearchService {
    fn search(
        &self,
        parameters: &SearchParameters,
    ) -> Result<Vec<SearchResultDTO>, ApiRequestError>;
}

pub struct SearchServiceImpl {
    request_handler: ApiRequestHandler,
}

impl SearchServiceImpl {
    pub fn new(request_handler: ApiRequestHandler) -> Self {
        Self { request_handler }
    }
}

impl SearchService for SearchServiceImpl {
    fn search(
        &self,
        parameters: &SearchParameters,
    ) -> Result<Vec<SearchResultDTO>, ApiRequestError> {
        let response: SearchResponseDTO =
            self.request_handler.post(api::SEARCH_PATH, parameters)?;
        Ok(response.results)
    }
}
