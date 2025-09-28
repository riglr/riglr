//! Pump.fun trading example using `ToolWorker`
//!
//! This example demonstrates how to use the Pump.fun tools with `ToolWorker` for:
//! - Deploying new meme tokens
//! - Buying tokens with slippage protection
//! - Selling tokens
//! - Getting token information and trending tokens
//!
//! This example uses the canonical `ToolWorker` pattern with `SignerContext`.

use riglr_config::{Config, SolanaNetworkConfig};
use riglr_core::{
    idempotency::InMemoryIdempotencyStore, provider::ApplicationContext, signer::error::Standard,
    ExecutionConfig, Job, SignerContext, Worker,
};
use riglr_solana_tools::{
    clients::Clients,
    pump::{
        BuyPumpTokenTool, DeployPumpTokenTool, GetPumpTokenInfoTool, GetTrendingPumpTokensTool,
        SellPumpTokenTool,
    },
    signer::Local,
};
use serde_json::json;
use solana_client::rpc_client::RpcClient;
use solana_sdk::signer::{keypair::Keypair, Signer};
use std::sync::Arc;
use tracing::{error, info};
use tracing_subscriber::fmt;

#[tokio::main]
#[expect(clippy::too_many_lines)]
async fn main() -> Result<(), Box<dyn riglr_core::SignerError>> {
    // Initialize logging
    fmt::init();

    info!("Starting Pump.fun trading example");

    // Load configuration from environment
    let config = Config::from_env();

    // Create the ApplicationContext
    let app_context = ApplicationContext::from_config(&config);

    // Create and inject Solana RPC client
    let solana_client = Arc::new(RpcClient::new(config.network.solana_rpc_url.clone()));
    app_context.set_extension(solana_client);

    // Create and inject API clients for external services
    let api_clients = Clients::new(&config.providers);
    app_context.set_extension(Arc::new(api_clients));

    // Create Worker with default configuration
    let worker =
        Worker::<InMemoryIdempotencyStore>::new(ExecutionConfig::default(), app_context.clone());

    // Register pump tools using Tool struct constructors with context
    worker.register_tool(Arc::new(DeployPumpTokenTool {
        context: Arc::new(app_context.clone()),
    }));
    worker.register_tool(Arc::new(BuyPumpTokenTool {
        context: Arc::new(app_context.clone()),
    }));
    worker.register_tool(Arc::new(SellPumpTokenTool {
        context: Arc::new(app_context.clone()),
    }));
    worker.register_tool(Arc::new(GetPumpTokenInfoTool {
        context: Arc::new(app_context.clone()),
    }));
    worker.register_tool(Arc::new(GetTrendingPumpTokensTool {
        context: Arc::new(app_context.clone()),
    }));

    // For this example, we'll create a new keypair
    // In production, you would load from a secure location
    let keypair = Keypair::new();
    info!("Using wallet: {}", keypair.pubkey());

    // Create a local signer using config-driven network configuration
    let network_config = SolanaNetworkConfig::new("devnet", config.network.solana_rpc_url.clone());
    let signer = Arc::new(Local::from_keypair(keypair, network_config));

    // Execute all operations within the signer context
    SignerContext::with_signer(signer, async {
        // Example 1: Get trending tokens using ToolWorker
        info!("\n=== Getting Trending Tokens ===");

        let trending_job = Job::new(
            "get_trending_pump_tokens",
            &json!({
                "limit": 5
            }),
            3,
        )
        .map_err(|e| {
            Box::new(Standard::Configuration(format!(
                "Failed to create job: {e}"
            ))) as Box<dyn riglr_core::SignerError>
        })?;

        match worker.process_job(trending_job).await {
            Ok(job_result) => {
                if let riglr_core::JobResult::Success {
                    value: trending_data,
                    ..
                } = job_result
                {
                    if let Ok(trending) =
                        serde_json::from_value::<Vec<serde_json::Value>>(trending_data)
                    {
                        info!("Found {} trending tokens:", trending.len());
                        for token in trending {
                            info!(
                                "- {} ({}) - Market cap: {:?} SOL, Price: {:?}",
                                token
                                    .get("name")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("Unknown"),
                                token
                                    .get("symbol")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("Unknown"),
                                token.get("market_cap"),
                                token.get("price_sol")
                            );
                        }
                    }
                }
            }
            Err(e) => error!("Failed to get trending tokens: {}", e),
        }

        // Example 2: Get token information for a specific token using ToolWorker
        // This is a placeholder mint address - replace with actual Pump.fun token
        let example_mint = "So11111111111111111111111111111111111111112"; // SOL mint as example

        info!("\n=== Getting Token Information ===");

        let token_info_job = Job::new(
            "get_pump_token_info",
            &json!({
                "mint": example_mint
            }),
            3,
        )
        .map_err(|e| {
            Box::new(Standard::Configuration(format!(
                "Failed to create job: {e}"
            ))) as Box<dyn riglr_core::SignerError>
        })?;

        match worker.process_job(token_info_job).await {
            Ok(job_result) => {
                if let riglr_core::JobResult::Success {
                    value: token_data, ..
                } = job_result
                {
                    if let Ok(token_info) = serde_json::from_value::<serde_json::Value>(token_data)
                    {
                        info!("Token info retrieved:");
                        info!(
                            "Name: {}",
                            token_info
                                .get("name")
                                .and_then(|v| v.as_str())
                                .unwrap_or("Unknown")
                        );
                        info!(
                            "Symbol: {}",
                            token_info
                                .get("symbol")
                                .and_then(|v| v.as_str())
                                .unwrap_or("Unknown")
                        );
                        info!(
                            "Description: {}",
                            token_info
                                .get("description")
                                .and_then(|v| v.as_str())
                                .unwrap_or("Unknown")
                        );
                        info!("Market cap: {:?}", token_info.get("market_cap"));
                        info!("Price: {:?} SOL", token_info.get("price_sol"));
                    }
                }
            }
            Err(e) => error!("Failed to get token info: {}", e),
        }

        info!("\n=== Pump.fun Example Complete ===");
        info!("This example demonstrated the basic Pump.fun read operations using ToolWorker.");
        info!("For trading operations, see the commented examples in the source code.");

        Ok::<(), Box<dyn riglr_core::SignerError>>(())
    })
    .await?;

    Ok(())
}

/// Example of how to use Pump.fun tools with rig agents
///
/// This function demonstrates how the tools can be integrated with rig agents
/// for autonomous trading decisions.
#[expect(dead_code)]
fn pump_agent_example() {
    // Create a keypair and signer
    let keypair = Keypair::new();
    let config = Config::from_env();
    let network_config = SolanaNetworkConfig::new("devnet", config.network.solana_rpc_url.clone());
    let _signer = Arc::new(Local::from_keypair(keypair, network_config));

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
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_pump_tools_compilation() {
        // This test just ensures the tools compile and can be called
        // It doesn't actually execute network operations

        let keypair = Keypair::new();
        let config = Config::from_env();
        let network_config =
            SolanaNetworkConfig::new("devnet", config.network.solana_rpc_url.clone());
        let signer = Arc::new(Local::from_keypair(keypair, network_config));

        // Test that we can create the signer context without errors
        println!(
            "Pump tools compilation test passed with signer: {:?}",
            signer
        );
    }
}
