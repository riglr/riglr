use riglr_core::signer::{SignerContext, LocalSolanaSigner, TransactionSigner, SignerError};
use std::sync::Arc;
use solana_sdk::signature::Keypair;

// Simple test function that uses SignerContext
async fn test_tool() -> Result<String, SignerError> {
    // This tests that SignerContext::current() works within tool context
    let signer = SignerContext::current().await?;
    
    // Test that we can access signer methods
    let public_key = signer.pubkey().ok_or(SignerError::NoSignerContext)?;
    
    Ok(format!("Tool executed with signer: {}", public_key))
}

#[tokio::test]
async fn test_core_integration_flow() {
    // Create a test signer
    let keypair = Keypair::new();
    let local_signer = Arc::new(LocalSolanaSigner::new(
        keypair, 
        "https://api.devnet.solana.com".to_string()
    )) as Arc<dyn TransactionSigner>;
    
    // Test SignerContext integration
    let result = SignerContext::with_signer(local_signer, async {
        test_tool().await
    }).await;
    
    assert!(result.is_ok());
    let response = result.unwrap();
    assert!(response.contains("Tool executed with signer"));
}

#[tokio::test]  
async fn test_signer_context_isolation() {
    let keypair1 = Keypair::new();
    let signer1 = Arc::new(LocalSolanaSigner::new(
        keypair1, 
        "https://api.devnet.solana.com".to_string()
    )) as Arc<dyn TransactionSigner>;
    
    let keypair2 = Keypair::new();
    let signer2 = Arc::new(LocalSolanaSigner::new(
        keypair2, 
        "https://api.devnet.solana.com".to_string()
    )) as Arc<dyn TransactionSigner>;
    
    let result1 = SignerContext::with_signer(signer1.clone(), async {
        let current = SignerContext::current().await?;
        current.pubkey().ok_or(SignerError::NoSignerContext)
    }).await.unwrap();
    
    let result2 = SignerContext::with_signer(signer2.clone(), async {
        let current = SignerContext::current().await?;
        current.pubkey().ok_or(SignerError::NoSignerContext)
    }).await.unwrap();
    
    assert_ne!(result1, result2);
}

#[tokio::test]
async fn test_signer_context_no_context() {
    // Test that SignerContext::current() returns error when not within a context
    let result = SignerContext::current().await;
    assert!(result.is_err());
    match result {
        Err(SignerError::NoSignerContext) => {},
        _ => panic!("Expected NoSignerContext error"),
    }
}

#[tokio::test]
async fn test_nested_calls() {
    async fn inner_function() -> Result<String, SignerError> {
        let signer = SignerContext::current().await?;
        let pubkey = signer.pubkey().ok_or(SignerError::NoSignerContext)?;
        Ok(format!("Inner: {}", pubkey))
    }
    
    async fn outer_function() -> Result<String, SignerError> {
        let signer = SignerContext::current().await?;
        let pubkey = signer.pubkey().ok_or(SignerError::NoSignerContext)?;
        let inner_result = inner_function().await?;
        Ok(format!("Outer: {}, {}", pubkey, inner_result))
    }
    
    let keypair = Keypair::new();
    let signer = Arc::new(LocalSolanaSigner::new(
        keypair, 
        "https://api.devnet.solana.com".to_string()
    )) as Arc<dyn TransactionSigner>;
    
    let result = SignerContext::with_signer(signer, async {
        outer_function().await
    }).await;
    
    assert!(result.is_ok());
    let response = result.unwrap();
    assert!(response.contains("Outer"));
    assert!(response.contains("Inner"));
}

#[tokio::test]
async fn test_error_propagation() {
    async fn failing_function() -> Result<String, SignerError> {
        Err(SignerError::Configuration("Test error".to_string()))
    }
    
    let keypair = Keypair::new();
    let signer = Arc::new(LocalSolanaSigner::new(
        keypair, 
        "https://api.devnet.solana.com".to_string()
    )) as Arc<dyn TransactionSigner>;
    
    let result = SignerContext::with_signer(signer, async {
        failing_function().await
    }).await;
    
    assert!(result.is_err());
    match result.unwrap_err() {
        SignerError::Configuration(msg) => assert_eq!(msg, "Test error"),
        _ => panic!("Expected Configuration error"),
    }
}

#[tokio::test]
async fn test_concurrent_contexts() {
    use tokio::task;
    
    let keypair1 = Keypair::new();
    let signer1 = Arc::new(LocalSolanaSigner::new(
        keypair1, 
        "https://api.devnet.solana.com".to_string()
    )) as Arc<dyn TransactionSigner>;
    
    let keypair2 = Keypair::new();  
    let signer2 = Arc::new(LocalSolanaSigner::new(
        keypair2, 
        "https://api.devnet.solana.com".to_string()
    )) as Arc<dyn TransactionSigner>;
    
    // Run two concurrent tasks with different signers
    let handle1 = task::spawn(async move {
        SignerContext::with_signer(signer1, async {
            // Simulate some work
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            let current = SignerContext::current().await?;
            current.pubkey().ok_or(SignerError::NoSignerContext)
        }).await
    });
    
    let handle2 = task::spawn(async move {
        SignerContext::with_signer(signer2, async {
            // Simulate some work
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            let current = SignerContext::current().await?;
            current.pubkey().ok_or(SignerError::NoSignerContext)
        }).await
    });
    
    let result1 = handle1.await.unwrap().unwrap();
    let result2 = handle2.await.unwrap().unwrap();
    
    // Verify that each context maintained its own signer
    assert_ne!(result1, result2);
}

#[tokio::test]
async fn test_signer_context_is_available() {
    // Outside context - should return false
    assert!(!SignerContext::is_available().await);
    
    let keypair = Keypair::new();
    let signer = Arc::new(LocalSolanaSigner::new(
        keypair, 
        "https://api.devnet.solana.com".to_string()
    )) as Arc<dyn TransactionSigner>;
    
    // Inside context - should return true
    SignerContext::with_signer(signer, async {
        assert!(SignerContext::is_available().await);
        Ok::<(), SignerError>(())
    }).await.unwrap();
    
    // Outside context again - should return false
    assert!(!SignerContext::is_available().await);
}