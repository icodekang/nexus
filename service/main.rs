use std::net::SocketAddr;
use std::sync::Arc;

use axum::{middleware::from_fn, Router};
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use api::{middleware, routes, state::AppState};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "api=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting Nexus API Gateway");

    let db = db::PostgresPool::new(
        &std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://nexus:password@localhost:5432/nexus".to_string()),
    )
    .await?;

    let redis = db::RedisPool::new(
        &std::env::var("REDIS_URL")
            .unwrap_or_else(|_| "redis://localhost:6379".to_string()),
    )
    .await?;

    tracing::info!("Database connections established");

    let state = Arc::new(AppState::new(db, redis));

    if let Err(e) = state.init_key_scheduler().await {
        tracing::warn!(
            "Failed to initialize key scheduler: {}. Using fallback mode.",
            e
        );
    }

    if let Err(e) = state.init_account_pool().await {
        tracing::warn!(
            "Failed to initialize account pool: {}. Browser accounts may not work.",
            e
        );
    }

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let auth_mw = from_fn(middleware::auth::validate_jwt_or_api_key);
    let admin_mw = from_fn(middleware::auth::require_admin);

    // ── Public routes (no auth) ─────────────────────────────────────────────
    let public = Router::new()
        .route("/health", axum::routing::get(routes::health))
        .nest("/v1/auth", routes::auth::routes());

    // ── All authenticated /v1/* endpoints ────────────────────────────────
    // Routes:
    //   /v1/chat/completions       — internal unified chat
    //   /v1/completions            — text completions
    //   /v1/embeddings             — embeddings
    //   /v1/models                 — model list
    //   /v1/openai/chat/completions  — OpenAI SDK compatible
    //   /v1/openai/models             — OpenAI SDK compatible
    //   /v1/anthropic/messages       — Anthropic SDK compatible
    //   /v1/anthropic/models          — Anthropic SDK compatible
    let v1 = routes::v1::routes().layer(auth_mw.clone());

    // ── Authenticated management routes ─────────────────────────────────
    let me = Router::new()
        .nest("/me", routes::me::routes())
        .layer(auth_mw.clone());

    // ── Admin routes ────────────────────────────────────────────────────
    let admin = Router::new()
        .nest("/admin", routes::admin::routes())
        .layer(admin_mw)
        .layer(auth_mw.clone());

    // Build app with all routes merged, then attach state ONCE at the end.
    // `.with_state()` converts the router to `Router<()>` so `into_make_service()` works.
    let app = Router::new()
        .merge(public)
        .merge(v1)
        .merge(me)
        .merge(admin)
        .layer(cors)
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    tracing::info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}
