use riglr_core::signer::{SignerContext, TransactionSigner};
use riglr_solana_tools::LocalSolanaSigner;
use std::sync::Arc;
use solana_sdk::signature::Keypair;

#[tokio::test]
#[ignore] // Use --ignored to run devnet tests
async fn test_solana_balance_integration() {
    let keypair = Keypair::new();
    let local_signer = Arc::new(LocalSolanaSigner::new(
        keypair,
        "https://api.devnet.solana.com".to_string()
    )) as Arc<dyn TransactionSigner>;
    
    let result = SignerContext::with_signer(local_signer, async {
        // This tests that tools can access SignerContext in real scenarios
        let signer = SignerContext::current().await?;
        
        // Get the public key to check balance
        let pubkey = signer.pubkey()
            .ok_or(riglr_core::signer::SignerError::Configuration("No pubkey".to_string()))?;
        
        println!("Checking balance for: {}", pubkey);
        
        // In a real tool, this would call the balance checking function
        // For now, we just verify the signer context works
        Ok::<String, riglr_core::signer::SignerError>(format!("Balance check for {}", pubkey))
    }).await;
    
    // Should work even if balance is 0
    assert!(result.is_ok());
    let response = result.unwrap();
    assert!(response.contains("Balance check"));
}

#[tokio::test]
async fn test_solana_signer_context_with_multiple_signers() {
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
    
    // Execute with first signer
    let result1 = SignerContext::with_signer(signer1, async {
        let signer = SignerContext::current().await?;
        Ok::<String, riglr_core::signer::SignerError>(
            signer.pubkey().unwrap_or_default()
        )
    }).await.unwrap();
    
    // Execute with second signer
    let result2 = SignerContext::with_signer(signer2, async {
        let signer = SignerContext::current().await?;
        Ok::<String, riglr_core::signer::SignerError>(
            signer.pubkey().unwrap_or_default()
        )
    }).await.unwrap();
    
    // Verify different signers were used
    assert_ne!(result1, result2);
}

#[tokio::test]
async fn test_solana_tool_error_propagation() {
    let keypair = Keypair::new();
    let local_signer = Arc::new(LocalSolanaSigner::new(
        keypair,
        "https://api.devnet.solana.com".to_string()
    )) as Arc<dyn TransactionSigner>;
    
    let result = SignerContext::with_signer(local_signer, async {
        // Simulate a tool error
        use riglr_solana_tools::error::SolanaToolError;
        use riglr_core::ToolError;
        
        let tool_error = SolanaToolError::Rpc("Connection failed".to_string());
        let core_error: ToolError = tool_error.into();
        
        match core_error {
            ToolError::Retriable(_) => Ok::<String, riglr_core::signer::SignerError>(
                "Error correctly converted to Retriable".to_string()
            ),
            _ => Err(riglr_core::signer::SignerError::Configuration(
                "Wrong error conversion".to_string()
            ))
        }
    }).await;
    
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Error correctly converted to Retriable");
}

#[tokio::test]
async fn test_solana_client_access() {
    let keypair = Keypair::new();
    let local_signer = Arc::new(LocalSolanaSigner::new(
        keypair,
        "https://api.devnet.solana.com".to_string()
    )) as Arc<dyn TransactionSigner>;
    
    let result = SignerContext::with_signer(local_signer, async {
        let signer = SignerContext::current().await?;
        
        // Test that we can get the Solana client
        let _client = signer.solana_client();
        
        Ok::<(), riglr_core::signer::SignerError>(())
    }).await;
    
    assert!(result.is_ok());
}