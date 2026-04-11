mod error;
mod routes;
mod middleware;
mod state;

use std::sync::Arc;
use axum::{Router, middleware::from_fn};
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer, AllowHeaders, AllowMethods};
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

    tracing::info!("Starting Nexus API Gateway");

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

    // Public routes (no auth required)
    let public_routes = Router::new()
        .nest("/v1/auth", routes::auth::routes())
        .route("/health", axum::routing::get(routes::health));

    // Protected routes (auth middleware applied)
    let protected_routes = Router::new()
        .nest("/v1", routes::v1::routes())
        .nest("/v1/me", routes::me::routes())
        .layer(from_fn(middleware::auth::validate_api_key));

    // State injection middleware — puts Arc<AppState> into request extensions
    // so the auth middleware can access it
    let app = Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .layer(axum::middleware::from_fn(inject_state))
        .layer(cors)
        .with_state(state);

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    tracing::info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// Middleware to inject AppState into request extensions
async fn inject_state(
    mut req: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    let state = req.extensions().get::<Arc<AppState>>().cloned();
    if let Some(state) = state {
        req.extensions_mut().insert(state);
    }
    next.run(req).await
}
