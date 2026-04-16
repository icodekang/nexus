//! 提供商 API Key 模块

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// LLM 提供商的加密 API Key
///
/// 用于系统端存储提供商的 API Key
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderKey {
    /// Key ID
    pub id: Uuid,
    /// 提供商标识符
    pub provider_slug: String,
    /// 加密的 API Key
    pub api_key_encrypted: String,
    /// API Key 前缀（用于显示）
    pub api_key_prefix: String,
    /// API 基础 URL
    pub base_url: String,
    /// 是否激活
    pub is_active: bool,
    /// 优先级（数值越低越优先）
    pub priority: i32,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 更新时间
    pub updated_at: DateTime<Utc>,
}

impl ProviderKey {
    /// 创建新的提供商 Key
    ///
    /// # 参数
    /// * `provider_slug` - 提供商标识符
    /// * `api_key_encrypted` - 加密的 API Key
    /// * `api_key_prefix` - API Key 前缀
    /// * `base_url` - API 基础 URL
    pub fn new(provider_slug: String, api_key_encrypted: String, api_key_prefix: String, base_url: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            provider_slug,
            api_key_encrypted,
            api_key_prefix,
            base_url,
            is_active: true,
            priority: 100,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    /// 设置优先级
    ///
    /// # 参数
    /// * `priority` - 优先级（数值越低越优先）
    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    /// 生成掩码显示的 API Key
    ///
    /// # 示例
    /// `"sk-...a1b2"`
    pub fn masked_key(&self) -> String {
        if self.api_key_prefix.len() >= 4 {
            format!("{}...{}", &self.api_key_prefix[..4], &self.api_key_prefix[self.api_key_prefix.len()-4..])
        } else {
            format!("{}...", self.api_key_prefix)
        }
    }
}
