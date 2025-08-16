//! Integration tests for riglr-core components
//!
//! These tests validate the complete functionality of signers, transaction processing,
//! and configuration management in realistic scenarios.

use alloy::primitives::U256;
use async_trait::async_trait;
use riglr_core::{
    config::{EvmNetworkConfig, SolanaNetworkConfig},
    error::ToolError,
    idempotency::InMemoryIdempotencyStore,
    signer::{
        evm::LocalEvmSigner, solana::LocalSolanaSigner, SignerContext, SignerError,
        TransactionSigner,
    },
    transactions::{
        evm::GasConfig,
        solana::{PriorityFeeConfig, SolanaTransactionProcessor},
        RetryConfig, TransactionProcessor,
    },
    ExecutionConfig, Job, JobResult, Tool, ToolWorker,
};

const TEST_EVM_RPC_URL: &str = "TEST_EVM_RPC_URL";
const TEST_SOLANA_RPC_URL: &str = "TEST_SOLANA_RPC_URL";
use std::sync::Arc;
use std::time::Duration;

/// Test helper to create a mock EVM signer
async fn create_test_evm_signer() -> Arc<dyn TransactionSigner> {
    // Use a well-known test private key (from Hardhat/Anvil)
    let private_key = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
    let rpc_url =
        std::env::var(TEST_EVM_RPC_URL).unwrap_or_else(|_| "http://localhost:8545".to_string());

    let network_config = EvmNetworkConfig {
        name: "ethereum_test".to_string(),
        chain_id: 1,
        rpc_url,
        explorer_url: Some("https://etherscan.io".to_string()),
    };

    Arc::new(
        LocalEvmSigner::new(private_key.to_string(), network_config)
            .expect("Failed to create EVM signer"),
    )
}

/// Test helper to create a mock Solana signer
async fn create_test_solana_signer() -> Arc<dyn TransactionSigner> {
    use solana_sdk::signer::keypair::Keypair;

    let keypair = Keypair::new();
    let rpc_url = std::env::var(TEST_SOLANA_RPC_URL)
        .unwrap_or_else(|_| "https://api.devnet.solana.com".to_string());

    let network_config = SolanaNetworkConfig {
        name: "devnet_test".to_string(),
        rpc_url,
        explorer_url: Some("https://explorer.solana.com".to_string()),
    };

    Arc::new(LocalSolanaSigner::from_keypair(keypair, network_config))
}

#[tokio::test]
async fn test_evm_signer_context() -> anyhow::Result<()> {
    let signer = create_test_evm_signer().await;

    // Test setting and retrieving signer context
    let result = SignerContext::with_signer(signer.clone(), async {
        let current = SignerContext::current().await?;

        // Verify we can get the address
        let address = current.address();
        assert!(address.is_some());

        // EVM signer should provide an address
        assert!(!address.unwrap().is_empty());

        Ok::<String, SignerError>("success".to_string())
    })
    .await;

    assert!(result.is_ok());

    // Verify context is cleared after scope
    let current = SignerContext::current().await;
    assert!(current.is_err());

    Ok(())
}

#[tokio::test]
async fn test_solana_signer_context() -> anyhow::Result<()> {
    let signer = create_test_solana_signer().await;

    let result = SignerContext::with_signer(signer.clone(), async {
        let current = SignerContext::current().await?;

        // Verify we can get the address
        let address = current.address();
        assert!(address.is_some());

        // For Solana, we should have Solana pubkey
        assert!(current.pubkey().is_some());

        Ok::<String, SignerError>("success".to_string())
    })
    .await;

    assert!(result.is_ok());
    Ok(())
}

#[tokio::test]
async fn test_nested_signer_contexts() -> anyhow::Result<()> {
    let evm_signer = create_test_evm_signer().await;
    let solana_signer = create_test_solana_signer().await;

    // Test nested contexts
    SignerContext::with_signer(evm_signer.clone(), async {
        let outer = SignerContext::current().await?;
        // EVM signer should have an address
        assert!(outer.address().is_some());

        // Inner context should override
        SignerContext::with_signer(solana_signer.clone(), async {
            let inner = SignerContext::current().await?;
            assert!(inner.pubkey().is_some());
            Ok::<String, SignerError>("inner".to_string())
        })
        .await?;

        // Outer context should be restored
        let restored = SignerContext::current().await?;
        assert!(restored.address().is_some());

        Ok::<String, SignerError>("outer".to_string())
    })
    .await?;

    Ok(())
}

#[tokio::test]
async fn test_transaction_retry_logic() -> anyhow::Result<()> {
    use riglr_core::transactions::GenericTransactionProcessor;

    let processor = GenericTransactionProcessor;
    let config = RetryConfig {
        max_attempts: 3,
        initial_delay: Duration::from_millis(100),
        max_delay: Duration::from_secs(1),
        backoff_multiplier: 2.0,
    };

    // Test successful operation
    let result = processor
        .process_with_retry(|| async { Ok::<_, ToolError>("success") }, config.clone())
        .await;
    assert_eq!(result?, "success");

    // Test retriable failure that eventually succeeds
    let counter = Arc::new(std::sync::Mutex::new(0));
    let counter_clone = counter.clone();

    let result = processor
        .process_with_retry(
            move || {
                let counter = counter_clone.clone();
                async move {
                    let mut count = counter.lock().unwrap();
                    *count += 1;
                    if *count < 3 {
                        Err(ToolError::retriable_string("temporary failure"))
                    } else {
                        Ok("success after retries")
                    }
                }
            },
            config.clone(),
        )
        .await;

    assert_eq!(result?, "success after retries");
    assert_eq!(*counter.lock().unwrap(), 3);

    // Test permanent failure (should not retry)
    let counter = Arc::new(std::sync::Mutex::new(0));
    let counter_clone = counter.clone();

    let result = processor
        .process_with_retry(
            move || {
                let counter = counter_clone.clone();
                async move {
                    let mut count = counter.lock().unwrap();
                    *count += 1;
                    Err::<String, _>(ToolError::permanent_string("permanent failure"))
                }
            },
            config.clone(),
        )
        .await;

    assert!(result.is_err());
    assert_eq!(*counter.lock().unwrap(), 1); // Should only try once

    Ok(())
}

#[tokio::test]
async fn test_solana_priority_fees() -> anyhow::Result<()> {
    use solana_client::rpc_client::RpcClient;
    use solana_sdk::pubkey::Pubkey;
    #[allow(deprecated)]
    use solana_sdk::system_instruction;

    // This test verifies priority fee instructions are added correctly
    let client = Arc::new(RpcClient::new("https://api.devnet.solana.com"));
    let priority_config = PriorityFeeConfig {
        enabled: true,
        microlamports_per_cu: 5000,
        additional_compute_units: Some(200_000),
    };

    let processor = SolanaTransactionProcessor::new(client, priority_config);

    let mut instructions = vec![system_instruction::transfer(
        &Pubkey::new_unique(),
        &Pubkey::new_unique(),
        1000,
    )];

    let original_len = instructions.len();
    processor.add_priority_fee_instructions(&mut instructions);

    // Should have added 2 compute budget instructions at the beginning
    assert_eq!(instructions.len(), original_len + 2);

    Ok(())
}

#[tokio::test]
async fn test_evm_gas_estimation() -> anyhow::Result<()> {
    // This test would require a real provider connection
    // For unit testing, we're mainly verifying the logic structure

    let gas_config = GasConfig {
        use_eip1559: true,
        gas_price_multiplier: 1.2,
        max_gas_price: Some(U256::from(100_000_000_000u64)), // 100 gwei max
        max_priority_fee: Some(U256::from(2_000_000_000u64)), // 2 gwei
    };

    // Verify default configuration
    assert!(gas_config.use_eip1559);
    assert_eq!(gas_config.gas_price_multiplier, 1.2);

    Ok(())
}

// Config module tests removed - config module was moved to riglr-showcase
// These tests are no longer relevant as config functionality is application-specific

#[tokio::test]
async fn test_tool_worker_with_signers() -> anyhow::Result<()> {
    // Create a tool that uses the signer context
    struct SignerTestTool;

    #[async_trait]
    impl Tool for SignerTestTool {
        async fn execute(
            &self,
            _params: serde_json::Value,
        ) -> Result<JobResult, Box<dyn std::error::Error + Send + Sync>> {
            // Try to get current signer
            match SignerContext::current().await {
                Ok(signer) => {
                    let address = signer.address().unwrap_or_else(|| "no address".to_string());
                    Ok(JobResult::success(&format!("Signer address: {}", address))?)
                }
                Err(_) => Ok(JobResult::success(&"No signer context")?),
            }
        }

        fn name(&self) -> &str {
            "signer_test"
        }

        fn description(&self) -> &str {
            ""
        }
    }

    let worker = ToolWorker::<InMemoryIdempotencyStore>::new(ExecutionConfig::default());

    worker.register_tool(Arc::new(SignerTestTool)).await;

    // Execute without signer context
    let job = Job::new("signer_test", &serde_json::json!({}), 3)?;
    let result = worker
        .process_job(job.clone())
        .await
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;

    match result {
        JobResult::Success { value, .. } => {
            assert_eq!(value.as_str().unwrap(), "No signer context");
        }
        _ => panic!("Expected success"),
    }

    // Execute with signer context
    let signer = create_test_evm_signer().await;
    let result = SignerContext::with_signer(signer, async {
        worker
            .process_job(job)
            .await
            .map_err(|e| SignerError::TransactionFailed(format!("Process failed: {}", e)))
    })
    .await?;

    match result {
        JobResult::Success { value, .. } => {
            assert!(value.as_str().unwrap().starts_with("Signer address: "));
        }
        _ => panic!("Expected success"),
    }

    Ok(())
}

#[tokio::test]
async fn test_end_to_end_workflow() -> anyhow::Result<()> {
    // This test simulates a complete workflow without config loading

    // Create execution config directly
    let exec_config = ExecutionConfig {
        max_retries: 3,
        initial_retry_delay: Duration::from_millis(1000),
        max_retry_delay: Duration::from_secs(30),
        default_timeout: Duration::from_secs(60),
        max_concurrency: 10,
        enable_idempotency: true,
        idempotency_ttl: Duration::from_secs(3600),
    };

    // Create worker
    let worker = ToolWorker::<InMemoryIdempotencyStore>::new(exec_config);

    // Define a workflow tool
    struct WorkflowTool;

    #[async_trait]
    impl Tool for WorkflowTool {
        async fn execute(
            &self,
            params: serde_json::Value,
        ) -> Result<JobResult, Box<dyn std::error::Error + Send + Sync>> {
            let step = params["step"].as_u64().unwrap_or(0);

            match step {
                1 => Ok(JobResult::success(&"Step 1 completed")?),
                2 => Ok(JobResult::success(&"Step 2 completed")?),
                3 => Ok(JobResult::success(&"Workflow completed")?),
                _ => Err("Invalid step".into()),
            }
        }

        fn name(&self) -> &str {
            "workflow"
        }

        fn description(&self) -> &str {
            ""
        }
    }

    worker.register_tool(Arc::new(WorkflowTool)).await;

    // Execute workflow steps
    for step in 1..=3 {
        let job = Job::new("workflow", &serde_json::json!({"step": step}), 3)?;

        let result = worker
            .process_job(job)
            .await
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;

        match result {
            JobResult::Success { value, .. } => {
                let message = value.as_str().unwrap();
                if step == 3 {
                    assert_eq!(message, "Workflow completed");
                } else {
                    assert!(message.contains(&format!("Step {}", step)));
                }
            }
            _ => panic!("Expected success at step {}", step),
        }
    }

    Ok(())
}

#[tokio::test]
async fn test_error_classification() -> anyhow::Result<()> {
    // Test different error types and their classification

    // Retriable errors
    let network_error = ToolError::retriable_string("Network timeout");
    assert!(network_error.is_retriable());
    assert!(!network_error.is_rate_limited());

    // Rate limited errors
    let rate_error = ToolError::rate_limited_string("Too many requests");
    assert!(rate_error.is_retriable());
    assert!(rate_error.is_rate_limited());

    // Permanent errors
    let perm_error = ToolError::permanent_string("Invalid private key");
    assert!(!perm_error.is_retriable());
    assert!(!perm_error.is_rate_limited());

    // Test error context preservation
    let source_error = std::io::Error::new(std::io::ErrorKind::TimedOut, "Connection timeout");
    let wrapped = ToolError::Retriable {
        source: Box::new(source_error),
        context: "Failed to connect to RPC".to_string(),
    };
    assert!(wrapped.is_retriable());

    Ok(())
}
