use std::time::Duration;

use reqwest::blocking::{Client, RequestBuilder};
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::api_request_error::ApiRequestError;
use crate::constants::api;
use crate::credentials::data_transfer_objects::api_key_secret_dto::ApiKeySecretDTO;

/// Sends requests authenticated with a gateway API key to Zeldoc.ai and
/// decodes the JSON responses.
pub struct ApiRequestHandler {
    http_client: Client,
    api_key: ApiKeySecretDTO,
}

impl ApiRequestHandler {
    pub fn new(api_key: ApiKeySecretDTO) -> Result<Self, ApiRequestError> {
        let http_client = Client::builder()
            .user_agent(concat!("zeldoc-cli/", env!("CARGO_PKG_VERSION")))
            .timeout(Duration::from_secs(60))
            .build()
            .map_err(ApiRequestError::ClientSetup)?;
        Ok(Self {
            http_client,
            api_key,
        })
    }

    pub fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T, ApiRequestError> {
        self.execute(self.http_client.get(self.url(path)))
    }

    pub fn post<B: Serialize, T: DeserializeOwned>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T, ApiRequestError> {
        self.execute(self.http_client.post(self.url(path)).json(body))
    }

    fn url(&self, path: &str) -> String {
        format!("{}{path}", api::BASE_URL)
    }

    fn execute<T: DeserializeOwned>(&self, request: RequestBuilder) -> Result<T, ApiRequestError> {
        let response = request
            .bearer_auth(self.api_key.expose())
            .send()
            .map_err(ApiRequestError::Network)?;
        let status = response.status();
        if !status.is_success() {
            let body = response.text().unwrap_or_default();
            return Err(ApiRequestError::from_response_body(status, &body));
        }
        response.json().map_err(ApiRequestError::Deserialization)
    }
}
