//! Error types for riglr-server

use thiserror::Error;

/// Result type alias for riglr-server operations
pub type Result<T> = std::result::Result<T, ServerError>;

/// Comprehensive error type for riglr-server operations
#[derive(Error, Debug)]
pub enum ServerError {
    #[error("Server binding error: {0}")]
    Bind(#[from] std::io::Error),
    
    #[error("Signer error: {0}")]
    Signer(#[from] riglr_core::signer::SignerError),
    
    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),
    
    #[error("Agent error: {0}")]
    Agent(String),
    
    #[error("Invalid request: {0}")]
    InvalidRequest(String),
    
    #[error("Configuration error: {0}")]
    Configuration(String),
    
    #[error("Internal server error: {0}")]
    Internal(String),
    
    #[error("Validation error: {0}")]
    Validation(String),
}

impl ServerError {
    /// Create a new agent error
    pub fn agent<T: std::fmt::Display>(msg: T) -> Self {
        Self::Agent(msg.to_string())
    }
    
    /// Create a new invalid request error
    pub fn invalid_request<T: std::fmt::Display>(msg: T) -> Self {
        Self::InvalidRequest(msg.to_string())
    }
    
    /// Create a new configuration error
    pub fn configuration<T: std::fmt::Display>(msg: T) -> Self {
        Self::Configuration(msg.to_string())
    }
    
    /// Create a new internal error
    pub fn internal<T: std::fmt::Display>(msg: T) -> Self {
        Self::Internal(msg.to_string())
    }
}

/// Convert ServerError to Actix-Web error response
impl actix_web::ResponseError for ServerError {
    fn error_response(&self) -> actix_web::HttpResponse {
        use actix_web::HttpResponse;
        use serde_json::json;
        
        let (status, error_type, message) = match self {
            ServerError::Bind(_) => (actix_web::http::StatusCode::INTERNAL_SERVER_ERROR, "bind_error", self.to_string()),
            ServerError::Signer(_) => (actix_web::http::StatusCode::BAD_REQUEST, "signer_error", self.to_string()),
            ServerError::Json(_) => (actix_web::http::StatusCode::BAD_REQUEST, "json_error", "Invalid JSON format".to_string()),
            ServerError::Agent(_) => (actix_web::http::StatusCode::INTERNAL_SERVER_ERROR, "agent_error", self.to_string()),
            ServerError::InvalidRequest(_) => (actix_web::http::StatusCode::BAD_REQUEST, "invalid_request", self.to_string()),
            ServerError::Configuration(_) => (actix_web::http::StatusCode::INTERNAL_SERVER_ERROR, "configuration_error", self.to_string()),
            ServerError::Internal(_) => (actix_web::http::StatusCode::INTERNAL_SERVER_ERROR, "internal_error", self.to_string()),
            ServerError::Validation(_) => (actix_web::http::StatusCode::BAD_REQUEST, "validation_error", self.to_string()),
        };
        
        HttpResponse::build(status).json(json!({
            "error": {
                "type": error_type,
                "message": message,
            }
        }))
    }
}

impl From<String> for ServerError {
    fn from(msg: String) -> Self {
        Self::Internal(msg)
    }
}

impl From<&str> for ServerError {
    fn from(msg: &str) -> Self {
        Self::Internal(msg.to_string())
    }
}