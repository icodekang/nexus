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
    pub id: Uuid,
    pub provider: String,
    pub name: Option<String>,
    pub email: Option<String>,
    pub session_data_encrypted: String,
    pub status: BrowserAccountStatus,
    pub session_status: String,
    pub session_expires_at: Option<DateTime<Utc>>,
    pub request_count: i64,
    pub last_used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl BrowserAccount {
    pub fn new(provider: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            provider,
            name: None,
            email: None,
            session_data_encrypted: String::new(),
            status: BrowserAccountStatus::Pending,
            session_status: "pending".to_string(),
            session_expires_at: None,
            request_count: 0,
            last_used_at: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    pub fn with_name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }

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

    pub fn activate(mut self) -> Self {
        self.status = BrowserAccountStatus::Active;
        self.session_status = "active".to_string();
        self.session_expires_at = Some(Utc::now() + chrono::Duration::days(30));
        self.updated_at = Utc::now();
        self
    }

    pub fn mark_error(mut self) -> Self {
        self.status = BrowserAccountStatus::Error;
        self.session_status = "error".to_string();
        self.updated_at = Utc::now();
        self
    }

    pub fn mark_expired(mut self) -> Self {
        self.status = BrowserAccountStatus::Expired;
        self.session_status = "expired".to_string();
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
