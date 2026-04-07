use super::AuthError;

pub struct KeyValidator;

impl KeyValidator {
    /// Validate API key format
    pub fn validate_format(key: &str) -> Result<(), AuthError> {
        if !key.starts_with("sk-nova-") {
            return Err(AuthError::InvalidToken);
        }
        if key.len() < 20 {
            return Err(AuthError::InvalidToken);
        }
        Ok(())
    }

    /// Check if key is active (not revoked)
    pub async fn is_active(&self, _key: &str) -> bool {
        // TODO: Check against database
        true
    }
}
