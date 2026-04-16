//! 密码模块
//!
//! 提供密码的哈希和验证功能，使用 bcrypt 算法

use bcrypt::{hash, verify, DEFAULT_COST};

/// 使用 bcrypt 对密码进行哈希
///
/// # 参数
/// * `password` - 明文密码
///
/// # 返回
/// 哈希后的密码字符串
pub fn hash_password(password: &str) -> Result<String, bcrypt::BcryptError> {
    hash(password, DEFAULT_COST)
}

/// 验证密码与哈希是否匹配
///
/// # 参数
/// * `password` - 明文密码
/// * `hash` - 存储的哈希值
///
/// # 返回
/// 如果匹配返回 true
pub fn verify_password(password: &str, hash: &str) -> Result<bool, bcrypt::BcryptError> {
    verify(password, hash)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_and_verify() {
        let password = "test_password123";
        let hashed = hash_password(password).unwrap();
        
        assert!(verify_password(password, &hashed).unwrap());
        assert!(!verify_password("wrong_password", &hashed).unwrap());
    }
}
