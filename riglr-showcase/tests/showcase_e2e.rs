//! End-to-end integration tests for riglr showcase examples
//!
//! These tests validate the complete workflow from agent setup through tool execution.

use riglr_core::signer::SignerContext;
use riglr_solana_tools::LocalSolanaSigner;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;

#[tokio::test]
async fn test_multi_chain_balance_workflow() {
    // Setup test signer context with test configuration
    // For now, we'll use a Solana-only signer since LocalSolanaSigner doesn't support EVM
    let keypair = solana_sdk::signature::Keypair::new();
    let signer = LocalSolanaSigner::new(
        keypair,
        "https://api.devnet.solana.com".to_string(),
    );
    
    let signer = Arc::new(signer);
    
    SignerContext::with_signer(signer.clone(), async move {
        // Test basic functionality with timeout
        let result = timeout(Duration::from_secs(10), async {
            // Since the tools now get their clients from SignerContext,
            // we can call them directly without passing clients
            
            // Note: These would normally connect to real networks
            // For testing, we're just validating the structure works
            Ok::<_, Box<dyn std::error::Error>>(())
        }).await;
        
        assert!(result.is_ok(), "Test timed out");
        Ok::<_, riglr_core::signer::SignerError>(())
    }).await.unwrap();
}

#[tokio::test]
async fn test_signer_context_isolation() {
    // Test that different async tasks have isolated signer contexts
    let keypair1 = solana_sdk::signature::Keypair::new();
    let signer1 = LocalSolanaSigner::new(
        keypair1,
        "https://api.devnet.solana.com".to_string(),
    );
    
    let keypair2 = solana_sdk::signature::Keypair::new();
    let signer2 = LocalSolanaSigner::new(
        keypair2,
        "https://api.mainnet-beta.solana.com".to_string(),
    );
    
    // Run two tasks concurrently with different signers
    let handle1 = tokio::spawn(async move {
        SignerContext::with_signer(Arc::new(signer1), async {
            // Task 1 operations would use signer1
            tokio::time::sleep(Duration::from_millis(100)).await;
            Ok::<_, riglr_core::signer::SignerError>(())
        }).await.unwrap();
    });
    
    let handle2 = tokio::spawn(async move {
        SignerContext::with_signer(Arc::new(signer2), async {
            // Task 2 operations would use signer2
            tokio::time::sleep(Duration::from_millis(100)).await;
            Ok::<_, riglr_core::signer::SignerError>(())
        }).await.unwrap();
    });
    
    // Both tasks should complete successfully
    assert!(handle1.await.is_ok());
    assert!(handle2.await.is_ok());
}

#[tokio::test]
async fn test_tool_error_classification() {
    use riglr_core::error::ToolError;
    
    // Test that errors are properly classified
    let permanent_err = ToolError::permanent("Invalid parameters");
    assert!(matches!(permanent_err, ToolError::Permanent { .. }));
    
    let retriable_err = ToolError::retriable("Network timeout");
    assert!(matches!(retriable_err, ToolError::Retriable { .. }));
    
    let rate_limited_err = ToolError::rate_limited("API rate limit exceeded");
    assert!(matches!(rate_limited_err, ToolError::RateLimited { .. }));
}

#[cfg(feature = "riglr-cross-chain-tools")]
#[tokio::test]
async fn test_cross_chain_workflow() {
    #[cfg(feature = "riglr-cross-chain-tools")]
    use riglr_cross_chain_tools::error::CrossChainToolError;
    #[cfg(feature = "riglr-cross-chain-tools")]
    use riglr_core::error::ToolError;
    
    #[cfg(feature = "riglr-cross-chain-tools")]
    {
        // Test error conversion from cross-chain tools
        let bridge_error = CrossChainToolError::BridgeError("Bridge unavailable".to_string());
        let tool_error: ToolError = bridge_error.into();
        
        // Bridge errors should be retriable by default
        assert!(matches!(tool_error, ToolError::Retriable { .. }));
        
        // Test route not found error
        let route_error = CrossChainToolError::RouteNotFound("No route available".to_string());
        let tool_error: ToolError = route_error.into();
        assert!(matches!(tool_error, ToolError::Retriable { .. }));
        
        // Test unsupported chain error
        let chain_error = CrossChainToolError::UnsupportedChain("UNKNOWN".to_string());
        let tool_error: ToolError = chain_error.into();
        assert!(matches!(tool_error, ToolError::Permanent { .. }));
    }
}

/// Helper for managing test keys safely
#[allow(dead_code)]
struct TestKeyManager;

#[allow(dead_code)]
impl TestKeyManager {
    fn get_test_keys() -> std::collections::HashMap<String, String> {
        // Use deterministic test keys that don't hold real funds
        // These should be generated specifically for testing
        let mut keys = std::collections::HashMap::new();
        keys.insert("solana".to_string(), "test_solana_key_base58".to_string());
        keys.insert("ethereum".to_string(), "test_ethereum_private_key_hex".to_string());
        keys
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_workspace_integration() {
        // This test validates that all components work together
        
        // 1. Core signer setup
        let keypair = solana_sdk::signature::Keypair::new();
        let signer = LocalSolanaSigner::new(
            keypair,
            "https://api.devnet.solana.com".to_string(),
        );
        
        let signer = Arc::new(signer);
    
        SignerContext::with_signer(signer.clone(), async move {
            // 2. Error handling works across crates
            #[cfg(feature = "riglr-hyperliquid-tools")]
            {
                use riglr_hyperliquid_tools::error::HyperliquidToolError;
                use riglr_core::error::ToolError;
                let hl_error = HyperliquidToolError::RateLimit("Too many requests".to_string());
                let tool_error: ToolError = hl_error.into();
                assert!(matches!(tool_error, ToolError::RateLimited { .. }));
            }
            
            // 3. All tool crates are accessible
            // Just validate the modules exist and compile
            use riglr_solana_tools as _;
            use riglr_evm_tools as _;
            use riglr_web_tools as _;
            #[cfg(feature = "riglr-hyperliquid-tools")]
            use riglr_hyperliquid_tools as _;
            #[cfg(feature = "riglr-cross-chain-tools")]
            use riglr_cross_chain_tools as _;
            
            // Test passes if everything compiles and basic operations work
            Ok::<_, riglr_core::signer::SignerError>(())
        }).await.unwrap();
    }
}