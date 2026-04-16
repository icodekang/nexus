//! 二维码会话模块
//!
//! 用于浏览器认证流程中的二维码会话管理

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 浏览器认证流程中的二维码会话
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QrCodeSession {
    /// 会话 ID
    pub id: Uuid,
    /// 关联的浏览器账户 ID
    pub account_id: Uuid,
    /// 6 位随机验证码
    pub code: String,
    /// 二维码过期时间（5 分钟）
    pub code_expires_at: DateTime<Utc>,
    /// 认证完成时间
    pub auth_completed_at: Option<DateTime<Utc>>,
    /// 创建时间
    pub created_at: DateTime<Utc>,
}

impl QrCodeSession {
    /// 创建新的二维码会话
    ///
    /// # 参数
    /// * `account_id` - 关联的浏览器账户 ID
    pub fn new(account_id: Uuid) -> Self {
        let code = format!("{:06}", rand::random::<u32>() % 900000 + 100000);
        Self {
            id: Uuid::new_v4(),
            account_id,
            code,
            code_expires_at: Utc::now() + chrono::Duration::minutes(5),
            auth_completed_at: None,
            created_at: Utc::now(),
        }
    }

    /// 检查是否已过期
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.code_expires_at
    }

    /// 检查是否已完成认证
    pub fn is_completed(&self) -> bool {
        self.auth_completed_at.is_some()
    }

    /// 标记为已完成认证
    pub fn mark_completed(&mut self) {
        self.auth_completed_at = Some(Utc::now());
    }
}
