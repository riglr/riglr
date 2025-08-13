//! Error types for authentication operations

use thiserror::Error;

/// Main error type for authentication operations
#[derive(Error, Debug)]
pub enum AuthError {
    /// Token validation failed
    #[error("Token validation failed: {0}")]
    TokenValidation(String),
    
    /// Missing required credentials
    #[error("Missing required credential: {0}")]
    MissingCredential(String),
    
    /// User not verified or authorized
    #[error("User not verified: {0}")]
    NotVerified(String),
    
    /// Network or API request failed
    #[error("API request failed: {0}")]
    ApiError(String),
    
    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    /// Unsupported operation
    #[error("Unsupported operation: {0}")]
    UnsupportedOperation(String),
    
    /// No wallet found for user
    #[error("No wallet found for user: {0}")]
    NoWallet(String),
    
    /// Generic error with source
    #[error("Authentication error: {0}")]
    Other(#[from] anyhow::Error),
}

impl AuthError {
    /// Check if error is retriable (network/transient issues)
    pub fn is_retriable(&self) -> bool {
        matches!(self, Self::ApiError(_))
    }
}

/// Result type alias for authentication operations
pub type AuthResult<T> = Result<T, AuthError>;