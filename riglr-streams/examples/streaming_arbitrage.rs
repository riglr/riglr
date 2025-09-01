//! Example: Event-driven arbitrage bot using riglr-streams
//!
//! This example demonstrates how to:
//! - Set up multiple streams (Solana, EVM, Binance)
//! - Create event-triggered tools
//! - React to price changes across chains
//! - Execute arbitrage opportunities
//!
//! Note: This example shows production stream configurations.
//! For testing without real API keys, uncomment the MockStream
//! sections and comment out the real stream configurations.

use async_trait::async_trait;
use riglr_events_core::{Event, EventKind};
use riglr_streams::prelude::*;
use riglr_streams::tools::condition::EventKindMatcher;
use riglr_streams::tools::event_triggered::EventTriggerBuilder;
use riglr_streams::tools::{ConditionCombinator, StreamingTool};
use std::sync::Arc;
use tracing::info;

// Production streams (require API keys)
use riglr_streams::evm::{ChainId, EvmStreamConfig, EvmWebSocketStream};
use riglr_streams::external::{BinanceConfig, BinanceStream};
use riglr_streams::solana::{GeyserConfig, SolanaGeyserStream};

// For testing without API keys (uncomment when using mock streams)
// use riglr_streams::core::mock_stream::{MockConfig, MockStream};

/// Example arbitrage tool
struct ArbitrageBot {
    name: String,
}

#[async_trait]
impl StreamingTool for ArbitrageBot {
    async fn execute(
        &self,
        event: &dyn Event,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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
        HandlerExecutionMode::ConcurrentBounded(5),
    ));

    info!("Stream manager configured with bounded concurrent execution (max 5 parallel handlers)");

    // Option 1: Use MockStream for testing (no API keys required)
    // Uncomment this section and comment out the real stream configurations below
    /*
    let mock_config = MockConfig {
        events_per_second: 5.0,
        event_kinds: vec![EventKind::Swap, EventKind::Price, EventKind::Transaction],
        max_events: Some(1000),
    };

    let mut solana_mock = MockStream::new("solana-mock".to_string(), mock_config.clone());
    solana_mock.start(()).await?;
    stream_manager.add_stream("solana".to_string(), solana_mock).await?;

    let mut eth_mock = MockStream::new("ethereum-mock".to_string(), mock_config.clone());
    eth_mock.start(()).await?;
    stream_manager.add_stream("ethereum".to_string(), eth_mock).await?;

    let mut binance_mock = MockStream::new("binance-mock".to_string(), mock_config);
    binance_mock.start(()).await?;
    stream_manager.add_stream("binance".to_string(), binance_mock).await?;
    */

    // Option 2: Use real streams (requires valid API keys/endpoints)
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
    stream_manager
        .add_stream("solana".to_string(), solana_stream)
        .await?;

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
    stream_manager
        .add_stream("ethereum".to_string(), evm_stream)
        .await?;

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
    stream_manager
        .add_stream("binance".to_string(), binance_stream)
        .await?;

    // Create arbitrage bot
    let arb_bot = ArbitrageBot {
        name: "ArbitrageBot".to_string(),
    };

    // Create event trigger with conditions using the library's EventKindMatcher
    let _trigger = EventTriggerBuilder::new(arb_bot, "arbitrage-trigger")
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
