use serde::{Deserialize, Serialize};

use super::api_key_secret_dto::ApiKeySecretDTO;

/// The file `zeldoc auth login` writes: `{"api_key": "sk-..."}`.
#[derive(Serialize, Deserialize)]
pub struct CredentialsFileDTO {
    pub api_key: ApiKeySecretDTO,
}
