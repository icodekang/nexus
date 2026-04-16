use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Browser account status for ZeroToken
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BrowserAccountStatus {
    Pending,   // QR code generated, awaiting auth
    Active,     // Authenticated and ready to use
    Expired,    // Session expired
    Error,      // Auth failed or error
}

impl BrowserAccountStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            BrowserAccountStatus::Pending => "pending",
            BrowserAccountStatus::Active => "active",
            BrowserAccountStatus::Expired => "expired",
            BrowserAccountStatus::Error => "error",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "active" => BrowserAccountStatus::Active,
            "expired" => BrowserAccountStatus::Expired,
            "error" => BrowserAccountStatus::Error,
            _ => BrowserAccountStatus::Pending,
        }
    }
}

/// Browser account for ZeroToken - authenticated via QR code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserAccount {
    pub id: Uuid,
    pub provider: String,                    // "claude", "chatgpt"
    pub email: Option<String>,               // Login email if applicable
    pub session_data_encrypted: String,      // Encrypted cookies/tokens JSON
    pub status: BrowserAccountStatus,
    pub request_count: i64,                  // Total requests served
    pub last_used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl BrowserAccount {
    pub fn new(provider: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            provider,
            email: None,
            session_data_encrypted: String::new(),
            status: BrowserAccountStatus::Pending,
            request_count: 0,
            last_used_at: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    pub fn with_email(mut self, email: String) -> Self {
        self.email = Some(email);
        self
    }

    pub fn with_session_data(mut self, data: String) -> Self {
        self.session_data_encrypted = data;
        self
    }

    pub fn activate(mut self) -> Self {
        self.status = BrowserAccountStatus::Active;
        self.updated_at = Utc::now();
        self
    }

    pub fn mark_error(mut self) -> Self {
        self.status = BrowserAccountStatus::Error;
        self.updated_at = Utc::now();
        self
    }

    pub fn mark_expired(mut self) -> Self {
        self.status = BrowserAccountStatus::Expired;
        self.updated_at = Utc::now();
        self
    }

    pub fn increment_request_count(&mut self) {
        self.request_count += 1;
        self.last_used_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }

    pub fn is_active(&self) -> bool {
        self.status == BrowserAccountStatus::Active
    }
}
