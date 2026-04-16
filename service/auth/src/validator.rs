//! 验证器模块
//!
//! 提供 API Key 和 Bearer Token 的验证功能

use db::PostgresPool;
use models::{User, ApiKey};
use uuid::Uuid;

use crate::AuthError;

/// API Key 验证器
///
/// 验证 API Key 并返回关联的用户信息
pub struct ApiKeyValidator<'a> {
    db: &'a PostgresPool,
}

impl<'a> ApiKeyValidator<'a> {
    /// 创建新的验证器
    pub fn new(db: &'a PostgresPool) -> Self {
        Self { db }
    }

    /// 验证 API Key（明文，不含 "Bearer " 前缀）并返回关联的用户
    ///
    /// # 参数
    /// * `key` - API Key 明文
    ///
    /// # 返回
    /// - `(ApiKey, User)`: API Key 和用户信息
    pub async fn validate(&self, key: &str) -> Result<(ApiKey, User), AuthError> {
        // 对提供的 Key 进行哈希
        let key_hash = super::keygen::ApiKeyGenerator::hash_key(key);

        // 在数据库中查找 Key
        let api_key = self.db.get_api_key_by_hash(&key_hash)
            .await
            .map_err(|_| AuthError::ApiKeyInvalid)?
            .ok_or(AuthError::ApiKeyNotFound)?;

        if !api_key.is_active {
            return Err(AuthError::ApiKeyInvalid);
        }

        // 获取用户
        let user = self.db.get_user_by_id(api_key.user_id)
            .await
            .map_err(|_| AuthError::ApiKeyInvalid)?
            .ok_or(AuthError::UserNotFound)?;

        // 检查订阅状态
        if !user.is_subscription_active() {
            return Err(AuthError::SubscriptionExpired);
        }

        // 更新最后使用时间戳
        let _ = self.db.update_api_key_last_used(api_key.id).await;

        Ok((api_key, user))
    }

    /// 验证 Authorization 头中的 Bearer Token
    ///
    /// 验证前会去除 "Bearer " 前缀
    ///
    /// # 参数
    /// * `auth_header` - Authorization 头的值
    pub async fn validate_bearer(&self, auth_header: &str) -> Result<(ApiKey, User), AuthError> {
        if !auth_header.starts_with("Bearer ") {
            return Err(AuthError::ApiKeyInvalid);
        }

        let key = &auth_header[7..];
        self.validate(key).await
    }
}

/// Bearer Token 验证器
pub struct BearerValidator<'a> {
    db: &'a PostgresPool,
}

impl<'a> BearerValidator<'a> {
    /// 创建新的验证器
    pub fn new(db: &'a PostgresPool) -> Self {
        Self { db }
    }

    /// 验证 Bearer Token 并返回用户
    ///
    /// # 参数
    /// * `auth_header` - Authorization 头的值
    pub async fn validate(&self, auth_header: &str) -> Result<User, AuthError> {
        if !auth_header.starts_with("Bearer ") {
            return Err(AuthError::ApiKeyInvalid);
        }

        let key = &auth_header[7..];

        // 对提供的 Key 进行哈希
        let key_hash = super::keygen::ApiKeyGenerator::hash_key(key);

        // 在数据库中查找 Key
        let api_key = self.db.get_api_key_by_hash(&key_hash)
            .await
            .map_err(|_| AuthError::ApiKeyInvalid)?
            .ok_or(AuthError::ApiKeyNotFound)?;

        if !api_key.is_active {
            return Err(AuthError::ApiKeyInvalid);
        }

        // 获取用户
        let user = self.db.get_user_by_id(api_key.user_id)
            .await
            .map_err(|_| AuthError::ApiKeyInvalid)?
            .ok_or(AuthError::UserNotFound)?;

        // 检查订阅状态
        if !user.is_subscription_active() {
            return Err(AuthError::SubscriptionExpired);
        }

        Ok(user)
    }
}
