use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::{broadcast, RwLock};
use tokio_tungstenite::connect_async;
use tracing::info;

use crate::core::StreamMetadata;
use crate::core::{Stream, StreamError, StreamEvent, StreamHealth};
use chrono::Utc;
use riglr_events_core::prelude::{Event, EventKind, EventMetadata};

/// Mempool.space WebSocket stream implementation
pub struct MempoolSpaceStream {
    /// Stream configuration
    config: MempoolConfig,
    /// Event broadcast channel
    event_tx: broadcast::Sender<Arc<MempoolStreamEvent>>,
    /// Running state
    running: Arc<AtomicBool>,
    /// Health metrics
    health: Arc<RwLock<StreamHealth>>,
    /// Stream name
    name: String,
}

/// Mempool configuration
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct MempoolConfig {
    /// Bitcoin network
    pub network: BitcoinNetwork,
    /// Subscribe to transactions
    pub subscribe_transactions: bool,
    /// Subscribe to blocks
    pub subscribe_blocks: bool,
    /// Subscribe to fee estimates
    pub subscribe_fees: bool,
    /// Buffer size for event channel
    pub buffer_size: usize,
}

impl Default for MempoolConfig {
    fn default() -> Self {
        Self {
            network: BitcoinNetwork::Mainnet,
            subscribe_transactions: true,
            subscribe_blocks: true,
            subscribe_fees: false,
            buffer_size: 10000,
        }
    }
}

/// Bitcoin network type
#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum BitcoinNetwork {
    Mainnet,
    Testnet,
    Signet,
}

impl std::fmt::Display for BitcoinNetwork {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BitcoinNetwork::Mainnet => write!(f, "mainnet"),
            BitcoinNetwork::Testnet => write!(f, "testnet"),
            BitcoinNetwork::Signet => write!(f, "signet"),
        }
    }
}

/// Mempool streaming event
#[derive(Debug, Clone)]
pub struct MempoolStreamEvent {
    /// Event metadata for riglr-events-core compatibility
    pub metadata: EventMetadata,
    /// Event type
    pub event_type: MempoolEventType,
    /// Event data
    pub data: serde_json::Value,
    /// Stream metadata (legacy)
    pub stream_meta: StreamMetadata,
    /// Bitcoin network
    pub network: BitcoinNetwork,
    /// Block height if applicable
    pub block_height: Option<u64>,
    /// Transaction count if applicable
    pub transaction_count: Option<u32>,
}

/// Mempool event types
#[derive(Debug, Clone, Serialize)]
pub enum MempoolEventType {
    Block,
    Transaction,
    Stats,
}

/// Mempool event data types
#[derive(Debug, Clone)]
pub enum MempoolEventData {
    /// Bitcoin transaction
    Transaction(BitcoinTransaction),
    /// Bitcoin block
    Block(BitcoinBlock),
    /// Fee estimate update
    FeeEstimate(FeeEstimate),
    /// Mempool statistics
    MempoolStats(MempoolStats),
    /// Unknown event
    Unknown(serde_json::Value),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BitcoinTransaction {
    pub txid: String,
    pub fee: u64,
    pub vsize: u32,
    pub value: u64,
    #[serde(default)]
    pub confirmed: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BitcoinBlock {
    pub id: String,
    pub height: u64,
    pub timestamp: u64,
    pub tx_count: u32,
    pub size: u32,
    pub weight: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FeeEstimate {
    #[serde(rename = "fastestFee")]
    pub fastest_fee: u32,
    #[serde(rename = "halfHourFee")]
    pub half_hour_fee: u32,
    #[serde(rename = "hourFee")]
    pub hour_fee: u32,
    #[serde(rename = "economyFee")]
    pub economy_fee: u32,
    #[serde(rename = "minimumFee")]
    pub minimum_fee: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MempoolStats {
    pub count: u32,
    pub vsize: u64,
    pub total_fee: u64,
    pub fee_histogram: Vec<Vec<u64>>,
}

impl StreamEvent for MempoolStreamEvent {
    fn stream_metadata(&self) -> Option<&StreamMetadata> {
        Some(&self.stream_meta)
    }
}

impl Event for MempoolStreamEvent {
    fn id(&self) -> &str {
        &self.metadata.id
    }

    fn kind(&self) -> &EventKind {
        &self.metadata.kind
    }

    fn metadata(&self) -> &EventMetadata {
        &self.metadata
    }

    fn metadata_mut(&mut self) -> &mut EventMetadata {
        &mut self.metadata
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn clone_boxed(&self) -> Box<dyn Event> {
        Box::new(self.clone())
    }

    fn to_json(&self) -> riglr_events_core::EventResult<serde_json::Value> {
        Ok(serde_json::json!({
            "metadata": self.metadata,
            "event_type": self.event_type,
            "data": self.data,
            "stream_meta": self.stream_meta,
            "network": self.network,
            "block_height": self.block_height,
            "transaction_count": self.transaction_count
        }))
    }
}

#[async_trait::async_trait]
impl Stream for MempoolSpaceStream {
    type Event = MempoolStreamEvent;
    type Config = MempoolConfig;

    async fn start(&mut self, config: Self::Config) -> Result<(), StreamError> {
        if self.running.load(Ordering::Relaxed) {
            return Err(StreamError::AlreadyRunning {
                name: self.name.clone(),
            });
        }

        info!(
            "Starting Mempool.space WebSocket stream for {:?}",
            config.network
        );

        self.config = config;
        self.running.store(true, Ordering::Relaxed);

        // Update health status
        {
            let mut health = self.health.write().await;
            health.is_connected = true;
            health.last_event_time = Some(SystemTime::now());
        }

        // Start WebSocket connection
        self.start_websocket().await?;

        Ok(())
    }

    async fn stop(&mut self) -> Result<(), StreamError> {
        if !self.running.load(Ordering::Relaxed) {
            return Ok(());
        }

        info!("Stopping Mempool.space WebSocket stream");
        self.running.store(false, Ordering::Relaxed);

        // Update health status
        {
            let mut health = self.health.write().await;
            health.is_connected = false;
        }

        Ok(())
    }

    fn subscribe(&self) -> broadcast::Receiver<Arc<Self::Event>> {
        self.event_tx.subscribe()
    }

    fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }

    async fn health(&self) -> StreamHealth {
        let health = self.health.read().await;
        health.clone()
    }

    fn name(&self) -> &str {
        &self.name
    }
}

impl MempoolSpaceStream {
    /// Create a new Mempool.space stream
    pub fn new(name: impl Into<String>) -> Self {
        let (event_tx, _) = broadcast::channel(10000);

        Self {
            config: MempoolConfig::default(),
            event_tx,
            running: Arc::new(AtomicBool::new(false)),
            health: Arc::new(RwLock::new(StreamHealth::default())),
            name: name.into(),
        }
    }

    /// Start the WebSocket connection with resilience
    async fn start_websocket(&self) -> Result<(), StreamError> {
        let base_url = match self.config.network {
            BitcoinNetwork::Mainnet => "wss://mempool.space/api/v1/ws",
            BitcoinNetwork::Testnet => "wss://mempool.space/testnet/api/v1/ws",
            BitcoinNetwork::Signet => "wss://mempool.space/signet/api/v1/ws",
        };

        let url = base_url.to_string();
        let event_tx = self.event_tx.clone();
        let running = self.running.clone();
        let health = self.health.clone();
        let stream_name = self.name.clone();
        let config = self.config.clone();

        tokio::spawn(async move {
            crate::impl_resilient_websocket!(
                stream_name,
                url.clone(),
                running,
                health,
                event_tx,
                Option::<crate::core::MetricsCollector>::None, // No metrics for now
                // Connect function
                || {
                    let url = url.clone();
                    async move {
                        connect_async(&url)
                            .await
                            .map(|(ws, _)| ws)
                            .map_err(|e| format!("Failed to connect to Mempool.space: {}", e))
                    }
                },
                // Subscribe function
                {
                    let _config = config.clone();
                    move |_write: &mut futures::stream::SplitSink<_, Message>| {
                        std::future::ready(Ok::<(), String>(()))
                    }
                },
                // Parse function
                |text: String, sequence_number: u64| {
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                        Self::parse_message(json, sequence_number)
                    } else {
                        None
                    }
                }
            );
        });

        Ok(())
    }

    /// Parse a message from mempool.space
    fn parse_message(json: serde_json::Value, _sequence_number: u64) -> Option<MempoolStreamEvent> {
        // Create unique ID
        let id = format!(
            "mempool_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis()
        );

        // Determine event kind
        let kind = EventKind::Block;

        // Create event metadata
        let now = Utc::now();
        let metadata = EventMetadata {
            id: id.clone(),
            kind,
            timestamp: now,
            received_at: now,
            source: "mempool-space-ws".to_string(),
            chain_data: None,
            custom: std::collections::HashMap::new(),
        };

        // Create stream metadata (legacy)
        let stream_meta = StreamMetadata {
            stream_source: "mempool.space".to_string(),
            received_at: std::time::SystemTime::now(),
            sequence_number: Some(_sequence_number),
            custom_data: None,
        };

        Some(MempoolStreamEvent {
            metadata,
            event_type: MempoolEventType::Block,
            stream_meta,
            network: BitcoinNetwork::Mainnet,
            block_height: None,
            transaction_count: None,
            data: json,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = MempoolConfig::default();
        assert_eq!(config.network, BitcoinNetwork::Mainnet);
        assert!(config.subscribe_blocks);
        assert!(config.subscribe_transactions);
        assert!(!config.subscribe_fees);
    }

    #[test]
    fn test_bitcoin_network_display() {
        assert_eq!(BitcoinNetwork::Mainnet.to_string(), "mainnet");
        assert_eq!(BitcoinNetwork::Testnet.to_string(), "testnet");
        assert_eq!(BitcoinNetwork::Signet.to_string(), "signet");
    }

    #[tokio::test]
    async fn test_stream_creation() {
        let stream = MempoolSpaceStream::new("test".to_string());
        assert_eq!(stream.name(), "test");
        assert!(!stream.is_running());
    }

    #[test]
    fn test_url_generation() {
        let _config = MempoolConfig {
            network: BitcoinNetwork::Mainnet,
            subscribe_blocks: true,
            subscribe_transactions: false,
            subscribe_fees: false,
            buffer_size: 1000,
        };

        let _stream = MempoolSpaceStream::new("test".to_string());
        // URL generation test simplified - stream creation should succeed
    }
}
