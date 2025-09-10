//! EVM tools demonstration commands.

use anyhow::Result;
use colored::Colorize;
use dialoguer::{Input, Select};
use indicatif::{ProgressBar, ProgressStyle};
use riglr_core::provider::ApplicationContext;
use std::sync::Arc;
// EVM tools with new client-first API
use riglr_evm_tools::{get_erc20_balance, get_eth_balance, get_uniswap_quote};
use std::collections::HashMap;
// use tracing::{info, warn}; // Temporarily disabled

/// Get wallet address from user input or use provided address.
async fn get_wallet_address(address: Option<String>) -> Result<String> {
    match address {
        Some(addr) => Ok(addr),
        None => {
            println!("\n{}", "Let's analyze an EVM wallet!".cyan());
            let default_address = "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045"; // vitalik.eth
            Input::new()
                .with_prompt("Enter EVM wallet address")
                .default(default_address.to_string())
                .interact_text()
                .map_err(Into::into)
        }
    }
}

/// Fetch and display native token balance.
async fn display_native_balance(
    wallet_address: &str,
    chain_info: &ChainInfo,
    pb: &ProgressBar,
    context: &ApplicationContext,
) -> Result<()> {
    pb.set_message("Fetching native token balance...");

    match get_eth_balance(wallet_address.to_string(), None, context).await {
        Ok(balance) => {
            println!(
                "\n{}",
                format!("💰 {} Balance", chain_info.native_token)
                    .green()
                    .bold()
            );
            println!("   Address: {}", balance.address);
            println!(
                "   Balance: {} {}",
                balance.balance_formatted.bright_green(),
                balance.unit
            );
            println!("   Wei: {}", balance.balance_raw);
            println!(
                "   Chain: {} (ID: {})",
                balance.chain_name, balance.chain_id
            );
            {
                println!("   Block: #{}", balance.block_number);
            }
        }
        Err(e) => {
            println!(
                "\n{}",
                format!("⚠️  Error fetching balance: {}", e).yellow()
            );
            println!("   Falling back to simulation mode...");
            println!(
                "\n{}",
                format!("💰 {} Balance (Simulated)", chain_info.native_token)
                    .green()
                    .bold()
            );
            println!("   Address: {}", wallet_address);
            println!(
                "   Balance: {} {}",
                "12.5847".bright_green(),
                chain_info.native_token
            );
            println!("   Wei: 12584700000000000000");
            println!("   Chain: {}", chain_info.name);
            println!("   Block: #19234567");
        }
    }
    Ok(())
}

/// Fetch and display ERC20 token balances.
async fn display_token_balances(
    wallet_address: &str,
    chain_id: u64,
    pb: &ProgressBar,
    context: &ApplicationContext,
) -> Result<()> {
    let popular_tokens = get_popular_tokens(chain_id);
    let token_limit = 2;

    for (token_count, (symbol, contract_address)) in popular_tokens.iter().enumerate() {
        if token_count >= token_limit {
            break;
        }

        pb.set_message(format!("Fetching {} balance...", symbol));

        match get_erc20_balance(
            contract_address.clone(),
            wallet_address.to_string(),
            Some(chain_id),
            context,
        )
        .await
        {
            Ok(balance) => {
                println!("\n{}", format!("🪙 {} Balance", symbol).green().bold());
                println!(
                    "   Balance: {} {}",
                    balance.balance_formatted.bright_green(),
                    balance.token_symbol.as_ref().unwrap_or(symbol)
                );
                println!("   Contract: {}", balance.token_address);
                println!("   Decimals: {}", balance.decimals);
            }
            Err(_) => {
                println!(
                    "\n{}",
                    format!("🪙 {} Balance (Simulated)", symbol).green().bold()
                );
                println!("   Balance: {} {}", "1000.00".bright_green(), symbol);
                println!("   Contract: {}", contract_address);
                println!("   Decimals: 6");
            }
        }
    }
    Ok(())
}

/// Display Uniswap quote for Ethereum mainnet.
async fn display_uniswap_quote(pb: &ProgressBar, context: &ApplicationContext) -> Result<()> {
    pb.set_message("Fetching Uniswap quote...");

    let weth = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2";
    let usdc = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48";

    match get_uniswap_quote(
        weth.to_string(), // token_in
        usdc.to_string(), // token_out
        "1".to_string(),  // amount_in (1 WETH)
        18,               // decimals_in (WETH has 18 decimals)
        6,                // decimals_out (USDC has 6 decimals)
        Some(3000),       // fee_tier (0.3% pool)
        Some(50),         // slippage (0.5%)
        Some(1),          // chain_id (Ethereum mainnet)
        context,
    )
    .await
    {
        Ok(quote) => {
            println!("\n{}", "🔄 Uniswap Quote (1 WETH → USDC)".green().bold());
            println!("   Input: {} {}", quote.amount_in, quote.token_in);
            println!(
                "   Output: {} {}",
                quote.amount_out.bright_green(),
                quote.token_out
            );
            println!("   Price: {:.2} USDC per WETH", quote.price);
            println!("   Pool Fee: {}", quote.fee_tier);
            println!("   Min Output: {}", quote.amount_out_minimum);
        }
        Err(e) => {
            println!(
                "\n{}",
                format!("⚠️  Error fetching Uniswap quote: {}", e).yellow()
            );
        }
    }
    Ok(())
}

/// Handle interactive menu selection.
async fn handle_menu_selection(
    selection: usize,
    chain_id: u64,
    context: Arc<ApplicationContext>,
    wallet_address: String,
) -> Result<bool> {
    match selection {
        0 => {
            println!("\n{}", "Let's analyze another wallet!".cyan());
            let new_address: String = Input::new()
                .with_prompt("Enter wallet address")
                .interact_text()?;
            Box::pin(run_demo(context, Some(new_address), chain_id)).await?;
            Ok(true)
        }
        1 => {
            println!("\n{}", "🪙 ERC20 Token Analysis (Simulated)".cyan());
            let token_address: String = Input::new()
                .with_prompt("Enter ERC20 contract address")
                .interact_text()?;

            println!("   Contract: {}", token_address);
            println!("   Symbol: {}", "CUSTOM".bright_cyan());
            println!("   Name: Custom Token");
            println!("   Balance: {} tokens", "1234.567".bright_green());
            println!("   Decimals: 18");
            println!("   {} Note: Running in simulation mode", "💡".yellow());
            Ok(false)
        }
        2 if chain_id == 1 => {
            println!("\n{}", "💱 Custom Uniswap Quote (Simulated)".cyan());
            let _token_in: String = Input::new()
                .with_prompt("Enter input token address")
                .default("0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".to_string())
                .interact_text()?;
            let _token_out: String = Input::new()
                .with_prompt("Enter output token address")
                .default("0xA0b86a33E6411617D1A03e63BDD7d9F5eF9b6EA9".to_string())
                .interact_text()?;
            let amount: String = Input::new()
                .with_prompt("Enter amount (in token units)")
                .default("1000000000000000000".to_string())
                .interact_text()?;

            println!("   Simulating quote...");
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;

            println!("   Input Amount: {}", amount);
            println!("   Output Amount: {}", "2845.67".bright_green());
            println!("   Pool Fee: 0.05%");
            println!("   {} Note: Running in simulation mode", "💡".yellow());
            Ok(false)
        }
        2 | 3 => {
            println!("\n{}", "🔗 Chain Selection".cyan());
            let chains = [
                ("Ethereum Mainnet", 1),
                ("Polygon", 137),
                ("Arbitrum One", 42161),
                ("Optimism", 10),
                ("Base", 8453),
            ];

            let chain_options: Vec<&str> = chains.iter().map(|(name, _)| *name).collect();
            let chain_selection = Select::new()
                .with_prompt("Select chain")
                .items(&chain_options)
                .interact()?;

            let new_chain_id = chains[chain_selection].1;
            Box::pin(run_demo(context, Some(wallet_address), new_chain_id)).await?;
            Ok(true)
        }
        _ => Ok(false),
    }
}

/// Run the EVM tools demo.
pub async fn run_demo(
    context: Arc<ApplicationContext>,
    address: Option<String>,
    chain_id: u64,
) -> Result<()> {
    println!("{}", "⚡ EVM Tools Demo".bright_blue().bold());
    println!("{}", "=".repeat(50).blue());

    // Get chain info
    let chain_info = get_chain_info(chain_id);
    println!(
        "\n{}",
        format!("🔗 Chain: {} (ID: {})", chain_info.name, chain_id).cyan()
    );

    // Get wallet address
    let wallet_address = get_wallet_address(address).await?;

    println!(
        "\n{}",
        format!("🔍 Analyzing wallet: {}", wallet_address).yellow()
    );

    // Show progress bar
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
            .template("{spinner:.green} {msg}")?,
    );
    pb.set_message("Fetching wallet data...");

    // Display native token balance
    display_native_balance(&wallet_address, &chain_info, &pb, &context).await?;

    // Display ERC20 token balances
    display_token_balances(&wallet_address, chain_id, &pb, &context).await?;

    // Display Uniswap quote for Ethereum mainnet
    if chain_id == 1 {
        display_uniswap_quote(&pb, &context).await?;
    }

    pb.finish_and_clear();

    // Interactive menu
    println!("\n{}", "🎮 Interactive Options".bright_blue().bold());
    let mut options = vec![
        "Analyze another wallet",
        "Check specific ERC20 token",
        "Switch to different chain",
        "Exit demo",
    ];

    if chain_id == 1 {
        options.insert(2, "Get custom Uniswap quote");
    }

    let selection = Select::new()
        .with_prompt("What would you like to do next?")
        .items(&options)
        .default(options.len() - 1)
        .interact()?;

    let should_return = handle_menu_selection(selection, chain_id, context, wallet_address).await?;

    if should_return {
        return Ok(());
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
        1 => ChainInfo {
            name: "Ethereum Mainnet".to_string(),
            native_token: "ETH".to_string(),
        },
        137 => ChainInfo {
            name: "Polygon".to_string(),
            native_token: "MATIC".to_string(),
        },
        42161 => ChainInfo {
            name: "Arbitrum One".to_string(),
            native_token: "ETH".to_string(),
        },
        10 => ChainInfo {
            name: "Optimism".to_string(),
            native_token: "ETH".to_string(),
        },
        8453 => ChainInfo {
            name: "Base".to_string(),
            native_token: "ETH".to_string(),
        },
        _ => ChainInfo {
            name: format!("Chain {}", chain_id),
            native_token: "ETH".to_string(),
        },
    }
}

fn get_popular_tokens(chain_id: u64) -> HashMap<String, String> {
    let mut tokens = HashMap::new();

    match chain_id {
        1 => {
            // Ethereum Mainnet
            tokens.insert(
                "USDC".to_string(),
                "0xA0b86a33E6411617D1A03e63BDD7d9F5eF9b6EA9".to_string(),
            );
            tokens.insert(
                "USDT".to_string(),
                "0xdAC17F958D2ee523a2206206994597C13D831ec7".to_string(),
            );
            tokens.insert(
                "WETH".to_string(),
                "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".to_string(),
            );
            tokens.insert(
                "WBTC".to_string(),
                "0x2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599".to_string(),
            );
        }
        137 => {
            // Polygon
            tokens.insert(
                "USDC".to_string(),
                "0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174".to_string(),
            );
            tokens.insert(
                "WMATIC".to_string(),
                "0x0d500B1d8E8eF31E21C99d1Db9A6444d3ADf1270".to_string(),
            );
            tokens.insert(
                "WETH".to_string(),
                "0x7ceB23fD6bC0adD59E62ac25578270cFf1b9f619".to_string(),
            );
        }
        42161 => {
            // Arbitrum One
            tokens.insert(
                "USDC".to_string(),
                "0xaf88d065e77c8cC2239327C5EDb3A432268e5831".to_string(),
            );
            tokens.insert(
                "WETH".to_string(),
                "0x82aF49447D8a07e3bd95BD0d56f35241523fBab1".to_string(),
            );
            tokens.insert(
                "ARB".to_string(),
                "0x912CE59144191C1204E64559FE8253a0e49E6548".to_string(),
            );
        }
        _ => {
            // Default popular tokens (Ethereum addresses)
            tokens.insert(
                "USDC".to_string(),
                "0xA0b86a33E6411617D1A03e63BDD7d9F5eF9b6EA9".to_string(),
            );
            tokens.insert(
                "WETH".to_string(),
                "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".to_string(),
            );
        }
    }

    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_wallet_address_when_provided_should_return_address() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let address = Some("0x742d35Cc6634C0532925a3b8D8a4B6fE38d92fB2".to_string());

        let result = rt.block_on(get_wallet_address(address));

        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            "0x742d35Cc6634C0532925a3b8D8a4B6fE38d92fB2"
        );
    }

    #[test]
    fn test_get_chain_info_when_ethereum_mainnet_should_return_correct_info() {
        let result = get_chain_info(1);

        assert_eq!(result.name, "Ethereum Mainnet");
        assert_eq!(result.native_token, "ETH");
    }

    #[test]
    fn test_get_chain_info_when_polygon_should_return_correct_info() {
        let result = get_chain_info(137);

        assert_eq!(result.name, "Polygon");
        assert_eq!(result.native_token, "MATIC");
    }

    #[test]
    fn test_get_chain_info_when_arbitrum_should_return_correct_info() {
        let result = get_chain_info(42161);

        assert_eq!(result.name, "Arbitrum One");
        assert_eq!(result.native_token, "ETH");
    }

    #[test]
    fn test_get_chain_info_when_optimism_should_return_correct_info() {
        let result = get_chain_info(10);

        assert_eq!(result.name, "Optimism");
        assert_eq!(result.native_token, "ETH");
    }

    #[test]
    fn test_get_chain_info_when_base_should_return_correct_info() {
        let result = get_chain_info(8453);

        assert_eq!(result.name, "Base");
        assert_eq!(result.native_token, "ETH");
    }

    #[test]
    fn test_get_chain_info_when_unknown_chain_should_return_default_info() {
        let result = get_chain_info(999999);

        assert_eq!(result.name, "Chain 999999");
        assert_eq!(result.native_token, "ETH");
    }

    #[test]
    fn test_get_popular_tokens_when_ethereum_mainnet_should_return_tokens() {
        let result = get_popular_tokens(1);

        assert!(result.contains_key("USDC"));
        assert!(result.contains_key("USDT"));
        assert!(result.contains_key("WETH"));
        assert!(result.contains_key("WBTC"));
        assert_eq!(result.len(), 4);
        assert_eq!(
            result.get("USDC").unwrap(),
            "0xA0b86a33E6411617D1A03e63BDD7d9F5eF9b6EA9"
        );
        assert_eq!(
            result.get("USDT").unwrap(),
            "0xdAC17F958D2ee523a2206206994597C13D831ec7"
        );
        assert_eq!(
            result.get("WETH").unwrap(),
            "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2"
        );
        assert_eq!(
            result.get("WBTC").unwrap(),
            "0x2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599"
        );
    }

    #[test]
    fn test_get_popular_tokens_when_polygon_should_return_tokens() {
        let result = get_popular_tokens(137);

        assert!(result.contains_key("USDC"));
        assert!(result.contains_key("WMATIC"));
        assert!(result.contains_key("WETH"));
        assert_eq!(result.len(), 3);
        assert_eq!(
            result.get("USDC").unwrap(),
            "0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174"
        );
        assert_eq!(
            result.get("WMATIC").unwrap(),
            "0x0d500B1d8E8eF31E21C99d1Db9A6444d3ADf1270"
        );
        assert_eq!(
            result.get("WETH").unwrap(),
            "0x7ceB23fD6bC0adD59E62ac25578270cFf1b9f619"
        );
    }

    #[test]
    fn test_get_popular_tokens_when_arbitrum_should_return_tokens() {
        let result = get_popular_tokens(42161);

        assert!(result.contains_key("USDC"));
        assert!(result.contains_key("WETH"));
        assert!(result.contains_key("ARB"));
        assert_eq!(result.len(), 3);
        assert_eq!(
            result.get("USDC").unwrap(),
            "0xaf88d065e77c8cC2239327C5EDb3A432268e5831"
        );
        assert_eq!(
            result.get("WETH").unwrap(),
            "0x82aF49447D8a07e3bd95BD0d56f35241523fBab1"
        );
        assert_eq!(
            result.get("ARB").unwrap(),
            "0x912CE59144191C1204E64559FE8253a0e49E6548"
        );
    }

    #[test]
    fn test_get_popular_tokens_when_unknown_chain_should_return_default_tokens() {
        let result = get_popular_tokens(999999);

        assert!(result.contains_key("USDC"));
        assert!(result.contains_key("WETH"));
        assert_eq!(result.len(), 2);
        assert_eq!(
            result.get("USDC").unwrap(),
            "0xA0b86a33E6411617D1A03e63BDD7d9F5eF9b6EA9"
        );
        assert_eq!(
            result.get("WETH").unwrap(),
            "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2"
        );
    }

    #[test]
    fn test_get_popular_tokens_when_empty_chain_should_return_default_tokens() {
        let result = get_popular_tokens(0);

        assert!(result.contains_key("USDC"));
        assert!(result.contains_key("WETH"));
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_chain_info_debug_trait() {
        let chain_info = ChainInfo {
            name: "Test Chain".to_string(),
            native_token: "TEST".to_string(),
        };

        let debug_output = format!("{:?}", chain_info);
        assert!(debug_output.contains("Test Chain"));
        assert!(debug_output.contains("TEST"));
    }

    // Integration test for display_native_balance (simulated)
    #[tokio::test]
    async fn test_display_native_balance_error_path() {
        use indicatif::{ProgressBar, ProgressStyle};

        let chain_info = ChainInfo {
            name: "Test Chain".to_string(),
            native_token: "ETH".to_string(),
        };
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
                .template("{spinner:.green} {msg}")
                .unwrap(),
        );

        // Test with invalid address to trigger error path
        let config = riglr_config::Config::from_env();
        let context = riglr_core::provider::ApplicationContext::from_config(&config);
        let result = display_native_balance("invalid_address", &chain_info, &pb, &context).await;

        // Should not panic and should return Ok (error handling is internal)
        assert!(result.is_ok());
    }

    // Integration test for display_token_balances (simulated)
    #[tokio::test]
    async fn test_display_token_balances_error_path() {
        use indicatif::{ProgressBar, ProgressStyle};

        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
                .template("{spinner:.green} {msg}")
                .unwrap(),
        );

        // Test with invalid address to trigger error path
        let config = riglr_config::Config::from_env();
        let context = riglr_core::provider::ApplicationContext::from_config(&config);
        let result = display_token_balances("invalid_address", 1, &pb, &context).await;

        // Should not panic and should return Ok (error handling is internal)
        assert!(result.is_ok());
    }

    // Integration test for display_uniswap_quote (simulated)
    #[tokio::test]
    async fn test_display_uniswap_quote_error_path() {
        use indicatif::{ProgressBar, ProgressStyle};

        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
                .template("{spinner:.green} {msg}")
                .unwrap(),
        );

        // This will likely trigger the error path due to network/API issues
        let config = riglr_config::Config::from_env();
        let context = riglr_core::provider::ApplicationContext::from_config(&config);
        let result = display_uniswap_quote(&pb, &context).await;

        // Should not panic and should return Ok (error handling is internal)
        assert!(result.is_ok());
    }

    // Test the edge cases in popular tokens function
    #[test]
    fn test_get_popular_tokens_edge_cases() {
        // Test with maximum u64 value
        let result = get_popular_tokens(u64::MAX);
        assert_eq!(result.len(), 2); // Should fall back to default

        // Test with commonly confused chain IDs
        let result_bsc = get_popular_tokens(56); // BSC
        assert_eq!(result_bsc.len(), 2); // Should fall back to default

        let result_avax = get_popular_tokens(43114); // Avalanche
        assert_eq!(result_avax.len(), 2); // Should fall back to default
    }

    // Test ChainInfo struct creation and fields
    #[test]
    fn test_chain_info_creation() {
        let chain_info = ChainInfo {
            name: "Custom Chain".to_string(),
            native_token: "CUSTOM".to_string(),
        };

        assert_eq!(chain_info.name, "Custom Chain");
        assert_eq!(chain_info.native_token, "CUSTOM");
    }

    // Test with empty strings
    #[test]
    fn test_chain_info_with_empty_strings() {
        let chain_info = ChainInfo {
            name: "".to_string(),
            native_token: "".to_string(),
        };

        assert_eq!(chain_info.name, "");
        assert_eq!(chain_info.native_token, "");
    }

    // Test HashMap behavior in get_popular_tokens
    #[test]
    fn test_get_popular_tokens_hashmap_behavior() {
        let tokens = get_popular_tokens(1);

        // Test that we can iterate over the HashMap
        let mut count = 0;
        for (symbol, address) in &tokens {
            assert!(!symbol.is_empty());
            assert!(!address.is_empty());
            assert!(address.starts_with("0x"));
            count += 1;
        }
        assert_eq!(count, 4);

        // Test that HashMap operations work
        let mut mutable_tokens = tokens.clone();
        mutable_tokens.insert("TEST".to_string(), "0x123".to_string());
        assert_eq!(mutable_tokens.len(), 5);
    }

    // Test all chain ID branches in get_chain_info
    #[test]
    fn test_get_chain_info_all_branches() {
        let test_cases = vec![
            (1, "Ethereum Mainnet", "ETH"),
            (137, "Polygon", "MATIC"),
            (42161, "Arbitrum One", "ETH"),
            (10, "Optimism", "ETH"),
            (8453, "Base", "ETH"),
            (999, "Chain 999", "ETH"), // Default case
        ];

        for (chain_id, expected_name, expected_token) in test_cases {
            let result = get_chain_info(chain_id);
            assert_eq!(result.name, expected_name);
            assert_eq!(result.native_token, expected_token);
        }
    }

    // Test all chain ID branches in get_popular_tokens
    #[test]
    fn test_get_popular_tokens_all_branches() {
        // Test Ethereum (case 1)
        let eth_tokens = get_popular_tokens(1);
        assert_eq!(eth_tokens.len(), 4);

        // Test Polygon (case 137)
        let poly_tokens = get_popular_tokens(137);
        assert_eq!(poly_tokens.len(), 3);

        // Test Arbitrum (case 42161)
        let arb_tokens = get_popular_tokens(42161);
        assert_eq!(arb_tokens.len(), 3);

        // Test default case
        let default_tokens = get_popular_tokens(99999);
        assert_eq!(default_tokens.len(), 2);
    }
}
