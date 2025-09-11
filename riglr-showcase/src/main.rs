//! # riglr-showcase
//!
//! Showcase application demonstrating the capabilities of the riglr ecosystem.
//!
//! This application serves as both a working example and a testing ground for
//! all riglr components, showing how to build sophisticated AI agents that
//! can interact with multiple blockchains, analyze market data, and maintain
//! complex memory systems.

use anyhow::Result;
use clap::{Parser, Subcommand};
use riglr_config::Config;
use riglr_core::provider::ApplicationContext;
use std::sync::Arc;
use tracing::info;

mod commands;

#[derive(Parser, Debug)]
#[command(name = "riglr-showcase")]
#[command(about = "Showcase application for the riglr ecosystem")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,

    /// Configuration file path
    #[arg(short, long, default_value = ".env")]
    config: String,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Run Solana tools demo
    Solana {
        /// Wallet address to analyze
        #[arg(short, long)]
        address: Option<String>,
    },
    /// Run EVM tools demo
    Evm {
        /// Wallet address to analyze
        #[arg(short, long)]
        address: Option<String>,

        /// Chain ID (1 for Ethereum, 137 for Polygon, etc.)
        #[arg(short, long, default_value = "1")]
        chain_id: u64,
    },
    /// Run web tools demo
    Web {
        /// Search query
        #[arg(short, long)]
        query: String,
    },
    /// Run graph memory demo
    Graph {
        /// Initialize with sample data
        #[arg(long)]
        init: bool,

        /// Query to run against the graph
        #[arg(short, long)]
        query: Option<String>,
    },
    /// Run full cross-chain analysis demo
    CrossChain {
        /// Token symbol to analyze (e.g., USDC, WETH)
        #[arg(short, long)]
        token: String,
    },
    /// Run multi-agent coordination demo
    Agents {
        /// Demo scenario to run (trading, risk, basic)
        #[arg(short, long, default_value = "trading")]
        scenario: String,
    },
    /// Interactive chat mode
    Interactive,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    init_logging(cli.verbose);

    tracing::info!("🚀 Initializing riglr-showcase with production-ready configuration...");

    // Phase 1.1: Centralize Configuration Loading
    // Load configuration once at application startup using Config::from_env()
    // This handles environment variables and .env files automatically
    let config = Arc::new(Config::from_env());

    tracing::info!("✅ Configuration loaded and validated successfully");

    // Phase 1.2: Establish and Inject ApplicationContext
    // Create the ApplicationContext from the loaded configuration
    let context = Arc::new(ApplicationContext::from_config(&config));

    // Initialize and inject all required clients into the context
    // 1. Solana RPC Client
    let solana_client = Arc::new(solana_client::rpc_client::RpcClient::new(
        config.network.solana_rpc_url.clone(),
    ));
    context.set_extension(solana_client);

    // 2. EVM Provider from alloy
    // Check if we have any EVM RPC URLs configured
    if let Some(evm_rpc_url) = config
        .network
        .get_rpc_url(&config.network.default_chain_id.to_string())
    {
        use alloy::providers::ProviderBuilder;

        let evm_provider = ProviderBuilder::new().on_http(evm_rpc_url.parse()?);
        context.set_extension(Arc::new(evm_provider));
    }

    // 3. Solana API Clients
    let api_clients = Arc::new(riglr_solana_tools::clients::ApiClients::new(
        &config.providers,
    ));
    context.set_extension(api_clients);

    // 4. Web Client
    use riglr_web_tools::client::HttpConfig;
    let web_client = Arc::new(riglr_web_tools::client::WebClient::with_config(
        HttpConfig::default(),
    )?);
    context.set_extension(web_client);

    // 5. Sentiment Analyzer implementation
    // TODO: LexiconSentimentAnalyzer::new() does not exist, needs to be fixed in Phase 2
    // let sentiment_analyzer = Arc::new(riglr_web_tools::news::LexiconSentimentAnalyzer::new());
    // context.set_extension(sentiment_analyzer);

    info!("Starting riglr-showcase v{}", env!("CARGO_PKG_VERSION"));

    // Run the appropriate command, passing the ApplicationContext instead of Config
    match cli.command {
        Commands::Solana { address } => {
            commands::solana::run_demo(context, address).await?;
        }
        Commands::Evm { address, chain_id } => {
            commands::evm::run_demo(context, address, chain_id).await?;
        }
        Commands::Web { query } => {
            commands::web::run_demo(context, query).await?;
        }
        Commands::Graph { init, query } => {
            commands::graph::run_demo(context, init, query).await?;
        }
        Commands::CrossChain { token } => {
            commands::cross_chain::run_demo(context, token).await?;
        }
        Commands::Agents { scenario } => {
            commands::agents::run_demo(context, scenario).await?;
        }
        Commands::Interactive => {
            commands::interactive::run_chat(context).await?;
        }
    }

    Ok(())
}

fn init_logging(verbose: bool) {
    use tracing_subscriber::{fmt, EnvFilter};

    let level = if verbose { "debug" } else { "info" };
    let filter_string = format!("riglr_showcase={},riglr_core={},riglr_solana_tools={},riglr_evm_tools={},riglr_web_tools={},riglr_graph_memory={}", level, level, level, level, level, level);

    fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(filter_string)),
        )
        .init();
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn test_cli_parsing_with_solana_command() {
        let args = vec![
            "riglr-showcase",
            "solana",
            "--address",
            "11111111111111111111111111111111",
        ];
        let cli = Cli::try_parse_from(args).unwrap();

        assert!(!cli.verbose);
        assert_eq!(cli.config, ".env");

        match cli.command {
            Commands::Solana { address } => {
                assert_eq!(
                    address,
                    Some("11111111111111111111111111111111".to_string())
                );
            }
            _ => panic!("Expected Solana command"),
        }
    }

    #[test]
    fn test_cli_parsing_with_solana_command_no_address() {
        let args = vec!["riglr-showcase", "solana"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Solana { address } => {
                assert_eq!(address, None);
            }
            _ => panic!("Expected Solana command"),
        }
    }

    #[test]
    fn test_cli_parsing_with_evm_command_full_args() {
        let args = vec![
            "riglr-showcase",
            "evm",
            "--address",
            "0x742637b2b12F8C53b48E6Cfe081e5B06DFe53FfE",
            "--chain-id",
            "137",
        ];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Evm { address, chain_id } => {
                assert_eq!(
                    address,
                    Some("0x742637b2b12F8C53b48E6Cfe081e5B06DFe53FfE".to_string())
                );
                assert_eq!(chain_id, 137);
            }
            _ => panic!("Expected Evm command"),
        }
    }

    #[test]
    fn test_cli_parsing_with_evm_command_default_chain_id() {
        let args = vec!["riglr-showcase", "evm"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Evm { address, chain_id } => {
                assert_eq!(address, None);
                assert_eq!(chain_id, 1); // default value
            }
            _ => panic!("Expected Evm command"),
        }
    }

    #[test]
    fn test_cli_parsing_with_web_command() {
        let args = vec!["riglr-showcase", "web", "--query", "blockchain analysis"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Web { query } => {
                assert_eq!(query, "blockchain analysis");
            }
            _ => panic!("Expected Web command"),
        }
    }

    #[test]
    fn test_cli_parsing_with_graph_command_init_only() {
        let args = vec!["riglr-showcase", "graph", "--init"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Graph { init, query } => {
                assert!(init);
                assert_eq!(query, None);
            }
            _ => panic!("Expected Graph command"),
        }
    }

    #[test]
    fn test_cli_parsing_with_graph_command_query_only() {
        let args = vec!["riglr-showcase", "graph", "--query", "find connections"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Graph { init, query } => {
                assert!(!init);
                assert_eq!(query, Some("find connections".to_string()));
            }
            _ => panic!("Expected Graph command"),
        }
    }

    #[test]
    fn test_cli_parsing_with_graph_command_both_args() {
        let args = vec!["riglr-showcase", "graph", "--init", "--query", "test query"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Graph { init, query } => {
                assert!(init);
                assert_eq!(query, Some("test query".to_string()));
            }
            _ => panic!("Expected Graph command"),
        }
    }

    #[test]
    fn test_cli_parsing_with_cross_chain_command() {
        let args = vec!["riglr-showcase", "cross-chain", "--token", "USDC"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::CrossChain { token } => {
                assert_eq!(token, "USDC");
            }
            _ => panic!("Expected CrossChain command"),
        }
    }

    #[test]
    fn test_cli_parsing_with_agents_command_default_scenario() {
        let args = vec!["riglr-showcase", "agents"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Agents { scenario } => {
                assert_eq!(scenario, "trading"); // default value
            }
            _ => panic!("Expected Agents command"),
        }
    }

    #[test]
    fn test_cli_parsing_with_agents_command_custom_scenario() {
        let args = vec!["riglr-showcase", "agents", "--scenario", "risk"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Agents { scenario } => {
                assert_eq!(scenario, "risk");
            }
            _ => panic!("Expected Agents command"),
        }
    }

    #[test]
    fn test_cli_parsing_with_interactive_command() {
        let args = vec!["riglr-showcase", "interactive"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Interactive => {
                // Success - this variant has no fields to check
            }
            _ => panic!("Expected Interactive command"),
        }
    }

    #[test]
    fn test_cli_parsing_with_verbose_flag() {
        let args = vec!["riglr-showcase", "--verbose", "interactive"];
        let cli = Cli::try_parse_from(args).unwrap();

        assert!(cli.verbose);
        assert_eq!(cli.config, ".env");

        match cli.command {
            Commands::Interactive => {}
            _ => panic!("Expected Interactive command"),
        }
    }

    #[test]
    fn test_cli_parsing_with_custom_config() {
        let args = vec![
            "riglr-showcase",
            "--config",
            "/custom/path/.env",
            "interactive",
        ];
        let cli = Cli::try_parse_from(args).unwrap();

        assert!(!cli.verbose);
        assert_eq!(cli.config, "/custom/path/.env");
    }

    #[test]
    fn test_cli_parsing_with_short_flags() {
        let args = vec![
            "riglr-showcase",
            "-v",
            "-c",
            "custom.env",
            "solana",
            "-a",
            "test_address",
        ];
        let cli = Cli::try_parse_from(args).unwrap();

        assert!(cli.verbose);
        assert_eq!(cli.config, "custom.env");

        match cli.command {
            Commands::Solana { address } => {
                assert_eq!(address, Some("test_address".to_string()));
            }
            _ => panic!("Expected Solana command"),
        }
    }

    #[test]
    fn test_cli_parsing_invalid_command_should_fail() {
        let args = vec!["riglr-showcase", "invalid-command"];
        let result = Cli::try_parse_from(args);
        assert!(result.is_err());
    }

    #[test]
    fn test_cli_parsing_missing_required_web_query_should_fail() {
        let args = vec!["riglr-showcase", "web"];
        let result = Cli::try_parse_from(args);
        assert!(result.is_err());
    }

    #[test]
    fn test_cli_parsing_missing_required_cross_chain_token_should_fail() {
        let args = vec!["riglr-showcase", "cross-chain"];
        let result = Cli::try_parse_from(args);
        assert!(result.is_err());
    }

    #[test]
    fn test_init_logging_with_verbose_true() {
        // Note: This test primarily verifies the function doesn't panic
        // and tests the branch where verbose = true
        init_logging(true);
        // If we reach here without panicking, the test passes
    }

    #[test]
    fn test_init_logging_with_verbose_false() {
        // Note: This test primarily verifies the function doesn't panic
        // and tests the branch where verbose = false
        init_logging(false);
        // If we reach here without panicking, the test passes
    }

    #[test]
    fn test_cli_struct_debug_implementation() {
        let cli = Cli {
            command: Commands::Interactive,
            verbose: true,
            config: "test.env".to_string(),
        };

        // Verify the struct can be debugged (tests derived Debug trait)
        let debug_str = format!("{:?}", cli);
        assert!(debug_str.contains("Interactive"));
        assert!(debug_str.contains("true"));
        assert!(debug_str.contains("test.env"));
    }

    #[test]
    fn test_commands_enum_debug_implementation() {
        let solana_cmd = Commands::Solana {
            address: Some("test_address".to_string()),
        };
        let debug_str = format!("{:?}", solana_cmd);
        assert!(debug_str.contains("Solana"));
        assert!(debug_str.contains("test_address"));

        let interactive_cmd = Commands::Interactive;
        let debug_str = format!("{:?}", interactive_cmd);
        assert!(debug_str.contains("Interactive"));
    }

    #[test]
    fn test_cli_parsing_with_all_evm_args_and_global_flags() {
        let args = vec![
            "riglr-showcase",
            "--verbose",
            "--config",
            "production.env",
            "evm",
            "--address",
            "0x1234567890123456789012345678901234567890",
            "--chain-id",
            "42161", // Arbitrum
        ];
        let cli = Cli::try_parse_from(args).unwrap();

        assert!(cli.verbose);
        assert_eq!(cli.config, "production.env");

        match cli.command {
            Commands::Evm { address, chain_id } => {
                assert_eq!(
                    address,
                    Some("0x1234567890123456789012345678901234567890".to_string())
                );
                assert_eq!(chain_id, 42161);
            }
            _ => panic!("Expected Evm command"),
        }
    }

    #[test]
    fn test_cli_parsing_with_empty_string_arguments() {
        let args = vec!["riglr-showcase", "web", "--query", ""];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Web { query } => {
                assert_eq!(query, "");
            }
            _ => panic!("Expected Web command"),
        }
    }

    #[test]
    fn test_cli_parsing_with_special_characters_in_arguments() {
        let args = vec![
            "riglr-showcase",
            "agents",
            "--scenario",
            "special-chars!@#$%^&*()",
        ];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Agents { scenario } => {
                assert_eq!(scenario, "special-chars!@#$%^&*()");
            }
            _ => panic!("Expected Agents command"),
        }
    }

    #[test]
    fn test_cli_parsing_with_unicode_characters() {
        let args = vec!["riglr-showcase", "web", "--query", "区块链分析"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Web { query } => {
                assert_eq!(query, "区块链分析");
            }
            _ => panic!("Expected Web command"),
        }
    }

    #[test]
    fn test_cli_parsing_with_very_long_string() {
        let long_string = "a".repeat(1000);
        let args = vec!["riglr-showcase", "cross-chain", "--token", &long_string];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::CrossChain { token } => {
                assert_eq!(token, long_string);
            }
            _ => panic!("Expected CrossChain command"),
        }
    }

    #[test]
    fn test_cli_parsing_with_numeric_chain_id_max_value() {
        let binding = u64::MAX.to_string();
        let args = vec!["riglr-showcase", "evm", "--chain-id", &binding];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Evm { address, chain_id } => {
                assert_eq!(address, None);
                assert_eq!(chain_id, u64::MAX);
            }
            _ => panic!("Expected Evm command"),
        }
    }

    #[test]
    fn test_cli_parsing_with_zero_chain_id() {
        let args = vec!["riglr-showcase", "evm", "--chain-id", "0"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Evm { address, chain_id } => {
                assert_eq!(address, None);
                assert_eq!(chain_id, 0);
            }
            _ => panic!("Expected Evm command"),
        }
    }

    #[test]
    fn test_cli_parsing_invalid_chain_id_should_fail() {
        let args = vec!["riglr-showcase", "evm", "--chain-id", "not_a_number"];
        let result = Cli::try_parse_from(args);
        assert!(result.is_err());
    }

    #[test]
    fn test_cli_parsing_negative_chain_id_should_fail() {
        let args = vec!["riglr-showcase", "evm", "--chain-id", "-1"];
        let result = Cli::try_parse_from(args);
        assert!(result.is_err());
    }
}
