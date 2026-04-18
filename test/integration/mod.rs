//! Integration Tests Module
//!
//! 本模块包含项目的集成测试，覆盖所有 API 端点和功能
//!
//! 运行测试:
//! ```bash
//! # 运行所有集成测试
//! cargo test --test integration -- --nocapture
//!
//! # 运行特定测试
//! cargo test --test backend_api_test -- --nocapture
//! cargo test --test client_api_test -- --nocapture
//! ```
//!
//! 注意: 集成测试需要后端服务运行在 localhost:8080
//! 可以设置环境变量指定 API 地址:
//! ```bash
//! NEXUS_API_BASE_URL=http://localhost:8080 cargo test
//! ```

mod backend_api_test;
mod client_api_test;

pub use backend_api_test::*;
pub use client_api_test::*;
