//! Comprehensive unit tests for error classification system
//!
//! These tests focus on the sophisticated error classification logic
//! introduced to handle Solana client errors intelligently.

use riglr_solana_tools::error::{
    classify_transaction_error, PermanentError, RateLimitError, RetryableError, TransactionErrorType,
};
use solana_client::client_error::{ClientError, ClientErrorKind};
use solana_client::rpc_request::{RpcError, RpcRequest, RpcResponseErrorData};

#[test]
fn test_transaction_error_type_classification() {
    // Test all retryable error variants
    let retryable_network = TransactionErrorType::Retryable(RetryableError::NetworkConnectivity);
    assert!(retryable_network.is_retryable());
    assert!(!retryable_network.is_rate_limited());

    let retryable_rpc = TransactionErrorType::Retryable(RetryableError::TemporaryRpcFailure);
    assert!(retryable_rpc.is_retryable());
    assert!(!retryable_rpc.is_rate_limited());

    let retryable_congestion = TransactionErrorType::Retryable(RetryableError::NetworkCongestion);
    assert!(retryable_congestion.is_retryable());
    assert!(!retryable_congestion.is_rate_limited());

    let retryable_pool = TransactionErrorType::Retryable(RetryableError::TransactionPoolFull);
    assert!(retryable_pool.is_retryable());
    assert!(!retryable_pool.is_rate_limited());

    // Test all permanent error variants
    let permanent_funds = TransactionErrorType::Permanent(PermanentError::InsufficientFunds);
    assert!(!permanent_funds.is_retryable());
    assert!(!permanent_funds.is_rate_limited());

    let permanent_sig = TransactionErrorType::Permanent(PermanentError::InvalidSignature);
    assert!(!permanent_sig.is_retryable());
    assert!(!permanent_sig.is_rate_limited());

    let permanent_account = TransactionErrorType::Permanent(PermanentError::InvalidAccount);
    assert!(!permanent_account.is_retryable());
    assert!(!permanent_account.is_rate_limited());

    let permanent_instruction = TransactionErrorType::Permanent(PermanentError::InstructionError);
    assert!(!permanent_instruction.is_retryable());
    assert!(!permanent_instruction.is_rate_limited());

    let permanent_tx = TransactionErrorType::Permanent(PermanentError::InvalidTransaction);
    assert!(!permanent_tx.is_retryable());
    assert!(!permanent_tx.is_rate_limited());

    let permanent_dup = TransactionErrorType::Permanent(PermanentError::DuplicateTransaction);
    assert!(!permanent_dup.is_retryable());
    assert!(!permanent_dup.is_rate_limited());

    // Test all rate limited error variants
    let rate_rpc = TransactionErrorType::RateLimited(RateLimitError::RpcRateLimit);
    assert!(rate_rpc.is_retryable());
    assert!(rate_rpc.is_rate_limited());

    let rate_many = TransactionErrorType::RateLimited(RateLimitError::TooManyRequests);
    assert!(rate_many.is_retryable());
    assert!(rate_many.is_rate_limited());

    // Test unknown error
    let unknown = TransactionErrorType::Unknown("Unknown error".to_string());
    assert!(!unknown.is_retryable());
    assert!(!unknown.is_rate_limited());
}

#[test]
fn test_io_error_classification_comprehensive() {
    let io_errors = vec![
        std::io::ErrorKind::ConnectionRefused,
        std::io::ErrorKind::ConnectionAborted,
        std::io::ErrorKind::NotConnected,
        std::io::ErrorKind::TimedOut,
        std::io::ErrorKind::Interrupted,
        std::io::ErrorKind::UnexpectedEof,
        std::io::ErrorKind::BrokenPipe,
    ];

    for error_kind in io_errors {
        let io_error = std::io::Error::new(error_kind, format!("IO error: {:?}", error_kind));
        let client_error = ClientError::new_with_request(
            ClientErrorKind::Io(io_error),
            RpcRequest::GetAccountInfo,
        );

        let result = classify_transaction_error(&client_error);
        assert_eq!(result, TransactionErrorType::Retryable(RetryableError::NetworkConnectivity));
    }
}

#[test]
fn test_serde_json_error_classification() {
    // Test different types of serde errors
    let invalid_jsons = vec![
        "{ invalid json",
        "{ \"key\": }",
        "[ 1, 2, 3,",
        "null null",
        "{ \"key\": undefined }",
    ];

    for invalid_json in invalid_jsons {
        let serde_error: serde_json::Error = serde_json::from_str::<serde_json::Value>(invalid_json).unwrap_err();
        let client_error = ClientError::new_with_request(
            ClientErrorKind::SerdeJson(serde_error),
            RpcRequest::SendTransaction,
        );

        let result = classify_transaction_error(&client_error);
        assert_eq!(result, TransactionErrorType::Permanent(PermanentError::InvalidTransaction));
    }
}

#[test]
fn test_reqwest_error_classification_comprehensive() {
    // Test timeout errors
    let timeout_error = reqwest::Error::from(reqwest::ErrorKind::Timeout);
    let client_error = ClientError::new_with_request(
        ClientErrorKind::Reqwest(timeout_error),
        RpcRequest::SendTransaction,
    );
    let result = classify_transaction_error(&client_error);
    assert_eq!(result, TransactionErrorType::Retryable(RetryableError::NetworkConnectivity));

    // Note: Creating specific reqwest errors with status codes is complex due to 
    // reqwest's internal structure. In real scenarios, these would be properly classified.
}

#[test]
fn test_custom_error_message_classification() {
    let test_cases = vec![
        // Insufficient funds variations
        ("InsufficientFundsForRent", TransactionErrorType::Permanent(PermanentError::InsufficientFunds)),
        ("insufficient funds", TransactionErrorType::Permanent(PermanentError::InsufficientFunds)),
        ("Insufficient balance for rent exemption", TransactionErrorType::Permanent(PermanentError::InsufficientFunds)),
        
        // Invalid account variations  
        ("InvalidAccountIndex", TransactionErrorType::Permanent(PermanentError::InvalidAccount)),
        ("Invalid account data", TransactionErrorType::Permanent(PermanentError::InvalidAccount)),
        
        // Invalid signature variations
        ("InvalidSignature", TransactionErrorType::Permanent(PermanentError::InvalidSignature)),
        ("Signature verification failed", TransactionErrorType::Permanent(PermanentError::InvalidSignature)),
        
        // Duplicate transaction variations
        ("DuplicateSignature", TransactionErrorType::Permanent(PermanentError::DuplicateTransaction)),
        ("Transaction already processed", TransactionErrorType::Permanent(PermanentError::DuplicateTransaction)),
        
        // Unknown errors
        ("Some random error", TransactionErrorType::Unknown("Some random error".to_string())),
        ("Network error of unknown type", TransactionErrorType::Unknown("Network error of unknown type".to_string())),
    ];

    for (error_message, expected_type) in test_cases {
        let client_error = ClientError::new_with_request(
            ClientErrorKind::Custom(error_message.to_string()),
            RpcRequest::SendTransaction,
        );

        let result = classify_transaction_error(&client_error);
        assert_eq!(result, expected_type, "Failed for error message: {}", error_message);
    }
}

#[test]
fn test_rpc_request_error_classification() {
    // Test rate limiting detection in RpcRequestError
    let rate_limit_messages = vec![
        "rate limit exceeded",
        "too many requests",
        "429 - Too Many Requests",
        "Request rate limit exceeded",
        "API rate limit reached",
    ];

    for message in rate_limit_messages {
        let rpc_error = RpcError::RpcRequestError(message.to_string());
        let client_error = ClientError::new_with_request(
            ClientErrorKind::RpcError(rpc_error),
            RpcRequest::SendTransaction,
        );

        let result = classify_transaction_error(&client_error);
        assert_eq!(result, TransactionErrorType::RateLimited(RateLimitError::RpcRateLimit));
    }

    // Test non-rate-limit RpcRequestError
    let general_messages = vec![
        "Connection failed",
        "Request timeout",
        "Server error",
        "Network unreachable",
    ];

    for message in general_messages {
        let rpc_error = RpcError::RpcRequestError(message.to_string());
        let client_error = ClientError::new_with_request(
            ClientErrorKind::RpcError(rpc_error),
            RpcRequest::SendTransaction,
        );

        let result = classify_transaction_error(&client_error);
        assert_eq!(result, TransactionErrorType::Retryable(RetryableError::TemporaryRpcFailure));
    }
}

#[test]
fn test_rpc_response_error_standard_codes() {
    let test_cases = vec![
        // Rate limiting
        (429, "Too Many Requests", TransactionErrorType::RateLimited(RateLimitError::RpcRateLimit)),
        
        // Internal error (retriable)
        (-32603, "Internal error", TransactionErrorType::Retryable(RetryableError::TemporaryRpcFailure)),
        
        // Transaction pool full (network congestion)
        (-32002, "Transaction pool is full", TransactionErrorType::Retryable(RetryableError::NetworkCongestion)),
        
        // Node behind (network congestion)
        (-32005, "Node is behind by X slots", TransactionErrorType::Retryable(RetryableError::NetworkCongestion)),
        
        // Parse error
        (-32700, "Parse error", TransactionErrorType::Unknown("RPC Error -32700: Parse error".to_string())),
        
        // Method not found
        (-32601, "Method not found", TransactionErrorType::Unknown("RPC Error -32601: Method not found".to_string())),
    ];

    for (code, message, expected_type) in test_cases {
        let rpc_error = RpcError::RpcResponseError {
            code,
            message: message.to_string(),
            data: RpcResponseErrorData::Empty,
        };
        let client_error = ClientError::new_with_request(
            ClientErrorKind::RpcError(rpc_error),
            RpcRequest::SendTransaction,
        );

        let result = classify_transaction_error(&client_error);
        assert_eq!(result, expected_type, "Failed for RPC code: {}", code);
    }
}

#[test]
fn test_rpc_response_error_message_analysis() {
    // Test message-based classification for unknown error codes
    let message_cases = vec![
        (-32600, "InsufficientFundsForRent", TransactionErrorType::Permanent(PermanentError::InsufficientFunds)),
        (-32600, "invalid signature provided", TransactionErrorType::Permanent(PermanentError::InvalidSignature)),
        (-32600, "invalid account reference", TransactionErrorType::Permanent(PermanentError::InvalidAccount)),
        (-32600, "Instruction error occurred", TransactionErrorType::Permanent(PermanentError::InstructionError)),
        (-32600, "Some other error", TransactionErrorType::Unknown("RPC Error -32600: Some other error".to_string())),
    ];

    for (code, message, expected_type) in message_cases {
        let rpc_error = RpcError::RpcResponseError {
            code,
            message: message.to_string(),
            data: RpcResponseErrorData::Empty,
        };
        let client_error = ClientError::new_with_request(
            ClientErrorKind::RpcError(rpc_error),
            RpcRequest::SendTransaction,
        );

        let result = classify_transaction_error(&client_error);
        assert_eq!(result, expected_type, "Failed for message: {}", message);
    }
}

#[test]
fn test_rpc_parse_error_classification() {
    let rpc_error = RpcError::ParseError("Failed to parse JSON response".to_string());
    let client_error = ClientError::new_with_request(
        ClientErrorKind::RpcError(rpc_error),
        RpcRequest::SendTransaction,
    );

    let result = classify_transaction_error(&client_error);
    assert_eq!(result, TransactionErrorType::Permanent(PermanentError::InvalidTransaction));
}

#[test]
fn test_rpc_for_user_error_classification() {
    let user_messages = vec![
        "User-facing error message",
        "Transaction failed for user",
        "Something went wrong",
    ];

    for message in user_messages {
        let rpc_error = RpcError::ForUser(message.to_string());
        let client_error = ClientError::new_with_request(
            ClientErrorKind::RpcError(rpc_error),
            RpcRequest::SendTransaction,
        );

        let result = classify_transaction_error(&client_error);
        assert_eq!(result, TransactionErrorType::Unknown(message.to_string()));
    }
}

#[test]
fn test_unknown_client_error_kinds() {
    // Test ClientErrorKind variants that fall through to Unknown
    
    // Note: We can't easily create all ClientErrorKind variants due to their 
    // internal structure, but we can test the fallback behavior with a custom message
    let client_error = ClientError::new_with_request(
        ClientErrorKind::Custom("Completely unknown error type".to_string()),
        RpcRequest::GetAccountInfo,
    );

    let result = classify_transaction_error(&client_error);
    match result {
        TransactionErrorType::Unknown(msg) => {
            assert!(msg.contains("Completely unknown error type"));
        }
        _ => panic!("Expected Unknown classification for unrecognized error"),
    }
}

#[test]
fn test_error_type_equality() {
    // Test PartialEq implementation for all enum variants
    assert_eq!(
        RetryableError::NetworkConnectivity,
        RetryableError::NetworkConnectivity
    );
    assert_ne!(
        RetryableError::NetworkConnectivity,
        RetryableError::TemporaryRpcFailure
    );

    assert_eq!(
        PermanentError::InsufficientFunds,
        PermanentError::InsufficientFunds
    );
    assert_ne!(
        PermanentError::InsufficientFunds,
        PermanentError::InvalidSignature
    );

    assert_eq!(
        RateLimitError::RpcRateLimit,
        RateLimitError::RpcRateLimit
    );
    assert_ne!(
        RateLimitError::RpcRateLimit,
        RateLimitError::TooManyRequests
    );

    let error1 = TransactionErrorType::Unknown("test".to_string());
    let error2 = TransactionErrorType::Unknown("test".to_string());
    let error3 = TransactionErrorType::Unknown("different".to_string());
    assert_eq!(error1, error2);
    assert_ne!(error1, error3);
}

#[test]
fn test_error_type_clone_and_debug() {
    let retryable = RetryableError::NetworkConnectivity;
    let cloned = retryable.clone();
    assert_eq!(retryable, cloned);

    let permanent = PermanentError::InsufficientFunds;
    let cloned = permanent.clone();
    assert_eq!(permanent, cloned);

    let rate_limited = RateLimitError::RpcRateLimit;
    let cloned = rate_limited.clone();
    assert_eq!(rate_limited, cloned);

    let tx_error = TransactionErrorType::Retryable(RetryableError::NetworkConnectivity);
    let cloned = tx_error.clone();
    assert_eq!(tx_error, cloned);

    // Test Debug formatting
    let debug_str = format!("{:?}", tx_error);
    assert!(debug_str.contains("Retryable"));
    assert!(debug_str.contains("NetworkConnectivity"));
}

#[test]
fn test_comprehensive_error_classification_edge_cases() {
    // Test empty error messages
    let client_error = ClientError::new_with_request(
        ClientErrorKind::Custom("".to_string()),
        RpcRequest::SendTransaction,
    );
    let result = classify_transaction_error(&client_error);
    match result {
        TransactionErrorType::Unknown(_) => {}, // Expected
        _ => panic!("Expected Unknown for empty error message"),
    }

    // Test very long error messages
    let long_message = "x".repeat(10000);
    let client_error = ClientError::new_with_request(
        ClientErrorKind::Custom(long_message.clone()),
        RpcRequest::SendTransaction,
    );
    let result = classify_transaction_error(&client_error);
    match result {
        TransactionErrorType::Unknown(msg) => {
            assert_eq!(msg, long_message);
        }
        _ => panic!("Expected Unknown for long error message"),
    }

    // Test case sensitivity
    let case_variations = vec![
        "insufficientfundsforrent",
        "INSUFFICIENTFUNDSFORRENT", 
        "InsufficientFundsForRent",
        "insufficient_funds_for_rent",
    ];

    for variation in case_variations {
        let client_error = ClientError::new_with_request(
            ClientErrorKind::Custom(variation.to_string()),
            RpcRequest::SendTransaction,
        );
        let result = classify_transaction_error(&client_error);
        // Only exact case match should work
        if variation == "InsufficientFundsForRent" || variation.contains("insufficient funds") {
            assert_eq!(result, TransactionErrorType::Permanent(PermanentError::InsufficientFunds));
        } else {
            match result {
                TransactionErrorType::Unknown(_) => {}, // Expected for non-matching cases
                _ => panic!("Unexpected classification for case variation: {}", variation),
            }
        }
    }
}