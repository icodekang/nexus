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
        .route("/accounts/:id/inject-session", post(inject_session))
        .route("/accounts/:id/phone-login/init", post(initiate_phone_login))
        .route("/accounts/:id/phone-login/verify", post(complete_phone_login))
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
    name: Option<String>,
    email: Option<String>,
    status: String,
    request_count: i64,
    last_used_at: Option<String>,
    created_at: String,
}

impl From<BrowserAccount> for AccountResponse {
    fn from(acc: BrowserAccount) -> Self {
        Self {
            id: acc.id.to_string(),
            provider: acc.provider,
            name: acc.name,
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
    provider: String,
    #[serde(default)]
    name: Option<String>,
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

    let mut account = BrowserAccount::new(provider);
    if let Some(ref name) = body.name {
        account = account.with_name(name.clone());
    }

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
struct StartLoginRequest {
    /// 是否使用无头浏览器（默认 true）
    #[serde(default = "default_true")]
    #[allow(dead_code)]
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

    let account = state.db.get_browser_account(account_id).await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Database error: {}", e)))?
        .ok_or(ApiError::InvalidRequest("Account not found".to_string()))?;

    let cookies_json = state.headless_browser.login_with_password(
        &account.provider,
        &body.email,
        &body.password,
    ).await
    .map_err(|e| match e {
        provider_client::ProviderError::ChromeNotFound => {
            ApiError::InvalidRequest("Chrome/Chromium browser not installed on server. Use manual session injection instead.".to_string())
        }
        provider_client::ProviderError::BlockedByProvider => {
            ApiError::InvalidRequest("Login page blocked by CAPTCHA / Cloudflare. Use 'Manual Injection' tab: log in via your browser, export Cookie JSON from DevTools, and paste here.".to_string())
        }
        provider_client::ProviderError::AuthenticationError(msg) => {
            ApiError::LoginFailed(msg)
        }
        provider_client::ProviderError::SessionExpired => {
            ApiError::SessionExpired
        }
        provider_client::ProviderError::PageStructureChanged(_) => {
            ApiError::PageStructureChanged
        }
        provider_client::ProviderError::CloudflareChallenge => {
            ApiError::CloudflareChallenge
        }
        provider_client::ProviderError::LoginFailed(msg) => {
            ApiError::LoginFailed(msg)
        }
        _ => {
            tracing::error!("Login failed with unexpected error: {:?}", e);
            ApiError::LoginFailed(format!("{}", e))
        }
    })?;

    let session_data: provider_client::PersistedSession =
        serde_json::from_str(&cookies_json)
            .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to parse session: {}", e)))?;

    state.db.update_browser_account_session(
        account_id,
        &cookies_json,
        Some(&body.email),
        None,
    ).await
    .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to update session: {}", e)))?;

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

/// 手动注入会话请求体
#[derive(Debug, Deserialize)]
struct InjectSessionRequest {
    email: Option<String>,
    name: Option<String>,
    cookies_json: String,
}

/// POST /admin/accounts/:id/inject-session
///
/// 手动注入浏览器会话（Cookie/Token）
///
/// # 使用场景
/// 当自动化无头浏览器登录被反爬机制阻挡时，管理员可以：
/// 1. 在本地普通浏览器中手动登录目标平台
/// 2. 使用浏览器开发者工具导出 Cookie JSON
/// 3. 通过此接口注入 Cookie 到平台
async fn inject_session(
    State(state): State<Arc<AppState>>,
    Extension(_auth): Extension<AuthContext>,
    Path(id): Path<String>,
    Json(body): Json<InjectSessionRequest>,
) -> Result<Json<PasswordLoginResponse>, ApiError> {
    let account_id = Uuid::parse_str(&id)
        .map_err(|_| ApiError::InvalidRequest("Invalid account ID".to_string()))?;

    let account = state.db.get_browser_account(account_id).await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Database error: {}", e)))?
        .ok_or(ApiError::InvalidRequest("Account not found".to_string()))?;

    let session_data: provider_client::PersistedSession =
        serde_json::from_str(&body.cookies_json)
            .map_err(|e| ApiError::InvalidRequest(format!("Invalid cookies JSON: {}", e)))?;

    state.db.update_browser_account_session(
        account_id,
        &body.cookies_json,
        body.email.as_deref(),
        body.name.as_deref(),
    ).await
    .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to update session: {}", e)))?;

    state.account_pool.register_account(
        account_id,
        account.provider.clone(),
        session_data.clone(),
    ).await
    .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to register account: {}", e)))?;

    if account.provider == "deepseek" {
        match provider_client::headless_browser::HeadlessBrowserManager::detect_chrome_binary() {
            Ok(_) => {
                match headless_chrome::Browser::default() {
                    Ok(browser) => {
                        tracing::info!("Chrome launched successfully for DeepSeek");
                        if let Ok(tab) = browser.new_tab() {
                            if tab.navigate_to("https://chat.deepseek.com/chat").is_ok() {
                                tracing::info!("Navigated to DeepSeek chat page");
                                std::thread::sleep(std::time::Duration::from_secs(3));

                                let cookie_pairs_json = serde_json::to_string(
                                    &session_data.cookies.iter()
                                        .map(|(k, v)| [k.as_str(), v.as_str()])
                                        .collect::<Vec<_>>()
                                ).unwrap_or_else(|_| "[]".to_string());
                                tracing::info!("Injecting {} DeepSeek cookies", session_data.cookies.len());
                                let cookies_script = format!(r#"
                                    (function() {{
                                        const pairs = {cookie_pairs_json};
                                        pairs.forEach(function(pair) {{
                                            const name = pair[0];
                                            const value = pair[1];
                                            document.cookie = name + '=' + value + '; path=/; domain=.deepseek.com';
                                        }});
                                        return document.cookie;
                                    }})();
                                "#, cookie_pairs_json = cookie_pairs_json);
                                let _ = tab.evaluate(&cookies_script, false);
                                tracing::info!("Cookies injected via JS");

                                // Reload page so cookies are sent to the server and session is activated
                                let _ = tab.navigate_to("https://chat.deepseek.com/");
                                tracing::info!("Reloading DeepSeek page with cookies");
                                std::thread::sleep(std::time::Duration::from_secs(5));

                                // Verify we're on the chat page (not login)
                                let check_script = r#"
                                    (function() {
                                        return JSON.stringify({
                                            url: window.location.href,
                                            title: document.title,
                                            hasTextarea: !!document.querySelector('textarea'),
                                            bodySnippet: (document.body?.innerText || '').substring(0, 300)
                                        });
                                    })();
                                "#;
                                if let Ok(result) = tab.evaluate(check_script, false) {
                                    if let Some(serde_json::Value::String(s)) = &result.value {
                                        tracing::info!("DeepSeek page after reload: {}", s);
                                    }
                                }

                                let browser_arc = Arc::new(browser);
                                tracing::info!("Registering browser for account {}", account_id);
                                state.account_pool.register_browser(account_id, browser_arc.clone()).await;
                                tracing::info!("DeepSeek browser registered for account {}", account_id);
                            } else {
                                tracing::warn!("Failed to navigate to DeepSeek chat");
                            }
                        } else {
                            tracing::warn!("Failed to create new tab");
                        }
                    }
                    Err(e) => {
                        tracing::warn!("Failed to launch Chrome for DeepSeek: {}", e);
                    }
                }
            }
            Err(e) => {
                tracing::warn!("Chrome not found for DeepSeek browser chat: {}", e);
            }
        }
    }

    tracing::info!("Session injected for account {} (provider: {})", account_id, account.provider);

    Ok(Json(PasswordLoginResponse {
        success: true,
        message: "Session injected successfully".to_string(),
    }))
}

/// 手机验证码登录 - 发起请求体
#[derive(Debug, Deserialize)]
struct PhoneLoginInitRequest {
    phone: String,
}

/// 手机验证码登录 - 验证请求体
#[derive(Debug, Deserialize)]
struct PhoneLoginVerifyRequest {
    code: String,
}

/// 手机验证码登录 - 发起响应
#[derive(Debug, Serialize)]
struct PhoneLoginInitResponse {
    session_id: String,
    message: String,
}

/// POST /admin/accounts/:id/phone-login/init
///
/// 发起手机验证码登录：headless 浏览器填写手机号并点击发送验证码，
/// 返回会话 ID，前端用此 ID 轮询状态或等待管理员输入验证码后调用 verify
async fn initiate_phone_login(
    State(state): State<Arc<AppState>>,
    Extension(_auth): Extension<AuthContext>,
    Path(id): Path<String>,
    Json(body): Json<PhoneLoginInitRequest>,
) -> Result<Json<PhoneLoginInitResponse>, ApiError> {
    let account = state.db.get_browser_account(
        Uuid::parse_str(&id).map_err(|_| ApiError::InvalidRequest("Invalid account ID".to_string()))?
    ).await
    .map_err(|e| ApiError::Internal(anyhow::anyhow!("Database error: {}", e)))?
    .ok_or(ApiError::InvalidRequest("Account not found".to_string()))?;

    let session = state.headless_browser.initiate_phone_login(
        &account.provider,
        &body.phone,
    ).await
    .map_err(|e| match e {
        provider_client::ProviderError::ChromeNotFound => ApiError::InvalidRequest(
            "Chrome/Chromium browser not installed on server. Use manual session injection instead.".to_string()
        ),
        provider_client::ProviderError::BlockedByProvider => ApiError::InvalidRequest(
            "Login page blocked by CAPTCHA / Cloudflare. Use 'Manual Injection' tab instead.".to_string()
        ),
        other => ApiError::LoginFailed(format!("{}", other)),
    })?;


    let redis_key = format!("phone_login:{}", id);
    state.redis.set_with_ttl(&redis_key, &session.id.to_string(), 600).await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Redis error: {}", e)))?;

    Ok(Json(PhoneLoginInitResponse {
        session_id: session.id.to_string(),
        message: "Verification code sent. Check your phone.".to_string(),
    }))
}

/// POST /admin/accounts/:id/phone-login/verify
///
/// 完成手机验证码登录：headless 浏览器填写验证码、点击提交、等待登录完成、提取 Cookie
async fn complete_phone_login(
    State(state): State<Arc<AppState>>,
    Extension(_auth): Extension<AuthContext>,
    Path(id): Path<String>,
    Json(body): Json<PhoneLoginVerifyRequest>,
) -> Result<Json<PasswordLoginResponse>, ApiError> {
    let account_id = Uuid::parse_str(&id)
        .map_err(|_| ApiError::InvalidRequest("Invalid account ID".to_string()))?;

    let account = state.db.get_browser_account(account_id).await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Database error: {}", e)))?
        .ok_or(ApiError::InvalidRequest("Account not found".to_string()))?;

    let redis_key = format!("phone_login:{}", id);
    let session_id = state.redis.get(&redis_key).await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Redis error: {}", e)))?
        .and_then(|s| Uuid::parse_str(&s).ok())
        .ok_or(ApiError::InvalidRequest("Phone login session expired. Please initiate again.".to_string()))?;

    let cookies_json = state.headless_browser.complete_phone_login(
        session_id,
        &body.code,
    ).await
    .map_err(|e| match e {
        provider_client::ProviderError::AuthenticationError(msg) => ApiError::LoginFailed(msg),
        provider_client::ProviderError::LoginFailed(msg) => ApiError::LoginFailed(msg),
        other => ApiError::LoginFailed(format!("{}", other)),
    })?;

    let session_data: provider_client::PersistedSession =
        serde_json::from_str(&cookies_json)
            .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to parse session: {}", e)))?;

    state.db.update_browser_account_session(
        account_id,
        &cookies_json,
        None,
        None,
    ).await
    .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to update session: {}", e)))?;

    state.account_pool.register_account(
        account_id,
        account.provider.clone(),
        session_data,
    ).await
    .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to register account: {}", e)))?;

    let _ = state.redis.delete(&redis_key).await;

    Ok(Json(PasswordLoginResponse {
        success: true,
        message: "Phone login successful".to_string(),
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
                                    None,
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
