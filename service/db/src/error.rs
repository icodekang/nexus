//! 数据库错误模块

use thiserror::Error;

/// 数据库错误枚举
///
/// 包含数据库操作过程中可能出现的各种错误情况
#[derive(Error, Debug)]
pub enum DbError {
    /// SQL 数据库错误
    #[error("数据库错误: {0}")]
    Sqlx(#[from] sqlx::Error),

    /// 记录不存在
    #[error("记录不存在")]
    NotFound,

    /// 重复记录
    #[error("重复记录: {0}")]
    Duplicate(String),

    /// 无效数据
    #[error("无效数据: {0}")]
    InvalidData(String),

    /// 序列化错误
    #[error("序列化错误: {0}")]
    Serialization(#[from] serde_json::Error),
}

impl From<redis::RedisError> for DbError {
    fn from(err: redis::RedisError) -> Self {
        DbError::InvalidData(err.to_string())
    }
}
