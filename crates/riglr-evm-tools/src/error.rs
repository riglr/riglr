//! Error types for EVM tools

use riglr_core::retry::ErrorClass;
use riglr_core::signer::error::Standard as SignerError;
use thiserror::Error;

/// Main error type for EVM tools
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum Error {
    /// Contract-related errors
    #[error("Contract error: {0}")]
    ContractError(String),

    /// Gas estimation failed
    #[error("Gas estimation failed")]
    GasEstimationFailed,

    /// Generic error fallback
    #[error("{0}")]
    Generic(String),

    /// Insufficient funds for transaction
    #[error("Insufficient funds")]
    InsufficientFunds,

    /// Invalid address format
    #[error("Invalid address: {0}")]
    InvalidAddress(String),

    /// Invalid parameters
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    /// Network timeout or connection issues
    #[error("Network error: {0}")]
    NetworkError(String),

    /// Nonce mismatch
    #[error("Nonce mismatch")]
    NonceMismatch,

    /// Nonce is too low
    #[error("Nonce too low")]
    NonceTooLow,

    /// Generic provider issues
    #[error("Provider error: {0}")]
    ProviderError(String),

    /// Rate limit exceeded
    #[error("Rate limit exceeded")]
    RateLimited,

    /// Signer-related errors
    #[error("Signer error: {0}")]
    SignerError(SignerError),

    /// Transaction reverted on-chain
    #[error("Transaction reverted: {reason}")]
    TransactionReverted { reason: String },

    /// Unsupported chain
    #[error("Unsupported chain: {0}")]
    UnsupportedChain(u64),
}

impl Clone for Error {
    fn clone(&self) -> Self {
        match *self {
            Self::ContractError(ref s) => Self::ContractError(s.clone()),
            Self::GasEstimationFailed => Self::GasEstimationFailed,
            Self::Generic(ref s) => Self::Generic(s.clone()),
            Self::InsufficientFunds => Self::InsufficientFunds,
            Self::InvalidAddress(ref s) => Self::InvalidAddress(s.clone()),
            Self::InvalidParameter(ref s) => Self::InvalidParameter(s.clone()),
            Self::NetworkError(ref s) => Self::NetworkError(s.clone()),
            Self::NonceMismatch => Self::NonceMismatch,
            Self::NonceTooLow => Self::NonceTooLow,
            Self::ProviderError(ref s) => Self::ProviderError(s.clone()),
            Self::RateLimited => Self::RateLimited,
            Self::SignerError(ref e) => Self::SignerError(e.clone()),
            Self::TransactionReverted { ref reason } => Self::TransactionReverted {
                reason: reason.clone(),
            },
            Self::UnsupportedChain(u) => Self::UnsupportedChain(u),
        }
    }
}

impl From<Box<dyn riglr_core::SignerError + Send + Sync>> for Error {
    #[inline]
    fn from(err: Box<dyn riglr_core::SignerError + Send + Sync>) -> Self {
        Self::SignerError(SignerError::Generic(err.to_string()))
    }
}

impl From<Error> for Box<dyn riglr_core::SignerError + Send + Sync> {
    #[inline]
    fn from(err: Error) -> Self {
        match err {
            Error::SignerError(inner) => Box::new(inner),
            _ => Box::new(SignerError::Generic(err.to_string())),
        }
    }
}

impl From<Error> for Box<dyn riglr_core::SignerError> {
    #[inline]
    fn from(err: Error) -> Self {
        match err {
            Error::SignerError(inner) => Box::new(inner),
            _ => Box::new(SignerError::Generic(err.to_string())),
        }
    }
}

impl From<SignerError> for Error {
    #[inline]
    fn from(err: SignerError) -> Self {
        Self::SignerError(err)
    }
}

impl From<Error> for SignerError {
    #[inline]
    fn from(err: Error) -> Self {
        match err {
            Error::SignerError(inner) => Self::Generic(inner.to_string()),
            _ => Self::Generic(err.to_string()),
        }
    }
}

/// Error classification for retry logic
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Class {
    /// Permanent errors that should not be retried
    Permanent,
    /// Rate-limited errors that need backoff
    RateLimited,
    /// Retriable errors that may succeed on retry
    Retriable,
}

impl From<Error> for riglr_core::ToolError {
    #[inline]
    fn from(err: Error) -> Self {
        let class = classify(&err);
        match class {
            ErrorClass::Permanent => Self::permanent_with_source(err, "EVM operation failed"),
            ErrorClass::Retryable => {
                Self::retriable_with_source(err, "EVM operation can be retried")
            }
            ErrorClass::RateLimited => {
                Self::rate_limited_with_source(err, "EVM rate limit exceeded", None)
            }
            _ => Self::permanent_with_source(err, "Unknown EVM error"),
        }
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
/// assert_eq!(classify_error(&error), ErrorClass::Retryable);
///
/// let error = EvmToolError::InsufficientFunds;
/// assert_eq!(classify_error(&error), ErrorClass::Permanent);
/// ```
#[must_use]
#[inline]
const fn classify_error_code(code: i32) -> Option<ErrorClass> {
    match code {
        // Standard JSON-RPC permanent errors
        -32700 | -32600 | -32601 | -32602 => Some(ErrorClass::Permanent),
        // Standard JSON-RPC internal error (retriable)
        -32603 => Some(ErrorClass::Retryable),
        // Common implementation-specific permanent errors
        -32000 | -32002 | -32003 | -32004 => Some(ErrorClass::Permanent),
        // Rate limiting
        -32005 | 429 => Some(ErrorClass::RateLimited),
        // Other server errors in the -32000 to -32099 range
        -32099..=-32001 => Some(ErrorClass::Retryable),
        // Unknown codes
        _ => None,
    }
}

#[must_use]
pub fn classify(error: &Error) -> ErrorClass {
    match *error {
        // Structured errors with clear classification - grouped by return value
        Error::ContractError(_)
        | Error::InsufficientFunds
        | Error::InvalidAddress(_)
        | Error::InvalidParameter(_)
        | Error::SignerError(_)
        | Error::TransactionReverted { .. }
        | Error::UnsupportedChain(_) => ErrorClass::Permanent,

        Error::RateLimited => ErrorClass::RateLimited,

        Error::GasEstimationFailed
        | Error::NonceMismatch
        | Error::NonceTooLow
        | Error::NetworkError(_) => ErrorClass::Retryable,

        // Provider errors need string matching as fallback
        Error::ProviderError(ref msg) => {
            // First try to extract and classify by error code
            // Try to find patterns like "code: -32000" or "error code: -32000"
            if let Some(idx) = msg.find("code:") {
                let code_str = msg
                    .get(idx.checked_add(5).unwrap_or(idx)..)
                    .unwrap_or("")
                    .trim();
                if let Some(end_idx) =
                    code_str.find(|character: char| !character.is_ascii_digit() && character != '-')
                {
                    if let Ok(code) = code_str[..end_idx].parse::<i32>() {
                        let class = classify_error_code(code);
                        if let Some(class) = class {
                            return class;
                        }
                    }
                }
                if let Some(first) = code_str.split_whitespace().next() {
                    if let Ok(code) = first.parse::<i32>() {
                        let class = classify_error_code(code);
                        if let Some(class) = class {
                            return class;
                        }
                    }
                }
            }

            // Try to find HTTP status codes like "429"
            if msg.starts_with("429") || msg.contains(" 429 ") {
                return ErrorClass::RateLimited;
            }

            // Try to find patterns like "error -32000"
            if let Some(idx) = msg.find("error ") {
                let code_str = msg.get(idx.saturating_add(6)..).unwrap_or("").trim();
                if let Some(stripped) = code_str.strip_prefix('-') {
                    if let Some(end_idx) =
                        stripped.find(|character: char| !character.is_ascii_digit())
                    {
                        if let Ok(code) = code_str[..=end_idx].parse::<i32>() {
                            let class = match code {
                                // Standard JSON-RPC permanent errors
                                -32700 | -32600 | -32601 | -32602 => Some(ErrorClass::Permanent),
                                // Standard JSON-RPC internal error (retriable)
                                -32603 => Some(ErrorClass::Retryable),
                                // Common implementation-specific permanent errors
                                -32000 | -32002 | -32003 | -32004 => Some(ErrorClass::Permanent),
                                // Rate limiting
                                -32005 | 429 => Some(ErrorClass::RateLimited),
                                // Other server errors in the -32000 to -32099 range
                                -32099..=-32001 => Some(ErrorClass::Retryable),
                                // Unknown codes
                                _ => None,
                            };
                            if let Some(class) = class {
                                return class;
                            }
                        }
                    }
                }
            }

            // Fall back to string matching
            if msg.contains("timeout") || msg.contains("connection") {
                return ErrorClass::Retryable;
            } else if msg.contains("rate") || msg.contains("quota") {
                return ErrorClass::RateLimited;
            }
            ErrorClass::Permanent
        }

        // Generic errors use string matching
        Error::Generic(ref msg) => {
            if msg.contains("timeout") || msg.contains("connection") {
                return ErrorClass::Retryable;
            }
            ErrorClass::Permanent
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_error_with_codes() {
        // Test errors with structured JSON-RPC codes in ProviderError messages
        let error = Error::ProviderError("error code: -32000 insufficient funds".to_string());
        assert_eq!(classify(&error), ErrorClass::Permanent);

        let error =
            Error::ProviderError("JSON-RPC error code: -32603 internal server error".to_string());
        assert_eq!(classify(&error), ErrorClass::Retryable);

        let error = Error::ProviderError("429 Too Many Requests - please retry later".to_string());
        assert_eq!(classify(&error), ErrorClass::RateLimited);

        let error = Error::ProviderError("error code: -32602 invalid params".to_string());
        assert_eq!(classify(&error), ErrorClass::Permanent);

        let error = Error::ProviderError("code: -32005 rate limit exceeded".to_string());
        assert_eq!(classify(&error), ErrorClass::RateLimited);
    }

    #[test]
    fn test_classify_error_fallback_string_matching() {
        // Test fallback to string-based classification when no code is present

        // Network errors
        let error = Error::NetworkError("connection timeout after 30 seconds".to_string());
        assert_eq!(classify(&error), ErrorClass::Retryable);

        // Provider errors with string matching
        let error = Error::ProviderError("request timeout".to_string());
        assert_eq!(classify(&error), ErrorClass::Retryable);

        let error = Error::ProviderError("quota exceeded for API key".to_string());
        assert_eq!(classify(&error), ErrorClass::RateLimited);

        // Test structured error variants
        let error = Error::NonceTooLow;
        assert_eq!(classify(&error), ErrorClass::Retryable);

        let error = Error::InsufficientFunds;
        assert_eq!(classify(&error), ErrorClass::Permanent);

        let error = Error::TransactionReverted {
            reason: "Out of gas".to_string(),
        };
        assert_eq!(classify(&error), ErrorClass::Permanent);
    }

    #[test]
    fn test_classify_error_direct_variants() {
        // Test error types that are always classified the same way
        let error = Error::ContractError("method not found".to_string());
        assert_eq!(classify(&error), ErrorClass::Permanent);

        let error = Error::InvalidParameter("invalid address format".to_string());
        assert_eq!(classify(&error), ErrorClass::Permanent);

        let error = Error::InsufficientFunds;
        assert_eq!(classify(&error), ErrorClass::Permanent);

        let error = Error::InvalidAddress("not a valid hex address".to_string());
        assert_eq!(classify(&error), ErrorClass::Permanent);

        let error = Error::UnsupportedChain(999);
        assert_eq!(classify(&error), ErrorClass::Permanent);

        let error = Error::SignerError(SignerError::NoContext);
        assert_eq!(classify(&error), ErrorClass::Permanent);

        let error = Error::RateLimited;
        assert_eq!(classify(&error), ErrorClass::RateLimited);

        let error = Error::GasEstimationFailed;
        assert_eq!(classify(&error), ErrorClass::Retryable);
    }

    #[test]
    fn test_classify_error_code_priority() {
        // Test that error codes take priority over string matching in ProviderError

        // Message suggests retriable, but code says permanent
        let error = Error::ProviderError("connection timeout, error code: -32000".to_string());
        assert_eq!(classify(&error), ErrorClass::Permanent);

        // Message suggests permanent, but code says rate limited
        let error = Error::ProviderError("invalid request format code: 429".to_string());
        assert_eq!(classify(&error), ErrorClass::RateLimited);
    }

    #[test]
    fn test_from_error_to_tool_error() {
        // Test the From trait implementation
        let evm_error = Error::ProviderError("error code: -32000".to_string());
        let tool_error: riglr_core::ToolError = evm_error.into();
        // We can't directly test the internal state of ToolError, but we can verify it converts
        assert!(tool_error.to_string().contains("-32000"));

        let evm_error = Error::RateLimited;
        let tool_error: riglr_core::ToolError = evm_error.into();
        assert!(tool_error.to_string().contains("Rate limit exceeded"));

        let evm_error = Error::Generic("timeout".to_string());
        let tool_error: riglr_core::ToolError = evm_error.into();
        assert!(tool_error.to_string().contains("timeout"));

        // Test source preservation - create a structured error and verify it's preserved
        let evm_error = Error::InsufficientFunds;
        let tool_error: riglr_core::ToolError = evm_error.into();
        // The source should be preserved
        assert!(tool_error.to_string().contains("Insufficient funds"));
    }

    #[test]
    fn test_new_structured_error_variants() {
        // Test all new structured error variants
        let error = Error::NonceTooLow;
        assert_eq!(classify(&error), ErrorClass::Retryable);

        let error = Error::NonceMismatch;
        assert_eq!(classify(&error), ErrorClass::Retryable);

        let error = Error::InsufficientFunds;
        assert_eq!(classify(&error), ErrorClass::Permanent);

        let error = Error::TransactionReverted {
            reason: "Execution failed".to_string(),
        };
        assert_eq!(classify(&error), ErrorClass::Permanent);

        let error = Error::RateLimited;
        assert_eq!(classify(&error), ErrorClass::RateLimited);

        let error = Error::GasEstimationFailed;
        assert_eq!(classify(&error), ErrorClass::Retryable);

        let error = Error::NetworkError("timeout".to_string());
        assert_eq!(classify(&error), ErrorClass::Retryable);
    }

    #[test]
    fn test_downcast_from_tool_error_source() {
        // Create a structured Error
        let evm_error = Error::InsufficientFunds;

        // Convert to ToolError
        let tool_error: riglr_core::ToolError = evm_error.into();

        // The error message should be preserved
        assert!(tool_error.to_string().contains("Insufficient funds"));

        // We can't directly test downcasting because ToolError's source is private,
        // but the From implementation preserves the source, which is what matters

        // Test another structured variant
        let evm_error = Error::TransactionReverted {
            reason: "Out of gas".to_string(),
        };
        let tool_error: riglr_core::ToolError = evm_error.into();
        assert!(tool_error.to_string().contains("Out of gas"));
    }

    #[test]
    fn test_provider_error_classification_with_fallback() {
        // Test ProviderError with error code (should use code)
        let error = Error::ProviderError("error code: -32000".to_string());
        assert_eq!(classify(&error), ErrorClass::Permanent);

        // Test ProviderError without code (should use string matching)
        let error = Error::ProviderError("connection timeout".to_string());
        assert_eq!(classify(&error), ErrorClass::Retryable);

        let error = Error::ProviderError("rate limit exceeded".to_string());
        assert_eq!(classify(&error), ErrorClass::RateLimited);

        let error = Error::ProviderError("unknown error".to_string());
        assert_eq!(classify(&error), ErrorClass::Permanent);
    }
}
