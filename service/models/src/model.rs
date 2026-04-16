//! 模型模块

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::Provider;

/// 支持的 LLM 模型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmModel {
    /// 模型 ID
    pub id: Uuid,
    /// 提供商 ID
    pub provider_id: String,
    /// 模型名称
    pub name: String,
    /// 模型标识符（URL 友好）
    pub slug: String,
    /// 提供商的实际模型 ID
    pub model_id: String,
    /// 模型模式
    pub mode: ModelMode,
    /// 上下文窗口大小（Token 数）
    pub context_window: i32,
    /// 支持的能力列表
    pub capabilities: Vec<String>,
    /// 是否激活
    pub is_active: bool,
    /// 创建时间
    pub created_at: DateTime<Utc>,
}

impl LlmModel {
    /// 创建新的 LLM 模型
    ///
    /// # 参数
    /// * `provider_id` - 提供商 ID
    /// * `name` - 模型名称
    /// * `slug` - 模型标识符
    /// * `model_id` - 提供商的实际模型 ID
    /// * `mode` - 模型模式
    /// * `context_window` - 上下文窗口大小
    pub fn new(
        provider_id: String,
        name: String,
        slug: String,
        model_id: String,
        mode: ModelMode,
        context_window: i32,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            provider_id,
            name,
            slug,
            model_id,
            mode,
            context_window,
            capabilities: vec![],
            is_active: true,
            created_at: Utc::now(),
        }
    }

    /// 设置模型能力
    ///
    /// # 参数
    /// * `capabilities` - 能力列表（如 vision, function_call 等）
    pub fn with_capabilities(mut self, capabilities: Vec<String>) -> Self {
        self.capabilities = capabilities;
        self
    }
}

/// 模型模式
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ModelMode {
    /// 聊天模式
    Chat,
    /// 完成模式
    Completion,
    /// 嵌入模式
    Embedding,
}

impl ModelMode {
    /// 转换为字符串
    pub fn as_str(&self) -> &'static str {
        match self {
            ModelMode::Chat => "chat",
            ModelMode::Completion => "completion",
            ModelMode::Embedding => "embedding",
        }
    }
}

/// 带提供商信息的模型（用于 API 响应）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelWithProvider {
    /// 模型标识符
    pub id: String,
    /// 模型名称
    pub name: String,
    /// 提供商标识符
    pub provider: String,
    /// 提供商名称
    pub provider_name: String,
    /// 上下文窗口大小
    pub context_window: i32,
    /// 支持的能力列表
    pub capabilities: Vec<String>,
}

impl ModelWithProvider {
    /// 从模型和提供商创建
    ///
    /// # 参数
    /// * `model` - LLM 模型
    /// * `provider` - 提供商
    pub fn from_model(model: &LlmModel, provider: &Provider) -> Self {
        Self {
            id: model.slug.clone(),
            name: model.name.clone(),
            provider: provider.slug.clone(),
            provider_name: provider.name.clone(),
            context_window: model.context_window,
            capabilities: model.capabilities.clone(),
        }
    }
}

/// 内置模型集合
pub struct BuiltinModels;

impl BuiltinModels {
    /// 获取所有内置模型
    pub fn all() -> Vec<LlmModel> {
        use ModelMode::*;

        vec![
            // OpenAI 模型
            Self::chat("openai", "GPT-4o", "gpt-4o", "gpt-4o", 128000, vec!["vision".to_string(), "function_call".to_string()]),
            Self::chat("openai", "GPT-4o Mini", "gpt-4o-mini", "gpt-4o-mini", 128000, vec!["function_call".to_string()]),
            Self::chat("openai", "GPT-4 Turbo", "gpt-4-turbo", "gpt-4-turbo", 128000, vec!["vision".to_string()]),
            Self::chat("openai", "GPT-3.5 Turbo", "gpt-3.5-turbo", "gpt-3.5-turbo-1106", 16385, vec![]),

            // Anthropic 模型
            Self::chat("anthropic", "Claude 3.5 Sonnet", "claude-3-5-sonnet", "claude-3-5-sonnet-20241022", 200000, vec!["vision".to_string()]),
            Self::chat("anthropic", "Claude 3 Opus", "claude-3-opus", "claude-3-opus-20240229", 200000, vec!["vision".to_string()]),
            Self::chat("anthropic", "Claude 3 Haiku", "claude-3-haiku", "claude-3-haiku-20240307", 200000, vec![]),

            // Google 模型
            Self::chat("google", "Gemini 1.5 Pro", "gemini-1-5-pro", "gemini-1.5-pro", 2000000, vec!["vision".to_string()]),
            Self::chat("google", "Gemini 1.5 Flash", "gemini-1-5-flash", "gemini-1.5-flash", 1000000, vec!["vision".to_string()]),
            Self::chat("google", "Gemini 1.0 Pro", "gemini-1-0-pro", "gemini-pro", 32768, vec![]),

            // DeepSeek 模型
            Self::chat("deepseek", "DeepSeek V3", "deepseek-chat", "deepseek-chat", 64000, vec![]),
            Self::chat("deepseek", "DeepSeek Coder", "deepseek-coder", "deepseek-coder", 64000, vec![]),
        ]
    }

    /// 创建聊天模式模型
    fn chat(provider: &str, name: &str, slug: &str, model_id: &str, context_window: i32, capabilities: Vec<String>) -> LlmModel {
        LlmModel::new(
            provider.to_string(),
            name.to_string(),
            slug.to_string(),
            model_id.to_string(),
            ModelMode::Chat,
            context_window,
        )
        .with_capabilities(capabilities)
    }
}