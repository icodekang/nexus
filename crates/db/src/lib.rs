pub mod postgres;
pub mod redis;

use sqlx::PgPoolOptions;
use deadpool_redis::{Pool as RedisPool, Config as RedisConfig};

/// Initialize PostgreSQL connection pool
pub async fn init_postgres(database_url: &str) -> anyhow::Result<sqlx::PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await?;

    tracing::info!("PostgreSQL connected");
    Ok(pool)
}

/// Initialize Redis connection pool
pub async fn init_redis(redis_url: &str) -> anyhow::Result<RedisPool> {
    let cfg = RedisConfig::from_url(redis_url);
    let pool = cfg.create_pool(Some(deadpool_redis::Runtime::Tokio1))?;

    tracing::info!("Redis connected");
    Ok(pool)
}
