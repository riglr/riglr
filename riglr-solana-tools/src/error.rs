//! Error types for riglr-solana-tools.

use thiserror::Error;
use riglr_core::error::{ToolError, SignerError};
use solana_client::client_error::{ClientError, ClientErrorKind};
use solana_client::rpc_request::RpcError;

/// Main error type for Solana tool operations.
#[derive(Error, Debug)]
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
    SolanaClient(#[from] ClientError),

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

/// Structured classification of transaction errors for intelligent retry logic
#[derive(Debug, Clone, PartialEq)]
pub enum TransactionErrorType {
    Retryable(RetryableError),
    Permanent(PermanentError),
    RateLimited(RateLimitError),
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
        matches!(self, TransactionErrorType::Retryable(_) | TransactionErrorType::RateLimited(_))
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
    match &error.kind {
        ClientErrorKind::RpcError(rpc_error) => {
            classify_rpc_error(rpc_error)
        },
        ClientErrorKind::SerdeJson(_) => {
            TransactionErrorType::Permanent(PermanentError::InvalidTransaction)
        },
        ClientErrorKind::Io(_) => {
            TransactionErrorType::Retryable(RetryableError::NetworkConnectivity)
        },
        ClientErrorKind::Reqwest(reqwest_error) => {
            if reqwest_error.status() == Some(reqwest::StatusCode::TOO_MANY_REQUESTS) {
                TransactionErrorType::RateLimited(RateLimitError::TooManyRequests)
            } else if reqwest_error.is_timeout() || reqwest_error.is_connect() {
                TransactionErrorType::Retryable(RetryableError::NetworkConnectivity)
            } else {
                TransactionErrorType::Unknown(error.to_string())
            }
        },
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
        },
        _ => TransactionErrorType::Unknown(error.to_string()),
    }
}

/// Classify RPC-specific errors
fn classify_rpc_error(rpc_error: &RpcError) -> TransactionErrorType {
    use solana_client::rpc_request::RpcError::*;
    
    match rpc_error {
        RpcRequestError(msg) => {
            if msg.contains("rate limit") || msg.contains("429") || msg.contains("too many requests") {
                TransactionErrorType::RateLimited(RateLimitError::RpcRateLimit)
            } else {
                TransactionErrorType::Retryable(RetryableError::TemporaryRpcFailure)
            }
        },
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
        },
        ParseError(_msg) => TransactionErrorType::Permanent(PermanentError::InvalidTransaction),
        ForUser(msg) => TransactionErrorType::Unknown(msg.clone()),
    }
}

// Implement From conversion to riglr_core::ToolError for proper error handling
impl From<SolanaToolError> for ToolError {
    fn from(err: SolanaToolError) -> Self {
        match err {
            SolanaToolError::ToolError(tool_err) => tool_err,
            SolanaToolError::SignerError(signer_err) => ToolError::SignerContext(signer_err),
            
            // RPC and HTTP errors are often retriable
            SolanaToolError::Rpc(msg) => {
                if msg.contains("429") || msg.contains("rate limit") || msg.contains("too many requests") {
                    ToolError::rate_limited(msg)
                } else {
                    ToolError::retriable(msg)
                }
            }
            
            SolanaToolError::SolanaClient(ref client_err) => {
                let error_type = classify_transaction_error(client_err);
                match error_type {
                    TransactionErrorType::Retryable(_) => ToolError::retriable(err.to_string()),
                    TransactionErrorType::RateLimited(_) => ToolError::rate_limited(err.to_string()),
                    TransactionErrorType::Permanent(_) => ToolError::permanent(err.to_string()),
                    TransactionErrorType::Unknown(_) => ToolError::retriable(err.to_string()),
                }
            }
            
            SolanaToolError::Http(ref http_err) => {
                if http_err.status() == Some(reqwest::StatusCode::TOO_MANY_REQUESTS) {
                    ToolError::rate_limited(err.to_string())
                } else if http_err.is_timeout() || http_err.is_connect() {
                    ToolError::retriable(err.to_string())
                } else {
                    ToolError::permanent(err.to_string())
                }
            }

            // Address and key validation errors are permanent
            SolanaToolError::InvalidAddress(_) => ToolError::invalid_input(err.to_string()),
            SolanaToolError::InvalidKey(_) => ToolError::invalid_input(err.to_string()),
            SolanaToolError::InvalidSignature(_) => ToolError::invalid_input(err.to_string()),
            SolanaToolError::InvalidTokenMint(_) => ToolError::invalid_input(err.to_string()),
            
            // Insufficient funds is permanent
            SolanaToolError::InsufficientFunds => ToolError::permanent(err.to_string()),
            
            // Transaction errors could be retriable if they're network related
            SolanaToolError::Transaction(msg) => {
                if msg.contains("insufficient funds") || msg.contains("invalid") {
                    ToolError::permanent(msg)
                } else {
                    ToolError::retriable(msg)
                }
            }
            
            // Serialization errors are permanent
            SolanaToolError::Serialization(_) => ToolError::permanent(err.to_string()),
            
            // Core errors inherit their retriable nature
            SolanaToolError::Core(_) => ToolError::retriable(err.to_string()),
            
            // Generic errors default to retriable
            SolanaToolError::Generic(msg) => ToolError::retriable(msg),
        }
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
        let io_error = std::io::Error::new(std::io::ErrorKind::ConnectionRefused, "connection refused");
        let client_error = ClientError::new_with_request(
            ClientErrorKind::Io(io_error),
            solana_client::rpc_request::RpcRequest::GetAccountInfo,
        );

        let result = classify_transaction_error(&client_error);
        assert_eq!(result, TransactionErrorType::Retryable(RetryableError::NetworkConnectivity));
    }

    #[test]
    fn test_serde_error_classification() {
        // Create a serde error by trying to parse invalid JSON
        let serde_error: serde_json::Error = serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err();
        let client_error = ClientError::new_with_request(
            ClientErrorKind::SerdeJson(serde_error),
            solana_client::rpc_request::RpcRequest::GetAccountInfo,
        );

        let result = classify_transaction_error(&client_error);
        assert_eq!(result, TransactionErrorType::Permanent(PermanentError::InvalidTransaction));
    }

    #[test]
    fn test_custom_error_classification() {
        // Test insufficient funds
        let client_error = ClientError::new_with_request(
            ClientErrorKind::Custom("InsufficientFundsForRent".to_string()),
            solana_client::rpc_request::RpcRequest::SendTransaction,
        );
        let result = classify_transaction_error(&client_error);
        assert_eq!(result, TransactionErrorType::Permanent(PermanentError::InsufficientFunds));

        // Test invalid signature
        let client_error = ClientError::new_with_request(
            ClientErrorKind::Custom("InvalidSignature".to_string()),
            solana_client::rpc_request::RpcRequest::SendTransaction,
        );
        let result = classify_transaction_error(&client_error);
        assert_eq!(result, TransactionErrorType::Permanent(PermanentError::InvalidSignature));

        // Test invalid account
        let client_error = ClientError::new_with_request(
            ClientErrorKind::Custom("InvalidAccountIndex".to_string()),
            solana_client::rpc_request::RpcRequest::SendTransaction,
        );
        let result = classify_transaction_error(&client_error);
        assert_eq!(result, TransactionErrorType::Permanent(PermanentError::InvalidAccount));

        // Test duplicate signature
        let client_error = ClientError::new_with_request(
            ClientErrorKind::Custom("DuplicateSignature".to_string()),
            solana_client::rpc_request::RpcRequest::SendTransaction,
        );
        let result = classify_transaction_error(&client_error);
        assert_eq!(result, TransactionErrorType::Permanent(PermanentError::DuplicateTransaction));
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
        assert_eq!(result, TransactionErrorType::RateLimited(RateLimitError::RpcRateLimit));

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
        assert_eq!(result, TransactionErrorType::Retryable(RetryableError::NetworkCongestion));

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
        assert_eq!(result, TransactionErrorType::Permanent(PermanentError::InsufficientFunds));
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
        assert_eq!(result, TransactionErrorType::RateLimited(RateLimitError::RpcRateLimit));

        // Test other RPC request error
        let rpc_error = RpcError::RpcRequestError("network timeout".to_string());
        let client_error = ClientError::new_with_request(
            ClientErrorKind::RpcError(rpc_error),
            solana_client::rpc_request::RpcRequest::SendTransaction,
        );
        let result = classify_transaction_error(&client_error);
        assert_eq!(result, TransactionErrorType::Retryable(RetryableError::TemporaryRpcFailure));
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
}
