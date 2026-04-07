mod error;
mod routes;
mod middleware;

use std::sync::Arc;
use axum::{Router, middleware as axum_middleware};
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "api_gateway=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting NovaChat API Gateway");

    // CORS layer
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Router context
    let ctx = Arc::new(routes::v1::RouterContext);

    // Build application
    let app = Router::new()
        .nest("/v1", routes::v1::routes())
        .nest("/v1/auth", routes::auth::routes())
        .nest("/v1/me", routes::me::routes())
        .route("/health", axum::routing::get(routes::health))
        .layer(cors)
        .with_state(ctx);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    tracing::info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
