//! 用户模块

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 用户账户
///
/// 包含用户的基本信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    /// 用户 ID
    pub id: Uuid,
    /// 用户邮箱
    pub email: String,
    /// 手机号（可选）
    pub phone: Option<String>,
    /// 密码哈希（可选，用于邮箱密码登录）
    pub password_hash: Option<String>,
    /// 是否为管理员
    pub is_admin: bool,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 更新时间
    pub updated_at: DateTime<Utc>,
}

impl User {
    /// 创建新用户
    ///
    /// # 参数
    /// * `email` - 用户邮箱
    pub fn new(email: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            email,
            phone: None,
            password_hash: None,
            is_admin: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    /// 设置密码哈希
    pub fn with_password(mut self, password_hash: String) -> Self {
        self.password_hash = Some(password_hash);
        self
    }

    /// 设置手机号
    pub fn with_phone(mut self, phone: String) -> Self {
        self.phone = Some(phone);
        self
    }
}

/// Nexus Key 类型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NexusKeyType {
    /// OpenAI SDK 兼容类型，端点 /v1/openai
    #[serde(rename = "openai_sdk")]
    OpenAiSdk,
    /// Anthropic SDK 兼容类型，端点 /v1/anthropic
    #[serde(rename = "anthropic_sdk")]
    AnthropicSdk,
    /// HTTP 协议类型（Anthropic Messages API），端点 /v1/messages
    #[serde(rename = "http_messages")]
    HttpMessages,
}

impl NexusKeyType {
    pub fn as_str(&self) -> &'static str {
        match self {
            NexusKeyType::OpenAiSdk => "openai_sdk",
            NexusKeyType::AnthropicSdk => "anthropic_sdk",
            NexusKeyType::HttpMessages => "http_messages",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "openai_sdk" => NexusKeyType::OpenAiSdk,
            "anthropic_sdk" => NexusKeyType::AnthropicSdk,
            _ => NexusKeyType::HttpMessages,
        }
    }
}

/// API Key
///
/// 用户生成的 API Key，用于认证和计费
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey {
    /// Key ID
    pub id: Uuid,
    /// 所属用户 ID
    pub user_id: Uuid,
    /// Key 的哈希值（存储用）
    pub key_hash: String,
    /// Key 的前缀（用于显示）
    pub key_prefix: String,
    /// Key 名称（可选）
    pub name: Option<String>,
    /// 是否激活
    pub is_active: bool,
    /// 最后使用时间
    pub last_used_at: Option<DateTime<Utc>>,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// Nexus Key 类型
    pub key_type: NexusKeyType,
}

impl ApiKey {
    /// 创建新的 API Key
    ///
    /// # 参数
    /// * `user_id` - 所属用户 ID
    /// * `key_hash` - Key 的哈希值
    /// * `key_prefix` - Key 的前缀
    pub fn new(user_id: Uuid, key_hash: String, key_prefix: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id,
            key_hash,
            key_prefix,
            name: None,
            is_active: true,
            last_used_at: None,
            created_at: Utc::now(),
            key_type: NexusKeyType::HttpMessages,
        }
    }

    /// 设置 Key 名称
    pub fn with_name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }

    /// 设置 Key 类型
    pub fn with_key_type(mut self, key_type: NexusKeyType) -> Self {
        self.key_type = key_type;
        self
    }
}
