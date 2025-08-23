//! Tests for function compatibility with Arc<ClientError>
//!
//! These tests verify that functions like classify_transaction_error() work
//! correctly with Arc<ClientError> after dereferencing, and that all error
//! classification logic still functions properly.

use riglr_core::signer::SignerError;
use solana_client::client_error::{ClientError, ClientErrorKind};
use solana_client::rpc_request::{RpcError, RpcRequest, RpcResponseErrorData};
use std::sync::Arc;

// Note: We need to simulate the classify_transaction_error function since it's in riglr-solana-tools
// For testing purposes, we'll create a simplified version that demonstrates the Arc dereferencing

/// Simplified error classification to test Arc dereferencing
#[derive(Debug, PartialEq)]
enum TestErrorType {
    Retryable,
    Permanent,
    RateLimited,
    Unknown,
}

/// Simplified classify function that takes &ClientError (like the real one)
fn classify_client_error(error: &ClientError) -> TestErrorType {
    match &error.kind {
        ClientErrorKind::Io(_) => TestErrorType::Retryable,
        ClientErrorKind::SerdeJson(_) => TestErrorType::Permanent,
        ClientErrorKind::RpcError(rpc_error) => match rpc_error {
            RpcError::RpcResponseError { code: 429, .. } => TestErrorType::RateLimited,
            RpcError::RpcResponseError { code: -32603, .. } => TestErrorType::Retryable,
            RpcError::RpcRequestError(msg) if msg.contains("rate limit") => {
                TestErrorType::RateLimited
            }
            _ => TestErrorType::Unknown,
        },
        ClientErrorKind::Custom(msg) => {
            if msg.contains("InsufficientFunds") {
                TestErrorType::Permanent
            } else if msg.contains("InvalidSignature") {
                TestErrorType::Permanent
            } else {
                TestErrorType::Unknown
            }
        }
        _ => TestErrorType::Unknown,
    }
}

/// Test Arc dereferencing with &* operator
#[test]
fn test_arc_dereference_with_ampersand_star() {
    let client_error = ClientError::new_with_request(
        ClientErrorKind::Io(std::io::Error::new(
            std::io::ErrorKind::TimedOut,
            "Network timeout",
        )),
        RpcRequest::GetAccountInfo,
    );

    let signer_error = SignerError::SolanaTransaction(Arc::new(client_error));

    // Test function compatibility using &* dereferencing
    if let SignerError::SolanaTransaction(arc_error) = signer_error {
        let result = classify_client_error(&*arc_error);
        assert_eq!(result, TestErrorType::Retryable);
    }
}

/// Test Arc dereferencing with as_ref() method
#[test]
fn test_arc_dereference_with_as_ref() {
    let client_error = ClientError::new_with_request(
        ClientErrorKind::Custom("InvalidSignature detected".to_string()),
        RpcRequest::SendTransaction,
    );

    let signer_error = SignerError::SolanaTransaction(Arc::new(client_error));

    // Test function compatibility using as_ref()
    if let SignerError::SolanaTransaction(arc_error) = signer_error {
        let result = classify_client_error(arc_error.as_ref());
        assert_eq!(result, TestErrorType::Permanent);
    }
}

/// Test that error classification works correctly through Arc for all error types
#[test]
fn test_comprehensive_error_classification_through_arc() {
    let test_cases = vec![
        // IO errors should be retryable
        (
            ClientErrorKind::Io(std::io::Error::new(
                std::io::ErrorKind::ConnectionRefused,
                "Connection refused",
            )),
            TestErrorType::Retryable,
        ),
        // Serde errors should be permanent
        (
            ClientErrorKind::SerdeJson(
                serde_json::from_str::<serde_json::Value>("invalid").unwrap_err(),
            ),
            TestErrorType::Permanent,
        ),
        // Rate limit errors should be rate limited
        (
            ClientErrorKind::RpcError(RpcError::RpcResponseError {
                code: 429,
                message: "Too Many Requests".to_string(),
                data: RpcResponseErrorData::Empty,
            }),
            TestErrorType::RateLimited,
        ),
        // Internal RPC errors should be retryable
        (
            ClientErrorKind::RpcError(RpcError::RpcResponseError {
                code: -32603,
                message: "Internal error".to_string(),
                data: RpcResponseErrorData::Empty,
            }),
            TestErrorType::Retryable,
        ),
        // Rate limit in request error
        (
            ClientErrorKind::RpcError(RpcError::RpcRequestError("rate limit exceeded".to_string())),
            TestErrorType::RateLimited,
        ),
        // Insufficient funds should be permanent
        (
            ClientErrorKind::Custom("InsufficientFundsForRent".to_string()),
            TestErrorType::Permanent,
        ),
        // Invalid signature should be permanent
        (
            ClientErrorKind::Custom("InvalidSignature".to_string()),
            TestErrorType::Permanent,
        ),
        // Unknown custom error
        (
            ClientErrorKind::Custom("Unknown custom error".to_string()),
            TestErrorType::Unknown,
        ),
    ];

    for (i, (kind, expected_type)) in test_cases.into_iter().enumerate() {
        let client_error = ClientError::new_with_request(kind, RpcRequest::SendTransaction);
        let signer_error = SignerError::SolanaTransaction(Arc::new(client_error));

        match signer_error {
            SignerError::SolanaTransaction(arc_error) => {
                // Test both dereferencing methods
                let result1 = classify_client_error(&*arc_error);
                let result2 = classify_client_error(arc_error.as_ref());

                assert_eq!(
                    result1, expected_type,
                    "Test case {} failed with &* dereferencing",
                    i
                );
                assert_eq!(
                    result2, expected_type,
                    "Test case {} failed with as_ref() dereferencing",
                    i
                );
                assert_eq!(
                    result1, result2,
                    "Different dereferencing methods should give same result for test case {}",
                    i
                );
            }
            _ => panic!("Test case {}: Expected SolanaTransaction variant", i),
        }
    }
}

/// Test that complex error information is preserved through Arc dereferencing
#[test]
fn test_complex_error_information_preservation() {
    let complex_rpc_error = RpcError::RpcResponseError {
        code: -32602,
        message: "Invalid params: account not found".to_string(),
        data: RpcResponseErrorData::Empty,
    };

    let client_error = ClientError::new_with_request(
        ClientErrorKind::RpcError(complex_rpc_error),
        RpcRequest::GetAccountInfo,
    );

    let signer_error = SignerError::SolanaTransaction(Arc::new(client_error));

    if let SignerError::SolanaTransaction(arc_error) = signer_error {
        // Test that all information is accessible through dereferencing
        let dereferenced: &ClientError = &*arc_error;

        // Verify request information is preserved
        assert_eq!(dereferenced.request, Some(RpcRequest::GetAccountInfo));

        // Verify error kind is preserved
        match &dereferenced.kind {
            ClientErrorKind::RpcError(RpcError::RpcResponseError { code, message, .. }) => {
                assert_eq!(*code, -32602);
                assert!(message.contains("Invalid params"));
                assert!(message.contains("account not found"));
            }
            _ => panic!("Expected RpcError with RpcResponseError"),
        }

        // Verify string representation preserves all information
        let error_string = dereferenced.to_string();
        assert!(error_string.contains("-32602"));
        assert!(error_string.contains("Invalid params"));
        assert!(error_string.contains("account not found"));
    }
}

/// Test function that takes multiple Arc<ClientError> parameters
#[test]
fn test_multiple_arc_parameters() {
    fn compare_errors(error1: &ClientError, error2: &ClientError) -> bool {
        // Simple comparison function that takes multiple ClientError references
        error1.to_string() == error2.to_string()
    }

    let client_error1 = ClientError::new_with_request(
        ClientErrorKind::Custom("Same error".to_string()),
        RpcRequest::GetAccountInfo,
    );

    let client_error2 = ClientError::new_with_request(
        ClientErrorKind::Custom("Same error".to_string()),
        RpcRequest::GetAccountInfo,
    );

    let client_error3 = ClientError::new_with_request(
        ClientErrorKind::Custom("Different error".to_string()),
        RpcRequest::SendTransaction,
    );

    let arc1 = Arc::new(client_error1);
    let arc2 = Arc::new(client_error2);
    let arc3 = Arc::new(client_error3);

    // Test function calls with dereferenced Arcs
    assert!(compare_errors(&*arc1, &*arc2)); // Same content
    assert!(!compare_errors(&*arc1, &*arc3)); // Different content
    assert!(!compare_errors(&*arc2, &*arc3)); // Different content

    // Test with as_ref()
    assert!(compare_errors(arc1.as_ref(), arc2.as_ref()));
    assert!(!compare_errors(arc1.as_ref(), arc3.as_ref()));
}

/// Test that pattern matching works correctly with Arc<ClientError>
#[test]
fn test_pattern_matching_with_arc() {
    let test_cases = vec![
        ClientErrorKind::Io(std::io::Error::new(std::io::ErrorKind::TimedOut, "Timeout")),
        ClientErrorKind::Custom("Test custom error".to_string()),
        ClientErrorKind::RpcError(RpcError::RpcRequestError("Test RPC error".to_string())),
    ];

    for (i, kind) in test_cases.into_iter().enumerate() {
        let client_error = ClientError::new_with_request(kind, RpcRequest::GetAccountInfo);
        let signer_error = SignerError::SolanaTransaction(Arc::new(client_error));

        if let SignerError::SolanaTransaction(arc_error) = signer_error {
            // Test pattern matching on dereferenced Arc
            let classification = match &arc_error.kind {
                ClientErrorKind::Io(_) => "IO Error",
                ClientErrorKind::Custom(_) => "Custom Error",
                ClientErrorKind::RpcError(_) => "RPC Error",
                _ => "Other Error",
            };

            let expected = match i {
                0 => "IO Error",
                1 => "Custom Error",
                2 => "RPC Error",
                _ => unreachable!(),
            };

            assert_eq!(classification, expected, "Test case {} failed", i);
        }
    }
}

/// Test error chain traversal through Arc
#[test]
fn test_error_chain_traversal_through_arc() {
    let io_error = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "Access denied");
    let client_error =
        ClientError::new_with_request(ClientErrorKind::Io(io_error), RpcRequest::GetAccountInfo);

    let signer_error = SignerError::SolanaTransaction(Arc::new(client_error));

    if let SignerError::SolanaTransaction(arc_error) = signer_error {
        // Test traversing the error chain through Arc
        let dereferenced: &ClientError = &*arc_error;

        match &dereferenced.kind {
            ClientErrorKind::Io(inner_io_error) => {
                // We can access the inner IO error through Arc dereferencing
                assert_eq!(inner_io_error.kind(), std::io::ErrorKind::PermissionDenied);
                assert!(inner_io_error.to_string().contains("Access denied"));

                // Test that we can still use std::error::Error methods
                let _error_string = inner_io_error.to_string();
            }
            _ => panic!("Expected IO error"),
        }
    }
}

/// Test that Arc doesn't interfere with error source chain
#[test]
fn test_error_source_chain_through_arc() {
    let io_error = std::io::Error::new(std::io::ErrorKind::BrokenPipe, "Broken pipe");
    let client_error =
        ClientError::new_with_request(ClientErrorKind::Io(io_error), RpcRequest::SendTransaction);

    let signer_error = SignerError::SolanaTransaction(Arc::new(client_error));

    if let SignerError::SolanaTransaction(arc_error) = signer_error {
        // Test that error source information is accessible
        let error_ref: &ClientError = arc_error.as_ref();

        // Verify the error displays correctly
        let display_string = format!("{}", error_ref);
        assert!(display_string.contains("Broken pipe"));

        // Verify debug representation
        let debug_string = format!("{:?}", error_ref);
        assert!(!debug_string.is_empty());

        // Test accessing the inner error
        if let ClientErrorKind::Io(inner_error) = &error_ref.kind {
            assert_eq!(inner_error.kind(), std::io::ErrorKind::BrokenPipe);
        }
    }
}

/// Test performance impact of Arc dereferencing in function calls
#[test]
fn test_arc_dereferencing_performance() {
    let client_error = ClientError::new_with_request(
        ClientErrorKind::Custom("Performance test".to_string()),
        RpcRequest::GetAccountInfo,
    );

    let arc_error = Arc::new(client_error);
    const ITERATIONS: usize = 1000;

    // Measure time for multiple dereferences (this is more of a smoke test)
    let start = std::time::Instant::now();
    for _ in 0..ITERATIONS {
        let _result = classify_client_error(&*arc_error);
    }
    let duration_star = start.elapsed();

    let start = std::time::Instant::now();
    for _ in 0..ITERATIONS {
        let _result = classify_client_error(arc_error.as_ref());
    }
    let duration_as_ref = start.elapsed();

    // Both should complete quickly (this is a smoke test, not a precise benchmark)
    assert!(
        duration_star.as_millis() < 100,
        "Arc dereferencing with &* took too long: {:?}",
        duration_star
    );
    assert!(
        duration_as_ref.as_millis() < 100,
        "Arc dereferencing with as_ref() took too long: {:?}",
        duration_as_ref
    );

    // Both methods should perform similarly
    let ratio = duration_star.as_nanos() as f64 / duration_as_ref.as_nanos() as f64;
    assert!(
        ratio > 0.1 && ratio < 10.0,
        "Performance difference too large: {:?} vs {:?}",
        duration_star,
        duration_as_ref
    );
}
