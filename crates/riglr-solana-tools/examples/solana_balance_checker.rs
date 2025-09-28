//! Example: Balance Checker using `ToolWorker`
//!
//! Demonstrates how to use riglr-solana-tools with the `ToolWorker` pattern,
//! which is the canonical way to execute tools in the riglr ecosystem.

use core::error::Error;
use riglr_config::Config;
use riglr_core::{
    idempotency::InMemoryIdempotencyStore, provider::ApplicationContext, ExecutionConfig, Job,
    Worker,
};
use riglr_solana_tools::{
    balance::{GetSolTool, GetSplTokenTool},
    clients::Clients,
};
use serde_json::json;
use solana_client::rpc_client::RpcClient;
use std::sync::Arc;
use tracing_subscriber::fmt;

fn setup_worker() -> (Worker<InMemoryIdempotencyStore>, ApplicationContext) {
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

    // Register tools using tool structs
    worker.register_tool(Arc::new(GetSolTool {
        context: Arc::new(app_context.clone()),
    }));
    worker.register_tool(Arc::new(GetSplTokenTool {
        context: Arc::new(app_context.clone()),
    }));

    (worker, app_context)
}

async fn check_sol_balances(worker: &Worker<InMemoryIdempotencyStore>) -> Result<(), Box<dyn Error>> {
    let addresses = vec![
        "11111111111111111111111111111111",            // System Program
        "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA", // Token Program
        "So11111111111111111111111111111111111111112", // Wrapped SOL
    ];

    println!("SOL Balances:");
    println!("{}", "-".repeat(50));

    for address in &addresses {
        let job = Job::new(
            "get_sol_balance",
            &json!({
                "address": address
            }),
            3,
        )?;

        match worker.process_job(job).await {
            Ok(job_result) => {
                println!("Address: {}...", &address[..8]);
                if let riglr_core::JobResult::Success {
                    value: balance_data,
                    ..
                } = job_result
                {
                    if let Ok(balance) = serde_json::from_value::<serde_json::Value>(balance_data) {
                        let sol_balance = balance
                            .get("sol")
                            .map_or_else(|| "N/A".to_string(), ToString::to_string);
                        let lamports_balance = balance
                            .get("lamports")
                            .map_or_else(|| "N/A".to_string(), ToString::to_string);
                        println!("Balance: {sol_balance} SOL ({lamports_balance} lamports)");
                    }
                }
                println!();
            }
            Err(e) => {
                println!("Error checking balance for {address}: {e}");
                println!();
            }
        }
    }
    Ok(())
}

async fn check_spl_token_balance(worker: &Worker<InMemoryIdempotencyStore>) -> Result<(), Box<dyn Error>> {
    println!("\nSPL Token Balances:");
    println!("{}", "-".repeat(50));

    let owner_address = "11111111111111111111111111111111";
    let usdc_mint = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"; // USDC mint

    let token_job = Job::new(
        "get_spl_token_balance",
        &json!({
            "ownerAddress": owner_address,
            "mintAddress": usdc_mint
        }),
        3,
    )?;

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
                    let ui_amount = balance
                        .get("uiAmount")
                        .map_or_else(|| "N/A".to_string(), ToString::to_string);
                    let raw_amount = balance
                        .get("rawAmount")
                        .map_or_else(|| "N/A".to_string(), ToString::to_string);
                    let decimals = balance
                        .get("decimals")
                        .map_or_else(|| "N/A".to_string(), ToString::to_string);
                    println!("Balance: {ui_amount} USDC");
                    println!("Raw Amount: {raw_amount}");
                    println!("Decimals: {decimals}");
                }
            }
        }
        Err(e) => {
            println!("Error checking USDC balance: {e}");
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize logging
    fmt::init();

    println!("=== Solana Balance Checker Example (ToolWorker) ===\n");

    let (worker, _app_context) = setup_worker();
    println!("Worker initialized with registered tools");

    check_sol_balances(&worker).await?;
    check_spl_token_balance(&worker).await?;

    println!("\n=== Example Complete ===");
    Ok(())
}
