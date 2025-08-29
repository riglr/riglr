//! Example: Balance Checker using ToolWorker
//!
//! Demonstrates how to use riglr-solana-tools with the ToolWorker pattern,
//! which is the canonical way to execute tools in the riglr ecosystem.

use riglr_config::Config;
use riglr_core::{
    idempotency::InMemoryIdempotencyStore, provider::ApplicationContext, ExecutionConfig, Job,
    ToolWorker,
};
use riglr_solana_tools::{
    balance::{get_sol_balance_tool, get_spl_token_balance_tool},
    clients::ApiClients,
};
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("=== Solana Balance Checker Example (ToolWorker) ===\n");

    // Load configuration from environment
    let config = Config::from_env();

    // Create the ApplicationContext
    let app_context = ApplicationContext::from_config(&config);

    // Create and inject Solana RPC client
    let solana_client = Arc::new(solana_client::rpc_client::RpcClient::new(
        config.network.solana_rpc_url.clone(),
    ));
    app_context.set_extension(solana_client);

    // Create and inject API clients for external services
    let api_clients = ApiClients::new(&config.providers);
    app_context.set_extension(Arc::new(api_clients));

    // Create ToolWorker with default configuration
    let worker =
        ToolWorker::<InMemoryIdempotencyStore>::new(ExecutionConfig::default(), app_context);

    // Register tools using factory functions
    worker.register_tool(get_sol_balance_tool()).await;
    worker.register_tool(get_spl_token_balance_tool()).await;

    println!("ToolWorker initialized with registered tools");

    // Example addresses (replace with your own)
    let addresses = vec![
        "11111111111111111111111111111111",            // System Program
        "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA", // Token Program
        "So11111111111111111111111111111111111111112", // Wrapped SOL
    ];

    // Check SOL balances using ToolWorker
    println!("SOL Balances:");
    println!("{}", "-".repeat(50));

    for address in &addresses {
        let job = Job {
            job_id: Uuid::new_v4(),
            tool_name: "get_sol_balance".to_string(),
            params: json!({
                "address": address
            }),
            retry_count: 0,
            max_retries: 3,
            idempotency_key: None,
        };

        match worker.process_job(job).await {
            Ok(job_result) => {
                println!("Address: {}...", &address[..8]);
                if let riglr_core::JobResult::Success {
                    value: balance_data,
                    ..
                } = job_result
                {
                    if let Ok(balance) = serde_json::from_value::<serde_json::Value>(balance_data) {
                        println!(
                            "Balance: {} SOL ({} lamports)",
                            balance["sol"], balance["lamports"]
                        );
                    }
                }
                println!();
            }
            Err(e) => {
                println!("Error checking balance for {}: {}", address, e);
                println!();
            }
        }
    }

    // Check SPL token balance (example with USDC)
    println!("\nSPL Token Balances:");
    println!("{}", "-".repeat(50));

    let owner_address = "11111111111111111111111111111111";
    let usdc_mint = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"; // USDC mint

    let token_job = Job {
        job_id: Uuid::new_v4(),
        tool_name: "get_spl_token_balance".to_string(),
        params: json!({
            "ownerAddress": owner_address,
            "mintAddress": usdc_mint
        }),
        retry_count: 0,
        max_retries: 3,
        idempotency_key: None,
    };

    match worker.process_job(token_job).await {
        Ok(job_result) => {
            println!("Owner: {}...", &owner_address[..8]);
            println!("Token: USDC");
            if let riglr_core::JobResult::Success {
                value: balance_data,
                ..
            } = job_result
            {
                if let Ok(balance) = serde_json::from_value::<serde_json::Value>(balance_data) {
                    println!("Balance: {} USDC", balance["uiAmount"]);
                    println!("Raw Amount: {}", balance["rawAmount"]);
                    println!("Decimals: {}", balance["decimals"]);
                }
            }
        }
        Err(e) => {
            println!("Error checking USDC balance: {}", e);
        }
    }

    println!("\n=== Example Complete ===");
    Ok(())
}
