//! Example: Simple Token Swapper using `Worker`
//!
//! Demonstrates how to use riglr-solana-tools to perform token swaps via Jupiter
//! using the canonical `Worker` pattern for tool execution.
//!
//! ## Security Note
//! This example loads private keys securely from `~/.riglr/keys/solana.key`
//! with fallback to `SOLANA_PRIVATE_KEY` environment variable for compatibility.

#![allow(clippy::expect_used)]

use core::error::Error;
use riglr_config::{Config, SolanaNetworkConfig};
use riglr_core::{
    idempotency::InMemoryIdempotencyStore,
    provider::ApplicationContext,
    signer::error::{Error as SignerError, Standard},
    util::{ensure_key_directory, load_private_key_with_fallback},
    ExecutionConfig, Job, SignerContext, Worker,
};
use riglr_solana_tools::{
    clients::Clients,
    signer::Local,
    swap::{GetJupiterQuoteTool, GetTokenPriceTool, PerformJupiterTool},
};
use serde_json::json;
use solana_client::rpc_client::RpcClient;
use solana_sdk::signature::Keypair;
use std::sync::Arc;
use tracing_subscriber::fmt;

const SOLANA_PRIVATE_KEY_ENV: &str = "SOLANA_PRIVATE_KEY";

#[tokio::main]
#[expect(clippy::too_many_lines)]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize logging
    fmt::init();

    println!("=== Jupiter Token Swap Example ===\n");
    println!("WARNING: This example requires a funded wallet!");
    println!("Only run on devnet/testnet unless you know what you're doing.\n");

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

    // Register tools using generated tool structs
    worker.register_tool(Arc::new(GetTokenPriceTool {
        context: Arc::new(app_context.clone()),
    }));
    worker.register_tool(Arc::new(GetJupiterQuoteTool {
        context: Arc::new(app_context.clone()),
    }));
    worker.register_tool(Arc::new(PerformJupiterTool {
        context: Arc::new(app_context.clone()),
    }));

    // Token mints
    let sol_mint = "So11111111111111111111111111111111111111112"; // Wrapped SOL
    let usdc_mint = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"; // USDC

    // Step 1: Get current price
    println!("Step 1: Getting current SOL/USDC price...");

    let price_job = Job::new(
        "get_token_price",
        &json!({
            "baseMint": sol_mint,
            "quoteMint": usdc_mint,
            "jupiterApiUrl": null
        }),
        3,
    )
    .expect("Failed to create price job");

    match worker.process_job(price_job).await {
        Ok(job_result) => {
            if let riglr_core::JobResult::Success {
                value: price_data, ..
            } = job_result
            {
                if let Ok(price_info) = serde_json::from_value::<serde_json::Value>(price_data) {
                    println!(
                        "Current SOL price: ${:.2} USDC",
                        price_info
                            .get("price")
                            .and_then(serde_json::Value::as_f64)
                            .unwrap_or(0.0)
                    );
                    println!(
                        "Price Impact: {:.4}%\n",
                        price_info
                            .get("priceImpactPct")
                            .and_then(serde_json::Value::as_f64)
                            .unwrap_or(0.0)
                    );
                }
            }
        }
        Err(e) => {
            println!("Error getting price: {e}\n");
        }
    }

    // Step 2: Get a swap quote
    println!("Step 2: Getting swap quote for 0.1 SOL -> USDC...");
    let amount = 100_000_000; // 0.1 SOL in lamports

    let quote_job = Job::new(
        "get_jupiter_quote",
        &json!({
            "inputMint": sol_mint,
            "outputMint": usdc_mint,
            "amount": amount,
            "slippageBps": 50,
            "onlyDirectRoutes": false,
            "jupiterApiUrl": null
        }),
        3,
    )
    .expect("Failed to create quote job");

    match worker.process_job(quote_job).await {
        Ok(job_result) => {
            if let riglr_core::JobResult::Success {
                value: quote_data, ..
            } = job_result
            {
                if let Ok(quote) = serde_json::from_value::<serde_json::Value>(quote_data) {
                    // Precision loss acceptable for cryptocurrency amount display calculations
                    #[expect(clippy::cast_precision_loss)]
                    let sol_amount = quote
                        .get("inAmount")
                        .and_then(serde_json::Value::as_u64)
                        .unwrap_or(0) as f64
                        / 1_000_000_000.0;
                    #[expect(clippy::cast_precision_loss)]
                    let usdc_amount = quote
                        .get("outAmount")
                        .and_then(serde_json::Value::as_u64)
                        .unwrap_or(0) as f64
                        / 1_000_000.0;

                    println!("Quote received:");
                    println!("  Input: {sol_amount} SOL");
                    println!("  Output: {usdc_amount} USDC (estimated)");
                    println!("  Minimum Output: {} USDC (after slippage)", {
                        #[expect(clippy::cast_precision_loss)]
                        {
                            quote
                                .get("otherAmountThreshold")
                                .and_then(serde_json::Value::as_u64)
                                .unwrap_or(0) as f64
                                / 1_000_000.0
                        }
                    });
                    println!(
                        "  Price Impact: {:.4}%",
                        quote
                            .get("priceImpactPct")
                            .and_then(serde_json::Value::as_f64)
                            .unwrap_or(0.0)
                    );

                    if let Some(route_plan) = quote.get("routePlan").and_then(|v| v.as_array()) {
                        println!("  Route Steps: {}", route_plan.len());

                        // Show route details
                        for (i, step) in route_plan.iter().enumerate() {
                            if let Some(swap_info) = step["swapInfo"].as_object() {
                                let label = swap_info["label"].as_str().unwrap_or("Unknown");
                                let percent = step["percent"].as_u64().unwrap_or(0);
                                println!(
                                    "    Step {}: {} ({}%)",
                                    i.saturating_add(1),
                                    label,
                                    percent
                                );
                            }
                        }
                    }
                    println!();
                }
            }
        }
        Err(e) => {
            println!("Error getting quote: {e}\n");
        }
    }

    // Step 3: Execute swap (requires funded wallet)
    println!("Step 3: Executing swap (demo only - requires funded wallet)...\n");

    // Load private key securely from file with environment variable fallback
    // Key directory creation is essential for example operation
    let key_dir = ensure_key_directory().expect("Failed to create key directory");
    let key_path = key_dir.join("solana.key");

    let private_key_result = load_private_key_with_fallback(&key_path, SOLANA_PRIVATE_KEY_ENV);
    if let Ok(private_key) = private_key_result {
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

            // Create a Local signer with config-driven network configuration
            let network_config =
                SolanaNetworkConfig::new("mainnet", config.network.solana_rpc_url.clone());
            let signer = Arc::new(Local::from_keypair(keypair, network_config));

            // Execute swap within SignerContext using Worker
            let swap_result = SignerContext::with_signer(signer, async {
                let swap_job = Job::new(
                    "perform_jupiter_swap",
                    &json!({
                        "inputMint": sol_mint,
                        "outputMint": usdc_mint,
                        "amount": amount,
                        "slippageBps": 50,
                        "jupiterApiUrl": null,
                        "useVersionedTransaction": false
                    }),
                    3,
                )
                .expect("Failed to create swap job");

                worker
                    .process_job(swap_job)
                    .await
                    .map_err(|e| Box::new(Standard::Generic(e.to_string())) as Box<dyn SignerError>)
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
                                swap_result
                                    .get("signature")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("unknown")
                            );
                            println!("  Input: {} SOL", {
                                #[expect(clippy::cast_precision_loss)]
                                {
                                    swap_result
                                        .get("inAmount")
                                        .and_then(serde_json::Value::as_u64)
                                        .unwrap_or(0) as f64
                                        / 1_000_000_000.0
                                }
                            });
                            println!("  Output: {} USDC", {
                                #[expect(clippy::cast_precision_loss)]
                                {
                                    swap_result
                                        .get("outAmount")
                                        .and_then(serde_json::Value::as_u64)
                                        .unwrap_or(0) as f64
                                        / 1_000_000.0
                                }
                            });
                        }
                    }
                }
                Err(e) => {
                    println!("Swap failed: {e}");
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
        println!("  2. Set {SOLANA_PRIVATE_KEY_ENV} environment variable");
        println!("\nSupported formats: base58 or comma-separated bytes");
        println!("\nWARNING: Only use test wallets! Never expose production keys!");
    }

    println!("\n=== Example Complete ===");
    Ok(())
}
