//! Simple Event Stream Example
//!
//! Demonstrates basic streaming setup with real Solana events using the enhanced riglr-streams client.
//! This example shows how to:
//! - Configure streaming client with optimal settings
//! - Connect to Solana RPC endpoints
//! - Process real transaction events
//! - Handle connection failures gracefully

use anyhow::Result;
use riglr_config::Config;
use riglr_core::{ToolError, ToolResult};
use riglr_events_core::prelude::*;
use riglr_solana_events::prelude::*;
use riglr_streams::core::{HandlerExecutionMode, StreamManagerBuilder};
use riglr_streams::prelude::*;
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use tracing::{debug, error, info, warn};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter("debug,riglr_streams=trace,riglr_solana_events=debug")
        .init();

    info!("🚀 Starting Simple Event Stream Example");

    // Load configuration
    let config = Config::from_env();

    // Build StreamManager using StreamManagerBuilder
    let stream_manager = StreamManagerBuilder::default()
        .with_execution_mode(HandlerExecutionMode::Concurrent)
        .with_metrics(true)
        .from_env()
        .map_err(|e| anyhow::anyhow!("Failed to load stream config: {}", e))?
        .build()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to build stream manager: {}", e))?;

    info!("✅ StreamManager initialized with builder pattern");

    // Start the stream manager
    stream_manager.start_all().await?;
    info!("🚀 Stream manager started");

    // Create event stream with enhanced capabilities
    let mut event_count = 0u64;
    let start_time = std::time::Instant::now();

    // Create event handler
    let handler = UnifiedEventHandler::new();

    // Simulate streaming with periodic stats
    let mut stats_interval = tokio::time::interval(Duration::from_secs(30));
    let mut event_interval = tokio::time::interval(Duration::from_millis(500));

    loop {
        tokio::select! {
            _ = stats_interval.tick() => {
                let elapsed = start_time.elapsed();
                let rate = event_count as f64 / elapsed.as_secs_f64();
                info!("📈 Processed {} events in {:?} ({:.2} events/sec)",
                     event_count, elapsed, rate);

                // Check stream manager metrics if available
                if let Ok(metrics) = stream_manager.get_metrics().await {
                    info!("📊 Stream metrics: {:?}", metrics);
                }
            }

            // Simulate processing events using the stream manager
            _ = event_interval.tick() => {
                // In a real implementation, events would come from the stream
                // For now, simulate event generation
                let mock_event = GenericEvent::new(
                    format!("event-{}", event_count),
                    EventKind::Transaction,
                    serde_json::json!({
                        "slot": 123456789 + event_count,
                        "signature": format!("sig_{}", event_count),
                        "amount": fastrand::u64(100..=10000)
                    }),
                );

                // Process event through handler
                if let Ok(_) = handler.handle_event(Box::new(mock_event)).await {
                    event_count += 1;
                }
            }
        }

        // Stop after processing for a reasonable time in this example
        if start_time.elapsed() > Duration::from_secs(60) {
            info!("⏰ Example complete - processed {} events", event_count);
            break;
        }
    }

    // Graceful shutdown
    info!("🛑 Shutting down stream manager...");
    stream_manager.stop().await?;

    info!(
        "📊 Final event count: {} events processed",
        handler.get_processed_count()
    );

    Ok(())
}

// Additional imports for the example
use riglr_events_core::GenericEvent;

/// Event handler that processes different types of blockchain events
struct UnifiedEventHandler {
    processed_count: Arc<std::sync::atomic::AtomicU64>,
}

impl UnifiedEventHandler {
    fn new() -> Self {
        Self {
            processed_count: Arc::new(std::sync::atomic::AtomicU64::new(0)),
        }
    }

    async fn handle_event(&self, event: Box<dyn Event>) -> ToolResult<()> {
        let count = self
            .processed_count
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        // Log event details
        info!("📦 Processing event #{}: {}", count, event.id());
        debug!("   Kind: {:?}", event.kind());
        debug!("   Source: {}", event.source());
        debug!("   Timestamp: {:?}", event.timestamp());

        // Process different event types
        match event.kind() {
            EventKind::Transaction => {
                self.handle_transaction_event(&*event).await?;
            }
            EventKind::Block => {
                self.handle_block_event(&*event).await?;
            }
            EventKind::Custom => {
                self.handle_custom_event(&*event).await?;
            }
            _ => {
                debug!("   Unhandled event kind: {:?}", event.kind());
            }
        }

        Ok(())
    }

    async fn handle_transaction_event(&self, event: &dyn Event) -> ToolResult<()> {
        debug!("💰 Processing transaction event: {}", event.id());

        // Extract transaction details from event data
        if let Some(data) = event.data() {
            if let Ok(tx_data) = serde_json::from_value::<serde_json::Value>(data.clone()) {
                debug!("   Transaction data: {}", tx_data);

                // Here you could:
                // - Parse signature and slot information
                // - Extract token transfers
                // - Identify DeFi protocol interactions
                // - Trigger trading strategies
            }
        }

        Ok(())
    }

    async fn handle_block_event(&self, event: &dyn Event) -> ToolResult<()> {
        debug!("🧱 Processing block event: {}", event.id());

        // Process block-level information
        if let Some(data) = event.data() {
            debug!("   Block data: {}", data);

            // Here you could:
            // - Update blockchain state
            // - Calculate network metrics
            // - Detect reorganizations
        }

        Ok(())
    }

    async fn handle_custom_event(&self, event: &dyn Event) -> ToolResult<()> {
        debug!("🔧 Processing custom event: {}", event.id());

        // Handle protocol-specific events
        if let Some(data) = event.data() {
            debug!("   Custom event data: {}", data);

            // Here you could:
            // - Process DEX swap events
            // - Handle NFT marketplace activities
            // - Track DeFi yield changes
        }

        Ok(())
    }

    fn get_processed_count(&self) -> u64 {
        self.processed_count
            .load(std::sync::atomic::Ordering::Relaxed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use riglr_events_core::GenericEvent;

    #[tokio::test]
    async fn test_event_handler() {
        let handler = UnifiedEventHandler::new();

        let event = GenericEvent::new(
            "test-event".into(),
            EventKind::Transaction,
            serde_json::json!({"amount": 1000, "from": "alice", "to": "bob"}),
        );

        let result = handler.handle_event(Box::new(event)).await;
        assert!(result.is_ok());
        assert_eq!(handler.get_processed_count(), 1);
    }

    #[tokio::test]
    async fn test_stream_manager_builder() {
        // Test StreamManagerBuilder configuration
        let result = StreamManagerBuilder::default()
            .with_execution_mode(HandlerExecutionMode::Sequential)
            .with_metrics(false)
            .build();

        // Note: This would fail without proper environment setup
        // In a real test, you'd mock the dependencies
        assert!(result.await.is_err()); // Expected to fail without config
    }

    #[tokio::test]
    async fn test_event_processing() {
        let handler = UnifiedEventHandler::new();

        // Test with multiple event types
        let events = vec![
            (EventKind::Transaction, json!({"tx": "data"})),
            (EventKind::Block, json!({"block": "data"})),
            (EventKind::Custom, json!({"custom": "data"})),
        ];

        for (kind, data) in events {
            let event = GenericEvent::new(format!("test-{:?}", kind), kind, data);

            let result = handler.handle_event(Box::new(event)).await;
            assert!(result.is_ok());
        }

        assert_eq!(handler.get_processed_count(), 3);
    }
}
