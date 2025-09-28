use core::result::Result as StdResult;
use riglr_core::error::ToolError;
use thiserror::Error;

/// Main error type for Hyperliquid tool operations
#[derive(Error, Debug)]
pub enum Error {
    /// API error returned by Hyperliquid exchange
    #[error("API error: {0}")]
    ApiError(String),

    /// Authentication or authorization error
    #[error("Authentication error: {0}")]
    AuthError(String),

    /// Configuration or setup error
    #[error("Configuration error: {0}")]
    Configuration(String),

    /// Insufficient balance for requested operation
    #[error("Insufficient balance: {0}")]
    InsufficientBalance(String),

    /// Invalid trading symbol provided
    #[error("Invalid symbol: {0}")]
    InvalidSymbol(String),

    /// Network connectivity error
    #[error("Network error: {0}")]
    NetworkError(String),

    /// Trading order related error
    #[error("Order error: {0}")]
    OrderError(String),

    /// Rate limit exceeded by Hyperliquid API
    #[error("Rate limited: {0}")]
    RateLimit(String),
}

/// Result type alias for Hyperliquid tool operations.
pub type Result<T> = StdResult<T, Error>;

impl From<Error> for ToolError {
    fn from(err: Error) -> Self {
        match err {
            Error::RateLimit(msg) => {
                // RateLimit errors are already properly categorized
                Self::rate_limited_string(msg)
            }
            Error::NetworkError(msg) => {
                // Check if it's a permanent network error
                if msg.contains("invalid host") {
                    return Self::permanent_string(msg);
                }
                // Most network errors are retriable
                Self::retriable_string(msg)
            }
            Error::ApiError(msg) => {
                // Check the error message for specific HTTP status codes
                if msg.contains("429") || msg.contains("rate limit") {
                    return Self::rate_limited_string(msg);
                } else if msg.contains("503") || msg.contains("service unavailable") {
                    return Self::retriable_string(msg);
                }
                // Other API errors (like 400) are permanent
                Self::permanent_string(msg)
            }
            Error::AuthError(msg)
            | Error::InvalidSymbol(msg)
            | Error::InsufficientBalance(msg)
            | Error::Configuration(msg) => Self::permanent_string(msg),
            Error::OrderError(msg) => Self::retriable_string(msg),
        }
    }
}

/*
#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn test_hyperliquid_tool_error_api_error_display() {
        let error = Error::ApiError("Invalid request".to_string());
        assert_eq!(format!("{}", error), "API error: Invalid request");
    }

    #[test]
    fn test_hyperliquid_tool_error_invalid_symbol_display() {
        let error = Error::InvalidSymbol("INVALID".to_string());
        assert_eq!(format!("{}", error), "Invalid symbol: INVALID");
    }

    #[test]
    fn test_hyperliquid_tool_error_network_error_display() {
        let error = Error::NetworkError("Connection timeout".to_string());
        assert_eq!(format!("{}", error), "Network error: Connection timeout");
    }

    #[test]
    fn test_hyperliquid_tool_error_rate_limit_display() {
        let error = Error::RateLimit("Too many requests".to_string());
        assert_eq!(format!("{}", error), "Rate limited: Too many requests");
    }

    #[test]
    fn test_hyperliquid_tool_error_auth_error_display() {
        let error = Error::AuthError("Invalid credentials".to_string());
        assert_eq!(format!("{}", error), "Authentication error: Invalid credentials");
    }

    #[test]
    fn test_hyperliquid_tool_error_insufficient_balance_display() {
        let error = Error::InsufficientBalance("Not enough funds".to_string());
        assert_eq!(format!("{}", error), "Insufficient balance: Not enough funds");
    }

    #[test]
    fn test_hyperliquid_tool_error_order_error_display() {
        let error = Error::OrderError("Order failed".to_string());
        assert_eq!(format!("{}", error), "Order error: Order failed");
    }

    #[test]
    fn test_hyperliquid_tool_error_configuration_display() {
        let error = Error::Configuration("Invalid config".to_string());
        assert_eq!(format!("{}", error), "Configuration error: Invalid config");
    }

    #[test]
    fn test_from_rate_limit_error() {
        let error = Error::RateLimit("Rate limit exceeded".to_string());
        let tool_error: riglr_core::error::ToolError = error.into();
        assert!(matches!(tool_error, riglr_core::error::ToolError::RateLimit(_)));
    }

    #[test]
    fn test_from_network_error_with_invalid_host() {
        let error = Error::NetworkError("invalid host specified".to_string());
        let tool_error: riglr_core::error::ToolError = error.into();
        assert!(matches!(tool_error, riglr_core::error::ToolError::Permanent(_)));
    }

    #[test]
    fn test_from_network_error_retriable() {
        let error = Error::NetworkError("Connection timeout".to_string());
        let tool_error: riglr_core::error::ToolError = error.into();
        assert!(matches!(tool_error, riglr_core::error::ToolError::Retriable(_)));
    }

    #[test]
    fn test_from_api_error_with_429_status() {
        let error = Error::ApiError("HTTP 429 error".to_string());
        let tool_error: riglr_core::error::ToolError = error.into();
        assert!(matches!(tool_error, riglr_core::error::ToolError::RateLimit(_)));
    }

    #[test]
    fn test_from_api_error_with_rate_limit_message() {
        let error = Error::ApiError("rate limit exceeded".to_string());
        let tool_error: riglr_core::error::ToolError = error.into();
        assert!(matches!(tool_error, riglr_core::error::ToolError::RateLimit(_)));
    }

    #[test]
    fn test_from_api_error_with_503_status() {
        let error = Error::ApiError("HTTP 503 error".to_string());
        let tool_error: riglr_core::error::ToolError = error.into();
        assert!(matches!(tool_error, riglr_core::error::ToolError::Retriable(_)));
    }

    #[test]
    fn test_from_api_error_with_service_unavailable() {
        let error = Error::ApiError("service unavailable".to_string());
        let tool_error: riglr_core::error::ToolError = error.into();
        assert!(matches!(tool_error, riglr_core::error::ToolError::Retriable(_)));
    }

    #[test]
    fn test_from_api_error_permanent() {
        let error = Error::ApiError("HTTP 400 Bad Request".to_string());
        let tool_error: riglr_core::error::ToolError = error.into();
        assert!(matches!(tool_error, riglr_core::error::ToolError::Permanent(_)));
    }

    #[test]
    fn test_from_auth_error() {
        let error = Error::AuthError("Invalid credentials".to_string());
        let tool_error: riglr_core::error::ToolError = error.into();
        assert!(matches!(tool_error, riglr_core::error::ToolError::Permanent(_)));
    }

    #[test]
    fn test_from_invalid_symbol_error() {
        let error = Error::InvalidSymbol("INVALID".to_string());
        let tool_error: riglr_core::error::ToolError = error.into();
        assert!(matches!(tool_error, riglr_core::error::ToolError::Permanent(_)));
    }

    #[test]
    fn test_from_insufficient_balance_error() {
        let error = Error::InsufficientBalance("Not enough funds".to_string());
        let tool_error: riglr_core::error::ToolError = error.into();
        assert!(matches!(tool_error, riglr_core::error::ToolError::Permanent(_)));
    }

    #[test]
    fn test_from_order_error() {
        let error = Error::OrderError("Order failed".to_string());
        let tool_error: riglr_core::error::ToolError = error.into();
        assert!(matches!(tool_error, riglr_core::error::ToolError::Retriable(_)));
    }

    #[test]
    fn test_from_configuration_error() {
        let error = Error::Configuration("Invalid config".to_string());
        let tool_error: riglr_core::error::ToolError = error.into();
        assert!(matches!(tool_error, riglr_core::error::ToolError::Permanent(_)));
    }

    #[test]
    fn test_result_type_alias_ok() {
        let result: Result<i32> = Ok(42);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_result_type_alias_err() {
        let result: Result<i32> = Err(Error::ApiError("Test error".to_string()));
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::ApiError(_)));
    }

    #[test]
    fn test_error_debug_trait() {
        let error = Error::ApiError("Debug test".to_string());
        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("ApiError"));
        assert!(debug_str.contains("Debug test"));
    }

    #[test]
    fn test_error_variants_with_empty_strings() {
        let errors = vec![
            Error::ApiError("".to_string()),
            Error::InvalidSymbol("".to_string()),
            Error::NetworkError("".to_string()),
            Error::RateLimit("".to_string()),
            Error::AuthError("".to_string()),
            Error::InsufficientBalance("".to_string()),
            Error::OrderError("".to_string()),
            Error::Configuration("".to_string()),
        ];

        for error in errors {
            let error_string = format!("{}", error);
            assert!(!error_string.is_empty());
        }
    }

    #[test]
    fn test_from_network_error_edge_case_partial_match() {
        let error = Error::NetworkError("The invalid host was rejected".to_string());
        let tool_error: riglr_core::error::ToolError = error.into();
        assert!(matches!(tool_error, riglr_core::error::ToolError::Permanent(_)));
    }

    #[test]
    fn test_from_api_error_case_sensitivity() {
        let errors = vec![
            Error::ApiError("Rate Limit exceeded".to_string()), // Capital R, L
            Error::ApiError("Service Unavailable".to_string()), // Capital S, U
        ];

        for error in errors {
            let tool_error: riglr_core::error::ToolError = error.into();
            // Should not match due to case sensitivity, should be permanent
            assert!(matches!(tool_error, riglr_core::error::ToolError::Permanent(_)));
        }
    }

    #[test]
    fn test_from_api_error_multiple_keywords() {
        let error = Error::ApiError("HTTP 429 rate limit exceeded".to_string());
        let tool_error: riglr_core::error::ToolError = error.into();
        // Should match the first condition (429)
        assert!(matches!(tool_error, riglr_core::error::ToolError::RateLimit(_)));
    }
}
*/
