//! 应用状态模块
//! 定义在所有 handler 之间共享的应用状态
//!
//! 状态包含：
//! - 数据库连接池
//! - Redis 连接池
//! - 全局 Key 调度器

use db::{PostgresPool, RedisPool};
use router::GlobalKeyScheduler;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 应用状态
///
/// 在所有 handler 之间共享的状态，包含数据库连接、Redis 连接、
/// Key 调度器和账户池。
///
/// # 字段说明
/// - `db`: PostgreSQL 数据库连接池
/// - `redis`: Redis 连接池
/// - `key_scheduler`: 全局 Key 调度器，用于 API Key 的负载均衡
#[derive(Clone)]
pub struct AppState {
    /// PostgreSQL 数据库连接池
    pub db: Arc<PostgresPool>,
    /// Redis 连接池
    pub redis: Arc<RedisPool>,
    /// 全局 Key 调度器
    pub key_scheduler: Arc<RwLock<GlobalKeyScheduler>>,
}

impl AppState {
    /// 创建新的应用状态
    ///
    /// # 参数
    /// * `db` - PostgreSQL 数据库连接池
    /// * `redis` - Redis 连接池
    pub fn new(db: PostgresPool, redis: RedisPool) -> Self {
        Self {
            db: Arc::new(db),
            redis: Arc::new(redis),
            key_scheduler: Arc::new(RwLock::new(GlobalKeyScheduler::new())),
        }
    }

    /// 初始化 Key 调度器
    ///
    /// 从数据库加载 Provider Keys 并初始化调度器
    ///
    /// # 返回
    /// 成功返回 Ok(()), 失败返回错误信息
    pub async fn init_key_scheduler(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let keys = self.db.list_provider_keys().await?;

        let mut keys_by_provider: std::collections::HashMap<String, Vec<_>> =
            std::collections::HashMap::new();
        for key in keys {
            keys_by_provider
                .entry(key.provider_slug.clone())
                .or_default()
                .push(key);
        }

        let mut scheduler = self.key_scheduler.write().await;
        for (provider, provider_keys) in keys_by_provider {
            scheduler.set_provider_keys(&provider, provider_keys);
        }

        tracing::info!("Key scheduler initialized");
        Ok(())
    }
}
