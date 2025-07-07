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
use tracing::info;

mod commands;
mod config;

#[derive(Parser)]
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

#[derive(Subcommand)]
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
    /// Interactive chat mode
    Interactive,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Initialize logging
    init_logging(cli.verbose);
    
    // Load configuration
    dotenvy::from_filename(&cli.config).ok();
    let config = config::Config::from_env()?;
    
    info!("Starting riglr-showcase v{}", env!("CARGO_PKG_VERSION"));
    
    // Run the appropriate command
    match cli.command {
        Commands::Solana { address } => {
            commands::solana::run_demo(config, address).await?;
        }
        Commands::Evm { address, chain_id } => {
            commands::evm::run_demo(config, address, chain_id).await?;
        }
        Commands::Web { query } => {
            commands::web::run_demo(config, query).await?;
        }
        Commands::Graph { init, query } => {
            commands::graph::run_demo(config, init, query).await?;
        }
        Commands::CrossChain { token } => {
            commands::cross_chain::run_demo(config, token).await?;
        }
        Commands::Interactive => {
            commands::interactive::run_chat(config).await?;
        }
    }
    
    Ok(())
}

fn init_logging(verbose: bool) {
    use tracing_subscriber::{fmt, EnvFilter};
    
    let level = if verbose { "debug" } else { "info" };
    
    fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new(format!("riglr_showcase={},riglr_core={},riglr_solana_tools={},riglr_evm_tools={},riglr_web_tools={},riglr_graph_memory={}", level, level, level, level, level, level)))
        )
        .init();
}