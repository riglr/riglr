//! Examples demonstrating stream composition patterns in riglr-streams
//!
//! Note: This example shows how to compose streams using real implementations.
//! For testing without API keys, replace the real streams with `MockStream`:
//! ```
//! use riglr_streams::core::mock_stream::{MockStream, MockConfig};
//! let mock = MockStream::new("test".to_string(), MockConfig::default());
//! ```

use core::{any::Any, error::Error, time::Duration};
use riglr_events_core::{error::EventResult, Event, EventKind, EventMetadata};
use riglr_solana_events::ProtocolType;
use riglr_streams::core::{ComposableStream, Stream};
use riglr_streams::core::{DynamicStreamed, IntoDynamicStreamed};
use std::{
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};
use tracing::{info, Level};

// Production streams (comment out if using MockStream)
use riglr_streams::evm::WebSocketStream as EvmWebSocketStream;
use riglr_streams::external::binance::BinanceStream;
use riglr_streams::solana::SolanaGeyserStream;

// For testing (uncomment to use)
// use riglr_streams::core::mock_stream::{MockStream, MockConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize logging
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    // Example 1: Simple Filter and Map
    example_filter_map();

    // Example 2: Merging Multiple Streams
    example_merge_streams();

    // Example 3: Throttling High-Frequency Events
    example_throttle();

    // Example 4: Batching for Efficiency
    example_batching();

    // Example 5: Complex Pipeline
    example_complex_pipeline();

    // Example 6: Cross-Chain Arbitrage Detection
    example_arbitrage_detection();

    Ok(())
}

/// Example 1: Filter and map events from a single stream
fn example_filter_map() {
    info!("Example 1: Filter and Map");

    // Option A: Create a Solana stream (requires valid WebSocket endpoint)
    let solana_stream = SolanaGeyserStream::new("solana-mainnet");

    // Option B: Use MockStream for testing (uncomment below, comment above)
    // use riglr_streams::core::mock_stream::{MockStream, MockConfig};
    // let config = MockConfig {
    //     events_per_second: 5.0,
    //     event_kinds: vec![EventKind::Swap, EventKind::Transaction],
    //     max_events: Some(100),
    // };
    // let solana_stream = MockStream::new("mock-solana".to_string(), config);

    // Filter for swap events and map to extract key data
    let _processed_stream = solana_stream
        .filter(|event| matches!(*event.inner.kind(), EventKind::Swap))
        .map(|event| {
            // Extract relevant swap data
            let swap_summary = SwapSummary::new(
                event.inner.id().to_string(),
                ProtocolType::Other("solana".to_string()),
                0, // slot would need to be extracted from chain_data
                event.inner.timestamp(),
            );
            (Box::new(swap_summary) as Box<dyn Event>)
                .with_default_stream_metadata("processed_swap")
        });

    // Subscribe to processed events
    // Note: rx is not Send, so we can't spawn it in a task
    // In a real application, you would process events here or use a Send-compatible channel
}

/// Example 2: Merge events from multiple chains
fn example_merge_streams() {
    info!("Example 2: Merging Multiple Streams");

    // Create streams for different chains
    let solana_stream = SolanaGeyserStream::new("solana");
    let _eth_stream = EvmWebSocketStream::new("ethereum".to_string());
    let _bsc_stream = EvmWebSocketStream::new("bsc".to_string());

    // For demonstration - normally you'd merge streams of the same type
    // Here we're just using the solana_stream as an example
    let _merged = solana_stream;

    // Subscribe to merged stream
    // Note: rx is not Send, so we can't spawn it in a task
    // In a real application, you would process events here or use a Send-compatible channel
}

/// Example 3: Throttle high-frequency events
fn example_throttle() {
    info!("Example 3: Throttling High-Frequency Events");

    // Create a Binance stream for price tickers
    let binance_stream = BinanceStream::new("binance");

    // Throttle to one event per second to avoid overwhelming downstream
    let _throttled = binance_stream.throttle(Duration::from_secs(1));

    // We need to subscribe directly since Arc<BinanceStreamEvent> doesn't implement Event

    // Process throttled events
    // Note: rx is not Send, so we can't spawn it in a task
    // In a real application, you would process events here or use a Send-compatible channel
}

/// Example 4: Batch events for efficient processing
fn example_batching() {
    info!("Example 4: Batching Events");

    let stream = SolanaGeyserStream::new("solana");

    // Batch events: up to 100 events or 5 seconds, whichever comes first
    let _batched = stream.batch(100, Duration::from_secs(5)).map(|event| {
        info!("Processing batched event: {}", event.inner().id());
        // For now, each event is processed individually as the current BatchedStream
        // sends individual events rather than true batch objects
        let result = BatchResult::new(1); // Single event processed
        (Box::new(result) as Box<dyn Event>).with_default_stream_metadata("batch_processed")
    });
}

/// Example 5: Complex processing pipeline
fn example_complex_pipeline() {
    info!("Example 5: Complex Pipeline");

    let stream = SolanaGeyserStream::new("solana");

    // Build a complex pipeline:
    // 1. Filter for DEX swaps
    // 2. Extract swap data
    // 3. Calculate running average
    // 4. Throttle output
    // 5. Batch for database writes

    let _pipeline = stream
        .filter(|_e| {
            // In real implementation, would check if it's a swap event
            // from Jupiter or Orca
            true
        })
        .map(extract_swap_metrics)
        .map(|metrics| {
            // Instead of scan, just process each metrics event individually
            // In production, you'd maintain state elsewhere (database, cache, etc.)
            let mut avg = RunningAverage::new();
            if let Some(swap_metrics) = metrics.inner().as_any().downcast_ref::<SwapMetrics>() {
                avg.update(swap_metrics.volume);
            }
            (Box::new(avg) as Box<dyn Event>).with_default_stream_metadata("running_average")
        })
        .throttle(Duration::from_secs(10))
        .batch(50, Duration::from_secs(60));

    // Subscribe to final processed stream
    // Note: rx is not Send, so we can't spawn it in a task
    // In a real application, you would process events here or use a Send-compatible channel
}

/// Example 6: Cross-chain arbitrage detection
fn example_arbitrage_detection() {
    info!("Example 6: Cross-Chain Arbitrage Detection");

    // Create price streams from different sources
    let binance_btc = create_price_stream("binance", "btcusdt");
    let _coinbase_btc = create_price_stream("coinbase", "BTC-USD");

    // Combine latest prices from both exchanges
    // For this demo, just use one stream
    let combined = binance_btc;

    // Detect arbitrage opportunities (simplified for demo)
    let _opportunities = combined
        .map(|_price_event| {
            let opportunity = ArbitrageOpportunity::new(
                "BTC".to_string(),
                "Binance".to_string(),
                "Coinbase".to_string(),
                0.1, // Demo spread
                SystemTime::now(),
            );
            (Box::new(opportunity) as Box<dyn Event>)
                .with_default_stream_metadata("arbitrage_opportunity")
        })
        .filter(|opp| {
            opp.inner()
                .as_any()
                .downcast_ref::<ArbitrageOpportunity>()
                .is_some_and(|arb_opp| arb_opp.spread_percentage > 0.1)
        })
        .debounce(Duration::from_secs(1)); // Avoid duplicate alerts

    // Subscribe to arbitrage opportunities
    // Note: rx is not Send, so we can't spawn it in a task
    // In a real application, you would process events here or use a Send-compatible channel
}

// Helper types and functions

#[derive(Debug, Clone)]
struct SwapSummary {
    metadata: EventMetadata,
    protocol: ProtocolType,
    slot: u64,
}
impl SwapSummary {
    fn new(id: String, protocol: ProtocolType, slot: u64, timestamp: SystemTime) -> Self {
        Self {
            metadata: EventMetadata::with_timestamp(
                id,
                EventKind::Swap,
                "swap_summary".to_string(),
                timestamp.into(),
            ),
            protocol,
            slot,
        }
    }
}
impl Event for SwapSummary {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn clone_boxed(&self) -> Box<dyn Event> {
        Box::new(self.clone())
    }
    fn id(&self) -> &str {
        &self.metadata.id
    }
    fn kind(&self) -> &EventKind {
        &self.metadata.kind
    }
    fn metadata(&self) -> &EventMetadata {
        &self.metadata
    }
    fn metadata_mut(&mut self) -> EventResult<&mut EventMetadata> {
        Ok(&mut self.metadata)
    }
    fn to_json(&self) -> EventResult<serde_json::Value> {
        Ok(serde_json::json!({
            "id": self.id(),
            "protocol": format!("{:?}", self.protocol),
            "slot": self.slot
        }))
    }
}

#[derive(Clone, Debug)]
struct SwapMetrics {
    _price: f64,
    _timestamp: SystemTime,
    metadata: EventMetadata,
    volume: f64,
}
impl SwapMetrics {
    fn new(volume: f64, price: f64, timestamp: SystemTime) -> Self {
        Self {
            _price: price,
            _timestamp: timestamp,
            metadata: EventMetadata::new(
                format!(
                    "swap_{}",
                    timestamp
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs()
                ),
                EventKind::Swap,
                "swap_metrics".to_string(),
            ),
            volume,
        }
    }
}
impl Event for SwapMetrics {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn clone_boxed(&self) -> Box<dyn Event> {
        Box::new(self.clone())
    }
    fn id(&self) -> &str {
        &self.metadata.id
    }
    fn kind(&self) -> &EventKind {
        &self.metadata.kind
    }
    fn metadata(&self) -> &EventMetadata {
        &self.metadata
    }
    fn metadata_mut(&mut self) -> EventResult<&mut EventMetadata> {
        Ok(&mut self.metadata)
    }
    fn to_json(&self) -> EventResult<serde_json::Value> {
        Ok(serde_json::json!({
            "id": self.id(),
            "volume": self.volume
        }))
    }
}

#[derive(Debug, Clone)]
struct RunningAverage {
    average: f64,
    count: usize,
    metadata: EventMetadata,
    sum: f64,
}
impl RunningAverage {
    fn new() -> Self {
        Self {
            average: 0.0,
            count: 0,
            metadata: EventMetadata::new(
                format!(
                    "running_average_{}",
                    SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_millis()
                ),
                EventKind::Custom("running_average".to_string()),
                "metrics".to_string(),
            ),
            sum: 0.0,
        }
    }

    fn update(&mut self, value: f64) {
        self.sum += value;
        self.count = self.count.saturating_add(1);
        #[expect(clippy::cast_precision_loss)]
        {
            self.average = self.sum / self.count as f64;
        }
    }
}
impl Event for RunningAverage {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn clone_boxed(&self) -> Box<dyn Event> {
        Box::new(self.clone())
    }
    fn id(&self) -> &str {
        &self.metadata.id
    }
    fn kind(&self) -> &EventKind {
        &self.metadata.kind
    }
    fn metadata(&self) -> &EventMetadata {
        &self.metadata
    }
    fn metadata_mut(&mut self) -> EventResult<&mut EventMetadata> {
        Ok(&mut self.metadata)
    }
    fn to_json(&self) -> EventResult<serde_json::Value> {
        Ok(serde_json::json!({
            "id": self.id(),
            "sum": self.sum,
            "count": self.count,
            "average": self.average
        }))
    }
}

#[derive(Debug, Clone)]
#[expect(dead_code)]
struct ArbitrageOpportunity {
    exchange1: String,
    exchange2: String,
    metadata: EventMetadata,
    pair: String,
    spread_percentage: f64,
}
impl ArbitrageOpportunity {
    fn new(
        pair: String,
        exchange1: String,
        exchange2: String,
        spread_percentage: f64,
        timestamp: SystemTime,
    ) -> Self {
        Self {
            exchange1,
            exchange2,
            metadata: EventMetadata::with_timestamp(
                format!("arb_{pair}_{spread_percentage}"),
                EventKind::Custom("arbitrage".to_string()),
                "arbitrage".to_string(),
                timestamp.into(),
            ),
            pair,
            spread_percentage,
        }
    }
}
impl Event for ArbitrageOpportunity {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn clone_boxed(&self) -> Box<dyn Event> {
        Box::new(self.clone())
    }
    fn id(&self) -> &str {
        &self.metadata.id
    }
    fn kind(&self) -> &EventKind {
        &self.metadata.kind
    }
    fn metadata(&self) -> &EventMetadata {
        &self.metadata
    }
    fn metadata_mut(&mut self) -> EventResult<&mut EventMetadata> {
        Ok(&mut self.metadata)
    }
    fn to_json(&self) -> EventResult<serde_json::Value> {
        Ok(serde_json::json!({
            "id": self.id(),
            "spread_percentage": self.spread_percentage
        }))
    }
}

fn _process_price_update(_event: Arc<dyn Any>) {
    // Simulate processing
    info!("Processing price update");
}

// process_batch function removed - using BatchResult::new() directly

#[derive(Debug, Clone)]
struct BatchResult {
    _processed: usize,
    _timestamp: SystemTime,
    metadata: EventMetadata,
}
impl BatchResult {
    fn new(processed: usize) -> Self {
        Self {
            _processed: processed,
            _timestamp: SystemTime::now(),
            metadata: EventMetadata::new(
                format!("batch_{processed}"),
                EventKind::Custom("batch_result".to_string()),
                "batch_processor".to_string(),
            ),
        }
    }
}
impl Event for BatchResult {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn clone_boxed(&self) -> Box<dyn Event> {
        Box::new(self.clone())
    }
    fn id(&self) -> &str {
        &self.metadata.id
    }
    fn kind(&self) -> &EventKind {
        &self.metadata.kind
    }
    fn metadata(&self) -> &EventMetadata {
        &self.metadata
    }
    fn metadata_mut(&mut self) -> EventResult<&mut EventMetadata> {
        Ok(&mut self.metadata)
    }
    fn to_json(&self) -> EventResult<serde_json::Value> {
        Ok(serde_json::json!({
            "id": self.id(),
            "type": "batch_result"
        }))
    }
}

#[expect(clippy::needless_pass_by_value)]
fn extract_swap_metrics(event: Arc<DynamicStreamed>) -> DynamicStreamed {
    let metrics = SwapMetrics::new(
        1000.0, // Would extract from event
        50.0,
        event.inner.timestamp(),
    );
    (Box::new(metrics) as Box<dyn Event>).with_default_stream_metadata("swap_metrics")
}

fn _write_to_database(data: &[RunningAverage]) {
    info!("Writing {} records to database", data.len());
}

fn create_price_stream(exchange: &str, pair: &str) -> impl Stream + use<> {
    // Would create actual price stream
    BinanceStream::new(format!("{exchange}-{pair}"))
}
fn _calculate_spread(price1: f64, price2: f64) -> f64 {
    ((price1 - price2).abs() / price1) * 100.0
}

fn _alert_arbitrage_opportunity(opp: &ArbitrageOpportunity) {
    info!(
        "🚨 Arbitrage Alert: {} spread on {} between {} and {}",
        opp.spread_percentage, opp.pair, opp.exchange1, opp.exchange2
    );
}
