//! 密码哈希测试
//!
//! 测试密码加密和验证功能

use nexus_auth::password::{hash_password, verify_password};

#[test]
fn test_password_hash_and_verify() {
    let password = "SecurePassword123!";
    let hashed = hash_password(password).unwrap();

    // 哈希后的密码不应等于原密码
    assert_ne!(hashed, password);

    // 验证正确密码
    assert!(verify_password(password, &hashed).unwrap());
}

#[test]
fn test_password_verify_wrong_password() {
    let password = "SecurePassword123!";
    let hashed = hash_password(password).unwrap();

    // 验证错误密码
    assert!(!verify_password("WrongPassword", &hashed).unwrap());
}

#[test]
fn test_different_passwords_different_hashes() {
    let password1 = "Password1";
    let password2 = "Password2";

    let hash1 = hash_password(password1).unwrap();
    let hash2 = hash_password(password2).unwrap();

    assert_ne!(hash1, hash2);
}

#[test]
fn test_same_password_different_hashes() {
    // 由于 bcrypt 使用随机 salt，同一密码应产生不同哈希
    let password = "SamePassword";

    let hash1 = hash_password(password).unwrap();
    let hash2 = hash_password(password).unwrap();

    assert_ne!(hash1, hash2);
    // 但两者都应能验证通过
    assert!(verify_password(password, &hash1).unwrap());
    assert!(verify_password(password, &hash2).unwrap());
}

#[test]
fn test_empty_password() {
    let result = hash_password("");
    // 空密码应该失败或返回错误
    assert!(result.is_err() || result.unwrap() == "".to_string());
}

#[test]
fn test_very_long_password() {
    let long_password = "a".repeat(1000);
    let hashed = hash_password(&long_password).unwrap();
    assert!(verify_password(&long_password, &hashed).unwrap());
}
