use super::data_transfer_objects::key_details_dto::KeyDetailsDTO;
use crate::api_request_error::ApiRequestError;
use crate::api_request_handler::ApiRequestHandler;
use crate::constants::api;

pub trait KeyDetailsService {
    /// The API key's name and its values for the organization's key fields.
    fn details(&self) -> Result<KeyDetailsDTO, ApiRequestError>;
}

pub struct KeyDetailsServiceImpl {
    request_handler: ApiRequestHandler,
}

impl KeyDetailsServiceImpl {
    pub fn new(request_handler: ApiRequestHandler) -> Self {
        Self { request_handler }
    }
}

impl KeyDetailsService for KeyDetailsServiceImpl {
    fn details(&self) -> Result<KeyDetailsDTO, ApiRequestError> {
        self.request_handler.get(api::KEY_DETAILS_PATH)
    }
}
