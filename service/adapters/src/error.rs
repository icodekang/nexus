//! 提供商客户端错误模块
//!
//! 定义了与 LLM 提供商通信时可能发生的各种错误

use thiserror::Error;

/// 提供商错误枚举
///
/// 包含与提供商通信过程中可能出现的各种错误情况
#[derive(Error, Debug)]
pub enum ProviderError {
    /// 指定的提供商不存在或不可用
    #[error("提供商不可用: {0}")]
    ProviderNotFound(String),

    /// 未设置提供商的 API Key
    #[error("未设置提供商 {0} 的 API Key")]
    ApiKeyNotSet(String),

    /// HTTP 请求失败
    #[error("HTTP 请求失败: {0}")]
    RequestFailed(#[from] reqwest::Error),

    /// 提供商返回了无效的响应
    #[error("无效的响应: {0}")]
    InvalidResponse(String),

    /// 该提供商不支持流式响应
    #[error("该提供商不支持流式响应")]
    StreamingNotSupported,

    /// 该提供商不支持嵌入请求
    #[error("该提供商不支持嵌入请求")]
    EmbeddingsNotSupported,

    /// 提供商返回了错误
    #[error("提供商错误: {0}")]
    ProviderError(String),

    /// 认证失败
    #[error("认证失败: {0}")]
    AuthenticationError(String),

    /// 内部错误（用于非 HTTP 错误）
    #[error("内部错误: {0}")]
    InternalError(String),
}