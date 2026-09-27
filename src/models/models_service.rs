use super::data_transfer_objects::model_dto::ModelDTO;
use super::data_transfer_objects::model_list_dto::ModelListDTO;
use crate::api_request_error::ApiRequestError;
use crate::api_request_handler::ApiRequestHandler;
use crate::constants::api;

pub trait ModelsService {
    /// The models the API key can use, in the order Zeldoc.ai lists them.
    fn list(&self) -> Result<Vec<ModelDTO>, ApiRequestError>;
}

pub struct ModelsServiceImpl {
    request_handler: ApiRequestHandler,
}

impl ModelsServiceImpl {
    pub fn new(request_handler: ApiRequestHandler) -> Self {
        Self { request_handler }
    }
}

impl ModelsService for ModelsServiceImpl {
    fn list(&self) -> Result<Vec<ModelDTO>, ApiRequestError> {
        let list: ModelListDTO = self.request_handler.get(api::MODEL_CATALOG_PATH)?;
        Ok(list.data)
    }
}
