use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// API call log record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiLog {
    pub id: Uuid,
    pub user_id: Uuid,
    pub api_key_id: Uuid,
    pub provider_id: String,
    pub model_id: String,
    pub mode: String,
    pub input_tokens: i32,
    pub output_tokens: i32,
    pub latency_ms: i32,
    pub status: ApiLogStatus,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl ApiLog {
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

    pub fn with_tokens(mut self, input_tokens: i32, output_tokens: i32) -> Self {
        self.input_tokens = input_tokens;
        self.output_tokens = output_tokens;
        self
    }

    pub fn with_latency(mut self, latency_ms: i32) -> Self {
        self.latency_ms = latency_ms;
        self
    }

    pub fn with_error(mut self, error: String) -> Self {
        self.status = ApiLogStatus::Error;
        self.error_message = Some(error);
        self
    }

    pub fn total_tokens(&self) -> i32 {
        self.input_tokens + self.output_tokens
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ApiLogStatus {
    Success,
    Error,
    RateLimited,
}

/// Usage statistics for a user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageStats {
    pub user_id: Uuid,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub total_requests: i64,
    pub total_input_tokens: i64,
    pub total_output_tokens: i64,
    pub total_latency_ms: i64,
    pub usage_by_provider: Vec<ProviderUsage>,
    pub usage_by_model: Vec<ModelUsage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderUsage {
    pub provider: String,
    pub requests: i64,
    pub input_tokens: i64,
    pub output_tokens: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelUsage {
    pub model: String,
    pub provider: String,
    pub requests: i64,
    pub input_tokens: i64,
    pub output_tokens: i64,
}
