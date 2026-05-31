//! 提供商客户端适配器模块
//!
//! 直接使用 Rust 实现调用 LLM 提供商 API。
//! 采用配置驱动的方式，便于扩展。
//!
//! # 支持的提供商
//!
//! - **内置**: OpenAI, Anthropic, Google, DeepSeek
//! - **自定义**: 通过 `CUSTOM_PROVIDERS` 环境变量定义
//! - **浏览器模拟器**: 模拟浏览器会话实现零 Token 访问（Claude.ai, ChatGPT）
//!
//! # 添加自定义提供商
//!
//! 设置 `CUSTOM_PROVIDERS` 环境变量为 JSON 数组格式：
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
pub mod tool_calling;
pub mod types;

pub use client::{HttpProviderClient, ProviderClient, ProviderClientFactory};
pub use config::*;
pub use error::*;
pub use tool_calling::*;
pub use types::*;
