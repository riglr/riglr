//! Example: Balance Checker
//!
//! Demonstrates how to use riglr-solana-tools to query SOL and SPL token balances

use riglr_solana_tools::{
    balance::{get_sol_balance, get_spl_token_balance},
    client::SolanaClient,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("=== Solana Balance Checker Example ===\n");

    // Create a client for devnet
    let client = SolanaClient::devnet();
    println!("Connected to Solana Devnet\n");

    // Example addresses (replace with your own)
    let addresses = vec![
        "11111111111111111111111111111111",            // System Program
        "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA", // Token Program
        "So11111111111111111111111111111111111111112", // Wrapped SOL
    ];

    // Check SOL balances
    println!("SOL Balances:");
    println!("{}", "-".repeat(50));

    for address in &addresses {
        match get_sol_balance(&client, address.to_string()).await {
            Ok(balance) => {
                println!("Address: {}...", &address[..8]);
                println!(
                    "Balance: {} SOL ({} lamports)",
                    balance.sol, balance.lamports
                );
                println!();
            }
            Err(e) => {
                println!("Error checking balance for {}: {}", address, e);
                println!();
            }
        }
    }

    // Check SPL token balance (example with USDC on devnet)
    println!("\nSPL Token Balances:");
    println!("{}", "-".repeat(50));

    // Example: Check USDC balance for an address
    let owner_address = "11111111111111111111111111111111".to_string();
    let usdc_mint = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(); // USDC mint

    match get_spl_token_balance(&client, owner_address.clone(), usdc_mint.clone()).await {
        Ok(balance) => {
            println!("Owner: {}...", &owner_address[..8]);
            println!("Token: USDC");
            println!("Balance: {} USDC", balance.ui_amount);
            println!("Raw Amount: {}", balance.raw_amount);
            println!("Decimals: {}", balance.decimals);
        }
        Err(e) => {
            println!("Error checking USDC balance: {}", e);
        }
    }

    println!("\n=== Example Complete ===");
    Ok(())
}
