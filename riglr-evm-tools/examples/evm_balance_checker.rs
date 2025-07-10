//! Example: Checking balances on EVM chains
//!
//! This example demonstrates how to check ETH and ERC20 token balances
//! across different EVM-compatible chains.

use riglr_evm_tools::{
    get_eth_balance, get_erc20_balance, BalanceResult, TokenBalanceResult,
};
use std::env;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("🔍 EVM Balance Checker Example\n");

    // Get address from command line or use a default
    let address = env::args()
        .nth(1)
        .unwrap_or_else(|| "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045".to_string()); // vitalik.eth

    // Check ETH balance on mainnet
    println!("📊 Checking ETH balance on Ethereum Mainnet...");
    check_eth_balance(&address, None).await?;

    // Check ETH balance on Polygon
    println!("\n📊 Checking MATIC balance on Polygon...");
    check_eth_balance(&address, Some("https://polygon-rpc.com".to_string())).await?;

    // Check USDC balance on Ethereum
    println!("\n📊 Checking USDC balance on Ethereum...");
    check_erc20_balance(
        &address,
        "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48", // USDC on Ethereum
        "USDC",
        6,
        None,
    )
    .await?;

    // Check USDT balance on Ethereum
    println!("\n📊 Checking USDT balance on Ethereum...");
    check_erc20_balance(
        &address,
        "0xdAC17F958D2ee523a2206206994597C13D831ec7", // USDT on Ethereum
        "USDT",
        6,
        None,
    )
    .await?;

    println!("\n✅ Balance check complete!");

    Ok(())
}

async fn check_eth_balance(address: &str, rpc_url: Option<String>) -> anyhow::Result<()> {
    // Note: The functions create their own clients internally based on the signer context
    let _rpc_url = rpc_url; // Keeping for potential future use
    
    match get_eth_balance(address.to_string(), None).await {
        Ok(balance) => {
            print_eth_balance(&balance);
        }
        Err(e) => {
            println!("  ❌ Error: {}", e);
        }
    }
    Ok(())
}

async fn check_erc20_balance(
    address: &str,
    token_address: &str,
    token_name: &str,
    _decimals: u8,
    rpc_url: Option<String>,
) -> anyhow::Result<()> {
    // Note: The functions create their own clients internally based on the signer context
    let _rpc_url = rpc_url; // Keeping for potential future use
    
    match get_erc20_balance(
        address.to_string(),
        token_address.to_string(),
        Some(true), // Fetch metadata
    )
    .await
    {
        Ok(balance) => {
            print_token_balance(&balance, token_name);
        }
        Err(e) => {
            println!("  ❌ Error checking {} balance: {}", token_name, e);
        }
    }
    Ok(())
}

fn print_eth_balance(balance: &BalanceResult) {
    println!("  📍 Address: {}", balance.address);
    println!("  🌐 Chain: {} (ID: {})", balance.chain_name, balance.chain_id);
    println!("  💰 Balance: {} {}", balance.balance_formatted, balance.unit);
    println!("  🔢 Raw: {} wei", balance.balance_raw);
    if let Some(block) = balance.block_number {
        println!("  📦 Block: #{}", block);
    }
}

fn print_token_balance(balance: &TokenBalanceResult, token_name: &str) {
    println!("  📍 Address: {}", balance.address);
    println!("  🪙 Token: {}", token_name);
    if let Some(symbol) = &balance.token_symbol {
        println!("  🏷️ Symbol: {}", symbol);
    }
    println!("  💰 Balance: {}", balance.balance_formatted);
    println!("  🔢 Decimals: {}", balance.decimals);
    println!("  🌐 Chain: {} (ID: {})", balance.chain_name, balance.chain_id);
}