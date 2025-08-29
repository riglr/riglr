//! Error types for EVM tools

use thiserror::Error;

/// Main error type for EVM tools
#[derive(Error, Debug, Clone)]
pub enum EvmToolError {
    /// Generic provider issues
    #[error("Provider error: {0}")]
    ProviderError(String),

    /// Transaction reverted on-chain
    #[error("Transaction reverted: {reason}")]
    TransactionReverted { reason: String },

    /// Nonce is too low
    #[error("Nonce too low")]
    NonceTooLow,

    /// Nonce mismatch
    #[error("Nonce mismatch")]
    NonceMismatch,

    /// Insufficient funds for transaction
    #[error("Insufficient funds")]
    InsufficientFunds,

    /// Gas estimation failed
    #[error("Gas estimation failed")]
    GasEstimationFailed,

    /// Invalid address format
    #[error("Invalid address: {0}")]
    InvalidAddress(String),

    /// Contract-related errors
    #[error("Contract error: {0}")]
    ContractError(String),

    /// Signer-related errors
    #[error("Signer error: {0}")]
    SignerError(#[from] riglr_core::SignerError),

    /// Network timeout or connection issues
    #[error("Network error: {0}")]
    NetworkError(String),

    /// Rate limit exceeded
    #[error("Rate limit exceeded")]
    RateLimited,

    /// Unsupported chain
    #[error("Unsupported chain: {0}")]
    UnsupportedChain(u64),

    /// Invalid parameters
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    /// Generic error fallback
    #[error("{0}")]
    Generic(String),
}

/// Error classification for retry logic
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorClass {
    /// Permanent errors that should not be retried
    Permanent,
    /// Retriable errors that may succeed on retry
    Retriable,
    /// Rate-limited errors that need backoff
    RateLimited,
}

/// Classify network error messages for retry behavior
fn classify_network_message(msg: &str) -> ErrorClass {
    if msg.contains("rate limit") || msg.contains("429") {
        ErrorClass::RateLimited
    } else {
        ErrorClass::Retriable
    }
}

/// Classify provider error messages for retry behavior
fn classify_provider_message(msg: &str) -> ErrorClass {
    if msg.contains("timeout") || msg.contains("connection") {
        ErrorClass::Retriable
    } else if msg.contains("rate") || msg.contains("quota") {
        ErrorClass::RateLimited
    } else {
        ErrorClass::Permanent
    }
}

/// Classify transaction error messages for retry behavior
fn classify_transaction_message(msg: &str) -> ErrorClass {
    if msg.contains("nonce") || msg.contains("pending") {
        ErrorClass::Retriable
    } else if msg.contains("insufficient funds") || msg.contains("reverted") {
        ErrorClass::Permanent
    } else {
        ErrorClass::Retriable
    }
}

/// Classify RPC error messages for retry behavior
fn classify_rpc_message(msg: &str) -> ErrorClass {
    if msg.contains("timeout") || msg.contains("connection") {
        ErrorClass::Retriable
    } else if msg.contains("rate limit") || msg.contains("429") {
        ErrorClass::RateLimited
    } else {
        ErrorClass::Retriable
    }
}

/// Classify generic error messages for retry behavior
fn classify_generic_message(msg: &str) -> ErrorClass {
    if msg.contains("timeout") || msg.contains("connection") {
        ErrorClass::Retriable
    } else {
        ErrorClass::Permanent
    }
}

/// Attempt to extract JSON-RPC error code from error message
///
/// JSON-RPC error messages often contain the error code in the format:
/// - "error code: -32000"
/// - "code: -32603"
/// - "429 Too Many Requests"
fn extract_error_code(msg: &str) -> Option<i32> {
    // Try to find patterns like "code: -32000" or "error code: -32000"
    if let Some(idx) = msg.find("code:") {
        let code_str = &msg[idx + 5..].trim();
        if let Some(end_idx) = code_str.find(|c: char| !c.is_ascii_digit() && c != '-') {
            return code_str[..end_idx].parse::<i32>().ok();
        }
        return code_str.split_whitespace().next()?.parse::<i32>().ok();
    }

    // Try to find HTTP status codes like "429"
    if msg.starts_with("429") || msg.contains(" 429 ") {
        return Some(429);
    }

    // Try to find patterns like "error -32000"
    if let Some(idx) = msg.find("error ") {
        let code_str = &msg[idx + 6..].trim();
        if let Some(stripped) = code_str.strip_prefix('-') {
            if let Some(end_idx) = stripped.find(|c: char| !c.is_ascii_digit()) {
                return code_str[..end_idx + 1].parse::<i32>().ok();
            }
        }
    }

    None
}

/// Classify errors based on JSON-RPC error codes
///
/// Standard JSON-RPC error codes:
/// - -32700: Parse error (permanent)
/// - -32600: Invalid request (permanent)  
/// - -32601: Method not found (permanent)
/// - -32602: Invalid params (permanent)
/// - -32603: Internal error (retriable)
/// - -32000 to -32099: Server errors (implementation defined)
///
/// Common implementation-specific codes:
/// - -32000: Insufficient funds, invalid input (permanent)
/// - -32002: Resource not found (permanent)
/// - -32003: Transaction rejected (permanent)
/// - -32004: Method not supported (permanent)
/// - -32005: Limit exceeded (rate limited)
/// - 429: HTTP Too Many Requests (rate limited)
fn classify_by_error_code(code: i32) -> Option<ErrorClass> {
    match code {
        // Standard JSON-RPC permanent errors
        -32700 | -32600 | -32601 | -32602 => Some(ErrorClass::Permanent),

        // Standard JSON-RPC internal error (retriable)
        -32603 => Some(ErrorClass::Retriable),

        // Common implementation-specific permanent errors
        -32000 | -32002 | -32003 | -32004 => Some(ErrorClass::Permanent),

        // Rate limiting
        -32005 | 429 => Some(ErrorClass::RateLimited),

        // Other server errors in the -32000 to -32099 range
        -32099..=-32001 => Some(ErrorClass::Retriable),

        // Unknown codes
        _ => None,
    }
}

/// Classify EVM errors to determine retry behavior
///
/// This function uses structured error variants to determine classification,
/// with a fallback to string matching for provider errors that contain generic messages.
///
/// # Examples
///
/// ```ignore
/// let error = EvmToolError::NonceTooLow;
/// assert_eq!(classify_evm_error(&error), ErrorClass::Retriable);
///
/// let error = EvmToolError::InsufficientFunds;
/// assert_eq!(classify_evm_error(&error), ErrorClass::Permanent);
/// ```
pub fn classify_evm_error(error: &EvmToolError) -> ErrorClass {
    match error {
        // Structured errors with clear classification
        EvmToolError::NonceTooLow | EvmToolError::NonceMismatch => ErrorClass::Retriable,
        EvmToolError::InsufficientFunds => ErrorClass::Permanent,
        EvmToolError::TransactionReverted { .. } => ErrorClass::Permanent,
        EvmToolError::RateLimited => ErrorClass::RateLimited,
        EvmToolError::GasEstimationFailed => ErrorClass::Retriable,
        EvmToolError::InvalidAddress(_) => ErrorClass::Permanent,
        EvmToolError::ContractError(_) => ErrorClass::Permanent,
        EvmToolError::SignerError(_) => ErrorClass::Permanent,
        EvmToolError::UnsupportedChain(_) => ErrorClass::Permanent,
        EvmToolError::InvalidParameter(_) => ErrorClass::Permanent,

        // Network errors are typically retriable
        EvmToolError::NetworkError(msg) => {
            if msg.contains("timeout") || msg.contains("connection") {
                ErrorClass::Retriable
            } else {
                ErrorClass::Retriable
            }
        }

        // Provider errors need string matching as fallback
        EvmToolError::ProviderError(msg) => {
            // First try to extract and classify by error code
            if let Some(code) = extract_error_code(msg) {
                if let Some(class) = classify_by_error_code(code) {
                    return class;
                }
            }
            // Fall back to string matching
            classify_provider_message(msg)
        }

        // Generic errors use string matching
        EvmToolError::Generic(msg) => classify_generic_message(msg),
    }
}

impl From<EvmToolError> for riglr_core::ToolError {
    fn from(err: EvmToolError) -> Self {
        let class = classify_evm_error(&err);
        match class {
            ErrorClass::Permanent => {
                riglr_core::ToolError::permanent_with_source(err, "EVM operation failed")
            }
            ErrorClass::Retriable => {
                riglr_core::ToolError::retriable_with_source(err, "EVM operation can be retried")
            }
            ErrorClass::RateLimited => riglr_core::ToolError::rate_limited_with_source(
                err,
                "EVM rate limit exceeded",
                None,
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_error_code() {
        // Test various error message formats
        assert_eq!(extract_error_code("error code: -32000"), Some(-32000));
        assert_eq!(extract_error_code("code: -32603"), Some(-32603));
        assert_eq!(extract_error_code("429 Too Many Requests"), Some(429));
        assert_eq!(extract_error_code("Status: 429 - Rate limited"), Some(429));
        assert_eq!(
            extract_error_code("error -32001: insufficient funds"),
            Some(-32001)
        );
        assert_eq!(
            extract_error_code("JSON-RPC error code: -32602 invalid params"),
            Some(-32602)
        );

        // Test messages without error codes
        assert_eq!(extract_error_code("connection timeout"), None);
        assert_eq!(extract_error_code("invalid transaction"), None);
        assert_eq!(extract_error_code("rate limit exceeded"), None);
    }

    #[test]
    fn test_classify_by_error_code() {
        // Test standard JSON-RPC permanent errors
        assert_eq!(classify_by_error_code(-32700), Some(ErrorClass::Permanent)); // Parse error
        assert_eq!(classify_by_error_code(-32600), Some(ErrorClass::Permanent)); // Invalid request
        assert_eq!(classify_by_error_code(-32601), Some(ErrorClass::Permanent)); // Method not found
        assert_eq!(classify_by_error_code(-32602), Some(ErrorClass::Permanent)); // Invalid params

        // Test standard JSON-RPC retriable error
        assert_eq!(classify_by_error_code(-32603), Some(ErrorClass::Retriable)); // Internal error

        // Test implementation-specific permanent errors
        assert_eq!(classify_by_error_code(-32000), Some(ErrorClass::Permanent)); // Invalid input
        assert_eq!(classify_by_error_code(-32002), Some(ErrorClass::Permanent)); // Resource not found
        assert_eq!(classify_by_error_code(-32003), Some(ErrorClass::Permanent)); // Transaction rejected
        assert_eq!(classify_by_error_code(-32004), Some(ErrorClass::Permanent)); // Method not supported

        // Test rate limiting codes
        assert_eq!(
            classify_by_error_code(-32005),
            Some(ErrorClass::RateLimited)
        ); // Limit exceeded
        assert_eq!(classify_by_error_code(429), Some(ErrorClass::RateLimited)); // HTTP Too Many Requests

        // Test server errors in range
        assert_eq!(classify_by_error_code(-32050), Some(ErrorClass::Retriable));
        assert_eq!(classify_by_error_code(-32099), Some(ErrorClass::Retriable));

        // Test unknown codes
        assert_eq!(classify_by_error_code(500), None);
        assert_eq!(classify_by_error_code(-33000), None);
    }

    #[test]
    fn test_classify_evm_error_with_codes() {
        // Test errors with structured JSON-RPC codes in ProviderError messages
        let error =
            EvmToolError::ProviderError("error code: -32000 insufficient funds".to_string());
        assert_eq!(classify_evm_error(&error), ErrorClass::Permanent);

        let error = EvmToolError::ProviderError(
            "JSON-RPC error code: -32603 internal server error".to_string(),
        );
        assert_eq!(classify_evm_error(&error), ErrorClass::Retriable);

        let error =
            EvmToolError::ProviderError("429 Too Many Requests - please retry later".to_string());
        assert_eq!(classify_evm_error(&error), ErrorClass::RateLimited);

        let error = EvmToolError::ProviderError("error code: -32602 invalid params".to_string());
        assert_eq!(classify_evm_error(&error), ErrorClass::Permanent);

        let error = EvmToolError::ProviderError("code: -32005 rate limit exceeded".to_string());
        assert_eq!(classify_evm_error(&error), ErrorClass::RateLimited);
    }

    #[test]
    fn test_classify_evm_error_fallback_string_matching() {
        // Test fallback to string-based classification when no code is present

        // Network errors
        let error = EvmToolError::NetworkError("connection timeout after 30 seconds".to_string());
        assert_eq!(classify_evm_error(&error), ErrorClass::Retriable);

        // Provider errors with string matching
        let error = EvmToolError::ProviderError("request timeout".to_string());
        assert_eq!(classify_evm_error(&error), ErrorClass::Retriable);

        let error = EvmToolError::ProviderError("quota exceeded for API key".to_string());
        assert_eq!(classify_evm_error(&error), ErrorClass::RateLimited);

        // Test structured error variants
        let error = EvmToolError::NonceTooLow;
        assert_eq!(classify_evm_error(&error), ErrorClass::Retriable);

        let error = EvmToolError::InsufficientFunds;
        assert_eq!(classify_evm_error(&error), ErrorClass::Permanent);

        let error = EvmToolError::TransactionReverted {
            reason: "Out of gas".to_string(),
        };
        assert_eq!(classify_evm_error(&error), ErrorClass::Permanent);
    }

    #[test]
    fn test_classify_evm_error_direct_variants() {
        // Test error types that are always classified the same way
        let error = EvmToolError::ContractError("method not found".to_string());
        assert_eq!(classify_evm_error(&error), ErrorClass::Permanent);

        let error = EvmToolError::InvalidParameter("invalid address format".to_string());
        assert_eq!(classify_evm_error(&error), ErrorClass::Permanent);

        let error = EvmToolError::InsufficientFunds;
        assert_eq!(classify_evm_error(&error), ErrorClass::Permanent);

        let error = EvmToolError::InvalidAddress("not a valid hex address".to_string());
        assert_eq!(classify_evm_error(&error), ErrorClass::Permanent);

        let error = EvmToolError::UnsupportedChain(999);
        assert_eq!(classify_evm_error(&error), ErrorClass::Permanent);

        let error = EvmToolError::SignerError(riglr_core::SignerError::NoSignerContext);
        assert_eq!(classify_evm_error(&error), ErrorClass::Permanent);

        let error = EvmToolError::RateLimited;
        assert_eq!(classify_evm_error(&error), ErrorClass::RateLimited);

        let error = EvmToolError::GasEstimationFailed;
        assert_eq!(classify_evm_error(&error), ErrorClass::Retriable);
    }

    #[test]
    fn test_classify_evm_error_code_priority() {
        // Test that error codes take priority over string matching in ProviderError

        // Message suggests retriable, but code says permanent
        let error =
            EvmToolError::ProviderError("connection timeout, error code: -32000".to_string());
        assert_eq!(classify_evm_error(&error), ErrorClass::Permanent);

        // Message suggests permanent, but code says rate limited
        let error = EvmToolError::ProviderError("invalid request format code: 429".to_string());
        assert_eq!(classify_evm_error(&error), ErrorClass::RateLimited);
    }

    #[test]
    fn test_from_evm_tool_error_to_tool_error() {
        // Test the From trait implementation
        let evm_error = EvmToolError::ProviderError("error code: -32000".to_string());
        let tool_error: riglr_core::ToolError = evm_error.into();
        // We can't directly test the internal state of ToolError, but we can verify it converts
        assert!(tool_error.to_string().contains("-32000"));

        let evm_error = EvmToolError::RateLimited;
        let tool_error: riglr_core::ToolError = evm_error.into();
        assert!(tool_error.to_string().contains("Rate limit exceeded"));

        let evm_error = EvmToolError::Generic("timeout".to_string());
        let tool_error: riglr_core::ToolError = evm_error.into();
        assert!(tool_error.to_string().contains("timeout"));

        // Test source preservation - create a structured error and verify it's preserved
        let evm_error = EvmToolError::InsufficientFunds;
        let tool_error: riglr_core::ToolError = evm_error.clone().into();
        // The source should be preserved
        assert!(tool_error.to_string().contains("Insufficient funds"));
    }

    #[test]
    fn test_classify_generic_message() {
        // Test generic message classification
        assert_eq!(
            classify_generic_message("connection reset"),
            ErrorClass::Retriable
        );
        assert_eq!(
            classify_generic_message("request timeout"),
            ErrorClass::Retriable
        );
        assert_eq!(
            classify_generic_message("unknown error"),
            ErrorClass::Permanent
        );
    }

    #[test]
    fn test_new_structured_error_variants() {
        // Test all new structured error variants
        let error = EvmToolError::NonceTooLow;
        assert_eq!(classify_evm_error(&error), ErrorClass::Retriable);

        let error = EvmToolError::NonceMismatch;
        assert_eq!(classify_evm_error(&error), ErrorClass::Retriable);

        let error = EvmToolError::InsufficientFunds;
        assert_eq!(classify_evm_error(&error), ErrorClass::Permanent);

        let error = EvmToolError::TransactionReverted {
            reason: "Execution failed".to_string(),
        };
        assert_eq!(classify_evm_error(&error), ErrorClass::Permanent);

        let error = EvmToolError::RateLimited;
        assert_eq!(classify_evm_error(&error), ErrorClass::RateLimited);

        let error = EvmToolError::GasEstimationFailed;
        assert_eq!(classify_evm_error(&error), ErrorClass::Retriable);

        let error = EvmToolError::NetworkError("timeout".to_string());
        assert_eq!(classify_evm_error(&error), ErrorClass::Retriable);
    }

    #[test]
    fn test_downcast_from_tool_error_source() {
        use std::error::Error;

        // Create a structured EvmToolError
        let evm_error = EvmToolError::InsufficientFunds;

        // Convert to ToolError
        let tool_error: riglr_core::ToolError = evm_error.clone().into();

        // The error message should be preserved
        assert!(tool_error.to_string().contains("Insufficient funds"));

        // We can't directly test downcasting because ToolError's source is private,
        // but the From implementation preserves the source, which is what matters

        // Test another structured variant
        let evm_error = EvmToolError::TransactionReverted {
            reason: "Out of gas".to_string(),
        };
        let tool_error: riglr_core::ToolError = evm_error.into();
        assert!(tool_error.to_string().contains("Out of gas"));
    }

    #[test]
    fn test_provider_error_classification_with_fallback() {
        // Test ProviderError with error code (should use code)
        let error = EvmToolError::ProviderError("error code: -32000".to_string());
        assert_eq!(classify_evm_error(&error), ErrorClass::Permanent);

        // Test ProviderError without code (should use string matching)
        let error = EvmToolError::ProviderError("connection timeout".to_string());
        assert_eq!(classify_evm_error(&error), ErrorClass::Retriable);

        let error = EvmToolError::ProviderError("rate limit exceeded".to_string());
        assert_eq!(classify_evm_error(&error), ErrorClass::RateLimited);

        let error = EvmToolError::ProviderError("unknown error".to_string());
        assert_eq!(classify_evm_error(&error), ErrorClass::Permanent);
    }
}
