use core::result::Result as StdResult;
use ethers_core::types::SignatureError;
use riglr_core::error::ToolError;
use thiserror::Error;

/// Main error type for Asterdex tool operations
#[derive(Error, Debug)]
pub enum Error {
    /// API error returned by Asterdex exchange
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

    /// Invalid input parameters
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// Network connectivity error
    #[error("Network error: {0}")]
    NetworkError(String),

    /// Trading order related error
    #[error("Order error: {0}")]
    OrderError(String),

    /// Rate limit exceeded by Asterdex API
    #[error("Rate limited: {0}")]
    RateLimit(String),

    /// Signature or signing error for v3 authentication
    #[error("Signature error: {0}")]
    SignatureError(String),

    /// Reduce-only order was rejected (-2022)
    #[error("Reduce-only order rejected: {0}")]
    ReduceOnlyRejected(String),

    /// Order's notional value is too small (-4164)
    #[error("Order notional value is below the minimum: {0}")]
    MinNotionalNotMet(String),

    /// Order would trigger immediately (-2021)
    #[error("Order would trigger immediately: {0}")]
    OrderWouldImmediatelyTrigger(String),
}

/// Result type alias for Asterdex tool operations.
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
            | Error::Configuration(msg)
            | Error::InvalidInput(msg)
            | Error::SignatureError(msg)
            | Error::ReduceOnlyRejected(msg)
            | Error::MinNotionalNotMet(msg)
            | Error::OrderWouldImmediatelyTrigger(msg) => Self::permanent_string(msg),
            Error::OrderError(msg) => Self::retriable_string(msg),
        }
    }
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            Self::NetworkError(format!("Request timeout: {err}"))
        } else if err.is_connect() {
            Self::NetworkError(format!("Connection error: {err}"))
        } else {
            Self::NetworkError(err.to_string())
        }
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Self::ApiError(format!("JSON parsing error: {err}"))
    }
}

impl From<SignatureError> for Error {
    fn from(err: SignatureError) -> Self {
        Self::SignatureError(format!("ECDSA signature error: {err}"))
    }
}

impl From<ethabi::Error> for Error {
    fn from(err: ethabi::Error) -> Self {
        Self::SignatureError(format!("ABI encoding error: {err}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display_messages() {
        let test_cases = vec![
            (
                Error::ApiError("test api error".to_string()),
                "API error: test api error",
            ),
            (
                Error::AuthError("auth failed".to_string()),
                "Authentication error: auth failed",
            ),
            (
                Error::Configuration("bad config".to_string()),
                "Configuration error: bad config",
            ),
            (
                Error::InsufficientBalance("not enough".to_string()),
                "Insufficient balance: not enough",
            ),
            (
                Error::InvalidSymbol("INVALID".to_string()),
                "Invalid symbol: INVALID",
            ),
            (
                Error::InvalidInput("bad input".to_string()),
                "Invalid input: bad input",
            ),
            (
                Error::NetworkError("timeout".to_string()),
                "Network error: timeout",
            ),
            (
                Error::OrderError("order failed".to_string()),
                "Order error: order failed",
            ),
            (
                Error::RateLimit("too many".to_string()),
                "Rate limited: too many",
            ),
            (
                Error::SignatureError("sig error".to_string()),
                "Signature error: sig error",
            ),
        ];

        for (error, expected) in test_cases {
            assert_eq!(error.to_string(), expected);
        }
    }

    #[test]
    fn test_error_debug_trait() {
        let error = Error::ApiError("debug test".to_string());
        let debug_str = format!("{error:?}");
        assert!(debug_str.contains("ApiError"));
        assert!(debug_str.contains("debug test"));
    }

    #[test]
    fn test_result_type_alias() {
        let ok_result: Result<i32> = Ok(42);
        assert!(ok_result.is_ok());
        if let Ok(value) = ok_result {
            assert_eq!(value, 42);
        }

        let err_result: Result<i32> = Err(Error::ApiError("test".to_string()));
        assert!(err_result.is_err());
        assert!(matches!(err_result, Err(Error::ApiError(_))));
    }

    #[test]
    fn test_tool_error_conversion_rate_limit() {
        let error = Error::RateLimit("rate limit exceeded".to_string());
        let tool_error: ToolError = error.into();
        assert!(matches!(tool_error, ToolError::RateLimited { .. }));
    }

    #[test]
    fn test_tool_error_conversion_network_error_invalid_host() {
        let error = Error::NetworkError("invalid host specified".to_string());
        let tool_error: ToolError = error.into();
        assert!(matches!(tool_error, ToolError::Permanent { .. }));
    }

    #[test]
    fn test_tool_error_conversion_network_error_retriable() {
        let error = Error::NetworkError("connection timeout".to_string());
        let tool_error: ToolError = error.into();
        assert!(matches!(tool_error, ToolError::Retriable { .. }));
    }

    #[test]
    fn test_tool_error_conversion_api_error_rate_limit() {
        let test_cases = vec![
            Error::ApiError("HTTP 429 error".to_string()),
            Error::ApiError("rate limit exceeded".to_string()),
        ];

        for error in test_cases {
            let tool_error: ToolError = error.into();
            assert!(matches!(tool_error, ToolError::RateLimited { .. }));
        }
    }

    #[test]
    fn test_tool_error_conversion_api_error_retriable() {
        let test_cases = vec![
            Error::ApiError("HTTP 503 error".to_string()),
            Error::ApiError("service unavailable".to_string()),
        ];

        for error in test_cases {
            let tool_error: ToolError = error.into();
            assert!(matches!(tool_error, ToolError::Retriable { .. }));
        }
    }

    #[test]
    fn test_tool_error_conversion_api_error_permanent() {
        let error = Error::ApiError("HTTP 400 Bad Request".to_string());
        let tool_error: ToolError = error.into();
        assert!(matches!(tool_error, ToolError::Permanent { .. }));
    }

    #[test]
    fn test_tool_error_conversion_permanent_errors() {
        let errors = vec![
            Error::AuthError("unauthorized".to_string()),
            Error::InvalidSymbol("INVALID".to_string()),
            Error::InsufficientBalance("not enough".to_string()),
            Error::Configuration("bad config".to_string()),
            Error::InvalidInput("bad input".to_string()),
            Error::SignatureError("sig failed".to_string()),
        ];

        for error in errors {
            let tool_error: ToolError = error.into();
            assert!(matches!(tool_error, ToolError::Permanent { .. }));
        }
    }

    #[test]
    fn test_tool_error_conversion_order_error_retriable() {
        let error = Error::OrderError("order processing failed".to_string());
        let tool_error: ToolError = error.into();
        assert!(matches!(tool_error, ToolError::Retriable { .. }));
    }

    #[test]
    fn test_from_reqwest_timeout_error() {
        // Create a mock timeout error scenario
        let error_msg = "request timed out";
        let asterdex_error = Error::NetworkError(format!("Request timeout: {error_msg}"));
        assert!(asterdex_error.to_string().contains("timeout"));
    }

    #[test]
    fn test_from_reqwest_connect_error() {
        // Test connection error conversion
        let error_msg = "failed to connect";
        let asterdex_error = Error::NetworkError(format!("Connection error: {error_msg}"));
        assert!(asterdex_error.to_string().contains("Connection"));
    }

    #[test]
    #[allow(clippy::unwrap_used, clippy::panic)]
    fn test_from_serde_json_error() {
        let json_err = serde_json::from_str::<String>("invalid json").unwrap_err();
        let asterdex_error = Error::from(json_err);
        match asterdex_error {
            Error::ApiError(msg) => assert!(msg.contains("JSON parsing error")),
            _ => panic!("Expected ApiError"),
        }
    }

    #[test]
    fn test_from_ethers_signature_error() {
        // Test signature error conversion
        let error_msg = "invalid signature";
        let asterdex_error = Error::SignatureError(format!("ECDSA signature error: {error_msg}"));
        assert!(asterdex_error.to_string().contains("ECDSA"));
    }

    #[test]
    fn test_from_ethabi_error() {
        // Test ABI encoding error conversion
        let error_msg = "encoding failed";
        let asterdex_error = Error::SignatureError(format!("ABI encoding error: {error_msg}"));
        assert!(asterdex_error.to_string().contains("ABI encoding"));
    }

    #[test]
    fn test_error_empty_strings() {
        let errors = vec![
            Error::ApiError(String::new()),
            Error::AuthError(String::new()),
            Error::Configuration(String::new()),
            Error::InsufficientBalance(String::new()),
            Error::InvalidSymbol(String::new()),
            Error::InvalidInput(String::new()),
            Error::NetworkError(String::new()),
            Error::OrderError(String::new()),
            Error::RateLimit(String::new()),
            Error::SignatureError(String::new()),
        ];

        for error in errors {
            let error_string = error.to_string();
            assert!(!error_string.is_empty());
            assert!(error_string.contains(':'));
        }
    }

    #[test]
    fn test_error_case_sensitivity_in_conversion() {
        // Test that case matters for special keyword detection
        let errors = vec![
            (Error::ApiError("Rate Limit".to_string()), true), // Should be Permanent (no special handling)
            (Error::ApiError("Service Unavailable".to_string()), true), // Should be Permanent (no special handling)
            (Error::NetworkError("Invalid Host".to_string()), false), // Should be Retriable (default for network errors)
        ];

        for (error, should_be_permanent) in errors {
            let tool_error: ToolError = error.into();
            if should_be_permanent {
                assert!(matches!(tool_error, ToolError::Permanent { .. }));
            } else {
                // NetworkError without "invalid host" becomes Retriable
                assert!(matches!(tool_error, ToolError::Retriable { .. }));
            }
        }
    }

    #[test]
    fn test_multiple_keyword_matching() {
        // Test that multiple keywords in message work correctly
        let error = Error::ApiError("HTTP 429 rate limit exceeded".to_string());
        let tool_error: ToolError = error.into();
        // Should match the first condition (429)
        assert!(matches!(tool_error, ToolError::RateLimited { .. }));
    }

    #[test]
    fn test_partial_keyword_matching() {
        // Test partial matches work correctly
        let error = Error::NetworkError("The invalid host was rejected".to_string());
        let tool_error: ToolError = error.into();
        // Should match "invalid host" substring
        assert!(matches!(tool_error, ToolError::Permanent { .. }));
    }
}
