//! Account Pool Manager for ZeroToken Browser Sessions
//!
//! Manages a pool of authenticated browser sessions for each provider.
//! Sessions are loaded from database and cached in memory for fast access.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::browser_emulator::{BrowserEmulatorClient, PersistedSession};
use crate::client::ProviderClient;
use crate::error::ProviderError;

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
}

impl AccountPool {
    pub fn new() -> Self {
        Self {
            clients: Arc::new(RwLock::new(HashMap::new())),
            accounts: Arc::new(RwLock::new(HashMap::new())),
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
