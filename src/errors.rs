use thiserror::Error;

#[derive(Error, Debug)]
pub enum OracleError {
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),
    
    #[error("JSON parsing error: {0}")]
    JsonError(#[from] serde_json::Error),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("API error: {0}")]
    ApiError(String),
    
    #[error("Price not found for symbol: {0}")]
    PriceNotFound(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("IO operation failed: {0}")]
    IoOperationFailed(String),
    
}

pub type Result<T> = std::result::Result<T, OracleError>;