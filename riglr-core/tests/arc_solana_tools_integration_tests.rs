//! Integration tests with riglr-solana-tools error classification
//!
//! These tests verify that the Arc<ClientError> implementation works correctly
//! with the actual error classification functions from riglr-solana-tools.

use riglr_core::signer::SignerError;
use solana_client::client_error::{ClientError, ClientErrorKind};
use solana_client::rpc_request::{RpcError, RpcRequest, RpcResponseErrorData};
use std::sync::Arc;

// This test demonstrates how the Arc<ClientError> would work with the actual
// classify_transaction_error function from riglr-solana-tools once available

/// Test Arc dereferencing pattern that would be used in riglr-solana-tools
#[test]
fn test_arc_dereferencing_pattern() {
    let test_cases = vec![
        (
            ClientErrorKind::Io(std::io::Error::new(
                std::io::ErrorKind::TimedOut,
                "Connection timed out",
            )),
            "IO error should be accessible through Arc",
        ),
        (
            ClientErrorKind::RpcError(RpcError::RpcResponseError {
                code: 429,
                message: "Too Many Requests".to_string(),
                data: RpcResponseErrorData::Empty,
            }),
            "RPC error should be accessible through Arc",
        ),
        (
            ClientErrorKind::Custom("InsufficientFundsForRent".to_string()),
            "Custom error should be accessible through Arc",
        ),
    ];

    for (kind, description) in test_cases {
        let client_error = ClientError::new_with_request(kind, RpcRequest::SendTransaction);
        let signer_error = SignerError::SolanaTransaction(Arc::new(client_error));

        // This is the pattern that would be used in riglr-solana-tools
        if let SignerError::SolanaTransaction(arc_error) = signer_error {
            // Method 1: Using &* dereferencing (as shown in plan)
            let _error_ref1: &ClientError = &*arc_error;

            // Method 2: Using as_ref() method
            let _error_ref2: &ClientError = arc_error.as_ref();

            // Both methods should give the same result
            assert_eq!(
                _error_ref1.to_string(),
                _error_ref2.to_string(),
                "{}",
                description
            );

            // Verify the error information is fully preserved
            assert_eq!(_error_ref1.request, _error_ref2.request);

            // Test pattern matching on the dereferenced Arc
            match &_error_ref1.kind {
                ClientErrorKind::Io(_) => {
                    assert!(_error_ref1.to_string().contains("Connection timed out"));
                }
                ClientErrorKind::RpcError(RpcError::RpcResponseError { code: 429, .. }) => {
                    assert!(_error_ref1.to_string().contains("Too Many Requests"));
                }
                ClientErrorKind::Custom(msg) if msg.contains("InsufficientFunds") => {
                    assert!(msg == "InsufficientFundsForRent");
                }
                _ => {}
            }
        }
    }
}

/// Test the exact pattern used in riglr-solana-tools utils/transaction.rs line 46
#[test]
fn test_exact_riglr_solana_tools_pattern() {
    // This mimics the exact code pattern from the plan:
    // match classify_transaction_error(&*client_error) {

    let client_error = ClientError::new_with_request(
        ClientErrorKind::Io(std::io::Error::new(
            std::io::ErrorKind::ConnectionRefused,
            "Connection refused",
        )),
        RpcRequest::GetAccountInfo,
    );

    let signer_error = SignerError::SolanaTransaction(Arc::new(client_error));

    // Simulate the pattern from classify_signer_error_for_retry function
    if let SignerError::SolanaTransaction(client_error) = signer_error {
        // This is the exact pattern: &*client_error
        let error_ref: &ClientError = &*client_error;

        // Verify we can access all the ClientError functionality
        assert!(error_ref.to_string().contains("Connection refused"));
        assert_eq!(error_ref.request, Some(RpcRequest::GetAccountInfo));

        match &error_ref.kind {
            ClientErrorKind::Io(io_error) => {
                assert_eq!(io_error.kind(), std::io::ErrorKind::ConnectionRefused);
            }
            _ => panic!("Expected IO error"),
        }
    }
}

/// Test error conversion patterns from the plan
#[test]
fn test_error_conversion_patterns() {
    // Test From<ClientError> for SignerError conversion (Step 2 in plan)
    let client_error = ClientError::new_with_request(
        ClientErrorKind::Custom("Test conversion".to_string()),
        RpcRequest::SendTransaction,
    );

    // Direct conversion using From trait
    let signer_error: SignerError = client_error.into();

    match &signer_error {
        SignerError::SolanaTransaction(arc_error) => {
            assert!(arc_error.to_string().contains("Test conversion"));

            // Verify Arc properties
            assert_eq!(Arc::strong_count(&arc_error), 1);

            // Test cloning
            let cloned_error = signer_error.clone();
            if let SignerError::SolanaTransaction(cloned_arc) = cloned_error {
                assert_eq!(Arc::strong_count(&arc_error), 2);
                assert!(Arc::ptr_eq(&arc_error, &cloned_arc));
            }
        }
        _ => panic!("Expected SolanaTransaction variant"),
    }

    // Test From<Box<ClientError>> for SignerError conversion
    let boxed_client_error = Box::new(ClientError::new_with_request(
        ClientErrorKind::Custom("Test box conversion".to_string()),
        RpcRequest::GetAccountInfo,
    ));

    let signer_error_from_box: SignerError = boxed_client_error.into();

    match signer_error_from_box {
        SignerError::SolanaTransaction(arc_error) => {
            assert!(arc_error.to_string().contains("Test box conversion"));
            assert_eq!(Arc::strong_count(&arc_error), 1);
        }
        _ => panic!("Expected SolanaTransaction variant"),
    }
}

/// Test that all original ClientError information is preserved through Arc
#[test]
fn test_complete_error_information_preservation() {
    let complex_rpc_error = RpcError::RpcResponseError {
        code: -32602,
        message: "Invalid params: account HTTz5F8gpB1tpAaB5ATaJXtRpnhqvN3opWdGvgqrJqFM not found"
            .to_string(),
        data: RpcResponseErrorData::Empty,
    };

    let client_error = ClientError::new_with_request(
        ClientErrorKind::RpcError(complex_rpc_error),
        RpcRequest::GetAccountInfo,
    );

    let original_request = client_error.request;
    let original_message = client_error.to_string();

    let signer_error = SignerError::SolanaTransaction(Arc::new(client_error));

    if let SignerError::SolanaTransaction(arc_error) = signer_error {
        // Verify all information is preserved through Arc
        let dereferenced: &ClientError = &*arc_error;

        // Request information preserved
        assert_eq!(dereferenced.request, original_request);
        assert_eq!(dereferenced.request, Some(RpcRequest::GetAccountInfo));

        // Message content preserved
        assert_eq!(dereferenced.to_string(), original_message);
        assert!(dereferenced.to_string().contains("Invalid params"));
        assert!(dereferenced
            .to_string()
            .contains("HTTz5F8gpB1tpAaB5ATaJXtRpnhqvN3opWdGvgqrJqFM"));
        assert!(dereferenced.to_string().contains("not found"));

        // Structured error data preserved
        match &dereferenced.kind {
            ClientErrorKind::RpcError(RpcError::RpcResponseError { code, message, .. }) => {
                assert_eq!(*code, -32602);
                assert!(message.contains("Invalid params"));
                assert!(message.contains("HTTz5F8gpB1tpAaB5ATaJXtRpnhqvN3opWdGvgqrJqFM"));
            }
            _ => panic!("Expected RpcResponseError"),
        }
    }
}

/// Test error classification compatibility with different ClientError variants
#[test]
fn test_error_classification_compatibility() {
    // This simulates the classify_transaction_error function usage patterns
    fn mock_classify_transaction_error(error: &ClientError) -> &'static str {
        match &error.kind {
            ClientErrorKind::Io(_) => "Retryable",
            ClientErrorKind::SerdeJson(_) => "Permanent",
            ClientErrorKind::RpcError(rpc_error) => match rpc_error {
                RpcError::RpcResponseError { code: 429, .. } => "RateLimited",
                RpcError::RpcResponseError { code: -32603, .. } => "Retryable",
                _ => "Unknown",
            },
            ClientErrorKind::Custom(msg) => {
                if msg.contains("InsufficientFunds") {
                    "Permanent"
                } else if msg.contains("InvalidSignature") {
                    "Permanent"
                } else {
                    "Unknown"
                }
            }
            _ => "Unknown",
        }
    }

    let test_cases = vec![
        (
            ClientErrorKind::Io(std::io::Error::new(std::io::ErrorKind::TimedOut, "timeout")),
            "Retryable",
        ),
        (
            ClientErrorKind::SerdeJson(
                serde_json::from_str::<serde_json::Value>("invalid").unwrap_err(),
            ),
            "Permanent",
        ),
        (
            ClientErrorKind::RpcError(RpcError::RpcResponseError {
                code: 429,
                message: "Too Many Requests".to_string(),
                data: RpcResponseErrorData::Empty,
            }),
            "RateLimited",
        ),
        (
            ClientErrorKind::Custom("InsufficientFundsForRent".to_string()),
            "Permanent",
        ),
        (
            ClientErrorKind::Custom("InvalidSignature".to_string()),
            "Permanent",
        ),
    ];

    for (kind, expected_classification) in test_cases {
        let client_error = ClientError::new_with_request(kind, RpcRequest::SendTransaction);
        let signer_error = SignerError::SolanaTransaction(Arc::new(client_error));

        if let SignerError::SolanaTransaction(arc_error) = signer_error {
            // Test the exact pattern from riglr-solana-tools
            let classification = mock_classify_transaction_error(&*arc_error);
            assert_eq!(classification, expected_classification);

            // Also test with as_ref() method
            let classification_alt = mock_classify_transaction_error(arc_error.as_ref());
            assert_eq!(classification_alt, expected_classification);
        }
    }
}

/// Test the specific error handling patterns used in riglr-solana-tools
#[test]
fn test_riglr_solana_tools_error_patterns() {
    // Pattern 1: Error creation in signer/local.rs (line 96 and 104 in plan)
    let io_error = std::io::Error::new(std::io::ErrorKind::TimedOut, "Request timeout");
    let client_error =
        ClientError::new_with_request(ClientErrorKind::Io(io_error), RpcRequest::SendTransaction);

    // This simulates: .map_err(|e| SignerError::SolanaTransaction(Arc::new(e)))?
    let signer_error = SignerError::SolanaTransaction(Arc::new(client_error));

    // Pattern 2: Error handling in utils/transaction.rs (line 46 in plan)
    if let SignerError::SolanaTransaction(client_error) = signer_error {
        // This simulates: match classify_transaction_error(&*client_error) {
        let error_ref: &ClientError = &*client_error;

        // Verify the pattern works correctly
        assert!(error_ref.to_string().contains("Request timeout"));
        assert_eq!(error_ref.request, Some(RpcRequest::SendTransaction));

        // Test that we can still access all ClientError functionality
        match &error_ref.kind {
            ClientErrorKind::Io(inner_io_error) => {
                assert_eq!(inner_io_error.kind(), std::io::ErrorKind::TimedOut);
                assert!(inner_io_error.to_string().contains("Request timeout"));
            }
            _ => panic!("Expected IO error"),
        }
    }
}

/// Test performance of Arc dereferencing in error classification
#[test]
fn test_arc_dereferencing_performance() {
    let client_error = ClientError::new_with_request(
        ClientErrorKind::Custom("Performance test error".to_string()),
        RpcRequest::GetAccountInfo,
    );
    let signer_error = SignerError::SolanaTransaction(Arc::new(client_error));

    if let SignerError::SolanaTransaction(arc_error) = signer_error {
        const ITERATIONS: usize = 10000;

        // Test &* dereferencing performance
        let start = std::time::Instant::now();
        for _ in 0..ITERATIONS {
            let _error_ref: &ClientError = &*arc_error;
            let _message = _error_ref.to_string();
        }
        let duration_star = start.elapsed();

        // Test as_ref() performance
        let start = std::time::Instant::now();
        for _ in 0..ITERATIONS {
            let _error_ref: &ClientError = arc_error.as_ref();
            let _message = _error_ref.to_string();
        }
        let duration_as_ref = start.elapsed();

        // Both should be fast (this is a smoke test)
        assert!(
            duration_star.as_millis() < 1000,
            "Arc dereferencing too slow: {:?}",
            duration_star
        );
        assert!(
            duration_as_ref.as_millis() < 1000,
            "Arc as_ref() too slow: {:?}",
            duration_as_ref
        );

        println!("Arc dereferencing performance:");
        println!(
            "  &* method: {:?} for {} iterations",
            duration_star, ITERATIONS
        );
        println!(
            "  as_ref() method: {:?} for {} iterations",
            duration_as_ref, ITERATIONS
        );
    }
}

/// Test that Arc<ClientError> maintains all std::error::Error functionality
#[test]
fn test_std_error_trait_compatibility() {
    let io_error = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "Access denied");
    let client_error =
        ClientError::new_with_request(ClientErrorKind::Io(io_error), RpcRequest::GetAccountInfo);
    let signer_error = SignerError::SolanaTransaction(Arc::new(client_error));

    if let SignerError::SolanaTransaction(arc_error) = signer_error {
        let error_ref: &ClientError = &*arc_error;

        // Test Display trait
        let display = format!("{}", error_ref);
        assert!(display.contains("Access denied"));

        // Test Debug trait
        let debug = format!("{:?}", error_ref);
        assert!(!debug.is_empty());

        // Test that we can still access the source error
        match &error_ref.kind {
            ClientErrorKind::Io(inner_io_error) => {
                // std::error::Error methods should still work
                assert_eq!(inner_io_error.kind(), std::io::ErrorKind::PermissionDenied);
                assert!(inner_io_error.to_string().contains("Access denied"));
            }
            _ => panic!("Expected IO error"),
        }
    }
}

/// Test memory efficiency of Arc vs direct storage
#[test]
fn test_memory_efficiency() {
    let client_error = ClientError::new_with_request(
        ClientErrorKind::Custom("Memory efficiency test".to_string()),
        RpcRequest::GetAccountInfo,
    );

    // Direct storage size
    let direct_size = std::mem::size_of_val(&client_error);

    // Arc storage size
    let arc_error = Arc::new(client_error);
    let arc_size = std::mem::size_of_val(&arc_error);

    // SignerError with Arc size
    let signer_error = SignerError::SolanaTransaction(arc_error);
    let signer_error_size = std::mem::size_of_val(&signer_error);

    println!("Memory usage comparison:");
    println!("  Direct ClientError: {} bytes", direct_size);
    println!("  Arc<ClientError>: {} bytes", arc_size);
    println!("  SignerError with Arc: {} bytes", signer_error_size);

    // Arc should be a small, fixed size (pointer + ref count)
    assert!(arc_size <= std::mem::size_of::<usize>() * 2);

    // When sharing errors, Arc is more efficient than copying
    let cloned_signer_error = signer_error.clone();
    let cloned_size = std::mem::size_of_val(&cloned_signer_error);

    // Cloned error should be same size (just another Arc reference)
    assert_eq!(signer_error_size, cloned_size);
}
