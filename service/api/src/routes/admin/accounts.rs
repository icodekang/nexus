use axum::{
    routing::{get, post, delete},
    Router, Json, Extension,
    extract::{Path, State, Query},
    response::{IntoResponse, Response, sse::{Event, Sse}},
    http::HeaderMap,
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
use models::{BrowserAccount, BrowserAccountStatus, QrCodeSession};

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/accounts", get(list_accounts).post(create_account))
        .route("/accounts/:id", delete(delete_account))
        .route("/accounts/:id/qrcode", get(get_qrcode))
        .route("/accounts/:id/status", get(get_account_status))
        .route("/accounts/complete-auth", post(complete_auth))
}

// ============ Browser Accounts CRUD ============

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

#[derive(Debug, Deserialize)]
struct CreateAccountRequest {
    provider: String,
}

async fn create_account(
    State(state): State<Arc<AppState>>,
    Extension(_auth): Extension<AuthContext>,
    Json(body): Json<CreateAccountRequest>,
) -> Result<Json<AccountResponse>, ApiError> {
    let provider = body.provider.to_lowercase();
    if provider != "claude" && provider != "chatgpt" {
        return Err(ApiError::InvalidRequest(
            "Provider must be 'claude' or 'chatgpt'".to_string()
        ));
    }

    let account = BrowserAccount::new(provider);

    state.db.create_browser_account(&account).await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to create account: {}", e)))?;

    Ok(Json(account.into()))
}

async fn delete_account(
    State(state): State<Arc<AppState>>,
    Extension(_auth): Extension<AuthContext>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let account_id = Uuid::parse_str(&id)
        .map_err(|_| ApiError::InvalidRequest("Invalid account ID".to_string()))?;

    // Delete from database
    state.db.delete_browser_account(account_id).await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to delete account: {}", e)))?;

    // Clean up Redis session
    let _ = state.redis.delete_account_session(&id).await;

    Ok(Json(serde_json::json!({ "deleted": true })))
}

// ============ QR Code Generation ============

#[derive(Debug, Serialize)]
struct QrCodeResponse {
    session_id: String,
    code: String,             // 6-digit code
    expires_at: String,
    auth_url: String,         // URL to open on mobile
}

async fn get_qrcode(
    State(state): State<Arc<AppState>>,
    Extension(_auth): Extension<AuthContext>,
    Path(id): Path<String>,
) -> Result<Json<QrCodeResponse>, ApiError> {
    let account_id = Uuid::parse_str(&id)
        .map_err(|_| ApiError::InvalidRequest("Invalid account ID".to_string()))?;

    // Get account to verify it exists
    let _account = state.db.get_browser_account(account_id).await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Database error: {}", e)))?
        .ok_or(ApiError::InvalidRequest("Account not found".to_string()))?;

    // Create QR session
    let qr_session = QrCodeSession::new(account_id);

    // Store QR session in Redis (5 min expiry)
    state.redis.store_qr_session(&qr_session.code, &qr_session.id.to_string()).await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Redis error: {}", e)))?;

    // Generate auth URL
    let base_url = std::env::var("NEXUS_BASE_URL").unwrap_or_else(|_| "http://localhost:3000".to_string());
    let auth_url = format!("{}/auth/callback?code={}&session_id={}",
        base_url, qr_session.code, account_id);

    Ok(Json(QrCodeResponse {
        session_id: qr_session.id.to_string(),
        code: qr_session.code,
        expires_at: qr_session.code_expires_at.to_rfc3339(),
        auth_url,
    }))
}

// ============ Real-time Status via SSE ============

async fn get_account_status(
    State(state): State<Arc<AppState>>,
    Extension(_auth): Extension<AuthContext>,
    Path(id): Path<String>,
) -> Result<Sse<impl Stream<Item = Result<Event, std::convert::Infallible>> + Send>, ApiError> {
    let account_id = Uuid::parse_str(&id)
        .map_err(|_| ApiError::InvalidRequest("Invalid account ID".to_string()))?;

    // Create broadcast channel for this client
    let (tx, rx) = broadcast::channel::<Result<Event, std::convert::Infallible>>(100);

    // Spawn task to poll database for status changes
    let state_clone = state.clone();
    tokio::spawn(async move {
        let mut last_status: Option<String> = None;

        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

            // Get current account status
            match state_clone.db.get_browser_account(account_id).await {
                Ok(Some(account)) => {
                    let current_status = account.status.as_str().to_string();
                    if last_status.as_ref() != Some(&current_status) {
                        last_status = Some(current_status.clone());

                        let event_data = serde_json::json!({
                            "status": current_status,
                            "email": account.email,
                        }).to_string();

                        let event = Event::default()
                            .event("status")
                            .data(event_data);

                        let _ = tx.send(Ok(event));

                        // If active or error, stop watching
                        if current_status == "active" || current_status == "error" {
                            break;
                        }
                    }
                }
                Ok(None) => {
                    // Account deleted
                    let event = Event::default()
                        .event("status")
                        .data(r#"{"status":"deleted"}"#);
                    let _ = tx.send(Ok(event));
                    break;
                }
                Err(_) => {
                    // DB error, keep trying
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

// ============ Complete Auth Callback ============

#[derive(Debug, Deserialize)]
struct CompleteAuthRequest {
    code: String,
    session_id: String,      // QR session ID
    session_data: String,    // JSON encrypted session data
    email: Option<String>,
}

async fn complete_auth(
    State(state): State<Arc<AppState>>,
    Json(body): Json<CompleteAuthRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // Verify QR session exists and hasn't expired
    let qr_session_id = state.redis.get_qr_session(&body.code).await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Redis error: {}", e)))?
        .ok_or(ApiError::InvalidRequest("Invalid or expired QR code".to_string()))?;

    // Parse account ID from path
    let account_id = Uuid::parse_str(&body.session_id)
        .map_err(|_| ApiError::InvalidRequest("Invalid session ID".to_string()))?;

    // Get account
    let account = state.db.get_browser_account(account_id).await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Database error: {}", e)))?
        .ok_or(ApiError::InvalidRequest("Account not found".to_string()))?;

    // Update account with session data
    state.db.update_browser_account_session(
        account_id,
        &body.session_data,
        body.email.as_deref(),
    ).await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to update account: {}", e)))?;

    // Store session in Redis
    state.redis.store_account_session(&account_id.to_string(), &body.session_data).await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Redis error: {}", e)))?;

    // Publish status update
    let status_update = serde_json::json!({
        "status": "active",
        "email": body.email,
    }).to_string();

    state.redis.publish_account_status(&account_id.to_string(), &status_update).await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Redis error: {}", e)))?;

    // Delete QR session from Redis
    state.redis.delete_qr_session(&body.code).await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Redis error: {}", e)))?;

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Authentication completed successfully"
    })))
}
