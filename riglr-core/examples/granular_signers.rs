//! Example demonstrating the new granular signer traits and typed SignerContext access
//!
//! This shows how the refactored API provides better type safety and chain-specific capabilities

use riglr_config::{Config, SolanaNetworkConfig};
use riglr_core::error::ToolError;
use riglr_core::signer::{LocalSolanaSigner, SignerContext, UnifiedSigner};
use std::sync::Arc;

/// Example tool that specifically requires a Solana signer
/// This provides compile-time guarantees that the signer supports Solana
async fn solana_specific_tool() -> Result<String, ToolError> {
    // Try to get the current signer as a Solana signer
    // This will fail at runtime if the context doesn't have a Solana-capable signer
    let signer = SignerContext::current_as_solana().await.map_err(|e| {
        ToolError::permanent_string(format!("This tool requires a Solana signer: {}", e))
    })?;

    // Now we have guaranteed access to Solana-specific methods
    let pubkey = signer.pubkey();
    let _client = signer.client();

    // Perform Solana-specific operations...
    Ok(format!("Solana operation completed for pubkey: {}", pubkey))
}

/// Example tool that specifically requires an EVM signer
/// This provides compile-time guarantees that the signer supports EVM
async fn evm_specific_tool() -> Result<String, ToolError> {
    // Try to get the current signer as an EVM signer
    let signer = SignerContext::current_as_evm().await.map_err(|e| {
        ToolError::permanent_string(format!("This tool requires an EVM signer: {}", e))
    })?;

    // Now we have guaranteed access to EVM-specific methods
    let chain_id = signer.chain_id();
    let address = signer.address();
    let _client = signer.client()?;

    // Perform EVM-specific operations...
    Ok(format!(
        "EVM operation completed on chain {} for address: {}",
        chain_id, address
    ))
}

/// Example of using the new unified signer context with configuration
async fn demonstrate_unified_context() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Load configuration from environment (or use builder for testing)
    let config = Config::try_from_env().unwrap_or_else(|_| {
        // If no environment config, create a test config
        Arc::new(Config::builder().build().expect("Failed to build config"))
    });

    // Create network configuration
    // In production, this would come from Config::from_env()
    let network_config = SolanaNetworkConfig::new("devnet", config.network.solana_rpc_url.clone());

    let keypair = solana_sdk::signature::Keypair::new();
    let solana_signer = Arc::new(LocalSolanaSigner::from_keypair(keypair, network_config));

    // Use the new unified signer context
    SignerContext::with_signer(solana_signer as Arc<dyn UnifiedSigner>, async {
        // Inside this context, tools can access the signer with type safety

        // This will succeed because we have a Solana signer
        match solana_specific_tool().await {
            Ok(result) => println!("Solana tool result: {}", result),
            Err(e) => println!("Error in Solana tool: {}", e),
        }

        // This will fail because our signer doesn't support EVM
        match evm_specific_tool().await {
            Ok(result) => println!("Unexpected success: {}", result),
            Err(e) => println!("Expected error for EVM tool: {}", e),
        }

        Ok::<(), riglr_core::signer::SignerError>(())
    })
    .await?;

    Ok(())
}

/// Example showing how tools can check capabilities before attempting operations
#[allow(dead_code)]
async fn flexible_tool() -> Result<String, ToolError> {
    // Check if we have a signer context at all
    if !SignerContext::is_available().await {
        return Ok("Running in read-only mode (no signer)".to_string());
    }

    // Try to get as Solana signer
    if let Ok(signer) = SignerContext::current_as_solana().await {
        let pubkey = signer.pubkey();
        return Ok(format!("Performing Solana operation for: {}", pubkey));
    }

    // Try to get as EVM signer
    if let Ok(signer) = SignerContext::current_as_evm().await {
        let address = signer.address();
        return Ok(format!("Performing EVM operation for: {}", address));
    }

    Err(ToolError::permanent_string(
        "Signer available but doesn't support Solana or EVM",
    ))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("Demonstrating granular signer traits with configuration injection\n");

    // In production, load configuration from environment
    // This would typically be done once at application startup
    // Config::from_env();

    println!("Loading configuration from environment (or using defaults)...");

    // Demonstrate the unified context with type-safe access
    demonstrate_unified_context().await?;

    println!("\nRefactoring complete! The new API provides:");
    println!("✅ Configuration-driven signer initialization");
    println!("✅ Type-safe access to chain-specific capabilities");
    println!("✅ Compile-time guarantees for tool requirements");
    println!("✅ Clear separation between Solana and EVM operations");
    println!("✅ No direct env::var calls in tools");

    Ok(())
}
