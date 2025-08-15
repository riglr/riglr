//! Simple end-to-end integration test for riglr showcase
//!
//! This test validates that the core components compile and integrate properly.

use riglr_core::signer::SignerContext;
use riglr_solana_tools::LocalSolanaSigner;
use riglr_core::error::ToolError;
use std::sync::Arc;

#[test]
fn test_error_classification() {
    // Test that errors are properly classified
    let permanent_err = ToolError::permanent_string("Invalid parameters");
    assert!(matches!(permanent_err, ToolError::Permanent { .. }));
    
    let retriable_err = ToolError::retriable_string("Network timeout");
    assert!(matches!(retriable_err, ToolError::Retriable { .. }));
    
    let rate_limited_err = ToolError::rate_limited_string("API rate limit exceeded");
    assert!(matches!(rate_limited_err, ToolError::RateLimited { .. }));
}

#[tokio::test]
async fn test_signer_context_basic() {
    // Create a test signer
    let keypair = solana_sdk::signature::Keypair::new();
    let signer = Arc::new(LocalSolanaSigner::new(
        keypair,
        "https://api.devnet.solana.com".to_string(),
    ));
    
    // Use the signer in a context
    let result = SignerContext::with_signer(signer, async {
        // Inside this block, tools can access the signer via SignerContext::current()
        let current_signer = SignerContext::current().await?;
        assert!(current_signer.solana_client().url().contains("devnet"));
        Ok::<_, riglr_core::signer::SignerError>(())
    }).await;
    
    assert!(result.is_ok());
}

#[test]
fn test_all_crates_accessible() {
    // Just validate the modules exist and compile
    use riglr_solana_tools as _;
    use riglr_evm_tools as _;
    use riglr_web_tools as _;
    use riglr_graph_memory as _;
    
    // Optional crates would need feature flags
    #[cfg(feature = "riglr-hyperliquid-tools")]
    use riglr_hyperliquid_tools as _;
    
    #[cfg(feature = "riglr-cross-chain-tools")]
    use riglr_cross_chain_tools as _;
}