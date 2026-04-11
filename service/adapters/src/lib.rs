//! Provider Client Module
//!
//! Direct Rust implementation for calling LLM provider APIs.
//! Uses configuration-driven approach for easy extensibility.
//!
//! # Supported Providers
//!
//! - **Built-in**: OpenAI, Anthropic, Google, DeepSeek
//! - **Custom**: Define via CUSTOM_PROVIDERS environment variable
//!
//! # Adding Custom Providers
//!
//! Set CUSTOM_PROVIDERS environment variable with JSON array:
//! ```json
//! [
//!   {
//!     "id": "ollama",
//!     "name": "Ollama",
//!     "baseUrl": "http://localhost:11434/v1",
//!     "auth": "bearer",
//!     "apiKeyEnv": "OLLAMA_API_KEY",
//!     "chatPath": "/chat/completions",
//!     "openaiCompatible": true
//!   }
//! ]
//! ```

pub mod client;
pub mod config;
pub mod error;
pub mod providers;
pub mod types;

pub use client::*;
pub use config::*;
pub use error::*;
pub use types::*;