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

    /// 获取提供商的登录 URL
    fn get_login_url(provider: &str) -> Result<String, ProviderError> {
        match provider.to_lowercase().as_str() {
            "deepseek" => Ok("https://chat.deepseek.com/signin".to_string()),
            "claude" | "claude.ai" => Ok("https://claude.ai/login".to_string()),
            "chatgpt" | "chat.openai.com" => Ok("https://chat.openai.com/login".to_string()),
            _ => Err(ProviderError::ProviderNotFound(provider.to_string())),
        }
    }

    /// 检测登录完成的 URL 模式
    fn detect_login_success(provider: &str, url: &str) -> bool {
        match provider.to_lowercase().as_str() {
            "deepseek" => url.contains("chat.deepseek.com") && !url.contains("/signin") && !url.contains("/login"),
            "claude" | "claude.ai" => url.contains("claude.ai") && !url.contains("/login"),
            "chatgpt" | "chat.openai.com" => url.contains("chat.openai.com") && !url.contains("/login"),
            _ => false,
        }
    }

    /// 创建一个新的登录会话
    pub async fn create_login_session(
        &self,
        provider: &str,
    ) -> Result<LoginSession, ProviderError> {
        let login_url = Self::get_login_url(provider)?;

        // 启动无头浏览器
        let browser = Browser::default()
            .map_err(|e| ProviderError::InternalError(format!("Failed to launch browser: {}", e)))?;

        // 创建新标签页并导航
        let tab = browser.new_tab()
            .map_err(|e| ProviderError::InternalError(format!("Failed to create tab: {}", e)))?;

        tab.navigate_to(&login_url)
            .map_err(|e| ProviderError::InternalError(format!("Failed to navigate: {}", e)))?;

        // 等待页面加载
        std::thread::sleep(std::time::Duration::from_secs(2));

        // 获取当前 URL
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
