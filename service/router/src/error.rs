use thiserror::Error;

#[derive(Error, Debug)]
pub enum RouterError {
    #[error("Model not found: {0}")]
    ModelNotFound(String),

    #[error("No provider available for this model")]
    NoProviderAvailable,

    #[error("Provider not found: {0}")]
    ProviderNotFound(String),

    #[error("Request failed: {0}")]
    RequestFailed(String),

    #[error("Invalid response from provider: {0}")]
    InvalidResponse(String),
}
