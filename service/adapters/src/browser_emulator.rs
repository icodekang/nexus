//! 浏览器模拟器提供商客户端
//!
//! 模拟浏览器会话以访问 LLM 网页界面，无需 API Key。
//! 这实现了对 Claude.ai 和 ChatGPT 等模型的"零 Token"访问。
//!
//! # 工作原理
//! 1. 维护浏览器会话状态（Cookie、localStorage）
//! 2. 发送看起来像来自真实浏览器的 HTTP 请求
//! 3. 解析网页界面的响应
//! 4. 通过 SSE 或模拟 WebSocket 处理流式响应
//!
//! Simulates browser sessions to access LLM web interfaces without API keys.
//! This enables "zero-token" access to models like Claude.ai and ChatGPT.
//!
//! # How it works
//! 1. Maintains browser session state (cookies, localStorage)
//! 2. Makes HTTP requests that appear to come from a real browser
//! 3. Parses responses from the web interface
//! 4. Handles streaming via SSE or WebSocket simulation

use crate::error::ProviderError;
use crate::types::{
    ChatRequest, ChatResponse, EmbeddingsRequest, EmbeddingsResponse, Message,
    ProviderType,
};
use async_trait::async_trait;
use futures_util::StreamExt;
use reqwest::Client;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

/// Persisted session data for storage (JSON-serializable)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedSession {
    pub cookies: HashMap<String, String>,
    pub auth_tokens: HashMap<String, String>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

use serde::{Deserialize, Serialize};

/// Browser session state
#[derive(Debug, Clone)]
pub struct BrowserSession {
    /// Session cookies
    pub cookies: HashMap<String, String>,
    /// Session tokens (Authorization headers)
    pub auth_tokens: HashMap<String, String>,
    /// Session expiry
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Associated user ID (for tracking)
    pub user_id: Option<uuid::Uuid>,
}

impl BrowserSession {
    pub fn new() -> Self {
        Self {
            cookies: HashMap::new(),
            auth_tokens: HashMap::new(),
            expires_at: None,
            user_id: None,
        }
    }

    pub fn with_user_id(mut self, user_id: uuid::Uuid) -> Self {
        self.user_id = Some(user_id);
        self
    }

    pub fn is_valid(&self) -> bool {
        if let Some(expires) = self.expires_at {
            return chrono::Utc::now() < expires;
        }
        true // No expiry means valid until invalidated
    }
}

impl Default for BrowserSession {
    fn default() -> Self {
        Self::new()
    }
}

/// Browser Emulator Client for accessing LLMs via web interface
pub struct BrowserEmulatorClient {
    /// Target LLM provider (claude, chatgpt)
    provider: String,
    /// HTTP client with browser-like configuration
    client: Client,
    /// Session state
    session: Arc<RwLock<BrowserSession>>,
    /// Base URL for the web interface
    base_url: String,
}

impl BrowserEmulatorClient {
    pub fn new(provider: &str) -> Result<Self, ProviderError> {
        let base_url = match provider {
            "claude" => "https://claude.ai",
            "chatgpt" => "https://chat.openai.com",
            "deepseek" => "https://chat.deepseek.com",
            _ => return Err(ProviderError::ProviderNotFound(provider.to_string())),
        };

        // Create HTTP client with browser-like headers
        let client = Client::builder()
            .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .build()
            .map_err(|e| ProviderError::RequestFailed(e))?;

        Ok(Self {
            provider: provider.to_string(),
            client,
            session: Arc::new(RwLock::new(BrowserSession::new())),
            base_url: base_url.to_string(),
        })
    }

    /// Create a new session for a user
    pub async fn create_session(&self, user_id: uuid::Uuid) -> Result<(), ProviderError> {
        let mut session = self.session.write().await;
        *session = BrowserSession::new().with_user_id(user_id);

        // Initialize session with the web interface
        self.init_session(&mut session).await?;

        Ok(())
    }

    /// Initialize session by hitting the main page to get cookies
    async fn init_session(&self, session: &mut BrowserSession) -> Result<(), ProviderError> {
        let resp = self.client
            .get(&self.base_url)
            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8")
            .header("Accept-Language", "en-US,en;q=0.9")
            .header("Accept-Encoding", "gzip, deflate, br")
            .header("Connection", "keep-alive")
            .header("Upgrade-Insecure-Requests", "1")
            .send()
            .await
            .map_err(|e| ProviderError::RequestFailed(e))?;

        if !resp.status().is_success() && resp.status().as_u16() != 403 {
            return Err(ProviderError::InvalidResponse(
                format!("Failed to initialize session: {}", resp.status())
            ));
        }

        // Extract cookies from headers (simple approach without cookie_store)
        if let Some(set_cookie) = resp.headers().get("set-cookie") {
            if let Ok(set_cookie_str) = set_cookie.to_str() {
                // Parse simple cookie format
                if let Some((name, value)) = set_cookie_str.split_once('=') {
                    let value = value.split(';').next().unwrap_or(value);
                    session.cookies.insert(name.to_string(), value.to_string());
                }
            }
        }

        Ok(())
    }

    /// Send a chat message via browser session
    pub async fn chat(&self, request: ChatRequest) -> Result<ChatResponse, ProviderError> {
        let session = self.session.read().await;
        if !session.is_valid() {
            return Err(ProviderError::AuthenticationError(
                "Browser session expired or invalid".to_string()
            ));
        }

        let start = Instant::now();
        let response = self.send_chat_request(&request, &session).await?;
        let latency_ms = start.elapsed().as_millis() as i32;

        self.parse_chat_response(response, request.model, latency_ms)
    }

    /// Send a streaming chat message via browser session
    pub async fn chat_stream(&self, request: ChatRequest) -> Result<Vec<crate::client::ChatChunk>, ProviderError> {
        let session = self.session.read().await;
        if !session.is_valid() {
            return Err(ProviderError::AuthenticationError(
                "Browser session expired or invalid".to_string()
            ));
        }

        self.send_chat_stream_request(&request, &session).await
    }

    /// Build the chat API URL for the provider
    fn build_chat_url(&self) -> String {
        match self.provider.as_str() {
            "deepseek" => format!("{}/api/v0/chat/completion", self.base_url),
            "claude" => format!("{}/api/chat", self.base_url),
            "chatgpt" => format!("{}/api/chat", self.base_url),
            _ => format!("{}/chat", self.base_url),
        }
    }

    /// Build the request payload specific to each provider's web API
    fn build_chat_payload(&self, request: &ChatRequest) -> serde_json::Value {
        match self.provider.as_str() {
            "deepseek" => {
                let prompt = request.messages.iter()
                    .map(|m| format!("{}: {}", m.role, m.content))
                    .collect::<Vec<_>>()
                    .join("\n");
                serde_json::json!({
                    "chat_session_id": uuid::Uuid::new_v4().to_string(),
                    "parent_message_id": null,
                    "prompt": prompt,
                    "ref_file_ids": [],
                    "thinking_enabled": false,
                    "search_enabled": false,
                })
            }
            _ => {
                let mut payload = serde_json::json!({
                    "model": request.model,
                    "messages": request.messages,
                    "temperature": request.temperature,
                });
                if let Some(max_tokens) = request.max_tokens {
                    payload["max_tokens"] = serde_json::json!(max_tokens);
                }
                payload
            }
        }
    }

    /// Send chat request via browser session
    async fn send_chat_request(
        &self,
        request: &ChatRequest,
        session: &BrowserSession,
    ) -> Result<serde_json::Value, ProviderError> {
        let url = self.build_chat_url();
        let payload = self.build_chat_payload(request);

        let mut req = self.client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json");

        // Add cookies
        let cookie_str: String = session.cookies.iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("; ");
        if !cookie_str.is_empty() {
            req = req.header("Cookie", cookie_str);
        }

        // Add auth token if available
        if let Some(token) = session.auth_tokens.get(&self.provider) {
            req = req.header("Authorization", format!("Bearer {}", token));
        }

        let resp = req
            .json(&payload)
            .send()
            .await
            .map_err(|e| ProviderError::RequestFailed(e))?;

        if resp.status() == 401 || resp.status() == 403 {
            return Err(ProviderError::AuthenticationError(
                "Browser session authentication failed".to_string()
            ));
        }

        resp.json().await.map_err(|e| ProviderError::InvalidResponse(e.to_string()))
    }

    /// Send streaming chat request
    async fn send_chat_stream_request(
        &self,
        request: &ChatRequest,
        session: &BrowserSession,
    ) -> Result<Vec<crate::client::ChatChunk>, ProviderError> {
        let url = self.build_chat_url();

        let mut payload = self.build_chat_payload(request);
        if let Some(obj) = payload.as_object_mut() {
            obj.insert("stream".to_string(), serde_json::json!(true));
        }

        let mut req = self.client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("Accept", "text/event-stream")
            .header("Cache-Control", "no-cache")
            .header("Connection", "keep-alive");

        // Add cookies
        let cookie_str: String = session.cookies.iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("; ");
        if !cookie_str.is_empty() {
            req = req.header("Cookie", cookie_str);
        }

        // Add auth token if available
        if let Some(token) = session.auth_tokens.get(&self.provider) {
            req = req.header("Authorization", format!("Bearer {}", token));
        }

        let resp = req
            .json(&payload)
            .send()
            .await
            .map_err(|e| ProviderError::RequestFailed(e))?;

        if resp.status() == 401 || resp.status() == 403 {
            return Err(ProviderError::AuthenticationError(
                "Browser session authentication failed".to_string()
            ));
        }

        // Parse SSE stream
        let mut chunks = Vec::new();
        let mut stream = resp.bytes_stream();

        while let Some(item) = stream.next().await {
            let bytes = item.map_err(|e| ProviderError::RequestFailed(e))?;
            let text = String::from_utf8(bytes.to_vec())
                .map_err(|e| ProviderError::InvalidResponse(e.to_string()))?;

            for line in text.lines() {
                if line.starts_with("data: ") {
                    let data = &line[6..];
                    if data == "[DONE]" {
                        chunks.push(crate::client::ChatChunk {
                            delta: String::new(),
                            finished: true,
                            finish_reason: Some("stop".to_string()),
                        });
                        return Ok(chunks);
                    }

                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(data) {
                        let (delta, finished, reason) = match self.provider.as_str() {
                            "deepseek" => {
                                let delta = json["choices"]
                                    .get(0)
                                    .and_then(|c| c.get("delta"))
                                    .and_then(|d| d.get("content"))
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("")
                                    .to_string();
                                let reason = json["choices"]
                                    .get(0)
                                    .and_then(|c| c.get("finish_reason"))
                                    .and_then(|v| v.as_str())
                                    .map(|s| s.to_string());
                                let finished = reason.is_some();
                                (delta, finished, reason)
                            }
                            _ => {
                                let delta = json["delta"]
                                    .as_str()
                                    .unwrap_or("")
                                    .to_string();
                                let finished = json["finished"].as_bool().unwrap_or(false);
                                let reason = json["finish_reason"].as_str().map(|s| s.to_string());
                                (delta, finished, reason)
                            }
                        };

                        chunks.push(crate::client::ChatChunk {
                            delta,
                            finished,
                            finish_reason: reason,
                        });

                        if finished {
                            return Ok(chunks);
                        }
                    }
                }
            }
        }

        Ok(chunks)
    }

    /// Parse chat response into ChatResponse
    fn parse_chat_response(
        &self,
        data: serde_json::Value,
        model: String,
        latency_ms: i32,
    ) -> Result<ChatResponse, ProviderError> {
        let (id, content) = match self.provider.as_str() {
            "claude" => {
                let id = data["id"].as_str().unwrap_or("unknown").to_string();
                let content = data["content"]
                    .as_str()
                    .unwrap_or("")
                    .to_string();
                (id, content)
            }
            "chatgpt" => {
                let id = data["id"].as_str().unwrap_or("unknown").to_string();
                let content = data["message"]
                    .as_str()
                    .unwrap_or("")
                    .to_string();
                (id, content)
            }
            "deepseek" => {
                let id = data["id"].as_str().unwrap_or("unknown").to_string();
                let content = data["choices"]
                    .get(0)
                    .and_then(|c| c.get("message"))
                    .and_then(|m| m.get("content"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                (id, content)
            }
            _ => {
                return Err(ProviderError::InvalidResponse(
                    "Unknown browser provider".to_string()
                ));
            }
        };

        let usage = data.get("usage").cloned().unwrap_or_default();
        let prompt_tokens = usage.get("prompt_tokens").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
        let completion_tokens = usage.get("completion_tokens").and_then(|v| v.as_i64()).unwrap_or(0) as i32;

        Ok(ChatResponse {
            id,
            model,
            message: Message {
                role: "assistant".to_string(),
                content,
            },
            usage: [
                ("prompt_tokens".to_string(), prompt_tokens),
                ("completion_tokens".to_string(), completion_tokens),
                ("total_tokens".to_string(), prompt_tokens + completion_tokens),
            ]
            .into_iter()
            .collect(),
            latency_ms,
        })
    }

    /// Restore session from persisted data
    pub async fn restore_session(&self, persisted: &PersistedSession) -> Result<(), ProviderError> {
        let mut session = self.session.write().await;
        session.cookies = persisted.cookies.clone();
        session.auth_tokens = persisted.auth_tokens.clone();
        session.expires_at = persisted.expires_at;
        Ok(())
    }

    /// Export session for storage
    pub async fn export_session(&self) -> Result<PersistedSession, ProviderError> {
        let session = self.session.read().await;
        Ok(PersistedSession {
            cookies: session.cookies.clone(),
            auth_tokens: session.auth_tokens.clone(),
            expires_at: session.expires_at,
        })
    }

    /// Get the login URL for the provider
    pub fn get_login_url(&self, callback_url: &str) -> String {
        match self.provider.as_str() {
            "claude" => format!(
                "https://claude.ai/login?return_to={}",
                urlencoding::encode(callback_url)
            ),
            "chatgpt" => format!(
                "https://chat.openai.com/auth/login?next={}",
                urlencoding::encode(callback_url)
            ),
            "deepseek" => format!(
                "https://chat.deepseek.com/login?return_to={}",
                urlencoding::encode(callback_url)
            ),
            _ => String::new(),
        }
    }

    /// Test if session is still valid
    pub async fn is_session_valid(&self) -> bool {
        let session = self.session.read().await;
        session.is_valid()
    }

    /// Get the current session state (for debugging)
    pub async fn get_session_debug_info(&self) -> PersistedSession {
        let session = self.session.read().await;
        PersistedSession {
            cookies: session.cookies.clone(),
            auth_tokens: session.auth_tokens.clone(),
            expires_at: session.expires_at,
        }
    }
}

#[async_trait]
impl crate::client::ProviderClient for BrowserEmulatorClient {
    fn provider_type(&self) -> ProviderType {
        // Use OpenAI as the base type for compatibility
        ProviderType::OpenAI
    }

    fn provider_id(&self) -> &str {
        &self.provider
    }

    fn key_id(&self) -> Option<uuid::Uuid> {
        None // Browser emulator doesn't use API keys
    }

    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse, ProviderError> {
        self.chat(request).await
    }

    async fn chat_stream(
        &self,
        request: ChatRequest,
    ) -> Result<Vec<crate::client::ChatChunk>, ProviderError> {
        self.chat_stream(request).await
    }

    async fn embeddings(
        &self,
        _request: EmbeddingsRequest,
    ) -> Result<EmbeddingsResponse, ProviderError> {
        Err(ProviderError::EmbeddingsNotSupported)
    }
}

/// Browser Emulator Factory for creating provider clients
pub struct BrowserEmulatorFactory;

impl BrowserEmulatorFactory {
    /// Create a browser emulator client for the specified provider
    pub fn create(provider: &str) -> Result<Arc<dyn crate::client::ProviderClient>, ProviderError> {
        Ok(Arc::new(BrowserEmulatorClient::new(provider)?))
    }

    /// List supported browser emulator providers
    pub fn supported_providers() -> Vec<&'static str> {
        vec!["claude", "chatgpt", "deepseek"]
    }
}