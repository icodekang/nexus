//! Provider client errors

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProviderError {
    #[error("Provider {0} is not available")]
    ProviderNotFound(String),

    #[error("API key not set for provider {0}")]
    ApiKeyNotSet(String),

    #[error("HTTP request failed: {0}")]
    RequestFailed(#[from] reqwest::Error),

    #[error("Invalid response from provider: {0}")]
    InvalidResponse(String),

    #[error("Streaming not supported for this provider")]
    StreamingNotSupported,

    #[error("Embeddings not supported for this provider")]
    EmbeddingsNotSupported,

    #[error("Provider error: {0}")]
    ProviderError(String),

    #[error("Authentication failed: {0}")]
    AuthenticationError(String),
}