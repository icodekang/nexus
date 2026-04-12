use std::sync::Arc;
use std::net::SocketAddr;
use axum::{Router, middleware::from_fn, Extension};
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use api::{routes, middleware, state::AppState};

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
        &std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://nexus:password@localhost:5432/nexus".to_string())
    ).await?;

    let redis = db::RedisPool::new(
        &std::env::var("REDIS_URL")
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

    // Debug logging middleware
    async fn debug_logger(req: axum::extract::Request, next: axum::middleware::Next) -> axum::response::Response {
        tracing::info!("{} {} - Headers: {:?}", req.method(), req.uri(), req.headers());
        let response = next.run(req).await;
        tracing::info!("Response status: {}", response.status());
        response
    }

    // Public routes (no auth required)
    let public_routes = Router::new()
        .nest("/v1/auth", routes::auth::routes())
        .route("/health", axum::routing::get(routes::health));

    // Protected routes — accepts both API keys and JWT tokens
    let protected_routes = Router::new()
        .nest("/v1", routes::v1::routes())
        .nest("/v1/me", routes::me::routes())
        .layer(from_fn(middleware::auth::validate_jwt_or_api_key));

    // Admin routes — requires JWT + admin role
    let admin_routes = Router::new()
        .nest("/admin", routes::admin::routes())
        .layer(from_fn(middleware::auth::require_admin))
        .layer(from_fn(middleware::auth::validate_jwt_or_api_key));

    let app = Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .merge(admin_routes)
        .layer(from_fn(debug_logger))
        .layer(Extension(state.clone()))
        .layer(cors)
        .with_state(state);

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    tracing::info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
