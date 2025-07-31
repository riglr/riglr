//! EVM tools demonstration commands.

use riglr_config::Config;
use std::sync::Arc;
use anyhow::Result;
use colored::Colorize;
use dialoguer::{Input, Select};
use indicatif::{ProgressBar, ProgressStyle};
// EVM tools with new client-first API
use riglr_evm_tools::{
    EvmClient,
    get_eth_balance, get_erc20_balance,
    get_uniswap_quote,
};
use std::collections::HashMap;
// use tracing::{info, warn}; // Temporarily disabled

/// Run the EVM tools demo.
pub async fn run_demo(config: Arc<Config>, address: Option<String>, chain_id: u64) -> Result<()> {
    println!("{}", "⚡ EVM Tools Demo".bright_blue().bold());
    println!("{}", "=".repeat(50).blue());
    
    // Get chain info
    let chain_info = get_chain_info(chain_id);
    println!("\n{}", format!("🔗 Chain: {} (ID: {})", chain_info.name, chain_id).cyan());
    
    // Get or prompt for wallet address  
    let wallet_address = match address {
        Some(addr) => addr,
        None => {
            println!("\n{}", "Let's analyze an EVM wallet!".cyan());
            let default_address = "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045"; // vitalik.eth
            Input::new()
                .with_prompt("Enter EVM wallet address")
                .default(default_address.to_string())
                .interact_text()?
        }
    };

    // Create EVM client
    let _client = match chain_id {
        1 => EvmClient::mainnet().await?,
        137 => EvmClient::new("https://polygon-rpc.com".to_string()).await?,
        42161 => EvmClient::new("https://arb1.arbitrum.io/rpc".to_string()).await?,
        10 => EvmClient::new("https://mainnet.optimism.io".to_string()).await?,
        8453 => EvmClient::new("https://mainnet.base.org".to_string()).await?,
        _ => EvmClient::mainnet().await?,
    };

    println!("\n{}", format!("🔍 Analyzing wallet: {}", wallet_address).yellow());
    
    // Show progress bar
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
            .template("{spinner:.green} {msg}")?,
    );
    pb.set_message("Fetching wallet data...");

    // Demo 1: Get native token balance (ETH)
    pb.set_message("Fetching native token balance...");
    
    match get_eth_balance(wallet_address.clone(), None).await {
        Ok(balance) => {
            println!("\n{}", format!("💰 {} Balance", chain_info.native_token).green().bold());
            println!("   Address: {}", balance.address);
            println!("   Balance: {} {}", balance.balance_formatted.bright_green(), balance.unit);
            println!("   Wei: {}", balance.balance_raw);
            println!("   Chain: {} (ID: {})", balance.chain_name, balance.chain_id);
            if let Some(block) = balance.block_number {
                println!("   Block: #{}", block);
            }
        }
        Err(e) => {
            println!("\n{}", format!("⚠️  Error fetching balance: {}", e).yellow());
            println!("   Falling back to simulation mode...");
            println!("\n{}", format!("💰 {} Balance (Simulated)", chain_info.native_token).green().bold());
            println!("   Address: {}", wallet_address);
            println!("   Balance: {} {}", "12.5847".bright_green(), chain_info.native_token);
            println!("   Wei: 12584700000000000000");
            println!("   Chain: {}", chain_info.name);
            println!("   Block: #19234567");
        }
    }

    // Demo 2: Check popular ERC20 token balances
    let popular_tokens = get_popular_tokens(chain_id);
    
    // Check first two tokens
    let token_limit = 2;
    
    for (token_count, (symbol, contract_address)) in popular_tokens.iter().enumerate() {
        if token_count >= token_limit {
            break;
        }
        
        pb.set_message(format!("Fetching {} balance...", symbol));
        
        match get_erc20_balance(wallet_address.clone(), contract_address.clone(), Some(true)).await {
            Ok(balance) => {
                println!("\n{}", format!("🪙 {} Balance", symbol).green().bold());
                println!("   Balance: {} {}", balance.balance_formatted.bright_green(), 
                    balance.token_symbol.as_ref().unwrap_or(symbol));
                println!("   Contract: {}", balance.token_address);
                println!("   Decimals: {}", balance.decimals);
            }
            Err(_) => {
                // Fallback to simulation if real call fails
                println!("\n{}", format!("🪙 {} Balance (Simulated)", symbol).green().bold());
                println!("   Balance: {} {}", "1000.00".bright_green(), symbol);
                println!("   Contract: {}", contract_address);
                println!("   Decimals: 6");
            }
        }
    }

    // Demo 3: Uniswap quote example (only for Ethereum mainnet)
    if chain_id == 1 {
        pb.set_message("Fetching Uniswap quote...");
        
        let weth = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2";
        let usdc = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48";
        
        match get_uniswap_quote(
            weth.to_string(),
            usdc.to_string(),
            "1".to_string(),  // 1 WETH
            18,               // WETH decimals
            6,                // USDC decimals
            Some(3000),       // 0.3% fee tier
            Some(50),         // 0.5% slippage
        ).await {
            Ok(quote) => {
                println!("\n{}", "🔄 Uniswap Quote (1 WETH → USDC)".green().bold());
                println!("   Input: {} {}", quote.amount_in, quote.token_in);
                println!("   Output: {} {}", quote.amount_out.bright_green(), quote.token_out);
                println!("   Price: {:.2} USDC per WETH", quote.price);
                println!("   Pool Fee: {:.2}%", quote.fee_tier as f64 / 10000.0);
                println!("   Min Output: {}", quote.amount_out_minimum);
            }
            Err(_) => {
                // Fallback to simulation
                println!("\n{}", "🔄 Uniswap Quote (1 WETH → USDC) - Simulated".green().bold());
                println!("   Input: 1.0 WETH");
                println!("   Output: ~{} USDC", "2891.45".bright_green());
                println!("   Pool Fee: 0.3%");
                println!("   Route: WETH → USDC");
                println!("   Price Impact: 0.01%");
            }
        }
    }

    pb.finish_and_clear();

    // Interactive menu for more demos
    println!("\n{}", "🎮 Interactive Options".bright_blue().bold());
    let mut options = vec![
        "Analyze another wallet",
        "Check specific ERC20 token",
        "Switch to different chain",
        "Exit demo",
    ];

    // Add Uniswap simulation for Ethereum mainnet
    if chain_id == 1 {
        options.insert(2, "Get custom Uniswap quote");
    }

    let selection = Select::new()
        .with_prompt("What would you like to do next?")
        .items(&options)
        .default(options.len() - 1)
        .interact()?;

    match selection {
        0 => {
            println!("\n{}", "Let's analyze another wallet!".cyan());
            let new_address: String = Input::new()
                .with_prompt("Enter wallet address")
                .interact_text()?;
            return Box::pin(run_demo(config, Some(new_address), chain_id)).await;
        }
        1 => {
            println!("\n{}", "🪙 ERC20 Token Analysis (Simulated)".cyan());
            let token_address: String = Input::new()
                .with_prompt("Enter ERC20 contract address")
                .interact_text()?;
            
            // Simulate token balance check
            println!("   Contract: {}", token_address);
            println!("   Symbol: {}", "CUSTOM".bright_cyan());
            println!("   Name: Custom Token");
            println!("   Balance: {} tokens", "1234.567".bright_green());
            println!("   Decimals: 18");
            println!("   {} Note: Running in simulation mode", "💡".yellow());
        }
        2 if chain_id == 1 => {
            println!("\n{}", "💱 Custom Uniswap Quote (Simulated)".cyan());
            let _token_in: String = Input::new()
                .with_prompt("Enter input token address")
                .default("0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".to_string()) // WETH
                .interact_text()?;
            let _token_out: String = Input::new()
                .with_prompt("Enter output token address")
                .default("0xA0b86a33E6411617D1A03e63BDD7d9F5eF9b6EA9".to_string()) // USDC
                .interact_text()?;
            let amount: String = Input::new()
                .with_prompt("Enter amount (in token units)")
                .default("1000000000000000000".to_string()) // 1 token with 18 decimals
                .interact_text()?;
            
            println!("   Simulating quote...");
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            
            println!("   Input Amount: {}", amount);
            println!("   Output Amount: {}", "2845.67".bright_green());
            println!("   Pool Fee: 0.05%");
            println!("   {} Note: Running in simulation mode", "💡".yellow());
        }
        2 | 3 => {
            println!("\n{}", "🔗 Chain Selection".cyan());
            let chains = [("Ethereum Mainnet", 1),
                ("Polygon", 137),
                ("Arbitrum One", 42161),
                ("Optimism", 10),
                ("Base", 8453)];
            
            let chain_options: Vec<&str> = chains.iter().map(|(name, _)| *name).collect();
            let chain_selection = Select::new()
                .with_prompt("Select chain")
                .items(&chain_options)
                .interact()?;
            
            let new_chain_id = chains[chain_selection].1;
            return Box::pin(run_demo(config, Some(wallet_address), new_chain_id)).await;
        }
        _ => {}
    }

    println!("\n{}", "✅ EVM demo completed!".bright_green().bold());
    println!("{}", "Thank you for exploring riglr-evm-tools!".dimmed());
    
    Ok(())
}

#[derive(Debug)]
struct ChainInfo {
    name: String,
    native_token: String,
}

fn get_chain_info(chain_id: u64) -> ChainInfo {
    match chain_id {
        1 => ChainInfo { name: "Ethereum Mainnet".to_string(), native_token: "ETH".to_string() },
        137 => ChainInfo { name: "Polygon".to_string(), native_token: "MATIC".to_string() },
        42161 => ChainInfo { name: "Arbitrum One".to_string(), native_token: "ETH".to_string() },
        10 => ChainInfo { name: "Optimism".to_string(), native_token: "ETH".to_string() },
        8453 => ChainInfo { name: "Base".to_string(), native_token: "ETH".to_string() },
        _ => ChainInfo { name: format!("Chain {}", chain_id), native_token: "ETH".to_string() },
    }
}

fn get_popular_tokens(chain_id: u64) -> HashMap<String, String> {
    let mut tokens = HashMap::new();
    
    match chain_id {
        1 => {
            // Ethereum Mainnet
            tokens.insert("USDC".to_string(), "0xA0b86a33E6411617D1A03e63BDD7d9F5eF9b6EA9".to_string());
            tokens.insert("USDT".to_string(), "0xdAC17F958D2ee523a2206206994597C13D831ec7".to_string());
            tokens.insert("WETH".to_string(), "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".to_string());
            tokens.insert("WBTC".to_string(), "0x2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599".to_string());
        }
        137 => {
            // Polygon
            tokens.insert("USDC".to_string(), "0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174".to_string());
            tokens.insert("WMATIC".to_string(), "0x0d500B1d8E8eF31E21C99d1Db9A6444d3ADf1270".to_string());
            tokens.insert("WETH".to_string(), "0x7ceB23fD6bC0adD59E62ac25578270cFf1b9f619".to_string());
        }
        42161 => {
            // Arbitrum One
            tokens.insert("USDC".to_string(), "0xaf88d065e77c8cC2239327C5EDb3A432268e5831".to_string());
            tokens.insert("WETH".to_string(), "0x82aF49447D8a07e3bd95BD0d56f35241523fBab1".to_string());
            tokens.insert("ARB".to_string(), "0x912CE59144191C1204E64559FE8253a0e49E6548".to_string());
        }
        _ => {
            // Default popular tokens (Ethereum addresses)
            tokens.insert("USDC".to_string(), "0xA0b86a33E6411617D1A03e63BDD7d9F5eF9b6EA9".to_string());
            tokens.insert("WETH".to_string(), "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".to_string());
        }
    }
    
    tokens
}
