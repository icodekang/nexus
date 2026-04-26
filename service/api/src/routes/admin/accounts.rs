//! 浏览器账户管理路由模块
//! 处理浏览器账户的 CRUD 操作和无头浏览器登录
//!
//! 路由列表：
//! - GET /accounts - 列出所有浏览器账户
//! - POST /accounts - 创建新的浏览器账户
//! - DELETE /accounts/:id - 删除浏览器账户
//! - POST /accounts/:id/start-login - 启动无头浏览器登录
//! - GET /accounts/:id/login-url - 获取登录页面 URL（二维码）
//! - GET /accounts/:id/status - 获取账户状态（SSE 流）
//! - POST /accounts/:id/complete - 完成登录（手动触发）

use axum::{
    routing::{get, post, delete},
    Router, Json, Extension,
    extract::{Path, State, Query},
    response::sse::{Event, Sse},
};
use std::sync::Arc;
use futures_util::{Stream, StreamExt};
use tokio_stream::wrappers::BroadcastStream;
use tokio::sync::broadcast;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::state::AppState;
use crate::error::ApiError;
use crate::middleware::auth::AuthContext;
use models::BrowserAccount;
use provider_client::headless_browser::{LoginEvent, BrowserSessionState};

/// 创建浏览器账户路由
pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/accounts", get(list_accounts).post(create_account))
        .route("/accounts/:id", delete(delete_account))
        .route("/accounts/:id/start-login", post(start_login))
        .route("/accounts/:id/password-login", post(password_login))
        .route("/accounts/:id/login-url", get(get_login_url))
        .route("/accounts/:id/status", get(get_account_status))
        .route("/accounts/complete-login", post(complete_login))
}

// ============ 浏览器账户 CRUD ============

/// 账户响应体
#[derive(Debug, Serialize)]
struct AccountResponse {
    id: String,
    provider: String,
    email: Option<String>,
    status: String,
    request_count: i64,
    last_used_at: Option<String>,
    created_at: String,
}

/// 从 BrowserAccount 转换为 AccountResponse
impl From<BrowserAccount> for AccountResponse {
    fn from(acc: BrowserAccount) -> Self {
        Self {
            id: acc.id.to_string(),
            provider: acc.provider,
            email: acc.email,
            status: acc.status.as_str().to_string(),
            request_count: acc.request_count,
            last_used_at: acc.last_used_at.map(|t| t.to_rfc3339()),
            created_at: acc.created_at.to_rfc3339(),
        }
    }
}

async fn list_accounts(
    State(state): State<Arc<AppState>>,
    Extension(_auth): Extension<AuthContext>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let accounts = state.db.list_browser_accounts().await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to list accounts: {}", e)))?;

    let data: Vec<AccountResponse> = accounts.into_iter().map(Into::into).collect();

    Ok(Json(serde_json::json!({ "data": data })))
}

/// 创建账户请求体
#[derive(Debug, Deserialize)]
struct CreateAccountRequest {
    /// Provider 名称（deepseek、claude 或 chatgpt）
    provider: String,
}

/// POST /admin/accounts
///
/// 创建新的浏览器账户
///
/// # 说明
/// 目前支持的 Provider：deepseek、claude、chatgpt
async fn create_account(
    State(state): State<Arc<AppState>>,
    Extension(_auth): Extension<AuthContext>,
    Json(body): Json<CreateAccountRequest>,
) -> Result<Json<AccountResponse>, ApiError> {
    let provider = body.provider.to_lowercase();
    if provider != "deepseek" && provider != "claude" && provider != "chatgpt" {
        return Err(ApiError::InvalidRequest(
            "Provider must be 'deepseek', 'claude' or 'chatgpt'".to_string()
        ));
    }

    let account = BrowserAccount::new(provider);

    state.db.create_browser_account(&account).await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to create account: {}", e)))?;

    Ok(Json(account.into()))
}

/// DELETE /admin/accounts/:id
///
/// 删除浏览器账户
///
/// # 说明
/// 同时会清理 Redis 中的会话数据，并关闭无头浏览器会话
async fn delete_account(
    State(state): State<Arc<AppState>>,
    Extension(_auth): Extension<AuthContext>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let account_id = Uuid::parse_str(&id)
        .map_err(|_| ApiError::InvalidRequest("Invalid account ID".to_string()))?;

    // Close headless browser session if exists
    let _ = state.headless_browser.close_session(account_id).await;

    // Delete from database
    state.db.delete_browser_account(account_id).await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to delete account: {}", e)))?;

    // Clean up Redis session
    let _ = state.redis.delete_account_session(&id).await;

    Ok(Json(serde_json::json!({ "deleted": true })))
}

// ============ 无头浏览器登录 ============

/// 启动登录请求体
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct StartLoginRequest {
    /// 是否使用无头浏览器（默认 true）
    #[serde(default = "default_true")]
    use_headless: bool,
}

#[derive(Debug, Deserialize)]
struct PasswordLoginRequest {
    email: String,
    password: String,
}

#[derive(Debug, Serialize)]
struct PasswordLoginResponse {
    success: bool,
    message: String,
}

fn default_true() -> bool {
    true
}

/// 登录 URL 响应体
#[derive(Debug, Serialize)]
struct LoginUrlResponse {
    /// 账户 ID
    account_id: String,
    /// 登录页面 URL（用于生成二维码）
    login_url: String,
    /// 简短验证码（6位数字）
    code: Option<String>,
    /// 过期时间
    expires_at: Option<String>,
    /// 是否正在等待登录
    waiting: bool,
}

/// POST /admin/accounts/:id/start-login
///
/// 启动无头浏览器登录流程
///
/// # 说明
/// 1. 启动无头 Chrome 浏览器
/// 2. 打开目标网站的登录页面
/// 3. 返回登录 URL 用于生成二维码
/// 4. 前端通过 SSE 监听登录状态变化
async fn start_login(
    State(state): State<Arc<AppState>>,
    Extension(_auth): Extension<AuthContext>,
    Path(id): Path<String>,
    Json(_body): Json<StartLoginRequest>,
) -> Result<Json<LoginUrlResponse>, ApiError> {
    let account_id = Uuid::parse_str(&id)
        .map_err(|_| ApiError::InvalidRequest("Invalid account ID".to_string()))?;

    // Get account from database
    let account = state.db.get_browser_account(account_id).await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Database error: {}", e)))?
        .ok_or(ApiError::InvalidRequest("Account not found".to_string()))?;

    // Create headless browser login session
    let session = state.headless_browser.create_login_session(&account.provider).await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to start login: {}", e)))?;

    // Generate a short code for verification (optional, for simpler auth)
    let code = format!("{:06}", rand::random::<u32>() % 900000 + 100000);
    let expires_at = chrono::Utc::now() + chrono::Duration::minutes(10);

    // Store QR session in Redis for optional code verification
    state.redis.store_qr_session(&code, &session.id.to_string()).await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Redis error: {}", e)))?;

    Ok(Json(LoginUrlResponse {
        account_id: id,
        login_url: session.current_url,
        code: Some(code),
        expires_at: Some(expires_at.to_rfc3339()),
        waiting: true,
    }))
}

/// POST /admin/accounts/:id/password-login
///
/// 使用账号密码登录（无头浏览器填表）
async fn password_login(
    State(state): State<Arc<AppState>>,
    Extension(_auth): Extension<AuthContext>,
    Path(id): Path<String>,
    Json(body): Json<PasswordLoginRequest>,
) -> Result<Json<PasswordLoginResponse>, ApiError> {
    let account_id = Uuid::parse_str(&id)
        .map_err(|_| ApiError::InvalidRequest("Invalid account ID".to_string()))?;

    // Get account from database
    let account = state.db.get_browser_account(account_id).await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Database error: {}", e)))?
        .ok_or(ApiError::InvalidRequest("Account not found".to_string()))?;

    // Use headless browser to login with password
    let cookies_json = state.headless_browser.login_with_password(
        &account.provider,
        &body.email,
        &body.password,
    ).await
    .map_err(|e| ApiError::Internal(anyhow::anyhow!("Login failed: {}", e)))?;

    // Parse cookies into PersistedSession
    let session_data: provider_client::PersistedSession =
        serde_json::from_str(&cookies_json)
            .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to parse session: {}", e)))?;

    // Update database with session data and set status to active
    state.db.update_browser_account_session(
        account_id,
        &cookies_json,
        Some(&body.email),
    ).await
    .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to update session: {}", e)))?;

    // Register account in pool
    state.account_pool.register_account(
        account_id,
        account.provider.clone(),
        session_data,
    ).await
    .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to register account: {}", e)))?;

    Ok(Json(PasswordLoginResponse {
        success: true,
        message: "Login successful".to_string(),
    }))
}

/// GET /admin/accounts/:id/login-url
///
/// 获取当前登录页面 URL
///
/// # 说明
/// 用于轮询获取当前 URL（无头浏览器可能重定向）
async fn get_login_url(
    State(state): State<Arc<AppState>>,
    Extension(_auth): Extension<AuthContext>,
    Path(id): Path<String>,
) -> Result<Json<LoginUrlResponse>, ApiError> {
    let account_id = Uuid::parse_str(&id)
        .map_err(|_| ApiError::InvalidRequest("Invalid account ID".to_string()))?;

    // Refresh session state
    let session = state.headless_browser.refresh_session(account_id).await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to refresh session: {}", e)))?;

    let is_waiting = session.state == BrowserSessionState::WaitingForLogin;

    Ok(Json(LoginUrlResponse {
        account_id: id,
        login_url: session.current_url,
        code: None,
        expires_at: None,
        waiting: is_waiting,
    }))
}

/// GET /admin/accounts/:id/status
///
/// 通过 SSE 流获取登录状态变化
///
/// # 说明
/// 使用 Server-Sent Events 实时推送登录状态
/// 同时启动后台任务监控登录完成
async fn get_account_status(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Sse<impl Stream<Item = Result<Event, std::convert::Infallible>> + Send>, ApiError> {
    let account_id = Uuid::parse_str(&id)
        .map_err(|_| ApiError::InvalidRequest("Invalid account ID".to_string()))?;

    // Validate token from query param for SSE (EventSource doesn't support headers)
    if let Some(token) = params.get("token") {
        let claims = auth::JwtService::validate_token(token)
            .map_err(|_| ApiError::Unauthorized)?;

        let user_id = claims.user_id()
            .map_err(|_| ApiError::Unauthorized)?;

        // Verify user is admin
        let user = state.db.get_user_by_id(user_id).await
            .map_err(|_| ApiError::Unauthorized)?
            .ok_or(ApiError::Unauthorized)?;

        if !user.is_admin {
            return Err(ApiError::Forbidden);
        }
    } else {
        return Err(ApiError::Unauthorized);
    }

    // Create broadcast channel for this client
    let (tx, rx) = broadcast::channel::<Result<Event, std::convert::Infallible>>(100);

    // Subscribe to headless browser events
    let mut event_rx = state.headless_browser.subscribe();

    // Spawn task to monitor login status
    let state_clone = state.clone();
    tokio::spawn(async move {
        let mut last_status: Option<String> = None;
        let mut last_url: Option<String> = None;

        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

            // Refresh session state
            match state_clone.headless_browser.refresh_session(account_id).await {
                Ok(session) => {
                    let current_status = format!("{:?}", session.state);
                    let current_url = session.current_url.clone();

                    // Check if status changed
                    if last_status.as_ref() != Some(&current_status) {
                        last_status = Some(current_status.clone());

                        // Build event data based on state
                        let (event_type, event_data) = match session.state {
                            BrowserSessionState::LoggedIn => {
                                // Login successful - capture cookies and update account
                                let cookies_json = session.cookies_json.clone();
                                // Extract email from cookies_json if possible
                                let email: Option<String> = None; // Simplified - actual email extraction would require parsing

                                // Update account in database
                                let _ = state_clone.db.update_browser_account_session(
                                    account_id,
                                    &cookies_json,
                                    email.as_deref(),
                                ).await;

                                // Store session in Redis
                                let _ = state_clone.redis.store_account_session(
                                    &account_id.to_string(),
                                    &cookies_json,
                                ).await;

                                // Register in account pool
                                use provider_client::PersistedSession;
                                if let Ok(session_data) = serde_json::from_str::<PersistedSession>(&cookies_json) {
                                    let _ = state_clone.account_pool.register_account(
                                        account_id,
                                        session.provider.clone(),
                                        session_data,
                                    ).await;
                                }

                                ("status", serde_json::json!({
                                    "status": "active",
                                    "email": email,
                                    "message": "Login successful!"
                                }))
                            }
                            BrowserSessionState::Failed => {
                                ("status", serde_json::json!({
                                    "status": "error",
                                    "message": "Login failed or timeout"
                                }))
                            }
                            BrowserSessionState::Closed => {
                                ("status", serde_json::json!({
                                    "status": "closed",
                                    "message": "Session closed"
                                }))
                            }
                            BrowserSessionState::WaitingForLogin => {
                                ("status", serde_json::json!({
                                    "status": "waiting",
                                    "message": "Waiting for login..."
                                }))
                            }
                        };

                        let event = Event::default()
                            .event(event_type)
                            .data(event_data.to_string());

                        let _ = tx.send(Ok(event));

                        // Stop if terminal state
                        if session.state == BrowserSessionState::LoggedIn
                            || session.state == BrowserSessionState::Failed
                            || session.state == BrowserSessionState::Closed
                        {
                            break;
                        }
                    }

                    // Check if URL changed (user navigated to different page)
                    if last_url.as_ref() != Some(&current_url) {
                        last_url = Some(current_url.clone());

                        let event = Event::default()
                            .event("url")
                            .data(serde_json::json!({
                                "url": current_url
                            }).to_string());

                        let _ = tx.send(Ok(event));
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to refresh session {}: {}", account_id, e);
                }
            }

            // Also check for headless browser events
            while let Ok(event) = event_rx.try_recv() {
                match event {
                    LoginEvent::StateChanged { session_id, new_state, cookies_json, .. } if session_id == account_id => {
                        let (event_type, event_data) = match new_state {
                            BrowserSessionState::LoggedIn => {
                                ("status", serde_json::json!({
                                    "status": "active",
                                    "cookies": cookies_json,
                                    "message": "Login successful!"
                                }))
                            }
                            BrowserSessionState::Failed => {
                                ("status", serde_json::json!({
                                    "status": "error",
                                    "message": "Login failed"
                                }))
                            }
                            _ => continue,
                        };

                        let event = Event::default()
                            .event(event_type)
                            .data(event_data.to_string());

                        let _ = tx.send(Ok(event));

                        if new_state == BrowserSessionState::LoggedIn || new_state == BrowserSessionState::Failed {
                            break;
                        }
                    }
                    LoginEvent::UrlChanged { session_id, url } if session_id == account_id => {
                        let event = Event::default()
                            .event("url")
                            .data(serde_json::json!({ "url": url }).to_string());
                        let _ = tx.send(Ok(event));
                    }
                    LoginEvent::Error { session_id, error } if session_id == account_id => {
                        let event = Event::default()
                            .event("error")
                            .data(serde_json::json!({ "error": error }).to_string());
                        let _ = tx.send(Ok(event));
                        break;
                    }
                    _ => {}
                }
            }
        }
    });

    let stream = BroadcastStream::new(rx).filter_map(|item| async {
        match item {
            Ok(Ok(event)) => Some(Ok::<_, std::convert::Infallible>(event)),
            _ => None,
        }
    });

    Ok(Sse::new(stream))
}

/// 完成登录请求体（用于手动触发或移动端回调）
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct CompleteLoginRequest {
    /// 验证码（可选）
    code: Option<String>,
    /// 加密的会话数据（可选，从 URL 参数提取）
    session_data: Option<String>,
    /// 账户邮箱（可选）
    email: Option<String>,
}

/// POST /admin/accounts/complete-login
///
/// 完成浏览器账户登录（手动触发或移动端回调）
///
/// # 说明
/// 当使用无头浏览器自动捕获时，通常不需要调用此接口
/// 此接口主要用于：
/// 1. 移动端扫码登录后的回调
/// 2. 手动保存登录信息
async fn complete_login(
    State(state): State<Arc<AppState>>,
    Json(body): Json<CompleteLoginRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // Verify QR session if code is provided
    if let Some(code) = &body.code {
        let _qr_session_id = state.redis.get_qr_session(code).await
            .map_err(|e| ApiError::Internal(anyhow::anyhow!("Redis error: {}", e)))?
            .ok_or(ApiError::InvalidRequest("Invalid or expired code".to_string()))?;
    }

    // If session_data is provided directly, use it
    if body.session_data.is_some() {
        return Err(ApiError::InvalidRequest(
            "session_data requires account_id in query params".to_string()
        ));
    }

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Login completion processed"
    })))
}
