//! Example: Event-driven arbitrage bot using riglr-streams
//!
//! This example demonstrates how to:
//! - Set up multiple streams (Solana, EVM, Binance)
//! - Create event-triggered tools
//! - React to price changes across chains
//! - Execute arbitrage opportunities

use std::sync::Arc;
use riglr_streams::prelude::*;
use riglr_streams::solana::{SolanaGeyserStream, GeyserConfig};
use riglr_streams::evm::{EvmWebSocketStream, EvmStreamConfig, ChainId};
use riglr_streams::external::{BinanceStream, BinanceConfig};
use riglr_streams::tools::{EventTriggerBuilder, StreamingTool, ConditionCombinator};
use riglr_streams::tools::condition::EventKindMatcher;
use riglr_events_core::{Event, EventKind};
use async_trait::async_trait;
use tracing::info;

/// Example arbitrage tool
struct ArbitrageBot {
    name: String,
}

#[async_trait]
impl StreamingTool for ArbitrageBot {
    async fn execute(&self, event: &dyn Event) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Arbitrage opportunity detected!");
        info!("Event kind: {:?}", event.kind());
        info!("Event ID: {}", event.id());
        
        // Here you would:
        // 1. Parse the event data
        // 2. Calculate profitability
        // 3. Execute trades if profitable
        
        Ok(())
    }
    
    fn name(&self) -> &str {
        &self.name
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    info!("Starting streaming arbitrage bot");
    
    // Create stream manager with concurrent handler execution for better throughput
    // You can choose between:
    // - HandlerExecutionMode::Sequential - preserves order but slower
    // - HandlerExecutionMode::Concurrent - fastest but no ordering guarantees
    // - HandlerExecutionMode::ConcurrentBounded(n) - limited parallelism
    let stream_manager = Arc::new(StreamManager::with_execution_mode(
        HandlerExecutionMode::ConcurrentBounded(5)
    ));
    
    info!("Stream manager configured with bounded concurrent execution (max 5 parallel handlers)");
    
    // Configure and start Solana stream
    let mut solana_stream = SolanaGeyserStream::new("solana-mainnet");
    let solana_config = GeyserConfig {
        ws_url: "wss://api.mainnet-beta.solana.com".to_string(),
        auth_token: None,
        program_ids: vec![
            "JUP6LkbBzZrFHgqWrokGNTdYvpan1Jw9Y9nW9pMvv77".to_string(), // Jupiter
        ],
        buffer_size: 10000,
    };
    solana_stream.start(solana_config).await?;
    stream_manager.add_stream("solana".to_string(), solana_stream).await?;
    
    // Configure and start EVM stream
    let mut evm_stream = EvmWebSocketStream::new("ethereum-mainnet");
    let evm_config = EvmStreamConfig {
        ws_url: "wss://eth-mainnet.g.alchemy.com/v2/your-api-key".to_string(),
        chain_id: ChainId::Ethereum,
        subscribe_pending_transactions: false,
        subscribe_new_blocks: true,
        contract_addresses: vec![
            "0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D".to_string(), // Uniswap V2 Router
        ],
        buffer_size: 10000,
    };
    evm_stream.start(evm_config).await?;
    stream_manager.add_stream("ethereum".to_string(), evm_stream).await?;
    
    // Configure and start Binance stream
    let mut binance_stream = BinanceStream::new("binance");
    let binance_config = BinanceConfig {
        streams: vec![
            "btcusdt@ticker".to_string(),
            "ethusdt@ticker".to_string(),
            "solusdt@ticker".to_string(),
        ],
        testnet: false,
        buffer_size: 10000,
    };
    binance_stream.start(binance_config).await?;
    stream_manager.add_stream("binance".to_string(), binance_stream).await?;
    
    // Create arbitrage bot
    let arb_bot = ArbitrageBot {
        name: "ArbitrageBot".to_string(),
    };
    
    // Create event trigger with conditions using the library's EventKindMatcher
    let trigger = EventTriggerBuilder::new(arb_bot, "arbitrage-trigger")
        .condition(Box::new(EventKindMatcher::new(vec![
            EventKind::Swap,
            EventKind::Custom("trade".to_string()),
            EventKind::Price,
        ])))
        .combinator(ConditionCombinator::Any)
        .register(&stream_manager)
        .await;
    
    info!("Arbitrage bot is running. Press Ctrl+C to stop.");
    
    // Process events (streams are already running)
    stream_manager.process_events().await?;
    
    Ok(())
}