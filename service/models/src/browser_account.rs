//! 浏览器账户模块（用于零 Token 访问）

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 零 Token 浏览器账户状态
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BrowserAccountStatus {
    /// 待处理 - 二维码已生成，等待认证
    Pending,
    /// 活跃 - 已认证并可使用
    Active,
    /// 已过期 - 会话已过期
    Expired,
    /// 错误 - 认证失败或发生错误
    Error,
}

impl BrowserAccountStatus {
    /// 转换为字符串
    pub fn as_str(&self) -> &'static str {
        match self {
            BrowserAccountStatus::Pending => "pending",
            BrowserAccountStatus::Active => "active",
            BrowserAccountStatus::Expired => "expired",
            BrowserAccountStatus::Error => "error",
        }
    }

    /// 从字符串解析
    ///
    /// # 参数
    /// * `s` - 状态字符串
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "active" => BrowserAccountStatus::Active,
            "expired" => BrowserAccountStatus::Expired,
            "error" => BrowserAccountStatus::Error,
            _ => BrowserAccountStatus::Pending,
        }
    }
}

/// 零 Token 浏览器账户（通过二维码认证）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserAccount {
    /// 账户 ID
    pub id: Uuid,
    /// 提供商（"claude", "chatgpt"）
    pub provider: String,
    /// 登录邮箱（如果有）
    pub email: Option<String>,
    /// 加密的会话数据（Cookie/Token 的 JSON）
    pub session_data_encrypted: String,
    /// 账户状态
    pub status: BrowserAccountStatus,
    /// 已服务的请求总数
    pub request_count: i64,
    /// 最后使用时间
    pub last_used_at: Option<DateTime<Utc>>,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 更新时间
    pub updated_at: DateTime<Utc>,
}

impl BrowserAccount {
    /// 创建新的浏览器账户
    ///
    /// # 参数
    /// * `provider` - 提供商标识符
    pub fn new(provider: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            provider,
            email: None,
            session_data_encrypted: String::new(),
            status: BrowserAccountStatus::Pending,
            request_count: 0,
            last_used_at: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    /// 设置邮箱
    pub fn with_email(mut self, email: String) -> Self {
        self.email = Some(email);
        self
    }

    /// 设置会话数据
    ///
    /// # 参数
    /// * `data` - 加密的会话数据 JSON
    pub fn with_session_data(mut self, data: String) -> Self {
        self.session_data_encrypted = data;
        self
    }

    /// 激活账户
    pub fn activate(mut self) -> Self {
        self.status = BrowserAccountStatus::Active;
        self.updated_at = Utc::now();
        self
    }

    /// 标记为错误状态
    pub fn mark_error(mut self) -> Self {
        self.status = BrowserAccountStatus::Error;
        self.updated_at = Utc::now();
        self
    }

    /// 标记为过期
    pub fn mark_expired(mut self) -> Self {
        self.status = BrowserAccountStatus::Expired;
        self.updated_at = Utc::now();
        self
    }

    /// 增加请求计数
    pub fn increment_request_count(&mut self) {
        self.request_count += 1;
        self.last_used_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }

    /// 检查账户是否活跃
    pub fn is_active(&self) -> bool {
        self.status == BrowserAccountStatus::Active
    }
}
