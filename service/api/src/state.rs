use std::sync::Arc;
use db::{PostgresPool, RedisPool};
use router::GlobalKeyScheduler;
use tokio::sync::RwLock;
use provider_client::AccountPool;

/// Application state shared across all handlers
#[derive(Clone)]
pub struct AppState {
    pub db: Arc<PostgresPool>,
    pub redis: Arc<RedisPool>,
    pub key_scheduler: Arc<RwLock<GlobalKeyScheduler>>,
    pub account_pool: Arc<AccountPool>,
}

impl AppState {
    pub fn new(db: PostgresPool, redis: RedisPool) -> Self {
        Self {
            db: Arc::new(db),
            redis: Arc::new(redis),
            key_scheduler: Arc::new(RwLock::new(GlobalKeyScheduler::new())),
            account_pool: Arc::new(AccountPool::new()),
        }
    }

    /// Initialize the key scheduler with keys from the database
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

    /// Initialize the account pool with browser accounts from the database
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
