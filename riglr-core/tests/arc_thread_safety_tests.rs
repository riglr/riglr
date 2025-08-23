//! Thread safety tests for Arc<ClientError> implementation
//!
//! These tests verify that Arc<ClientError> works correctly in multi-threaded
//! scenarios, including concurrent access, reference counting, and thread safety.

use riglr_core::signer::SignerError;
use solana_client::client_error::{ClientError, ClientErrorKind};
use solana_client::rpc_request::{RpcError, RpcRequest, RpcResponseErrorData};
use std::sync::{Arc, Barrier, Mutex};
use std::thread;
use std::time::{Duration, Instant};

/// Test basic thread safety of Arc<ClientError>
#[test]
fn test_basic_thread_safety() {
    let client_error = ClientError::new_with_request(
        ClientErrorKind::Custom("Thread safety test".to_string()),
        RpcRequest::GetAccountInfo,
    );
    let signer_error = SignerError::SolanaTransaction(Arc::new(client_error));

    const NUM_THREADS: usize = 8;
    let barrier = Arc::new(Barrier::new(NUM_THREADS));

    let handles: Vec<_> = (0..NUM_THREADS)
        .map(|thread_id| {
            let error_clone = signer_error.clone();
            let barrier_clone = barrier.clone();

            thread::spawn(move || {
                // Wait for all threads to be ready
                barrier_clone.wait();

                // Each thread performs operations on the shared error
                match error_clone {
                    SignerError::SolanaTransaction(arc_error) => {
                        // Read operations that should be thread-safe
                        let _display = arc_error.to_string();
                        let _debug = format!("{:?}", arc_error);
                        let _ref: &ClientError = &*arc_error;

                        // Verify content
                        assert!(arc_error.to_string().contains("Thread safety test"));

                        thread_id
                    }
                    _ => panic!("Thread {}: Expected SolanaTransaction", thread_id),
                }
            })
        })
        .collect();

    // Wait for all threads to complete
    let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();

    // Verify all threads completed successfully
    assert_eq!(results.len(), NUM_THREADS);
    for i in 0..NUM_THREADS {
        assert!(results.contains(&i));
    }
}

/// Test concurrent error creation and sharing
#[test]
fn test_concurrent_error_creation() {
    const NUM_THREADS: usize = 6;
    let shared_errors = Arc::new(Mutex::new(Vec::new()));

    let handles: Vec<_> = (0..NUM_THREADS)
        .map(|thread_id| {
            let errors_vec = shared_errors.clone();

            thread::spawn(move || {
                // Each thread creates its own error
                let client_error = ClientError::new_with_request(
                    ClientErrorKind::Custom(format!("Error from thread {}", thread_id)),
                    RpcRequest::SendTransaction,
                );
                let signer_error = SignerError::SolanaTransaction(Arc::new(client_error));

                // Add to shared collection
                {
                    let mut errors = errors_vec.lock().unwrap();
                    errors.push(signer_error);
                }

                thread_id
            })
        })
        .collect();

    // Wait for all threads
    let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();
    assert_eq!(results.len(), NUM_THREADS);

    // Verify all errors were created correctly
    let errors = shared_errors.lock().unwrap();
    assert_eq!(errors.len(), NUM_THREADS);

    for (i, error) in errors.iter().enumerate() {
        match error {
            SignerError::SolanaTransaction(arc_error) => {
                let error_message = arc_error.to_string();
                assert!(error_message.contains(&format!("Error from thread {}", i)));
            }
            _ => panic!("Expected SolanaTransaction variant"),
        }
    }
}

/// Test Arc sharing across thread boundaries
#[test]
fn test_arc_sharing_across_threads() {
    let client_error = ClientError::new_with_request(
        ClientErrorKind::RpcError(RpcError::RpcResponseError {
            code: 429,
            message: "Rate limited across threads".to_string(),
            data: RpcResponseErrorData::Empty,
        }),
        RpcRequest::SendTransaction,
    );

    let arc_error = Arc::new(client_error);

    // Share the same Arc across multiple threads
    const NUM_READERS: usize = 5;
    let readers: Vec<_> = (0..NUM_READERS)
        .map(|reader_id| {
            let arc_clone = arc_error.clone();

            thread::spawn(move || {
                // Simulate different types of access
                for iteration in 0..10 {
                    match iteration % 3 {
                        0 => {
                            // Display access
                            let display = arc_clone.to_string();
                            assert!(display.contains("Rate limited across threads"));
                        }
                        1 => {
                            // Pattern matching access
                            match &arc_clone.kind {
                                ClientErrorKind::RpcError(RpcError::RpcResponseError {
                                    code,
                                    ..
                                }) => {
                                    assert_eq!(*code, 429);
                                }
                                _ => panic!("Reader {}: Expected RpcError", reader_id),
                            }
                        }
                        2 => {
                            // Direct field access
                            assert_eq!(arc_clone.request, Some(RpcRequest::SendTransaction));
                        }
                        _ => unreachable!(),
                    }
                }

                reader_id
            })
        })
        .collect();

    // Verify reference count during concurrent access
    assert_eq!(Arc::strong_count(&arc_error), NUM_READERS + 1);

    // Wait for readers to complete
    let completed: Vec<_> = readers.into_iter().map(|h| h.join().unwrap()).collect();
    assert_eq!(completed.len(), NUM_READERS);

    // Reference count should be back to 1
    assert_eq!(Arc::strong_count(&arc_error), 1);
}

/// Test concurrent error classification
#[test]
fn test_concurrent_error_classification() {
    let test_errors = vec![
        ClientError::new_with_request(
            ClientErrorKind::Io(std::io::Error::new(std::io::ErrorKind::TimedOut, "Timeout")),
            RpcRequest::GetAccountInfo,
        ),
        ClientError::new_with_request(
            ClientErrorKind::Custom("InsufficientFundsForRent".to_string()),
            RpcRequest::SendTransaction,
        ),
        ClientError::new_with_request(
            ClientErrorKind::RpcError(RpcError::RpcResponseError {
                code: 429,
                message: "Too Many Requests".to_string(),
                data: RpcResponseErrorData::Empty,
            }),
            RpcRequest::SendTransaction,
        ),
    ];

    let arc_errors: Vec<_> = test_errors.into_iter().map(|e| Arc::new(e)).collect();

    const CLASSIFIERS: usize = 4;
    let results = Arc::new(Mutex::new(Vec::new()));

    let handles: Vec<_> = (0..CLASSIFIERS)
        .map(|classifier_id| {
            let errors = arc_errors.clone();
            let results_clone = results.clone();

            thread::spawn(move || {
                let mut local_results = Vec::new();

                for (error_idx, arc_error) in errors.iter().enumerate() {
                    // Classify each error
                    let classification = classify_error_type(&**arc_error);
                    local_results.push((error_idx, classification));
                }

                // Store results
                {
                    let mut shared_results = results_clone.lock().unwrap();
                    shared_results.push((classifier_id, local_results));
                }

                classifier_id
            })
        })
        .collect();

    let completed: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();
    assert_eq!(completed.len(), CLASSIFIERS);

    // Verify all classifiers got the same results
    let all_results = results.lock().unwrap();
    assert_eq!(all_results.len(), CLASSIFIERS);

    let first_classification = &all_results[0].1;
    for (classifier_id, classification) in all_results.iter() {
        assert_eq!(
            classification, first_classification,
            "Classifier {} got different results",
            classifier_id
        );
    }

    // Verify expected classifications
    assert_eq!(first_classification[0].1, ErrorType::Retryable); // Timeout
    assert_eq!(first_classification[1].1, ErrorType::Permanent); // Insufficient funds
    assert_eq!(first_classification[2].1, ErrorType::RateLimited); // Rate limit
}

#[derive(Debug, PartialEq, Clone)]
enum ErrorType {
    Retryable,
    Permanent,
    RateLimited,
}

fn classify_error_type(error: &ClientError) -> ErrorType {
    match &error.kind {
        ClientErrorKind::Io(_) => ErrorType::Retryable,
        ClientErrorKind::Custom(msg) if msg.contains("InsufficientFunds") => ErrorType::Permanent,
        ClientErrorKind::RpcError(RpcError::RpcResponseError { code: 429, .. }) => {
            ErrorType::RateLimited
        }
        _ => ErrorType::Retryable,
    }
}

/// Test high-concurrency scenario with many threads
#[test]
fn test_high_concurrency() {
    let client_error = ClientError::new_with_request(
        ClientErrorKind::Custom("High concurrency test".to_string()),
        RpcRequest::GetAccountInfo,
    );
    let signer_error = SignerError::SolanaTransaction(Arc::new(client_error));

    const HIGH_THREAD_COUNT: usize = 20;
    const OPERATIONS_PER_THREAD: usize = 100;

    let start_time = Instant::now();
    let handles: Vec<_> = (0..HIGH_THREAD_COUNT)
        .map(|thread_id| {
            let error_clone = signer_error.clone();

            thread::spawn(move || {
                let mut operations_completed = 0;

                for op_id in 0..OPERATIONS_PER_THREAD {
                    match &error_clone {
                        SignerError::SolanaTransaction(arc_error) => {
                            // Simulate realistic error handling operations
                            let _message = arc_error.to_string();
                            let _is_io_error = matches!(arc_error.kind, ClientErrorKind::Io(_));
                            let _request_type = arc_error.request;

                            // Verify content integrity
                            assert!(arc_error.to_string().contains("High concurrency test"));

                            operations_completed += 1;
                        }
                        _ => panic!(
                            "Thread {} op {}: Expected SolanaTransaction",
                            thread_id, op_id
                        ),
                    }
                }

                (thread_id, operations_completed)
            })
        })
        .collect();

    // Wait for all threads and collect results
    let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();
    let total_time = start_time.elapsed();

    // Verify all operations completed successfully
    assert_eq!(results.len(), HIGH_THREAD_COUNT);
    let total_operations: usize = results.iter().map(|(_, ops)| *ops).sum();
    assert_eq!(total_operations, HIGH_THREAD_COUNT * OPERATIONS_PER_THREAD);

    // Performance check - should complete reasonably quickly
    assert!(
        total_time < Duration::from_secs(5),
        "High concurrency test took too long: {:?}",
        total_time
    );

    println!(
        "Completed {} operations across {} threads in {:?}",
        total_operations, HIGH_THREAD_COUNT, total_time
    );
}

/// Test Arc weak references (if needed for cleanup scenarios)
#[test]
fn test_arc_weak_references() {
    let client_error = ClientError::new_with_request(
        ClientErrorKind::Custom("Weak reference test".to_string()),
        RpcRequest::GetAccountInfo,
    );
    let arc_error = Arc::new(client_error);

    // Create weak reference
    let weak_ref = Arc::downgrade(&arc_error);
    assert_eq!(Arc::weak_count(&arc_error), 1);
    assert_eq!(Arc::strong_count(&arc_error), 1);

    // Clone Arc for thread
    let arc_clone = arc_error.clone();
    assert_eq!(Arc::strong_count(&arc_error), 2);

    let weak_ref_clone = weak_ref.clone();
    let handle = thread::spawn(move || {
        // Use the Arc in another thread
        let message = arc_clone.to_string();
        assert!(message.contains("Weak reference test"));

        // Weak reference should still be valid
        let upgraded = weak_ref_clone.upgrade();
        assert!(upgraded.is_some());

        if let Some(upgraded_arc) = upgraded {
            assert!(upgraded_arc.to_string().contains("Weak reference test"));
        }
    });

    handle.join().unwrap();

    // After thread completes, strong count should be back to 1
    assert_eq!(Arc::strong_count(&arc_error), 1);

    // Weak reference should still be valid
    assert!(weak_ref.upgrade().is_some());

    // Test weak reference behavior after dropping the last strong reference
    {
        // Move arc_error into this scope so it gets dropped when the scope ends
        let _arc_to_drop = arc_error;
        // _arc_to_drop is automatically dropped here
    }

    // Now weak reference should be invalid
    assert!(weak_ref.upgrade().is_none());
}

/// Test thread safety with error mutations (creating new errors)
#[test]
fn test_thread_safety_with_error_creation() {
    const CREATOR_THREADS: usize = 8;
    let created_errors = Arc::new(Mutex::new(Vec::new()));

    let creators: Vec<_> = (0..CREATOR_THREADS)
        .map(|creator_id| {
            let errors_collection = created_errors.clone();

            thread::spawn(move || {
                let mut local_errors = Vec::new();

                // Each thread creates multiple errors
                for error_id in 0..10 {
                    let client_error = ClientError::new_with_request(
                        ClientErrorKind::Custom(format!(
                            "Creator {} Error {}",
                            creator_id, error_id
                        )),
                        RpcRequest::GetAccountInfo,
                    );
                    let signer_error = SignerError::SolanaTransaction(Arc::new(client_error));
                    local_errors.push(signer_error);
                }

                // Add to shared collection
                {
                    let mut shared_errors = errors_collection.lock().unwrap();
                    shared_errors.extend(local_errors);
                }

                creator_id
            })
        })
        .collect();

    // Wait for all creators
    let completed_creators: Vec<_> = creators.into_iter().map(|h| h.join().unwrap()).collect();
    assert_eq!(completed_creators.len(), CREATOR_THREADS);

    // Verify all errors were created and test concurrent reading
    let expected_count = CREATOR_THREADS * 10;
    {
        let all_errors = created_errors.lock().unwrap();
        assert_eq!(all_errors.len(), expected_count);

        // Test that we can read all errors
        for error in all_errors.iter() {
            match error {
                SignerError::SolanaTransaction(arc_error) => {
                    let message = arc_error.to_string();
                    assert!(message.contains("Creator"));
                    assert!(message.contains("Error"));
                }
                _ => panic!("Expected SolanaTransaction"),
            }
        }
    } // Release the mutex

    // Test concurrent access to the shared collection
    const READER_THREADS: usize = 4;
    let readers: Vec<_> = (0..READER_THREADS)
        .map(|reader_id| {
            let errors_collection = created_errors.clone();

            thread::spawn(move || {
                let errors = errors_collection.lock().unwrap();
                let mut messages_read = 0;

                for error in errors.iter() {
                    match error {
                        SignerError::SolanaTransaction(arc_error) => {
                            let message = arc_error.to_string();
                            assert!(message.contains("Creator"));
                            assert!(message.contains("Error"));
                            messages_read += 1;
                        }
                        _ => panic!("Reader {}: Expected SolanaTransaction", reader_id),
                    }
                }

                (reader_id, messages_read)
            })
        })
        .collect();

    let reader_results: Vec<_> = readers.into_iter().map(|h| h.join().unwrap()).collect();

    // Verify all readers processed all errors
    for (reader_id, messages_read) in reader_results {
        assert_eq!(
            messages_read, expected_count,
            "Reader {} didn't read all messages",
            reader_id
        );
    }
}

/// Test deadlock prevention with multiple Arcs
#[test]
fn test_deadlock_prevention() {
    let error1 = Arc::new(ClientError::new_with_request(
        ClientErrorKind::Custom("Error 1".to_string()),
        RpcRequest::GetAccountInfo,
    ));

    let error2 = Arc::new(ClientError::new_with_request(
        ClientErrorKind::Custom("Error 2".to_string()),
        RpcRequest::SendTransaction,
    ));

    const THREAD_PAIRS: usize = 4;
    let handles: Vec<_> = (0..THREAD_PAIRS)
        .map(|pair_id| {
            let e1 = error1.clone();
            let e2 = error2.clone();

            // Thread A: accesses error1 then error2
            let thread_a = thread::spawn(move || {
                for _ in 0..100 {
                    let _msg1 = e1.to_string();
                    let _msg2 = e2.to_string();
                }
                format!("A{}", pair_id)
            });

            let e1 = error1.clone();
            let e2 = error2.clone();

            // Thread B: accesses error2 then error1 (reverse order)
            let thread_b = thread::spawn(move || {
                for _ in 0..100 {
                    let _msg2 = e2.to_string();
                    let _msg1 = e1.to_string();
                }
                format!("B{}", pair_id)
            });

            (thread_a, thread_b)
        })
        .collect();

    // This should complete without deadlock
    let start = Instant::now();
    for (thread_a, thread_b) in handles {
        let result_a = thread_a.join().unwrap();
        let result_b = thread_b.join().unwrap();

        assert!(result_a.starts_with('A'));
        assert!(result_b.starts_with('B'));
    }

    let duration = start.elapsed();
    assert!(
        duration < Duration::from_secs(2),
        "Deadlock prevention test took too long: {:?}",
        duration
    );
}
