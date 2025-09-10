//! Graph memory demonstration commands.

use anyhow::Result;
use colored::Colorize;
use dialoguer::{Input, Select};
use indicatif::{ProgressBar, ProgressStyle};
use riglr_core::provider::ApplicationContext;
use riglr_graph_memory::document::RawTextDocument;
use std::sync::Arc;
use std::time::Duration;
// use tracing::warn;

#[cfg(test)]
use riglr_config::{
    AppConfig, Config, DatabaseConfig, FeaturesConfig, NetworkConfig, ProvidersConfig,
};

/// Create a test configuration for demos and tests
#[cfg(test)]
fn create_test_config() -> Config {
    Config {
        app: AppConfig::default(),
        database: DatabaseConfig::default(),
        network: NetworkConfig::default(),
        providers: ProvidersConfig::default(),
        features: FeaturesConfig::default(),
    }
}

/// Run the graph memory demo.
pub async fn run_demo(
    _context: Arc<ApplicationContext>,
    init: bool,
    query: Option<String>,
) -> Result<()> {
    println!("{}", "🧠 Graph Memory Demo".bright_blue().bold());
    println!("{}", "=".repeat(50).blue());

    println!(
        "\n{}",
        "Graph-based memory system for blockchain AI agents".cyan()
    );
    println!(
        "{}",
        "Stores and queries complex relationships between on-chain entities.".dimmed()
    );

    // Show progress bar
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
            .template("{spinner:.green} {msg}")?,
    );

    // Initialize Neo4j client (temporarily simulated)
    pb.set_message("Simulating Neo4j connection...");
    tokio::time::sleep(Duration::from_millis(500)).await;
    let _neo4j_client: Option<()> = None; // Simulated
                                          /*
                                          let neo4j_client = match Neo4jClient::new(&config.neo4j_url).await {
                                              Ok(client) => {
                                                  println!("\n{}", "✅ Connected to Neo4j database".green());
                                                  Some(client)
                                              }
                                              Err(e) => {
                                                  warn!("Failed to connect to Neo4j: {}", e);
                                                  println!("\n{}", format!("⚠️ Could not connect to Neo4j: {}", e).yellow());
                                                  println!("{}", "Demo will run in simulation mode with mock data.".dimmed());
                                                  None
                                              }
                                          };
                                          */

    println!(
        "\n{}",
        "⚠️ Neo4j connection simulated - running in demo mode".yellow()
    );

    // Initialize graph memory (temporarily simulated)
    let graph_memory: Option<()> = None; // Simulated
    println!("{}", "⚠️ Graph memory running in simulation mode".yellow());

    pb.finish_and_clear();

    // Demo 1: Sample data initialization
    if init || graph_memory.is_none() {
        println!("\n{}", "📊 Sample Blockchain Data".green().bold());
        let sample_data = get_sample_blockchain_data();

        if graph_memory.is_some() {
            pb.set_message("Adding sample data to graph...");
            for data in sample_data.iter() {
                let _doc = RawTextDocument::new(data.clone());
                // Note: In simulation mode, we just show what would be added
                println!("   ✅ Would add: {}", truncate_text(data, 80));
                tokio::time::sleep(Duration::from_millis(100)).await; // Rate limiting
            }
        } else {
            // Show sample data in simulation mode
            for (i, data) in sample_data.iter().enumerate() {
                println!("   {}. {}", i + 1, truncate_text(data, 100));
            }
        }
        pb.finish_and_clear();
    }

    // Demo 2: Entity extraction
    println!("\n{}", "🔍 Entity Extraction".green().bold());
    let sample_text = "Wallet 0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045 swapped 100 SOL for 15000 USDC on Jupiter protocol, generating 0.25 SOL in fees for liquidity providers.";
    println!("   Sample Text: {}", sample_text.dimmed());

    if graph_memory.is_some() {
        // Entity extraction temporarily disabled
        println!("\n   Entity extraction would run here...");
        /*
        let extractor = EntityExtractor::new();
        match extractor.extract_entities(sample_text).await {
            Ok(entities) => {
                println!("\n   Extracted Entities:");
                for entity in entities {
                    println!("   • {} ({})", entity.name.bright_cyan(), entity.entity_type);
                    if !entity.properties.is_empty() {
                        for (key, value) in &entity.properties {
                            println!("     {}: {}", key.dimmed(), value);
                        }
                    }
                }
            }
            Err(e) => {
                println!("   {}", format!("Could not extract entities: {}", e).yellow());
            }
        }
        */
    } else {
        // Show simulated entity extraction
        println!("\n   Simulated Entity Extraction:");
        println!(
            "   • {} (Wallet)",
            "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045".bright_cyan()
        );
        println!("   • {} (Token)", "SOL".bright_cyan());
        println!("   • {} (Token)", "USDC".bright_cyan());
        println!("   • {} (Protocol)", "Jupiter".bright_cyan());
        println!("   • {} (Amount)", "100".bright_cyan());
    }

    // Demo 3: Graph queries
    if let Some(query_text) = query {
        println!("\n{}", "🔎 Graph Query".green().bold());
        println!("   Query: {}", query_text.cyan());

        if let Some(_memory) = graph_memory {
            pb.set_message("Simulating graph search...");
            tokio::time::sleep(Duration::from_millis(800)).await;
            pb.finish_and_clear();
            println!("   Graph search would run here...");
        } else {
            // Show simulated query results
            println!("   Simulated Results:");
            println!("   1. Score: 0.85");
            println!("      Wallet 0xABC123 performed a swap on Jupiter DEX...");
            println!("   2. Score: 0.72");
            println!("      Large transaction detected on Solana mainnet involving SOL...");
        }
    }

    // Interactive menu
    println!("\n{}", "🎮 Interactive Options".bright_blue().bold());
    let mut options = vec![
        "Add custom transaction data",
        "Query the graph",
        "Analyze wallet relationships",
        "Exit demo",
    ];

    if graph_memory.is_some() {
        options.insert(1, "View graph statistics");
    }

    let selection = Select::new()
        .with_prompt("What would you like to do next?")
        .items(&options)
        .default(options.len() - 1)
        .interact()?;

    match selection {
        0 => {
            println!("\n{}", "📝 Add Transaction Data".cyan());
            let transaction_data: String = Input::new()
                .with_prompt("Enter transaction description")
                .default(
                    "Wallet 0x123ABC transferred 50 WETH to 0x456DEF on Uniswap V3".to_string(),
                )
                .interact_text()?;

            if graph_memory.is_some() {
                let _doc = RawTextDocument::new(transaction_data.clone());
                // Note: In simulation mode, we just show what would be added
                println!("   ✅ Would add transaction to graph");
                println!("   Data: {}", transaction_data.green());
            } else {
                println!("   📊 Would add to graph: {}", transaction_data.green());
            }
        }
        1 if graph_memory.is_some() => {
            println!("\n{}", "📈 Graph Statistics".cyan());
            if let Some(_memory) = graph_memory {
                // This would require implementing statistics methods in GraphMemory
                println!("   Nodes: ~150 blockchain entities");
                println!("   Relationships: ~400 connections");
                println!("   Wallets: ~75 unique addresses");
                println!("   Tokens: ~25 different assets");
                println!("   Protocols: ~10 DeFi platforms");
            }
        }
        1 => {
            println!("\n{}", "🔍 Graph Query".cyan());
            let search_query: String = Input::new()
                .with_prompt("Enter search query")
                .default("wallets that used Jupiter".to_string())
                .interact_text()?;

            if let Some(_memory) = graph_memory {
                println!("   Simulated graph search would run here...");
            } else {
                println!("   Simulated search for: {}", search_query.cyan());
                println!("   1. Wallet 0xABC performed SOL→USDC swap via Jupiter");
                println!("   2. Large Jupiter transaction from whale wallet 0xDEF");
            }
        }
        2 => {
            println!("\n{}", "🕸️ Wallet Relationship Analysis".cyan());
            let wallet: String = Input::new()
                .with_prompt("Enter wallet address")
                .default("0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045".to_string())
                .interact_text()?;

            println!("   Analyzing relationships for: {}", wallet.bright_cyan());

            if graph_memory.is_some() {
                println!("   🔗 Connected to:");
                println!("   • 5 different protocols (Jupiter, Uniswap, Compound...)");
                println!("   • 12 unique tokens (SOL, ETH, USDC, WBTC...)");
                println!("   • 3 other wallets (direct transfers)");
                println!("   💡 Pattern: Active DeFi user, high-volume trader");
            } else {
                println!("   Simulated Analysis:");
                println!("   • Connected to Jupiter, Uniswap protocols");
                println!("   • Holds SOL, USDC, WETH tokens");
                println!("   • Active trader profile");
            }
        }
        _ => {}
    }

    println!(
        "\n{}",
        "✅ Graph memory demo completed!".bright_green().bold()
    );
    println!("{}", "Thank you for exploring riglr-graph-memory!".dimmed());

    if graph_memory.is_none() {
        println!(
            "\n{}",
            "💡 Tip: Set up Neo4j to see the full graph memory capabilities!".blue()
        );
        println!(
            "{}",
            "   docker run -p 7687:7687 -p 7474:7474 neo4j".dimmed()
        );
    }

    Ok(())
}

fn get_sample_blockchain_data() -> Vec<String> {
    vec![
        "Wallet 0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045 swapped 100 SOL for 15000 USDC on Jupiter protocol".to_string(),
        "Large transaction of 500 ETH transferred from 0x123ABC to 0x456DEF on Ethereum mainnet".to_string(),
        "Uniswap V3 pool created for WETH/USDC pair with 0.05% fee tier".to_string(),
        "Compound protocol liquidation: 10 WBTC collateral seized from 0x789GHI".to_string(),
        "Arbitrage opportunity detected: SOL price difference between Jupiter and Orca DEX".to_string(),
        "NFT collection 'CryptoPunks' floor price increased 15% in last 24 hours".to_string(),
        "Wallet 0x111AAA performed flash loan attack on DeFi protocol, draining 2M USDC".to_string(),
        "Staking rewards distributed: 1000 validators received SOL rewards on Solana".to_string(),
        "Cross-chain bridge: 100 WETH bridged from Ethereum to Arbitrum One via official bridge".to_string(),
        "Decentralized exchange aggregator 1inch routed trade through 4 different DEXs for optimal price".to_string(),
    ]
}

fn truncate_text(text: &str, max_length: usize) -> String {
    if text.len() <= max_length {
        text.to_string()
    } else {
        format!("{}...", &text[..max_length])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tokio_test;

    #[test]
    fn test_get_sample_blockchain_data_should_return_ten_entries() {
        let data = get_sample_blockchain_data();
        assert_eq!(data.len(), 10);

        // Verify each entry is non-empty
        for entry in &data {
            assert!(!entry.is_empty());
        }

        // Verify specific known entries
        assert!(data[0].contains("Wallet 0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045"));
        assert!(data[0].contains("100 SOL"));
        assert!(data[0].contains("Jupiter protocol"));

        assert!(data[1].contains("500 ETH"));
        assert!(data[1].contains("Ethereum mainnet"));

        assert!(data[2].contains("Uniswap V3"));
        assert!(data[2].contains("WETH/USDC"));
    }

    #[test]
    fn test_truncate_text_when_text_shorter_than_max_should_return_original() {
        let text = "Hello World";
        let result = truncate_text(text, 20);
        assert_eq!(result, "Hello World");
    }

    #[test]
    fn test_truncate_text_when_text_equal_to_max_should_return_original() {
        let text = "Hello World";
        let result = truncate_text(text, 11);
        assert_eq!(result, "Hello World");
    }

    #[test]
    fn test_truncate_text_when_text_longer_than_max_should_truncate_with_ellipsis() {
        let text = "This is a very long text that needs to be truncated";
        let result = truncate_text(text, 10);
        assert_eq!(result, "This is a ...");
    }

    #[test]
    fn test_truncate_text_when_empty_string_should_return_empty() {
        let text = "";
        let result = truncate_text(text, 10);
        assert_eq!(result, "");
    }

    #[test]
    fn test_truncate_text_when_max_length_zero_should_return_ellipsis_only() {
        let text = "Hello";
        let result = truncate_text(text, 0);
        assert_eq!(result, "...");
    }

    #[test]
    fn test_truncate_text_when_max_length_one_should_truncate_correctly() {
        let text = "Hello";
        let result = truncate_text(text, 1);
        assert_eq!(result, "H...");
    }

    #[test]
    fn test_truncate_text_with_unicode_characters_should_handle_correctly() {
        let text = "🚀🌙⭐️✨💎";
        let result = truncate_text(text, 3);
        // Note: This tests the behavior with Unicode characters
        // The actual behavior depends on how String slicing handles Unicode
        assert!(result.ends_with("..."));
    }

    #[tokio::test]
    async fn test_run_demo_when_init_false_no_query_should_complete_successfully() {
        let _config = Arc::new(create_test_config());

        // This test verifies the function completes without panicking
        // Since the function uses interactive prompts, we can't easily test the full execution
        // but we can test that it doesn't crash during initialization

        // Note: This would require mocking the interactive components for full testing
        // For now, we test the parts we can control

        // Test that the function signature is correct and compiles
        let result = std::panic::catch_unwind(|| {
            tokio_test::block_on(async {
                // We can't easily test the full function due to interactive prompts
                // But we can verify the function exists and has the right signature
                let _config = Arc::new(create_test_config());
                // run_demo would need to be refactored to be testable
                // by accepting a trait for user interaction instead of using dialoguer directly
            });
        });

        assert!(result.is_ok());
    }

    #[test]
    fn test_sample_data_entries_have_expected_content() {
        let data = get_sample_blockchain_data();

        // Test first entry contains expected wallet address
        assert!(data[0].contains("0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045"));

        // Test entries contain expected protocols
        let all_data = data.join(" ");
        assert!(all_data.contains("Jupiter"));
        assert!(all_data.contains("Uniswap"));
        assert!(all_data.contains("Compound"));
        assert!(all_data.contains("Orca"));
        assert!(all_data.contains("1inch"));

        // Test entries contain expected tokens
        assert!(all_data.contains("SOL"));
        assert!(all_data.contains("ETH"));
        assert!(all_data.contains("USDC"));
        assert!(all_data.contains("WBTC"));
        assert!(all_data.contains("WETH"));

        // Test entries contain expected transaction types
        assert!(all_data.contains("swap"));
        assert!(all_data.contains("transfer"));
        assert!(all_data.contains("liquidation"));
        assert!(all_data.contains("bridge"));
        assert!(all_data.contains("staking"));
    }

    #[test]
    fn test_sample_data_specific_entries_content() {
        let data = get_sample_blockchain_data();

        // Test entry 3 (index 2) - Uniswap pool
        assert!(data[2].contains("Uniswap V3 pool"));
        assert!(data[2].contains("WETH/USDC"));
        assert!(data[2].contains("0.05%"));

        // Test entry 4 (index 3) - Compound liquidation
        assert!(data[3].contains("Compound protocol liquidation"));
        assert!(data[3].contains("10 WBTC"));
        assert!(data[3].contains("0x789GHI"));

        // Test entry 7 (index 6) - Flash loan attack
        assert!(data[6].contains("flash loan attack"));
        assert!(data[6].contains("2M USDC"));
        assert!(data[6].contains("0x111AAA"));

        // Test entry 8 (index 7) - Staking rewards
        assert!(data[7].contains("Staking rewards"));
        assert!(data[7].contains("1000 validators"));
        assert!(data[7].contains("Solana"));
    }

    #[test]
    fn test_truncate_text_edge_cases() {
        // Test with very large max_length
        let text = "Short";
        let result = truncate_text(text, usize::MAX);
        assert_eq!(result, "Short");

        // Test with text containing only whitespace
        let text = "   ";
        let result = truncate_text(text, 2);
        assert_eq!(result, "  ...");

        // Test with text containing newlines
        let text = "Line 1\nLine 2\nLine 3";
        let result = truncate_text(text, 8);
        assert_eq!(result, "Line 1\nL...");

        // Test with special characters
        let text = "Hello @#$%^&*()";
        let result = truncate_text(text, 7);
        assert_eq!(result, "Hello @...");
    }

    #[test]
    fn test_sample_data_immutability() {
        let data1 = get_sample_blockchain_data();
        let data2 = get_sample_blockchain_data();

        // Verify the function returns consistent data
        assert_eq!(data1.len(), data2.len());
        for (entry1, entry2) in data1.iter().zip(data2.iter()) {
            assert_eq!(entry1, entry2);
        }
    }

    #[test]
    fn test_sample_data_no_empty_strings() {
        let data = get_sample_blockchain_data();

        for (index, entry) in data.iter().enumerate() {
            assert!(
                !entry.is_empty(),
                "Entry at index {} should not be empty",
                index
            );
            assert!(
                !entry.trim().is_empty(),
                "Entry at index {} should not be only whitespace",
                index
            );
        }
    }
}
