use riglr_core::{
    signer::{SignerContext, TransactionSigner, SignerError, EvmClient},
    error::ToolError,
    util::{must_get_env, get_env_or_default},
};
use std::sync::Arc;
use tokio::task;
use solana_sdk::transaction::Transaction;
use futures::future;

/// Mock signer for testing signer context isolation
struct MockSigner {
    id: String,
    pubkey: Option<String>,
    address: Option<String>,
}

impl MockSigner {
    fn new_solana(id: String, pubkey: String) -> Self {
        Self {
            id,
            pubkey: Some(pubkey),
            address: None,
        }
    }

    fn new_evm(id: String, address: String) -> Self {
        Self {
            id,
            pubkey: None,
            address: Some(address),
        }
    }
}

#[async_trait::async_trait]
impl TransactionSigner for MockSigner {
    fn pubkey(&self) -> Option<String> {
        self.pubkey.clone()
    }

    fn address(&self) -> Option<String> {
        self.address.clone()
    }

    async fn sign_and_send_solana_transaction(&self, _tx: &mut Transaction) -> Result<String, SignerError> {
        Ok(format!("solana_tx_{}", self.id))
    }

    async fn sign_and_send_evm_transaction(&self, _tx: alloy::rpc::types::TransactionRequest) -> Result<String, SignerError> {
        Ok(format!("evm_tx_{}", self.id))
    }

    fn solana_client(&self) -> Option<Arc<solana_client::rpc_client::RpcClient>> {
        Some(Arc::new(solana_client::rpc_client::RpcClient::new("https://api.devnet.solana.com".to_string())))
    }

    fn evm_client(&self) -> Result<Arc<dyn EvmClient>, SignerError> {
        Err(SignerError::UnsupportedOperation("Mock signer does not provide EVM client".to_string()))
    }
}

impl std::fmt::Debug for MockSigner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MockSigner")
            .field("id", &self.id)
            .field("pubkey", &self.pubkey)
            .field("address", &self.address)
            .finish()
    }
}

#[tokio::test]
async fn test_signer_context_isolation_between_tasks() {
    // Create different signers for different tasks
    let signer1 = MockSigner::new_solana("task1".to_string(), "11111111111111111111111111111112".to_string());
    let signer2 = MockSigner::new_solana("task2".to_string(), "11111111111111111111111111111113".to_string());

    // Spawn concurrent tasks with different signer contexts
    let handle1 = task::spawn(async move {
        SignerContext::with_signer(Arc::new(signer1), async {
            // Wait a bit to ensure concurrency
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            
            let current = SignerContext::current().await?;
            Ok::<String, SignerError>(current.pubkey().unwrap())
        }).await
    });

    let handle2 = task::spawn(async move {
        SignerContext::with_signer(Arc::new(signer2), async {
            // Wait a bit to ensure concurrency
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            
            let current = SignerContext::current().await?;
            Ok::<String, SignerError>(current.pubkey().unwrap())
        }).await
    });

    // Verify each task gets its own signer context
    let pubkey1 = handle1.await.unwrap().unwrap();
    let pubkey2 = handle2.await.unwrap().unwrap();

    assert_eq!(pubkey1, "11111111111111111111111111111112");
    assert_eq!(pubkey2, "11111111111111111111111111111113");
    assert_ne!(pubkey1, pubkey2);
}

#[tokio::test]
async fn test_cross_chain_signer_context() {
    let solana_pubkey = "11111111111111111111111111111112";
    let evm_address = "0x742d35Cc2F5f8a89A0D2EAd5a53c97c49444E34F";

    // Test Solana signer context
    let solana_signer = MockSigner::new_solana("solana".to_string(), solana_pubkey.to_string());
    
    SignerContext::with_signer(Arc::new(solana_signer), async {
        let current = SignerContext::current().await?;
        assert!(current.pubkey().is_some());
        assert!(current.address().is_none());
        assert_eq!(current.pubkey().unwrap(), solana_pubkey);
        Ok::<(), SignerError>(())
    }).await.unwrap();

    // Test EVM signer context
    let evm_signer = MockSigner::new_evm("evm".to_string(), evm_address.to_string());
    
    SignerContext::with_signer(Arc::new(evm_signer), async {
        let current = SignerContext::current().await?;
        assert!(current.pubkey().is_none());
        assert!(current.address().is_some());
        assert_eq!(current.address().unwrap(), evm_address);
        Ok::<(), SignerError>(())
    }).await.unwrap();
}

#[tokio::test]
async fn test_tool_error_classification() {
    // Test retriable error
    let retriable = ToolError::retriable("Network timeout");
    match retriable {
        ToolError::Retriable { context, .. } => assert_eq!(context, "Network timeout"),
        _ => panic!("Expected retriable error"),
    }

    // Test permanent error
    let permanent = ToolError::permanent("Invalid signature");
    match permanent {
        ToolError::Permanent { context, .. } => assert_eq!(context, "Invalid signature"),
        _ => panic!("Expected permanent error"),
    }

    // Test rate limited error
    let rate_limited = ToolError::rate_limited("API limit exceeded");
    match rate_limited {
        ToolError::RateLimited { context, .. } => assert_eq!(context, "API limit exceeded"),
        _ => panic!("Expected rate limited error"),
    }

    // Test invalid input error
    let invalid_input = ToolError::invalid_input("Missing required parameter");
    match invalid_input {
        ToolError::InvalidInput { context, .. } => assert_eq!(context, "Missing required parameter"),
        _ => panic!("Expected invalid input error"),
    }
}

#[test]
fn test_environment_configuration_helpers() {
    // Test must_get_env with existing environment variable
    std::env::set_var("TEST_EXISTING_VAR", "test_value");
    let value = must_get_env("TEST_EXISTING_VAR");
    assert_eq!(value, "test_value");

    // Test get_env_or_default with existing variable
    let value = get_env_or_default("TEST_EXISTING_VAR", "default_value");
    assert_eq!(value, "test_value");

    // Test get_env_or_default with non-existing variable
    std::env::remove_var("TEST_NON_EXISTING_VAR");
    let value = get_env_or_default("TEST_NON_EXISTING_VAR", "default_value");
    assert_eq!(value, "default_value");

    // Clean up
    std::env::remove_var("TEST_EXISTING_VAR");
}

#[test]
#[should_panic(expected = "FATAL: Environment variable 'TEST_MISSING_VAR' is required but not set")]
fn test_must_get_env_panics_on_missing_var() {
    std::env::remove_var("TEST_MISSING_VAR");
    must_get_env("TEST_MISSING_VAR");
}

#[tokio::test]
async fn test_concurrent_signer_contexts() {
    let mut handles = Vec::new();
    
    // Spawn multiple concurrent tasks with different signers
    for i in 0..10 {
        let signer = MockSigner::new_solana(
            format!("task_{}", i), 
            format!("1111111111111111111111111111111{}", i)
        );
        
        let handle = task::spawn(async move {
            SignerContext::with_signer(Arc::new(signer), async move {
                // Simulate some work
                tokio::time::sleep(tokio::time::Duration::from_millis(5)).await;
                
                let current = SignerContext::current().await?;
                let pubkey = current.pubkey().unwrap();
                
                // Verify the pubkey matches what we expect for this task
                Ok::<String, SignerError>(pubkey)
            }).await
        });
        
        handles.push(handle);
    }
    
    // Collect results and verify isolation
    let results = future::join_all(handles).await;
    let mut pubkeys = std::collections::HashSet::new();
    
    for (i, result) in results.into_iter().enumerate() {
        let pubkey = result.unwrap().unwrap();
        let expected = format!("1111111111111111111111111111111{}", i);
        assert_eq!(pubkey, expected);
        assert!(pubkeys.insert(pubkey), "Duplicate pubkey found");
    }
    
    assert_eq!(pubkeys.len(), 10);
}

#[tokio::test]
async fn test_nested_signer_contexts() {
    let outer_signer = MockSigner::new_solana("outer".to_string(), "11111111111111111111111111111112".to_string());
    let inner_signer = MockSigner::new_evm("inner".to_string(), "0x742d35Cc2F5f8a89A0D2EAd5a53c97c49444E34F".to_string());
    
    SignerContext::with_signer(Arc::new(outer_signer), async {
        // Verify outer context
        let outer_current = SignerContext::current().await?;
        assert!(outer_current.pubkey().is_some());
        assert!(outer_current.address().is_none());
        let outer_pubkey = outer_current.pubkey().unwrap();
        
        // Create nested context
        SignerContext::with_signer(Arc::new(inner_signer), async {
            // Verify inner context overrides outer
            let inner_current = SignerContext::current().await?;
            assert!(inner_current.pubkey().is_none());
            assert!(inner_current.address().is_some());
            let inner_address = inner_current.address().unwrap();
            
            assert_eq!(inner_address, "0x742d35Cc2F5f8a89A0D2EAd5a53c97c49444E34F");
            
            Ok::<(), SignerError>(())
        }).await?;
        
        // Verify outer context is restored
        let restored_current = SignerContext::current().await?;
        assert!(restored_current.pubkey().is_some());
        assert!(restored_current.address().is_none());
        assert_eq!(restored_current.pubkey().unwrap(), outer_pubkey);
        
        Ok::<(), SignerError>(())
    }).await.unwrap();
}