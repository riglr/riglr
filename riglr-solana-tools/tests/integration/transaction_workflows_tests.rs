//! Integration tests for complete transaction workflows
//!
//! These tests focus on testing complete workflows including error handling,
//! signer integration, and proper tool error conversion.

use riglr_core::jobs::{Job, JobResult};
use riglr_core::signer::{SignerContext, TransactionSigner, SignerError};
use riglr_core::tool::{Tool, ToolWorker, ExecutionConfig};
use riglr_core::idempotency::InMemoryIdempotencyStore;
use riglr_solana_tools::error::{SolanaToolError, classify_transaction_error, TransactionErrorType};
use riglr_solana_tools::signer::local::LocalSolanaSigner;
use async_trait::async_trait;
use serde_json::json;
use solana_sdk::{
    signature::{Keypair, Signer},
    transaction::Transaction,
    system_instruction,
    pubkey::Pubkey,
};
use solana_client::rpc_client::RpcClient;
use std::sync::Arc;
use std::str::FromStr;

/// Mock Solana tool for testing transaction workflows
struct MockSolanaTransferTool;

#[async_trait]
impl Tool for MockSolanaTransferTool {
    async fn execute(
        &self,
        params: serde_json::Value,
    ) -> Result<JobResult, Box<dyn std::error::Error + Send + Sync>> {
        // Extract parameters
        let to_address = params["to"].as_str()
            .ok_or("Missing 'to' parameter")?;
        let amount = params["amount"].as_f64()
            .ok_or("Missing 'amount' parameter")?;

        // Validate address format
        if Pubkey::from_str(to_address).is_err() {
            return Ok(JobResult::permanent_failure(
                format!("Invalid Solana address: {}", to_address)
            ));
        }

        // Check if signer is available
        if !SignerContext::is_available().await {
            return Ok(JobResult::permanent_failure(
                "No signer context available for transfer"
            ));
        }

        let signer = match SignerContext::current().await {
            Ok(signer) => signer,
            Err(e) => return Ok(JobResult::permanent_failure(
                format!("Failed to get signer: {}", e)
            )),
        };

        // Mock transaction creation and signing
        let from_pubkey = match signer.address() {
            Some(addr) => match Pubkey::from_str(&addr) {
                Ok(pk) => pk,
                Err(_) => return Ok(JobResult::permanent_failure("Invalid signer address")),
            },
            None => return Ok(JobResult::permanent_failure("Signer has no address")),
        };

        let to_pubkey = Pubkey::from_str(to_address).unwrap(); // Already validated above

        // Create a system transfer instruction
        let instruction = system_instruction::transfer(
            &from_pubkey,
            &to_pubkey,
            (amount * 1_000_000_000.0) as u64, // Convert SOL to lamports
        );

        let mut transaction = Transaction::new_with_payer(&[instruction], Some(&from_pubkey));

        // Try to sign and send transaction
        match signer.sign_and_send_solana_transaction(&mut transaction).await {
            Ok(signature) => {
                Ok(JobResult::success_with_tx(
                    &json!({
                        "from": from_pubkey.to_string(),
                        "to": to_address,
                        "amount": amount,
                        "status": "submitted"
                    }),
                    signature,
                )?)
            }
            Err(SignerError::SolanaTransaction(e)) => {
                // Classify the error using our sophisticated error classification
                let client_error = e.as_ref();
                if let Some(client_err) = client_error.downcast_ref::<solana_client::client_error::ClientError>() {
                    let classified = classify_transaction_error(client_err);
                    match classified {
                        TransactionErrorType::Retryable(_) | TransactionErrorType::RateLimited(_) => {
                            Ok(JobResult::retriable_failure(format!("Transaction failed (retryable): {}", client_err)))
                        }
                        TransactionErrorType::Permanent(_) => {
                            Ok(JobResult::permanent_failure(format!("Transaction failed (permanent): {}", client_err)))
                        }
                        TransactionErrorType::Unknown(_) => {
                            Ok(JobResult::retriable_failure(format!("Transaction failed (unknown, treating as retryable): {}", client_err)))
                        }
                    }
                } else {
                    Ok(JobResult::retriable_failure(format!("Transaction failed: {}", e)))
                }
            }
            Err(e) => Ok(JobResult::permanent_failure(format!("Signer error: {}", e))),
        }
    }

    fn name(&self) -> &str {
        "solana_transfer"
    }
}

/// Mock signer that simulates various failure scenarios
struct MockFailingSigner {
    failure_mode: FailureMode,
    keypair: Keypair,
    rpc_url: String,
}

#[derive(Debug, Clone)]
enum FailureMode {
    Success,
    NetworkError,
    InsufficientFunds,
    InvalidSignature,
    RateLimited,
    UnknownError,
}

impl MockFailingSigner {
    fn new(failure_mode: FailureMode) -> Self {
        Self {
            failure_mode,
            keypair: Keypair::new(),
            rpc_url: "https://api.devnet.solana.com".to_string(),
        }
    }
}

#[async_trait]
impl TransactionSigner for MockFailingSigner {
    fn address(&self) -> Option<String> {
        Some(self.keypair.pubkey().to_string())
    }

    fn pubkey(&self) -> Option<String> {
        Some(self.keypair.pubkey().to_string())
    }

    async fn sign_and_send_solana_transaction(
        &self,
        _tx: &mut Transaction,
    ) -> Result<String, SignerError> {
        match self.failure_mode {
            FailureMode::Success => Ok("mock_tx_signature_12345".to_string()),
            FailureMode::NetworkError => {
                let io_error = std::io::Error::new(
                    std::io::ErrorKind::ConnectionRefused,
                    "Connection refused"
                );
                let client_error = solana_client::client_error::ClientError::new_with_request(
                    solana_client::client_error::ClientErrorKind::Io(io_error),
                    solana_client::rpc_request::RpcRequest::SendTransaction,
                );
                Err(SignerError::SolanaTransaction(Box::new(client_error)))
            }
            FailureMode::InsufficientFunds => {
                let client_error = solana_client::client_error::ClientError::new_with_request(
                    solana_client::client_error::ClientErrorKind::Custom("InsufficientFundsForRent".to_string()),
                    solana_client::rpc_request::RpcRequest::SendTransaction,
                );
                Err(SignerError::SolanaTransaction(Box::new(client_error)))
            }
            FailureMode::InvalidSignature => {
                let client_error = solana_client::client_error::ClientError::new_with_request(
                    solana_client::client_error::ClientErrorKind::Custom("InvalidSignature".to_string()),
                    solana_client::rpc_request::RpcRequest::SendTransaction,
                );
                Err(SignerError::SolanaTransaction(Box::new(client_error)))
            }
            FailureMode::RateLimited => {
                let rpc_error = solana_client::rpc_request::RpcError::RpcResponseError {
                    code: 429,
                    message: "Too Many Requests".to_string(),
                    data: solana_client::rpc_request::RpcResponseErrorData::Empty,
                };
                let client_error = solana_client::client_error::ClientError::new_with_request(
                    solana_client::client_error::ClientErrorKind::RpcError(rpc_error),
                    solana_client::rpc_request::RpcRequest::SendTransaction,
                );
                Err(SignerError::SolanaTransaction(Box::new(client_error)))
            }
            FailureMode::UnknownError => {
                Err(SignerError::Generic("Unknown error occurred".to_string()))
            }
        }
    }

    async fn sign_and_send_evm_transaction(
        &self,
        _tx: alloy::rpc::types::TransactionRequest,
    ) -> Result<String, SignerError> {
        Err(SignerError::Configuration(
            "MockFailingSigner does not support EVM transactions".to_string()
        ))
    }

    fn solana_client(&self) -> Option<Arc<solana_client::rpc_client::RpcClient>> {
        Some(Arc::new(RpcClient::new(self.rpc_url.clone())))
    }

    fn evm_client(&self) -> Result<Arc<dyn std::any::Any + Send + Sync>, SignerError> {
        Err(SignerError::Configuration(
            "MockFailingSigner does not support EVM clients".to_string()
        ))
    }
}

#[tokio::test]
async fn test_successful_transfer_workflow() {
    let worker = ToolWorker::<InMemoryIdempotencyStore>::new(ExecutionConfig::default());
    worker.register_tool(Arc::new(MockSolanaTransferTool)).await;

    // Set up successful signer context
    let signer = Arc::new(MockFailingSigner::new(FailureMode::Success));
    SignerContext::set_current(signer).await;

    let job = Job::new(
        "solana_transfer",
        &json!({
            "to": "11111111111111111111111111111112", // Valid address
            "amount": 0.001
        }),
        3,
    ).unwrap();

    let result = worker.process_job(job).await.unwrap();
    assert!(result.is_success());

    match result {
        JobResult::Success { value, tx_hash } => {
            assert!(tx_hash.is_some());
            assert_eq!(tx_hash.unwrap(), "mock_tx_signature_12345");
            assert_eq!(value["status"], "submitted");
        }
        _ => panic!("Expected successful result"),
    }
}

#[tokio::test]
async fn test_network_error_retry_workflow() {
    let worker = ToolWorker::<InMemoryIdempotencyStore>::new(ExecutionConfig::default());
    worker.register_tool(Arc::new(MockSolanaTransferTool)).await;

    // Set up failing signer context (network error - should be retriable)
    let signer = Arc::new(MockFailingSigner::new(FailureMode::NetworkError));
    SignerContext::set_current(signer).await;

    let job = Job::new(
        "solana_transfer",
        &json!({
            "to": "11111111111111111111111111111112",
            "amount": 0.001
        }),
        3,
    ).unwrap();

    let result = worker.process_job(job).await.unwrap();
    assert!(!result.is_success());
    assert!(result.is_retriable()); // Network errors should be retriable
}

#[tokio::test]
async fn test_insufficient_funds_permanent_failure() {
    let worker = ToolWorker::<InMemoryIdempotencyStore>::new(ExecutionConfig::default());
    worker.register_tool(Arc::new(MockSolanaTransferTool)).await;

    // Set up failing signer context (insufficient funds - should be permanent)
    let signer = Arc::new(MockFailingSigner::new(FailureMode::InsufficientFunds));
    SignerContext::set_current(signer).await;

    let job = Job::new(
        "solana_transfer",
        &json!({
            "to": "11111111111111111111111111111112",
            "amount": 0.001
        }),
        3,
    ).unwrap();

    let result = worker.process_job(job).await.unwrap();
    assert!(!result.is_success());
    assert!(!result.is_retriable()); // Insufficient funds should be permanent
}

#[tokio::test]
async fn test_invalid_signature_permanent_failure() {
    let worker = ToolWorker::<InMemoryIdempotencyStore>::new(ExecutionConfig::default());
    worker.register_tool(Arc::new(MockSolanaTransferTool)).await;

    // Set up failing signer context (invalid signature - should be permanent)
    let signer = Arc::new(MockFailingSigner::new(FailureMode::InvalidSignature));
    SignerContext::set_current(signer).await;

    let job = Job::new(
        "solana_transfer",
        &json!({
            "to": "11111111111111111111111111111112",
            "amount": 0.001
        }),
        3,
    ).unwrap();

    let result = worker.process_job(job).await.unwrap();
    assert!(!result.is_success());
    assert!(!result.is_retriable()); // Invalid signature should be permanent
}

#[tokio::test]
async fn test_rate_limited_retry_workflow() {
    let worker = ToolWorker::<InMemoryIdempotencyStore>::new(ExecutionConfig::default());
    worker.register_tool(Arc::new(MockSolanaTransferTool)).await;

    // Set up failing signer context (rate limited - should be retriable)
    let signer = Arc::new(MockFailingSigner::new(FailureMode::RateLimited));
    SignerContext::set_current(signer).await;

    let job = Job::new(
        "solana_transfer",
        &json!({
            "to": "11111111111111111111111111111112",
            "amount": 0.001
        }),
        3,
    ).unwrap();

    let result = worker.process_job(job).await.unwrap();
    assert!(!result.is_success());
    assert!(result.is_retriable()); // Rate limiting should be retriable
}

#[tokio::test]
async fn test_unknown_error_retry_workflow() {
    let worker = ToolWorker::<InMemoryIdempotencyStore>::new(ExecutionConfig::default());
    worker.register_tool(Arc::new(MockSolanaTransferTool)).await;

    // Set up failing signer context (unknown error - should default to retriable)
    let signer = Arc::new(MockFailingSigner::new(FailureMode::UnknownError));
    SignerContext::set_current(signer).await;

    let job = Job::new(
        "solana_transfer",
        &json!({
            "to": "11111111111111111111111111111112",
            "amount": 0.001
        }),
        3,
    ).unwrap();

    let result = worker.process_job(job).await.unwrap();
    assert!(!result.is_success());
    // Unknown errors are treated as permanent in this specific case (SignerError::Generic)
    assert!(!result.is_retriable());
}

#[tokio::test]
async fn test_invalid_address_validation() {
    let worker = ToolWorker::<InMemoryIdempotencyStore>::new(ExecutionConfig::default());
    worker.register_tool(Arc::new(MockSolanaTransferTool)).await;

    // Set up successful signer context
    let signer = Arc::new(MockFailingSigner::new(FailureMode::Success));
    SignerContext::set_current(signer).await;

    let job = Job::new(
        "solana_transfer",
        &json!({
            "to": "invalid_address", // Invalid Solana address
            "amount": 0.001
        }),
        3,
    ).unwrap();

    let result = worker.process_job(job).await.unwrap();
    assert!(!result.is_success());
    assert!(!result.is_retriable()); // Invalid address should be permanent

    match result {
        JobResult::Failure { error, .. } => {
            assert!(error.contains("Invalid Solana address"));
        }
        _ => panic!("Expected failure for invalid address"),
    }
}

#[tokio::test]
async fn test_missing_parameters() {
    let worker = ToolWorker::<InMemoryIdempotencyStore>::new(ExecutionConfig::default());
    worker.register_tool(Arc::new(MockSolanaTransferTool)).await;

    // Set up successful signer context
    let signer = Arc::new(MockFailingSigner::new(FailureMode::Success));
    SignerContext::set_current(signer).await;

    // Test missing 'to' parameter
    let job1 = Job::new(
        "solana_transfer",
        &json!({
            "amount": 0.001
            // Missing 'to' parameter
        }),
        3,
    ).unwrap();

    let result = worker.process_job(job1).await;
    assert!(result.is_err()); // Should return Err for missing parameters

    // Test missing 'amount' parameter
    let job2 = Job::new(
        "solana_transfer",
        &json!({
            "to": "11111111111111111111111111111112"
            // Missing 'amount' parameter
        }),
        3,
    ).unwrap();

    let result = worker.process_job(job2).await;
    assert!(result.is_err()); // Should return Err for missing parameters
}

#[tokio::test]
async fn test_no_signer_context() {
    let worker = ToolWorker::<InMemoryIdempotencyStore>::new(ExecutionConfig::default());
    worker.register_tool(Arc::new(MockSolanaTransferTool)).await;

    // Clear any existing signer context
    SignerContext::clear().await;

    let job = Job::new(
        "solana_transfer",
        &json!({
            "to": "11111111111111111111111111111112",
            "amount": 0.001
        }),
        3,
    ).unwrap();

    let result = worker.process_job(job).await.unwrap();
    assert!(!result.is_success());
    assert!(!result.is_retriable()); // No signer context should be permanent

    match result {
        JobResult::Failure { error, .. } => {
            assert!(error.contains("No signer context available"));
        }
        _ => panic!("Expected failure for no signer context"),
    }
}

#[tokio::test]
async fn test_solana_tool_error_conversion_to_tool_error() {
    // Test various SolanaToolError conversions to riglr_core::error::ToolError

    // RPC errors
    let rpc_error = SolanaToolError::Rpc("Connection timeout".to_string());
    let tool_error: riglr_core::error::ToolError = rpc_error.into();
    assert!(tool_error.is_retriable());

    let rate_limit_rpc_error = SolanaToolError::Rpc("429 rate limit exceeded".to_string());
    let tool_error: riglr_core::error::ToolError = rate_limit_rpc_error.into();
    assert!(tool_error.is_rate_limited());

    // HTTP errors
    let timeout_http_error = reqwest::Error::from(reqwest::ErrorKind::Timeout);
    let solana_error = SolanaToolError::Http(timeout_http_error);
    let tool_error: riglr_core::error::ToolError = solana_error.into();
    assert!(tool_error.is_retriable());

    // Address validation errors
    let addr_error = SolanaToolError::InvalidAddress("Invalid base58".to_string());
    let tool_error: riglr_core::error::ToolError = addr_error.into();
    assert!(!tool_error.is_retriable()); // Should be permanent

    // Key validation errors
    let key_error = SolanaToolError::InvalidKey("Invalid keypair".to_string());
    let tool_error: riglr_core::error::ToolError = key_error.into();
    assert!(!tool_error.is_retriable()); // Should be permanent

    // Transaction errors
    let insufficient_tx_error = SolanaToolError::Transaction("insufficient funds".to_string());
    let tool_error: riglr_core::error::ToolError = insufficient_tx_error.into();
    assert!(!tool_error.is_retriable()); // Should be permanent

    let network_tx_error = SolanaToolError::Transaction("network timeout".to_string());
    let tool_error: riglr_core::error::ToolError = network_tx_error.into();
    assert!(tool_error.is_retriable()); // Should be retriable

    // Serialization errors
    let json_error = serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err();
    let serialization_error = SolanaToolError::Serialization(json_error);
    let tool_error: riglr_core::error::ToolError = serialization_error.into();
    assert!(!tool_error.is_retriable()); // Should be permanent

    // Core errors
    let core_error = riglr_core::error::CoreError::Generic("Core failure".to_string());
    let solana_core_error = SolanaToolError::Core(core_error);
    let tool_error: riglr_core::error::ToolError = solana_core_error.into();
    assert!(tool_error.is_retriable()); // Core errors inherit their retriable nature

    // Generic errors
    let generic_error = SolanaToolError::Generic("Generic failure".to_string());
    let tool_error: riglr_core::error::ToolError = generic_error.into();
    assert!(tool_error.is_retriable()); // Generic errors default to retriable
}

#[tokio::test]
async fn test_local_solana_signer_integration() {
    // Test LocalSolanaSigner integration
    let keypair = Keypair::new();
    let signer = LocalSolanaSigner::new(keypair, "https://api.devnet.solana.com".to_string());

    // Test basic properties
    assert!(signer.address().is_some());
    assert!(signer.pubkey().is_some());
    assert_eq!(signer.rpc_url(), "https://api.devnet.solana.com");

    // Test EVM methods (should fail)
    let evm_tx = alloy::rpc::types::TransactionRequest::default();
    let result = signer.sign_and_send_evm_transaction(evm_tx).await;
    assert!(result.is_err());

    let evm_client_result = signer.evm_client();
    assert!(evm_client_result.is_err());

    // Test Solana client access
    let _solana_client = signer.solana_client();

    // Test debug formatting (should not expose private keys)
    let debug_str = format!("{:?}", signer);
    assert!(debug_str.contains("LocalSolanaSigner"));
    assert!(debug_str.contains("pubkey"));
    assert!(!debug_str.contains("keypair")); // Should not expose private key
}

#[tokio::test]
async fn test_local_signer_from_seed_phrase() {
    // Test creating LocalSolanaSigner from seed phrase
    let seed_phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    let result = LocalSolanaSigner::from_seed_phrase(seed_phrase, "https://api.devnet.solana.com".to_string());
    assert!(result.is_ok());

    let signer = result.unwrap();
    assert!(signer.address().is_some());

    // Test invalid seed phrase
    let invalid_seed = "invalid seed phrase";
    let result = LocalSolanaSigner::from_seed_phrase(invalid_seed, "https://api.devnet.solana.com".to_string());
    assert!(result.is_err());

    match result {
        Err(SignerError::Configuration(msg)) => {
            assert!(msg.contains("Invalid seed phrase"));
        }
        _ => panic!("Expected configuration error for invalid seed phrase"),
    }
}

#[tokio::test]
async fn test_idempotent_transaction_workflow() {
    let worker = ToolWorker::<InMemoryIdempotencyStore>::new(ExecutionConfig::default());
    worker.register_tool(Arc::new(MockSolanaTransferTool)).await;

    let store = Arc::new(InMemoryIdempotencyStore::new());
    let worker = worker.with_idempotency_store(store);

    // Set up successful signer context
    let signer = Arc::new(MockFailingSigner::new(FailureMode::Success));
    SignerContext::set_current(signer).await;

    let job = Job::new_idempotent(
        "solana_transfer",
        &json!({
            "to": "11111111111111111111111111111112",
            "amount": 0.001
        }),
        3,
        "transfer_test_key_123",
    ).unwrap();

    // First execution
    let result1 = worker.process_job(job.clone()).await.unwrap();
    assert!(result1.is_success());

    // Second execution should return cached result
    let result2 = worker.process_job(job.clone()).await.unwrap();
    assert!(result2.is_success());

    // Both should have the same transaction hash
    match (&result1, &result2) {
        (JobResult::Success { tx_hash: Some(hash1), .. }, JobResult::Success { tx_hash: Some(hash2), .. }) => {
            assert_eq!(hash1, hash2);
        }
        _ => panic!("Expected successful results with transaction hashes"),
    }
}

#[tokio::test]
async fn test_comprehensive_error_scenarios() {
    let worker = ToolWorker::<InMemoryIdempotencyStore>::new(ExecutionConfig::default());
    worker.register_tool(Arc::new(MockSolanaTransferTool)).await;

    // Test all failure modes with their expected classifications
    let test_cases = vec![
        (FailureMode::NetworkError, true),      // Should be retriable
        (FailureMode::InsufficientFunds, false), // Should be permanent
        (FailureMode::InvalidSignature, false),  // Should be permanent
        (FailureMode::RateLimited, true),        // Should be retriable
        (FailureMode::UnknownError, false),      // Should be permanent (SignerError::Generic)
    ];

    for (failure_mode, should_be_retriable) in test_cases {
        let signer = Arc::new(MockFailingSigner::new(failure_mode.clone()));
        SignerContext::set_current(signer).await;

        let job = Job::new(
            "solana_transfer",
            &json!({
                "to": "11111111111111111111111111111112",
                "amount": 0.001
            }),
            3,
        ).unwrap();

        let result = worker.process_job(job).await.unwrap();
        assert!(!result.is_success(), "Expected failure for {:?}", failure_mode);
        assert_eq!(
            result.is_retriable(),
            should_be_retriable,
            "Incorrect retriability for {:?}",
            failure_mode
        );
    }
}

/// Mock balance checker tool for testing different workflows
struct MockBalanceCheckerTool;

#[async_trait]
impl Tool for MockBalanceCheckerTool {
    async fn execute(
        &self,
        params: serde_json::Value,
    ) -> Result<JobResult, Box<dyn std::error::Error + Send + Sync>> {
        let address = params["address"].as_str()
            .ok_or("Missing 'address' parameter")?;

        // Validate address format
        if Pubkey::from_str(address).is_err() {
            return Ok(JobResult::permanent_failure(
                format!("Invalid Solana address: {}", address)
            ));
        }

        // Mock balance check - always return success for valid addresses
        Ok(JobResult::success(&json!({
            "address": address,
            "balance": 1.5,
            "unit": "SOL"
        }))?)
    }

    fn name(&self) -> &str {
        "solana_balance_check"
    }
}

#[tokio::test]
async fn test_read_only_tool_workflow() {
    // Test tools that don't require signer context
    let worker = ToolWorker::<InMemoryIdempotencyStore>::new(ExecutionConfig::default());
    worker.register_tool(Arc::new(MockBalanceCheckerTool)).await;

    // Clear signer context
    SignerContext::clear().await;

    let job = Job::new(
        "solana_balance_check",
        &json!({
            "address": "11111111111111111111111111111112"
        }),
        0,
    ).unwrap();

    let result = worker.process_job(job).await.unwrap();
    assert!(result.is_success());

    match result {
        JobResult::Success { value, .. } => {
            assert_eq!(value["address"], "11111111111111111111111111111112");
            assert_eq!(value["balance"], 1.5);
            assert_eq!(value["unit"], "SOL");
        }
        _ => panic!("Expected successful balance check"),
    }
}

#[tokio::test]
#[ignore] // Long-running concurrent test
async fn test_concurrent_transaction_workflows() {
    let worker = Arc::new(ToolWorker::<InMemoryIdempotencyStore>::new(ExecutionConfig::default()));
    worker.register_tool(Arc::new(MockSolanaTransferTool)).await;

    // Set up successful signer context
    let signer = Arc::new(MockFailingSigner::new(FailureMode::Success));
    SignerContext::set_current(signer).await;

    // Create multiple concurrent transfer jobs
    let mut handles = vec![];
    for i in 0..10 {
        let worker_clone = worker.clone();
        let handle = tokio::spawn(async move {
            let job = Job::new(
                "solana_transfer",
                &json!({
                    "to": "11111111111111111111111111111112",
                    "amount": 0.001 * (i + 1) as f64
                }),
                3,
            ).unwrap();

            worker_clone.process_job(job).await.unwrap()
        });
        handles.push(handle);
    }

    // Wait for all transfers to complete
    let mut success_count = 0;
    for handle in handles {
        let result = handle.await.unwrap();
        if result.is_success() {
            success_count += 1;
        }
    }

    assert_eq!(success_count, 10, "All concurrent transfers should succeed");
}