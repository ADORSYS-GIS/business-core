use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Validation error: {0}")]
    ValidationError(String),
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Internal error: {0}")]
    InternalError(String),
    
    #[error("Database error: {0}")]
    DatabaseError(String),
}

pub type ApiResult<T> = Result<T, ApiError>;