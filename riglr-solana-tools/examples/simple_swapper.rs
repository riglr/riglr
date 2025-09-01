//! Example: Simple Token Swapper using ToolWorker
//!
//! Demonstrates how to use riglr-solana-tools to perform token swaps via Jupiter
//! using the canonical ToolWorker pattern for tool execution.
//!
//! ## Security Note
//! This example loads private keys securely from `~/.riglr/keys/solana.key`
//! with fallback to SOLANA_PRIVATE_KEY environment variable for compatibility.

use riglr_config::{Config, SolanaNetworkConfig};
use riglr_core::{
    idempotency::InMemoryIdempotencyStore,
    provider::ApplicationContext,
    util::{ensure_key_directory, load_private_key_with_fallback},
    ExecutionConfig, Job, SignerContext, ToolWorker,
};
use riglr_solana_tools::{
    clients::ApiClients,
    swap::{get_jupiter_quote_tool, get_token_price_tool, perform_jupiter_swap_tool},
    LocalSolanaSigner,
};
use serde_json::json;
use solana_sdk::signature::Keypair;
use std::sync::Arc;
use uuid::Uuid;

const SOLANA_PRIVATE_KEY_ENV: &str = "SOLANA_PRIVATE_KEY";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("=== Jupiter Token Swap Example ===\n");
    println!("WARNING: This example requires a funded wallet!");
    println!("Only run on devnet/testnet unless you know what you're doing.\n");

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
    worker.register_tool(get_token_price_tool()).await;
    worker.register_tool(get_jupiter_quote_tool()).await;
    worker.register_tool(perform_jupiter_swap_tool()).await;

    // Token mints
    let sol_mint = "So11111111111111111111111111111111111111112"; // Wrapped SOL
    let usdc_mint = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"; // USDC

    // Step 1: Get current price
    println!("Step 1: Getting current SOL/USDC price...");

    let price_job = Job {
        job_id: Uuid::new_v4(),
        tool_name: "get_token_price".to_string(),
        params: json!({
            "baseMint": sol_mint,
            "quoteMint": usdc_mint,
            "jupiterApiUrl": null
        }),
        retry_count: 0,
        max_retries: 3,
        idempotency_key: None,
    };

    match worker.process_job(price_job).await {
        Ok(job_result) => {
            if let riglr_core::JobResult::Success {
                value: price_data, ..
            } = job_result
            {
                if let Ok(price_info) = serde_json::from_value::<serde_json::Value>(price_data) {
                    println!("Current SOL price: ${:.2} USDC", price_info["price"]);
                    println!("Price Impact: {:.4}%\n", price_info["priceImpactPct"]);
                }
            }
        }
        Err(e) => {
            println!("Error getting price: {}\n", e);
        }
    }

    // Step 2: Get a swap quote
    println!("Step 2: Getting swap quote for 0.1 SOL -> USDC...");
    let amount = 100_000_000; // 0.1 SOL in lamports

    let quote_job = Job {
        job_id: Uuid::new_v4(),
        tool_name: "get_jupiter_quote".to_string(),
        params: json!({
            "inputMint": sol_mint,
            "outputMint": usdc_mint,
            "amount": amount,
            "slippageBps": 50,
            "onlyDirectRoutes": false,
            "jupiterApiUrl": null
        }),
        retry_count: 0,
        max_retries: 3,
        idempotency_key: None,
    };

    match worker.process_job(quote_job).await {
        Ok(job_result) => {
            if let riglr_core::JobResult::Success {
                value: quote_data, ..
            } = job_result
            {
                if let Ok(quote) = serde_json::from_value::<serde_json::Value>(quote_data) {
                    let sol_amount =
                        quote["inAmount"].as_u64().unwrap_or(0) as f64 / 1_000_000_000.0;
                    let usdc_amount = quote["outAmount"].as_u64().unwrap_or(0) as f64 / 1_000_000.0;

                    println!("Quote received:");
                    println!("  Input: {} SOL", sol_amount);
                    println!("  Output: {} USDC (estimated)", usdc_amount);
                    println!(
                        "  Minimum Output: {} USDC (after slippage)",
                        quote["otherAmountThreshold"].as_u64().unwrap_or(0) as f64 / 1_000_000.0
                    );
                    println!(
                        "  Price Impact: {:.4}%",
                        quote["priceImpactPct"].as_f64().unwrap_or(0.0)
                    );

                    if let Some(route_plan) = quote["routePlan"].as_array() {
                        println!("  Route Steps: {}", route_plan.len());

                        // Show route details
                        for (i, step) in route_plan.iter().enumerate() {
                            if let Some(swap_info) = step["swapInfo"].as_object() {
                                let label = swap_info["label"].as_str().unwrap_or("Unknown");
                                let percent = step["percent"].as_u64().unwrap_or(0);
                                println!("    Step {}: {} ({}%)", i + 1, label, percent);
                            }
                        }
                    }
                    println!();
                }
            }
        }
        Err(e) => {
            println!("Error getting quote: {}\n", e);
        }
    }

    // Step 3: Execute swap (requires funded wallet)
    println!("Step 3: Executing swap (demo only - requires funded wallet)...\n");

    // Load private key securely from file with environment variable fallback
    let key_dir = ensure_key_directory().expect("Failed to create key directory");
    let key_path = key_dir.join("solana.key");

    if let Ok(private_key) = load_private_key_with_fallback(&key_path, SOLANA_PRIVATE_KEY_ENV) {
        println!("Private key loaded. Initializing signer...");
        println!(
            "(Loaded from file: {}, or env var fallback)",
            key_path.display()
        );

        // Parse private key - supports both base58 and comma-separated formats
        let key_bytes: Vec<u8> = if private_key.contains(',') {
            // Comma-separated format
            private_key
                .split(',')
                .filter_map(|s| s.parse().ok())
                .collect()
        } else {
            // Try base58 format
            bs58::decode(&private_key)
                .into_vec()
                .unwrap_or_else(|_| Vec::new())
        };

        if key_bytes.len() == 64 {
            let keypair = Keypair::try_from(key_bytes.as_slice())?;

            // Create a LocalSolanaSigner with config-driven network configuration
            let network_config =
                SolanaNetworkConfig::new("mainnet", config.network.solana_rpc_url.clone());
            let signer = Arc::new(LocalSolanaSigner::from_keypair(keypair, network_config));

            // Execute swap within SignerContext using ToolWorker
            let swap_result = SignerContext::with_signer(signer, async {
                let swap_job = Job {
                    job_id: Uuid::new_v4(),
                    tool_name: "perform_jupiter_swap".to_string(),
                    params: json!({
                        "inputMint": sol_mint,
                        "outputMint": usdc_mint,
                        "amount": amount,
                        "slippageBps": 50,
                        "jupiterApiUrl": null,
                        "useVersionedTransaction": false
                    }),
                    retry_count: 0,
                    max_retries: 3,
                    idempotency_key: None,
                };

                worker
                    .process_job(swap_job)
                    .await
                    .map_err(|e| riglr_core::SignerError::ProviderError(e.to_string()))
            })
            .await;

            // Handle result
            match swap_result {
                Ok(job_result) => {
                    if let riglr_core::JobResult::Success {
                        value: swap_data, ..
                    } = job_result
                    {
                        if let Ok(swap_result) =
                            serde_json::from_value::<serde_json::Value>(swap_data)
                        {
                            println!("Swap successful!");
                            println!(
                                "  Transaction: {}",
                                swap_result["signature"].as_str().unwrap_or("unknown")
                            );
                            println!(
                                "  Input: {} SOL",
                                swap_result["inAmount"].as_u64().unwrap_or(0) as f64
                                    / 1_000_000_000.0
                            );
                            println!(
                                "  Output: {} USDC",
                                swap_result["outAmount"].as_u64().unwrap_or(0) as f64 / 1_000_000.0
                            );
                        }
                    }
                }
                Err(e) => {
                    println!("Swap failed: {}", e);
                    println!("This is expected if the wallet has insufficient funds.");
                }
            }
        } else {
            println!("Invalid private key format in environment variable.");
        }
    } else {
        println!("No private key found.");
        println!("Skipping actual swap execution.");
        println!("\nTo run the swap, either:");
        println!("  1. Place your key in: {}", key_path.display());
        println!("  2. Set {} environment variable", SOLANA_PRIVATE_KEY_ENV);
        println!("\nSupported formats: base58 or comma-separated bytes");
        println!("\nWARNING: Only use test wallets! Never expose production keys!");
    }

    println!("\n=== Example Complete ===");
    Ok(())
}
