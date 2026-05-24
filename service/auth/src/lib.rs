//! 认证模块
//!
//! 提供用户认证和 API Key 验证功能
//!
//! # 主要功能
//! - JWT Token 生成和验证
//! - API Key 生成和验证
//! - 密码哈希和验证
//! - Bearer Token 验证

pub mod error;
pub mod jwt;
pub mod keygen;
pub mod password;
pub mod validator;

pub use error::*;
pub use jwt::*;
pub use keygen::*;
pub use password::*;
pub use validator::*;
