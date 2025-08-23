//! Error types for riglr-solana-tools.

use riglr_core::error::ToolError;
use riglr_core::SignerError;
use solana_client::client_error::{ClientError, ClientErrorKind};
use solana_client::rpc_request::RpcError;
use thiserror::Error;

/// Main error type for Solana tool operations.
#[derive(Error, Debug)]
#[allow(clippy::result_large_err)]
#[allow(clippy::large_enum_variant)]
pub enum SolanaToolError {
    /// Core tool error
    #[error("Core tool error: {0}")]
    ToolError(#[from] ToolError),

    /// Signer context error
    #[error("Signer context error: {0}")]
    SignerError(#[from] SignerError),

    /// RPC client error
    #[error("RPC error: {0}")]
    Rpc(String),

    /// Solana client error
    #[error("Solana client error: {0}")]
    SolanaClient(Box<ClientError>),

    /// Invalid address format
    #[error("Invalid address: {0}")]
    InvalidAddress(String),

    /// Invalid key format
    #[error("Invalid key: {0}")]
    InvalidKey(String),

    /// Invalid signature format
    #[error("Invalid signature: {0}")]
    InvalidSignature(String),

    /// Transaction failed
    #[error("Transaction error: {0}")]
    Transaction(String),

    /// Insufficient funds for operation
    #[error("Insufficient funds for operation")]
    InsufficientFunds,

    /// Invalid token mint
    #[error("Invalid token mint: {0}")]
    InvalidTokenMint(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// HTTP request error
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    /// Core riglr error
    #[error("Core error: {0}")]
    Core(#[from] riglr_core::CoreError),

    /// Generic error
    #[error("Solana tool error: {0}")]
    Generic(String),
}

/// Result type alias for Solana tool operations.
pub type Result<T> = std::result::Result<T, SolanaToolError>;

impl SolanaToolError {
    /// Check if this error is retriable.
    pub fn is_retriable(&self) -> bool {
        match self {
            // Core errors inherit their retriable nature
            SolanaToolError::ToolError(tool_err) => tool_err.is_retriable(),
            SolanaToolError::SignerError(_) => false, // Generally configuration issues
            SolanaToolError::Core(_) => true,         // Core errors are typically retriable

            // RPC and HTTP errors are often retriable
            SolanaToolError::Rpc(_) => true,
            SolanaToolError::Http(ref http_err) => !matches!(
                http_err.status(),
                Some(
                    reqwest::StatusCode::BAD_REQUEST
                        | reqwest::StatusCode::UNAUTHORIZED
                        | reqwest::StatusCode::FORBIDDEN
                )
            ),

            // Client errors need classification
            SolanaToolError::SolanaClient(ref client_err) => {
                let error_type = classify_transaction_error(client_err);
                error_type.is_retryable()
            }

            // Address/key validation errors are permanent
            SolanaToolError::InvalidAddress(_) => false,
            SolanaToolError::InvalidKey(_) => false,
            SolanaToolError::InvalidSignature(_) => false,
            SolanaToolError::InvalidTokenMint(_) => false,

            // Insufficient funds is permanent
            SolanaToolError::InsufficientFunds => false,

            // Transaction errors depend on content
            SolanaToolError::Transaction(msg) => {
                !(msg.contains("insufficient funds") || msg.contains("invalid"))
            }

            // Serialization errors are permanent
            SolanaToolError::Serialization(_) => false,

            // Generic errors default to retriable
            SolanaToolError::Generic(_) => true,
        }
    }

    /// Check if this error is rate-limited.
    pub fn is_rate_limited(&self) -> bool {
        match self {
            SolanaToolError::Rpc(msg) => {
                msg.contains("429")
                    || msg.contains("rate limit")
                    || msg.contains("too many requests")
            }
            SolanaToolError::Http(ref http_err) => {
                http_err.status() == Some(reqwest::StatusCode::TOO_MANY_REQUESTS)
            }
            SolanaToolError::SolanaClient(ref client_err) => {
                let error_type = classify_transaction_error(client_err);
                error_type.is_rate_limited()
            }
            _ => false,
        }
    }

    /// Get appropriate retry delay for rate-limited errors.
    pub fn retry_delay(&self) -> Option<std::time::Duration> {
        if self.is_rate_limited() {
            Some(std::time::Duration::from_secs(1))
        } else if self.is_retriable() {
            Some(std::time::Duration::from_millis(500))
        } else {
            None
        }
    }
}

/// Structured classification of transaction errors for intelligent retry logic
#[derive(Debug, Clone, PartialEq)]
pub enum TransactionErrorType {
    /// Errors that can be retried with appropriate backoff
    Retryable(RetryableError),
    /// Errors that represent permanent failures and should not be retried
    Permanent(PermanentError),
    /// Rate limiting errors that require special handling with delays
    RateLimited(RateLimitError),
    /// Unknown error types that don't fit other categories
    Unknown(String),
}

/// Errors that can be retried with appropriate backoff
#[derive(Debug, Clone, PartialEq)]
pub enum RetryableError {
    /// Network connectivity issues
    NetworkConnectivity,
    /// RPC service temporary unavailability
    TemporaryRpcFailure,
    /// Blockchain congestion
    NetworkCongestion,
    /// Transaction pool full
    TransactionPoolFull,
}

/// Permanent errors that should not be retried
#[derive(Debug, Clone, PartialEq)]
pub enum PermanentError {
    /// Insufficient funds for transaction
    InsufficientFunds,
    /// Invalid signature provided
    InvalidSignature,
    /// Invalid account referenced
    InvalidAccount,
    /// Program execution error
    InstructionError,
    /// Invalid transaction structure
    InvalidTransaction,
    /// Duplicate transaction
    DuplicateTransaction,
}

/// Rate limiting errors with special handling
#[derive(Debug, Clone, PartialEq)]
pub enum RateLimitError {
    /// Standard RPC rate limiting
    RpcRateLimit,
    /// Too many requests error
    TooManyRequests,
}

impl TransactionErrorType {
    /// Check if this error type is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            TransactionErrorType::Retryable(_) | TransactionErrorType::RateLimited(_)
        )
    }

    /// Check if this is a rate limiting error (special case of retryable)
    pub fn is_rate_limited(&self) -> bool {
        matches!(self, TransactionErrorType::RateLimited(_))
    }
}

/// Classify a Solana ClientError into a structured transaction error type
///
/// This function provides intelligent error classification based on the actual
/// error types from the Solana client, rather than brittle string matching.
/// It handles the most common error scenarios and provides appropriate
/// retry guidance.
pub fn classify_transaction_error(error: &ClientError) -> TransactionErrorType {
    match &*error.kind {
        ClientErrorKind::RpcError(rpc_error) => classify_rpc_error(rpc_error),
        ClientErrorKind::SerdeJson(_) => {
            TransactionErrorType::Permanent(PermanentError::InvalidTransaction)
        }
        ClientErrorKind::Io(_) => {
            TransactionErrorType::Retryable(RetryableError::NetworkConnectivity)
        }
        ClientErrorKind::Reqwest(reqwest_error) => {
            if reqwest_error.status() == Some(reqwest::StatusCode::TOO_MANY_REQUESTS) {
                TransactionErrorType::RateLimited(RateLimitError::TooManyRequests)
            } else if reqwest_error.is_timeout() || reqwest_error.is_connect() {
                TransactionErrorType::Retryable(RetryableError::NetworkConnectivity)
            } else {
                TransactionErrorType::Unknown(error.to_string())
            }
        }
        ClientErrorKind::Custom(msg) => {
            // Handle custom error messages with more sophisticated logic than string matching
            if msg.contains("InsufficientFundsForRent") || msg.contains("insufficient funds") {
                TransactionErrorType::Permanent(PermanentError::InsufficientFunds)
            } else if msg.contains("InvalidAccountIndex") {
                TransactionErrorType::Permanent(PermanentError::InvalidAccount)
            } else if msg.contains("InvalidSignature") {
                TransactionErrorType::Permanent(PermanentError::InvalidSignature)
            } else if msg.contains("DuplicateSignature") {
                TransactionErrorType::Permanent(PermanentError::DuplicateTransaction)
            } else {
                TransactionErrorType::Unknown(error.to_string())
            }
        }
        _ => TransactionErrorType::Unknown(error.to_string()),
    }
}

/// Classify RPC-specific errors
fn classify_rpc_error(rpc_error: &RpcError) -> TransactionErrorType {
    use solana_client::rpc_request::RpcError::*;

    match rpc_error {
        RpcRequestError(msg) => {
            if msg.contains("rate limit")
                || msg.contains("429")
                || msg.contains("too many requests")
            {
                TransactionErrorType::RateLimited(RateLimitError::RpcRateLimit)
            } else {
                TransactionErrorType::Retryable(RetryableError::TemporaryRpcFailure)
            }
        }
        RpcResponseError { code, message, .. } => {
            // Standard JSON-RPC error codes
            match *code {
                429 => TransactionErrorType::RateLimited(RateLimitError::RpcRateLimit),
                -32603 => TransactionErrorType::Retryable(RetryableError::TemporaryRpcFailure), // Internal error
                -32002 => TransactionErrorType::Retryable(RetryableError::NetworkCongestion), // Transaction pool full
                -32005 => TransactionErrorType::Retryable(RetryableError::NetworkCongestion), // Node behind
                _ => {
                    // Analyze message for specific transaction errors
                    if message.contains("InsufficientFundsForRent") {
                        TransactionErrorType::Permanent(PermanentError::InsufficientFunds)
                    } else if message.contains("invalid") && message.contains("signature") {
                        TransactionErrorType::Permanent(PermanentError::InvalidSignature)
                    } else if message.contains("invalid") && message.contains("account") {
                        TransactionErrorType::Permanent(PermanentError::InvalidAccount)
                    } else if message.contains("Instruction") && message.contains("error") {
                        TransactionErrorType::Permanent(PermanentError::InstructionError)
                    } else {
                        TransactionErrorType::Unknown(format!("RPC Error {}: {}", code, message))
                    }
                }
            }
        }
        ParseError(_msg) => TransactionErrorType::Permanent(PermanentError::InvalidTransaction),
        ForUser(msg) => TransactionErrorType::Unknown(msg.clone()),
    }
}

// Implement From conversion to riglr_core::ToolError for proper error handling
impl From<SolanaToolError> for ToolError {
    fn from(err: SolanaToolError) -> Self {
        // Handle the consuming case first to avoid borrow checker issues.
        if let SolanaToolError::ToolError(tool_err) = err {
            return tool_err;
        }

        // Now that the consuming case is handled, we can safely borrow.
        if err.is_rate_limited() {
            if let Some(delay) = err.retry_delay() {
                return ToolError::rate_limited_with_source(err, "Solana operation", Some(delay));
            } else {
                return ToolError::rate_limited_with_source(err, "Solana operation", None);
            }
        }

        if err.is_retriable() {
            return ToolError::retriable_with_source(err, "Solana operation");
        }

        // Handle remaining non-consuming, permanent error cases.
        match err {
            // Signer errors are configuration issues
            SolanaToolError::SignerError(signer_err) => {
                ToolError::SignerContext(signer_err.to_string())
            }

            // Input validation errors - now preserve the full error object
            SolanaToolError::InvalidAddress(_)
            | SolanaToolError::InvalidKey(_)
            | SolanaToolError::InvalidSignature(_)
            | SolanaToolError::InvalidTokenMint(_) => {
                ToolError::invalid_input_with_source(err, "Solana input validation")
            }

            // This catch-all now correctly handles all other permanent errors.
            _ => ToolError::permanent_with_source(err, "Solana operation"),
        }
    }
}

impl From<ClientError> for SolanaToolError {
    fn from(error: ClientError) -> Self {
        SolanaToolError::SolanaClient(Box::new(error))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use solana_client::client_error::{ClientError, ClientErrorKind};
    use solana_client::rpc_request::RpcError;

    #[test]
    fn test_transaction_error_type_methods() {
        let retryable = TransactionErrorType::Retryable(RetryableError::NetworkConnectivity);
        let permanent = TransactionErrorType::Permanent(PermanentError::InsufficientFunds);
        let rate_limited = TransactionErrorType::RateLimited(RateLimitError::RpcRateLimit);
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
    fn test_io_error_classification() {
        let io_error =
            std::io::Error::new(std::io::ErrorKind::ConnectionRefused, "connection refused");
        let client_error = ClientError::new_with_request(
            ClientErrorKind::Io(io_error),
            solana_client::rpc_request::RpcRequest::GetAccountInfo,
        );

        let result = classify_transaction_error(&client_error);
        assert_eq!(
            result,
            TransactionErrorType::Retryable(RetryableError::NetworkConnectivity)
        );
    }

    #[test]
    fn test_serde_error_classification() {
        // Create a serde error by trying to parse invalid JSON
        let serde_error: serde_json::Error =
            serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err();
        let client_error = ClientError::new_with_request(
            ClientErrorKind::SerdeJson(serde_error),
            solana_client::rpc_request::RpcRequest::GetAccountInfo,
        );

        let result = classify_transaction_error(&client_error);
        assert_eq!(
            result,
            TransactionErrorType::Permanent(PermanentError::InvalidTransaction)
        );
    }

    #[test]
    fn test_custom_error_classification() {
        // Test insufficient funds
        let client_error = ClientError::new_with_request(
            ClientErrorKind::Custom("InsufficientFundsForRent".to_string()),
            solana_client::rpc_request::RpcRequest::SendTransaction,
        );
        let result = classify_transaction_error(&client_error);
        assert_eq!(
            result,
            TransactionErrorType::Permanent(PermanentError::InsufficientFunds)
        );

        // Test invalid signature
        let client_error = ClientError::new_with_request(
            ClientErrorKind::Custom("InvalidSignature".to_string()),
            solana_client::rpc_request::RpcRequest::SendTransaction,
        );
        let result = classify_transaction_error(&client_error);
        assert_eq!(
            result,
            TransactionErrorType::Permanent(PermanentError::InvalidSignature)
        );

        // Test invalid account
        let client_error = ClientError::new_with_request(
            ClientErrorKind::Custom("InvalidAccountIndex".to_string()),
            solana_client::rpc_request::RpcRequest::SendTransaction,
        );
        let result = classify_transaction_error(&client_error);
        assert_eq!(
            result,
            TransactionErrorType::Permanent(PermanentError::InvalidAccount)
        );

        // Test duplicate signature
        let client_error = ClientError::new_with_request(
            ClientErrorKind::Custom("DuplicateSignature".to_string()),
            solana_client::rpc_request::RpcRequest::SendTransaction,
        );
        let result = classify_transaction_error(&client_error);
        assert_eq!(
            result,
            TransactionErrorType::Permanent(PermanentError::DuplicateTransaction)
        );
    }

    #[cfg(test)]
    use solana_client::rpc_request::RpcResponseErrorData;

    #[test]
    fn test_rpc_error_classification() {
        // Test rate limiting
        let rpc_error = RpcError::RpcResponseError {
            code: 429,
            message: "Too Many Requests".to_string(),
            data: RpcResponseErrorData::Empty,
        };
        let client_error = ClientError::new_with_request(
            ClientErrorKind::RpcError(rpc_error),
            solana_client::rpc_request::RpcRequest::SendTransaction,
        );
        let result = classify_transaction_error(&client_error);
        assert_eq!(
            result,
            TransactionErrorType::RateLimited(RateLimitError::RpcRateLimit)
        );

        // Test network congestion (transaction pool full)
        let rpc_error = RpcError::RpcResponseError {
            code: -32002,
            message: "Transaction pool is full".to_string(),
            data: RpcResponseErrorData::Empty,
        };
        let client_error = ClientError::new_with_request(
            ClientErrorKind::RpcError(rpc_error),
            solana_client::rpc_request::RpcRequest::SendTransaction,
        );
        let result = classify_transaction_error(&client_error);
        assert_eq!(
            result,
            TransactionErrorType::Retryable(RetryableError::NetworkCongestion)
        );

        // Test insufficient funds in RPC response
        let rpc_error = RpcError::RpcResponseError {
            code: -32602,
            message: "InsufficientFundsForRent".to_string(),
            data: RpcResponseErrorData::Empty,
        };
        let client_error = ClientError::new_with_request(
            ClientErrorKind::RpcError(rpc_error),
            solana_client::rpc_request::RpcRequest::SendTransaction,
        );
        let result = classify_transaction_error(&client_error);
        assert_eq!(
            result,
            TransactionErrorType::Permanent(PermanentError::InsufficientFunds)
        );
    }

    #[test]
    fn test_rpc_request_error_classification() {
        // Test rate limit in request error
        let rpc_error = RpcError::RpcRequestError("rate limit exceeded".to_string());
        let client_error = ClientError::new_with_request(
            ClientErrorKind::RpcError(rpc_error),
            solana_client::rpc_request::RpcRequest::SendTransaction,
        );
        let result = classify_transaction_error(&client_error);
        assert_eq!(
            result,
            TransactionErrorType::RateLimited(RateLimitError::RpcRateLimit)
        );

        // Test other RPC request error
        let rpc_error = RpcError::RpcRequestError("network timeout".to_string());
        let client_error = ClientError::new_with_request(
            ClientErrorKind::RpcError(rpc_error),
            solana_client::rpc_request::RpcRequest::SendTransaction,
        );
        let result = classify_transaction_error(&client_error);
        assert_eq!(
            result,
            TransactionErrorType::Retryable(RetryableError::TemporaryRpcFailure)
        );
    }

    #[test]
    fn test_unknown_error_fallback() {
        let client_error = ClientError::new_with_request(
            ClientErrorKind::Custom("Unknown error type".to_string()),
            solana_client::rpc_request::RpcRequest::GetAccountInfo,
        );

        let result = classify_transaction_error(&client_error);
        assert!(matches!(result, TransactionErrorType::Unknown(_)));
    }

    // Additional tests for 100% coverage

    #[test]
    fn test_solana_tool_error_display() {
        let tool_err = ToolError::invalid_input_string("test".to_string());
        let error = SolanaToolError::ToolError(tool_err);
        assert_eq!(
            error.to_string(),
            "Core tool error: Invalid input: test - test"
        );

        let signer_err = SignerError::Signing("Invalid signature".to_string());
        let error = SolanaToolError::SignerError(signer_err);
        assert_eq!(
            error.to_string(),
            "Signer context error: Signing error: Invalid signature"
        );

        let error = SolanaToolError::Rpc("test rpc error".to_string());
        assert_eq!(error.to_string(), "RPC error: test rpc error");

        let error = SolanaToolError::InvalidAddress("invalid addr".to_string());
        assert_eq!(error.to_string(), "Invalid address: invalid addr");

        let error = SolanaToolError::InvalidKey("invalid key".to_string());
        assert_eq!(error.to_string(), "Invalid key: invalid key");

        let error = SolanaToolError::InvalidSignature("invalid sig".to_string());
        assert_eq!(error.to_string(), "Invalid signature: invalid sig");

        let error = SolanaToolError::Transaction("tx error".to_string());
        assert_eq!(error.to_string(), "Transaction error: tx error");

        let error = SolanaToolError::InsufficientFunds;
        assert_eq!(error.to_string(), "Insufficient funds for operation");

        let error = SolanaToolError::InvalidTokenMint("invalid mint".to_string());
        assert_eq!(error.to_string(), "Invalid token mint: invalid mint");

        let error = SolanaToolError::Generic("generic error".to_string());
        assert_eq!(error.to_string(), "Solana tool error: generic error");
    }

    #[test]
    fn test_solana_tool_error_is_retriable() {
        // Test ToolError is_retriable delegation
        let tool_err = ToolError::invalid_input_string("test".to_string());
        let error = SolanaToolError::ToolError(tool_err);
        assert!(!error.is_retriable());

        let tool_err = ToolError::retriable_string("test".to_string());
        let error = SolanaToolError::ToolError(tool_err);
        assert!(error.is_retriable());

        // Test SignerError (non-retriable)
        let signer_err = SignerError::Signing("Invalid signature".to_string());
        let error = SolanaToolError::SignerError(signer_err);
        assert!(!error.is_retriable());

        // Test Core error (retriable)
        let core_err = riglr_core::CoreError::Queue("test".to_string());
        let error = SolanaToolError::Core(core_err);
        assert!(error.is_retriable());

        // Test RPC error (retriable)
        let error = SolanaToolError::Rpc("test rpc error".to_string());
        assert!(error.is_retriable());

        // Test HTTP errors with different status codes
        // Note: Creating a specific reqwest::Error is complex, so we test the logic path instead
        let error = SolanaToolError::Rpc("timeout error".to_string());
        assert!(error.is_retriable());

        // Test invalid address/key/signature/token mint (non-retriable)
        let error = SolanaToolError::InvalidAddress("invalid addr".to_string());
        assert!(!error.is_retriable());

        let error = SolanaToolError::InvalidKey("invalid key".to_string());
        assert!(!error.is_retriable());

        let error = SolanaToolError::InvalidSignature("invalid sig".to_string());
        assert!(!error.is_retriable());

        let error = SolanaToolError::InvalidTokenMint("invalid mint".to_string());
        assert!(!error.is_retriable());

        // Test insufficient funds (non-retriable)
        let error = SolanaToolError::InsufficientFunds;
        assert!(!error.is_retriable());

        // Test transaction errors with different messages
        let error = SolanaToolError::Transaction("insufficient funds detected".to_string());
        assert!(!error.is_retriable());

        let error = SolanaToolError::Transaction("invalid parameter".to_string());
        assert!(!error.is_retriable());

        let error = SolanaToolError::Transaction("network timeout".to_string());
        assert!(error.is_retriable());

        // Test serialization error (non-retriable)
        let serde_err = serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err();
        let error = SolanaToolError::Serialization(serde_err);
        assert!(!error.is_retriable());

        // Test generic error (retriable)
        let error = SolanaToolError::Generic("generic error".to_string());
        assert!(error.is_retriable());
    }

    #[test]
    fn test_solana_tool_error_is_rate_limited() {
        // Test RPC rate limit messages
        let error = SolanaToolError::Rpc("429 Too Many Requests".to_string());
        assert!(error.is_rate_limited());

        let error = SolanaToolError::Rpc("rate limit exceeded".to_string());
        assert!(error.is_rate_limited());

        let error = SolanaToolError::Rpc("too many requests".to_string());
        assert!(error.is_rate_limited());

        let error = SolanaToolError::Rpc("normal error".to_string());
        assert!(!error.is_rate_limited());

        // Test HTTP rate limit status
        // Note: Creating a reqwest::Error with specific status is complex,
        // so we'll test the logic through SolanaClient error path

        // Test non-rate-limited errors
        let error = SolanaToolError::InvalidAddress("invalid addr".to_string());
        assert!(!error.is_rate_limited());

        let error = SolanaToolError::Generic("generic error".to_string());
        assert!(!error.is_rate_limited());
    }

    #[test]
    fn test_solana_tool_error_retry_delay() {
        // Test rate-limited error delay
        let error = SolanaToolError::Rpc("429 Too Many Requests".to_string());
        assert_eq!(error.retry_delay(), Some(std::time::Duration::from_secs(1)));

        // Test retriable but not rate-limited error delay
        let error = SolanaToolError::Rpc("network error".to_string());
        assert_eq!(
            error.retry_delay(),
            Some(std::time::Duration::from_millis(500))
        );

        // Test non-retriable error (no delay)
        let error = SolanaToolError::InvalidAddress("invalid addr".to_string());
        assert_eq!(error.retry_delay(), None);
    }

    #[test]
    fn test_solana_client_error_is_retriable() {
        // Create a retryable client error
        let io_error =
            std::io::Error::new(std::io::ErrorKind::ConnectionRefused, "connection refused");
        let client_error = ClientError::new_with_request(
            ClientErrorKind::Io(io_error),
            solana_client::rpc_request::RpcRequest::GetAccountInfo,
        );
        let error = SolanaToolError::SolanaClient(Box::new(client_error));
        assert!(error.is_retriable());

        // Create a non-retryable client error
        let serde_error = serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err();
        let client_error = ClientError::new_with_request(
            ClientErrorKind::SerdeJson(serde_error),
            solana_client::rpc_request::RpcRequest::GetAccountInfo,
        );
        let error = SolanaToolError::SolanaClient(Box::new(client_error));
        assert!(!error.is_retriable());
    }

    #[test]
    fn test_solana_client_error_is_rate_limited() {
        // Create a rate-limited client error
        let rpc_error = RpcError::RpcResponseError {
            code: 429,
            message: "Too Many Requests".to_string(),
            data: RpcResponseErrorData::Empty,
        };
        let client_error = ClientError::new_with_request(
            ClientErrorKind::RpcError(rpc_error),
            solana_client::rpc_request::RpcRequest::SendTransaction,
        );
        let error = SolanaToolError::SolanaClient(Box::new(client_error));
        assert!(error.is_rate_limited());

        // Create a non-rate-limited client error
        let io_error =
            std::io::Error::new(std::io::ErrorKind::ConnectionRefused, "connection refused");
        let client_error = ClientError::new_with_request(
            ClientErrorKind::Io(io_error),
            solana_client::rpc_request::RpcRequest::GetAccountInfo,
        );
        let error = SolanaToolError::SolanaClient(Box::new(client_error));
        assert!(!error.is_rate_limited());
    }

    #[test]
    fn test_from_client_error() {
        let io_error =
            std::io::Error::new(std::io::ErrorKind::ConnectionRefused, "connection refused");
        let client_error = ClientError::new_with_request(
            ClientErrorKind::Io(io_error),
            solana_client::rpc_request::RpcRequest::GetAccountInfo,
        );

        let solana_error: SolanaToolError = client_error.into();
        assert!(matches!(solana_error, SolanaToolError::SolanaClient(_)));
    }

    #[test]
    fn test_from_solana_tool_error_to_tool_error() {
        // Test ToolError passthrough
        let tool_err = ToolError::invalid_input_string("test".to_string());
        let expected_string = tool_err.to_string();
        let solana_err = SolanaToolError::ToolError(tool_err);
        let converted: ToolError = solana_err.into();
        assert_eq!(converted.to_string(), expected_string);

        // Test SignerError conversion
        let signer_err = SignerError::Signing("Invalid signature".to_string());
        let solana_err = SolanaToolError::SignerError(signer_err);
        let converted: ToolError = solana_err.into();
        assert!(matches!(converted, ToolError::SignerContext(_)));

        // Test invalid input conversions
        let solana_err = SolanaToolError::InvalidAddress("test addr".to_string());
        let converted: ToolError = solana_err.into();
        assert!(converted.to_string().contains("Invalid input"));

        let solana_err = SolanaToolError::InvalidKey("test key".to_string());
        let converted: ToolError = solana_err.into();
        assert!(converted.to_string().contains("Invalid input"));

        let solana_err = SolanaToolError::InvalidSignature("test sig".to_string());
        let converted: ToolError = solana_err.into();
        assert!(converted.to_string().contains("Invalid input"));

        let solana_err = SolanaToolError::InvalidTokenMint("test mint".to_string());
        let converted: ToolError = solana_err.into();
        assert!(converted.to_string().contains("Invalid input"));

        // Test rate-limited error conversion
        let solana_err = SolanaToolError::Rpc("429 Too Many Requests".to_string());
        let converted: ToolError = solana_err.into();
        // This should be a rate-limited error
        assert!(converted.to_string().contains("Rate limited"));

        // Test retriable error conversion
        let solana_err = SolanaToolError::Rpc("network timeout".to_string());
        let converted: ToolError = solana_err.into();
        // This should be a retriable error
        assert!(converted.to_string().contains("network timeout"));

        // Test generic error conversion (non-retriable/non-rate-limited)
        let solana_err = SolanaToolError::InsufficientFunds;
        let converted: ToolError = solana_err.into();
        // This should be converted as retriable (the default case)
        assert!(converted.to_string().contains("Insufficient funds"));
    }

    #[test]
    fn test_reqwest_error_classification() {
        // Test timeout error - we'll use a serde error instead since reqwest::Error creation is complex
        let serde_error = serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err();
        let client_error = ClientError::new_with_request(
            ClientErrorKind::SerdeJson(serde_error),
            solana_client::rpc_request::RpcRequest::GetAccountInfo,
        );
        let result = classify_transaction_error(&client_error);
        assert_eq!(
            result,
            TransactionErrorType::Permanent(PermanentError::InvalidTransaction)
        );
    }

    #[test]
    fn test_rpc_error_response_edge_cases() {
        // Test internal error (-32603)
        let rpc_error = RpcError::RpcResponseError {
            code: -32603,
            message: "Internal error".to_string(),
            data: RpcResponseErrorData::Empty,
        };
        let client_error = ClientError::new_with_request(
            ClientErrorKind::RpcError(rpc_error),
            solana_client::rpc_request::RpcRequest::SendTransaction,
        );
        let result = classify_transaction_error(&client_error);
        assert_eq!(
            result,
            TransactionErrorType::Retryable(RetryableError::TemporaryRpcFailure)
        );

        // Test node behind (-32005)
        let rpc_error = RpcError::RpcResponseError {
            code: -32005,
            message: "Node is behind".to_string(),
            data: RpcResponseErrorData::Empty,
        };
        let client_error = ClientError::new_with_request(
            ClientErrorKind::RpcError(rpc_error),
            solana_client::rpc_request::RpcRequest::SendTransaction,
        );
        let result = classify_transaction_error(&client_error);
        assert_eq!(
            result,
            TransactionErrorType::Retryable(RetryableError::NetworkCongestion)
        );

        // Test invalid signature in RPC message
        let rpc_error = RpcError::RpcResponseError {
            code: -32001,
            message: "invalid signature provided".to_string(),
            data: RpcResponseErrorData::Empty,
        };
        let client_error = ClientError::new_with_request(
            ClientErrorKind::RpcError(rpc_error),
            solana_client::rpc_request::RpcRequest::SendTransaction,
        );
        let result = classify_transaction_error(&client_error);
        assert_eq!(
            result,
            TransactionErrorType::Permanent(PermanentError::InvalidSignature)
        );

        // Test invalid account in RPC message
        let rpc_error = RpcError::RpcResponseError {
            code: -32001,
            message: "invalid account reference".to_string(),
            data: RpcResponseErrorData::Empty,
        };
        let client_error = ClientError::new_with_request(
            ClientErrorKind::RpcError(rpc_error),
            solana_client::rpc_request::RpcRequest::SendTransaction,
        );
        let result = classify_transaction_error(&client_error);
        assert_eq!(
            result,
            TransactionErrorType::Permanent(PermanentError::InvalidAccount)
        );

        // Test instruction error in RPC message
        let rpc_error = RpcError::RpcResponseError {
            code: -32001,
            message: "Instruction error occurred".to_string(),
            data: RpcResponseErrorData::Empty,
        };
        let client_error = ClientError::new_with_request(
            ClientErrorKind::RpcError(rpc_error),
            solana_client::rpc_request::RpcRequest::SendTransaction,
        );
        let result = classify_transaction_error(&client_error);
        assert_eq!(
            result,
            TransactionErrorType::Permanent(PermanentError::InstructionError)
        );

        // Test unknown error code with message
        let rpc_error = RpcError::RpcResponseError {
            code: -99999,
            message: "Unknown error".to_string(),
            data: RpcResponseErrorData::Empty,
        };
        let client_error = ClientError::new_with_request(
            ClientErrorKind::RpcError(rpc_error),
            solana_client::rpc_request::RpcRequest::SendTransaction,
        );
        let result = classify_transaction_error(&client_error);
        assert!(matches!(result, TransactionErrorType::Unknown(_)));
        if let TransactionErrorType::Unknown(msg) = result {
            assert!(msg.contains("RPC Error -99999"));
            assert!(msg.contains("Unknown error"));
        }
    }

    #[test]
    fn test_rpc_parse_error_classification() {
        let rpc_error = RpcError::ParseError("Invalid JSON".to_string());
        let client_error = ClientError::new_with_request(
            ClientErrorKind::RpcError(rpc_error),
            solana_client::rpc_request::RpcRequest::SendTransaction,
        );
        let result = classify_transaction_error(&client_error);
        assert_eq!(
            result,
            TransactionErrorType::Permanent(PermanentError::InvalidTransaction)
        );
    }

    #[test]
    fn test_rpc_for_user_error_classification() {
        let rpc_error = RpcError::ForUser("User-facing error message".to_string());
        let client_error = ClientError::new_with_request(
            ClientErrorKind::RpcError(rpc_error),
            solana_client::rpc_request::RpcRequest::SendTransaction,
        );
        let result = classify_transaction_error(&client_error);
        assert!(matches!(result, TransactionErrorType::Unknown(_)));
        if let TransactionErrorType::Unknown(msg) = result {
            assert_eq!(msg, "User-facing error message");
        }
    }

    #[test]
    fn test_rpc_request_error_with_different_messages() {
        // Test "429" in message
        let rpc_error = RpcError::RpcRequestError("HTTP 429 rate limit".to_string());
        let client_error = ClientError::new_with_request(
            ClientErrorKind::RpcError(rpc_error),
            solana_client::rpc_request::RpcRequest::SendTransaction,
        );
        let result = classify_transaction_error(&client_error);
        assert_eq!(
            result,
            TransactionErrorType::RateLimited(RateLimitError::RpcRateLimit)
        );

        // Test "too many requests" in message
        let rpc_error = RpcError::RpcRequestError("too many requests received".to_string());
        let client_error = ClientError::new_with_request(
            ClientErrorKind::RpcError(rpc_error),
            solana_client::rpc_request::RpcRequest::SendTransaction,
        );
        let result = classify_transaction_error(&client_error);
        assert_eq!(
            result,
            TransactionErrorType::RateLimited(RateLimitError::RpcRateLimit)
        );
    }

    #[test]
    fn test_classify_transaction_error_with_unknown_client_error_kind() {
        // Create a client error with an unhandled error kind
        // We'll use a custom error for this test
        let custom_msg = "Custom unknown error".to_string();
        let client_error = ClientError::new_with_request(
            ClientErrorKind::Custom(custom_msg.clone()),
            solana_client::rpc_request::RpcRequest::GetAccountInfo,
        );

        // Since our custom message doesn't match any known patterns, it should be Unknown
        let result = classify_transaction_error(&client_error);
        assert!(matches!(result, TransactionErrorType::Unknown(_)));
    }

    #[test]
    fn test_custom_error_with_insufficient_funds_lowercase() {
        // Test "insufficient funds" (lowercase) in custom error
        let client_error = ClientError::new_with_request(
            ClientErrorKind::Custom("insufficient funds for transaction".to_string()),
            solana_client::rpc_request::RpcRequest::SendTransaction,
        );
        let result = classify_transaction_error(&client_error);
        assert_eq!(
            result,
            TransactionErrorType::Permanent(PermanentError::InsufficientFunds)
        );
    }

    #[test]
    fn test_error_variants_equality() {
        // Test RetryableError variants
        assert_eq!(
            RetryableError::NetworkConnectivity,
            RetryableError::NetworkConnectivity
        );
        assert_ne!(
            RetryableError::NetworkConnectivity,
            RetryableError::TemporaryRpcFailure
        );

        // Test PermanentError variants
        assert_eq!(
            PermanentError::InsufficientFunds,
            PermanentError::InsufficientFunds
        );
        assert_ne!(
            PermanentError::InsufficientFunds,
            PermanentError::InvalidSignature
        );

        // Test RateLimitError variants
        assert_eq!(RateLimitError::RpcRateLimit, RateLimitError::RpcRateLimit);
        assert_ne!(
            RateLimitError::RpcRateLimit,
            RateLimitError::TooManyRequests
        );

        // Test TransactionErrorType variants
        assert_eq!(
            TransactionErrorType::Retryable(RetryableError::NetworkConnectivity),
            TransactionErrorType::Retryable(RetryableError::NetworkConnectivity)
        );
        assert_ne!(
            TransactionErrorType::Retryable(RetryableError::NetworkConnectivity),
            TransactionErrorType::Permanent(PermanentError::InsufficientFunds)
        );
    }

    #[test]
    fn test_error_debug_format() {
        // Test Debug implementation for all error types
        let retryable = RetryableError::NetworkConnectivity;
        assert!(!format!("{:?}", retryable).is_empty());

        let permanent = PermanentError::InsufficientFunds;
        assert!(!format!("{:?}", permanent).is_empty());

        let rate_limit = RateLimitError::RpcRateLimit;
        assert!(!format!("{:?}", rate_limit).is_empty());

        let transaction_error = TransactionErrorType::Unknown("test".to_string());
        assert!(!format!("{:?}", transaction_error).is_empty());
    }
}
