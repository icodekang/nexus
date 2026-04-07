use std::sync::Arc;
use db::{PostgresPool, RedisPool};

/// Application state shared across all handlers
#[derive(Clone)]
pub struct AppState {
    pub db: Arc<PostgresPool>,
    pub redis: Arc<RedisPool>,
}

impl AppState {
    pub fn new(db: PostgresPool, redis: RedisPool) -> Self {
        Self {
            db: Arc::new(db),
            redis: Arc::new(redis),
        }
    }
}
