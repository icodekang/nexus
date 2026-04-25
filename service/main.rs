//! Nexus API Gateway 主入口
//!
//! 提供统一的 LLM API 网关服务，支持：
//! - OpenAI 兼容接口
//! - Anthropic 兼容接口
//! - 统一的 Chat 接口
//! - 用户管理、订阅管理
//! - API Key 管理
//! - 管理后台

use std::net::SocketAddr;
use std::sync::Arc;

use axum::{middleware::from_fn, Router};
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use api::{middleware, routes, state::AppState};

/// 主函数
///
/// # 环境变量
/// - DATABASE_URL: PostgreSQL 数据库连接 URL
/// - REDIS_URL: Redis 连接 URL
/// - NEXUS_BASE_URL: 服务基础 URL（用于生成认证回调 URL）
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 初始化日志
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "api=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting Nexus API Gateway");

    // 初始化数据库连接
    let db = db::PostgresPool::new(
        &std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://nexus:password@localhost:5432/nexus".to_string()),
    )
    .await?;

    // 执行数据库迁移
    let migrations_path = std::env::var("MIGRATIONS_PATH")
        .unwrap_or_else(|_| "./db/migrations".to_string());
    if let Err(e) = db::run_migrations(db.pool(), &migrations_path).await {
        tracing::warn!("Failed to run migrations: {}. Continuing...", e);
    }

    // 初始化 Redis 连接
    let redis = db::RedisPool::new(
        &std::env::var("REDIS_URL")
            .unwrap_or_else(|_| "redis://localhost:6379".to_string()),
    )
    .await?;

    tracing::info!("Database connections established");

    // 创建应用状态
    let state = Arc::new(AppState::new(db, redis));

    // 初始化 Key 调度器（从数据库加载 Provider Keys）
    if let Err(e) = state.init_key_scheduler().await {
        tracing::warn!(
            "Failed to initialize key scheduler: {}. Using fallback mode.",
            e
        );
    }

    // 初始化账户池（加载浏览器账户）
    if let Err(e) = state.init_account_pool().await {
        tracing::warn!(
            "Failed to initialize account pool: {}. Browser accounts may not work.",
            e
        );
    }

    // 配置 CORS - 只允许指定的域名
    let allowed_origins: Vec<axum::http::HeaderValue> = std::env::var("CORS_ALLOWED_ORIGINS")
        .map(|s| {
            s.split(',')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .map(|s| axum::http::HeaderValue::from_str(s).expect("Invalid CORS origin"))
                .collect()
        })
        .unwrap_or_else(|_| {
            vec![
                axum::http::HeaderValue::from_str("http://localhost:3000").unwrap(),
                axum::http::HeaderValue::from_str("http://localhost:3001").unwrap(),
            ]
        });

    tracing::info!("CORS configured with {} allowed origins", allowed_origins.len());

    let cors = CorsLayer::new()
        .allow_origin(tower_http::cors::AllowOrigin::list(allowed_origins))
        .allow_methods(Any)
        .allow_headers(Any);

    // 认证中间件
    let auth_mw = from_fn(middleware::auth::validate_jwt_or_api_key);
    // 管理员权限中间件
    let admin_mw = from_fn(middleware::auth::require_admin);

    // ── 公开路由（无需认证）────────────────────────────────────────────
    let public = Router::new()
        .route("/health", axum::routing::get(routes::health))
        .nest("/v1/auth", routes::auth::routes());

    // ── 需要认证的 /v1/* 端点 ────────────────────────────────────────────
    // 路由说明：
    //   /v1/chat/completions       — 统一聊天接口
    //   /v1/completions            — 文本补全（未实现）
    //   /v1/embeddings             — 向量嵌入
    //   /v1/models                 — 模型列表
    //   /v1/openai/chat/completions  — OpenAI SDK 兼容
    //   /v1/openai/models             — OpenAI SDK 兼容
    //   /v1/anthropic/messages       — Anthropic SDK 兼容
    //   /v1/anthropic/models          — Anthropic SDK 兼容
    let v1 = routes::v1::routes().layer(auth_mw.clone());

    // ── 需要认证的用户管理路由 ─────────────────────────────────────────
    let me = Router::new()
        .nest("/me", routes::me::routes())
        .layer(auth_mw.clone());

    // ── 管理员路由 ────────────────────────────────────────────────────
    // 注意：auth_mw 必须在 admin_mw 之前执行（先添加的 layer 先执行）
    let admin = Router::new()
        .nest("/admin", routes::admin::routes())
        .layer(admin_mw)
        .layer(auth_mw.clone());

    // 合并所有路由并附加状态
    // `.with_state()` 将 router 转换为 `Router<()>` 以支持 `into_make_service()`
    // `Extension(state)` 让 from_fn 中间件能通过 req.extensions() 获取 AppState
    let app = Router::new()
        .merge(public)
        .merge(v1)
        .merge(me)
        .merge(admin)
        .layer(cors)
        .layer(axum::Extension(state.clone()))
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    tracing::info!("Listening on {}", addr);

    // 启动服务
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}
