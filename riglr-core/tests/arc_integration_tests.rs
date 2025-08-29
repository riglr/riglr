//! Integration tests for Arc<ClientError> error flow
//!
//! These tests verify the entire error flow from Solana client operations
//! through SignerError to consumer code, ensuring the Arc implementation
//! works correctly in realistic scenarios.

use riglr_core::signer::SignerError;
use solana_client::client_error::{ClientError, ClientErrorKind};
use solana_client::rpc_request::{RpcError, RpcRequest, RpcResponseErrorData};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

/// Simulates a function that would be in riglr-solana-tools that uses SignerError
fn simulate_solana_operation_result(
    should_fail: bool,
    error_type: &str,
) -> Result<String, SignerError> {
    if !should_fail {
        return Ok("transaction_signature_123".to_string());
    }

    let client_error = match error_type {
        "network_timeout" => ClientError::new_with_request(
            ClientErrorKind::Io(std::io::Error::new(
                std::io::ErrorKind::TimedOut,
                "Network request timed out",
            )),
            RpcRequest::SendTransaction,
        ),
        "rate_limit" => ClientError::new_with_request(
            ClientErrorKind::RpcError(RpcError::RpcResponseError {
                code: 429,
                message: "Too Many Requests".to_string(),
                data: RpcResponseErrorData::Empty,
            }),
            RpcRequest::SendTransaction,
        ),
        "insufficient_funds" => ClientError::new_with_request(
            ClientErrorKind::Custom("InsufficientFundsForRent".to_string()),
            RpcRequest::SendTransaction,
        ),
        "invalid_signature" => ClientError::new_with_request(
            ClientErrorKind::Custom("InvalidSignature".to_string()),
            RpcRequest::SendTransaction,
        ),
        "connection_refused" => ClientError::new_with_request(
            ClientErrorKind::Io(std::io::Error::new(
                std::io::ErrorKind::ConnectionRefused,
                "Connection refused by server",
            )),
            RpcRequest::GetAccountInfo,
        ),
        _ => ClientError::new_with_request(
            ClientErrorKind::Custom("Unknown error".to_string()),
            RpcRequest::GetAccountInfo,
        ),
    };

    Err(SignerError::from(client_error))
}

/// Simulates consumer code that handles SignerError
fn handle_signer_error(error: &SignerError) -> (String, bool, bool) {
    match error {
        SignerError::SolanaTransaction(arc_error) => {
            let error_message = arc_error.to_string();
            let is_retryable = classify_error_for_retry(&*arc_error);
            let is_rate_limited = classify_error_for_rate_limit(&*arc_error);
            (error_message, is_retryable, is_rate_limited)
        }
        SignerError::NoSignerContext => ("No signer context".to_string(), false, false),
        SignerError::Configuration(msg) => (format!("Configuration error: {}", msg), false, false),
        _ => (
            error.to_string(),
            true, // Default to retryable for other errors
            false,
        ),
    }
}

/// Helper function to classify if error is retryable (simulates riglr-solana-tools logic)
fn classify_error_for_retry(error: &ClientError) -> bool {
    match &*error.kind {
        ClientErrorKind::Io(_) => true,
        ClientErrorKind::RpcError(rpc_error) => match rpc_error {
            RpcError::RpcResponseError { code: 429, .. } => true,
            RpcError::RpcResponseError { code: -32603, .. } => true,
            RpcError::RpcRequestError(msg) if msg.contains("timeout") => true,
            _ => false,
        },
        ClientErrorKind::Custom(msg) => {
            !(msg.contains("InsufficientFunds") || msg.contains("InvalidSignature"))
        }
        _ => false,
    }
}

/// Helper function to classify if error is rate limited
fn classify_error_for_rate_limit(error: &ClientError) -> bool {
    match &*error.kind {
        ClientErrorKind::RpcError(rpc_error) => match rpc_error {
            RpcError::RpcResponseError { code: 429, .. } => true,
            RpcError::RpcRequestError(msg) => {
                msg.contains("rate limit") || msg.contains("too many requests")
            }
            _ => false,
        },
        _ => false,
    }
}

/// Test complete error flow from operation to consumer handling
#[test]
fn test_complete_error_flow() {
    let test_cases = vec![
        ("network_timeout", true, false),     // Retryable, not rate limited
        ("rate_limit", true, true),           // Retryable and rate limited
        ("insufficient_funds", false, false), // Not retryable, not rate limited
        ("invalid_signature", false, false),  // Not retryable, not rate limited
        ("connection_refused", true, false),  // Retryable, not rate limited
    ];

    for (error_type, expected_retryable, expected_rate_limited) in test_cases {
        // Simulate operation failure
        let result = simulate_solana_operation_result(true, error_type);
        assert!(result.is_err(), "Expected error for {}", error_type);

        let error = result.unwrap_err();

        // Handle error in consumer code
        let (message, is_retryable, is_rate_limited) = handle_signer_error(&error);

        // Verify error classification
        assert_eq!(
            is_retryable, expected_retryable,
            "Error type '{}' retryable classification incorrect. Message: {}",
            error_type, message
        );
        assert_eq!(
            is_rate_limited, expected_rate_limited,
            "Error type '{}' rate limit classification incorrect. Message: {}",
            error_type, message
        );

        // Verify error message contains expected content
        match error_type {
            "network_timeout" => assert!(message.contains("timed out")),
            "rate_limit" => assert!(message.contains("Too Many Requests")),
            "insufficient_funds" => assert!(message.contains("InsufficientFunds")),
            "invalid_signature" => assert!(message.contains("InvalidSignature")),
            "connection_refused" => assert!(message.contains("Connection refused")),
            _ => {}
        }
    }
}

/// Test successful operation flow
#[test]
fn test_successful_operation_flow() {
    let result = simulate_solana_operation_result(false, "");
    assert!(result.is_ok(), "Expected successful operation");

    let signature = result.unwrap();
    assert_eq!(signature, "transaction_signature_123");
}

/// Test error cloning in distributed worker scenario
#[test]
fn test_distributed_worker_error_handling() {
    let operation_error = simulate_solana_operation_result(true, "network_timeout").unwrap_err();

    // Simulate multiple workers processing the same error
    let workers: Vec<_> = (0..4)
        .map(|worker_id| {
            let error_clone = operation_error.clone();
            thread::spawn(move || {
                let (message, is_retryable, is_rate_limited) = handle_signer_error(&error_clone);

                // Each worker should get consistent results
                WorkerResult {
                    worker_id,
                    message,
                    is_retryable,
                    is_rate_limited,
                }
            })
        })
        .collect();

    let results: Vec<WorkerResult> = workers.into_iter().map(|h| h.join().unwrap()).collect();

    // Verify all workers got consistent results
    assert_eq!(results.len(), 4);
    let first_result = &results[0];

    for result in &results {
        assert_eq!(result.message, first_result.message);
        assert_eq!(result.is_retryable, first_result.is_retryable);
        assert_eq!(result.is_rate_limited, first_result.is_rate_limited);
        assert!(result.message.contains("timed out"));
        assert!(result.is_retryable);
        assert!(!result.is_rate_limited);
    }
}

#[derive(Debug)]
struct WorkerResult {
    #[allow(dead_code)]
    worker_id: usize,
    message: String,
    is_retryable: bool,
    is_rate_limited: bool,
}

/// Test error serialization/deserialization compatibility
#[test]
fn test_error_serialization_compatibility() {
    let client_error = ClientError::new_with_request(
        ClientErrorKind::Custom("Serialization test".to_string()),
        RpcRequest::GetAccountInfo,
    );

    let signer_error = SignerError::SolanaTransaction(Arc::new(client_error));

    // Test Display trait for logging/serialization
    let display_string = signer_error.to_string();
    assert!(display_string.contains("Solana transaction error:"));
    assert!(display_string.contains("Serialization test"));

    // Test Debug trait for debugging
    let debug_string = format!("{:?}", signer_error);
    assert!(debug_string.contains("SolanaTransaction"));

    // Verify the Arc contents are accessible for custom serialization
    if let SignerError::SolanaTransaction(arc_error) = signer_error {
        let inner_display = arc_error.to_string();
        let inner_debug = format!("{:?}", arc_error);

        assert!(inner_display.contains("Serialization test"));
        assert!(!inner_debug.is_empty());

        // Test that we can access structured data
        assert_eq!(arc_error.request, Some(RpcRequest::GetAccountInfo));
        if let ClientErrorKind::Custom(msg) = &*arc_error.kind {
            assert_eq!(msg, "Serialization test");
        }

        // Arc error will be automatically dropped when it goes out of scope
    }
}

/// Test retry logic with Arc<ClientError>
#[test]
fn test_retry_logic_with_arc() {
    // Simulate a retry mechanism that uses Arc<ClientError>
    fn should_retry_operation(error: &SignerError, attempt: u32) -> bool {
        if attempt >= 3 {
            return false; // Max retries reached
        }

        match error {
            SignerError::SolanaTransaction(arc_error) => {
                let is_retryable = classify_error_for_retry(arc_error.as_ref());
                let is_rate_limited = classify_error_for_rate_limit(arc_error.as_ref());

                // Rate limited errors have longer backoff
                if is_rate_limited {
                    thread::sleep(Duration::from_millis(100)); // Simulated backoff
                }

                is_retryable
            }
            _ => false,
        }
    }

    // Test with retryable error
    let retryable_error = simulate_solana_operation_result(true, "network_timeout").unwrap_err();
    assert!(should_retry_operation(&retryable_error, 1));
    assert!(should_retry_operation(&retryable_error, 2));
    assert!(!should_retry_operation(&retryable_error, 3)); // Max retries

    // Test with non-retryable error
    let non_retryable_error =
        simulate_solana_operation_result(true, "insufficient_funds").unwrap_err();
    assert!(!should_retry_operation(&non_retryable_error, 1));

    // Test with rate limited error (includes backoff)
    let rate_limited_error = simulate_solana_operation_result(true, "rate_limit").unwrap_err();
    let start = std::time::Instant::now();
    assert!(should_retry_operation(&rate_limited_error, 1));
    let duration = start.elapsed();
    assert!(
        duration >= Duration::from_millis(90),
        "Expected backoff delay for rate limited error"
    );
}

/// Test error context preservation through Arc
#[test]
fn test_error_context_preservation() {
    let complex_error = ClientError::new_with_request(
        ClientErrorKind::RpcError(RpcError::RpcResponseError {
            code: -32002,
            message: "Transaction simulation failed: BlockhashNotFound".to_string(),
            data: RpcResponseErrorData::Empty,
        }),
        RpcRequest::SimulateTransaction,
    );

    let signer_error = SignerError::SolanaTransaction(Arc::new(complex_error));

    // Test that context is preserved through multiple levels of handling
    let context = extract_error_context(&signer_error);

    assert_eq!(context.request_type, "SimulateTransaction");
    assert_eq!(context.error_code, Some(-32002));
    assert!(context.error_message.contains("BlockhashNotFound"));
    assert!(context
        .error_message
        .contains("Transaction simulation failed"));
}

#[derive(Debug)]
struct ErrorContext {
    request_type: String,
    error_code: Option<i64>,
    error_message: String,
}

fn extract_error_context(error: &SignerError) -> ErrorContext {
    match error {
        SignerError::SolanaTransaction(arc_error) => {
            let request_type = match arc_error.request {
                Some(ref req) => format!("{:?}", req),
                None => "Unknown".to_string(),
            };

            let (error_code, error_message) = match &*arc_error.kind {
                ClientErrorKind::RpcError(RpcError::RpcResponseError { code, message, .. }) => {
                    (Some(*code), message.clone())
                }
                _ => (None, arc_error.to_string()),
            };

            ErrorContext {
                request_type,
                error_code,
                error_message,
            }
        }
        _ => ErrorContext {
            request_type: "Non-Solana".to_string(),
            error_code: None,
            error_message: error.to_string(),
        },
    }
}

/// Test memory usage in high-volume error scenarios
#[test]
fn test_high_volume_error_scenarios() {
    let base_error = ClientError::new_with_request(
        ClientErrorKind::Custom("High volume test".to_string()),
        RpcRequest::GetAccountInfo,
    );

    // Create many error instances that share the same Arc
    let arc_error = Arc::new(base_error);

    let total_processed: usize = {
        let mut errors = Vec::new();

        for _ in 0..1000 {
            errors.push(SignerError::SolanaTransaction(arc_error.clone()));
        }

        // Verify all errors share the same Arc instance
        assert_eq!(Arc::strong_count(&arc_error), 1001); // Original + 1000 copies

        // Process errors in "workers"
        let worker_count = 4;
        let errors_per_worker = errors.len() / worker_count;
        let workers: Vec<_> = (0..worker_count)
            .map(|worker_id| {
                let worker_errors: Vec<_> = errors
                    .iter()
                    .skip(worker_id * errors_per_worker)
                    .take(errors_per_worker)
                    .cloned()
                    .collect();

                thread::spawn(move || {
                    let mut processed_count = 0;
                    for error in worker_errors {
                        if let SignerError::SolanaTransaction(arc_error) = error {
                            let _message = arc_error.to_string();
                            let _is_retryable = classify_error_for_retry(arc_error.as_ref());
                            processed_count += 1;
                        }
                    }
                    processed_count
                })
            })
            .collect();

        workers.into_iter().map(|h| h.join().unwrap()).sum()
        // errors vector automatically drops here when leaving scope
    };

    assert_eq!(total_processed, 1000);

    // Verify Arc is still alive and reference count is back to original
    assert_eq!(Arc::strong_count(&arc_error), 1);
}

/// Test backwards compatibility with existing error handling patterns
#[test]
fn test_backwards_compatibility() {
    // This test ensures that existing error handling code still works
    // with the new Arc<ClientError> approach

    let legacy_errors = vec![
        SignerError::NoSignerContext,
        SignerError::Configuration("Config error".to_string()),
        SignerError::Signing("Sign error".to_string()),
        SignerError::EvmTransaction("EVM error".to_string()),
        SignerError::SolanaTransaction(Arc::new(ClientError::new_with_request(
            ClientErrorKind::Custom("Solana error".to_string()),
            RpcRequest::GetAccountInfo,
        ))),
    ];

    for (i, error) in legacy_errors.iter().enumerate() {
        // Test that all error types can still be matched
        let error_type = match error {
            SignerError::NoSignerContext => "NoSignerContext",
            SignerError::Configuration(_) => "Configuration",
            SignerError::Signing(_) => "Signing",
            SignerError::EvmTransaction(_) => "EvmTransaction",
            SignerError::SolanaTransaction(_) => "SolanaTransaction",
            _ => "Other",
        };

        let expected_types = [
            "NoSignerContext",
            "Configuration",
            "Signing",
            "EvmTransaction",
            "SolanaTransaction",
        ];
        assert_eq!(error_type, expected_types[i]);

        // Test that Display and Debug still work
        let _display = error.to_string();
        let _debug = format!("{:?}", error);

        // Test that cloning still works
        let _cloned = error.clone();

        // Clone will be automatically dropped when it goes out of scope
    }
}
