//! Tests for Arc<ClientError> implementation in SignerError
//!
//! These tests verify that the Arc<ClientError> implementation works correctly
//! for cloneability, thread safety, error conversion, and function compatibility.

use riglr_core::signer::SignerError;
use solana_client::client_error::{ClientError, ClientErrorKind};
use solana_client::rpc_request::{RpcError, RpcRequest, RpcResponseErrorData};
use std::sync::Arc;
use std::thread;

/// Test error conversion from ClientError to SignerError::SolanaTransaction(Arc<ClientError>)
#[test]
fn test_client_error_to_signer_error_conversion() {
    let io_error = std::io::Error::new(std::io::ErrorKind::TimedOut, "Connection timed out");
    let client_error =
        ClientError::new_with_request(ClientErrorKind::Io(io_error), RpcRequest::GetAccountInfo);

    // Test direct conversion
    let signer_error: SignerError = client_error.into();

    match signer_error {
        SignerError::SolanaTransaction(arc_error) => {
            assert!(arc_error.to_string().contains("Connection timed out"));
            assert!(matches!(arc_error.kind, ClientErrorKind::Io(_)));
        }
        _ => panic!("Expected SignerError::SolanaTransaction variant"),
    }
}

/// Test error conversion from Box<ClientError> to SignerError::SolanaTransaction(Arc<ClientError>)
#[test]
fn test_boxed_client_error_to_signer_error_conversion() {
    let io_error = std::io::Error::new(std::io::ErrorKind::ConnectionRefused, "Connection refused");
    let client_error =
        ClientError::new_with_request(ClientErrorKind::Io(io_error), RpcRequest::SendTransaction);
    let boxed_error = Box::new(client_error);

    // Test conversion from Box<ClientError>
    let signer_error: SignerError = boxed_error.into();

    match signer_error {
        SignerError::SolanaTransaction(arc_error) => {
            assert!(arc_error.to_string().contains("Connection refused"));
            assert!(matches!(arc_error.kind, ClientErrorKind::Io(_)));
        }
        _ => panic!("Expected SignerError::SolanaTransaction variant"),
    }
}

/// Test that Arc<ClientError> implements Clone correctly
#[test]
fn test_arc_client_error_clone() {
    let rpc_error = RpcError::RpcResponseError {
        code: 429,
        message: "Too Many Requests".to_string(),
        data: RpcResponseErrorData::Empty,
    };
    let client_error = ClientError::new_with_request(
        ClientErrorKind::RpcError(rpc_error),
        RpcRequest::SendTransaction,
    );

    let signer_error = SignerError::SolanaTransaction(Arc::new(client_error));

    // Test cloning the SignerError
    let cloned_error = signer_error.clone();

    // Verify both errors contain the same information
    match (&signer_error, &cloned_error) {
        (SignerError::SolanaTransaction(original), SignerError::SolanaTransaction(cloned)) => {
            assert_eq!(original.to_string(), cloned.to_string());
            assert_eq!(Arc::strong_count(original), 2); // Original + clone
            assert_eq!(Arc::strong_count(cloned), 2); // Same reference
            assert!(Arc::ptr_eq(original, cloned)); // Same Arc instance
        }
        _ => panic!("Expected both to be SignerError::SolanaTransaction variants"),
    }
}

/// Test error message preservation through Arc wrapping
#[test]
fn test_error_message_preservation() {
    let test_cases = vec![
        (
            ClientErrorKind::Custom("InsufficientFundsForRent".to_string()),
            "InsufficientFundsForRent",
        ),
        (
            ClientErrorKind::Custom("InvalidSignature".to_string()),
            "InvalidSignature",
        ),
        (
            ClientErrorKind::Custom("DuplicateSignature".to_string()),
            "DuplicateSignature",
        ),
    ];

    for (kind, expected_msg) in test_cases {
        let client_error = ClientError::new_with_request(kind, RpcRequest::SendTransaction);
        let signer_error = SignerError::SolanaTransaction(Arc::new(client_error));

        let error_string = signer_error.to_string();
        assert!(
            error_string.contains(expected_msg),
            "Error '{}' should contain '{}'",
            error_string,
            expected_msg
        );

        // Verify the Arc can be dereferenced to access original error
        if let SignerError::SolanaTransaction(arc_error) = signer_error {
            let dereferenced_error: &ClientError = &arc_error;
            assert!(dereferenced_error.to_string().contains(expected_msg));
        }
    }
}

/// Test thread safety of Arc<ClientError>
#[test]
fn test_arc_client_error_thread_safety() {
    let client_error = ClientError::new_with_request(
        ClientErrorKind::Custom("Thread safety test".to_string()),
        RpcRequest::GetAccountInfo,
    );
    let signer_error = SignerError::SolanaTransaction(Arc::new(client_error));

    let handles: Vec<_> = (0..4)
        .map(|i| {
            let error_clone = signer_error.clone();
            thread::spawn(move || {
                // Each thread can access the error independently
                match error_clone {
                    SignerError::SolanaTransaction(arc_error) => {
                        let message = arc_error.to_string();
                        assert!(message.contains("Thread safety test"));
                        format!("Thread {} processed: {}", i, message)
                    }
                    _ => panic!("Expected SignerError::SolanaTransaction in thread {}", i),
                }
            })
        })
        .collect();

    // Wait for all threads and verify results
    let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();
    assert_eq!(results.len(), 4);

    for (i, result) in results.iter().enumerate() {
        assert!(
            result.contains(&format!("Thread {} processed:", i)),
            "Result: {}",
            result
        );
        assert!(result.contains("Thread safety test"));
    }

    // Verify original error is still accessible
    match signer_error {
        SignerError::SolanaTransaction(arc_error) => {
            assert!(arc_error.to_string().contains("Thread safety test"));
        }
        _ => panic!("Expected SignerError::SolanaTransaction"),
    }
}

/// Test Arc reference counting works as expected
#[test]
fn test_arc_reference_counting() {
    let client_error = ClientError::new_with_request(
        ClientErrorKind::Custom("Reference counting test".to_string()),
        RpcRequest::GetAccountInfo,
    );
    let arc_error = Arc::new(client_error);

    // Initial reference count should be 1
    assert_eq!(Arc::strong_count(&arc_error), 1);

    let signer_error = SignerError::SolanaTransaction(arc_error.clone());
    // Reference count should now be 2
    assert_eq!(Arc::strong_count(&arc_error), 2);

    // Create additional clones in their own scope
    {
        let clones: Vec<_> = (0..3).map(|_| signer_error.clone()).collect();
        // Reference count should now be 5 (original + signer_error + 3 clones)
        assert_eq!(Arc::strong_count(&arc_error), 5);
        // Explicitly use clones to avoid unused variable warning
        assert_eq!(clones.len(), 3);
    } // clones go out of scope here
      // Reference count should be back to 2
    assert_eq!(Arc::strong_count(&arc_error), 2);

    // Test moving signer_error and dropping it by scope
    {
        let _moved_error = signer_error;
    } // signer_error is dropped here when _moved_error goes out of scope
      // Reference count should be back to 1
    assert_eq!(Arc::strong_count(&arc_error), 1);
}

/// Test various ClientError types work correctly with Arc
#[test]
fn test_different_client_error_types_with_arc() {
    let test_cases = vec![
        // IO Error
        ClientErrorKind::Io(std::io::Error::new(
            std::io::ErrorKind::TimedOut,
            "Network timeout",
        )),
        // Serde Error
        ClientErrorKind::SerdeJson(
            serde_json::from_str::<serde_json::Value>("invalid").unwrap_err(),
        ),
        // RPC Error
        ClientErrorKind::RpcError(RpcError::RpcResponseError {
            code: -32603,
            message: "Internal error".to_string(),
            data: RpcResponseErrorData::Empty,
        }),
        // Custom Error
        ClientErrorKind::Custom("Custom error message".to_string()),
    ];

    for (i, kind) in test_cases.into_iter().enumerate() {
        let client_error = ClientError::new_with_request(kind, RpcRequest::GetAccountInfo);
        let signer_error = SignerError::SolanaTransaction(Arc::new(client_error));

        // Test cloning
        let cloned = signer_error.clone();

        match (&signer_error, &cloned) {
            (
                SignerError::SolanaTransaction(original),
                SignerError::SolanaTransaction(cloned_arc),
            ) => {
                assert_eq!(original.to_string(), cloned_arc.to_string());
                assert!(Arc::ptr_eq(original, cloned_arc));

                // Verify the Arc can be dereferenced
                let _: &ClientError = original;
                let _: &ClientError = cloned_arc;
            }
            _ => panic!(
                "Test case {} failed: expected SolanaTransaction variants",
                i
            ),
        }
    }
}

/// Test Arc<ClientError> works with error display and debug traits
#[test]
fn test_arc_client_error_traits() {
    let client_error = ClientError::new_with_request(
        ClientErrorKind::Custom("Traits test".to_string()),
        RpcRequest::SendTransaction,
    );
    let signer_error = SignerError::SolanaTransaction(Arc::new(client_error));

    // Test Display trait
    let display_string = signer_error.to_string();
    assert!(display_string.contains("Solana transaction error:"));
    assert!(display_string.contains("Traits test"));

    // Test Debug trait
    let debug_string = format!("{:?}", signer_error);
    assert!(debug_string.contains("SolanaTransaction"));

    // Test that Arc doesn't interfere with error information
    if let SignerError::SolanaTransaction(arc_error) = signer_error {
        let inner_display = arc_error.to_string();
        let inner_debug = format!("{:?}", arc_error);

        assert!(inner_display.contains("Traits test"));
        assert!(!inner_debug.is_empty());
    }
}

/// Test memory efficiency - Arc should not cause excessive memory usage
#[test]
fn test_arc_memory_efficiency() {
    let client_error = ClientError::new_with_request(
        ClientErrorKind::Custom("Memory test".to_string()),
        RpcRequest::GetAccountInfo,
    );
    let original_size = std::mem::size_of_val(&client_error);

    let arc_error = Arc::new(client_error);
    let arc_size = std::mem::size_of_val(&arc_error);

    // Arc should be smaller than the original error (just a pointer + ref count)
    assert!(arc_size <= std::mem::size_of::<usize>() * 2); // Pointer + ref count
    assert!(arc_size < original_size); // Arc should be more memory efficient for sharing

    let signer_error = SignerError::SolanaTransaction(arc_error);
    let signer_error_size = std::mem::size_of_val(&signer_error);

    // SignerError should not be excessively large
    assert!(signer_error_size < original_size + 100); // Reasonable overhead
}

/// Test that Arc<ClientError> preserves error source information
#[test]
fn test_arc_preserves_error_source() {
    let io_error = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "Access denied");
    let client_error =
        ClientError::new_with_request(ClientErrorKind::Io(io_error), RpcRequest::GetAccountInfo);

    let signer_error = SignerError::SolanaTransaction(Arc::new(client_error));

    // Test that we can still extract the source error through Arc
    if let SignerError::SolanaTransaction(arc_error) = signer_error {
        match &arc_error.kind {
            ClientErrorKind::Io(inner_io_error) => {
                assert_eq!(inner_io_error.kind(), std::io::ErrorKind::PermissionDenied);
                assert!(inner_io_error.to_string().contains("Access denied"));
            }
            _ => panic!("Expected IO error kind"),
        }
    }
}

/// Test concurrent access to Arc<ClientError> from multiple threads
#[test]
fn test_concurrent_arc_access() {
    let client_error = ClientError::new_with_request(
        ClientErrorKind::Custom("Concurrent access test".to_string()),
        RpcRequest::SendTransaction,
    );
    let signer_error = SignerError::SolanaTransaction(Arc::new(client_error));

    const NUM_THREADS: usize = 10;
    const OPERATIONS_PER_THREAD: usize = 100;

    let handles: Vec<_> = (0..NUM_THREADS)
        .map(|thread_id| {
            let error_clone = signer_error.clone();
            thread::spawn(move || {
                for op_id in 0..OPERATIONS_PER_THREAD {
                    match &error_clone {
                        SignerError::SolanaTransaction(arc_error) => {
                            // Simulate various operations
                            let display = arc_error.to_string();
                            let debug = format!("{:?}", arc_error);
                            let error_ref: &ClientError = arc_error;

                            // Use the values to prevent unused variable warnings
                            assert!(!display.is_empty());
                            assert!(!debug.is_empty());
                            assert!(error_ref.to_string().contains("Concurrent access test"));

                            // Verify error content
                            assert!(arc_error.to_string().contains("Concurrent access test"));
                        }
                        _ => panic!(
                            "Thread {} operation {}: Expected SolanaTransaction",
                            thread_id, op_id
                        ),
                    }
                }
                thread_id
            })
        })
        .collect();

    // Wait for all threads to complete
    let completed_threads: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();
    assert_eq!(completed_threads.len(), NUM_THREADS);

    // Verify all thread IDs are present (no panics occurred)
    for i in 0..NUM_THREADS {
        assert!(completed_threads.contains(&i));
    }
}

/// Test error equality through Arc (same error, different Arc instances)
#[test]
fn test_arc_error_content_equality() {
    let create_error = || {
        ClientError::new_with_request(
            ClientErrorKind::Custom("Same error content".to_string()),
            RpcRequest::GetAccountInfo,
        )
    };

    let error1 = SignerError::SolanaTransaction(Arc::new(create_error()));
    let error2 = SignerError::SolanaTransaction(Arc::new(create_error()));

    // Different Arc instances, same content
    match (&error1, &error2) {
        (SignerError::SolanaTransaction(arc1), SignerError::SolanaTransaction(arc2)) => {
            // Should have same content
            assert_eq!(arc1.to_string(), arc2.to_string());
            // But different Arc instances
            assert!(!Arc::ptr_eq(arc1, arc2));
            // Both should have ref count of 1
            assert_eq!(Arc::strong_count(arc1), 1);
            assert_eq!(Arc::strong_count(arc2), 1);
        }
        _ => panic!("Expected both to be SolanaTransaction variants"),
    }
}
