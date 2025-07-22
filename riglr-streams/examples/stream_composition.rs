//! Examples demonstrating stream composition patterns in riglr-streams

use std::sync::Arc;
use std::time::Duration;
use std::time::SystemTime;
use std::any::Any;
use riglr_streams::core::{
    StreamManager, StreamManagerBuilder, HandlerExecutionMode,
    ComposableStream, combinators, BufferStrategy, Stream
};
use riglr_streams::solana::geyser::{SolanaGeyserStream, GeyserConfig};
use riglr_streams::evm::websocket::{EvmWebSocketStream, EvmStreamConfig, ChainId};
use riglr_streams::external::binance::{BinanceStream, BinanceConfig};
use riglr_solana_events::{EventType, ProtocolType, UnifiedEvent};
use tracing::{info, Level};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();
    
    // Example 1: Simple Filter and Map
    example_filter_map().await?;
    
    // Example 2: Merging Multiple Streams
    example_merge_streams().await?;
    
    // Example 3: Throttling High-Frequency Events
    example_throttle().await?;
    
    // Example 4: Batching for Efficiency
    example_batching().await?;
    
    // Example 5: Complex Pipeline
    example_complex_pipeline().await?;
    
    // Example 6: Cross-Chain Arbitrage Detection
    example_arbitrage_detection().await?;
    
    Ok(())
}

/// Example 1: Filter and map events from a single stream
async fn example_filter_map() -> Result<(), Box<dyn std::error::Error>> {
    info!("Example 1: Filter and Map");
    
    // Create a Solana stream
    let solana_stream = SolanaGeyserStream::new("solana-mainnet");
    
    // Filter for swap events and map to extract key data
    let processed_stream = solana_stream
        .filter(|event| event.event_type() == EventType::Swap)
        .map(|event| {
            // Extract relevant swap data
            SwapSummary {
                id: event.id().to_string(),
                protocol: event.protocol_type(),
                slot: event.slot(),
                timestamp: event.timestamp(),
            }
        });
    
    // Subscribe to processed events
    let mut rx = processed_stream.subscribe();
    
    tokio::spawn(async move {
        while let Ok(swap_summary) = rx.recv().await {
            info!("Processed swap: {:?}", swap_summary);
        }
    });
    
    Ok(())
}

/// Example 2: Merge events from multiple chains
async fn example_merge_streams() -> Result<(), Box<dyn std::error::Error>> {
    info!("Example 2: Merging Multiple Streams");
    
    // Create streams for different chains
    let solana_stream = SolanaGeyserStream::new("solana");
    let eth_stream = EvmWebSocketStream::new("ethereum");
    let bsc_stream = EvmWebSocketStream::new("bsc");
    
    // Merge all blockchain events
    let merged = combinators::merge_all(vec![
        solana_stream,
        eth_stream,
        bsc_stream,
    ]);
    
    // Subscribe to merged stream
    let mut rx = merged.subscribe();
    
    tokio::spawn(async move {
        while let Ok(event) = rx.recv().await {
            info!("Event from merged stream: {}", event.id());
        }
    });
    
    Ok(())
}

/// Example 3: Throttle high-frequency events
async fn example_throttle() -> Result<(), Box<dyn std::error::Error>> {
    info!("Example 3: Throttling High-Frequency Events");
    
    // Create a Binance stream for price tickers
    let binance_stream = BinanceStream::new("binance");
    
    // Throttle to one event per second to avoid overwhelming downstream
    let throttled = binance_stream
        .throttle(Duration::from_secs(1))
        .map(|event| {
            info!("Throttled price update: {}", event.id());
            event
        });
    
    // Process throttled events
    let mut rx = throttled.subscribe();
    
    tokio::spawn(async move {
        while let Ok(event) = rx.recv().await {
            // Process at a sustainable rate
            process_price_update(event).await;
        }
    });
    
    Ok(())
}

/// Example 4: Batch events for efficient processing
async fn example_batching() -> Result<(), Box<dyn std::error::Error>> {
    info!("Example 4: Batching Events");
    
    let stream = SolanaGeyserStream::new("solana");
    
    // Batch events: up to 100 events or 5 seconds, whichever comes first
    let batched = stream
        .batch(100, Duration::from_secs(5))
        .map(|batch| {
            info!("Processing batch of {} events", batch.len());
            // Process entire batch at once (e.g., bulk database insert)
            process_batch(batch)
        });
    
    Ok(())
}

/// Example 5: Complex processing pipeline
async fn example_complex_pipeline() -> Result<(), Box<dyn std::error::Error>> {
    info!("Example 5: Complex Pipeline");
    
    let stream = SolanaGeyserStream::new("solana");
    
    // Build a complex pipeline:
    // 1. Filter for DEX swaps
    // 2. Extract swap data
    // 3. Calculate running average
    // 4. Throttle output
    // 5. Batch for database writes
    
    let pipeline = stream
        .filter(|e| {
            e.event_type() == EventType::Swap && 
            matches!(e.protocol_type(), ProtocolType::Jupiter | ProtocolType::Orca)
        })
        .map(|e| extract_swap_metrics(e))
        .scan(RunningAverage::new(), |mut avg, metrics| {
            avg.update(metrics.volume);
            avg
        })
        .throttle(Duration::from_secs(10))
        .batch(50, Duration::from_secs(60));
    
    // Subscribe to final processed stream
    let mut rx = pipeline.subscribe();
    
    tokio::spawn(async move {
        while let Ok(batch) = rx.recv().await {
            write_to_database(batch).await;
        }
    });
    
    Ok(())
}

/// Example 6: Cross-chain arbitrage detection
async fn example_arbitrage_detection() -> Result<(), Box<dyn std::error::Error>> {
    info!("Example 6: Cross-Chain Arbitrage Detection");
    
    // Create price streams from different sources
    let binance_btc = create_price_stream("binance", "btcusdt");
    let coinbase_btc = create_price_stream("coinbase", "BTC-USD");
    
    // Combine latest prices from both exchanges
    let combined = combinators::combine_latest(binance_btc, coinbase_btc);
    
    // Detect arbitrage opportunities
    let opportunities = combined
        .map(|(binance_price, coinbase_price)| {
            let spread = calculate_spread(binance_price, coinbase_price);
            ArbitrageOpportunity {
                pair: "BTC".to_string(),
                exchange1: "Binance".to_string(),
                exchange2: "Coinbase".to_string(),
                spread_percentage: spread,
                timestamp: std::time::SystemTime::now(),
            }
        })
        .filter(|opp| opp.spread_percentage > 0.1) // Only significant opportunities
        .debounce(Duration::from_secs(1)); // Avoid duplicate alerts
    
    // Subscribe to arbitrage opportunities
    let mut rx = opportunities.subscribe();
    
    tokio::spawn(async move {
        while let Ok(opportunity) = rx.recv().await {
            alert_arbitrage_opportunity(opportunity).await;
        }
    });
    
    Ok(())
}

// Helper types and functions

#[derive(Debug, Clone)]
struct SwapSummary {
    id: String,
    protocol: ProtocolType,
    slot: u64,
    timestamp: std::time::SystemTime,
}

#[derive(Debug, Clone)]
struct SwapMetrics {
    volume: f64,
    price: f64,
    timestamp: std::time::SystemTime,
}

#[derive(Debug, Clone)]
struct RunningAverage {
    sum: f64,
    count: usize,
    average: f64,
}

impl RunningAverage {
    fn new() -> Self {
        Self {
            sum: 0.0,
            count: 0,
            average: 0.0,
        }
    }
    
    fn update(&mut self, value: f64) {
        self.sum += value;
        self.count += 1;
        self.average = self.sum / self.count as f64;
    }
}

#[derive(Debug, Clone)]
struct ArbitrageOpportunity {
    pair: String,
    exchange1: String,
    exchange2: String,
    spread_percentage: f64,
    timestamp: std::time::SystemTime,
}

async fn process_price_update(event: Arc<dyn Any>) {
    // Simulate processing
    info!("Processing price update");
}

fn process_batch(batch: Vec<Arc<dyn Any>>) -> BatchResult {
    BatchResult {
        processed: batch.len(),
        timestamp: std::time::SystemTime::now(),
    }
}

#[derive(Debug)]
struct BatchResult {
    processed: usize,
    timestamp: std::time::SystemTime,
}

fn extract_swap_metrics(event: Arc<dyn Any>) -> SwapMetrics {
    SwapMetrics {
        volume: 1000.0, // Would extract from event
        price: 50.0,
        timestamp: std::time::SystemTime::now(),
    }
}

async fn write_to_database(data: Vec<RunningAverage>) {
    info!("Writing {} records to database", data.len());
}

fn create_price_stream(exchange: &str, pair: &str) -> impl Stream {
    // Would create actual price stream
    BinanceStream::new(format!("{}-{}", exchange, pair))
}

fn calculate_spread(price1: f64, price2: f64) -> f64 {
    ((price1 - price2).abs() / price1) * 100.0
}

async fn alert_arbitrage_opportunity(opp: ArbitrageOpportunity) {
    info!("🚨 Arbitrage Alert: {} spread on {} between {} and {}", 
          opp.spread_percentage, opp.pair, opp.exchange1, opp.exchange2);
}