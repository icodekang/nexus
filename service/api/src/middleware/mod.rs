//! 中间件模块
//! 提供认证和授权相关的中间件
//!
//! 主要中间件：
//! - validate_api_key: 验证 API Key
//! - validate_jwt_or_api_key: 验证 JWT 或 API Key
//! - require_admin: 要求管理员权限

pub mod auth;

/// 认证上下文提取器
///
/// 从请求扩展中提取认证后的用户信息
pub use auth::AuthContext;
