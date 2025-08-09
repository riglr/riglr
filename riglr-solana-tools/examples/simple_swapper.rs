//! Example: Simple Token Swapper
//!
//! Demonstrates how to use riglr-solana-tools to perform token swaps via Jupiter

use riglr_solana_tools::{
    client::SolanaClient,
    swap::{get_jupiter_quote, get_token_price, perform_jupiter_swap},
    transaction::{init_signer_context, SignerContext},
};
use solana_sdk::signature::Keypair;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("=== Jupiter Token Swap Example ===\n");
    println!("WARNING: This example requires a funded wallet!");
    println!("Only run on devnet/testnet unless you know what you're doing.\n");

    // Create client for mainnet (Jupiter works best on mainnet)
    let client = SolanaClient::mainnet();

    // Token mints
    let sol_mint = "So11111111111111111111111111111111111111112".to_string(); // Wrapped SOL
    let usdc_mint = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(); // USDC

    // Step 1: Get current price
    println!("Step 1: Getting current SOL/USDC price...");
    match get_token_price(sol_mint.clone(), usdc_mint.clone(), None).await {
        Ok(price_info) => {
            println!("Current SOL price: ${:.2} USDC", price_info.price);
            println!("Price Impact: {:.4}%\n", price_info.price_impact_pct);
        }
        Err(e) => {
            println!("Error getting price: {}\n", e);
        }
    }

    // Step 2: Get a swap quote
    println!("Step 2: Getting swap quote for 0.1 SOL -> USDC...");
    let amount = 100_000_000; // 0.1 SOL in lamports

    match get_jupiter_quote(
        sol_mint.clone(),
        usdc_mint.clone(),
        amount,
        50, // 0.5% slippage
        false,
        None,
    )
    .await
    {
        Ok(quote) => {
            let sol_amount = quote.in_amount as f64 / 1_000_000_000.0;
            let usdc_amount = quote.out_amount as f64 / 1_000_000.0; // USDC has 6 decimals

            println!("Quote received:");
            println!("  Input: {} SOL", sol_amount);
            println!("  Output: {} USDC (estimated)", usdc_amount);
            println!(
                "  Minimum Output: {} USDC (after slippage)",
                quote.other_amount_threshold as f64 / 1_000_000.0
            );
            println!("  Price Impact: {:.4}%", quote.price_impact_pct);
            println!("  Route Steps: {}", quote.route_plan.len());

            // Show route details
            for (i, step) in quote.route_plan.iter().enumerate() {
                println!(
                    "    Step {}: {} ({}%)",
                    i + 1,
                    step.swap_info
                        .label
                        .as_ref()
                        .unwrap_or(&"Unknown".to_string()),
                    step.percent
                );
            }
            println!();
        }
        Err(e) => {
            println!("Error getting quote: {}\n", e);
        }
    }

    // Step 3: Execute swap (requires funded wallet)
    println!("Step 3: Executing swap (demo only - requires funded wallet)...\n");

    // Check for private key in environment variable (for demo purposes only!)
    if let Ok(private_key) = env::var("SOLANA_PRIVATE_KEY") {
        println!("Private key found in environment. Initializing signer...");

        // Parse private key (this is just for demo - use secure key management in production!)
        let key_bytes: Vec<u8> = private_key
            .split(',')
            .filter_map(|s| s.parse().ok())
            .collect();

        if key_bytes.len() == 64 {
            let keypair = Keypair::from_bytes(&key_bytes)?;

            // Initialize signer context
            let mut context = SignerContext::new();
            context.add_signer("swapper", keypair)?;
            init_signer_context(context);

            // Attempt swap
            match perform_jupiter_swap(
                &client,
                sol_mint.clone(),
                usdc_mint.clone(),
                amount,
                50,
                Some("swapper".to_string()),
                None,
                false,
            )
            .await
            {
                Ok(result) => {
                    println!("Swap successful!");
                    println!("  Transaction: {}", result.signature);
                    println!("  Input: {} SOL", result.in_amount as f64 / 1_000_000_000.0);
                    println!("  Output: {} USDC", result.out_amount as f64 / 1_000_000.0);
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
        println!("No private key found in SOLANA_PRIVATE_KEY environment variable.");
        println!("Skipping actual swap execution.");
        println!("\nTo run the swap, set your private key:");
        println!("  export SOLANA_PRIVATE_KEY='1,2,3,...,64' (comma-separated bytes)");
        println!("\nWARNING: Only use test wallets! Never expose production keys!");
    }

    println!("\n=== Example Complete ===");
    Ok(())
}
