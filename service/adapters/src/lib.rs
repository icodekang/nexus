//! Provider Client Module
//!
//! Direct Rust implementation for calling LLM provider APIs.
//! Uses configuration-driven approach for easy extensibility.
//!
//! # Supported Providers
//!
//! - **Built-in**: OpenAI, Anthropic, Google, DeepSeek
//! - **Custom**: Define via CUSTOM_PROVIDERS environment variable
//! - **Browser Emulator**: Simulates browser sessions for zero-token access (Claude.ai, ChatGPT)
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

pub mod browser_emulator;
pub mod client;
pub mod config;
pub mod error;
pub mod providers;
pub mod types;
pub mod account_pool;

pub use browser_emulator::*;
pub use client::*;
pub use config::*;
pub use error::*;
pub use types::*;
pub use account_pool::*;