//! 路由入口模块
//! 聚合所有子路由并提供健康检查端点
//!
//! 子模块：
//! - auth: 认证相关路由
//! - me: 用户个人中心路由
//! - admin: 管理后台路由
//! - v1: API v1 版本路由

pub mod v1;
pub mod auth;
pub mod me;
pub mod admin;

use axum::Json;

/// 健康检查端点
///
/// 返回服务状态信息
pub async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "service": "nexus-api",
        "version": env!("CARGO_PKG_VERSION")
    }))
}
