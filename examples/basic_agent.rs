//! Basic example of creating an AI agent with riglr tools
//!
//! This example demonstrates how to create a simple agent that can use
//! the various tools provided by the riglr ecosystem.

use riglr_core::{JobProcessor, JobRequest, ToolError};
use riglr_macros::tool;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Example tool for getting crypto prices
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct PriceInfo {
    token: String,
    price_usd: f64,
    change_24h: f64,
}

#[tool]
async fn get_token_price(
    token_symbol: String,
    chain: Option<String>,
) -> Result<PriceInfo, ToolError> {
    // Mock implementation - in production this would call a real API
    match token_symbol.to_uppercase().as_str() {
        "BTC" => Ok(PriceInfo {
            token: "Bitcoin".to_string(),
            price_usd: 65000.0,
            change_24h: 2.5,
        }),
        "ETH" => Ok(PriceInfo {
            token: "Ethereum".to_string(),
            price_usd: 3500.0,
            change_24h: 3.2,
        }),
        "SOL" => Ok(PriceInfo {
            token: "Solana".to_string(),
            price_usd: 150.0,
            change_24h: -1.5,
        }),
        _ => Err(ToolError::permanent(format!(
            "Unknown token: {}",
            token_symbol
        ))),
    }
}

/// Example tool for getting wallet balance
#[tool]
async fn get_wallet_balance(
    address: String,
    chain: String,
) -> Result<f64, ToolError> {
    // Mock implementation
    if address.starts_with("0x") {
        // Ethereum-style address
        Ok(1.5) // Mock balance in ETH
    } else if address.len() == 44 {
        // Solana-style address
        Ok(100.0) // Mock balance in SOL
    } else {
        Err(ToolError::permanent("Invalid address format"))
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("🦀 Riglr Basic Agent Example\n");

    // Example 1: Get token price
    println!("📊 Getting token prices...");
    
    let btc_price = get_token_price("BTC".to_string(), None).await?;
    println!(
        "  {} ({}): ${:.2} ({:+.1}%)",
        btc_price.token, "BTC", btc_price.price_usd, btc_price.change_24h
    );

    let eth_price = get_token_price("ETH".to_string(), Some("ethereum".to_string())).await?;
    println!(
        "  {} ({}): ${:.2} ({:+.1}%)",
        eth_price.token, "ETH", eth_price.price_usd, eth_price.change_24h
    );

    let sol_price = get_token_price("SOL".to_string(), None).await?;
    println!(
        "  {} ({}): ${:.2} ({:+.1}%)",
        sol_price.token, "SOL", sol_price.price_usd, sol_price.change_24h
    );

    // Example 2: Get wallet balance
    println!("\n💰 Getting wallet balances...");

    let eth_balance = get_wallet_balance(
        "0x742d35Cc6634C0532925a3b844Bc9e7595f0eA4B".to_string(),
        "ethereum".to_string(),
    )
    .await?;
    println!("  Ethereum wallet: {} ETH", eth_balance);

    let sol_balance = get_wallet_balance(
        "11111111111111111111111111111111".to_string(),
        "solana".to_string(),
    )
    .await?;
    println!("  Solana wallet: {} SOL", sol_balance);

    // Example 3: Error handling
    println!("\n⚠️  Testing error handling...");
    
    match get_token_price("UNKNOWN".to_string(), None).await {
        Ok(_) => println!("  Unexpected success"),
        Err(e) => println!("  Expected error: {}", e),
    }

    match get_wallet_balance("invalid".to_string(), "unknown".to_string()).await {
        Ok(_) => println!("  Unexpected success"),
        Err(e) => println!("  Expected error: {}", e),
    }

    println!("\n✅ Example completed successfully!");

    Ok(())
}