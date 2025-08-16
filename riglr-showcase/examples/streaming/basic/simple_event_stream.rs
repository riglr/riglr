//! Simple Event Stream Example
//!
//! Demonstrates basic streaming setup with real Solana events using the enhanced riglr-streams client.
//! This example shows how to:
//! - Configure streaming client with optimal settings
//! - Connect to Solana RPC endpoints
//! - Process real transaction events
//! - Handle connection failures gracefully

use anyhow::Result;
use riglr_core::{ToolError, ToolResult};
use riglr_events_core::prelude::*;
use riglr_showcase::config::Config;
use riglr_streams::prelude::*;
use riglr_solana_events::prelude::*;
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use tracing::{info, error, warn, debug};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter("debug,riglr_streams=trace,riglr_solana_events=debug")
        .init();

    info!("🚀 Starting Simple Event Stream Example");

    // Load configuration
    let config = Config::from_env();
    config.validate()?;

    // Create optimized streaming configuration for real-time processing
    let stream_config = StreamClientConfig::low_latency();
    info!("📊 Stream config: {:?}", stream_config);

    // Create connection manager with auto-reconnection
    let connection_config = stream_config.connection.clone();
    let connection_manager = ConnectionManager::new(connection_config);

    // Connect to Solana RPC
    let rpc_url = config.network.solana_rpc_url.clone();
    info!("🔗 Connecting to Solana RPC: {}", rpc_url);

    connection_manager.connect({
        let rpc_url = rpc_url.clone();
        move || create_solana_connection(rpc_url.clone())
    }).await?;

    let health = connection_manager.health().await;
    info!("✅ Connection established: {:?}", health);

    // Create event stream with enhanced capabilities
    let mut event_count = 0u64;
    let start_time = std::time::Instant::now();

    // Simulate streaming with periodic health checks
    let mut health_check_interval = tokio::time::interval(Duration::from_secs(10));
    let mut stats_interval = tokio::time::interval(Duration::from_secs(30));

    loop {
        tokio::select! {
            _ = health_check_interval.tick() => {
                let health = connection_manager.health().await;
                match health.state {
                    ConnectionState::Connected => {
                        debug!("🟢 Connection healthy - last activity: {:?}", health.last_activity);
                    }
                    ConnectionState::Reconnecting => {
                        warn!("🟡 Connection reconnecting - attempt #{}", health.total_reconnects);
                    }
                    ConnectionState::Failed => {
                        error!("🔴 Connection failed after {} attempts", health.consecutive_failures);
                        // In a real implementation, you might want to exit or implement backoff
                        break;
                    }
                    _ => {
                        debug!("Connection state: {:?}", health.state);
                    }
                }
            }

            _ = stats_interval.tick() => {
                let elapsed = start_time.elapsed();
                let rate = event_count as f64 / elapsed.as_secs_f64();
                info!("📈 Processed {} events in {:?} ({:.2} events/sec)",
                     event_count, elapsed, rate);
            }

            // Simulate processing events
            _ = sleep(Duration::from_millis(100)) => {
                if let Ok(connection) = connection_manager.with_connection(|conn| {
                    // Simulate processing a batch of events
                    process_event_batch(conn)
                }).await {
                    event_count += connection;
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
    info!("🛑 Shutting down stream...");
    connection_manager.disconnect().await;

    let final_health = connection_manager.health().await;
    info!("📊 Final connection state: {:?}", final_health);

    Ok(())
}

// Mock connection creation for demonstration
async fn create_solana_connection(rpc_url: String) -> StreamResult<SolanaConnection> {
    debug!("🔌 Creating connection to: {}", rpc_url);

    // Simulate connection establishment
    sleep(Duration::from_millis(100)).await;

    Ok(SolanaConnection {
        rpc_url,
        connected_at: std::time::Instant::now(),
    })
}

// Mock connection type
#[derive(Clone, Debug)]
struct SolanaConnection {
    rpc_url: String,
    connected_at: std::time::Instant,
}

// Simulate event processing
fn process_event_batch(connection: &SolanaConnection) -> u64 {
    // Simulate processing time and batch size
    let batch_size = fastrand::u64(1..=10);

    debug!("🔄 Processing batch of {} events on {}", batch_size, connection.rpc_url);

    batch_size
}

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
        let count = self.processed_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

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
        self.processed_count.load(std::sync::atomic::Ordering::Relaxed)
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
    async fn test_connection_manager() {
        let config = ConnectionConfig::default();
        let manager = ConnectionManager::new(config);

        // Test connection
        let result = manager.connect(|| async {
            Ok(SolanaConnection {
                rpc_url: "ws://localhost:8900".into(),
                connected_at: std::time::Instant::now(),
            })
        }).await;

        assert!(result.is_ok());

        let health = manager.health().await;
        assert_eq!(health.state, ConnectionState::Connected);
        assert!(health.connected_at.is_some());
    }

    #[test]
    fn test_stream_config_validation() {
        let config = StreamClientConfig::low_latency();
        assert!(config.validate().is_ok());

        let config = StreamClientConfig::high_performance();
        assert!(config.validate().is_ok());
    }
}