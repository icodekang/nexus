//! API 路由测试
//!
//! 测试所有 API 端点的请求和响应

use serde::{Deserialize, Serialize};
use serde_json::json;

// ============ 认证相关测试 ============

#[derive(Debug, Serialize, Deserialize)]
struct LoginRequest {
    email: String,
    password: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct LoginResponse {
    token: String,
    user: UserInfo,
}

#[derive(Debug, Serialize, Deserialize)]
struct UserInfo {
    id: String,
    email: String,
    subscription_plan: String,
    is_admin: bool,
}

#[test]
fn test_login_request_serialization() {
    let request = LoginRequest {
        email: "test@example.com".to_string(),
        password: "password123".to_string(),
    };

    let json = serde_json::to_string(&request).unwrap();
    assert!(json.contains("test@example.com"));
    assert!(json.contains("password123"));

    let deserialized: LoginRequest = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.email, request.email);
}

#[test]
fn test_login_response_parsing() {
    let response_json = r#"{
        "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
        "user": {
            "id": "user-123",
            "email": "test@example.com",
            "subscription_plan": "monthly",
            "is_admin": false
        }
    }"#;

    let response: LoginResponse = serde_json::from_str(response_json).unwrap();
    assert!(!response.token.is_empty());
    assert_eq!(response.user.email, "test@example.com");
    assert_eq!(response.user.subscription_plan, "monthly");
}

// ============ 用户相关测试 ============

#[derive(Debug, Serialize, Deserialize)]
struct UserResponse {
    id: String,
    email: String,
    phone: Option<String>,
    subscription_plan: String,
    is_admin: bool,
    is_active: bool,
    created_at: String,
}

#[test]
fn test_user_response_parsing() {
    let json = json!({
        "id": "user-123",
        "email": "test@example.com",
        "phone": "+1234567890",
        "subscription_plan": "yearly",
        "is_admin": false,
        "is_active": true,
        "created_at": "2024-01-15T10:30:00Z"
    });

    let user: UserResponse = serde_json::from_value(json).unwrap();
    assert_eq!(user.email, "test@example.com");
    assert_eq!(user.phone, Some("+1234567890".to_string()));
    assert!(user.is_active);
}

#[test]
fn test_user_without_phone() {
    let json = json!({
        "id": "user-456",
        "email": "nop@example.com",
        "phone": null,
        "subscription_plan": "none",
        "is_admin": false,
        "is_active": true,
        "created_at": "2024-01-15T10:30:00Z"
    });

    let user: UserResponse = serde_json::from_value(json).unwrap();
    assert!(user.phone.is_none());
}

// ============ 提供商相关测试 ============

#[derive(Debug, Serialize, Deserialize)]
struct ProviderResponse {
    id: String,
    name: String,
    slug: String,
    logo_url: Option<String>,
    api_base_url: String,
    is_active: bool,
    priority: i32,
}

#[test]
fn test_provider_response_parsing() {
    let json = json!({
        "id": "provider-1",
        "name": "OpenAI",
        "slug": "openai",
        "logo_url": "https://openai.com/logo.png",
        "api_base_url": "https://api.openai.com/v1",
        "is_active": true,
        "priority": 1
    });

    let provider: ProviderResponse = serde_json::from_value(json).unwrap();
    assert_eq!(provider.name, "OpenAI");
    assert_eq!(provider.slug, "openai");
    assert!(provider.is_active);
}

#[test]
fn test_create_provider_request() {
    #[derive(Debug, Serialize)]
    struct CreateProviderRequest {
        name: String,
        slug: String,
        api_base_url: Option<String>,
        priority: Option<i32>,
        is_active: bool,
    }

    let request = CreateProviderRequest {
        name: "DeepSeek".to_string(),
        slug: "deepseek".to_string(),
        api_base_url: Some("https://api.deepseek.com".to_string()),
        priority: Some(3),
        is_active: true,
    };

    let json = serde_json::to_string(&request).unwrap();
    assert!(json.contains("DeepSeek"));
    assert!(json.contains("deepseek"));
}

// ============ 模型相关测试 ============

#[derive(Debug, Serialize, Deserialize)]
struct ModelResponse {
    id: String,
    provider_id: String,
    name: String,
    slug: String,
    model_id: String,
    mode: String,
    context_window: i32,
    capabilities: Vec<String>,
    is_active: bool,
}

#[test]
fn test_model_response_parsing() {
    let json = json!({
        "id": "model-1",
        "provider_id": "provider-1",
        "name": "GPT-4",
        "slug": "gpt-4",
        "model_id": "gpt-4",
        "mode": "chat",
        "context_window": 128000,
        "capabilities": ["chat", "function"],
        "is_active": true
    });

    let model: ModelResponse = serde_json::from_value(json).unwrap();
    assert_eq!(model.name, "GPT-4");
    assert_eq!(model.context_window, 128000);
    assert!(model.capabilities.contains(&"chat".to_string()));
}

#[test]
fn test_create_model_request() {
    #[derive(Debug, Serialize)]
    struct CreateModelRequest {
        provider_id: String,
        name: String,
        slug: String,
        model_id: String,
        mode: Option<String>,
        context_window: Option<i32>,
        capabilities: Option<Vec<String>>,
    }

    let request = CreateModelRequest {
        provider_id: "provider-1".to_string(),
        name: "Claude 3".to_string(),
        slug: "claude-3".to_string(),
        model_id: "claude-3-opus".to_string(),
        mode: Some("chat".to_string()),
        context_window: Some(200000),
        capabilities: Some(vec!["chat".to_string(), "vision".to_string()]),
    };

    let json = serde_json::to_string(&request).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed["name"], "Claude 3");
    assert_eq!(parsed["context_window"], 200000);
}

#[test]
fn test_update_model_request() {
    #[derive(Debug, Serialize)]
    struct UpdateModelRequest {
        name: Option<String>,
        slug: Option<String>,
        model_id: Option<String>,
        provider_id: Option<String>,
        context_window: Option<i32>,
        capabilities: Option<Vec<String>>,
        is_active: Option<bool>,
    }

    // 部分更新
    let request = UpdateModelRequest {
        name: Some("Updated Name".to_string()),
        slug: None,
        model_id: None,
        provider_id: None,
        context_window: Some(100000),
        capabilities: None,
        is_active: Some(false),
    };

    let json = serde_json::to_string(&request).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed["name"], "Updated Name");
    assert_eq!(parsed["context_window"], 100000);
    assert_eq!(parsed["is_active"], false);
    assert!(!parsed.as_object().unwrap().contains_key("slug"));
}

// ============ 订阅相关测试 ============

#[derive(Debug, Serialize, Deserialize)]
struct SubscriptionResponse {
    subscription_plan: String,
    subscription_start: Option<String>,
    subscription_end: Option<String>,
    is_active: bool,
}

#[test]
fn test_subscription_response_parsing() {
    let json = json!({
        "subscription_plan": "monthly",
        "subscription_start": "2024-01-01T00:00:00Z",
        "subscription_end": "2024-02-01T00:00:00Z",
        "is_active": true
    });

    let sub: SubscriptionResponse = serde_json::from_value(json).unwrap();
    assert_eq!(sub.subscription_plan, "monthly");
    assert!(sub.is_active);
}

#[test]
fn test_subscribe_request() {
    #[derive(Debug, Serialize)]
    struct SubscribeRequest {
        plan: String,
    }

    let request = SubscribeRequest {
        plan: "yearly".to_string(),
    };

    let json = serde_json::to_string(&request).unwrap();
    assert!(json.contains("yearly"));
}

// ============ 使用量相关测试 ============

#[derive(Debug, Serialize, Deserialize)]
struct UsageResponse {
    period_start: String,
    period_end: String,
    total_requests: i64,
    total_input_tokens: i64,
    total_output_tokens: i64,
    total_tokens: i64,
    token_quota: Option<i64>,
    quota_used_percent: f64,
}

#[test]
fn test_usage_response_parsing() {
    let json = json!({
        "period_start": "2024-01-01T00:00:00Z",
        "period_end": "2024-01-31T23:59:59Z",
        "total_requests": 1500,
        "total_input_tokens": 500000,
        "total_output_tokens": 1000000,
        "total_tokens": 1500000,
        "token_quota": 5000000,
        "quota_used_percent": 30.0
    });

    let usage: UsageResponse = serde_json::from_value(json).unwrap();
    assert_eq!(usage.total_requests, 1500);
    assert_eq!(usage.total_tokens, 1500000);
    assert!((usage.quota_used_percent - 30.0).abs() < 0.01);
}

// ============ 交易相关测试 ============

#[derive(Debug, Serialize, Deserialize)]
struct TransactionResponse {
    id: String,
    user_id: String,
    user_email: String,
    transaction_type: String,
    amount: f64,
    plan: Option<String>,
    status: String,
    description: Option<String>,
    created_at: String,
}

#[test]
fn test_transaction_response_parsing() {
    let json = json!({
        "id": "tx-123",
        "user_id": "user-1",
        "user_email": "user@example.com",
        "transaction_type": "purchase",
        "amount": 19.99,
        "plan": "monthly",
        "status": "completed",
        "description": "Monthly subscription",
        "created_at": "2024-01-15T10:30:00Z"
    });

    let tx: TransactionResponse = serde_json::from_value(json).unwrap();
    assert_eq!(tx.transaction_type, "purchase");
    assert!((tx.amount - 19.99).abs() < 0.001);
    assert_eq!(tx.status, "completed");
}

// ============ 错误响应测试 ============

#[derive(Debug, Serialize, Deserialize)]
struct ErrorResponse {
    error: ErrorDetail,
}

#[derive(Debug, Serialize, Deserialize)]
struct ErrorDetail {
    message: String,
    code: Option<String>,
}

#[test]
fn test_error_response_parsing() {
    let json = json!({
        "error": {
            "message": "Invalid credentials",
            "code": "invalid_credentials"
        }
    });

    let error: ErrorResponse = serde_json::from_value(json).unwrap();
    assert_eq!(error.error.message, "Invalid credentials");
    assert_eq!(error.error.code, Some("invalid_credentials".to_string()));
}

// ============ API 密钥相关测试 ============

#[derive(Debug, Serialize, Deserialize)]
struct ApiKeyResponse {
    id: String,
    name: Option<String>,
    key_prefix: String,
    is_active: bool,
    last_used_at: Option<String>,
    created_at: String,
}

#[test]
fn test_api_key_response_parsing() {
    let json = json!({
        "id": "key-123",
        "name": "Production Key",
        "key_prefix": "nk_abc1",
        "is_active": true,
        "last_used_at": "2024-01-15T10:30:00Z",
        "created_at": "2024-01-01T00:00:00Z"
    });

    let key: ApiKeyResponse = serde_json::from_value(json).unwrap();
    assert_eq!(key.key_prefix, "nk_abc1");
    assert!(key.is_active);
    assert_eq!(key.name, Some("Production Key".to_string()));
}

#[test]
fn test_create_api_key_response() {
    #[derive(Debug, Serialize, Deserialize)]
    struct CreateApiKeyResponse {
        id: String,
        key: String,
        name: String,
        created_at: String,
    }

    let json = json!({
        "id": "key-new",
        "key": "nk_full_key_here_secret",
        "name": "Dev Key",
        "created_at": "2024-01-15T10:30:00Z"
    });

    let response: CreateApiKeyResponse = serde_json::from_value(json).unwrap();
    // 完整密钥只在此响应中返回一次
    assert!(response.key.starts_with("nk_"));
}
