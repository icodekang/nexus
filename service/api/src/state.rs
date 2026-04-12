use std::sync::Arc;
use db::{PostgresPool, RedisPool};
use router::GlobalKeyScheduler;
use tokio::sync::RwLock;

/// Application state shared across all handlers
#[derive(Clone)]
pub struct AppState {
    pub db: Arc<PostgresPool>,
    pub redis: Arc<RedisPool>,
    pub key_scheduler: Arc<RwLock<GlobalKeyScheduler>>,
}

impl AppState {
    pub fn new(db: PostgresPool, redis: RedisPool) -> Self {
        Self {
            db: Arc::new(db),
            redis: Arc::new(redis),
            key_scheduler: Arc::new(RwLock::new(GlobalKeyScheduler::new())),
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
}
