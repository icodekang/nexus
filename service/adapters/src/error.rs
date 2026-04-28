//! Provider client error types

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProviderError {
    #[error("Provider not available: {0}")]
    ProviderNotFound(String),

    #[error("API key not set for provider {0}")]
    ApiKeyNotSet(String),

    #[error("HTTP request failed: {0}")]
    RequestFailed(#[from] reqwest::Error),

    #[error("Invalid response: {0}")]
    InvalidResponse(String),

    #[error("Streaming not supported for this provider")]
    StreamingNotSupported,

    #[error("Embeddings not supported for this provider")]
    EmbeddingsNotSupported,

    #[error("Provider error: {0}")]
    ProviderError(String),

    #[error("Authentication failed: {0}")]
    AuthenticationError(String),

    #[error("Login failed: {0}")]
    LoginFailed(String),

    #[error("Session expired")]
    SessionExpired,

    #[error("Page structure changed, selectors need update")]
    PageStructureChanged(String),

    #[error("Cloudflare challenge detected, manual intervention required")]
    CloudflareChallenge,

    #[error("Chrome/Chromium browser not found on system")]
    ChromeNotFound,

    #[error("Provider blocked headless browser (CAPTCHA / Cloudflare / bot detection)")]
    BlockedByProvider,

    #[error("Internal error: {0}")]
    InternalError(String),
}