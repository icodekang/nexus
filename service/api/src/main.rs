mod error;
mod routes;
mod middleware;
mod state;

use std::sync::Arc;
use axum::{Router};
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "api=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting nexus API Gateway");

    // Initialize database connections
    let db = db::PostgresPool::new(
        std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://nexus:password@localhost:5432/nexus".to_string())
    ).await?;

    let redis = db::RedisPool::new(
        std::env::var("REDIS_URL")
            .unwrap_or_else(|_| "redis://localhost:6379".to_string())
    ).await?;

    tracing::info!("Database connections established");

    // Create application state
    let state = Arc::new(AppState::new(db, redis));

    // CORS layer
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Build application
    let app = Router::new()
        .nest("/v1", routes::v1::routes())
        .nest("/v1/auth", routes::auth::routes())
        .nest("/v1/me", routes::me::routes())
        .route("/health", axum::routing::get(routes::health))
        .layer(cors)
        .with_state(state);

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    tracing::info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
