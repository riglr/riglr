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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_validation_error_creation_and_display() {
        let error = AuthError::TokenValidation("Invalid JWT signature".to_string());
        assert_eq!(
            error.to_string(),
            "Token validation failed: Invalid JWT signature"
        );
    }

    #[test]
    fn test_token_validation_error_with_empty_message() {
        let error = AuthError::TokenValidation("".to_string());
        assert_eq!(error.to_string(), "Token validation failed: ");
    }

    #[test]
    fn test_missing_credential_error_creation_and_display() {
        let error = AuthError::MissingCredential("API key".to_string());
        assert_eq!(error.to_string(), "Missing required credential: API key");
    }

    #[test]
    fn test_missing_credential_error_with_empty_message() {
        let error = AuthError::MissingCredential("".to_string());
        assert_eq!(error.to_string(), "Missing required credential: ");
    }

    #[test]
    fn test_not_verified_error_creation_and_display() {
        let error = AuthError::NotVerified("Email not confirmed".to_string());
        assert_eq!(error.to_string(), "User not verified: Email not confirmed");
    }

    #[test]
    fn test_not_verified_error_with_empty_message() {
        let error = AuthError::NotVerified("".to_string());
        assert_eq!(error.to_string(), "User not verified: ");
    }

    #[test]
    fn test_api_error_creation_and_display() {
        let error = AuthError::ApiError("Connection timeout".to_string());
        assert_eq!(error.to_string(), "API request failed: Connection timeout");
    }

    #[test]
    fn test_api_error_with_empty_message() {
        let error = AuthError::ApiError("".to_string());
        assert_eq!(error.to_string(), "API request failed: ");
    }

    #[test]
    fn test_config_error_creation_and_display() {
        let error = AuthError::ConfigError("Invalid configuration file".to_string());
        assert_eq!(
            error.to_string(),
            "Configuration error: Invalid configuration file"
        );
    }

    #[test]
    fn test_config_error_with_empty_message() {
        let error = AuthError::ConfigError("".to_string());
        assert_eq!(error.to_string(), "Configuration error: ");
    }

    #[test]
    fn test_unsupported_operation_error_creation_and_display() {
        let error = AuthError::UnsupportedOperation("OAuth with this provider".to_string());
        assert_eq!(
            error.to_string(),
            "Unsupported operation: OAuth with this provider"
        );
    }

    #[test]
    fn test_unsupported_operation_error_with_empty_message() {
        let error = AuthError::UnsupportedOperation("".to_string());
        assert_eq!(error.to_string(), "Unsupported operation: ");
    }

    #[test]
    fn test_no_wallet_error_creation_and_display() {
        let error = AuthError::NoWallet("user123".to_string());
        assert_eq!(error.to_string(), "No wallet found for user: user123");
    }

    #[test]
    fn test_no_wallet_error_with_empty_message() {
        let error = AuthError::NoWallet("".to_string());
        assert_eq!(error.to_string(), "No wallet found for user: ");
    }

    #[test]
    fn test_other_error_from_anyhow_error() {
        let anyhow_error = anyhow::anyhow!("Something went wrong");
        let auth_error = AuthError::Other(anyhow_error);
        assert_eq!(
            auth_error.to_string(),
            "Authentication error: Something went wrong"
        );
    }

    #[test]
    fn test_other_error_from_trait_implementation() {
        let anyhow_error = anyhow::anyhow!("Network failure");
        let auth_error: AuthError = anyhow_error.into();
        assert_eq!(
            auth_error.to_string(),
            "Authentication error: Network failure"
        );
    }

    #[test]
    fn test_is_retriable_returns_true_for_api_error() {
        let error = AuthError::ApiError("Network timeout".to_string());
        assert!(error.is_retriable());
    }

    #[test]
    fn test_is_retriable_returns_false_for_token_validation() {
        let error = AuthError::TokenValidation("Invalid token".to_string());
        assert!(!error.is_retriable());
    }

    #[test]
    fn test_is_retriable_returns_false_for_missing_credential() {
        let error = AuthError::MissingCredential("API key".to_string());
        assert!(!error.is_retriable());
    }

    #[test]
    fn test_is_retriable_returns_false_for_not_verified() {
        let error = AuthError::NotVerified("Email pending".to_string());
        assert!(!error.is_retriable());
    }

    #[test]
    fn test_is_retriable_returns_false_for_config_error() {
        let error = AuthError::ConfigError("Bad config".to_string());
        assert!(!error.is_retriable());
    }

    #[test]
    fn test_is_retriable_returns_false_for_unsupported_operation() {
        let error = AuthError::UnsupportedOperation("Feature disabled".to_string());
        assert!(!error.is_retriable());
    }

    #[test]
    fn test_is_retriable_returns_false_for_no_wallet() {
        let error = AuthError::NoWallet("user456".to_string());
        assert!(!error.is_retriable());
    }

    #[test]
    fn test_is_retriable_returns_false_for_other_error() {
        let anyhow_error = anyhow::anyhow!("Some other error");
        let error = AuthError::Other(anyhow_error);
        assert!(!error.is_retriable());
    }

    #[test]
    fn test_auth_result_type_alias_ok() {
        let result: AuthResult<String> = Ok("success".to_string());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");
    }

    #[test]
    fn test_auth_result_type_alias_err() {
        let result: AuthResult<String> = Err(AuthError::TokenValidation("bad token".to_string()));
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Token validation failed: bad token"
        );
    }

    #[test]
    fn test_debug_trait_implementation() {
        let error = AuthError::TokenValidation("debug test".to_string());
        let debug_output = format!("{:?}", error);
        assert!(debug_output.contains("TokenValidation"));
        assert!(debug_output.contains("debug test"));
    }

    #[test]
    fn test_error_trait_source_for_other_variant() {
        use std::error::Error;
        let anyhow_error = anyhow::anyhow!("root cause");
        let auth_error = AuthError::Other(anyhow_error);
        assert!(auth_error.source().is_some());
    }

    #[test]
    fn test_error_trait_source_for_non_other_variants() {
        use std::error::Error;
        let error = AuthError::TokenValidation("no source".to_string());
        assert!(error.source().is_none());
    }
}
