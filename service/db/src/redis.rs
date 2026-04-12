use redis::AsyncCommands;
use deadpool_redis::{Pool, Config as DeadpoolConfig, Runtime};
use anyhow::Result;

pub struct RedisPool(Pool);

impl RedisPool {
    pub async fn new(redis_url: &str) -> Result<Self> {
        let cfg = DeadpoolConfig::from_url(redis_url);
        let pool = cfg.create_pool(Some(Runtime::Tokio1))?;

        Ok(Self(pool))
    }

    async fn conn(&self) -> Result<deadpool_redis::Connection> {
        Ok(self.0.get().await?)
    }

    // ============ Rate Limiting ============

    /// Check rate limit for a user
    /// Returns (allowed, remaining, reset_time)
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

        // Remove old entries
        let _: () = conn.zrembyscore(&key, 0, window_start).await?;

        // Count current requests
        let count: i64 = conn.zcard(&key).await?;

        if count >= limit {
            // Get the oldest entry to calculate reset time
            let oldest_score: Option<i64> = conn.zrange_withscores::<_, Vec<(String, i64)>>(&key, 0, 0)
                .await?
                .into_iter()
                .next()
                .map(|(_, score)| score);

            let reset_time = oldest_score.unwrap_or(now) + window_seconds;
            return Ok((false, 0, reset_time));
        }

        // Add new request
        let _: () = conn.zadd(&key, format!("{}", now), now).await?;
        let _: () = conn.expire(&key, window_seconds).await?;

        Ok((true, limit - count - 1, now + window_seconds))
    }

    // ============ Caching ============

    /// Cache a value with TTL
    pub async fn set_with_ttl(&self, key: &str, value: &str, ttl_seconds: i64) -> Result<()> {
        let mut conn = self.conn().await?;
        let _: () = conn.set_ex(key, value, ttl_seconds as u64).await?;
        Ok(())
    }

    /// Get a cached value
    pub async fn get(&self, key: &str) -> Result<Option<String>> {
        let mut conn = self.conn().await?;
        let result: Option<String> = conn.get(key).await?;
        Ok(result)
    }

    /// Delete a cached value
    pub async fn delete(&self, key: &str) -> Result<()> {
        let mut conn = self.conn().await?;
        let _: () = conn.del(key).await?;
        Ok(())
    }

    // ============ Session Management ============

    /// Store a session token
    pub async fn store_session(&self, token: &str, user_id: &str, ttl_seconds: i64) -> Result<()> {
        let mut conn = self.conn().await?;
        let key = format!("session:{}", token);
        let _: () = conn.set_ex(&key, user_id, ttl_seconds as u64).await?;
        Ok(())
    }

    /// Get user_id from session token
    pub async fn get_session(&self, token: &str) -> Result<Option<String>> {
        let mut conn = self.conn().await?;
        let key = format!("session:{}", token);
        let result: Option<String> = conn.get(&key).await?;
        Ok(result)
    }

    /// Delete a session
    pub async fn delete_session(&self, token: &str) -> Result<()> {
        let mut conn = self.conn().await?;
        let key = format!("session:{}", token);
        let _: () = conn.del(&key).await?;
        Ok(())
    }

    // ============ Model Cache ============

    /// Cache model list
    pub async fn cache_models(&self, models_json: &str) -> Result<()> {
        let mut conn = self.conn().await?;
        let _: () = conn.set_ex("models:all", models_json, 3600).await?;
        Ok(())
    }

    /// Get cached model list
    pub async fn get_cached_models(&self) -> Result<Option<String>> {
        let mut conn = self.conn().await?;
        let result: Option<String> = conn.get("models:all").await?;
        Ok(result)
    }

    // ============ SMS Code Verification ============

    /// Store SMS verification code with TTL (5 minutes)
    pub async fn store_sms_code(&self, phone: &str, code: &str) -> Result<()> {
        let mut conn = self.conn().await?;
        let key = format!("sms_code:{}", phone);
        let _: () = conn.set_ex(&key, code, 300).await?; // 5 minutes TTL
        Ok(())
    }

    /// Get SMS verification code
    pub async fn get_sms_code(&self, phone: &str) -> Result<Option<String>> {
        let mut conn = self.conn().await?;
        let key = format!("sms_code:{}", phone);
        let result: Option<String> = conn.get(&key).await?;
        Ok(result)
    }

    /// Delete SMS verification code (after successful verification)
    pub async fn delete_sms_code(&self, phone: &str) -> Result<()> {
        let mut conn = self.conn().await?;
        let key = format!("sms_code:{}", phone);
        let _: () = conn.del(&key).await?;
        Ok(())
    }

    /// Check rate limit for SMS sending (max 5 per hour per phone)
    pub async fn check_sms_rate_limit(&self, phone: &str) -> Result<(bool, i64)> {
        let mut conn = self.conn().await?;
        let key = format!("sms_rate:{}", phone);
        let now = chrono::Utc::now().timestamp();
        let window_start = now - 3600; // 1 hour window

        // Remove old entries
        let _: () = conn.zrembyscore(&key, 0, window_start).await?;

        // Count current SMS sent
        let count: i64 = conn.zcard(&key).await?;

        if count >= 5 {
            return Ok((false, 3600 - (now - window_start)));
        }

        // Add new entry
        let _: () = conn.zadd(&key, format!("{}", now), now).await?;
        let _: () = conn.expire(&key, 3600).await?;

        Ok((true, 0))
    }
}
