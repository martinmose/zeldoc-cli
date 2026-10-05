use serde::{Deserialize, Serialize};

use super::api_key_secret_dto::ApiKeySecretDTO;

/// One saved API key.
#[derive(Clone, Serialize, Deserialize)]
pub struct ProfileDTO {
    pub api_key: ApiKeySecretDTO,
}
