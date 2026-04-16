//! API 日志模块

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// API 调用日志记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiLog {
    /// 日志 ID
    pub id: Uuid,
    /// 用户 ID
    pub user_id: Uuid,
    /// API Key ID
    pub api_key_id: Uuid,
    /// 提供商 ID
    pub provider_id: String,
    /// 模型 ID
    pub model_id: String,
    /// 模式（chat/completion/embedding）
    pub mode: String,
    /// 输入 Token 数
    pub input_tokens: i32,
    /// 输出 Token 数
    pub output_tokens: i32,
    /// 延迟（毫秒）
    pub latency_ms: i32,
    /// 状态
    pub status: ApiLogStatus,
    /// 错误消息（如果有）
    pub error_message: Option<String>,
    /// 创建时间
    pub created_at: DateTime<Utc>,
}

impl ApiLog {
    /// 创建新的 API 日志
    ///
    /// # 参数
    /// * `user_id` - 用户 ID
    /// * `api_key_id` - API Key ID
    /// * `provider_id` - 提供商 ID
    /// * `model_id` - 模型 ID
    /// * `mode` - 模式
    pub fn new(
        user_id: Uuid,
        api_key_id: Uuid,
        provider_id: String,
        model_id: String,
        mode: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id,
            api_key_id,
            provider_id,
            model_id,
            mode,
            input_tokens: 0,
            output_tokens: 0,
            latency_ms: 0,
            status: ApiLogStatus::Success,
            error_message: None,
            created_at: Utc::now(),
        }
    }

    /// 设置 Token 数量
    pub fn with_tokens(mut self, input_tokens: i32, output_tokens: i32) -> Self {
        self.input_tokens = input_tokens;
        self.output_tokens = output_tokens;
        self
    }

    /// 设置延迟
    pub fn with_latency(mut self, latency_ms: i32) -> Self {
        self.latency_ms = latency_ms;
        self
    }

    /// 设置错误信息
    ///
    /// # 参数
    /// * `error` - 错误消息
    pub fn with_error(mut self, error: String) -> Self {
        self.status = ApiLogStatus::Error;
        self.error_message = Some(error);
        self
    }

    /// 获取总 Token 数
    pub fn total_tokens(&self) -> i32 {
        self.input_tokens + self.output_tokens
    }
}

/// API 日志状态
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ApiLogStatus {
    /// 成功
    Success,
    /// 错误
    Error,
    /// 限流
    RateLimited,
}

/// 用户使用统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageStats {
    /// 用户 ID
    pub user_id: Uuid,
    /// 统计周期开始时间
    pub period_start: DateTime<Utc>,
    /// 统计周期结束时间
    pub period_end: DateTime<Utc>,
    /// 总请求数
    pub total_requests: i64,
    /// 总输入 Token 数
    pub total_input_tokens: i64,
    /// 总输出 Token 数
    pub total_output_tokens: i64,
    /// 总延迟（毫秒）
    pub total_latency_ms: i64,
    /// 按提供商统计的使用量
    pub usage_by_provider: Vec<ProviderUsage>,
    /// 按模型统计的使用量
    pub usage_by_model: Vec<ModelUsage>,
}

/// 按提供商统计的使用量
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderUsage {
    /// 提供商标识符
    pub provider: String,
    /// 请求数
    pub requests: i64,
    /// 输入 Token 数
    pub input_tokens: i64,
    /// 输出 Token 数
    pub output_tokens: i64,
}

/// 按模型统计的使用量
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelUsage {
    /// 模型标识符
    pub model: String,
    /// 提供商标识符
    pub provider: String,
    /// 请求数
    pub requests: i64,
    /// 输入 Token 数
    pub input_tokens: i64,
    /// 输出 Token 数
    pub output_tokens: i64,
}
