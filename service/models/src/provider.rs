//! 提供商模块

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// LLM 提供商（OpenAI, Anthropic, Google 等）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Provider {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub logo_url: Option<String>,
    pub api_base_url: String,
    pub api_type: String,
    pub is_active: bool,
    pub priority: i32,
    pub created_at: DateTime<Utc>,
}

impl Provider {
    /// 创建新的提供商
    ///
    /// # 参数
    /// * `name` - 提供商名称
    /// * `slug` - 提供商标识符
    /// * `api_base_url` - API 基础 URL
    pub fn new(name: String, slug: String, api_base_url: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            slug,
            logo_url: None,
            api_base_url,
            api_type: "openai".to_string(),
            is_active: true,
            priority: 100,
            created_at: Utc::now(),
        }
    }

    pub fn with_api_type(mut self, api_type: String) -> Self {
        self.api_type = api_type;
        self
    }

    /// 设置 Logo URL
    pub fn with_logo(mut self, logo_url: String) -> Self {
        self.logo_url = Some(logo_url);
        self
    }

    /// 设置优先级
    ///
    /// # 参数
    /// * `priority` - 优先级（数值越低越优先）
    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }
}

/// 内置提供商集合
pub struct Providers;

impl Providers {
    /// OpenAI 提供商标识符
    pub const OPENAI: &'static str = "openai";
    /// Anthropic 提供商标识符
    pub const ANTHROPIC: &'static str = "anthropic";
    /// Google 提供商标识符
    pub const GOOGLE: &'static str = "google";
    /// DeepSeek 提供商标识符
    pub const DEEPSEEK: &'static str = "deepseek";

    /// 获取所有内置提供商
    pub fn all() -> Vec<Provider> {
        vec![
            Provider::new(
                "OpenAI".to_string(),
                Self::OPENAI.to_string(),
                "https://api.openai.com/v1".to_string(),
            )
            .with_api_type("openai".to_string())
            .with_priority(10),
            Provider::new(
                "Anthropic".to_string(),
                Self::ANTHROPIC.to_string(),
                "https://api.anthropic.com/v1".to_string(),
            )
            .with_api_type("anthropic".to_string())
            .with_priority(20),
            Provider::new(
                "Google".to_string(),
                Self::GOOGLE.to_string(),
                "https://generativelanguage.googleapis.com/v1beta".to_string(),
            )
            .with_priority(30),
            Provider::new(
                "DeepSeek".to_string(),
                Self::DEEPSEEK.to_string(),
                "https://api.deepseek.com/v1".to_string(),
            )
            .with_priority(40),
        ]
    }
}