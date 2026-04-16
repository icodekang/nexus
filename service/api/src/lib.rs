//! API 模块
//!
//! 核心模块包括：
//! - error: API 错误类型定义
//! - routes: HTTP 路由处理
//! - middleware: 认证和授权中间件
//! - state: 应用状态管理

pub mod error;
pub mod routes;
pub mod middleware;
pub mod state;
