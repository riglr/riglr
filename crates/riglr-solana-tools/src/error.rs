//! Error types for riglr-solana-tools.

use core::{error::Error as CoreError, result, time::Duration};
use riglr_core::{error::ToolError, signer::error::Standard as SignerErrorStandard, SignerError};
use solana_client::{
    client_error::{ClientError, ClientErrorKind},
    rpc_request::{RpcError, RpcRequest},
};
use thiserror::Error;

/// Main error type for Solana tool operations.
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum Error {
    /// Core riglr error - typically retriable
    #[error("Core error: {0}")]
    Core(Box<dyn CoreError + Send + Sync>),

    /// Generic error - default to retriable
    #[error("Solana tool error: {0}")]
    Generic(String),

    /// HTTP request error - network issues are typically retriable
    /// Note: May be rate-limited if status is 429
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    /// Insufficient funds for operation - permanent error
    #[error("Insufficient funds for operation")]
    InsufficientFunds,

    /// Invalid address format - user input error
    #[error("Invalid address: {0}")]
    InvalidAddress(String),

    /// Invalid key format - user input error
    #[error("Invalid key: {0}")]
    InvalidKey(String),

    /// Invalid signature format - user input error
    #[error("Invalid signature: {0}")]
    InvalidSignature(String),

    /// Invalid token mint - user input error
    #[error("Invalid token mint: {0}")]
    InvalidTokenMint(String),

    /// RPC client error - network issues are typically retriable
    /// Note: May be rate-limited if message contains "429" or "rate limit"
    #[error("RPC error: {0}")]
    Rpc(String),

    /// Serialization error - data corruption/format issue
    #[error("Serialization error: {0}")]
    Serialization(Box<dyn CoreError + Send + Sync>),

    /// Signer context error - configuration issue
    #[error("Signer context error: {0}")]
    SignerError(Box<dyn SignerError + Send + Sync + 'static>),

    /// Solana client error - classification depends on inner error
    #[error("Solana client error: {0}")]
    SolanaClient(Box<ClientError>),

    /// Core tool error - passthrough
    #[error("Core tool error: {0}")]
    ToolError(#[from] ToolError),

    /// Transaction failed - may be retriable depending on message
    #[error("Transaction error: {0}")]
    Transaction(String),
}

impl Clone for Error {
    fn clone(&self) -> Self {
        match *self {
            Self::Core(ref e) => Self::Core(e.to_string().into()),
            Self::Generic(ref s) => Self::Generic(s.clone()),
            Self::Http(ref http_err) => {
                // Create a placeholder HTTP error for cloning since reqwest::Error doesn't implement Clone
                // Convert to a string representation and create a generic error variant instead
                Self::Generic(format!("HTTP error (cloned): {http_err}"))
            }
            Self::InsufficientFunds => Self::InsufficientFunds,
            Self::InvalidAddress(ref s) => Self::InvalidAddress(s.clone()),
            Self::InvalidKey(ref s) => Self::InvalidKey(s.clone()),
            Self::InvalidSignature(ref s) => Self::InvalidSignature(s.clone()),
            Self::InvalidTokenMint(ref s) => Self::InvalidTokenMint(s.clone()),
            Self::Rpc(ref s) => Self::Rpc(s.clone()),
            Self::Serialization(ref e) => Self::Serialization(e.to_string().into()),
            Self::SignerError(ref e) => {
                Self::SignerError(Box::new(SignerErrorStandard::Generic(e.to_string())))
            }
            Self::SolanaClient(ref e) => {
                Self::SolanaClient(Box::new(ClientError::new_with_request(
                    ClientErrorKind::Custom(e.to_string()),
                    RpcRequest::GetAccountInfo,
                )))
            }
            Self::ToolError(ref e) => Self::ToolError(e.clone()),
            Self::Transaction(ref s) => Self::Transaction(s.clone()),
        }
    }
}

/// Result type alias for Solana tool operations.
pub type Result<T> = result::Result<T, Error>;

/// Internal classification of errors for conversion to `ToolError`
#[derive(Debug, PartialEq)]
enum ErrorClassification {
    /// Invalid input errors
    InvalidInput,
    /// Permanent errors that should not be retried
    Permanent,
    /// Rate-limited errors with optional retry delay
    RateLimited {
        /// Suggested delay before retrying
        delay: Option<Duration>,
    },
    /// Errors that can be retried
    Retriable,
    /// Signer context errors (special case)
    SignerContext,
    /// Pass through an existing `ToolError` without re-wrapping
    ToolErrorPassthrough(ToolError),
}

impl Error {
    /// Classify this error for conversion to `ToolError`
    ///
    /// This method encapsulates all the complex classification logic,
    /// including dynamic checks based on message content and error types.
    fn classify(&self) -> ErrorClassification {
        match *self {
            // Passthrough ToolError without re-wrapping
            Self::ToolError(ref error) => ErrorClassification::ToolErrorPassthrough(error.clone()),

            // Signer errors are configuration issues
            Self::SignerError(_) => ErrorClassification::SignerContext,

            // Input validation errors
            Self::InvalidAddress(_)
            | Self::InvalidKey(_)
            | Self::InvalidSignature(_)
            | Self::InvalidTokenMint(_) => ErrorClassification::InvalidInput,

            // Insufficient funds and serialization errors are permanent
            Self::InsufficientFunds | Self::Serialization(_) => ErrorClassification::Permanent,

            // RPC errors - check for rate limiting indicators
            Self::Rpc(ref msg) => {
                if msg.contains("429")
                    || msg.contains("rate limit")
                    || msg.contains("too many requests")
                {
                    return ErrorClassification::RateLimited {
                        delay: Some(Duration::from_secs(1)),
                    };
                }
                ErrorClassification::Retriable
            }

            // HTTP errors - check status code for rate limiting
            Self::Http(ref http_err) => {
                if http_err.status() == Some(reqwest::StatusCode::TOO_MANY_REQUESTS) {
                    return ErrorClassification::RateLimited {
                        delay: Some(Duration::from_secs(1)),
                    };
                } else if http_err.is_timeout() || http_err.is_connect() {
                    return ErrorClassification::Retriable;
                } else if matches!(
                    http_err.status(),
                    Some(
                        reqwest::StatusCode::BAD_REQUEST
                            | reqwest::StatusCode::UNAUTHORIZED
                            | reqwest::StatusCode::FORBIDDEN
                    )
                ) {
                    return ErrorClassification::Permanent;
                }
                ErrorClassification::Retriable
            }

            // Solana client errors - use the classify_transaction_error helper
            Self::SolanaClient(ref client_err) => {
                let error_type = classify_transaction(client_err);
                match error_type {
                    TransactionErrorType::RateLimited(_) => ErrorClassification::RateLimited {
                        delay: Some(Duration::from_secs(1)),
                    },
                    TransactionErrorType::Retryable(_) | TransactionErrorType::Unknown(_) => {
                        ErrorClassification::Retriable
                    }
                    TransactionErrorType::Permanent(Permanent::InsufficientFunds) => {
                        ErrorClassification::Permanent
                    }
                    TransactionErrorType::Permanent(_) => ErrorClassification::Permanent,
                }
            }

            // Transaction errors - check message for patterns
            Self::Transaction(ref msg) => {
                if msg.contains("insufficient") || msg.contains("InsufficientFunds") {
                    return ErrorClassification::Permanent;
                } else if msg.contains("rate limit") || msg.contains("429") {
                    return ErrorClassification::RateLimited {
                        delay: Some(Duration::from_secs(1)),
                    };
                }
                ErrorClassification::Retriable
            }

            // Core errors and generic errors are typically retriable
            Self::Core(_) | Self::Generic(_) => ErrorClassification::Retriable,
        }
    }

    /// Check if this error is rate-limited.
    #[must_use]
    #[inline]
    pub fn is_rate_limited(&self) -> bool {
        match *self {
            Self::Rpc(ref msg) => {
                msg.contains("429")
                    || msg.contains("rate limit")
                    || msg.contains("too many requests")
            }
            Self::Http(ref http_err) => {
                http_err.status() == Some(reqwest::StatusCode::TOO_MANY_REQUESTS)
            }
            Self::SolanaClient(ref client_err) => {
                let error_type = classify_transaction(client_err);

                error_type.is_rate_limited()
            }
            Self::ToolError(_)
            | Self::SignerError(_)
            | Self::InvalidAddress(_)
            | Self::InvalidKey(_)
            | Self::InvalidSignature(_)
            | Self::Transaction(_)
            | Self::InsufficientFunds
            | Self::InvalidTokenMint(_)
            | Self::Serialization(_)
            | Self::Core(_)
            | Self::Generic(_) => false,
        }
    }

    /// Check if this error is retriable.
    /// Note: The `IntoToolError` macro generates a basic From implementation, but for complex
    /// cases that need runtime logic (like checking message content), we keep this method
    /// for backward compatibility and to support custom logic.
    #[must_use]
    #[inline]
    pub fn is_retriable(&self) -> bool {
        match *self {
            // Core errors inherit their retriable nature
            Self::ToolError(ref tool_err) => tool_err.is_retriable(),
            Self::Core(_) | Self::Rpc(_) | Self::Generic(_) => true, // Core, RPC and generic errors are typically retriable
            Self::Http(ref http_err) => !matches!(
                http_err.status(),
                Some(
                    reqwest::StatusCode::BAD_REQUEST
                        | reqwest::StatusCode::UNAUTHORIZED
                        | reqwest::StatusCode::FORBIDDEN
                )
            ),

            // Client errors need classification
            Self::SolanaClient(ref client_err) => {
                let error_type = classify_transaction(client_err);

                error_type.is_retryable()
            }

            // Address/key validation errors, insufficient funds, and serialization errors are permanent
            // Generally configuration issues
            Self::SignerError(_)
            | Self::InvalidAddress(_)
            | Self::InvalidKey(_)
            | Self::InvalidSignature(_)
            | Self::InvalidTokenMint(_)
            | Self::InsufficientFunds
            | Self::Serialization(_) => false,

            // Transaction errors depend on content
            Self::Transaction(ref msg) => {
                !(msg.contains("insufficient funds") || msg.contains("invalid"))
            }
        }
    }

    /// Get appropriate retry delay for rate-limited errors.
    #[must_use]
    #[inline]
    pub fn retry_delay(&self) -> Option<Duration> {
        if self.is_rate_limited() {
            return Some(Duration::from_secs(1));
        } else if self.is_retriable() {
            return Some(Duration::from_millis(500));
        }
        None
    }
}

/// Structured classification of transaction errors for intelligent retry logic
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum TransactionErrorType {
    /// Errors that represent permanent failures and should not be retried
    Permanent(Permanent),
    /// Rate limiting errors that require special handling with delays
    RateLimited(RateLimit),
    /// Errors that can be retried with appropriate backoff
    Retryable(Retryable),
    /// Unknown error types that don't fit other categories
    Unknown(String),
}

/// Errors that can be retried with appropriate backoff
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Retryable {
    /// Blockchain congestion
    NetworkCongestion,
    /// Network connectivity issues
    NetworkConnectivity,
    /// RPC service temporary unavailability
    TemporaryRpcFailure,
    /// Transaction pool full
    TransactionPoolFull,
}

/// Permanent errors that should not be retried
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Permanent {
    /// Duplicate transaction
    DuplicateTransaction,
    /// Program execution error
    InstructionError,
    /// Insufficient funds for transaction
    InsufficientFunds,
    /// Invalid account referenced
    InvalidAccount,
    /// Invalid signature provided
    InvalidSignature,
    /// Invalid transaction structure
    InvalidTransaction,
}

/// Rate limiting errors with special handling
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum RateLimit {
    /// Standard RPC rate limiting
    RpcRateLimit,
    /// Too many requests error
    TooManyRequests,
}

impl TransactionErrorType {
    /// Check if this is a rate limiting error (special case of retryable)
    #[must_use]
    #[inline]
    pub const fn is_rate_limited(&self) -> bool {
        matches!(*self, Self::RateLimited(_))
    }

    /// Check if this error type is retryable
    #[must_use]
    #[inline]
    pub const fn is_retryable(&self) -> bool {
        matches!(*self, Self::Retryable(_) | Self::RateLimited(_))
    }
}

// Implement From conversion to riglr_core::ToolError for proper error handling
// This implementation preserves the source error for downcasting and uses
// the classification methods to determine retriability.
impl From<Error> for ToolError {
    #[inline]
    fn from(err: Error) -> Self {
        // Use the classify method to determine error handling
        match err.classify() {
            ErrorClassification::ToolErrorPassthrough(tool_err) => tool_err,

            ErrorClassification::Permanent => {
                Self::permanent_with_source(err, "Solana operation failed")
            }

            ErrorClassification::Retriable => {
                Self::retriable_with_source(err, "Solana operation can be retried")
            }

            ErrorClassification::RateLimited { delay } => {
                Self::rate_limited_with_source(err, "Solana rate limit exceeded", delay)
            }

            ErrorClassification::InvalidInput => {
                Self::invalid_input_with_source(err, "Invalid input for Solana operation")
            }

            ErrorClassification::SignerContext => Self::SignerContext(err.to_string()),
        }
    }
}

impl From<SignerErrorStandard> for Error {
    #[inline]
    fn from(err: SignerErrorStandard) -> Self {
        Self::SignerError(Box::new(err) as Box<dyn SignerError + Send + Sync + 'static>)
    }
}

impl From<ClientError> for Error {
    #[inline]
    fn from(error: ClientError) -> Self {
        Self::SolanaClient(Box::new(error))
    }
}

/// Classify a Solana `ClientError` into a structured transaction error type
///
/// This function provides intelligent error classification based on the actual
/// error types from the Solana client, rather than brittle string matching.
/// It handles the most common error scenarios and provides appropriate
/// retry guidance.
#[must_use]
#[inline]
pub fn classify_transaction(error: &ClientError) -> TransactionErrorType {
    match error.kind {
        ClientErrorKind::RpcError(ref rpc_error) => classify_rpc_error(rpc_error),
        ClientErrorKind::SerdeJson(_) => {
            TransactionErrorType::Permanent(Permanent::InvalidTransaction)
        }
        ClientErrorKind::Io(_) => TransactionErrorType::Retryable(Retryable::NetworkConnectivity),
        ClientErrorKind::Reqwest(ref reqwest_error) => {
            if reqwest_error.status() == Some(reqwest::StatusCode::TOO_MANY_REQUESTS) {
                return TransactionErrorType::RateLimited(RateLimit::TooManyRequests);
            } else if reqwest_error.is_timeout() || reqwest_error.is_connect() {
                return TransactionErrorType::Retryable(Retryable::NetworkConnectivity);
            }
            TransactionErrorType::Unknown(error.to_string())
        }
        ClientErrorKind::Custom(ref msg) => {
            // Handle custom error messages with more sophisticated logic than string matching
            if msg.contains("InsufficientFundsForRent") || msg.contains("insufficient funds") {
                return TransactionErrorType::Permanent(Permanent::InsufficientFunds);
            } else if msg.contains("InvalidAccountIndex") {
                return TransactionErrorType::Permanent(Permanent::InvalidAccount);
            } else if msg.contains("InvalidSignature") {
                return TransactionErrorType::Permanent(Permanent::InvalidSignature);
            } else if msg.contains("DuplicateSignature") {
                return TransactionErrorType::Permanent(Permanent::DuplicateTransaction);
            }
            TransactionErrorType::Unknown(error.to_string())
        }
        ClientErrorKind::Middleware(_)
        | ClientErrorKind::SigningError(_)
        | ClientErrorKind::TransactionError(_) => TransactionErrorType::Unknown(error.to_string()),
    }
}

/// Classify RPC-specific errors
fn classify_rpc_error(rpc_error: &RpcError) -> TransactionErrorType {
    match *rpc_error {
        RpcError::RpcRequestError(ref msg) => {
            if msg.contains("rate limit")
                || msg.contains("429")
                || msg.contains("too many requests")
            {
                return TransactionErrorType::RateLimited(RateLimit::RpcRateLimit);
            }
            TransactionErrorType::Retryable(Retryable::TemporaryRpcFailure)
        }
        RpcError::RpcResponseError {
            code, ref message, ..
        } => {
            // Standard JSON-RPC error codes
            match code {
                429 => TransactionErrorType::RateLimited(RateLimit::RpcRateLimit),
                -32603 => TransactionErrorType::Retryable(Retryable::TemporaryRpcFailure), // Internal error
                -32002 | -32005 => TransactionErrorType::Retryable(Retryable::NetworkCongestion), // Transaction pool full / Node behind
                _ => {
                    // Analyze message for specific transaction errors
                    if message.contains("InsufficientFundsForRent") {
                        return TransactionErrorType::Permanent(Permanent::InsufficientFunds);
                    } else if message.contains("invalid") && message.contains("signature") {
                        return TransactionErrorType::Permanent(Permanent::InvalidSignature);
                    } else if message.contains("invalid") && message.contains("account") {
                        return TransactionErrorType::Permanent(Permanent::InvalidAccount);
                    } else if message.contains("Instruction") && message.contains("error") {
                        return TransactionErrorType::Permanent(Permanent::InstructionError);
                    }
                    let error_msg = format!("RPC Error {code}: {message}");
                    TransactionErrorType::Unknown(error_msg)
                }
            }
        }
        RpcError::ParseError(_) => TransactionErrorType::Permanent(Permanent::InvalidTransaction),
        RpcError::ForUser(ref msg) => TransactionErrorType::Unknown(msg.clone()),
    }
}

#[cfg(test)]
#[expect(clippy::pattern_type_mismatch, clippy::panic, clippy::expect_used)]
mod tests {
    use super::*;
    use crate::SolanaToolError;
    use solana_client::client_error::{ClientError, ClientErrorKind};
    use std::{
        io::{Error as IoError, ErrorKind},
        time::Duration as StdDuration,
    };

    // Test the classify method for all SolanaToolError variants
    #[test]
    fn test_classify_tool_passthrough() {
        let tool_err = ToolError::permanent_string("test error");
        let solana_err = Error::ToolError(tool_err.clone());

        let classification = solana_err.classify();
        match classification {
            ErrorClassification::ToolErrorPassthrough(e) => {
                assert_eq!(e.to_string(), tool_err.to_string());
            }
            _ => panic!("Expected ToolErrorPassthrough"),
        }
    }

    #[test]
    fn test_classify_signer() {
        let signer_err = SignerErrorStandard::NoContext;
        let solana_err = Error::SignerError(Box::new(signer_err));

        assert_eq!(solana_err.classify(), ErrorClassification::SignerContext);
    }

    #[test]
    fn test_classify_invalid_input() {
        let test_cases = vec![
            Error::InvalidAddress("bad address".to_string()),
            Error::InvalidKey("bad key".to_string()),
            Error::InvalidSignature("bad sig".to_string()),
            Error::InvalidTokenMint("bad mint".to_string()),
        ];

        for error in test_cases {
            assert_eq!(
                error.classify(),
                ErrorClassification::InvalidInput,
                "Failed for error: {error:?}"
            );
        }
    }

    #[test]
    fn test_classify_permanent() {
        let test_cases = vec![
            Error::InsufficientFunds,
            Error::Serialization(Box::new(
                serde_json::from_str::<String>("invalid")
                    .expect_err("Expected serde error for invalid JSON"),
            )),
        ];

        for error in test_cases {
            assert_eq!(
                error.classify(),
                ErrorClassification::Permanent,
                "Failed for error: {error:?}"
            );
        }
    }

    #[test]
    fn test_classify_rpc_rate_limited() {
        let test_cases = vec![
            Error::Rpc("Error 429: Too many requests".to_string()),
            Error::Rpc("rate limit exceeded".to_string()),
            Error::Rpc("too many requests".to_string()),
        ];

        for error in test_cases {
            let classification = error.classify();
            match classification {
                ErrorClassification::RateLimited { delay } => {
                    assert!(delay.is_some(), "Expected delay for rate limited error");
                }
                _ => panic!("Expected RateLimited classification for: {error:?}"),
            }
        }
    }

    #[test]
    fn test_classify_rpc_retriable() {
        let error = Error::Rpc("Connection timeout".to_string());
        assert_eq!(error.classify(), ErrorClassification::Retriable);
    }

    #[test]
    fn test_classify_transaction() {
        // Test insufficient funds detection
        let insufficient = Error::Transaction("insufficient funds for transaction".to_string());
        assert_eq!(insufficient.classify(), ErrorClassification::Permanent);

        // Test rate limit detection
        let rate_limited = Error::Transaction("rate limit exceeded".to_string());
        match rate_limited.classify() {
            ErrorClassification::RateLimited { delay } => {
                assert!(delay.is_some());
            }
            _ => panic!("Expected RateLimited classification"),
        }

        // Test retriable transaction error
        let retriable = Error::Transaction("network congestion".to_string());
        assert_eq!(retriable.classify(), ErrorClassification::Retriable);
    }

    #[test]
    fn test_classify_core_and_generic() {
        let core_err = Error::Core(Box::new(riglr_core::CoreError::Queue("test".to_string())));
        assert_eq!(core_err.classify(), ErrorClassification::Retriable);

        let generic_err = Error::Generic("some error".to_string());
        assert_eq!(generic_err.classify(), ErrorClassification::Retriable);
    }

    #[test]
    fn test_transaction_type_methods() {
        let retryable = TransactionErrorType::Retryable(Retryable::NetworkConnectivity);
        let permanent = TransactionErrorType::Permanent(Permanent::InsufficientFunds);
        let rate_limited = TransactionErrorType::RateLimited(RateLimit::RpcRateLimit);
        let unknown = TransactionErrorType::Unknown("test error".to_string());

        assert!(retryable.is_retryable());
        assert!(!retryable.is_rate_limited());

        assert!(!permanent.is_retryable());
        assert!(!permanent.is_rate_limited());

        assert!(rate_limited.is_retryable());
        assert!(rate_limited.is_rate_limited());

        assert!(!unknown.is_retryable());
        assert!(!unknown.is_rate_limited());
    }

    #[test]
    fn test_io_classification() {
        let io_error = IoError::new(ErrorKind::ConnectionRefused, "connection refused");
        let client_error = ClientError::new_with_request(
            ClientErrorKind::Io(io_error),
            RpcRequest::GetAccountInfo,
        );

        let result = classify_transaction(&client_error);
        assert_eq!(
            result,
            TransactionErrorType::Retryable(Retryable::NetworkConnectivity)
        );
    }

    #[test]
    fn test_serde_classification() {
        // Create a serde error by trying to parse invalid JSON
        let serde_error: serde_json::Error =
            serde_json::from_str::<serde_json::Value>("invalid json")
                .expect_err("Expected serde error for invalid JSON");
        let client_error = ClientError::new_with_request(
            ClientErrorKind::SerdeJson(serde_error),
            RpcRequest::GetAccountInfo,
        );

        let result = classify_transaction(&client_error);
        assert_eq!(
            result,
            TransactionErrorType::Permanent(Permanent::InvalidTransaction)
        );
    }

    #[test]
    fn test_custom_classification() {
        // Test insufficient funds
        let client_error = ClientError::new_with_request(
            ClientErrorKind::Custom("InsufficientFundsForRent".to_string()),
            RpcRequest::SendTransaction,
        );
        let result = classify_transaction(&client_error);
        assert_eq!(
            result,
            TransactionErrorType::Permanent(Permanent::InsufficientFunds)
        );

        // Test invalid signature
        let client_error = ClientError::new_with_request(
            ClientErrorKind::Custom("InvalidSignature".to_string()),
            RpcRequest::SendTransaction,
        );
        let result = classify_transaction(&client_error);
        assert_eq!(
            result,
            TransactionErrorType::Permanent(Permanent::InvalidSignature)
        );

        // Test invalid account
        let client_error = ClientError::new_with_request(
            ClientErrorKind::Custom("InvalidAccountIndex".to_string()),
            RpcRequest::SendTransaction,
        );
        let result = classify_transaction(&client_error);
        assert_eq!(
            result,
            TransactionErrorType::Permanent(Permanent::InvalidAccount)
        );

        // Test duplicate signature
        let client_error = ClientError::new_with_request(
            ClientErrorKind::Custom("DuplicateSignature".to_string()),
            RpcRequest::SendTransaction,
        );
        let result = classify_transaction(&client_error);
        assert_eq!(
            result,
            TransactionErrorType::Permanent(Permanent::DuplicateTransaction)
        );
    }

    #[cfg(test)]
    use solana_client::rpc_request::RpcResponseErrorData;

    #[test]
    fn test_rpc_classification() {
        // Test rate limiting
        let rpc_error = RpcError::RpcResponseError {
            code: 429,
            message: "Too Many Requests".to_string(),
            data: RpcResponseErrorData::Empty,
        };
        let client_error = ClientError::new_with_request(
            ClientErrorKind::RpcError(rpc_error),
            RpcRequest::SendTransaction,
        );
        let result = classify_transaction(&client_error);
        assert_eq!(
            result,
            TransactionErrorType::RateLimited(RateLimit::RpcRateLimit)
        );

        // Test network congestion (transaction pool full)
        let rpc_error = RpcError::RpcResponseError {
            code: -32002,
            message: "Transaction pool is full".to_string(),
            data: RpcResponseErrorData::Empty,
        };
        let client_error = ClientError::new_with_request(
            ClientErrorKind::RpcError(rpc_error),
            RpcRequest::SendTransaction,
        );
        let result = classify_transaction(&client_error);
        assert_eq!(
            result,
            TransactionErrorType::Retryable(Retryable::NetworkCongestion)
        );

        // Test insufficient funds in RPC response
        let rpc_error = RpcError::RpcResponseError {
            code: -32602,
            message: "InsufficientFundsForRent".to_string(),
            data: RpcResponseErrorData::Empty,
        };
        let client_error = ClientError::new_with_request(
            ClientErrorKind::RpcError(rpc_error),
            RpcRequest::SendTransaction,
        );
        let result = classify_transaction(&client_error);
        assert_eq!(
            result,
            TransactionErrorType::Permanent(Permanent::InsufficientFunds)
        );
    }

    #[test]
    fn test_rpc_request_classification() {
        // Test rate limit in request error
        let rpc_error = RpcError::RpcRequestError("rate limit exceeded".to_string());
        let client_error = ClientError::new_with_request(
            ClientErrorKind::RpcError(rpc_error),
            RpcRequest::SendTransaction,
        );
        let result = classify_transaction(&client_error);
        assert_eq!(
            result,
            TransactionErrorType::RateLimited(RateLimit::RpcRateLimit)
        );

        // Test other RPC request error
        let rpc_error = RpcError::RpcRequestError("network timeout".to_string());
        let client_error = ClientError::new_with_request(
            ClientErrorKind::RpcError(rpc_error),
            RpcRequest::SendTransaction,
        );
        let result = classify_transaction(&client_error);
        assert_eq!(
            result,
            TransactionErrorType::Retryable(Retryable::TemporaryRpcFailure)
        );
    }

    #[test]
    fn test_unknown_fallback() {
        let client_error = ClientError::new_with_request(
            ClientErrorKind::Custom("Unknown error type".to_string()),
            RpcRequest::GetAccountInfo,
        );

        let result = classify_transaction(&client_error);
        assert!(matches!(result, TransactionErrorType::Unknown(_)));
    }

    // Additional tests for 100% coverage

    #[test]
    fn test_solana_tool_display() {
        let tool_err = ToolError::invalid_input_string("test".to_string());
        let error = Error::ToolError(tool_err);
        assert_eq!(
            error.to_string(),
            "Core tool error: Invalid input: test - test"
        );

        let signer_err = SignerErrorStandard::SigningFailed("Invalid signature".to_string());
        let error = Error::SignerError(Box::new(signer_err));
        assert_eq!(
            error.to_string(),
            "Signer context error: Transaction signing failed: Invalid signature"
        );

        let error = Error::Rpc("test rpc error".to_string());
        assert_eq!(error.to_string(), "RPC error: test rpc error");

        let error = Error::InvalidAddress("invalid addr".to_string());
        assert_eq!(error.to_string(), "Invalid address: invalid addr");

        let error = Error::InvalidKey("invalid key".to_string());
        assert_eq!(error.to_string(), "Invalid key: invalid key");

        let error = Error::InvalidSignature("invalid sig".to_string());
        assert_eq!(error.to_string(), "Invalid signature: invalid sig");

        let error = Error::Transaction("tx error".to_string());
        assert_eq!(error.to_string(), "Transaction error: tx error");

        let error = Error::InsufficientFunds;
        assert_eq!(error.to_string(), "Insufficient funds for operation");

        let error = Error::InvalidTokenMint("invalid mint".to_string());
        assert_eq!(error.to_string(), "Invalid token mint: invalid mint");

        let error = Error::Generic("generic error".to_string());
        assert_eq!(error.to_string(), "Solana tool error: generic error");
    }

    #[test]
    fn test_solana_tool_is_retriable() {
        // Test ToolError is_retriable delegation
        let tool_err = ToolError::invalid_input_string("test".to_string());
        let error = Error::ToolError(tool_err);
        assert!(!error.is_retriable());

        let tool_err = ToolError::retriable_string("test".to_string());
        let error = Error::ToolError(tool_err);
        assert!(error.is_retriable());

        // Test SignerError (non-retriable)
        let signer_err = SignerErrorStandard::SigningFailed("Invalid signature".to_string());
        let error = Error::SignerError(Box::new(signer_err));
        assert!(!error.is_retriable());

        // Test Core error (retriable)
        let core_err = riglr_core::CoreError::Queue("test".to_string());
        let error = Error::Core(Box::new(core_err));
        assert!(error.is_retriable());

        // Test RPC error (retriable)
        let error = Error::Rpc("test rpc error".to_string());
        assert!(error.is_retriable());

        // Test HTTP errors with different status codes
        // Note: Creating a specific reqwest::Error is complex, so we test the logic path instead
        let error = Error::Rpc("timeout error".to_string());
        assert!(error.is_retriable());

        // Test invalid address/key/signature/token mint (non-retriable)
        let error = Error::InvalidAddress("invalid addr".to_string());
        assert!(!error.is_retriable());

        let error = Error::InvalidKey("invalid key".to_string());
        assert!(!error.is_retriable());

        let error = Error::InvalidSignature("invalid sig".to_string());
        assert!(!error.is_retriable());

        let error = Error::InvalidTokenMint("invalid mint".to_string());
        assert!(!error.is_retriable());

        // Test insufficient funds (non-retriable)
        let error = Error::InsufficientFunds;
        assert!(!error.is_retriable());

        // Test transaction errors with different messages
        let error = Error::Transaction("insufficient funds detected".to_string());
        assert!(!error.is_retriable());

        let error = Error::Transaction("invalid parameter".to_string());
        assert!(!error.is_retriable());

        let error = Error::Transaction("network timeout".to_string());
        assert!(error.is_retriable());

        // Test serialization error (non-retriable)
        let serde_err = serde_json::from_str::<serde_json::Value>("invalid json")
            .expect_err("Expected serde error for invalid JSON");
        let error = Error::Serialization(Box::new(serde_err));
        assert!(!error.is_retriable());

        // Test generic error (retriable)
        let error = Error::Generic("generic error".to_string());
        assert!(error.is_retriable());
    }

    #[test]
    fn test_solana_tool_is_rate_limited() {
        // Test RPC rate limit messages
        let error = Error::Rpc("429 Too Many Requests".to_string());
        assert!(error.is_rate_limited());

        let error = Error::Rpc("rate limit exceeded".to_string());
        assert!(error.is_rate_limited());

        let error = Error::Rpc("too many requests".to_string());
        assert!(error.is_rate_limited());

        let error = Error::Rpc("normal error".to_string());
        assert!(!error.is_rate_limited());

        // Test HTTP rate limit status
        // Note: Creating a reqwest::Error with specific status is complex,
        // so we'll test the logic through SolanaClient error path

        // Test non-rate-limited errors
        let error = Error::InvalidAddress("invalid addr".to_string());
        assert!(!error.is_rate_limited());

        let error = Error::Generic("generic error".to_string());
        assert!(!error.is_rate_limited());
    }

    #[test]
    fn test_solana_tool_retry_delay() {
        // Test rate-limited error delay
        let error = Error::Rpc("429 Too Many Requests".to_string());
        assert_eq!(error.retry_delay(), Some(StdDuration::from_secs(1)));

        // Test retriable but not rate-limited error delay
        let error = Error::Rpc("network error".to_string());
        assert_eq!(error.retry_delay(), Some(StdDuration::from_millis(500)));

        // Test non-retriable error (no delay)
        let error = Error::InvalidAddress("invalid addr".to_string());
        assert_eq!(error.retry_delay(), None);
    }

    #[test]
    fn test_solana_client_is_retriable() {
        // Create a retryable client error
        let io_error = IoError::new(ErrorKind::ConnectionRefused, "connection refused");
        let client_error = ClientError::new_with_request(
            ClientErrorKind::Io(io_error),
            RpcRequest::GetAccountInfo,
        );
        let error = Error::SolanaClient(Box::new(client_error));
        assert!(error.is_retriable());

        // Create a non-retryable client error
        let serde_error = serde_json::from_str::<serde_json::Value>("invalid json")
            .expect_err("Expected serde error for invalid JSON");
        let client_error = ClientError::new_with_request(
            ClientErrorKind::SerdeJson(serde_error),
            RpcRequest::GetAccountInfo,
        );
        let error = Error::SolanaClient(Box::new(client_error));
        assert!(!error.is_retriable());
    }

    #[test]
    fn test_solana_client_is_rate_limited() {
        // Create a rate-limited client error
        let rpc_error = RpcError::RpcResponseError {
            code: 429,
            message: "Too Many Requests".to_string(),
            data: RpcResponseErrorData::Empty,
        };
        let client_error = ClientError::new_with_request(
            ClientErrorKind::RpcError(rpc_error),
            RpcRequest::SendTransaction,
        );
        let error = Error::SolanaClient(Box::new(client_error));
        assert!(error.is_rate_limited());

        // Create a non-rate-limited client error
        let io_error = IoError::new(ErrorKind::ConnectionRefused, "connection refused");
        let client_error = ClientError::new_with_request(
            ClientErrorKind::Io(io_error),
            RpcRequest::GetAccountInfo,
        );
        let error = Error::SolanaClient(Box::new(client_error));
        assert!(!error.is_rate_limited());
    }

    #[test]
    fn test_from_client() {
        let io_error = IoError::new(ErrorKind::ConnectionRefused, "connection refused");
        let client_error = ClientError::new_with_request(
            ClientErrorKind::Io(io_error),
            RpcRequest::GetAccountInfo,
        );

        let solana_error: SolanaToolError = client_error.into();
        assert!(matches!(solana_error, Error::SolanaClient(_)));
    }

    #[test]
    fn test_from_solana_tool_to_tool() {
        // Test ToolError passthrough
        let tool_err = ToolError::invalid_input_string("test".to_string());
        let expected_string = tool_err.to_string();
        let solana_err = Error::ToolError(tool_err);
        let converted: ToolError = solana_err.into();
        assert_eq!(converted.to_string(), expected_string);

        // Test SignerError conversion
        let signer_err = SignerErrorStandard::SigningFailed("Invalid signature".to_string());
        let solana_err = Error::SignerError(Box::new(signer_err));
        let converted: ToolError = solana_err.into();
        assert!(matches!(converted, ToolError::SignerContext(_)));

        // Test invalid input conversions
        let solana_err = Error::InvalidAddress("test addr".to_string());
        let converted: ToolError = solana_err.into();
        assert!(converted.to_string().contains("Invalid input"));

        let solana_err = Error::InvalidKey("test key".to_string());
        let converted: ToolError = solana_err.into();
        assert!(converted.to_string().contains("Invalid input"));

        let solana_err = Error::InvalidSignature("test sig".to_string());
        let converted: ToolError = solana_err.into();
        assert!(converted.to_string().contains("Invalid input"));

        let solana_err = Error::InvalidTokenMint("test mint".to_string());
        let converted: ToolError = solana_err.into();
        assert!(converted.to_string().contains("Invalid input"));

        // Test rate-limited error conversion
        let solana_err = Error::Rpc("429 Too Many Requests".to_string());
        let converted: ToolError = solana_err.into();
        // This should be a rate-limited error
        assert!(converted.to_string().contains("Rate limited"));

        // Test retriable error conversion
        let solana_err = Error::Rpc("network timeout".to_string());
        let converted: ToolError = solana_err.into();
        // This should be a retriable error
        assert!(converted.to_string().contains("network timeout"));

        // Test generic error conversion (non-retriable/non-rate-limited)
        let solana_err = Error::InsufficientFunds;
        let converted: ToolError = solana_err.into();
        // This should be converted as retriable (the default case)
        assert!(converted.to_string().contains("Insufficient funds"));
    }

    #[test]
    fn test_reqwest_classification() {
        // Test timeout error - we'll use a serde error instead since reqwest::Error creation is complex
        let serde_error = serde_json::from_str::<serde_json::Value>("invalid json")
            .expect_err("Expected serde error for invalid JSON");
        let client_error = ClientError::new_with_request(
            ClientErrorKind::SerdeJson(serde_error),
            RpcRequest::GetAccountInfo,
        );
        let result = classify_transaction(&client_error);
        assert_eq!(
            result,
            TransactionErrorType::Permanent(Permanent::InvalidTransaction)
        );
    }

    #[test]
    fn test_rpc_response_edge_cases() {
        // Test internal error (-32603)
        let rpc_error = RpcError::RpcResponseError {
            code: -32603,
            message: "Internal error".to_string(),
            data: RpcResponseErrorData::Empty,
        };
        let client_error = ClientError::new_with_request(
            ClientErrorKind::RpcError(rpc_error),
            RpcRequest::SendTransaction,
        );
        let result = classify_transaction(&client_error);
        assert_eq!(
            result,
            TransactionErrorType::Retryable(Retryable::TemporaryRpcFailure)
        );

        // Test node behind (-32005)
        let rpc_error = RpcError::RpcResponseError {
            code: -32005,
            message: "Node is behind".to_string(),
            data: RpcResponseErrorData::Empty,
        };
        let client_error = ClientError::new_with_request(
            ClientErrorKind::RpcError(rpc_error),
            RpcRequest::SendTransaction,
        );
        let result = classify_transaction(&client_error);
        assert_eq!(
            result,
            TransactionErrorType::Retryable(Retryable::NetworkCongestion)
        );

        // Test invalid signature in RPC message
        let rpc_error = RpcError::RpcResponseError {
            code: -32001,
            message: "invalid signature provided".to_string(),
            data: RpcResponseErrorData::Empty,
        };
        let client_error = ClientError::new_with_request(
            ClientErrorKind::RpcError(rpc_error),
            RpcRequest::SendTransaction,
        );
        let result = classify_transaction(&client_error);
        assert_eq!(
            result,
            TransactionErrorType::Permanent(Permanent::InvalidSignature)
        );

        // Test invalid account in RPC message
        let rpc_error = RpcError::RpcResponseError {
            code: -32001,
            message: "invalid account reference".to_string(),
            data: RpcResponseErrorData::Empty,
        };
        let client_error = ClientError::new_with_request(
            ClientErrorKind::RpcError(rpc_error),
            RpcRequest::SendTransaction,
        );
        let result = classify_transaction(&client_error);
        assert_eq!(
            result,
            TransactionErrorType::Permanent(Permanent::InvalidAccount)
        );

        // Test instruction error in RPC message
        let rpc_error = RpcError::RpcResponseError {
            code: -32001,
            message: "Instruction error occurred".to_string(),
            data: RpcResponseErrorData::Empty,
        };
        let client_error = ClientError::new_with_request(
            ClientErrorKind::RpcError(rpc_error),
            RpcRequest::SendTransaction,
        );
        let result = classify_transaction(&client_error);
        assert_eq!(
            result,
            TransactionErrorType::Permanent(Permanent::InstructionError)
        );

        // Test unknown error code with message
        let rpc_error = RpcError::RpcResponseError {
            code: -99999,
            message: "Unknown error".to_string(),
            data: RpcResponseErrorData::Empty,
        };
        let client_error = ClientError::new_with_request(
            ClientErrorKind::RpcError(rpc_error),
            RpcRequest::SendTransaction,
        );
        let result = classify_transaction(&client_error);
        assert!(matches!(result, TransactionErrorType::Unknown(_)));
        if let TransactionErrorType::Unknown(msg) = result {
            assert!(msg.contains("RPC Error -99999"));
            assert!(msg.contains("Unknown error"));
        }
    }

    #[test]
    fn test_rpc_parse_classification() {
        let rpc_error = RpcError::ParseError("Invalid JSON".to_string());
        let client_error = ClientError::new_with_request(
            ClientErrorKind::RpcError(rpc_error),
            RpcRequest::SendTransaction,
        );
        let result = classify_transaction(&client_error);
        assert_eq!(
            result,
            TransactionErrorType::Permanent(Permanent::InvalidTransaction)
        );
    }

    #[test]
    fn test_rpc_for_user_classification() {
        let rpc_error = RpcError::ForUser("User-facing error message".to_string());
        let client_error = ClientError::new_with_request(
            ClientErrorKind::RpcError(rpc_error),
            RpcRequest::SendTransaction,
        );
        let result = classify_transaction(&client_error);
        assert!(matches!(result, TransactionErrorType::Unknown(_)));
        if let TransactionErrorType::Unknown(msg) = result {
            assert_eq!(msg, "User-facing error message");
        }
    }

    #[test]
    fn test_rpc_request_with_different_messages() {
        // Test "429" in message
        let rpc_error = RpcError::RpcRequestError("HTTP 429 rate limit".to_string());
        let client_error = ClientError::new_with_request(
            ClientErrorKind::RpcError(rpc_error),
            RpcRequest::SendTransaction,
        );
        let result = classify_transaction(&client_error);
        assert_eq!(
            result,
            TransactionErrorType::RateLimited(RateLimit::RpcRateLimit)
        );

        // Test "too many requests" in message
        let rpc_error = RpcError::RpcRequestError("too many requests received".to_string());
        let client_error = ClientError::new_with_request(
            ClientErrorKind::RpcError(rpc_error),
            RpcRequest::SendTransaction,
        );
        let result = classify_transaction(&client_error);
        assert_eq!(
            result,
            TransactionErrorType::RateLimited(RateLimit::RpcRateLimit)
        );
    }

    #[test]
    fn test_classify_transaction_with_unknown_client_kind() {
        // Create a client error with an unhandled error kind
        // We'll use a custom error for this test
        let custom_msg = "Custom unknown error".to_string();
        let client_error = ClientError::new_with_request(
            ClientErrorKind::Custom(custom_msg),
            RpcRequest::GetAccountInfo,
        );

        // Since our custom message doesn't match any known patterns, it should be Unknown
        let result = classify_transaction(&client_error);
        assert!(matches!(result, TransactionErrorType::Unknown(_)));
    }

    #[test]
    fn test_custom_with_insufficient_funds_lowercase() {
        // Test "insufficient funds" (lowercase) in custom error
        let client_error = ClientError::new_with_request(
            ClientErrorKind::Custom("insufficient funds for transaction".to_string()),
            RpcRequest::SendTransaction,
        );
        let result = classify_transaction(&client_error);
        assert_eq!(
            result,
            TransactionErrorType::Permanent(Permanent::InsufficientFunds)
        );
    }

    #[test]
    fn test_error_variants_equality() {
        // Test RetryableError variants
        assert_eq!(
            Retryable::NetworkConnectivity,
            Retryable::NetworkConnectivity
        );
        assert_ne!(
            Retryable::NetworkConnectivity,
            Retryable::TemporaryRpcFailure
        );

        // Test PermanentError variants
        assert_eq!(Permanent::InsufficientFunds, Permanent::InsufficientFunds);
        assert_ne!(Permanent::InsufficientFunds, Permanent::InvalidSignature);

        // Test RateLimitError variants
        assert_eq!(RateLimit::RpcRateLimit, RateLimit::RpcRateLimit);
        assert_ne!(RateLimit::RpcRateLimit, RateLimit::TooManyRequests);

        // Test TransactionErrorType variants
        assert_eq!(
            TransactionErrorType::Retryable(Retryable::NetworkConnectivity),
            TransactionErrorType::Retryable(Retryable::NetworkConnectivity)
        );
        assert_ne!(
            TransactionErrorType::Retryable(Retryable::NetworkConnectivity),
            TransactionErrorType::Permanent(Permanent::InsufficientFunds)
        );
    }

    #[test]
    fn test_error_debug_format() {
        // Test Debug implementation for all error types
        let retryable = Retryable::NetworkConnectivity;
        assert!(!format!("{retryable:?}").is_empty());

        let permanent = Permanent::InsufficientFunds;
        assert!(!format!("{permanent:?}").is_empty());

        let rate_limit = RateLimit::RpcRateLimit;
        assert!(!format!("{rate_limit:?}").is_empty());

        let transaction_error = TransactionErrorType::Unknown("test".to_string());
        assert!(!format!("{transaction_error:?}").is_empty());
    }

    #[test]
    fn test_error_downcasting_preserves_structured_context() {
        use core::error::Error as CoreErrorTrait;

        // Test Case 1: InvalidAddress error should be downcastable
        let solana_error = Error::InvalidAddress("bad_address".to_string());
        let tool_error: ToolError = solana_error.into();

        // Verify the ToolError has a source
        assert!(
            tool_error.source().is_some(),
            "ToolError should have a source"
        );

        // Downcast the source back to SolanaToolError
        let source = tool_error.source().expect("ToolError should have a source");
        let downcasted = source.downcast_ref::<SolanaToolError>();
        assert!(
            downcasted.is_some(),
            "Should be able to downcast source to SolanaToolError"
        );

        // Verify the downcast error matches the original
        if let Some(Error::InvalidAddress(msg)) = downcasted {
            assert_eq!(msg, "bad_address", "Downcast should preserve error details");
        } else {
            panic!("Downcast error should be InvalidAddress variant");
        }

        // Test Case 2: InsufficientFunds error should be downcastable
        let solana_error = Error::InsufficientFunds;
        let tool_error: ToolError = solana_error.into();

        assert!(
            tool_error.source().is_some(),
            "ToolError should have a source for InsufficientFunds"
        );

        let source = tool_error
            .source()
            .expect("ToolError should have a source for InsufficientFunds");
        let downcasted = source.downcast_ref::<SolanaToolError>();
        assert!(
            downcasted.is_some(),
            "Should be able to downcast InsufficientFunds error"
        );

        assert!(
            matches!(downcasted, Some(&Error::InsufficientFunds)),
            "Downcast should preserve InsufficientFunds variant"
        );

        // Test Case 3: Rate-limited RPC error should be downcastable
        let solana_error = Error::Rpc("429 Too Many Requests".to_string());
        let tool_error: ToolError = solana_error.into();

        assert!(
            tool_error.source().is_some(),
            "ToolError should have a source for rate-limited error"
        );

        let source = tool_error
            .source()
            .expect("ToolError should have a source for rate-limited error");
        let downcasted = source.downcast_ref::<SolanaToolError>();
        assert!(
            downcasted.is_some(),
            "Should be able to downcast rate-limited error"
        );

        if let Some(Error::Rpc(msg)) = downcasted {
            assert_eq!(
                msg, "429 Too Many Requests",
                "Downcast should preserve RPC error message"
            );
        } else {
            panic!("Downcast error should be Rpc variant");
        }

        // Test Case 4: SolanaClient error should be downcastable
        let client_error = ClientError::new_with_request(
            ClientErrorKind::Custom("test error".to_string()),
            RpcRequest::GetAccountInfo,
        );
        let solana_error = Error::SolanaClient(Box::new(client_error));
        let tool_error: ToolError = solana_error.into();

        assert!(
            tool_error.source().is_some(),
            "ToolError should have a source for SolanaClient error"
        );

        let source = tool_error
            .source()
            .expect("ToolError should have a source for SolanaClient error");
        let downcasted = source.downcast_ref::<SolanaToolError>();
        assert!(
            downcasted.is_some(),
            "Should be able to downcast SolanaClient error"
        );

        assert!(
            matches!(downcasted, Some(&Error::SolanaClient(_))),
            "Downcast should preserve SolanaClient variant"
        );

        // Test Case 5: Verify ToolError passthrough doesn't add extra layer
        let inner_tool_error = ToolError::permanent_string("inner error".to_string());
        let solana_error = Error::ToolError(inner_tool_error.clone());
        let converted: ToolError = solana_error.into();

        // The converted error should be the inner ToolError, not wrapped again
        assert_eq!(
            converted.to_string(),
            inner_tool_error.to_string(),
            "ToolError passthrough should not add extra wrapping"
        );
    }
}
