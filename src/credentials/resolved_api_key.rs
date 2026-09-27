use super::api_key_source::ApiKeySource;
use super::data_transfer_objects::api_key_secret_dto::ApiKeySecretDTO;

/// The API key the CLI uses and where it came from.
pub struct ResolvedApiKey {
    pub secret: ApiKeySecretDTO,
    pub source: ApiKeySource,
}
