//! 应用状态模块
//! 定义在所有 handler 之间共享的应用状态
//!
//! 状态包含：
//! - 数据库连接池
//! - Redis 连接池
//! - 全局 Key 调度器
//! - 账户池（用于 ZeroToken 用户）

use std::sync::Arc;
use db::{PostgresPool, RedisPool};
use router::GlobalKeyScheduler;
use tokio::sync::RwLock;
use provider_client::AccountPool;

/// 应用状态
///
/// 在所有 handler 之间共享的状态，包含数据库连接、Redis 连接、
/// Key 调度器和账户池。
///
/// # 字段说明
/// - `db`: PostgreSQL 数据库连接池
/// - `redis`: Redis 连接池
/// - `key_scheduler`: 全局 Key 调度器，用于 API Key 的负载均衡
/// - `account_pool`: 浏览器账户池（ZeroToken 用户使用）
#[derive(Clone)]
pub struct AppState {
    /// PostgreSQL 数据库连接池
    pub db: Arc<PostgresPool>,
    /// Redis 连接池
    pub redis: Arc<RedisPool>,
    /// 全局 Key 调度器
    pub key_scheduler: Arc<RwLock<GlobalKeyScheduler>>,
    /// 浏览器账户池（ZeroToken 用户使用）
    pub account_pool: Arc<AccountPool>,
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
            account_pool: Arc::new(AccountPool::new()),
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

        // Group keys by provider
        let mut keys_by_provider: std::collections::HashMap<String, Vec<_>> = std::collections::HashMap::new();
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

    /// 初始化账户池
    ///
    /// 从数据库加载浏览器账户并注册到账户池中
    /// 只注册状态为 Active 且有会话数据的账户
    ///
    /// # 返回
    /// 成功返回 Ok(()), 失败返回错误信息
    pub async fn init_account_pool(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use provider_client::PersistedSession;
        use models::BrowserAccountStatus;
        use uuid::Uuid;

        let accounts = self.db.list_browser_accounts().await?;

        for account in accounts {
            // Only register active accounts with session data
            if account.status == BrowserAccountStatus::Active && !account.session_data_encrypted.is_empty() {
                // Parse session data from JSON
                if let Ok(session_data) = serde_json::from_str::<PersistedSession>(&account.session_data_encrypted) {
                    if let Err(e) = self.account_pool.register_account(account.id, account.provider.clone(), session_data).await {
                        tracing::warn!("Failed to register browser account {}: {}", account.id, e);
                    } else {
                        tracing::info!("Registered browser account: {} ({})", account.id, account.provider);
                    }
                }
            }
        }

        tracing::info!("Account pool initialized");
        Ok(())
    }
}
