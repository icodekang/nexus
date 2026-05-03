//! 零 Token 浏览器会话账户池管理器
//!
//! 管理每个提供商的已认证浏览器会话池。
//! 会话从数据库加载并在内存中缓存以实现快速访问。
//!
//! Manages a pool of authenticated browser sessions for each provider.
//! Sessions are loaded from database and cached in memory for fast access.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use headless_chrome::Browser;

use crate::browser_emulator::{BrowserEmulatorClient, PersistedSession};
use crate::client::ProviderClient;
use crate::error::ProviderError;
use crate::types::Message;

/// Account pool entry with client and metadata
struct PoolEntry {
    client: Arc<BrowserEmulatorClient>,
    provider: String,
    is_healthy: bool,
}

/// Account Pool for managing ZeroToken browser sessions
pub struct AccountPool {
    /// Active clients keyed by account ID
    clients: Arc<RwLock<HashMap<Uuid, PoolEntry>>>,
    /// Account metadata cache (account_id -> (provider, session_data))
    accounts: Arc<RwLock<HashMap<Uuid, (String, PersistedSession)>>>,
    /// Active browser instances (for JS-based providers like DeepSeek)
    browsers: Arc<RwLock<HashMap<Uuid, Arc<Browser>>>>,
}

impl AccountPool {
    pub fn new() -> Self {
        Self {
            clients: Arc::new(RwLock::new(HashMap::new())),
            accounts: Arc::new(RwLock::new(HashMap::new())),
            browsers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a browser instance for JS-based chat (DeepSeek)
    pub async fn register_browser(&self, account_id: Uuid, browser: Arc<Browser>) {
        let mut browsers = self.browsers.write().await;
        browsers.insert(account_id, browser);
    }

    /// Get browser for an account
    pub async fn get_browser(&self, account_id: Uuid) -> Option<Arc<Browser>> {
        let browsers = self.browsers.read().await;
        browsers.get(&account_id).cloned()
    }

    /// Unregister a browser instance
    pub async fn unregister_browser(&self, account_id: Uuid) {
        let mut browsers = self.browsers.write().await;
        browsers.remove(&account_id);
    }

    /// Execute JS-based chat for providers like DeepSeek that require browser execution
    ///
    /// Returns the response content on success
    pub async fn execute_browser_chat(
        &self,
        provider: &str,
        messages: Vec<Message>,
        model: &str,
    ) -> Result<String, ProviderError> {
        let browsers = self.browsers.read().await;
        let account_id = {
            let accounts = self.accounts.read().await;
            accounts.iter()
                .find(|(_, (p, _))| p == provider)
                .map(|(id, _)| *id)
        };

        let account_id = account_id.ok_or_else(|| ProviderError::InternalError(
            format!("No browser account registered for provider: {}", provider)
        ))?;

        tracing::debug!("execute_browser_chat: found account_id {:?} for provider {}", account_id, provider);

        let browser = browsers.get(&account_id)
            .ok_or_else(|| ProviderError::InternalError(
                format!("Browser not found for account: {}", account_id)
            ))?;

        tracing::debug!("execute_browser_chat: got browser for account {:?}", account_id);

        // Get the first tab
        let tab = {
            let tabs = browser.get_tabs();
            let tabs_guard = tabs.lock().map_err(|_| ProviderError::InternalError("Failed to lock tabs".to_string()))?;
            tabs_guard.first()
                .ok_or_else(|| ProviderError::InternalError("No tab found".to_string()))?
                .clone()
        };

        // Execute JS chat
        Self::execute_js_chat_on_tab(&tab, messages, model).await
    }

    async fn execute_js_chat_on_tab(
        tab: &headless_chrome::Tab,
        messages: Vec<Message>,
        model: &str,
    ) -> Result<String, ProviderError> {
        use std::collections::HashMap;

        let messages_json = serde_json::to_string(&messages)
            .map_err(|e| ProviderError::InternalError(format!("Failed to serialize messages: {}", e)))?;

        let chat_script = format!(r#"
            (async () => {{
                const messages = {messages_json};
                const model = "{model}";

                const debug = {{}};

                // Find the chat input - try multiple selectors
                const selectors = [
                    'textarea[name="prompt"]',
                    'textarea[placeholder*="Message"]',
                    'textarea[placeholder*="Search"]',
                    'textarea',
                    '[data-node-id][contenteditable="true"]',
                    '[role="textbox"]'
                ];

                let textarea = null;
                for (const sel of selectors) {{
                    const el = document.querySelector(sel);
                    if (el) {{ textarea = el; debug.found = sel; break; }}
                }}

                if (!textarea) {{
                    const bodyText = document.body ? document.body.innerText.substring(0, 500) : 'no body';
                    const allTextareas = Array.from(document.querySelectorAll('textarea')).map(el => el.outerHTML).join('\n');
                    debug.noTextarea = true;
                    debug.bodyText = bodyText.substring(0, 200);
                    debug.allTextareas = allTextareas.substring(0, 500);
                    return JSON.stringify({{ error: "no textarea", debug }});
                }}

                debug.textareaFound = textarea.outerHTML.substring(0, 200);

                // Build prompt from messages
                const prompt = messages.map(m => m.role + ": " + m.content).join('\n');
                debug.promptLength = prompt.length;

                // Type into input
                textarea.focus();
                textarea.value = prompt;
                textarea.dispatchEvent(new Event('input', {{ bubbles: true }}));

                await new Promise(r => setTimeout(r, 500));

                // Find send button
                const allButtons = Array.from(document.querySelectorAll('button'));
                debug.buttons = allButtons.map(b => b.textContent.trim()).slice(0, 10);

                let sendBtn = allButtons.find(btn =>
                    btn.textContent.includes('Send') ||
                    btn.textContent.includes('发送') ||
                    btn.querySelector('svg')
                );

                if (sendBtn) {{
                    debug.clickingButton = sendBtn.textContent.trim();
                    sendBtn.click();
                }} else {{
                    debug.pressingEnter = true;
                    textarea.dispatchEvent(new KeyboardEvent('keydown', {{ key: 'Enter', code: 'Enter', keyCode: 13, which: 13, bubbles: true }}));
                }}

                // Wait for response
                await new Promise(r => setTimeout(r, 8000));

                // Get last response
                const responseDivs = document.querySelectorAll('[data-node-id]');
                debug.responseDivsCount = responseDivs.length;

                if (responseDivs.length > 0) {{
                    const lastResponse = responseDivs[responseDivs.length - 1];
                    return JSON.stringify({{ content: lastResponse.textContent.substring(0, 1000), debug }});
                }}

                return JSON.stringify({{ error: "no response", debug }});
            }})();
        "#);

        tracing::debug!("execute_js_chat_on_tab: executing JS script");

        let result = tab.evaluate(&chat_script, false)
            .map_err(|e| ProviderError::InternalError(format!("Failed to execute chat JS: {}", e)))?;

        let response_text = match &result.value {
            Some(serde_json::Value::String(s)) => s.clone(),
            Some(v) => v.to_string(),
            None => {
                tracing::warn!("tab.evaluate returned no value");
                "{\"error\":\"No response\"}".to_string()
            }
        };

        tracing::debug!("JS chat response_text: {}", response_text);

        let response_json: HashMap<String, serde_json::Value> = serde_json::from_str(&response_text)
            .map_err(|e| ProviderError::InvalidResponse(format!("Failed to parse response: {} (text: {})", e, response_text)))?;

        if let Some(content) = response_json.get("content").and_then(|v| v.as_str()) {
            Ok(content.to_string())
        } else if let Some(err) = response_json.get("error").and_then(|v| v.as_str()) {
            Err(ProviderError::InvalidResponse(format!("JS chat error: {}", err)))
        } else {
            Err(ProviderError::InvalidResponse("No content or error in response".to_string()))
        }
    }

    /// Register an account with its session data
    pub async fn register_account(&self, account_id: Uuid, provider: String, session_data: PersistedSession) -> Result<(), ProviderError> {
        // Create client
        let client = Arc::new(BrowserEmulatorClient::new(&provider)?);

        // Restore session
        client.restore_session(&session_data).await?;

        // Store entry
        let entry = PoolEntry {
            client: client.clone(),
            provider: provider.clone(),
            is_healthy: true,
        };

        let mut clients = self.clients.write().await;
        clients.insert(account_id, entry);

        // Store account metadata
        let mut accounts = self.accounts.write().await;
        accounts.insert(account_id, (provider, session_data));

        Ok(())
    }

    /// Unregister an account
    pub async fn unregister_account(&self, account_id: Uuid) {
        let mut clients = self.clients.write().await;
        clients.remove(&account_id);

        let mut accounts = self.accounts.write().await;
        accounts.remove(&account_id);
    }

    /// Get an available client for a provider (load-balanced)
    pub async fn get_client(&self, provider: &str) -> Option<Arc<BrowserEmulatorClient>> {
        let clients = self.clients.read().await;

        // Find all healthy accounts for this provider with lowest request count
        let mut candidates: Vec<_> = clients
            .iter()
            .filter(|(_, entry)| entry.provider == provider && entry.is_healthy)
            .collect();

        if candidates.is_empty() {
            return None;
        }

        // Sort by request count (simple load balancing)
        // In a real implementation, we'd track request counts per account
        candidates.sort_by(|a, b| {
            let count_a = a.1.client.key_id().is_some(); // Placeholder for request count
            let count_b = b.1.client.key_id().is_some();
            count_a.cmp(&count_b)
        });

        Some(candidates[0].1.client.clone())
    }

    /// Mark an account as unhealthy (will be skipped until revived)
    pub async fn mark_unhealthy(&self, account_id: Uuid) {
        let mut clients = self.clients.write().await;
        if let Some(entry) = clients.get_mut(&account_id) {
            entry.is_healthy = false;
        }
    }

    /// Revive an account (mark as healthy again)
    pub async fn revive_account(&self, account_id: Uuid) {
        let mut clients = self.clients.write().await;
        if let Some(entry) = clients.get_mut(&account_id) {
            entry.is_healthy = true;
        }
    }

    /// Get account info
    pub async fn get_account_info(&self, account_id: Uuid) -> Option<(String, bool)> {
        let clients = self.clients.read().await;
        clients.get(&account_id).map(|e| (e.provider.clone(), e.is_healthy))
    }

    /// List all registered account IDs
    pub async fn list_accounts(&self) -> Vec<Uuid> {
        let clients = self.clients.read().await;
        clients.keys().cloned().collect()
    }

    /// Check if an account exists
    pub async fn has_account(&self, account_id: Uuid) -> bool {
        let clients = self.clients.read().await;
        clients.contains_key(&account_id)
    }
}

impl Default for AccountPool {
    fn default() -> Self {
        Self::new()
    }
}
