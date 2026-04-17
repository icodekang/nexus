//! Redis 数据库模块
//!
//! 提供限流、会话管理、缓存、短信验证等功能

use redis::AsyncCommands;
use deadpool_redis::{Pool, Config as DeadpoolConfig, Runtime};
use anyhow::Result;

/// Redis 连接池
pub struct RedisPool(Pool);

impl RedisPool {
    /// 创建新的 Redis 连接池
    ///
    /// # 参数
    /// * `redis_url` - Redis 连接 URL
    pub async fn new(redis_url: &str) -> Result<Self> {
        let cfg = DeadpoolConfig::from_url(redis_url);
        let pool = cfg.create_pool(Some(Runtime::Tokio1))?;

        Ok(Self(pool))
    }

    /// 获取数据库连接
    async fn conn(&self) -> Result<deadpool_redis::Connection> {
        Ok(self.0.get().await?)
    }

    // ============ 限流 ============

    /// 检查用户的限流
    ///
    /// # 参数
    /// * `user_id` - 用户 ID
    /// * `limit` - 时间窗口内的最大请求数
    /// * `window_seconds` - 时间窗口大小（秒）
    ///
    /// # 返回
    /// `(allowed, remaining, reset_time)` - 是否允许、剩余请求数、重置时间戳
    pub async fn check_rate_limit(
        &self,
        user_id: &str,
        limit: i64,
        window_seconds: i64,
    ) -> Result<(bool, i64, i64)> {
        let mut conn = self.conn().await?;
        let key = format!("rate_limit:{}", user_id);
        let now = chrono::Utc::now().timestamp();
        let window_start = now - window_seconds;

        // 移除过期条目
        let _: () = conn.zrembyscore(&key, 0, window_start).await?;

        // 计算当前请求数
        let count: i64 = conn.zcard(&key).await?;

        if count >= limit {
            // 获取最旧的条目以计算重置时间
            let oldest_score: Option<i64> = conn.zrange_withscores::<_, Vec<(String, i64)>>(&key, 0, 0)
                .await?
                .into_iter()
                .next()
                .map(|(_, score)| score);

            let reset_time = oldest_score.unwrap_or(now) + window_seconds;
            return Ok((false, 0, reset_time));
        }

        // 添加新请求
        let _: () = conn.zadd(&key, format!("{}", now), now).await?;
        let _: () = conn.expire(&key, window_seconds).await?;

        Ok((true, limit - count - 1, now + window_seconds))
    }

    // ============ 缓存 ============

    /// 设置带 TTL 的缓存值
    ///
    /// # 参数
    /// * `key` - 缓存键
    /// * `value` - 缓存值
    /// * `ttl_seconds` - 过期时间（秒）
    pub async fn set_with_ttl(&self, key: &str, value: &str, ttl_seconds: i64) -> Result<()> {
        let mut conn = self.conn().await?;
        let _: () = conn.set_ex(key, value, ttl_seconds as u64).await?;
        Ok(())
    }

    /// 获取缓存值
    ///
    /// # 参数
    /// * `key` - 缓存键
    pub async fn get(&self, key: &str) -> Result<Option<String>> {
        let mut conn = self.conn().await?;
        let result: Option<String> = conn.get(key).await?;
        Ok(result)
    }

    /// 删除缓存值
    ///
    /// # 参数
    /// * `key` - 缓存键
    pub async fn delete(&self, key: &str) -> Result<()> {
        let mut conn = self.conn().await?;
        let _: () = conn.del(key).await?;
        Ok(())
    }

    // ============ 会话管理 ============

    /// 存储会话 Token
    ///
    /// # 参数
    /// * `token` - 会话 Token
    /// * `user_id` - 用户 ID
    /// * `ttl_seconds` - 过期时间（秒）
    pub async fn store_session(&self, token: &str, user_id: &str, ttl_seconds: i64) -> Result<()> {
        let mut conn = self.conn().await?;
        let key = format!("session:{}", token);
        let _: () = conn.set_ex(&key, user_id, ttl_seconds as u64).await?;
        Ok(())
    }

    /// 从会话 Token 获取用户 ID
    ///
    /// # 参数
    /// * `token` - 会话 Token
    pub async fn get_session(&self, token: &str) -> Result<Option<String>> {
        let mut conn = self.conn().await?;
        let key = format!("session:{}", token);
        let result: Option<String> = conn.get(&key).await?;
        Ok(result)
    }

    /// 删除会话
    ///
    /// # 参数
    /// * `token` - 会话 Token
    pub async fn delete_session(&self, token: &str) -> Result<()> {
        let mut conn = self.conn().await?;
        let key = format!("session:{}", token);
        let _: () = conn.del(&key).await?;
        Ok(())
    }

    // ============ 模型缓存 ============

    /// 缓存模型列表
    ///
    /// # 参数
    /// * `models_json` - 模型列表的 JSON 字符串
    pub async fn cache_models(&self, models_json: &str) -> Result<()> {
        let mut conn = self.conn().await?;
        let _: () = conn.set_ex("models:all", models_json, 3600).await?;
        Ok(())
    }

    /// 获取缓存的模型列表
    pub async fn get_cached_models(&self) -> Result<Option<String>> {
        let mut conn = self.conn().await?;
        let result: Option<String> = conn.get("models:all").await?;
        Ok(result)
    }

    // ============ 短信验证码 ============

    /// 存储短信验证码（5 分钟有效期）
    ///
    /// # 参数
    /// * `phone` - 手机号
    /// * `code` - 验证码
    pub async fn store_sms_code(&self, phone: &str, code: &str) -> Result<()> {
        let mut conn = self.conn().await?;
        let key = format!("sms_code:{}", phone);
        let _: () = conn.set_ex(&key, code, 300).await?; // 5 分钟 TTL
        Ok(())
    }

    /// 获取短信验证码
    ///
    /// # 参数
    /// * `phone` - 手机号
    pub async fn get_sms_code(&self, phone: &str) -> Result<Option<String>> {
        let mut conn = self.conn().await?;
        let key = format!("sms_code:{}", phone);
        let result: Option<String> = conn.get(&key).await?;
        Ok(result)
    }

    /// 删除短信验证码（验证成功后）
    ///
    /// # 参数
    /// * `phone` - 手机号
    pub async fn delete_sms_code(&self, phone: &str) -> Result<()> {
        let mut conn = self.conn().await?;
        let key = format!("sms_code:{}", phone);
        let _: () = conn.del(&key).await?;
        Ok(())
    }

    /// 检查短信发送限流（每手机号每小时最多 5 条）
    ///
    /// # 参数
    /// * `phone` - 手机号
    ///
    /// # 返回
    /// `(allowed, retry_after)` - 是否允许发送、重试等待秒数
    pub async fn check_sms_rate_limit(&self, phone: &str) -> Result<(bool, i64)> {
        let mut conn = self.conn().await?;
        let key = format!("sms_rate:{}", phone);
        let now = chrono::Utc::now().timestamp();
        let window_start = now - 3600; // 1 小时窗口

        // 移除过期条目
        let _: () = conn.zrembyscore(&key, 0, window_start).await?;

        // 计算当前发送数
        let count: i64 = conn.zcard(&key).await?;

        if count >= 5 {
            return Ok((false, 3600 - (now - window_start)));
        }

        // 添加新条目
        let _: () = conn.zadd(&key, format!("{}", now), now).await?;
        let _: () = conn.expire(&key, 3600).await?;

        Ok((true, 0))
    }

    // ============ 二维码会话（零 Token 认证） ============

    /// 存储二维码会话（5 分钟有效期）
    ///
    /// # 参数
    /// * `code` - 二维码中的验证码
    /// * `session_id` - 会话 ID
    pub async fn store_qr_session(&self, code: &str, session_id: &str) -> Result<()> {
        let mut conn = self.conn().await?;
        let key = format!("qr:code:{}", code);
        let _: () = conn.set_ex(&key, session_id, 300).await?; // 5 分钟 TTL
        Ok(())
    }

    /// 通过验证码获取二维码会话
    ///
    /// # 参数
    /// * `code` - 二维码中的验证码
    pub async fn get_qr_session(&self, code: &str) -> Result<Option<String>> {
        let mut conn = self.conn().await?;
        let key = format!("qr:code:{}", code);
        let result: Option<String> = conn.get(&key).await?;
        Ok(result)
    }

    /// 删除二维码会话
    ///
    /// # 参数
    /// * `code` - 二维码中的验证码
    pub async fn delete_qr_session(&self, code: &str) -> Result<()> {
        let mut conn = self.conn().await?;
        let key = format!("qr:code:{}", code);
        let _: () = conn.del(&key).await?;
        Ok(())
    }

    // ============ 账户会话（零 Token 浏览器会话） ============

    /// 存储账户会话数据（24 小时有效期）
    ///
    /// # 参数
    /// * `account_id` - 账户 ID
    /// * `session_data` - 会话数据 JSON
    pub async fn store_account_session(&self, account_id: &str, session_data: &str) -> Result<()> {
        let mut conn = self.conn().await?;
        let key = format!("account:session:{}", account_id);
        let _: () = conn.set_ex(&key, session_data, 86400).await?; // 24 小时 TTL
        Ok(())
    }

    /// 获取账户会话数据
    ///
    /// # 参数
    /// * `account_id` - 账户 ID
    pub async fn get_account_session(&self, account_id: &str) -> Result<Option<String>> {
        let mut conn = self.conn().await?;
        let key = format!("account:session:{}", account_id);
        let result: Option<String> = conn.get(&key).await?;
        Ok(result)
    }

    /// 删除账户会话
    ///
    /// # 参数
    /// * `account_id` - 账户 ID
    pub async fn delete_account_session(&self, account_id: &str) -> Result<()> {
        let mut conn = self.conn().await?;
        let key = format!("account:session:{}", account_id);
        let _: () = conn.del(&key).await?;
        Ok(())
    }

    // ============ 发布/订阅（实时状态更新） ============

    /// 发布账户状态更新（用于 SSE 通知）
    ///
    /// # 参数
    /// * `account_id` - 账户 ID
    /// * `status` - 状态值
    pub async fn publish_account_status(&self, account_id: &str, status: &str) -> Result<()> {
        let mut conn = self.conn().await?;
        let channel = format!("account:{}", account_id);
        let _: () = conn.publish(&channel, status).await?;
        Ok(())
    }

    // ============ JWT Token 黑名单 ============

    /// 将 Token 加入黑名单（注销时调用）
    ///
    /// # 参数
    /// * `token` - JWT Token
    /// * `ttl_seconds` - 黑名单过期时间（应等于 token 剩余有效期）
    pub async fn blacklist_token(&self, token: &str, ttl_seconds: i64) -> Result<()> {
        let mut conn = self.conn().await?;
        let key = format!("blacklist:jwt:{}", token);
        let _: () = conn.set_ex(&key, "1", ttl_seconds as u64).await?;
        Ok(())
    }

    /// 检查 Token 是否在黑名单中
    ///
    /// # 参数
    /// * `token` - JWT Token
    pub async fn is_token_blacklisted(&self, token: &str) -> Result<bool> {
        let mut conn = self.conn().await?;
        let key = format!("blacklist:jwt:{}", token);
        let exists: bool = conn.exists(&key).await?;
        Ok(exists)
    }
}
