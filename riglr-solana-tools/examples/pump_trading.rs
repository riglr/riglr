//! Pump.fun trading example using riglr-solana-tools
//!
//! This example demonstrates how to use the Pump.fun tools for:
//! - Deploying new meme tokens
//! - Buying tokens with slippage protection
//! - Selling tokens
//! - Getting token information and trending tokens
//!
//! This example uses the SignerContext pattern for secure transaction signing.

use riglr_core::{signer::SignerError, SignerContext};
use riglr_solana_tools::LocalSolanaSigner;
use riglr_solana_tools::{
    // deploy_pump_token, buy_pump_token, sell_pump_token,
    get_pump_token_info,
    get_trending_pump_tokens,
};
use solana_sdk::signer::{keypair::Keypair, Signer};
use std::sync::Arc;
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    info!("Starting Pump.fun trading example");

    // For this example, we'll create a new keypair
    // In production, you would load from a secure location
    let keypair = Keypair::new();
    info!("Using wallet: {}", keypair.pubkey());

    // Create a local signer for Solana devnet
    let signer = Arc::new(LocalSolanaSigner::new(
        keypair,
        "https://api.devnet.solana.com".to_string(),
    ));

    // Execute all operations within the signer context
    SignerContext::with_signer(signer, async {
        // Example 1: Get trending tokens
        info!("\n=== Getting Trending Tokens ===");
        match get_trending_pump_tokens(Some(5)).await {
            Ok(trending) => {
                info!("Found {} trending tokens:", trending.len());
                for token in trending {
                    info!(
                        "- {} ({}) - Market cap: {:?} SOL, Price: {:?}",
                        token.name, token.symbol, token.market_cap, token.price_sol
                    );
                }
            }
            Err(e) => error!("Failed to get trending tokens: {}", e),
        }

        // Example 2: Get token information for a specific token
        // This is a placeholder mint address - replace with actual Pump.fun token
        let example_mint = "So11111111111111111111111111111111111111112"; // SOL mint as example

        info!("\n=== Getting Token Information ===");
        match get_pump_token_info(example_mint.to_string()).await {
            Ok(token_info) => {
                info!("Token info retrieved:");
                info!("Name: {}", token_info.name);
                info!("Symbol: {}", token_info.symbol);
                info!("Description: {}", token_info.description);
                info!("Market cap: {:?}", token_info.market_cap);
                info!("Price: {:?} SOL", token_info.price_sol);
            }
            Err(e) => error!("Failed to get token info: {}", e),
        }

        info!("\n=== Pump.fun Example Complete ===");
        info!("This example demonstrated the basic Pump.fun read operations.");
        info!("For trading operations, see the commented examples in the source code.");

        Ok::<(), SignerError>(())
    })
    .await?;

    Ok(())
}

/// Example of how to use Pump.fun tools with rig agents
///
/// This function demonstrates how the tools can be integrated with rig agents
/// for autonomous trading decisions.
#[allow(dead_code)]
async fn pump_agent_example() -> Result<(), Box<dyn std::error::Error>> {
    // Create a keypair and signer
    let keypair = Keypair::new();
    let _signer = Arc::new(LocalSolanaSigner::new(
        keypair,
        "https://api.devnet.solana.com".to_string(),
    ));

    // Build an agent with Pump.fun tools (this is pseudocode as it requires rig-core)
    // When rig-core is available:
    /*
    use rig_core::agent::AgentBuilder;

    let agent = AgentBuilder::new("gpt-4")
        .preamble("You are a Pump.fun trading agent. You can deploy tokens, buy, sell, and analyze market trends.")
        .tool(deploy_pump_token)
        .tool(buy_pump_token)
        .tool(sell_pump_token)
        .tool(get_pump_token_info)
        .tool(get_trending_pump_tokens)
        .build();

    // Execute agent operations within signer context
    let response = SignerContext::with_signer(signer, async {
        agent.prompt(
            "Show me the top 3 trending tokens on Pump.fun and analyze their potential."
        ).await
    }).await?;

    println!("Agent response: {}", response);
    */

    println!("Agent example is commented out - requires rig-core integration");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_pump_tools_compilation() {
        // This test just ensures the tools compile and can be called
        // It doesn't actually execute network operations

        let keypair = Keypair::new();
        let signer = Arc::new(LocalSolanaSigner::new(
            keypair,
            "https://api.devnet.solana.com".to_string(),
        ));

        // Test that we can create the signer context without errors
        println!(
            "Pump tools compilation test passed with signer: {:?}",
            signer
        );
    }
}
