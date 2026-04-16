use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// QR code session for browser authentication flow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QrCodeSession {
    pub id: Uuid,
    pub account_id: Uuid,                    // FK to BrowserAccount
    pub code: String,                        // 6-digit random code
    pub code_expires_at: DateTime<Utc>,      // QR code expires (5 minutes)
    pub auth_completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl QrCodeSession {
    pub fn new(account_id: Uuid) -> Self {
        let code = format!("{:06}", rand::random::<u32>() % 900000 + 100000);
        Self {
            id: Uuid::new_v4(),
            account_id,
            code,
            code_expires_at: Utc::now() + chrono::Duration::minutes(5),
            auth_completed_at: None,
            created_at: Utc::now(),
        }
    }

    pub fn is_expired(&self) -> bool {
        Utc::now() > self.code_expires_at
    }

    pub fn is_completed(&self) -> bool {
        self.auth_completed_at.is_some()
    }

    pub fn mark_completed(&mut self) {
        self.auth_completed_at = Some(Utc::now());
    }
}
