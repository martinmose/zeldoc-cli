use serde::Deserialize;

use super::search_result_dto::SearchResultDTO;

#[derive(Debug, Deserialize)]
pub struct SearchResponseDTO {
    pub results: Vec<SearchResultDTO>,
}
