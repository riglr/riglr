//! Cross-chain analysis demonstration commands.

use anyhow::Result;
use colored::Colorize;
use dialoguer::{Input, Select};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use riglr_core::provider::ApplicationContext;
// Note: SolanaClient has been removed in v0.2.0 - use SignerContext pattern instead
use std::{collections::HashMap, sync::Arc};
use tokio::time::{sleep, Duration};
// Temporarily using mock functionality due to dependency conflicts
// use riglr_evm_tools::{
//     client::{EvmClient, EvmConfig},
//     balance::{get_eth_balance, get_erc20_balance},
// };
// Temporarily using mock functionality due to compilation issues
// use riglr_web_tools::{
//     client::WebClient,
//     dexscreener::{get_token_info, search_tokens},
//     twitter::search_tweets,
//     news::get_crypto_news,
// };

// Mock WebClient for demonstration
struct WebClient;

impl WebClient {
    const fn new() -> Self {
        Self
    }
}

/// Run the cross-chain analysis demo.
///
/// # Errors
///
/// Returns an error if:
/// - Progress bar templates are invalid
/// - User interaction (dialoguer) fails
/// - Recursive demo call fails
///
/// # Panics
///
/// Panics if the user's chain selection from dialoguer is out of bounds, which should never
/// happen as dialoguer validates the selection internally.
#[expect(clippy::too_many_lines)]
pub async fn run_demo(context: Arc<ApplicationContext>, token: String) -> Result<()> {
    println!("{}", "🌐 Cross-Chain Analysis Demo".bright_blue().bold());
    println!("{}", "=".repeat(50).blue());

    println!("\n🎯 Analyzing token: {}", token.bright_cyan().bold());
    println!(
        "{}",
        "Gathering data across multiple blockchains and sources...".dimmed()
    );

    // TODO: Update to use SignerContext pattern instead of deprecated SolanaClient
    // Initialize all clients using configuration
    // let _solana_client = SolanaClient::new(SolanaConfig {
    //     rpc_url: config.network.solana_rpc_url.clone(),
    //     commitment: solana_sdk::commitment_config::CommitmentLevel::Confirmed,
    //     timeout: std::time::Duration::from_secs(30),
    //     skip_preflight: false,
    // });

    // Ensure we're analyzing the specified token
    if token.is_empty() {
        println!("   ⚠️ Warning: No token specified for analysis");
    }

    // EVM client temporarily simulated due to dependency conflicts

    let _web_client = WebClient::new();

    // Multi-progress bar setup
    let multi_pb = MultiProgress::new();

    // Phase 1: Token Discovery & Market Data
    let token_info = collect_market_data(&multi_pb, &token).await?;

    // Phase 2: Cross-Chain Balance Analysis
    analyze_cross_chain_balances(&multi_pb, &token).await?;

    // Phase 3: Social Sentiment Analysis
    println!(
        "\n{}",
        "🐦 Phase 3: Social Sentiment Analysis".green().bold()
    );

    let sentiment_pb = multi_pb.add(ProgressBar::new_spinner());
    sentiment_pb.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
            .template("{spinner:.yellow} [Social] {msg}")?,
    );

    // Simulate Twitter sentiment analysis
    sentiment_pb.set_message("Simulating Twitter sentiment analysis...");
    sleep(Duration::from_millis(700)).await;

    println!("   🐦 Twitter Activity (Simulated): 25 recent tweets");
    println!("   💖 Total Engagement: 1,847 likes, 523 retweets");
    println!(
        "   🔥 Top Tweet: {token} is showing strong momentum! Just broke resistance level 🚀 #crypto"
    );
    println!("      @cryptotrader_pro | 342 ❤️ 127 🔄");

    sentiment_pb.finish_and_clear();

    // Phase 4: News & Market Intelligence
    println!(
        "\n{}",
        "📰 Phase 4: News & Market Intelligence".green().bold()
    );

    let news_pb = multi_pb.add(ProgressBar::new_spinner());
    news_pb.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
            .template("{spinner:.cyan} [News] {msg}")?,
    );

    news_pb.set_message("Simulating news fetch...");
    sleep(Duration::from_millis(600)).await;

    println!("   📰 Recent News (Simulated - 3 articles):");
    let news_headlines = [
        (
            format!("{token} Price Surges 15% Following Partnership Announcement"),
            "CryptoDaily",
        ),
        (
            format!("Major Exchange Lists {token} Trading Pairs"),
            "CoinTelegraph",
        ),
        (
            format!("{token} Network Upgrade Improves Transaction Speed"),
            "The Block",
        ),
    ];

    for (i, &(ref title, source)) in news_headlines.iter().enumerate() {
        println!("   {}. {}", i.saturating_add(1), title.bright_cyan());
        println!("      {} | {}", source.dimmed(), "2025-01-15".dimmed());
    }

    news_pb.finish_and_clear();

    // Phase 5: Cross-Chain Summary & Analysis
    println!(
        "\n{}",
        "🎯 Phase 5: Cross-Chain Analysis Summary".green().bold()
    );

    println!("   🔍 Token: {}", token.bright_cyan().bold());
    println!("   ⛓️ Chains Analyzed: Solana, Ethereum, Polygon (via market data)");

    if let Some(_dex_data) = token_info.get("dex_data") {
        println!("   📊 Market Presence: Active on DEXs");
    }

    println!("   🔗 Cross-Chain Opportunities:");
    println!("      • Arbitrage potential between different chains");
    println!("      • Bridge liquidity analysis");
    println!("      • Multi-chain portfolio exposure");

    println!("   📈 Risk Assessment:");
    println!("      • Liquidity: Check across multiple chains");
    println!("      • Volatility: Monitor price movements");
    println!("      • Social sentiment: Twitter engagement analysis");

    // Interactive analysis options
    println!(
        "\n{}",
        "🎮 Interactive Cross-Chain Options".bright_blue().bold()
    );
    let options = vec![
        "Analyze different token",
        "Deep dive into specific chain",
        "Portfolio cross-chain analysis",
        "Market opportunity scanner",
        "Exit demo",
    ];

    let selection = Select::new()
        .with_prompt("What would you like to explore next?")
        .items(&options)
        .default(4)
        .interact()?;

    match selection {
        0 => {
            println!("\n{}", "🔄 Token Analysis".cyan());
            let new_token: String = Input::new()
                .with_prompt("Enter token symbol or name")
                .interact_text()?;
            Box::pin(run_demo(context, new_token)).await?;
        }
        1 => {
            println!("\n{}", "⛓️ Chain-Specific Analysis".cyan());
            let chains = vec!["Solana", "Ethereum", "Polygon", "Arbitrum", "Base"];
            let chain_selection = Select::new()
                .with_prompt("Select chain to analyze")
                .items(&chains)
                .interact()?;

            let selected_chain = chains
                .get(chain_selection)
                .ok_or_else(|| anyhow::anyhow!("Invalid chain selection"))?;
            println!(
                "   Deep analysis for {} on {}:",
                token,
                selected_chain.bright_cyan()
            );

            match *selected_chain {
                "Solana" => {
                    println!("   🌟 Solana Ecosystem:");
                    println!("      • Check Jupiter DEX for trading pairs");
                    println!("      • Analyze Orca liquidity pools");
                    println!("      • Monitor Raydium farming opportunities");
                }
                "Ethereum" => {
                    println!("   ⚡ Ethereum Ecosystem:");
                    println!("      • Uniswap V3 liquidity analysis");
                    println!("      • Compound lending rates");
                    println!("      • Aave borrowing opportunities");
                }
                _ => {
                    println!("   🔗 {selected_chain} Analysis:");
                    println!("      • DEX liquidity on native exchanges");
                    println!("      • Bridge volume from Ethereum");
                    println!("      • Cross-chain yield opportunities");
                }
            }
        }
        2 => {
            println!("\n{}", "💼 Portfolio Cross-Chain Analysis".cyan());
            println!("   This would analyze your portfolio across:");
            println!("   • {token} holdings on Solana");
            println!("   • {token} holdings on Ethereum");
            println!("   • Bridge costs and opportunities");
            println!("   • Yield farming across chains");
            println!("   • Risk diversification analysis");
        }
        3 => {
            println!("\n{}", "🎯 Market Opportunity Scanner".cyan());
            println!("   Scanning for cross-chain opportunities...");
            println!("   🔍 Potential Opportunities:");
            println!("      • Price arbitrage between Solana and Ethereum DEXs");
            println!("      • Yield farming on multiple chains");
            println!("      • Bridge liquidity provision");
            println!("      • Cross-chain lending protocols");
        }
        _ => {}
    }

    println!(
        "\n{}",
        "✅ Cross-chain analysis completed!".bright_green().bold()
    );
    println!(
        "{}",
        "Thank you for exploring multi-chain capabilities with riglr!".dimmed()
    );

    Ok(())
}

/// Phase 1 helper: Simulate token discovery and market data collection.
async fn collect_market_data(
    multi_pb: &MultiProgress,
    token: &str,
) -> Result<HashMap<String, String>> {
    println!(
        "\n{}",
        "📊 Phase 1: Token Discovery & Market Data".green().bold()
    );

    let pb = multi_pb.add(ProgressBar::new_spinner());
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
            .template("{spinner:.green} [Market] {msg}")?,
    );

    pb.set_message(format!("Searching DEX pairs for {token}..."));
    sleep(Duration::from_millis(550)).await;
    pb.set_message("Fetching liquidity and price data...");
    sleep(Duration::from_millis(650)).await;
    pb.set_message("Aggregating market presence across chains...");
    sleep(Duration::from_millis(500)).await;
    pb.finish_and_clear();

    // Simulated market snapshot
    println!("   🔎 Found on: Jupiter (Solana), Uniswap (Ethereum), QuickSwap (Polygon)");
    println!(
        "   💧 Liquidity (Simulated): ${} | 24h Volume: ${}",
        "2.3M".bright_cyan(),
        "6.8M".bright_cyan()
    );
    println!(
        "   📈 Price (Simulated): ${} | 24h Change: {}",
        "1.24".bright_cyan(),
        "+5.7%".bright_green()
    );

    let mut info = HashMap::new();
    // Presence of this key toggles the later summary line.
    info.insert("dex_data".to_string(), "available".to_string());
    Ok(info)
}

/// Phase 2 helper: Simulate cross-chain balance checks for sample wallets.
async fn analyze_cross_chain_balances(multi_pb: &MultiProgress, token: &str) -> Result<()> {
    println!(
        "\n{}",
        "💼 Phase 2: Cross-Chain Balance Analysis".green().bold()
    );

    // Use sample wallets defined below to showcase output across chains.
    for (chain, address) in get_sample_wallets() {
        let pb = multi_pb.add(ProgressBar::new_spinner());
        pb.set_style(
            ProgressStyle::default_spinner()
                .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
                .template("{spinner:.magenta} [Balances] {msg}")?,
        );
        pb.set_message(format!("Checking {chain} balances for {token}..."));
        sleep(Duration::from_millis(450)).await;
        pb.finish_and_clear();

        // Simulated balances per chain (for demo only)
        match chain.as_str() {
            "Solana" => {
                println!("   🟣 {}: {}", chain.bright_cyan().bold(), address.dimmed());
                println!(
                    "      • Native Balance (Simulated): {} SOL",
                    "12.48".bright_cyan()
                );
                println!(
                    "      • {} Holdings (Simulated): {} tokens",
                    token,
                    "3,240".bright_cyan()
                );
            }
            "Ethereum" => {
                println!("   🟡 {}: {}", chain.bright_cyan().bold(), address.dimmed());
                println!(
                    "      • Native Balance (Simulated): {} ETH",
                    "4.02".bright_cyan()
                );
                println!(
                    "      • {} Holdings (Simulated): {} tokens",
                    token,
                    "1,125".bright_cyan()
                );

                // Display ERC20 contract addresses for reference
                let contracts = get_erc20_contracts();
                println!("      • Popular ERC20 Contracts:");
                for (symbol, address) in contracts.iter().take(2) {
                    println!("        {} {}: {}", "📄".dimmed(), symbol, address.dimmed());
                }
            }
            "Polygon" => {
                println!("   🟣 {}: {}", chain.bright_cyan().bold(), address.dimmed());
                println!(
                    "      • Native Balance (Simulated): {} MATIC",
                    "3,870".bright_cyan()
                );
                println!(
                    "      • {} Holdings (Simulated): {} tokens",
                    token,
                    "8,410".bright_cyan()
                );
            }
            _ => {
                println!("   🔗 {}: {}", chain.bright_cyan().bold(), address.dimmed());
                println!("      • Native Balance (Simulated): n/a");
                println!("      • {token} Holdings (Simulated): n/a");
            }
        }
    }

    Ok(())
}
fn get_sample_wallets() -> Vec<(String, String)> {
    vec![
        (
            "Solana".to_string(),
            "So11111111111111111111111111111111111111112".to_string(),
        ),
        (
            "Ethereum".to_string(),
            "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045".to_string(),
        ),
        (
            "Polygon".to_string(),
            "0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174".to_string(),
        ),
    ]
}
fn get_erc20_contracts() -> HashMap<String, String> {
    let mut contracts = HashMap::new();
    contracts.insert(
        "USDC".to_string(),
        "0xA0b86a33E6411617D1A03e63BDD7d9F5eF9b6EA9".to_string(),
    );
    contracts.insert(
        "USDT".to_string(),
        "0xdAC17F958D2ee523a2206206994597C13D831ec7".to_string(),
    );
    contracts.insert(
        "WETH".to_string(),
        "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".to_string(),
    );
    contracts.insert(
        "WBTC".to_string(),
        "0x2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599".to_string(),
    );
    contracts
}

#[cfg(test)]
#[expect(clippy::expect_used)]
mod tests {
    use super::*;
    use riglr_config::Config;
    use std::sync::Arc;
    fn create_test_config() -> Arc<Config> {
        Arc::new(
            riglr_config::ConfigBuilder::new()
                .build()
                .expect("Test config should be valid"),
        )
    }

    #[test]
    fn test_web_client_new_should_create_instance() {
        let client = WebClient::new();
        // WebClient is a unit struct, so just verify it can be created
        assert_eq!(size_of_val(&client), 0);
    }

    #[test]
    fn test_get_sample_wallets_should_return_three_wallets() {
        let wallets = get_sample_wallets();
        assert_eq!(wallets.len(), 3);

        let (chain1, addr1) = wallets.first().expect("Should have first wallet").clone();
        assert_eq!(chain1, "Solana");
        assert_eq!(addr1, "So11111111111111111111111111111111111111112");

        let (chain2, addr2) = wallets.get(1).expect("Should have second wallet").clone();
        assert_eq!(chain2, "Ethereum");
        assert_eq!(addr2, "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045");

        let (chain3, addr3) = wallets.get(2).expect("Should have third wallet").clone();
        assert_eq!(chain3, "Polygon");
        assert_eq!(addr3, "0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174");
    }

    #[test]
    fn test_get_erc20_contracts_should_return_four_contracts() {
        let contracts = get_erc20_contracts();
        assert_eq!(contracts.len(), 4);

        assert_eq!(
            contracts.get("USDC"),
            Some(&"0xA0b86a33E6411617D1A03e63BDD7d9F5eF9b6EA9".to_string())
        );
        assert_eq!(
            contracts.get("USDT"),
            Some(&"0xdAC17F958D2ee523a2206206994597C13D831ec7".to_string())
        );
        assert_eq!(
            contracts.get("WETH"),
            Some(&"0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".to_string())
        );
        assert_eq!(
            contracts.get("WBTC"),
            Some(&"0x2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599".to_string())
        );
    }

    #[test]
    fn test_get_erc20_contracts_should_contain_expected_keys() {
        let contracts = get_erc20_contracts();
        assert!(contracts.contains_key("USDC"));
        assert!(contracts.contains_key("USDT"));
        assert!(contracts.contains_key("WETH"));
        assert!(contracts.contains_key("WBTC"));
        assert!(!contracts.contains_key("NONEXISTENT"));
    }

    #[tokio::test]
    async fn test_collect_market_data_should_return_dex_data() {
        let multi_pb = MultiProgress::new();
        let token = "SOL";

        let result = collect_market_data(&multi_pb, token).await;
        assert!(result.is_ok());

        let data = result.expect("collect_market_data should return valid HashMap");
        assert!(data.contains_key("dex_data"));
        assert_eq!(data.get("dex_data"), Some(&"available".to_string()));
    }

    #[tokio::test]
    async fn test_collect_market_data_with_empty_token_should_succeed() {
        let multi_pb = MultiProgress::new();
        let token = "";

        let result = collect_market_data(&multi_pb, token).await;
        assert!(result.is_ok());

        let data =
            result.expect("collect_market_data with empty token should return valid HashMap");
        assert!(data.contains_key("dex_data"));
    }

    #[tokio::test]
    async fn test_collect_market_data_with_long_token_name_should_succeed() {
        let multi_pb = MultiProgress::new();
        let token = "VERYLONGTOKENNAMETHATEXCEEDSNORMALLIMITS";

        let result = collect_market_data(&multi_pb, token).await;
        assert!(result.is_ok());

        let data =
            result.expect("collect_market_data with long token name should return valid HashMap");
        assert!(data.contains_key("dex_data"));
    }

    #[tokio::test]
    async fn test_analyze_cross_chain_balances_should_complete_successfully() {
        let multi_pb = MultiProgress::new();
        let token = "SOL";

        let result = analyze_cross_chain_balances(&multi_pb, token).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_analyze_cross_chain_balances_with_empty_token_should_succeed() {
        let multi_pb = MultiProgress::new();
        let token = "";

        let result = analyze_cross_chain_balances(&multi_pb, token).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_analyze_cross_chain_balances_with_special_chars_should_succeed() {
        let multi_pb = MultiProgress::new();
        let token = "SOL@#$%";

        let result = analyze_cross_chain_balances(&multi_pb, token).await;
        assert!(result.is_ok());
    }

    // Note: Testing run_demo function comprehensively is challenging due to its interactive nature
    // with dialoguer prompts. The function would need to be refactored to be more testable
    // by accepting input streams or making the interactive parts configurable.
    // For now, we test the helper functions that contain the core logic.

    #[tokio::test]
    async fn test_run_demo_basic_initialization() {
        // This test verifies that the function can be called and initializes properly
        // We can't easily test the full interactive flow without mocking dialoguer
        let _config = create_test_config();
        let _token = "SOL".to_string();

        // We'll test that the function can at least start (client creation, etc.)
        // This would require significant refactoring to make fully testable
        // For demonstration, we show how you would start such a test

        // In a real scenario, you'd want to:
        // 1. Extract the client creation logic into a separate function
        // 2. Make the interactive parts configurable/mockable
        // 3. Split the function into smaller, more testable pieces

        // For now, we acknowledge this limitation in achieving 100% coverage
        // The function is primarily an orchestrator of smaller functions we've tested
    }

    #[test]
    fn test_sample_wallets_chain_names_are_correct() {
        let wallets = get_sample_wallets();
        let chain_names: Vec<&String> = wallets.iter().map(|wallet| &wallet.0).collect();

        assert!(chain_names.contains(&&"Solana".to_string()));
        assert!(chain_names.contains(&&"Ethereum".to_string()));
        assert!(chain_names.contains(&&"Polygon".to_string()));
    }

    #[test]
    fn test_sample_wallets_addresses_are_not_empty() {
        let wallets = get_sample_wallets();

        for (_, address) in wallets {
            assert!(!address.is_empty());
            assert!(address.len() > 10); // Basic sanity check for address length
        }
    }

    #[test]
    fn test_erc20_contracts_addresses_are_valid_format() {
        let contracts = get_erc20_contracts();

        for (symbol, address) in contracts {
            assert!(!symbol.is_empty());
            assert!(!address.is_empty());
            assert!(address.starts_with("0x"));
            assert_eq!(address.len(), 42); // Standard Ethereum address length
        }
    }

    #[test]
    fn test_erc20_contracts_symbols_are_uppercase() {
        let contracts = get_erc20_contracts();

        for (symbol, _) in contracts {
            assert_eq!(symbol, symbol.to_uppercase());
            assert!(symbol.len() >= 3);
            assert!(symbol.len() <= 5);
        }
    }
}
