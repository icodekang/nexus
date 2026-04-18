//! JWT 认证测试
//!
//! 测试 JWT Token 的生成和验证功能

use nexus_auth::jwt::{create_token, verify_token, TokenClaims};
use chrono::{Utc, Duration};

#[test]
fn test_token_creation_and_verification() {
    let secret = "test-secret-key-must-be-at-least-32-bytes-long";
    let user_id = "user-123";
    let email = "test@example.com";

    let token = create_token(secret, user_id, email, false).unwrap();
    assert!(!token.is_empty());

    let claims = verify_token(secret, &token).unwrap();
    assert_eq!(claims.sub, user_id);
    assert_eq!(claims.email, email);
    assert!(!claims.is_admin);
}

#[test]
fn test_admin_token() {
    let secret = "test-secret-key-must-be-at-least-32-bytes-long";
    let token = create_token(secret, "admin-1", "admin@example.com", true).unwrap();

    let claims = verify_token(secret, &token).unwrap();
    assert!(claims.is_admin);
}

#[test]
fn test_token_expiration() {
    let secret = "test-secret-key-must-be-at-least-32-bytes-long";

    // 创建 1 小时过期的 token
    let token = create_token_with_expiry(
        secret,
        "user-123",
        "test@example.com",
        false,
        Utc::now() + Duration::hours(1),
    ).unwrap();

    let claims = verify_token(secret, &token).unwrap();
    assert!(!claims.is_expired());
}

#[test]
fn test_expired_token() {
    let secret = "test-secret-key-must-be-at-least-32-bytes-long";

    // 创建已过期的 token (1小时前)
    let token = create_token_with_expiry(
        secret,
        "user-123",
        "test@example.com",
        false,
        Utc::now() - Duration::hours(1),
    ).unwrap();

    let result = verify_token(secret, &token);
    assert!(result.is_err());
}

#[test]
fn test_invalid_secret() {
    let token = create_token("secret-1", "user-123", "test@example.com", false).unwrap();

    // 使用不同的 secret 验证
    let result = verify_token("secret-2-different", &token);
    assert!(result.is_err());
}

#[test]
fn test_malformed_token() {
    let secret = "test-secret-key-must-be-at-least-32-bytes-long";

    let result = verify_token(secret, "not.a.valid.token");
    assert!(result.is_err());

    let result = verify_token(secret, "completely_invalid_token_format");
    assert!(result.is_err());
}

#[test]
fn test_token_contains_required_claims() {
    let secret = "test-secret-key-must-be-at-least-32-bytes-long";
    let token = create_token(secret, "user-456", "user@example.com", false).unwrap();

    let claims = verify_token(secret, &token).unwrap();

    // 验证必要的声明存在
    assert!(claims.sub.is_some());
    assert!(claims.email.is_some());
    assert!(claims.iat.is_some());
    assert!(claims.exp.is_some());
}

#[test]
fn test_token_issuer() {
    let secret = "test-secret-key-must-be-at-least-32-bytes-long";
    let token = create_token(secret, "user-123", "test@example.com", false).unwrap();

    let claims = verify_token(secret, &token).unwrap();
    assert_eq!(claims.iss, Some("nexus".to_string()));
}

// 辅助函数：创建带自定义过期时间的 token
fn create_token_with_expiry(
    secret: &str,
    user_id: &str,
    email: &str,
    is_admin: bool,
    expires_at: chrono::DateTime<Utc>,
) -> Result<String, nexus_auth::error::AuthError> {
    use nexus_auth::jwt::encode_token;
    use chrono::Utc;

    let claims = TokenClaims {
        sub: user_id.to_string(),
        email: email.to_string(),
        is_admin,
        iss: Some("nexus".to_string()),
        iat: Utc::now().timestamp(),
        exp: expires_at.timestamp(),
    };

    encode_token(secret, &claims)
}
