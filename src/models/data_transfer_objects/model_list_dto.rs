use serde::Deserialize;

use super::model_dto::ModelDTO;

/// The model catalog response: the models an API key may call, in the order
/// the gateway lists them.
#[derive(Debug, Deserialize)]
pub struct ModelListDTO {
    pub data: Vec<ModelDTO>,
}
