//! Critical test to verify SignerContext propagation through RigToolAdapter.
//!
//! This test verifies the fix for Major Task 1.1 from the TODO.md:
//! ensuring that SignerContext is properly propagated through the RigToolAdapter.

use riglr_agents::adapter::RigToolAdapter;
use riglr_config::Config;
use riglr_core::{
    provider::ApplicationContext,
    signer::{SignerContext, UnifiedSigner},
    JobResult, ToolError,
};
use riglr_solana_tools::signer::LocalSolanaSigner;
use serde_json::Value as JsonValue;
use solana_sdk::signature::Keypair;
use std::sync::Arc;

/// Test tool that requires SignerContext to function
#[derive(Debug, Clone)]
struct SignerRequiringTool;

#[async_trait::async_trait]
impl riglr_core::Tool for SignerRequiringTool {
    fn name(&self) -> &str {
        "signer_test_tool"
    }

    fn description(&self) -> &str {
        "Tool that verifies SignerContext propagation"
    }

    fn schema(&self) -> JsonValue {
        serde_json::json!({
            "type": "object",
            "properties": {
                "test": { "type": "string" }
            }
        })
    }

    async fn execute(
        &self,
        _params: JsonValue,
        _context: &ApplicationContext,
    ) -> Result<JobResult, ToolError> {
        // This is the critical test - check if SignerContext is available
        match SignerContext::current().await {
            Ok(_) => {
                // SUCCESS - SignerContext was properly propagated!
                println!("✓ SignerContext successfully propagated through RigToolAdapter");
                Ok(JobResult::Success {
                    value: serde_json::json!({
                        "success": true,
                        "signer_available": true,
                        "message": "SignerContext was properly propagated - security fix verified!"
                    }),
                    tx_hash: None,
                })
            }
            Err(_) => {
                // FAILURE - SignerContext was NOT propagated
                eprintln!("✗ CRITICAL: SignerContext not propagated! Security vulnerability!");
                Err(ToolError::permanent_string(
                    "CRITICAL FAILURE: SignerContext not available! Security vulnerability detected.",
                ))
            }
        }
    }
}

#[tokio::test]
async fn test_signer_context_propagates_through_adapter() {
    // Set environment variables needed for testing
    std::env::set_var("FEATURES_ENABLE_BRIDGING", "false");
    std::env::set_var("LIFI_API_KEY", "dummy-key-for-testing");

    println!("\n=== Testing SignerContext Propagation Through RigToolAdapter ===\n");

    // Create a mock signer
    let keypair = Keypair::new();
    let signer: Arc<dyn UnifiedSigner> = Arc::new(LocalSolanaSigner::from_keypair_with_url(
        keypair,
        "http://localhost:8899".to_string(),
    ));

    // Create the tool that requires SignerContext
    let tool = Arc::new(SignerRequiringTool);

    // Create the RigToolAdapter with extensions pattern
    let config = Config::from_env();
    let context = ApplicationContext::from_config(&config);
    let adapter = RigToolAdapter::new(tool, context);

    // Test arguments
    let args = serde_json::json!({ "test": "propagation_test" });

    // Execute within a SignerContext - this simulates real usage
    let result = SignerContext::with_signer(signer, async move {
        use rig::tool::Tool;
        adapter
            .call(args)
            .await
            .map_err(|e| riglr_core::SignerError::TransactionFailed(e.to_string()))
    })
    .await;

    // Verify the result
    assert!(
        result.is_ok(),
        "Tool execution should succeed when SignerContext is propagated. Error: {:?}",
        result.err()
    );

    let json_result = result.unwrap();
    assert_eq!(
        json_result["success"],
        serde_json::Value::Bool(true),
        "Tool should report success"
    );
    assert_eq!(
        json_result["signer_available"],
        serde_json::Value::Bool(true),
        "Signer should be available"
    );

    println!("\n✓ Test PASSED: SignerContext propagation fix is working correctly!");
    println!("  The critical security vulnerability has been fixed.\n");
}

/// Test that tools work without SignerContext when they don't need it
#[tokio::test]
async fn test_adapter_works_without_signer_for_non_signing_tools() {
    // Set environment variables needed for testing
    std::env::set_var("FEATURES_ENABLE_BRIDGING", "false");
    std::env::set_var("LIFI_API_KEY", "dummy-key-for-testing");

    println!("\n=== Testing Non-Signing Tools Still Work ===\n");

    // Create a tool that doesn't check for SignerContext
    #[derive(Debug, Clone)]
    struct NonSigningTool;

    #[async_trait::async_trait]
    impl riglr_core::Tool for NonSigningTool {
        fn name(&self) -> &str {
            "non_signing_tool"
        }

        fn description(&self) -> &str {
            "Tool that doesn't require signing"
        }

        fn schema(&self) -> JsonValue {
            serde_json::json!({ "type": "object" })
        }

        async fn execute(
            &self,
            _params: JsonValue,
            _context: &ApplicationContext,
        ) -> Result<JobResult, ToolError> {
            // This tool doesn't check for SignerContext
            Ok(JobResult::Success {
                value: serde_json::json!({
                    "success": true,
                    "message": "Tool executed without requiring SignerContext"
                }),
                tx_hash: None,
            })
        }
    }

    let tool = Arc::new(NonSigningTool);
    let config = Config::from_env();
    let context = ApplicationContext::from_config(&config);
    let adapter = RigToolAdapter::new(tool, context);

    let args = serde_json::json!({});

    // Execute WITHOUT SignerContext
    use rig::tool::Tool;
    let result = adapter.call(args).await;

    assert!(
        result.is_ok(),
        "Non-signing tools should work without SignerContext"
    );

    println!("✓ Test PASSED: Non-signing tools still work correctly!\n");
}
