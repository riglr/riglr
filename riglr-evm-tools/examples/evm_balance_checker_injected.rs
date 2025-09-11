//! Example: Checking ETH balances using dependency injection
//!
//! This example demonstrates the client injection pattern where the application
//! creates and injects the EVM client into the ApplicationContext.

use alloy::providers::ProviderBuilder;
use riglr_config::Config;
use riglr_core::provider::ApplicationContext;
use riglr_evm_tools::{
    balance::{get_erc20_balance, get_eth_balance},
    EthBalance, TokenBalance,
};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("=== EVM Balance Checker with Injection Pattern ===\n");

    // Load configuration from environment
    let config = Config::from_env();

    // Create the ApplicationContext
    let app_context = ApplicationContext::from_config(&config);

    // Create and inject EVM provider for Ethereum mainnet
    let url = "https://eth.llamarpc.com".parse()?;
    let provider = ProviderBuilder::new().on_http(url);
    app_context.set_extension(Arc::new(provider));

    println!("Connected to Ethereum mainnet via injection pattern\n");

    // Example addresses
    let addresses = vec![
        "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045", // vitalik.eth
        "0xBE0eB53F46cd790Cd13851d5EFf43D12404d33E8", // Binance 7
        "0x0000000000000000000000000000000000000000", // Zero address
    ];

    // Check ETH balances using the injected context
    println!("ETH Balances:");
    println!("{}", "-".repeat(50));

    for address in &addresses {
        match get_eth_balance(
            address.to_string(),
            Some(1), // Ethereum mainnet
            &app_context,
        )
        .await
        {
            Ok(balance) => {
                print_eth_balance(&balance);
                println!();
            }
            Err(e) => {
                println!("Error checking balance for {}: {}", address, e);
                println!();
            }
        }
    }

    // Check ERC20 token balance (example with USDC)
    println!("\nERC20 Token Balances:");
    println!("{}", "-".repeat(50));

    let usdc_address = "0xA0b86991c31df06eB84A750a53748A83E6B94a89"; // USDC on Ethereum
    let wallet_address = addresses[0].to_string();

    match get_erc20_balance(
        usdc_address.to_string(),
        wallet_address.clone(),
        Some(1),
        &app_context,
    )
    .await
    {
        Ok(balance) => {
            print_token_balance(&balance, "USDC");
        }
        Err(e) => {
            println!("Error checking USDC balance: {}", e);
        }
    }

    println!("\n=== Example Complete ===");
    println!("This example demonstrated the client injection pattern where:");
    println!("1. The application creates the EVM client");
    println!("2. The client is injected into ApplicationContext");
    println!("3. Tools retrieve the client from context's extensions");
    println!("4. No circular dependencies between core and tools");

    Ok(())
}

fn print_eth_balance(balance: &EthBalance) {
    println!("  📍 Address: {}", balance.address);
    println!(
        "  🌐 Chain: {} (ID: {})",
        balance.chain_name, balance.chain_id
    );
    println!("  💰 Balance: {}", balance.balance_formatted);
    println!("  🔢 Raw: {} wei", balance.raw_balance);
    if balance.block_number > 0 {
        println!("  📦 Block: #{}", balance.block_number);
    }
}

fn print_token_balance(balance: &TokenBalance, token_name: &str) {
    println!("  📍 Token: {}", token_name);
    println!("  💰 Balance: {}", balance.balance_formatted);
    if let Some(symbol) = &balance.token_symbol {
        println!("  🏷️ Symbol: {}", symbol);
    }
    println!("  🔢 Decimals: {}", balance.decimals);
    println!("  📜 Contract: {}", balance.token_address);
}
