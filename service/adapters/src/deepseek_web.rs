use crate::browser_emulator::BrowserSession;
use crate::client::{ChatChunk, ProviderClient};
use crate::error::ProviderError;
use crate::types::{ChatRequest, ChatResponse, EmbeddingsRequest, EmbeddingsResponse, Message, ProviderType};
use async_trait::async_trait;
use futures_util::StreamExt;
use reqwest::Client;
use sha2::{Digest, Sha256};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use uuid::Uuid;

const DEEPSEEK_API_BASE: &str = "https://chat.deepseek.com/api/v0";
const DEEPSEEK_ORIGIN: &str = "https://chat.deepseek.com";

pub struct DeepSeekWebClient {
    client: Client,
    session: Arc<RwLock<BrowserSession>>,
}

impl DeepSeekWebClient {
    pub fn new() -> Result<Self, ProviderError> {
        let client = Client::builder()
            .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36")
            .build()
            .map_err(|e| ProviderError::RequestFailed(e))?;

        Ok(Self {
            client,
            session: Arc::new(RwLock::new(BrowserSession::new())),
        })
    }

    pub async fn restore_session(&self, cookies: &std::collections::HashMap<String, String>, auth_tokens: &std::collections::HashMap<String, String>) {
        let mut session = self.session.write().await;
        session.cookies = cookies.clone();
        session.auth_tokens = auth_tokens.clone();
    }

    pub async fn export_session(&self) -> crate::browser_emulator::PersistedSession {
        let session = self.session.read().await;
        crate::browser_emulator::PersistedSession {
            cookies: session.cookies.clone(),
            auth_tokens: session.auth_tokens.clone(),
            expires_at: session.expires_at,
        }
    }

    async fn fetch_pow_challenge(&self) -> Result<PowChallenge, ProviderError> {
        let resp = self
            .client
            .post(format!("{}/chat/create_pow_challenge", DEEPSEEK_API_BASE))
            .header("Origin", DEEPSEEK_ORIGIN)
            .header("Referer", format!("{}/", DEEPSEEK_ORIGIN))
            .header("Accept", "*/*")
            .send()
            .await
            .map_err(|e| ProviderError::RequestFailed(e))?;

        let json: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| ProviderError::InvalidResponse(format!("PoW challenge parse: {}", e)))?;

        let data = json.get("data").ok_or_else(|| {
            ProviderError::InvalidResponse("Missing data in PoW challenge".into())
        })?;

        let challenge = data["biz_data"]["challenge"]
            .as_str()
            .unwrap_or("")
            .to_string();
        let algorithm = data["biz_data"]["algorithm"]
            .as_str()
            .unwrap_or("sha256")
            .to_string();
        let target_path = data["biz_data"]["target_path"]
            .as_str()
            .unwrap_or("")
            .to_string();

        Ok(PowChallenge {
            challenge,
            algorithm,
            target_path,
        })
    }

    fn solve_pow(challenge: &PowChallenge) -> PowSolution {
        match challenge.algorithm.as_str() {
            "DeepSeekHashV1" => solve_deepseek_hash_v1(&challenge.challenge),
            _ => solve_sha256_pow(&challenge.challenge),
        }
    }

    async fn solve_and_attach(&self) -> Result<String, ProviderError> {
        let challenge = self.fetch_pow_challenge().await?;
        let solution = Self::solve_pow(&challenge);
        let answer = serde_json::json!({
            "algorithm": challenge.algorithm,
            "challenge": challenge.challenge,
            "answer": solution.answer,
            "nonce": solution.nonce,
            "signature": solution.signature,
        });

        Ok(serde_json::to_string(&answer).unwrap_or_default())
    }

    pub async fn chat_internal(
        &self,
        request: ChatRequest,
    ) -> Result<ChatResponse, ProviderError> {
        let session = self.session.read().await;
        let start = Instant::now();

        let pow_header = self.solve_and_attach().await?;

        let payload = build_chat_payload(&request);

        let url = format!("{}/chat/completions", DEEPSEEK_API_BASE);

        let mut req = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .header("Origin", DEEPSEEK_ORIGIN)
            .header("Referer", format!("{}/", DEEPSEEK_ORIGIN))
            .header("x-ds-pow-response", &pow_header);

        if let Some(token) = session.auth_tokens.get("deepseek") {
            req = req.header("Authorization", format!("Bearer {}", token));
        }

        let cookie_str: String = session
            .cookies
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("; ");
        if !cookie_str.is_empty() {
            req = req.header("Cookie", &cookie_str);
        }

        let resp = req.json(&payload).send().await?;

        if resp.status() == 401 || resp.status() == 403 {
            return Err(ProviderError::AuthenticationError(
                "DeepSeek session expired".into(),
            ));
        }

        let data: serde_json::Value = resp.json().await?;

        let id = data["id"].as_str().unwrap_or("unknown").to_string();
        let content = data["choices"]
            .get(0)
            .and_then(|c| c.get("message"))
            .and_then(|m| m.get("content"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let usage = data["usage"].clone();
        let prompt_tokens = usage["prompt_tokens"].as_i64().unwrap_or(0) as i32;
        let completion_tokens = usage["completion_tokens"].as_i64().unwrap_or(0) as i32;

        Ok(ChatResponse {
            id,
            model: request.model,
            message: Message::assistant(content),
            usage: [
                ("prompt_tokens".into(), prompt_tokens),
                ("completion_tokens".into(), completion_tokens),
                ("total_tokens".into(), prompt_tokens + completion_tokens),
            ]
            .into(),
            latency_ms: start.elapsed().as_millis() as i32,
        })
    }

    pub async fn chat_stream_internal(
        &self,
        request: ChatRequest,
    ) -> Result<Vec<ChatChunk>, ProviderError> {
        let session = self.session.read().await;

        let pow_header = self.solve_and_attach().await?;

        let mut payload = build_chat_payload(&request);
        payload["stream"] = serde_json::json!(true);

        let url = format!("{}/chat/completions", DEEPSEEK_API_BASE);

        let mut req = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("Accept", "text/event-stream")
            .header("Origin", DEEPSEEK_ORIGIN)
            .header("Referer", format!("{}/", DEEPSEEK_ORIGIN))
            .header("x-ds-pow-response", &pow_header);

        if let Some(token) = session.auth_tokens.get("deepseek") {
            req = req.header("Authorization", format!("Bearer {}", token));
        }

        let cookie_str: String = session
            .cookies
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("; ");
        if !cookie_str.is_empty() {
            req = req.header("Cookie", &cookie_str);
        }

        let resp = req.json(&payload).send().await?;

        if resp.status() == 401 || resp.status() == 403 {
            return Err(ProviderError::AuthenticationError(
                "DeepSeek session expired".into(),
            ));
        }

        let mut chunks = Vec::new();
        let mut stream = resp.bytes_stream();

        while let Some(item) = stream.next().await {
            let bytes = item.map_err(|e| ProviderError::RequestFailed(e))?;
            let text = String::from_utf8(bytes.to_vec())
                .map_err(|e| ProviderError::InvalidResponse(e.to_string()))?;

            for line in text.lines() {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }

                if let Some(data) = line.strip_prefix("data: ") {
                    if data == "[DONE]" {
                        chunks.push(ChatChunk {
                            delta: String::new(),
                            finished: true,
                            finish_reason: Some("stop".into()),
                        });
                        return Ok(chunks);
                    }

                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(data) {
                        let delta = json["choices"]
                            .get(0)
                            .and_then(|c| c.get("delta"))
                            .and_then(|d| d.get("content"))
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();

                        let finish_reason = json["choices"]
                            .get(0)
                            .and_then(|c| c.get("finish_reason"))
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());

                        let finished = finish_reason.is_some();

                        chunks.push(ChatChunk {
                            delta,
                            finished,
                            finish_reason,
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
}

fn build_chat_payload(request: &ChatRequest) -> serde_json::Value {
    serde_json::json!({
        "model": request.model,
        "messages": request.messages,
        "temperature": request.temperature,
        "max_tokens": request.max_tokens.unwrap_or(4096),
        "stream": false,
    })
}

struct PowChallenge {
    challenge: String,
    algorithm: String,
    #[allow(dead_code)]
    target_path: String,
}

struct PowSolution {
    answer: String,
    nonce: String,
    signature: String,
}

fn solve_sha256_pow(challenge: &str) -> PowSolution {
    for nonce in 0u64..100_000_000 {
        let input = format!("{}{}", challenge, nonce);
        let hash = Sha256::digest(input.as_bytes());
        let hex = format!("{:x}", hash);

        if hex.starts_with("000000") {
            return PowSolution {
                answer: hex.clone(),
                nonce: nonce.to_string(),
                signature: hex,
            };
        }
    }

    PowSolution {
        answer: String::new(),
        nonce: "0".into(),
        signature: String::new(),
    }
}

fn solve_deepseek_hash_v1(challenge: &str) -> PowSolution {
    let mut nonce = 0u64;
    loop {
        let input = format!("{}{}", challenge, nonce);
        let hash = Sha256::digest(input.as_bytes());
        let hex = format!("{:x}", hash);

        if hex.starts_with("000000") {
            return PowSolution {
                answer: hex.clone(),
                nonce: nonce.to_string(),
                signature: hex,
            };
        }

        nonce += 1;
        if nonce > 10_000_000 {
            break;
        }
    }

    PowSolution {
        answer: String::new(),
        nonce: "0".into(),
        signature: String::new(),
    }
}

#[async_trait]
impl ProviderClient for DeepSeekWebClient {
    fn provider_type(&self) -> ProviderType {
        ProviderType::DeepSeek
    }

    fn provider_id(&self) -> &str {
        "deepseek-web"
    }

    fn key_id(&self) -> Option<Uuid> {
        None
    }

    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse, ProviderError> {
        self.chat_internal(request).await
    }

    async fn chat_stream(
        &self,
        request: ChatRequest,
    ) -> Result<Vec<ChatChunk>, ProviderError> {
        self.chat_stream_internal(request).await
    }

    async fn embeddings(
        &self,
        _request: EmbeddingsRequest,
    ) -> Result<EmbeddingsResponse, ProviderError> {
        Err(ProviderError::EmbeddingsNotSupported)
    }
}
