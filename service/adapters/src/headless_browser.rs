//! 无头浏览器管理器模块
//!
//! 使用 Chrome DevTools Protocol 实现无头浏览器控制，
//! 用于自动捕获登录会话的 Cookie 和 Token。

use headless_chrome::browser::Browser;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use uuid::Uuid;

use crate::error::ProviderError;

/// 无头浏览器会话状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BrowserSessionState {
    /// 等待用户扫码登录
    WaitingForLogin,
    /// 登录成功
    LoggedIn,
    /// 登录失败或超时
    Failed,
    /// 会话已关闭
    Closed,
}

/// 登录会话信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginSession {
    /// 会话 ID
    pub id: Uuid,
    /// 提供商名称
    pub provider: String,
    /// 登录页 URL（用于生成二维码）
    pub login_url: String,
    /// 会话状态
    pub state: BrowserSessionState,
    /// 当前页面 URL（可能变化）
    pub current_url: String,
    /// 捕获的 Cookie（JSON 字符串）
    pub cookies_json: String,
    /// 创建时间
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// 无头浏览器管理器
///
/// 管理多个提供商的无头浏览器会话
pub struct HeadlessBrowserManager {
    /// 活跃的登录会话
    sessions: Arc<RwLock<std::collections::HashMap<Uuid, LoginSession>>>,
    /// 活跃的浏览器实例
    browsers: Arc<RwLock<std::collections::HashMap<Uuid, Arc<Browser>>>>,
    /// 事件广播器（用于通知登录状态变化）
    event_tx: broadcast::Sender<LoginEvent>,
}

/// 登录事件
#[derive(Debug, Clone)]
pub enum LoginEvent {
    /// 状态变化
    StateChanged {
        session_id: Uuid,
        old_state: BrowserSessionState,
        new_state: BrowserSessionState,
        cookies_json: Option<String>,
    },
    /// URL 变化
    UrlChanged {
        session_id: Uuid,
        url: String,
    },
    /// 手机验证码已发送，等待用户输入
    WaitingForCode {
        session_id: Uuid,
        phone: String,
        message: String,
    },
    /// 错误
    Error {
        session_id: Uuid,
        error: String,
    },
}

impl HeadlessBrowserManager {
    /// 创建新的管理器
    pub fn new() -> Self {
        let (event_tx, _) = broadcast::channel(100);
        Self {
            sessions: Arc::new(RwLock::new(std::collections::HashMap::new())),
            browsers: Arc::new(RwLock::new(std::collections::HashMap::new())),
            event_tx,
        }
    }

    /// 获取事件订阅器
    pub fn subscribe(&self) -> broadcast::Receiver<LoginEvent> {
        self.event_tx.subscribe()
    }

    fn get_login_url(provider: &str) -> Result<String, ProviderError> {
        match provider.to_lowercase().as_str() {
            "deepseek" => Ok("https://chat.deepseek.com/sign_in".to_string()),
            "claude" | "claude.ai" => Ok("https://claude.ai/login".to_string()),
            "chatgpt" | "chat.openai.com" | "openai" => Ok("https://chatgpt.com/auth/login".to_string()),
            _ => Err(ProviderError::ProviderNotFound(provider.to_string())),
        }
    }

    fn detect_chrome_binary() -> Result<(), ProviderError> {
        if std::env::var("CHROME_PATH").is_ok() {
            return Ok(());
        }

        #[cfg(target_os = "macos")]
        {
            let chrome_paths = [
                "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
                "/Applications/Chromium.app/Contents/MacOS/Chromium",
                "/Applications/Brave Browser.app/Contents/MacOS/Brave Browser",
                "/Applications/Microsoft Edge.app/Contents/MacOS/Microsoft Edge",
            ];
            for path in &chrome_paths {
                if std::path::Path::new(path).exists() {
                    std::env::set_var("CHROME_PATH", path);
                    return Ok(());
                }
            }
        }

        #[cfg(target_os = "linux")]
        {
            let chrome_paths = [
                "/usr/bin/google-chrome",
                "/usr/bin/chromium-browser",
                "/usr/bin/chromium",
                "/usr/bin/brave-browser",
                "/snap/bin/chromium",
            ];
            for path in &chrome_paths {
                if std::path::Path::new(path).exists() {
                    std::env::set_var("CHROME_PATH", path);
                    return Ok(());
                }
            }
        }

        #[cfg(target_os = "windows")]
        {
            let chrome_paths = [
                "C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe",
                "C:\\Program Files (x86)\\Google\\Chrome\\Application\\chrome.exe",
            ];
            for path in &chrome_paths {
                if std::path::Path::new(path).exists() {
                    std::env::set_var("CHROME_PATH", path);
                    return Ok(());
                }
            }
        }

        Err(ProviderError::ChromeNotFound)
    }

    fn detect_login_success(provider: &str, url: &str) -> bool {
        let url_lower = url.to_lowercase();
        match provider.to_lowercase().as_str() {
            "deepseek" => url_lower.contains("chat.deepseek.com") && !url_lower.contains("/signin") && !url_lower.contains("/login"),
            "claude" | "claude.ai" => url_lower.contains("claude.ai") && !url_lower.contains("/login") && !url_lower.contains("/authenticate"),
            "chatgpt" | "chat.openai.com" => {
                (url_lower.contains("chat.openai.com") || url_lower.contains("chatgpt.com"))
                && !url_lower.contains("/login")
                && !url_lower.contains("/auth")
            },
            _ => false,
        }
    }

    /// 创建一个新的登录会话
    pub async fn create_login_session(
        &self,
        provider: &str,
    ) -> Result<LoginSession, ProviderError> {
        Self::detect_chrome_binary()?;

        let login_url = Self::get_login_url(provider)?;

        let browser = Browser::default()
            .map_err(|e| ProviderError::InternalError(format!("Failed to launch browser: {}", e)))?;

        let tab = browser.new_tab()
            .map_err(|e| ProviderError::InternalError(format!("Failed to create tab: {}", e)))?;

        tab.navigate_to(&login_url)
            .map_err(|e| ProviderError::InternalError(format!("Failed to navigate: {}", e)))?;

        std::thread::sleep(std::time::Duration::from_secs(3));

        let _ = tab.evaluate(
            "Object.defineProperty(navigator, 'webdriver', { get: () => false })",
            true,
        );

        let current_url = tab.get_url();

        // 创建会话
        let session_id = Uuid::new_v4();
        let session = LoginSession {
            id: session_id,
            provider: provider.to_string(),
            login_url: login_url.clone(),
            state: BrowserSessionState::WaitingForLogin,
            current_url: current_url.clone(),
            cookies_json: String::new(),
            created_at: chrono::Utc::now(),
        };

        // 存储会话
        {
            let mut sessions = self.sessions.write().await;
            sessions.insert(session_id, session.clone());
        }
        {
            let mut browsers = self.browsers.write().await;
            browsers.insert(session_id, Arc::new(browser));
        }

        Ok(session)
    }

    /// 刷新会话状态
    pub async fn refresh_session(
        &self,
        session_id: Uuid,
    ) -> Result<LoginSession, ProviderError> {
        let (browser, mut session) = {
            let browsers = self.browsers.read().await;
            let sessions = self.sessions.read().await;

            let browser = browsers.get(&session_id)
                .ok_or_else(|| ProviderError::InternalError("Session not found".to_string()))?
                .clone();

            let session = sessions.get(&session_id)
                .ok_or_else(|| ProviderError::InternalError("Session not found".to_string()))?
                .clone();

            (browser, session)
        };

        // 获取标签页并检查当前 URL - 在同步块中完成避免 Send 问题
        let current_url = {
            let tabs = browser.get_tabs();
            match tabs.lock() {
                Ok(tabs_guard) => {
                    if let Some(tab) = tabs_guard.first() {
                        tab.get_url()
                    } else {
                        session.current_url.clone()
                    }
                }
                Err(_) => session.current_url.clone()
            }
        };

        let old_state = session.state;

        // 检测是否登录成功
        if Self::detect_login_success(&session.provider, &current_url) {
            session.state = BrowserSessionState::LoggedIn;
            session.current_url = current_url;

            // 广播事件
            let _ = self.event_tx.send(LoginEvent::StateChanged {
                session_id,
                old_state,
                new_state: BrowserSessionState::LoggedIn,
                cookies_json: Some(session.cookies_json.clone()),
            });
        } else {
            session.current_url = current_url;
        }

        // 更新会话
        {
            let mut sessions = self.sessions.write().await;
            sessions.insert(session_id, session.clone());
        }

        Ok(session)
    }

    /// 获取会话状态
    pub async fn get_session(
        &self,
        session_id: Uuid,
    ) -> Result<Option<LoginSession>, ProviderError> {
        let sessions = self.sessions.read().await;
        Ok(sessions.get(&session_id).cloned())
    }

    /// 关闭会话
    pub async fn close_session(&self, session_id: Uuid) -> Result<(), ProviderError> {
        // 关闭浏览器
        {
            let mut browsers = self.browsers.write().await;
            browsers.remove(&session_id);
        }

        // 更新会话状态
        {
            let mut sessions = self.sessions.write().await;
            if let Some(session) = sessions.get_mut(&session_id) {
                session.state = BrowserSessionState::Closed;
            }
        }

        Ok(())
    }

    pub async fn initiate_phone_login(
        &self,
        provider: &str,
        phone: &str,
    ) -> Result<LoginSession, ProviderError> {
        Self::detect_chrome_binary()?;

        let login_url = Self::get_login_url(provider)?;
        tracing::info!("Initiating phone login for provider: {}, phone: {}", provider, phone);

        let browser = Browser::default()
            .map_err(|e| ProviderError::InternalError(format!("Failed to launch browser: {}", e)))?;

        let tab = browser.new_tab()
            .map_err(|e| ProviderError::InternalError(format!("Failed to create tab: {}", e)))?;

        tab.navigate_to(&login_url)
            .map_err(|e| ProviderError::InternalError(format!("Failed to navigate: {}", e)))?;

        self.wait_for_page_ready(&tab).await?;
        self.inject_stealth_scripts(&tab).await?;

        let current_url = tab.get_url();
        Self::detect_page_block(provider, &current_url)?;
        self.check_page_blocked(&tab).await?;

        self.fill_phone_field(&tab, phone).await?;
        self.click_send_code_button(&tab).await?;

        let session_id = Uuid::new_v4();
        let session = LoginSession {
            id: session_id,
            provider: provider.to_string(),
            login_url: login_url.clone(),
            state: BrowserSessionState::WaitingForLogin,
            current_url: tab.get_url(),
            cookies_json: String::new(),
            created_at: chrono::Utc::now(),
        };

        {
            let mut sessions = self.sessions.write().await;
            sessions.insert(session_id, session.clone());
        }
        {
            let mut browsers = self.browsers.write().await;
            browsers.insert(session_id, Arc::new(browser));
        }

        let _ = self.event_tx.send(LoginEvent::WaitingForCode {
            session_id,
            phone: phone.to_string(),
            message: "验证码已发送，请查收短信".to_string(),
        });

        Ok(session)
    }

    pub async fn complete_phone_login(
        &self,
        session_id: Uuid,
        code: &str,
    ) -> Result<String, ProviderError> {
        let (browser, session) = {
            let browsers = self.browsers.read().await;
            let sessions = self.sessions.read().await;
            let browser = browsers.get(&session_id)
                .ok_or_else(|| ProviderError::InternalError("Phone login session not found".to_string()))?
                .clone();
            let session = sessions.get(&session_id)
                .ok_or_else(|| ProviderError::InternalError("Phone login session not found".to_string()))?
                .clone();
            (browser, session)
        };

        let tab = {
            let tabs = browser.get_tabs();
            match tabs.lock() {
                Ok(tabs_guard) => tabs_guard.first().cloned()
                    .ok_or_else(|| ProviderError::InternalError("No tab found in browser".to_string()))?,
                Err(_) => return Err(ProviderError::InternalError("Failed to lock tabs".to_string())),
            }
        };

        self.fill_code_field(&tab, code).await?;
        self.click_verify_button(&tab).await?;

        self.wait_for_login_complete(&tab, &session.provider).await?;

        let cookies_json = self.get_cookies(&tab).await?;

        {
            let mut sessions = self.sessions.write().await;
            if let Some(s) = sessions.get_mut(&session_id) {
                s.state = BrowserSessionState::LoggedIn;
                s.cookies_json = cookies_json.clone();
            }
        }

        let _ = self.event_tx.send(LoginEvent::StateChanged {
            session_id,
            old_state: BrowserSessionState::WaitingForLogin,
            new_state: BrowserSessionState::LoggedIn,
            cookies_json: Some(cookies_json.clone()),
        });

        self.close_session(session_id).await.ok();

        Ok(cookies_json)
    }

    async fn fill_phone_field(&self, tab: &headless_chrome::Tab, phone: &str) -> Result<(), ProviderError> {
        let phone_selectors = [
            "input[type='tel']",
            "input[name='phone']",
            "input[name='mobile']",
            "input[name='phoneNumber']",
            "input[placeholder*='手机' i]",
            "input[placeholder*='电话' i]",
            "input[placeholder*='phone' i]",
            "input[placeholder*='mobile' i]",
            "input[autocomplete='tel']",
            "input[type='text']",
        ];

        let mut filled = false;
        for selector in &phone_selectors {
            match self.fill_input(tab, selector, phone).await {
                Ok(_) => {
                    tracing::info!("Filled phone using selector: {}", selector);
                    filled = true;
                    break;
                }
                Err(e) => {
                    tracing::debug!("Failed phone fill with {}: {}", selector, e);
                }
            }
        }

        if !filled {
            return Err(ProviderError::InternalError("Could not find phone input field".to_string()));
        }

        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        Ok(())
    }

    async fn click_send_code_button(&self, tab: &headless_chrome::Tab) -> Result<(), ProviderError> {
        let script = r#"
            (function() {
                const texts = ['获取验证码', '发送验证码', 'get code', 'send code', '发送', '获取', 'verify'];
                const buttons = Array.from(document.querySelectorAll('button, span, a, div[role="button"]'));
                for (const btn of buttons) {
                    if (btn.offsetParent === null) continue;
                    const text = (btn.innerText || btn.textContent || '').toLowerCase();
                    if (texts.some(t => text.includes(t))) {
                        btn.click();
                        return true;
                    }
                }
                const smsBtn = document.querySelector('[data-testid="send-code"], .send-code-btn, .sms-btn');
                if (smsBtn && smsBtn.offsetParent !== null) {
                    smsBtn.click();
                    return true;
                }
                return false;
            })()
        "#;

        let clicked = tab.evaluate(script, false)
            .map_err(|e| ProviderError::InternalError(format!("Failed to click send code: {}", e)))?
            .value.and_then(|v| v.as_bool()).unwrap_or(false);

        if !clicked {
            return Err(ProviderError::InternalError("Could not find send-code button".to_string()));
        }

        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        Ok(())
    }

    async fn fill_code_field(&self, tab: &headless_chrome::Tab, code: &str) -> Result<(), ProviderError> {
        let code_selectors = [
            "input[type='tel']",
            "input[name='code']",
            "input[name='verifyCode']",
            "input[name='verificationCode']",
            "input[placeholder*='验证码' i]",
            "input[placeholder*='code' i]",
            "input[maxlength='6']",
            "input[maxlength='4']",
            "input[type='number']",
            "input[type='text']",
        ];

        let mut filled = false;
        for selector in &code_selectors {
            match self.fill_input(tab, selector, code).await {
                Ok(_) => {
                    tracing::info!("Filled code using selector: {}", selector);
                    filled = true;
                    break;
                }
                Err(e) => {
                    tracing::debug!("Failed code fill with {}: {}", selector, e);
                }
            }
        }

        if !filled {
            return Err(ProviderError::InternalError("Could not find verification code input field".to_string()));
        }

        tokio::time::sleep(std::time::Duration::from_millis(300)).await;
        Ok(())
    }

    async fn click_verify_button(&self, tab: &headless_chrome::Tab) -> Result<(), ProviderError> {
        let script = r#"
            (function() {
                const texts = ['验证', 'verify', '登录', 'login', '确认', 'confirm', '提交', 'submit'];
                const buttons = Array.from(document.querySelectorAll('button, input[type="submit"], span, a, div[role="button"]'));
                for (const btn of buttons) {
                    if (btn.offsetParent === null) continue;
                    const text = (btn.innerText || btn.textContent || btn.value || '').toLowerCase();
                    if (texts.some(t => text.includes(t))) {
                        btn.click();
                        return true;
                    }
                }
                const submitBtn = document.querySelector('button[type="submit"], input[type="submit"]');
                if (submitBtn && submitBtn.offsetParent !== null) {
                    submitBtn.click();
                    return true;
                }
                return false;
            })()
        "#;

        let clicked = tab.evaluate(script, false)
            .map_err(|e| ProviderError::InternalError(format!("Failed to click verify: {}", e)))?
            .value.and_then(|v| v.as_bool()).unwrap_or(false);

        if !clicked {
            let enter_script = r#"
                (function() {
                    const inputs = document.querySelectorAll('input[type="tel"], input[type="number"]');
                    if (inputs.length > 0) {
                        inputs[inputs.length - 1].dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', code: 'Enter', keyCode: 13, bubbles: true }));
                        return true;
                    }
                    return false;
                })()
            "#;
            let enter_ok = tab.evaluate(enter_script, false)
                .map_err(|e| ProviderError::InternalError(format!("Failed to submit via enter: {}", e)))?;

            if !enter_ok.value.and_then(|v| v.as_bool()).unwrap_or(false) {
                return Err(ProviderError::InternalError("Could not find verify/login button".to_string()));
            }
        }

        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        Ok(())
    }

    /// 获取登录 URL 用于生成二维码
    pub async fn get_login_url_for_qr(
        &self,
        session_id: Uuid,
    ) -> Result<String, ProviderError> {
        let sessions = self.sessions.read().await;
        let session = sessions.get(&session_id)
            .ok_or_else(|| ProviderError::InternalError("Session not found".to_string()))?;

        Ok(session.current_url.clone())
    }

    fn detect_page_block(provider: &str, url: &str) -> Result<(), ProviderError> {
        let url_lower = url.to_lowercase();

        if url_lower.contains("cloudflare") || url_lower.contains("/cdn-cgi/") {
            return Err(ProviderError::BlockedByProvider);
        }

        if url_lower.contains("challenge") && url_lower.contains("captcha") {
            return Err(ProviderError::BlockedByProvider);
        }

        if url_lower.contains("auth.openai.com") || url_lower.contains("api.openai.com/auth") {
            return Err(ProviderError::BlockedByProvider);
        }

        let known_login_urls = [
            ("chatgpt", "chatgpt.com/auth/login"),
            ("chatgpt", "chat.openai.com"),
            ("claude", "claude.ai/login"),
            ("claude", "claude.ai"),
            ("deepseek", "chat.deepseek.com/sign_in"),
            ("deepseek", "chat.deepseek.com"),
        ];

        let matched_url = known_login_urls.iter().find(|(p, _)| {
            provider.to_lowercase() == *p
        });

        if let Some((_, expected_url)) = matched_url {
            if url_lower.contains(expected_url) {
                return Ok(());
            }
        }

        if url_lower.contains("error") && url_lower.contains("blocked") {
            return Err(ProviderError::BlockedByProvider);
        }

        Ok(())
    }

    async fn check_page_blocked(&self, tab: &headless_chrome::Tab) -> Result<(), ProviderError> {
        let script = r#"
            (function() {
                if (document.title && (
                    document.title.includes('Cloudflare') ||
                    document.title.includes('Just a moment') ||
                    document.title.includes('Checking your browser') ||
                    document.title.includes('blocked') ||
                    document.title.includes('Access denied') ||
                    document.title.includes('are you a robot')
                )) { return 'BLOCKED'; }
                if (document.querySelector && (
                    document.querySelector('#challenge-form') ||
                    document.querySelector('.cf-browser-verification') ||
                    document.querySelector('#cf-challenge-container') ||
                    document.querySelector('.g-recaptcha') ||
                    document.querySelector('iframe[src*="recaptcha"]') ||
                    document.querySelector('iframe[src*="hcaptcha"]') ||
                    document.querySelector('#px-captcha') ||
                    document.querySelector('div[data-testid="auth-wall"]')
                )) { return 'BLOCKED'; }
                return '';
            })()
        "#;
        match tab.evaluate(script, false) {
            Ok(result) => {
                if let Some(v) = result.value {
                    if v.as_str() == Some("BLOCKED") {
                        return Err(ProviderError::BlockedByProvider);
                    }
                }
            }
            Err(_) => {}
        }
        Ok(())
    }

    pub async fn login_with_password(
        &self,
        provider: &str,
        email: &str,
        password: &str,
    ) -> Result<String, ProviderError> {
        Self::detect_chrome_binary()?;

        let login_url = Self::get_login_url(provider)?;

        tracing::info!("Starting password login for provider: {}, URL: {}", provider, login_url);

        let browser = Browser::default()
            .map_err(|e| ProviderError::InternalError(format!("Failed to launch browser: {}", e)))?;

        let tab = browser.new_tab()
            .map_err(|e| ProviderError::InternalError(format!("Failed to create tab: {}", e)))?;

        tab.navigate_to(&login_url)
            .map_err(|e| ProviderError::InternalError(format!("Failed to navigate: {}", e)))?;

        self.wait_for_page_ready(&tab).await?;

        self.inject_stealth_scripts(&tab).await?;

        let current_url = tab.get_url();
        tracing::info!("Page loaded, current URL: {}", current_url);

        Self::detect_page_block(provider, &current_url)?;
        self.check_page_blocked(&tab).await?;

        self.perform_login_flow(&tab, email, password).await?;

        self.wait_for_login_complete(&tab, provider).await?;

        let cookies_json = self.get_cookies(&tab).await?;

        drop(browser);

        Ok(cookies_json)
    }

    async fn wait_for_page_ready(&self, tab: &headless_chrome::Tab) -> Result<(), ProviderError> {
        let script = r#"
            (function() {
                if (document.readyState === 'complete') return true;
                return false;
            })()
        "#;
        for _ in 0..30 {
            match tab.evaluate(script, false) {
                Ok(result) => {
                    if result.value.and_then(|v| v.as_bool()).unwrap_or(false) {
                        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                        return Ok(());
                    }
                }
                Err(_) => {}
            }
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        }
        tracing::warn!("Page load timeout after 15s, continuing anyway");
        Ok(())
    }

    async fn inject_stealth_scripts(&self, tab: &headless_chrome::Tab) -> Result<(), ProviderError> {
        let stealth_script = r#"
            (function() {
                Object.defineProperty(navigator, 'webdriver', { get: () => false });
                Object.defineProperty(navigator, 'plugins', { get: () => [1, 2, 3, 4, 5] });
                Object.defineProperty(navigator, 'languages', { get: () => ['en-US', 'en'] });
                window.chrome = { runtime: {} };
                Object.defineProperty(navigator, 'permissions', { get: () => ({ query: async () => ({ state: 'granted' }) }) });
            })()
        "#;
        let _ = tab.evaluate(stealth_script, true);
        Ok(())
    }

    async fn perform_login_flow(
        &self,
        tab: &headless_chrome::Tab,
        email: &str,
        password: &str,
    ) -> Result<(), ProviderError> {
        let email_selectors = [
            "input[type='email']",
            "input[name='username']",
            "input[name='email']",
            "input#username",
            "input#email",
            "input[type='text']",
            "input[autocomplete='username']",
            "input[autocomplete='email']",
            "input[placeholder*='email' i]",
            "input[placeholder*='邮箱' i]",
            "input[placeholder*='账号' i]",
            "form input",
        ];

        let mut email_filled = false;
        for selector in &email_selectors {
            match self.fill_input(tab, selector, email).await {
                Ok(_) => {
                    tracing::info!("Filled email using selector: {}", selector);
                    email_filled = true;
                    break;
                }
                Err(e) => {
                    tracing::debug!("Failed to fill with selector {}: {}", selector, e);
                }
            }
        }

        if !email_filled {
            let html = tab.get_content()
                .map_err(|e| ProviderError::InternalError(format!("Failed to get page content: {}", e)))?;
            tracing::error!("Could not find email field. Page HTML (first 2000 chars): {}", &html[..html.len().min(2000)]);
            return Err(ProviderError::InternalError("Could not find email input field".to_string()));
        }

        tokio::time::sleep(std::time::Duration::from_millis(800)).await;

        if self.is_password_field_visible(tab).await {
            tracing::info!("Password field visible, filling password");
            self.fill_password_and_submit(tab, password).await?;
        } else {
            tracing::info!("Password field not visible, clicking continue");
            self.click_continue_button(tab).await?;

            tokio::time::sleep(std::time::Duration::from_secs(3)).await;

            self.fill_password_and_submit(tab, password).await?;
        }

        Ok(())
    }

    async fn is_password_field_visible(&self, tab: &headless_chrome::Tab) -> bool {
        let script = r#"
            (function() {
                const pw = document.querySelector('input[type="password"], input[name="password"], input#password, input[autocomplete="current-password"]');
                return !!(pw && pw.offsetParent !== null);
            })()
        "#;
        tab.evaluate(script, false)
            .ok()
            .and_then(|r| r.value.and_then(|v| v.as_bool()))
            .unwrap_or(false)
    }

    async fn click_continue_button(&self, tab: &headless_chrome::Tab) -> Result<(), ProviderError> {
        let script = r#"
            (function() {
                const buttonTexts = ['continue', 'next', 'proceed', 'submit'];
                const buttons = Array.from(document.querySelectorAll('button, input[type="submit"], a.btn'));

                for (const btn of buttons) {
                    if (btn.offsetParent === null) continue;
                    const text = (btn.innerText || btn.value || '').toLowerCase();
                    if (buttonTexts.some(t => text.includes(t))) {
                        btn.click();
                        return true;
                    }
                }

                const submitBtn = document.querySelector('button[type="submit"], input[type="submit"]');
                if (submitBtn && submitBtn.offsetParent !== null) {
                    submitBtn.click();
                    return true;
                }

                const primaryBtn = document.querySelector('.btn-primary, .Button--primary, [data-testid="continue-button"]');
                if (primaryBtn && primaryBtn.offsetParent !== null) {
                    primaryBtn.click();
                    return true;
                }

                const inputs = document.querySelectorAll('input[type="email"], input[type="text"]');
                if (inputs.length > 0) {
                    inputs[inputs.length - 1].dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', code: 'Enter', keyCode: 13, bubbles: true }));
                    return true;
                }
                return false;
            })()
        "#;

        let clicked = tab.evaluate(script, false)
            .map_err(|e| ProviderError::InternalError(format!("Failed to click continue: {}", e)))?
            .value.and_then(|v| v.as_bool()).unwrap_or(false);

        if !clicked {
            return Err(ProviderError::InternalError("Could not click continue button".to_string()));
        }

        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        Ok(())
    }

    async fn fill_password_and_submit(
        &self,
        tab: &headless_chrome::Tab,
        password: &str,
    ) -> Result<(), ProviderError> {
        let password_selectors = [
            "input[type='password']",
            "input[name='password']",
            "input#password",
            "input[autocomplete='current-password']",
            "input[placeholder*='password' i]",
            "input[placeholder*='密码' i]",
            "form input[type='password']",
        ];

        let mut password_filled = false;
        for selector in &password_selectors {
            match self.fill_input(tab, selector, password).await {
                Ok(_) => {
                    tracing::info!("Filled password using selector: {}", selector);
                    password_filled = true;
                    break;
                }
                Err(e) => {
                    tracing::debug!("Failed to fill password with selector {}: {}", selector, e);
                }
            }
        }

        if !password_filled {
            let html = tab.get_content()
                .map_err(|e| ProviderError::InternalError(format!("Failed to get page content: {}", e)))?;
            tracing::error!("Could not find password field. Page HTML (first 2000 chars): {}", &html[..html.len().min(2000)]);
            return Err(ProviderError::InternalError("Could not find password input field".to_string()));
        }

        tokio::time::sleep(std::time::Duration::from_millis(800)).await;

        self.submit_form(tab).await?;

        Ok(())
    }

    /// 等待页面元素出现
    #[allow(dead_code)]
    async fn wait_for_element(&self, tab: &headless_chrome::Tab, selector: &str) -> Result<(), ProviderError> {
        let max_attempts = 60;
        let mut attempts = 0;

        let escaped_selector = selector.replace("'", "\\'");
        let script = format!("document.querySelector('{}') !== null", escaped_selector);

        while attempts < max_attempts {
            match tab.evaluate(&script, false) {
                Ok(result) => {
                    if result.value.and_then(|v| v.as_bool()).unwrap_or(false) {
                        return Ok(());
                    }
                }
                Err(_) => {}
            }
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            attempts += 1;
        }

        Err(ProviderError::InternalError(format!("Timeout waiting for element: {}", selector)))
    }

    /// 填写输入框
    async fn fill_input(&self, tab: &headless_chrome::Tab, selector: &str, value: &str) -> Result<(), ProviderError> {
        let escaped_selector = selector.replace("'", "\\'");
        let escaped_value = value.replace("\\", "\\\\").replace("'", "\\'").replace("\"", "\\\"");

        let script = format!(
            r#"(function() {{
                const el = document.querySelector('{}');
                if (!el) return false;
                if (el.offsetParent === null) return false;
                el.scrollIntoView({{ behavior: 'instant', block: 'center' }});
                el.focus();
                el.value = "{}";
                el.dispatchEvent(new Event('input', {{ bubbles: true }}));
                el.dispatchEvent(new Event('change', {{ bubbles: true }}));
                el.dispatchEvent(new Event('blur', {{ bubbles: true }}));
                el.dispatchEvent(new KeyboardEvent('keyup', {{ key: 'a', bubbles: true }}));
                return true;
            }})()"#,
            escaped_selector, escaped_value
        );

        let result = tab.evaluate(&script, false)
            .map_err(|e| ProviderError::InternalError(format!("Failed to execute fill script: {}", e)))?;

        if !result.value.and_then(|v| v.as_bool()).unwrap_or(false) {
            return Err(ProviderError::InternalError(format!("Element not found or not visible: {}", selector)));
        }

        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        Ok(())
    }

    async fn submit_form(&self, tab: &headless_chrome::Tab) -> Result<(), ProviderError> {
        let click_script = r#"
            (function() {
                const buttonTexts = ['log in', 'login', 'sign in', 'signin', 'continue', 'submit'];
                const buttons = Array.from(document.querySelectorAll('button, input[type="submit"]'));

                for (const btn of buttons) {
                    if (btn.offsetParent === null) continue;
                    const text = (btn.innerText || btn.value || '').toLowerCase();
                    if (buttonTexts.some(t => text.includes(t))) {
                        btn.click();
                        return true;
                    }
                }

                const submitBtn = document.querySelector('button[type="submit"], input[type="submit"]');
                if (submitBtn && submitBtn.offsetParent !== null) {
                    submitBtn.click();
                    return true;
                }

                const primaryBtn = document.querySelector('.btn-primary, .Button--primary, [data-testid="login-button"]');
                if (primaryBtn && primaryBtn.offsetParent !== null) {
                    primaryBtn.click();
                    return true;
                }

                return false;
            })()
        "#;

        let clicked = tab.evaluate(click_script, false)
            .map_err(|e| ProviderError::InternalError(format!("Failed to click submit: {}", e)))?
            .value.and_then(|v| v.as_bool()).unwrap_or(false);

        if !clicked {
            let enter_script = r#"
                (function() {
                    const inputs = document.querySelectorAll('input[type="password"], input[name="password"]');
                    if (inputs.length > 0) {
                        const lastInput = inputs[inputs.length - 1];
                        lastInput.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', code: 'Enter', keyCode: 13, bubbles: true }));
                        lastInput.dispatchEvent(new KeyboardEvent('keyup', { key: 'Enter', code: 'Enter', keyCode: 13, bubbles: true }));
                        return true;
                    }
                    return false;
                })()
            "#;
            let enter_result = tab.evaluate(enter_script, false)
                .map_err(|e| ProviderError::InternalError(format!("Failed to submit: {}", e)))?;

            if !enter_result.value.and_then(|v| v.as_bool()).unwrap_or(false) {
                return Err(ProviderError::InternalError("Could not find submit button or password field".to_string()));
            }
        }

        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        Ok(())
    }

    async fn wait_for_login_complete(&self, tab: &headless_chrome::Tab, provider: &str) -> Result<(), ProviderError> {
        let max_attempts = 120;

        for i in 0..max_attempts {
            let current_url = tab.get_url();

            if Self::detect_login_success(provider, &current_url) {
                return Ok(());
            }

            let error_script = r#"
                (function() {
                    if (document.title.includes('Cloudflare') || document.title.includes('Checking your browser')) {
                        return 'CLOUDFLARE_CHALLENGE';
                    }
                    const cfSelectors = ['#challenge-form', '.cf-content', '#cf-challenge-container'];
                    for (const sel of cfSelectors) {
                        if (document.querySelector(sel)) return 'CLOUDFLARE_CHALLENGE';
                    }
                    const selectors = [
                        '.error:not([aria-hidden="true"])',
                        '.alert-danger',
                        '[data-error]:not([data-error=""])',
                        '[role="alert"]',
                        '.form-error',
                        '.Toastify__toast--error',
                        '.error-message',
                        '[data-testid="error"]'
                    ];
                    for (const sel of selectors) {
                        try {
                            const el = document.querySelector(sel);
                            if (el && el.offsetParent !== null) {
                                const text = el.innerText || el.textContent || el.getAttribute('data-error');
                                if (text && text.trim()) return text.trim();
                            }
                        } catch (e) {}
                    }
                    const urlError = new URLSearchParams(window.location.search).get('error');
                    if (urlError) return decodeURIComponent(urlError);
                    return '';
                })()
            "#;

            if let Ok(result) = tab.evaluate(error_script, false) {
                if let Some(value) = result.value {
                    if let Some(text) = value.as_str() {
                        if !text.is_empty() {
                            if text == "CLOUDFLARE_CHALLENGE" {
                                return Err(ProviderError::CloudflareChallenge);
                            }
                            if text.to_lowercase().contains("incorrect password")
                                || text.to_lowercase().contains("invalid credentials")
                                || text.to_lowercase().contains("wrong password") {
                                return Err(ProviderError::AuthenticationError("Incorrect password".to_string()));
                            }
                            return Err(ProviderError::LoginFailed(format!("Login error: {}", text)));
                        }
                    }
                }
            }

            if i % 10 == 0 {
                let _ = tab.evaluate("window.scrollTo(0, document.body.scrollHeight);", false);
            }

            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        }

        Err(ProviderError::AuthenticationError("Login timeout".to_string()))
    }

    /// 获取页面所有 cookies
    async fn get_cookies(&self, tab: &headless_chrome::Tab) -> Result<String, ProviderError> {
        let cookies_script = "document.cookie.split(';').reduce(function(acc, c) { var parts = c.trim().split('='); if (parts.length >= 2) { acc[parts[0]] = parts.slice(1).join('='); } return acc; }, {})";

        let result = tab.evaluate(cookies_script, false)
            .map_err(|e| ProviderError::InternalError(format!("Failed to get cookies: {}", e)))?;

        let cookies = result.value
            .and_then(|v| v.as_object().cloned())
            .unwrap_or_default();

        let session = serde_json::json!({
            "cookies": cookies,
            "auth_tokens": {},
            "expires_at": null
        });

        serde_json::to_string(&session)
            .map_err(|e| ProviderError::InternalError(format!("Failed to serialize session: {}", e)))
    }

    /// 检查所有活跃会话的登录状态
    pub async fn check_all_sessions(&self) {
        let session_ids: Vec<Uuid> = {
            let sessions = self.sessions.read().await;
            sessions.keys().cloned().collect()
        };

        for session_id in session_ids {
            if let Err(e) = self.refresh_session(session_id).await {
                tracing::warn!("Failed to refresh session {}: {}", session_id, e);
            }
        }
    }
}

impl Default for HeadlessBrowserManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_login_success() {
        assert!(HeadlessBrowserManager::detect_login_success(
            "deepseek",
            "https://chat.deepseek.com/chat"
        ));
        assert!(!HeadlessBrowserManager::detect_login_success(
            "deepseek",
            "https://chat.deepseek.com/signin"
        ));

        assert!(HeadlessBrowserManager::detect_login_success(
            "claude",
            "https://claude.ai/chat"
        ));
        assert!(!HeadlessBrowserManager::detect_login_success(
            "claude",
            "https://claude.ai/login"
        ));
    }
}