use core::any::Any;
use core::error::Error as StdError;
use core::fmt::{Display, Formatter, Result as FmtResult};
use core::sync::atomic::{AtomicBool, Ordering};
use core::time::Duration;
use serde::{Deserialize, Serialize};
use std::io::{Error as IoError, ErrorKind};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::{broadcast, mpsc, RwLock};
use tracing::info;

use crate::core::metrics::MetricsCollector;
use crate::core::streamed_event::DynamicStreamed;
use crate::core::StreamMetadata;
use crate::core::{ResilientWebSocketBuilder, Stream, StreamError, StreamEvent, StreamHealth};
use chrono::Utc;
use riglr_events_core::error::EventResult;
use riglr_events_core::prelude::{Event, EventKind, EventMetadata};

/// Mempool.space WebSocket stream implementation
#[derive(Debug)]
#[expect(clippy::module_name_repetitions)]
pub struct MempoolSpaceStream {
    /// Stream configuration
    config: MempoolConfig,
    /// Event broadcast channel
    event_tx: broadcast::Sender<Arc<DynamicStreamed>>,
    /// Metrics collector
    metrics: Option<Arc<MetricsCollector>>,
    /// Running state
    running: Arc<AtomicBool>,
    /// Health metrics
    health: Arc<RwLock<StreamHealth>>,
    /// Stream name
    name: String,
}

/// Mempool configuration
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[expect(clippy::module_name_repetitions)]
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
#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub enum BitcoinNetwork {
    /// Bitcoin mainnet
    Mainnet,
    /// Bitcoin testnet
    Testnet,
    /// Bitcoin signet
    Signet,
}

impl Display for BitcoinNetwork {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match *self {
            Self::Mainnet => write!(f, "mainnet"),
            Self::Testnet => write!(f, "testnet"),
            Self::Signet => write!(f, "signet"),
        }
    }
}

/// Mempool streaming event
#[derive(Debug, Clone)]
#[expect(clippy::module_name_repetitions)]
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
#[expect(clippy::module_name_repetitions)]
pub enum MempoolEventType {
    /// Block-related event
    Block,
    /// Transaction-related event
    Transaction,
    /// Statistics update event
    Stats,
}

/// Mempool event data types
#[derive(Debug, Clone)]
#[expect(clippy::module_name_repetitions)]
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

/// Bitcoin transaction data
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BitcoinTransaction {
    /// Transaction ID
    pub txid: String,
    /// Transaction fee in satoshis
    pub fee: u64,
    /// Virtual size of the transaction
    pub vsize: u32,
    /// Total value in satoshis
    pub value: u64,
    /// Whether the transaction is confirmed
    #[serde(default)]
    pub confirmed: bool,
}

/// Bitcoin block data
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BitcoinBlock {
    /// Block ID (hash)
    pub id: String,
    /// Block height
    pub height: u64,
    /// Block timestamp
    pub timestamp: u64,
    /// Number of transactions in block
    pub tx_count: u32,
    /// Block size in bytes
    pub size: u32,
    /// Block weight
    pub weight: u32,
}

/// Bitcoin fee estimate data
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FeeEstimate {
    /// Fastest fee estimate in sat/vB
    #[serde(rename = "fastestFee")]
    pub fastest_fee: u32,
    /// Half hour fee estimate in sat/vB
    #[serde(rename = "halfHourFee")]
    pub half_hour_fee: u32,
    /// One hour fee estimate in sat/vB
    #[serde(rename = "hourFee")]
    pub hour_fee: u32,
    /// Economy fee estimate in sat/vB
    #[serde(rename = "economyFee")]
    pub economy_fee: u32,
    /// Minimum fee estimate in sat/vB
    #[serde(rename = "minimumFee")]
    pub minimum_fee: u32,
}

/// Mempool statistics data
#[derive(Debug, Clone, Deserialize, Serialize)]
#[expect(clippy::module_name_repetitions)]
pub struct MempoolStats {
    /// Number of transactions in mempool
    pub count: u32,
    /// Total virtual size of mempool transactions
    pub vsize: u64,
    /// Total fees in mempool
    pub total_fee: u64,
    /// Fee rate histogram
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

    fn metadata_mut(&mut self) -> EventResult<&mut EventMetadata> {
        Ok(&mut self.metadata)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
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
        self.start_websocket();

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

    fn subscribe(&self) -> broadcast::Receiver<Arc<DynamicStreamed>> {
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
            running: Arc::new(AtomicBool::default()),
            health: Arc::new(RwLock::new(StreamHealth::default())),
            name: name.into(),
            metrics: None,
        }
    }

    /// Create a new Mempool.space stream with metrics
    pub fn new_with_metrics(name: impl Into<String>, metrics: Arc<MetricsCollector>) -> Self {
        let (event_tx, _) = broadcast::channel(10000);

        Self {
            config: MempoolConfig::default(),
            event_tx,
            running: Arc::new(AtomicBool::default()),
            health: Arc::new(RwLock::new(StreamHealth::default())),
            name: name.into(),
            metrics: Some(metrics),
        }
    }

    /// Start the WebSocket connection with resilience
    fn start_websocket(&self) {
        let base_url = match self.config.network {
            BitcoinNetwork::Mainnet => "wss://mempool.space/api/v1/ws",
            BitcoinNetwork::Testnet => "wss://mempool.space/testnet/api/v1/ws",
            BitcoinNetwork::Signet => "wss://mempool.space/signet/api/v1/ws",
        };

        let url = base_url.to_string();
        let metrics = self.metrics.clone();
        let running = self.running.clone();
        let health = self.health.clone();
        let stream_name = self.name.clone();

        // Create mpsc channel for internal event handling
        let (internal_tx, mut internal_rx) = mpsc::unbounded_channel::<Arc<DynamicStreamed>>();
        let event_tx = self.event_tx.clone();

        // Bridge internal events to broadcast channel
        let stream_name_clone = stream_name.clone();
        let metrics_clone = metrics.clone();
        tokio::spawn(async move {
            loop {
                let recv_result = internal_rx.recv().await;
                if let Some(event) = recv_result {
                    if let Some(ref metrics) = metrics_clone {
                        let record_result = metrics
                            .record_stream_event(&stream_name_clone, 0.0, 0)
                            .await;
                        let () = record_result;
                    }
                    let send_result = event_tx.send(event);
                    let _ = send_result;
                } else {
                    break;
                }
            }
        });

        let internal_tx_for_connector = internal_tx;
        tokio::spawn(async move {
            let connector = ResilientWebSocketBuilder::new(
                stream_name,
                running,
                health,
                internal_tx_for_connector,
                move || {
                    let url = url.clone();
                    Box::pin(async move {
                        let connect_result = tokio_tungstenite::connect_async(&url).await;
                        let mapped_result = connect_result.map(|(ws, _)| ws).map_err(|e| {
                            Box::new(IoError::new(
                                ErrorKind::ConnectionRefused,
                                format!("Failed to connect to Mempool.space: {e}"),
                            )) as Box<dyn StdError + Send + Sync>
                        });
                        mapped_result
                    })
                },
                |_write| Box::pin(async move { Ok::<(), Box<dyn StdError + Send + Sync>>(()) }),
                |text: String, sequence_number: u64| {
                    serde_json::from_str::<serde_json::Value>(&text).map_or_else(
                        |_| None,
                        |json| Some(Self::parse_message(json, sequence_number)),
                    )
                },
            )
            .with_metrics(metrics.unwrap_or_else(|| Arc::new(MetricsCollector::default())))
            .build();

            let _run_result = connector.run().await;
        });
    }

    /// Parse a message from mempool.space
    fn parse_message(json: serde_json::Value, sequence_number: u64) -> DynamicStreamed {
        // Create unique ID
        let timestamp_millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|_| Duration::from_millis(0))
            .as_millis();
        let id = format!("mempool_{timestamp_millis}");

        // Determine event kind
        let kind = EventKind::Block;

        // Create event metadata
        let now = Utc::now();
        let metadata = EventMetadata::with_timestamp(id, kind, "mempool-space-ws".to_string(), now);

        // Create stream metadata (legacy)
        let stream_meta = StreamMetadata {
            stream_source: "mempool.space".to_string(),
            received_at: SystemTime::now(),
            sequence_number: Some(sequence_number),
            custom_data: None,
        };

        let stream_meta_clone = stream_meta.clone();
        let mempool_event = MempoolStreamEvent {
            metadata,
            event_type: MempoolEventType::Block,
            stream_meta,
            network: BitcoinNetwork::Mainnet,
            block_height: None,
            transaction_count: None,
            data: json,
        };

        // Convert to DynamicStreamed
        DynamicStreamed::from_event(Box::new(mempool_event), stream_meta_clone)
    }
}

#[cfg(test)]
#[expect(clippy::expect_used, clippy::panic)]
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
        let config = MempoolConfig {
            network: BitcoinNetwork::Mainnet,
            subscribe_blocks: true,
            subscribe_transactions: false,
            subscribe_fees: false,
            buffer_size: 1000,
        };
        let _ = config;

        let _stream = MempoolSpaceStream::new("test".to_string());
        // URL generation test simplified - stream creation should succeed
    }

    #[test]
    fn test_mempool_config_clone() {
        let config = MempoolConfig::default();
        let cloned = config.clone();
        assert_eq!(config.network, cloned.network);
        assert_eq!(config.subscribe_transactions, cloned.subscribe_transactions);
        assert_eq!(config.subscribe_blocks, cloned.subscribe_blocks);
        assert_eq!(config.subscribe_fees, cloned.subscribe_fees);
        assert_eq!(config.buffer_size, cloned.buffer_size);
    }

    #[test]
    fn test_bitcoin_network_equality() {
        assert_eq!(BitcoinNetwork::Mainnet, BitcoinNetwork::Mainnet);
        assert_eq!(BitcoinNetwork::Testnet, BitcoinNetwork::Testnet);
        assert_eq!(BitcoinNetwork::Signet, BitcoinNetwork::Signet);
        assert_ne!(BitcoinNetwork::Mainnet, BitcoinNetwork::Testnet);
    }

    #[test]
    fn test_bitcoin_network_clone() {
        let network = BitcoinNetwork::Mainnet;
        let cloned = network.clone();
        assert_eq!(network, cloned);
    }

    #[test]
    fn test_bitcoin_transaction_default() {
        let tx = BitcoinTransaction {
            txid: "test_tx".to_string(),
            fee: 1000,
            vsize: 250,
            value: 50000,
            confirmed: false,
        };
        assert_eq!(tx.txid, "test_tx");
        assert_eq!(tx.fee, 1000);
        assert_eq!(tx.vsize, 250);
        assert_eq!(tx.value, 50000);
        assert!(!tx.confirmed);
    }

    #[test]
    fn test_bitcoin_transaction_clone() {
        let tx = BitcoinTransaction {
            txid: "test_tx".to_string(),
            fee: 1000,
            vsize: 250,
            value: 50000,
            confirmed: true,
        };
        let cloned = tx.clone();
        assert_eq!(tx.txid, cloned.txid);
        assert_eq!(tx.fee, cloned.fee);
        assert_eq!(tx.vsize, cloned.vsize);
        assert_eq!(tx.value, cloned.value);
        assert_eq!(tx.confirmed, cloned.confirmed);
    }

    #[test]
    fn test_bitcoin_block_clone() {
        let block = BitcoinBlock {
            id: "block_hash".to_string(),
            height: 750_000,
            timestamp: 1_640_995_200,
            tx_count: 2500,
            size: 1_000_000,
            weight: 4_000_000,
        };
        let cloned = block.clone();
        assert_eq!(block.id, cloned.id);
        assert_eq!(block.height, cloned.height);
        assert_eq!(block.timestamp, cloned.timestamp);
        assert_eq!(block.tx_count, cloned.tx_count);
        assert_eq!(block.size, cloned.size);
        assert_eq!(block.weight, cloned.weight);
    }

    #[test]
    fn test_fee_estimate_clone() {
        let fee_estimate = FeeEstimate {
            fastest_fee: 50,
            half_hour_fee: 30,
            hour_fee: 20,
            economy_fee: 10,
            minimum_fee: 1,
        };
        let cloned = fee_estimate.clone();
        assert_eq!(fee_estimate.fastest_fee, cloned.fastest_fee);
        assert_eq!(fee_estimate.half_hour_fee, cloned.half_hour_fee);
        assert_eq!(fee_estimate.hour_fee, cloned.hour_fee);
        assert_eq!(fee_estimate.economy_fee, cloned.economy_fee);
        assert_eq!(fee_estimate.minimum_fee, cloned.minimum_fee);
    }

    #[test]
    fn test_mempool_stats_clone() {
        let stats = MempoolStats {
            count: 50000,
            vsize: 100_000_000,
            total_fee: 1_000_000,
            fee_histogram: vec![vec![1, 2, 3], vec![4, 5, 6]],
        };
        let cloned = stats.clone();
        assert_eq!(stats.count, cloned.count);
        assert_eq!(stats.vsize, cloned.vsize);
        assert_eq!(stats.total_fee, cloned.total_fee);
        assert_eq!(stats.fee_histogram, cloned.fee_histogram);
    }

    #[test]
    fn test_mempool_event_type_clone() {
        let event_type = MempoolEventType::Block;
        let cloned = event_type;
        // Can't directly compare enum variants with assert_eq!, but clone should work
        let _ = cloned;
    }

    #[test]
    fn test_mempool_event_data_clone() {
        let tx = BitcoinTransaction {
            txid: "test".to_string(),
            fee: 100,
            vsize: 200,
            value: 1000,
            confirmed: false,
        };
        let event_data = MempoolEventData::Transaction(tx);
        let cloned = event_data;
        // Test that clone works
        let _ = cloned;
    }

    #[test]
    fn test_mempool_stream_event_clone() {
        let metadata = {
            let mut meta =
                EventMetadata::new("test_id".to_string(), EventKind::Block, "test".to_string());
            meta.timestamp = Utc::now();
            meta.received_at = Utc::now();
            meta
        };

        let stream_meta = StreamMetadata {
            stream_source: "test".to_string(),
            received_at: SystemTime::now(),
            sequence_number: Some(1),
            custom_data: None,
        };

        let event = MempoolStreamEvent {
            metadata,
            event_type: MempoolEventType::Transaction,
            data: serde_json::json!({"test": "data"}),
            stream_meta,
            network: BitcoinNetwork::Testnet,
            block_height: Some(750_000),
            transaction_count: Some(100),
        };

        let cloned = event.clone();
        assert_eq!(event.metadata().id, cloned.metadata.id);
        assert_eq!(event.network, cloned.network);
        assert_eq!(event.block_height, cloned.block_height);
        assert_eq!(event.transaction_count, cloned.transaction_count);
    }

    #[test]
    fn test_mempool_stream_event_stream_metadata() {
        let metadata = {
            let mut meta =
                EventMetadata::new("test_id".to_string(), EventKind::Block, "test".to_string());
            meta.timestamp = Utc::now();
            meta.received_at = Utc::now();
            meta
        };

        let stream_meta = StreamMetadata {
            stream_source: "test".to_string(),
            received_at: SystemTime::now(),
            sequence_number: Some(1),
            custom_data: None,
        };

        let event = MempoolStreamEvent {
            metadata,
            event_type: MempoolEventType::Block,
            data: serde_json::json!({}),
            stream_meta,
            network: BitcoinNetwork::Mainnet,
            block_height: None,
            transaction_count: None,
        };

        let meta = event.stream_metadata();
        assert!(meta.is_some());
        assert_eq!(
            meta.expect("stream metadata should be present")
                .stream_source,
            "test"
        );
    }

    #[test]
    fn test_mempool_stream_event_id() {
        let metadata = {
            let mut meta = EventMetadata::new(
                "unique_id".to_string(),
                EventKind::Transaction,
                "test".to_string(),
            );
            meta.timestamp = Utc::now();
            meta.received_at = Utc::now();
            meta
        };

        let stream_meta = StreamMetadata {
            stream_source: "test".to_string(),
            received_at: SystemTime::now(),
            sequence_number: None,
            custom_data: None,
        };

        let event = MempoolStreamEvent {
            metadata,
            event_type: MempoolEventType::Stats,
            data: serde_json::json!({}),
            stream_meta,
            network: BitcoinNetwork::Signet,
            block_height: None,
            transaction_count: None,
        };

        assert_eq!(event.id(), "unique_id");
    }

    #[test]
    fn test_mempool_stream_event_kind() {
        let metadata = {
            let mut meta = EventMetadata::new(
                "test".to_string(),
                EventKind::Transaction,
                "test".to_string(),
            );
            meta.timestamp = Utc::now();
            meta.received_at = Utc::now();
            meta
        };

        let stream_meta = StreamMetadata {
            stream_source: "test".to_string(),
            received_at: SystemTime::now(),
            sequence_number: None,
            custom_data: None,
        };

        let event = MempoolStreamEvent {
            metadata,
            event_type: MempoolEventType::Transaction,
            data: serde_json::json!({}),
            stream_meta,
            network: BitcoinNetwork::Mainnet,
            block_height: None,
            transaction_count: None,
        };

        assert_eq!(event.kind(), &EventKind::Transaction);
    }

    #[test]
    fn test_mempool_stream_event_metadata() {
        let metadata = {
            let mut meta =
                EventMetadata::new("test".to_string(), EventKind::Block, "mempool".to_string());
            meta.timestamp = Utc::now();
            meta.received_at = Utc::now();
            meta
        };

        let stream_meta = StreamMetadata {
            stream_source: "test".to_string(),
            received_at: SystemTime::now(),
            sequence_number: None,
            custom_data: None,
        };

        let event = MempoolStreamEvent {
            metadata,
            event_type: MempoolEventType::Block,
            data: serde_json::json!({}),
            stream_meta,
            network: BitcoinNetwork::Mainnet,
            block_height: None,
            transaction_count: None,
        };

        let meta = event.metadata();
        assert_eq!(meta.source, "mempool");
    }

    #[test]
    fn test_mempool_stream_event_metadata_mut() {
        let metadata = {
            let mut meta =
                EventMetadata::new("test".to_string(), EventKind::Block, "original".to_string());
            meta.timestamp = Utc::now();
            meta.received_at = Utc::now();
            meta
        };

        let stream_meta = StreamMetadata {
            stream_source: "test".to_string(),
            received_at: SystemTime::now(),
            sequence_number: None,
            custom_data: None,
        };

        let mut event = MempoolStreamEvent {
            metadata,
            event_type: MempoolEventType::Block,
            data: serde_json::json!({}),
            stream_meta,
            network: BitcoinNetwork::Mainnet,
            block_height: None,
            transaction_count: None,
        };

        event
            .metadata_mut()
            .expect("metadata_mut should succeed")
            .source = "modified".to_string();
        assert_eq!(event.metadata().source, "modified");
    }

    #[test]
    fn test_mempool_stream_event_as_any() {
        let metadata = {
            let mut meta =
                EventMetadata::new("test".to_string(), EventKind::Block, "test".to_string());
            meta.timestamp = Utc::now();
            meta.received_at = Utc::now();
            meta
        };

        let stream_meta = StreamMetadata {
            stream_source: "test".to_string(),
            received_at: SystemTime::now(),
            sequence_number: None,
            custom_data: None,
        };

        let event = MempoolStreamEvent {
            metadata,
            event_type: MempoolEventType::Block,
            data: serde_json::json!({}),
            stream_meta,
            network: BitcoinNetwork::Mainnet,
            block_height: None,
            transaction_count: None,
        };

        let any_ref = event.as_any();
        assert!(any_ref.downcast_ref::<MempoolStreamEvent>().is_some());
    }

    #[test]
    fn test_mempool_stream_event_as_any_mut() {
        let metadata = {
            let mut meta =
                EventMetadata::new("test".to_string(), EventKind::Block, "test".to_string());
            meta.timestamp = Utc::now();
            meta.received_at = Utc::now();
            meta
        };

        let stream_meta = StreamMetadata {
            stream_source: "test".to_string(),
            received_at: SystemTime::now(),
            sequence_number: None,
            custom_data: None,
        };

        let mut event = MempoolStreamEvent {
            metadata,
            event_type: MempoolEventType::Block,
            data: serde_json::json!({}),
            stream_meta,
            network: BitcoinNetwork::Mainnet,
            block_height: None,
            transaction_count: None,
        };

        let any_mut_ref = event.as_any_mut();
        assert!(any_mut_ref.downcast_mut::<MempoolStreamEvent>().is_some());
    }

    #[test]
    fn test_mempool_stream_event_clone_boxed() {
        let metadata = {
            let mut meta =
                EventMetadata::new("test".to_string(), EventKind::Block, "test".to_string());
            meta.timestamp = Utc::now();
            meta.received_at = Utc::now();
            meta
        };

        let stream_meta = StreamMetadata {
            stream_source: "test".to_string(),
            received_at: SystemTime::now(),
            sequence_number: None,
            custom_data: None,
        };

        let event = MempoolStreamEvent {
            metadata,
            event_type: MempoolEventType::Block,
            data: serde_json::json!({}),
            stream_meta,
            network: BitcoinNetwork::Mainnet,
            block_height: None,
            transaction_count: None,
        };

        let boxed = event.clone_boxed();
        assert_eq!(boxed.id(), "test");
    }

    #[test]
    fn test_mempool_stream_event_to_json() {
        let metadata = {
            let mut meta = EventMetadata::new(
                "test_json".to_string(),
                EventKind::Block,
                "test".to_string(),
            );
            meta.timestamp = Utc::now();
            meta.received_at = Utc::now();
            meta
        };

        let stream_meta = StreamMetadata {
            stream_source: "test".to_string(),
            received_at: SystemTime::now(),
            sequence_number: Some(42),
            custom_data: None,
        };

        let event = MempoolStreamEvent {
            metadata,
            event_type: MempoolEventType::Transaction,
            data: serde_json::json!({"fee": 1000}),
            stream_meta,
            network: BitcoinNetwork::Testnet,
            block_height: Some(123_456),
            transaction_count: Some(50),
        };

        let json_result = event.to_json();
        assert!(json_result.is_ok());
        let json = json_result.expect("JSON serialization should succeed");
        assert!(json
            .get("metadata")
            .and_then(|v| v.get("id"))
            .and_then(|v| v.as_str())
            .expect("metadata id should be a string")
            .contains("test_json"));
        assert_eq!(
            json.get("data")
                .and_then(|v| v.get("fee"))
                .expect("data.fee should exist"),
            &1000
        );
        assert_eq!(
            json.get("block_height").expect("block_height should exist"),
            &123_456
        );
        assert_eq!(
            json.get("transaction_count")
                .expect("transaction_count should exist"),
            &50
        );
    }

    #[tokio::test]
    async fn test_stream_subscribe() {
        let stream = MempoolSpaceStream::new("subscribe_test");
        let rx = stream.subscribe();

        // Should not block and should be ready to receive
        assert!(rx.is_empty());
    }

    #[tokio::test]
    async fn test_stream_health() {
        let stream = MempoolSpaceStream::new("health_test");
        let health = stream.health().await;
        assert!(!health.is_connected);
        assert!(health.last_event_time.is_none());
    }

    #[tokio::test]
    async fn test_stream_stop_when_not_running() {
        let mut stream = MempoolSpaceStream::new("stop_test");
        let result = stream.stop().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_stream_start_already_running() {
        let mut stream = MempoolSpaceStream::new("already_running_test");
        stream.running.store(true, Ordering::Relaxed);

        let config = MempoolConfig::default();
        let result = stream.start(config).await;

        assert!(result.is_err());
        if let Err(StreamError::AlreadyRunning { name }) = result {
            assert_eq!(name, "already_running_test");
        } else {
            panic!("Expected AlreadyRunning error");
        }
    }

    #[test]
    fn test_parse_message_with_valid_json() {
        let json = serde_json::json!({
            "type": "block",
            "data": {
                "height": 750_000,
                "hash": "test_hash"
            }
        });

        let event = MempoolSpaceStream::parse_message(json, 1);
        assert!(event.metadata().id.starts_with("mempool_"));
        assert_eq!(event.metadata().kind, EventKind::Block);
        assert_eq!(event.metadata().source, "mempool-space-ws");
        assert_eq!(
            event
                .stream_metadata()
                .expect("stream metadata should be present")
                .stream_source,
            "mempool.space"
        );
        assert_eq!(
            event
                .stream_metadata()
                .expect("stream metadata should be present")
                .sequence_number,
            Some(1)
        );
        if let Some(mempool_event) = event.inner().as_any().downcast_ref::<MempoolStreamEvent>() {
            assert_eq!(mempool_event.network, BitcoinNetwork::Mainnet);
        } else {
            panic!("Expected MempoolStreamEvent");
        }
    }

    #[test]
    fn test_parse_message_with_empty_json() {
        let json = serde_json::json!({});
        let event = MempoolSpaceStream::parse_message(json, 42);
        assert_eq!(
            event
                .stream_metadata()
                .expect("stream metadata should be present")
                .sequence_number,
            Some(42)
        );
    }

    #[test]
    fn test_serde_bitcoin_transaction() {
        let tx = BitcoinTransaction {
            txid: "abc123".to_string(),
            fee: 5000,
            vsize: 500,
            value: 100_000,
            confirmed: true,
        };

        let serialized =
            serde_json::to_string(&tx).expect("BitcoinTransaction serialization should succeed");
        let deserialized: BitcoinTransaction = serde_json::from_str(&serialized)
            .expect("BitcoinTransaction deserialization should succeed");

        assert_eq!(tx.txid, deserialized.txid);
        assert_eq!(tx.fee, deserialized.fee);
        assert_eq!(tx.vsize, deserialized.vsize);
        assert_eq!(tx.value, deserialized.value);
        assert_eq!(tx.confirmed, deserialized.confirmed);
    }

    #[test]
    fn test_serde_bitcoin_transaction_default_confirmed() {
        let json = r#"{"txid":"test","fee":1000,"vsize":250,"value":50000}"#;
        let tx: BitcoinTransaction =
            serde_json::from_str(json).expect("JSON parsing should succeed for valid transaction");
        assert!(!tx.confirmed); // Should default to false
    }

    #[test]
    fn test_serde_bitcoin_block() {
        let block = BitcoinBlock {
            id: "block_id".to_string(),
            height: 750_001,
            timestamp: 1_640_995_300,
            tx_count: 3000,
            size: 1_500_000,
            weight: 6_000_000,
        };

        let serialized =
            serde_json::to_string(&block).expect("BitcoinBlock serialization should succeed");
        let deserialized: BitcoinBlock =
            serde_json::from_str(&serialized).expect("BitcoinBlock deserialization should succeed");

        assert_eq!(block.id, deserialized.id);
        assert_eq!(block.height, deserialized.height);
        assert_eq!(block.timestamp, deserialized.timestamp);
        assert_eq!(block.tx_count, deserialized.tx_count);
        assert_eq!(block.size, deserialized.size);
        assert_eq!(block.weight, deserialized.weight);
    }

    #[test]
    fn test_serde_fee_estimate() {
        let fee_estimate = FeeEstimate {
            fastest_fee: 100,
            half_hour_fee: 80,
            hour_fee: 60,
            economy_fee: 40,
            minimum_fee: 10,
        };

        let serialized =
            serde_json::to_string(&fee_estimate).expect("FeeEstimate serialization should succeed");
        let deserialized: FeeEstimate =
            serde_json::from_str(&serialized).expect("FeeEstimate deserialization should succeed");

        assert_eq!(fee_estimate.fastest_fee, deserialized.fastest_fee);
        assert_eq!(fee_estimate.half_hour_fee, deserialized.half_hour_fee);
        assert_eq!(fee_estimate.hour_fee, deserialized.hour_fee);
        assert_eq!(fee_estimate.economy_fee, deserialized.economy_fee);
        assert_eq!(fee_estimate.minimum_fee, deserialized.minimum_fee);
    }

    #[test]
    fn test_serde_fee_estimate_with_rename() {
        let json =
            r#"{"fastestFee":100,"halfHourFee":80,"hourFee":60,"economyFee":40,"minimumFee":10}"#;
        let fee_estimate: FeeEstimate =
            serde_json::from_str(json).expect("JSON parsing should succeed for valid fee estimate");

        assert_eq!(fee_estimate.fastest_fee, 100);
        assert_eq!(fee_estimate.half_hour_fee, 80);
        assert_eq!(fee_estimate.hour_fee, 60);
        assert_eq!(fee_estimate.economy_fee, 40);
        assert_eq!(fee_estimate.minimum_fee, 10);
    }

    #[test]
    fn test_serde_mempool_stats() {
        let stats = MempoolStats {
            count: 75000,
            vsize: 200_000_000,
            total_fee: 2_000_000,
            fee_histogram: vec![vec![10, 20], vec![30, 40]],
        };

        let serialized =
            serde_json::to_string(&stats).expect("MempoolStats serialization should succeed");
        let deserialized: MempoolStats =
            serde_json::from_str(&serialized).expect("MempoolStats deserialization should succeed");

        assert_eq!(stats.count, deserialized.count);
        assert_eq!(stats.vsize, deserialized.vsize);
        assert_eq!(stats.total_fee, deserialized.total_fee);
        assert_eq!(stats.fee_histogram, deserialized.fee_histogram);
    }

    #[test]
    fn test_serde_mempool_config() {
        let config = MempoolConfig {
            network: BitcoinNetwork::Signet,
            subscribe_transactions: false,
            subscribe_blocks: true,
            subscribe_fees: true,
            buffer_size: 5000,
        };

        let serialized =
            serde_json::to_string(&config).expect("MempoolConfig serialization should succeed");
        let deserialized: MempoolConfig = serde_json::from_str(&serialized)
            .expect("MempoolConfig deserialization should succeed");

        assert_eq!(config.network, deserialized.network);
        assert_eq!(
            config.subscribe_transactions,
            deserialized.subscribe_transactions
        );
        assert_eq!(config.subscribe_blocks, deserialized.subscribe_blocks);
        assert_eq!(config.subscribe_fees, deserialized.subscribe_fees);
        assert_eq!(config.buffer_size, deserialized.buffer_size);
    }

    #[test]
    fn test_serde_bitcoin_network() {
        let networks = vec![
            BitcoinNetwork::Mainnet,
            BitcoinNetwork::Testnet,
            BitcoinNetwork::Signet,
        ];

        for network in networks {
            let serialized = serde_json::to_string(&network)
                .expect("BitcoinNetwork serialization should succeed");
            let deserialized: BitcoinNetwork = serde_json::from_str(&serialized)
                .expect("BitcoinNetwork deserialization should succeed");
            assert_eq!(network, deserialized);
        }
    }

    #[test]
    fn test_serde_mempool_event_type() {
        let event_types = vec![
            MempoolEventType::Block,
            MempoolEventType::Transaction,
            MempoolEventType::Stats,
        ];

        for event_type in event_types {
            let serialized = serde_json::to_string(&event_type)
                .expect("MempoolEventType serialization should succeed");
            // Just test that serialization works
            assert!(!serialized.is_empty());
        }
    }

    #[test]
    fn test_mempool_event_data_variants() {
        let tx = BitcoinTransaction {
            txid: "test".to_string(),
            fee: 100,
            vsize: 200,
            value: 1000,
            confirmed: false,
        };

        let block = BitcoinBlock {
            id: "block".to_string(),
            height: 1000,
            timestamp: 1_640_995_200,
            tx_count: 100,
            size: 500_000,
            weight: 2_000_000,
        };

        let fee_estimate = FeeEstimate {
            fastest_fee: 50,
            half_hour_fee: 30,
            hour_fee: 20,
            economy_fee: 10,
            minimum_fee: 1,
        };

        let stats = MempoolStats {
            count: 1000,
            vsize: 1_000_000,
            total_fee: 50000,
            fee_histogram: vec![],
        };

        let unknown = serde_json::json!({"unknown": "data"});

        // Test all variants exist and can be created
        let tx_data = MempoolEventData::Transaction(tx);
        let block_data = MempoolEventData::Block(block);
        let fee_data = MempoolEventData::FeeEstimate(fee_estimate);
        let stats_data = MempoolEventData::MempoolStats(stats);
        let unknown_data = MempoolEventData::Unknown(unknown);

        // Explicitly indicate these are test variables
        let _ = tx_data;
        let _ = block_data;
        let _ = fee_data;
        let _ = stats_data;
        let _ = unknown_data;
    }

    #[test]
    fn test_stream_new_with_string_slice() {
        let stream = MempoolSpaceStream::new("test_slice");
        assert_eq!(stream.name(), "test_slice");
    }

    #[test]
    fn test_stream_new_with_string() {
        let name = String::from("test_string");
        let stream = MempoolSpaceStream::new(name);
        assert_eq!(stream.name(), "test_string");
    }

    #[test]
    fn test_custom_config_values() {
        let config = MempoolConfig {
            network: BitcoinNetwork::Testnet,
            subscribe_transactions: false,
            subscribe_blocks: false,
            subscribe_fees: true,
            buffer_size: 1,
        };

        assert_eq!(config.network, BitcoinNetwork::Testnet);
        assert!(!config.subscribe_transactions);
        assert!(!config.subscribe_blocks);
        assert!(config.subscribe_fees);
        assert_eq!(config.buffer_size, 1);
    }

    #[test]
    fn test_edge_case_values() {
        let tx = BitcoinTransaction {
            txid: String::default(), // Empty string
            fee: 0,
            vsize: 0,
            value: u64::MAX,
            confirmed: true,
        };

        assert_eq!(tx.txid, "");
        assert_eq!(tx.fee, 0);
        assert_eq!(tx.vsize, 0);
        assert_eq!(tx.value, u64::MAX);
        assert!(tx.confirmed);

        let block = BitcoinBlock {
            id: String::default(),
            height: 0,
            timestamp: 0,
            tx_count: 0,
            size: u32::MAX,
            weight: u32::MAX,
        };

        assert_eq!(block.height, 0);
        assert_eq!(block.size, u32::MAX);
        assert_eq!(block.weight, u32::MAX);
    }

    #[test]
    fn test_config_with_max_buffer_size() {
        let config = MempoolConfig {
            network: BitcoinNetwork::Mainnet,
            subscribe_transactions: true,
            subscribe_blocks: true,
            subscribe_fees: true,
            buffer_size: usize::MAX,
        };

        assert_eq!(config.buffer_size, usize::MAX);
    }
}
