//! 适配器模块测试
//!
//! 测试 LLM 适配器的配置和类型转换

use serde::{Deserialize, Serialize};

// ============ 提供商配置测试 ============

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProviderConfig {
    name: String,
    api_base_url: String,
    api_key: Option<String>,
    is_active: bool,
}

#[test]
fn test_provider_config_creation() {
    let config = ProviderConfig {
        name: "OpenAI".to_string(),
        api_base_url: "https://api.openai.com/v1".to_string(),
        api_key: Some("sk-xxx".to_string()),
        is_active: true,
    };

    assert_eq!(config.name, "OpenAI");
    assert!(config.api_base_url.contains("openai.com"));
    assert!(config.is_active);
}

#[test]
fn test_provider_config_without_api_key() {
    let config = ProviderConfig {
        name: "Custom Provider".to_string(),
        api_base_url: "https://custom.api.com/v1".to_string(),
        api_key: None,
        is_active: false,
    };

    assert!(config.api_key.is_none());
    assert!(!config.is_active);
}

// ============ 模型选择器测试 ============

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ModelSelector {
    provider: String,
    model_id: String,
    mode: String,
}

#[test]
fn test_model_selector_creation() {
    let selector = ModelSelector {
        provider: "openai".to_string(),
        model_id: "gpt-4".to_string(),
        mode: "chat".to_string(),
    };

    assert_eq!(selector.provider, "openai");
    assert_eq!(selector.model_id, "gpt-4");
}

#[test]
fn test_model_selector_equality() {
    let selector1 = ModelSelector {
        provider: "anthropic".to_string(),
        model_id: "claude-3".to_string(),
        mode: "chat".to_string(),
    };

    let selector2 = ModelSelector {
        provider: "anthropic".to_string(),
        model_id: "claude-3".to_string(),
        mode: "chat".to_string(),
    };

    assert_eq!(selector1.provider, selector2.provider);
    assert_eq!(selector1.model_id, selector2.model_id);
}

// ============ 请求/响应类型测试 ============

#[derive(Debug, Serialize, Deserialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    stream: bool,
    max_tokens: Option<u32>,
    temperature: Option<f32>,
}

#[test]
fn test_chat_request_serialization() {
    let request = ChatRequest {
        model: "gpt-4".to_string(),
        messages: vec![
            ChatMessage {
                role: "system".to_string(),
                content: "You are a helpful assistant.".to_string(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: "Hello!".to_string(),
            },
        ],
        stream: false,
        max_tokens: Some(1000),
        temperature: Some(0.7),
    };

    let json = serde_json::to_string(&request).unwrap();
    assert!(json.contains("gpt-4"));
    assert!(json.contains("system"));
    assert!(json.contains("user"));
}

#[test]
fn test_chat_request_without_optional_fields() {
    let request = ChatRequest {
        model: "gpt-3.5-turbo".to_string(),
        messages: vec![ChatMessage {
            role: "user".to_string(),
            content: "Hi".to_string(),
        }],
        stream: true,
        max_tokens: None,
        temperature: None,
    };

    let json = serde_json::to_string(&request).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

    assert!(parsed["max_tokens"].is_null());
    assert!(parsed["temperature"].is_null());
    assert_eq!(parsed["stream"], true);
}

#[derive(Debug, Serialize, Deserialize)]
struct ChatResponse {
    id: String,
    choices: Vec<Choice>,
    usage: Usage,
}

#[derive(Debug, Serialize, Deserialize)]
struct Choice {
    message: ChatMessage,
    finish_reason: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Usage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

#[test]
fn test_chat_response_parsing() {
    let json = r#"{
        "id": "chatcmpl-123",
        "choices": [{
            "message": {
                "role": "assistant",
                "content": "Hello! How can I help you?"
            },
            "finish_reason": "stop"
        }],
        "usage": {
            "prompt_tokens": 10,
            "completion_tokens": 20,
            "total_tokens": 30
        }
    }"#;

    let response: ChatResponse = serde_json::from_str(json).unwrap();
    assert_eq!(response.choices[0].message.content, "Hello! How can I help you?");
    assert_eq!(response.usage.total_tokens, 30);
}

// ============ 流式响应测试 ============

#[derive(Debug, Deserialize)]
struct StreamChunk {
    choices: Vec<StreamChoice>,
}

#[derive(Debug, Deserialize)]
struct StreamChoice {
    delta: Delta,
}

#[derive(Debug, Deserialize)]
struct Delta {
    content: Option<String>,
}

#[test]
fn test_stream_chunk_parsing() {
    let json = r#"{"choices":[{"delta":{"content":"Hello"}}]}"#;
    let chunk: StreamChunk = serde_json::from_str(json).unwrap();
    assert_eq!(chunk.choices[0].delta.content, Some("Hello".to_string()));
}

#[test]
fn test_stream_done_chunk() {
    #[derive(Debug, Deserialize)]
    struct StreamDone {
        choices: Vec<StreamChoice>,
    }

    let json = r#"{"choices":[{"delta":{}}]}"#;
    let done: StreamDone = serde_json::from_str(json).unwrap();
    assert!(done.choices[0].delta.content.is_none());
}

// ============ 错误响应测试 ============

#[derive(Debug, Deserialize)]
struct ProviderError {
    error: ProviderErrorDetail,
}

#[derive(Debug, Deserialize)]
struct ProviderErrorDetail {
    message: String,
    #[serde(rename = "type")]
    error_type: Option<String>,
    code: Option<String>,
}

#[test]
fn test_provider_error_parsing() {
    let json = r#"{
        "error": {
            "message": "Invalid API key",
            "type": "authentication_error",
            "code": "invalid_api_key"
        }
    }"#;

    let error: ProviderError = serde_json::from_str(json).unwrap();
    assert_eq!(error.error.message, "Invalid API key");
    assert_eq!(error.error.error_type, Some("authentication_error".to_string()));
}

// ============ 浏览器模拟器测试 ============

#[derive(Debug, Serialize, Deserialize)]
struct BrowserAccount {
    id: String,
    provider: String,
    email: Option<String>,
    status: String,
    request_count: i32,
}

#[test]
fn test_browser_account_creation() {
    let account = BrowserAccount {
        id: "acc-123".to_string(),
        provider: "claude".to_string(),
        email: Some("user@anthropic.com".to_string()),
        status: "active".to_string(),
        request_count: 0,
    };

    assert_eq!(account.provider, "claude");
    assert!(account.email.is_some());
}

#[test]
fn test_browser_account_status() {
    let statuses = vec!["pending", "active", "expired", "error"];

    for status in statuses {
        let json = format!(r#"{{"id":"acc-1","provider":"claude","email":null,"status":"{}","request_count":0}}"#, status);
        let account: BrowserAccount = serde_json::from_str(&json).unwrap();
        assert_eq!(account.status, status);
    }
}

// ============ 二维码会话测试 ============

#[derive(Debug, Serialize, Deserialize)]
struct QrCodeSession {
    session_id: String,
    code: String,
    expires_at: String,
    auth_url: String,
}

#[test]
fn test_qr_code_session_parsing() {
    let json = r#"{
        "session_id": "sess_abc123",
        "code": "AUTH123",
        "expires_at": "2024-01-15T10:30:00Z",
        "auth_url": "https://claude.ai/auth?code=AUTH123"
    }"#;

    let session: QrCodeSession = serde_json::from_str(json).unwrap();
    assert_eq!(session.session_id, "sess_abc123");
    assert!(session.auth_url.contains("claude.ai"));
}

// ============ API 密钥掩码测试 ============

#[derive(Debug, Serialize, Deserialize)]
struct MaskedApiKey {
    id: String,
    api_key_masked: String,
    api_key_preview: String,
}

#[test]
fn test_api_key_masking() {
    let json = r#"{
        "id": "key-123",
        "api_key_masked": "nk_****************************xyz",
        "api_key_preview": "nk_abc1...xyz"
    }"#;

    let key: MaskedApiKey = serde_json::from_str(json).unwrap();
    assert!(key.api_key_masked.contains("*"));
    assert!(key.api_key_preview.contains("..."));
    assert!(!key.api_key_masked.contains("sk-"));
}
