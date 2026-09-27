/// Addresses and paths of the Zeldoc.ai API.
pub mod api {
    /// The gateway's OpenAI-compatible API. ZELDOC_BASE_URL overrides it.
    pub const DEFAULT_BASE_URL: &str = "https://api.zeldoc.ai/v1";
    /// The model catalog. Unlike `/v1/models`, which only lists names, it has
    /// limits, prices and capabilities.
    pub const MODEL_CATALOG_PATH: &str = "/zeldoc/models";
    pub const SEARCH_PATH: &str = "/search/zeldoc-search";
    pub const API_KEY_DOCS_URL: &str =
        "https://docs.zeldoc.ai/connect-opencode#generate-an-api-key";
}

/// Environment variables the CLI reads.
pub mod environment {
    /// The API key; takes precedence over the saved key.
    pub const API_KEY: &str = "ZELDOC_API_KEY";
    /// Overrides `api::DEFAULT_BASE_URL`, for pointing at another gateway.
    pub const BASE_URL: &str = "ZELDOC_BASE_URL";
    /// Overrides where the saved key lives.
    pub const CONFIG_DIRECTORY: &str = "ZELDOC_CONFIG_DIR";
}
