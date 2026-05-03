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

        // Navigate to DeepSeek chat page and wait for it to load with cookies
        let _ = tab.navigate_to("https://chat.deepseek.com/");
        tracing::debug!("Navigated to DeepSeek chat page");
        std::thread::sleep(std::time::Duration::from_secs(5));

        let messages_json = serde_json::to_string(&messages)
            .map_err(|e| ProviderError::InternalError(format!("Failed to serialize messages: {}", e)))?;

        let chat_script = format!(r#"
            (async () => {{
                const messages = {messages_json};
                const model = "{model}";
                const debug = {{}};

                debug.url = window.location.href;
                debug.title = document.title;

                // Check if we're on a login page
                const bodyText = (document.body?.innerText || '').substring(0, 500);
                debug.bodyPreview = bodyText.substring(0, 200);

                if (bodyText.includes('Sign In') || bodyText.includes('Log in') || bodyText.includes('登录') || bodyText.includes('sign_up')) {{
                    return JSON.stringify({{ error: "on login page, not authenticated", debug }});
                }}

                // React inputs need nativeInputValueSetter to trigger React's onChange
                const nativeInputValueSetter = Object.getOwnPropertyDescriptor(
                    window.HTMLTextAreaElement.prototype, 'value'
                ).set;

                const selectors = [
                    'textarea[placeholder*="message" i]',
                    'textarea[placeholder*="any" i]',
                    'textarea',
                    '[contenteditable="true"]',
                    '[role="textbox"]'
                ];

                let inputEl = null;
                for (const sel of selectors) {{
                    const el = document.querySelector(sel);
                    if (el) {{ inputEl = el; debug.inputSelector = sel; break; }}
                }}

                if (!inputEl) {{
                    debug.allInputs = Array.from(document.querySelectorAll('textarea, [contenteditable], [role="textbox"]'))
                        .map(el => el.outerHTML.substring(0, 150)).join('\\n');
                    return JSON.stringify({{ error: "no textarea found on page", debug }});
                }}

                debug.inputTag = inputEl.tagName;

                // Build the prompt from messages
                const prompt = messages.map(m => m.role + ": " + m.content).join('\\n');
                debug.promptLength = prompt.length;

                // Focus and set value using native setter (works with React)
                inputEl.focus();
                if (inputEl.tagName === 'TEXTAREA') {{
                    nativeInputValueSetter.call(inputEl, prompt);
                }} else {{
                    inputEl.textContent = prompt;
                }}
                inputEl.dispatchEvent(new Event('input', {{ bubbles: true }}));
                inputEl.dispatchEvent(new Event('change', {{ bubbles: true }}));

                await new Promise(r => setTimeout(r, 800));

                // Find and click the send button
                const buttons = Array.from(document.querySelectorAll('button, [role="button"]'));
                debug.buttonCount = buttons.length;
                debug.buttonTexts = buttons.map(b => b.textContent.trim()).slice(0, 8);

                const sendBtn = buttons.find(btn => {{
                    const text = btn.textContent.trim().toLowerCase();
                    return text === 'send' || text === '发送' ||
                           (btn.querySelector('svg') && !btn.closest('nav, header'));
                }});

                if (sendBtn) {{
                    debug.sendMethod = 'click: ' + sendBtn.textContent.trim();
                    sendBtn.click();
                }} else {{
                    debug.sendMethod = 'enter key';
                    inputEl.dispatchEvent(new KeyboardEvent('keydown', {{
                        key: 'Enter', code: 'Enter', keyCode: 13, which: 13, bubbles: true
                    }}));
                }}

                // Wait for response (DeepSeek can be slow)
                await new Promise(r => setTimeout(r, 15000));

                // Try multiple selectors for response content
                const responseSelectors = [
                    '[data-node-id]',
                    '.ds-markdown',
                    '.markdown',
                    '[class*="message"] [class*="content"]',
                    '.prose'
                ];

                for (const sel of responseSelectors) {{
                    const elements = document.querySelectorAll(sel);
                    if (elements.length > 0) {{
                        const last = elements[elements.length - 1];
                        const text = last.textContent.trim();
                        if (text.length > 10) {{
                            debug.responseSelector = sel;
                            debug.responseCount = elements.length;
                            debug.responseLength = text.length;
                            return JSON.stringify({{ content: text.substring(0, 2000), debug }});
                        }}
                    }}
                }}

                // Fallback: get all page text changes
                const allDivs = document.querySelectorAll('[class*="ds-"], [class*="chat"], [class*="message"]');
                debug.allDivsCount = allDivs.length;
                if (allDivs.length > 0) {{
                    const lastDiv = allDivs[allDivs.length - 1];
                    const text = lastDiv.textContent.trim();
                    if (text.length > 10) {{
                        return JSON.stringify({{ content: text.substring(0, 2000), debug }});
                    }}
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
            let debug_info = response_json.get("debug")
                .map(|d| format!(" | debug: {}", d))
                .unwrap_or_default();
            tracing::warn!("DeepSeek JS chat failed: {}{}", err, debug_info);
            Err(ProviderError::InvalidResponse(format!("JS chat error: {}{}", err, debug_info)))
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
