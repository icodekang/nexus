use thiserror::Error;

#[derive(Error, Debug)]
pub enum DbError {
    #[error("Database error: {0}")]
    Sqlx(#[from] sqlx::Error),
    
    #[error("Record not found")]
    NotFound,
    
    #[error("Duplicate record: {0}")]
    Duplicate(String),
    
    #[error("Invalid data: {0}")]
    InvalidData(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

impl From<redis::RedisError> for DbError {
    fn from(err: redis::RedisError) -> Self {
        DbError::InvalidData(err.to_string())
    }
}
