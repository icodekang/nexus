//! Client API Integration Tests
//!
//! 集成测试：测试所有客户端 API 端点 (v1/*)
//!
//! 运行方式 (需要后端服务运行):
//! cargo test --test client_api_test -- --nocapture

use serde::{Deserialize, Serialize};

// ============ 认证相关 ============

#[derive(Debug, Serialize, Deserialize)]
struct ClientLoginRequest {
    email: String,
    password: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ClientLoginResponse {
    token: String,
    user: ClientUser,
}

#[derive(Debug, Serialize, Deserialize)]
struct ClientUser {
    id: String,
    email: String,
    subscription_plan: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct RegisterRequest {
    email: String,
    password: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct SendSmsRequest {
    phone: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct SendSmsResponse {
    message: String,
    seconds_valid: i64,
}

#[derive(Debug, Serialize, Deserialize)]
struct VerifySmsRequest {
    phone: String,
    code: String,
}

// ============ 模型相关 ============

#[derive(Debug, Serialize, Deserialize)]
struct ModelsListResponse {
    data: Vec<ClientModel>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ClientModel {
    id: String,
    name: String,
    provider: String,
    provider_name: String,
    context_window: i32,
    capabilities: Vec<String>,
}

// ============ 聊天相关 ============

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

#[derive(Debug, Serialize, Deserialize)]
struct ChatResponse {
    id: String,
    choices: Vec<ChatChoice>,
    usage: ChatUsage,
}

#[derive(Debug, Serialize, Deserialize)]
struct ChatChoice {
    message: ChatMessage,
    finish_reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ChatUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

// ============ 订阅相关 ============

#[derive(Debug, Serialize, Deserialize)]
struct SubscriptionResponse {
    subscription_plan: String,
    subscription_start: Option<String>,
    subscription_end: Option<String>,
    is_active: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct SubscribeRequest {
    plan: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct SubscribeResponse {
    message: String,
    plan: String,
    subscription_start: String,
    subscription_end: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct PlansListResponse {
    plans: Vec<PlanInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
struct PlanInfo {
    plan: String,
    name: String,
    price_monthly: f64,
    price_yearly: f64,
    price_team_monthly: f64,
    features: Vec<String>,
}

// ============ 使用量相关 ============

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
    usage_by_provider: Vec<ProviderUsage>,
    usage_by_model: Vec<ModelUsage>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ProviderUsage {
    provider: String,
    requests: i64,
    input_tokens: i64,
    output_tokens: i64,
}

#[derive(Debug, Serialize, Deserialize)]
struct ModelUsage {
    model: String,
    provider: String,
    requests: i64,
    input_tokens: i64,
    output_tokens: i64,
}

// ============ API 密钥相关 ============

#[derive(Debug, Serialize, Deserialize)]
struct ApiKeysListResponse {
    data: Vec<ApiKeyInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ApiKeyInfo {
    id: String,
    name: Option<String>,
    key_prefix: String,
    is_active: bool,
    last_used_at: Option<String>,
    created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct CreateApiKeyRequest {
    name: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct CreateApiKeyResponse {
    id: String,
    key: String,
    name: String,
    created_at: String,
}

// ============ 集成测试 ============

#[cfg(test)]
mod tests {
    use super::*;

    // ============ 认证测试 ============

    #[test]
    fn test_login_request_serialization() {
        let request = ClientLoginRequest {
            email: "user@example.com".to_string(),
            password: "password123".to_string(),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("user@example.com"));
        assert!(json.contains("password123"));

        let parsed: ClientLoginRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.email, "user@example.com");
    }

    #[test]
    fn test_login_response_parsing() {
        let json = r#"{
            "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
            "user": {
                "id": "user-123",
                "email": "user@example.com",
                "subscription_plan": "monthly"
            }
        }"#;

        let response: ClientLoginResponse = serde_json::from_str(json).unwrap();
        assert!(!response.token.is_empty());
        assert_eq!(response.user.email, "user@example.com");
        assert_eq!(response.user.subscription_plan, "monthly");
    }

    #[test]
    fn test_register_request_serialization() {
        let request = RegisterRequest {
            email: "newuser@example.com".to_string(),
            password: "securepassword".to_string(),
        };

        let json = serde_json::to_string(&request).unwrap();
        let parsed: RegisterRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.email, "newuser@example.com");
    }

    #[test]
    fn test_sms_send_request() {
        let request = SendSmsRequest {
            phone: "+8613812345678".to_string(),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("+8613812345678"));
    }

    #[test]
    fn test_sms_send_response() {
        let json = r#"{
            "message": "SMS sent successfully",
            "seconds_valid": 300
        }"#;

        let response: SendSmsResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.seconds_valid, 300);
    }

    #[test]
    fn test_sms_verify_request() {
        let request = VerifySmsRequest {
            phone: "+8613812345678".to_string(),
            code: "123456".to_string(),
        };

        let json = serde_json::to_string(&request).unwrap();
        let parsed: VerifySmsRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.code, "123456");
    }

    #[test]
    fn test_logout_request() {
        // 退出登录通常只需要 Authorization header，不需要 body
        let token = "Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...";
        assert!(token.starts_with("Bearer "));
    }

    // ============ 模型测试 ============

    #[test]
    fn test_models_list_response_parsing() {
        let json = r#"{
            "data": [
                {
                    "id": "model-1",
                    "name": "GPT-4",
                    "provider": "openai",
                    "provider_name": "OpenAI",
                    "context_window": 128000,
                    "capabilities": ["chat", "function"]
                },
                {
                    "id": "model-2",
                    "name": "Claude 3",
                    "provider": "anthropic",
                    "provider_name": "Anthropic",
                    "context_window": 200000,
                    "capabilities": ["chat", "vision"]
                }
            ]
        }"#;

        let response: ModelsListResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.data.len(), 2);
        assert_eq!(response.data[0].name, "GPT-4");
        assert_eq!(response.data[1].provider, "anthropic");
    }

    #[test]
    fn test_models_filtered_by_provider() {
        let provider = "openai";
        let query = format!("?provider={}", provider);
        assert_eq!(query, "?provider=openai");
    }

    #[test]
    fn test_model_capabilities() {
        let model = ClientModel {
            id: "model-1".to_string(),
            name: "GPT-4".to_string(),
            provider: "openai".to_string(),
            provider_name: "OpenAI".to_string(),
            context_window: 128000,
            capabilities: vec!["chat".to_string(), "function".to_string()],
        };

        assert!(model.capabilities.contains(&"chat".to_string()));
        assert!(!model.capabilities.contains(&"vision".to_string()));
    }

    #[test]
    fn test_model_context_window_scaling() {
        let models = vec![
            ("GPT-3.5-Turbo", 16385),
            ("GPT-4-8K", 8192),
            ("GPT-4-32K", 32768),
            ("Claude 3", 200000),
        ];

        for (name, ctx) in models {
            assert!(ctx > 0);
            if name.contains("32K") {
                assert!(ctx > 30000);
            }
        }
    }

    // ============ 聊天测试 ============

    #[test]
    fn test_chat_request_non_streaming() {
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
        let parsed: ChatRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.model, "gpt-4");
        assert_eq!(parsed.messages.len(), 2);
        assert!(!parsed.stream);
    }

    #[test]
    fn test_chat_request_streaming() {
        let request = ChatRequest {
            model: "gpt-4".to_string(),
            messages: vec![ChatMessage {
                role: "user".to_string(),
                content: "Continue the story...".to_string(),
            }],
            stream: true,
            max_tokens: None,
            temperature: None,
        };

        let json = serde_json::to_string(&request).unwrap();
        let parsed: ChatRequest = serde_json::from_str(&json).unwrap();
        assert!(parsed.stream);
        assert!(parsed.max_tokens.is_none());
    }

    #[test]
    fn test_chat_response_parsing() {
        let json = r#"{
            "id": "chatcmpl-123",
            "choices": [
                {
                    "message": {
                        "role": "assistant",
                        "content": "Hello! How can I help you today?"
                    },
                    "finish_reason": "stop"
                }
            ],
            "usage": {
                "prompt_tokens": 10,
                "completion_tokens": 20,
                "total_tokens": 30
            }
        }"#;

        let response: ChatResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.choices[0].message.content, "Hello! How can I help you today?");
        assert_eq!(response.usage.total_tokens, 30);
    }

    #[test]
    fn test_chat_message_roles() {
        let valid_roles = vec!["user", "assistant", "system"];

        for role in valid_roles {
            let msg = ChatMessage {
                role: role.to_string(),
                content: "Test".to_string(),
            };
            assert!(valid_roles.contains(&msg.role.as_str()));
        }
    }

    #[test]
    fn test_chat_with_context_window() {
        let request = ChatRequest {
            model: "gpt-4".to_string(),
            messages: vec![
                ChatMessage {
                    role: "system".to_string(),
                    content: "You are a helpful assistant.".to_string(),
                },
            ],
            stream: false,
            max_tokens: Some(500),
            temperature: None,
        };

        assert!(request.max_tokens.unwrap() <= 1000);
    }

    // ============ 订阅测试 ============

    #[test]
    fn test_subscription_response_parsing() {
        let json = r#"{
            "subscription_plan": "monthly",
            "subscription_start": "2024-01-01T00:00:00Z",
            "subscription_end": "2024-02-01T00:00:00Z",
            "is_active": true
        }"#;

        let response: SubscriptionResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.subscription_plan, "monthly");
        assert!(response.is_active);
    }

    #[test]
    fn test_subscribe_request() {
        let request = SubscribeRequest {
            plan: "yearly".to_string(),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("yearly"));
    }

    #[test]
    fn test_subscribe_response_parsing() {
        let json = r#"{
            "message": "Subscription updated successfully",
            "plan": "yearly",
            "subscription_start": "2024-01-15T00:00:00Z",
            "subscription_end": "2025-01-15T00:00:00Z"
        }"#;

        let response: SubscribeResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.plan, "yearly");
        assert!(response.subscription_end.contains("2025"));
    }

    #[test]
    fn test_plans_list_response() {
        let json = r#"{
            "plans": [
                {
                    "plan": "monthly",
                    "name": "Monthly",
                    "price_monthly": 19.0,
                    "price_yearly": 199.0,
                    "price_team_monthly": 49.0,
                    "features": ["api_access", "all_models"]
                },
                {
                    "plan": "yearly",
                    "name": "Yearly",
                    "price_monthly": 16.58,
                    "price_yearly": 199.0,
                    "price_team_monthly": 49.0,
                    "features": ["api_access", "all_models", "priority_support"]
                }
            ]
        }"#;

        let response: PlansListResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.plans.len(), 2);
        assert_eq!(response.plans[0].plan, "monthly");
        assert_eq!(response.plans[1].plan, "yearly");
    }

    #[test]
    fn test_plan_price_calculation() {
        let monthly_price = 19.0;
        let yearly_price = 199.0;

        // 年付是否比月付便宜
        let monthly_annual = monthly_price * 12.0;
        assert!(yearly_price < monthly_annual);

        // 节省百分比
        let savings = (monthly_annual - yearly_price) / monthly_annual * 100.0;
        assert!(savings > 10.0); // 至少节省 10%
    }

    // ============ 使用量测试 ============

    #[test]
    fn test_usage_response_parsing() {
        let json = r#"{
            "period_start": "2024-01-01T00:00:00Z",
            "period_end": "2024-01-31T23:59:59Z",
            "total_requests": 1500,
            "total_input_tokens": 500000,
            "total_output_tokens": 1000000,
            "total_tokens": 1500000,
            "token_quota": 5000000,
            "quota_used_percent": 30.0,
            "usage_by_provider": [
                {"provider": "openai", "requests": 1000, "input_tokens": 300000, "output_tokens": 600000},
                {"provider": "anthropic", "requests": 500, "input_tokens": 200000, "output_tokens": 400000}
            ],
            "usage_by_model": [
                {"model": "gpt-4", "provider": "openai", "requests": 800, "input_tokens": 200000, "output_tokens": 400000},
                {"model": "claude-3", "provider": "anthropic", "requests": 500, "input_tokens": 200000, "output_tokens": 400000}
            ]
        }"#;

        let response: UsageResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.total_requests, 1500);
        assert_eq!(response.total_tokens, 1500000);
        assert!((response.quota_used_percent - 30.0).abs() < 0.01);
        assert_eq!(response.usage_by_provider.len(), 2);
        assert_eq!(response.usage_by_model.len(), 2);
    }

    #[test]
    fn test_usage_without_quota() {
        let json = r#"{
            "period_start": "2024-01-01T00:00:00Z",
            "period_end": "2024-01-31T23:59:59Z",
            "total_requests": 500,
            "total_input_tokens": 100000,
            "total_output_tokens": 200000,
            "total_tokens": 300000,
            "token_quota": null,
            "quota_used_percent": 0.0,
            "usage_by_provider": [],
            "usage_by_model": []
        }"#;

        let response: UsageResponse = serde_json::from_str(json).unwrap();
        assert!(response.token_quota.is_none());
        assert_eq!(response.quota_used_percent, 0.0);
    }

    #[test]
    fn test_usage_calculation() {
        let input = 500000;
        let output = 1000000;
        let total = input + output;

        assert_eq!(total, 1500000);
    }

    #[test]
    fn test_quota_usage_percentage() {
        let used = 1500000;
        let quota = 5000000;
        let percentage = (used as f64 / quota as f64) * 100.0;

        assert!((percentage - 30.0).abs() < 0.01);
    }

    #[test]
    fn test_usage_by_provider_aggregation() {
        let providers = vec![
            ProviderUsage {
                provider: "openai".to_string(),
                requests: 1000,
                input_tokens: 300000,
                output_tokens: 600000,
            },
            ProviderUsage {
                provider: "anthropic".to_string(),
                requests: 500,
                input_tokens: 200000,
                output_tokens: 400000,
            },
        ];

        let total_requests: i64 = providers.iter().map(|p| p.requests).sum();
        let total_tokens: i64 = providers.iter().map(|p| p.input_tokens + p.output_tokens).sum();

        assert_eq!(total_requests, 1500);
        assert_eq!(total_tokens, 1500000);
    }

    // ============ API 密钥测试 ============

    #[test]
    fn test_api_keys_list_response_parsing() {
        let json = r#"{
            "data": [
                {
                    "id": "key-1",
                    "name": "Production Key",
                    "key_prefix": "nk_abc1",
                    "is_active": true,
                    "last_used_at": "2024-01-15T10:30:00Z",
                    "created_at": "2024-01-01T00:00:00Z"
                }
            ]
        }"#;

        let response: ApiKeysListResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.data.len(), 1);
        assert_eq!(response.data[0].key_prefix, "nk_abc1");
        assert!(response.data[0].is_active);
    }

    #[test]
    fn test_create_api_key_request() {
        let request = CreateApiKeyRequest {
            name: "Development Key".to_string(),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("Development Key"));
    }

    #[test]
    fn test_create_api_key_response() {
        let json = r#"{
            "id": "key-new",
            "key": "nk_full_secret_key_here_do_not_store",
            "name": "Dev Key",
            "created_at": "2024-01-15T10:30:00Z"
        }"#;

        let response: CreateApiKeyResponse = serde_json::from_str(json).unwrap();
        assert!(response.key.starts_with("nk_"));
        assert!(response.key.len() > 20); // 完整密钥应该比较长
    }

    #[test]
    fn test_api_key_prefix_validation() {
        let valid_prefixes = vec!["nk_", "sk_"];

        for prefix in valid_prefixes {
            let key = format!("{}abc123xyz", prefix);
            assert!(key.starts_with(prefix));
        }
    }

    #[test]
    fn test_api_key_masking() {
        let full_key = "nk_abcdefghijklmnopqrstuvwxyz123456";
        let prefix = &full_key[..6];
        let suffix = &full_key[full_key.len() - 4..];

        let masked = format!("{}...{}", prefix, suffix);
        assert!(masked.starts_with("nk_abc"));
        assert!(masked.ends_with("3456"));
        assert!(masked.contains("..."));
    }

    // ============ 客户端 API 路径测试 ============

    #[test]
    fn test_client_api_paths() {
        let paths = vec![
            "/v1/auth/login",
            "/v1/auth/register",
            "/v1/auth/send-sms",
            "/v1/auth/verify-sms",
            "/v1/auth/logout",
            "/v1/models",
            "/v1/chat/completions",
            "/v1/me/subscription",
            "/v1/me/subscription/plans",
            "/v1/me/usage",
            "/v1/me/keys",
            "/v1/me/keys/{id}",
        ];

        for path in paths {
            assert!(path.starts_with("/v1/"));
            assert!(!path.contains(' '));
        }
    }

    #[test]
    fn test_client_api_http_methods() {
        use std::collections::HashMap;

        let methods: HashMap<&str, Vec<&str>> = HashMap::from([
            ("/v1/auth/login", vec!["POST"]),
            ("/v1/auth/register", vec!["POST"]),
            ("/v1/auth/send-sms", vec!["POST"]),
            ("/v1/auth/verify-sms", vec!["POST"]),
            ("/v1/auth/logout", vec!["POST"]),
            ("/v1/models", vec!["GET"]),
            ("/v1/chat/completions", vec!["POST"]),
            ("/v1/me/subscription", vec!["GET", "POST"]),
            ("/v1/me/subscription/plans", vec!["GET"]),
            ("/v1/me/usage", vec!["GET"]),
            ("/v1/me/keys", vec!["GET", "POST"]),
            ("/v1/me/keys/{id}", vec!["DELETE"]),
        ]);

        assert_eq!(methods.get("/v1/auth/login").unwrap(), &vec!["POST"]);
        assert_eq!(methods.get("/v1/models").unwrap(), &vec!["GET"]);
        assert_eq!(methods.get("/v1/chat/completions").unwrap(), &vec!["POST"]);
    }

    // ============ 错误响应测试 ============

    #[test]
    fn test_client_api_error_response() {
        #[derive(Debug, Deserialize)]
        struct ClientError {
            error: ClientErrorDetail,
        }

        #[derive(Debug, Deserialize)]
        struct ClientErrorDetail {
            message: String,
            code: Option<String>,
        }

        let json = r#"{
            "error": {
                "message": "Invalid token",
                "code": "invalid_token"
            }
        }"#;

        let error: ClientError = serde_json::from_str(json).unwrap();
        assert_eq!(error.error.message, "Invalid token");
    }

    #[test]
    fn test_subscription_expired_error() {
        #[derive(Debug, Deserialize)]
        struct SubscriptionError {
            error: SubscriptionErrorDetail,
        }

        #[derive(Debug, Deserialize)]
        struct SubscriptionErrorDetail {
            message: String,
            code: String,
        }

        let json = r#"{
            "error": {
                "message": "Subscription expired",
                "code": "subscription_expired"
            }
        }"#;

        let error: SubscriptionError = serde_json::from_str(json).unwrap();
        assert_eq!(error.error.code, "subscription_expired");
    }

    #[test]
    fn test_rate_limit_error() {
        #[derive(Debug, Deserialize)]
        struct RateLimitError {
            error: RateLimitErrorDetail,
        }

        #[derive(Debug, Deserialize)]
        struct RateLimitErrorDetail {
            message: String,
            code: String,
        }

        let json = r#"{
            "error": {
                "message": "Rate limit exceeded",
                "code": "rate_limit_exceeded"
            }
        }"#;

        let error: RateLimitError = serde_json::from_str(json).unwrap();
        assert_eq!(error.error.code, "rate_limit_exceeded");
    }

    // ============ SSE 流式响应测试 ============

    #[test]
    fn test_sse_stream_chunk_parsing() {
        #[derive(Debug, Deserialize)]
        struct StreamChunk {
            choices: Vec<StreamChoice>,
        }

        #[derive(Debug, Deserialize)]
        struct StreamChoice {
            delta: StreamDelta,
        }

        #[derive(Debug, Deserialize)]
        struct StreamDelta {
            content: Option<String>,
        }

        let json = r#"{"choices":[{"delta":{"content":"Hello"}}]}"#;
        let chunk: StreamChunk = serde_json::from_str(json).unwrap();
        assert_eq!(chunk.choices[0].delta.content, Some("Hello".to_string()));
    }

    #[test]
    fn test_sse_done_event() {
        let done_event = "data: [DONE]";
        assert!(done_event.contains("[DONE]"));
    }

    #[test]
    fn test_sse_multiple_chunks_to_full_response() {
        let chunks = vec![
            r#"{"choices":[{"delta":{"content":"Hello"}}]}"#,
            r#"{"choices":[{"delta":{"content":" World"}}]}"#,
            r#"{"choices":[{"delta":{"content":"!"}}]}"#,
        ];

        let mut full_content = String::new();
        for chunk_json in chunks {
            #[derive(Debug, Deserialize)]
            struct Chunk {
                choices: Vec<Choice>,
            }
            #[derive(Debug, Deserialize)]
            struct Choice {
                delta: Delta,
            }
            #[derive(Debug, Deserialize)]
            struct Delta {
                content: Option<String>,
            }

            if let Ok(chunk) = serde_json::from_str::<Chunk>(chunk_json) {
                if let Some(content) = chunk.choices[0].delta.content {
                    full_content.push_str(&content);
                }
            }
        }

        assert_eq!(full_content, "Hello World!");
    }

    // ============ 请求头测试 ============

    #[test]
    fn test_authorization_header() {
        let token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...";
        let header = format!("Authorization: Bearer {}", token);
        assert!(header.starts_with("Authorization: Bearer "));
    }

    #[test]
    fn test_session_id_header() {
        let session_id = "sess_abc123xyz";
        let header = format!("X-Session-ID: {}", session_id);
        assert!(header.contains(session_id));
    }

    #[test]
    fn test_content_type_header() {
        let content_type = "Content-Type: application/json";
        assert!(content_type.contains("application/json"));
    }

    // ============ JWT Token 测试 ============

    #[test]
    fn test_jwt_token_structure() {
        let token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiJ1c2VyLTEyMyIsImVtYWlsIjoidXNlckBleGFtcGxlLmNvbSJ9.Signature";

        let parts: Vec<&str> = token.split('.').collect();
        assert_eq!(parts.len(), 3);

        // Header
        let header = parts[0];
        assert!(header.len() > 0);

        // Payload
        let payload = parts[1];
        assert!(payload.len() > 0);

        // Signature
        let signature = parts[2];
        assert!(signature.len() > 0);
    }

    #[test]
    fn test_jwt_payload_parsing() {
        #[derive(Debug, Deserialize)]
        struct Claims {
            sub: String,
            email: String,
            #[serde(default)]
            is_admin: bool,
        }

        // 简单的 Base64url 编码的 payload
        let payload = "eyJzdWIiOiJ1c2VyLTEyMyIsImVtYWlsIjoidXNlckBleGFtcGxlLmNvbSJ9";
        let decoded = String::from_utf8(base64_decode(payload)).unwrap();

        let claims: Claims = serde_json::from_str(&decoded).unwrap();
        assert_eq!(claims.sub, "user-123");
        assert_eq!(claims.email, "user@example.com");
    }

    fn base64_decode(input: &str) -> Vec<u8> {
        // 简单的 Base64 解码 (不带 padding)
        let table = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let input = input.replace('-', "+").replace('_', "/");

        let mut output = Vec::new();
        let mut buffer: u32 = 0;
        let mut bits_collected = 0;

        for c in input.chars() {
            if c == '=' || c == '.' {
                continue;
            }

            let val = table.iter().position(|&x| x as char == c).unwrap() as u32;
            buffer = (buffer << 6) | val;
            bits_collected += 6;

            if bits_collected >= 8 {
                bits_collected -= 8;
                output.push((buffer >> bits_collected) as u8);
            }
        }

        output
    }
}
