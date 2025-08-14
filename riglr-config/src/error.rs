//! Configuration error types

use thiserror::Error;

/// Configuration result type
pub type ConfigResult<T> = Result<T, ConfigError>;

/// Configuration errors
#[derive(Debug, Error)]
pub enum ConfigError {
    /// Environment variable not found
    #[error("Missing environment variable: {0}")]
    MissingEnvVar(String),
    
    /// Invalid configuration value
    #[error("Invalid configuration: {0}")]
    ValidationError(String),
    
    /// Failed to parse configuration
    #[error("Failed to parse configuration: {0}")]
    ParseError(String),
    
    /// IO error
    #[error("IO error: {0}")]
    IoError(String),
    
    /// Chain not supported
    #[error("Chain {0} is not supported")]
    ChainNotSupported(u64),
    
    /// Provider not configured
    #[error("Provider {0} is not configured")]
    ProviderNotConfigured(String),
    
    /// Generic error
    #[error("{0}")]
    Generic(String),
}

impl ConfigError {
    /// Create a validation error
    pub fn validation<S: Into<String>>(msg: S) -> Self {
        Self::ValidationError(msg.into())
    }
    
    /// Create a parse error
    pub fn parse<S: Into<String>>(msg: S) -> Self {
        Self::ParseError(msg.into())
    }
    
    /// Create an IO error
    pub fn io<S: Into<String>>(msg: S) -> Self {
        Self::IoError(msg.into())
    }
    
    /// Create a generic error
    pub fn generic<S: Into<String>>(msg: S) -> Self {
        Self::Generic(msg.into())
    }
}

impl From<std::io::Error> for ConfigError {
    fn from(err: std::io::Error) -> Self {
        Self::IoError(err.to_string())
    }
}

impl From<toml::de::Error> for ConfigError {
    fn from(err: toml::de::Error) -> Self {
        Self::ParseError(err.to_string())
    }
}

impl From<envy::Error> for ConfigError {
    fn from(err: envy::Error) -> Self {
        Self::ParseError(err.to_string())
    }
}