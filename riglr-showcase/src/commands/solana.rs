//! Solana tools demonstration commands.

use crate::config::Config;
use anyhow::Result;
use colored::Colorize;
use dialoguer::{Input, Select};
use indicatif::{ProgressBar, ProgressStyle};
use riglr_solana_tools::{
    client::{SolanaClient, SolanaConfig},
    balance::get_sol_balance,
    swap::get_jupiter_quote,
};
use solana_sdk::commitment_config::CommitmentLevel;
use std::str::FromStr;
use tracing::warn;

/// Run the Solana tools demo.
pub async fn run_demo(config: Config, address: Option<String>) -> Result<()> {
    println!("{}", "🌟 Solana Tools Demo".bright_blue().bold());
    println!("{}", "=".repeat(50).blue());
    
    // Get or prompt for wallet address
    let wallet_address = match address {
        Some(addr) => addr,
        None => {
            println!("\n{}", "Let's analyze a Solana wallet!".cyan());
            let default_address = "So11111111111111111111111111111111111111112"; // SOL token mint
            Input::new()
                .with_prompt("Enter Solana wallet address")
                .default(default_address.to_string())
                .interact_text()?
        }
    };

    // Initialize Solana client
    let solana_config = SolanaConfig {
        rpc_url: config.solana_rpc_url.clone(),
        commitment: CommitmentLevel::Confirmed,
        timeout: std::time::Duration::from_secs(30),
        skip_preflight: false,
    };
    let client = SolanaClient::new(solana_config);

    println!("\n{}", format!("🔍 Analyzing wallet: {}", wallet_address).yellow());
    
    // Show progress bar
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
            .template("{spinner:.green} {msg}")?,
    );
    pb.set_message("Fetching wallet data...");

    // Demo 1: Get SOL balance
    pb.set_message("Checking SOL balance...");
    match get_sol_balance(&client, wallet_address.clone()).await {
        Ok(balance) => {
            println!("\n{}", "💰 SOL Balance".green().bold());
            println!("   Address: {}", balance.address);
            println!("   Balance: {} SOL", balance.formatted.bright_green());
            println!("   Lamports: {}", balance.lamports);
            println!("   SOL: {:.9}", balance.sol);
        }
        Err(e) => {
            warn!("Failed to get SOL balance: {}", e);
            println!("\n{}", format!("⚠️ Could not fetch SOL balance: {}", e).yellow());
        }
    }

    // Demo 2: Simulated token accounts (function not available)
    pb.set_message("Simulating token accounts...");
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    
    println!("\n{}", "🪙 Token Accounts (Simulated)".green().bold());
    let simulated_tokens = vec![
        ("USDC", "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", "1,250.50"),
        ("USDT", "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB", "850.75"),
        ("RAY", "4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R", "45.25"),
    ];
    
    for (i, (symbol, mint, balance)) in simulated_tokens.iter().enumerate() {
        println!("   {}. {} ({})", i + 1, symbol, mint);
        println!("      Balance: {}", balance.bright_green());
    }

    // Demo 3: Jupiter quote example
    pb.set_message("Getting Jupiter swap quote...");
    let sol_mint = "So11111111111111111111111111111111111111112";
    let usdc_mint = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
    
    match get_jupiter_quote(sol_mint.to_string(), usdc_mint.to_string(), 1_000_000_000, 50, false, None).await {
        Ok(quote) => {
            println!("\n{}", "🔄 Jupiter Swap Quote (1 SOL → USDC)".green().bold());
            println!("   Input: {} SOL", quote.in_amount as f64 / 1_000_000_000.0);
            println!("   Output: {} USDC", quote.out_amount.to_string().bright_green());
            println!("   Price Impact: {:.3}%", quote.price_impact_pct);
            if let Some(route_plan) = quote.route_plan.first() {
                println!("   Route: {} via {}", 
                    route_plan.swap_info.label.as_deref().unwrap_or("Unknown"), 
                    route_plan.swap_info.amm_key
                );
            }
        }
        Err(e) => {
            warn!("Failed to get Jupiter quote: {}", e);
            println!("\n{}", format!("⚠️ Could not fetch Jupiter quote: {}", e).yellow());
        }
    }

    pb.finish_and_clear();

    // Interactive menu for more demos
    println!("\n{}", "🎮 Interactive Options".bright_blue().bold());
    let options = vec![
        "Analyze another wallet",
        "Get detailed token info",
        "Simulate a swap",
        "Exit demo",
    ];

    let selection = Select::new()
        .with_prompt("What would you like to do next?")
        .items(&options)
        .default(3)
        .interact()?;

    match selection {
        0 => {
            println!("\n{}", "Let's analyze another wallet!".cyan());
            let new_address: String = Input::new()
                .with_prompt("Enter wallet address")
                .interact_text()?;
            return Box::pin(run_demo(config, Some(new_address))).await;
        }
        1 => {
            println!("\n{}", "🔍 Token Analysis".cyan());
            let token_mint: String = Input::new()
                .with_prompt("Enter token mint address")
                .default(usdc_mint.to_string())
                .interact_text()?;
            
            match solana_sdk::pubkey::Pubkey::from_str(&token_mint) {
                Ok(pubkey) => {
                    match client.rpc_client.get_token_supply(&pubkey) {
                        Ok(supply) => {
                            println!("   Token Mint: {}", token_mint);
                            // Display token amount properly
                            println!("   Total Supply: {}", supply.amount.bright_green());
                            println!("   Decimals: {}", supply.decimals);
                        }
                        Err(e) => {
                            println!("   {}", format!("Could not fetch token supply: {}", e).yellow());
                        }
                    }
                }
                Err(_) => {
                    println!("   {}", "Invalid token mint address".yellow());
                }
            }
        }
        2 => {
            println!("\n{}", "💱 Swap Simulation".cyan());
            println!("   This would simulate a token swap using Jupiter...");
            println!("   {}", "(Implementation would require wallet private key)".dimmed());
        }
        _ => {}
    }

    println!("\n{}", "✅ Solana demo completed!".bright_green().bold());
    println!("{}", "Thank you for exploring riglr-solana-tools!".dimmed());
    
    Ok(())
}
