use db::PostgresPool;
use models::{User, ApiKey};
use uuid::Uuid;

use crate::AuthError;

/// API Key validator
pub struct ApiKeyValidator<'a> {
    db: &'a PostgresPool,
}

impl<'a> ApiKeyValidator<'a> {
    pub fn new(db: &'a PostgresPool) -> Self {
        Self { db }
    }

    /// Validate an API key (plain text, no "Bearer " prefix) and return the associated user
    pub async fn validate(&self, key: &str) -> Result<(ApiKey, User), AuthError> {
        // Hash the provided key
        let key_hash = super::keygen::ApiKeyGenerator::hash_key(key);

        // Look up the key in the database
        let api_key = self.db.get_api_key_by_hash(&key_hash)
            .await
            .map_err(|_| AuthError::ApiKeyInvalid)?
            .ok_or(AuthError::ApiKeyNotFound)?;

        if !api_key.is_active {
            return Err(AuthError::ApiKeyInvalid);
        }

        // Get the user
        let user = self.db.get_user_by_id(api_key.user_id)
            .await
            .map_err(|_| AuthError::ApiKeyInvalid)?
            .ok_or(AuthError::UserNotFound)?;

        // Check subscription
        if !user.is_subscription_active() {
            return Err(AuthError::SubscriptionExpired);
        }

        // Update last used timestamp
        let _ = self.db.update_api_key_last_used(api_key.id).await;

        Ok((api_key, user))
    }

    /// Validate a Bearer token from Authorization header
    /// Strips "Bearer " prefix before validation
    pub async fn validate_bearer(&self, auth_header: &str) -> Result<(ApiKey, User), AuthError> {
        if !auth_header.starts_with("Bearer ") {
            return Err(AuthError::ApiKeyInvalid);
        }

        let key = &auth_header[7..];
        self.validate(key).await
    }
}

/// Bearer token validator
pub struct BearerValidator<'a> {
    db: &'a PostgresPool,
}

impl<'a> BearerValidator<'a> {
    pub fn new(db: &'a PostgresPool) -> Self {
        Self { db }
    }

    /// Validate a Bearer token and return the user
    pub async fn validate(&self, auth_header: &str) -> Result<User, AuthError> {
        if !auth_header.starts_with("Bearer ") {
            return Err(AuthError::ApiKeyInvalid);
        }

        let key = &auth_header[7..];

        // Hash the provided key (without "Bearer " prefix)
        let key_hash = super::keygen::ApiKeyGenerator::hash_key(key);

        // Look up the key in the database
        let api_key = self.db.get_api_key_by_hash(&key_hash)
            .await
            .map_err(|_| AuthError::ApiKeyInvalid)?
            .ok_or(AuthError::ApiKeyNotFound)?;

        if !api_key.is_active {
            return Err(AuthError::ApiKeyInvalid);
        }

        // Get the user
        let user = self.db.get_user_by_id(api_key.user_id)
            .await
            .map_err(|_| AuthError::ApiKeyInvalid)?
            .ok_or(AuthError::UserNotFound)?;

        // Check subscription
        if !user.is_subscription_active() {
            return Err(AuthError::SubscriptionExpired);
        }

        Ok(user)
    }
}
