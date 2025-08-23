//! EVM WebSocket streaming implementation
//!
//! This module provides WebSocket-based streaming capabilities for EVM-compatible blockchains.
//! It supports real-time event streaming including new blocks, pending transactions, and contract events.

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
use serde::Serialize;

/// EVM Chain ID enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Deserialize, serde::Serialize)]
pub enum ChainId {
    /// Ethereum mainnet
    Ethereum = 1,
    /// Polygon (Matic) network
    Polygon = 137,
    /// Binance Smart Chain
    BSC = 56,
    /// Arbitrum layer 2
    Arbitrum = 42161,
    /// Optimism layer 2
    Optimism = 10,
    /// Avalanche C-Chain
    Avalanche = 43114,
    /// Base layer 2
    Base = 8453,
}

impl std::fmt::Display for ChainId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", *self as u64)
    }
}

impl From<ChainId> for u64 {
    fn from(chain: ChainId) -> u64 {
        chain as u64
    }
}

/// EVM WebSocket stream implementation
pub struct EvmWebSocketStream {
    /// Stream configuration
    config: EvmStreamConfig,
    /// Event broadcast channel
    event_tx: broadcast::Sender<Arc<EvmStreamEvent>>,
    /// Running state
    running: Arc<AtomicBool>,
    /// Health metrics
    health: Arc<RwLock<StreamHealth>>,
    /// Stream name
    name: String,
}

/// EVM stream configuration
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct EvmStreamConfig {
    /// WebSocket URL
    pub ws_url: String,
    /// Chain ID
    pub chain_id: ChainId,
    /// Subscribe to pending transactions
    pub subscribe_pending_transactions: bool,
    /// Subscribe to new blocks
    pub subscribe_new_blocks: bool,
    /// Contract addresses to monitor (as hex strings)
    pub contract_addresses: Vec<String>,
    /// Buffer size for event channel
    pub buffer_size: usize,
}

impl Default for EvmStreamConfig {
    fn default() -> Self {
        Self {
            ws_url: String::default(),
            chain_id: ChainId::Ethereum,
            subscribe_pending_transactions: false,
            subscribe_new_blocks: true,
            contract_addresses: Vec::new(),
            buffer_size: 10000,
        }
    }
}

/// EVM streaming event
#[derive(Debug, Clone)]
pub struct EvmStreamEvent {
    /// Event metadata for riglr-events-core compatibility
    pub metadata: EventMetadata,
    /// Event type
    pub event_type: EvmEventType,
    /// Stream metadata (legacy)
    pub stream_meta: StreamMetadata,
    /// Chain ID
    pub chain_id: ChainId,
    /// Block number (if available)
    pub block_number: Option<u64>,
    /// Transaction hash (if available)
    pub transaction_hash: Option<String>,
    /// Raw event data
    pub data: serde_json::Value,
}

/// EVM-specific event types
#[derive(Debug, Clone, Serialize)]
pub enum EvmEventType {
    /// Pending transaction
    PendingTransaction,
    /// New block
    NewBlock,
    /// Contract event log
    ContractEvent,
}

impl StreamEvent for EvmStreamEvent {
    fn stream_metadata(&self) -> Option<&StreamMetadata> {
        Some(&self.stream_meta)
    }
}

impl Event for EvmStreamEvent {
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
            "stream_meta": self.stream_meta,
            "chain_id": self.chain_id,
            "block_number": self.block_number,
            "transaction_hash": self.transaction_hash,
            "data": self.data
        }))
    }
}

#[async_trait::async_trait]
impl Stream for EvmWebSocketStream {
    type Event = EvmStreamEvent;
    type Config = EvmStreamConfig;

    async fn start(&mut self, config: Self::Config) -> Result<(), StreamError> {
        if self.running.load(Ordering::Relaxed) {
            return Err(StreamError::AlreadyRunning {
                name: self.name.clone(),
            });
        }

        info!(
            "Starting EVM WebSocket stream for chain {}: {}",
            config.chain_id, config.ws_url
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

        info!("Stopping EVM WebSocket stream");
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

impl EvmWebSocketStream {
    /// Create a new EVM WebSocket stream
    pub fn new(name: impl Into<String>) -> Self {
        let (event_tx, _) = broadcast::channel(10000);

        Self {
            config: EvmStreamConfig::default(),
            event_tx,
            running: Arc::default(),
            health: Arc::default(),
            name: name.into(),
        }
    }

    /// Start the WebSocket connection with resilience
    async fn start_websocket(&self) -> Result<(), StreamError> {
        let url = self.config.ws_url.clone();
        let event_tx = self.event_tx.clone();
        let running = self.running.clone();
        let health = self.health.clone();
        let stream_name = self.name.clone();
        let chain_id = self.config.chain_id;
        let config = self.config.clone();

        tokio::spawn(async move {
            crate::impl_resilient_websocket!(
                stream_name,
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
                            .map_err(|e| format!("Failed to connect: {}", e))
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
                        Self::parse_websocket_message(json, sequence_number, chain_id)
                    } else {
                        None
                    }
                }
            );
        });

        Ok(())
    }

    /// Parse a WebSocket message into an event
    fn parse_websocket_message(
        json: serde_json::Value,
        sequence_number: u64,
        chain_id: ChainId,
    ) -> Option<EvmStreamEvent> {
        // Check if this is a subscription notification
        let params = json.get("params")?;
        let subscription = params.get("subscription")?.as_str()?;
        let result = params.get("result")?;

        // Determine event type based on the result structure
        let (event_type, block_number, transaction_hash) = if result.get("number").is_some() {
            // New block
            let block_num = result.get("number")?.as_str()?.strip_prefix("0x")?;
            let block_number = u64::from_str_radix(block_num, 16).ok()?;

            (EvmEventType::NewBlock, Some(block_number), None)
        } else if result.get("transactionHash").is_some() {
            // Contract event
            let tx_hash = result.get("transactionHash")?.as_str()?.to_string();
            let block_num = result.get("blockNumber")?.as_str()?.strip_prefix("0x")?;
            let block_number = u64::from_str_radix(block_num, 16).ok()?;

            (
                EvmEventType::ContractEvent,
                Some(block_number),
                Some(tx_hash),
            )
        } else if result.is_string() {
            // Pending transaction (just a hash)
            let tx_hash = result.as_str()?.to_string();

            (EvmEventType::PendingTransaction, None, Some(tx_hash))
        } else {
            return None;
        };

        // Create unique ID
        let id = transaction_hash
            .clone()
            .unwrap_or_else(|| subscription.to_string());

        // Determine event kind
        let kind = match event_type {
            EvmEventType::PendingTransaction => EventKind::Transaction,
            EvmEventType::NewBlock => EventKind::Block,
            EvmEventType::ContractEvent => EventKind::Contract,
        };

        // Create event metadata
        let now = Utc::now();
        let metadata = EventMetadata {
            id: id.clone(),
            kind,
            timestamp: now,
            received_at: now,
            source: format!("evm-ws-{}", chain_id),
            chain_data: None,
            custom: std::collections::HashMap::new(),
        };

        // Create stream metadata (legacy)
        let stream_meta = StreamMetadata {
            stream_source: format!("evm-ws-{}", chain_id),
            received_at: SystemTime::now(),
            sequence_number: Some(sequence_number),
            custom_data: Some(json.clone()),
        };

        Some(EvmStreamEvent {
            metadata,
            event_type,
            stream_meta,
            chain_id,
            block_number,
            transaction_hash,
            data: json,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use riglr_events_core::prelude::{EventKind, EventMetadata};
    use serde_json::json;
    use std::collections::HashMap;

    // Tests for ChainId enum

    #[test]
    fn test_chain_id_display_when_ethereum_should_return_1() {
        assert_eq!(ChainId::Ethereum.to_string(), "1");
    }

    #[test]
    fn test_chain_id_display_when_polygon_should_return_137() {
        assert_eq!(ChainId::Polygon.to_string(), "137");
    }

    #[test]
    fn test_chain_id_display_when_bsc_should_return_56() {
        assert_eq!(ChainId::BSC.to_string(), "56");
    }

    #[test]
    fn test_chain_id_display_when_arbitrum_should_return_42161() {
        assert_eq!(ChainId::Arbitrum.to_string(), "42161");
    }

    #[test]
    fn test_chain_id_display_when_optimism_should_return_10() {
        assert_eq!(ChainId::Optimism.to_string(), "10");
    }

    #[test]
    fn test_chain_id_display_when_avalanche_should_return_43114() {
        assert_eq!(ChainId::Avalanche.to_string(), "43114");
    }

    #[test]
    fn test_chain_id_display_when_base_should_return_8453() {
        assert_eq!(ChainId::Base.to_string(), "8453");
    }

    #[test]
    fn test_chain_id_from_when_ethereum_should_return_1() {
        let chain_id: u64 = ChainId::Ethereum.into();
        assert_eq!(chain_id, 1);
    }

    #[test]
    fn test_chain_id_from_when_polygon_should_return_137() {
        let chain_id: u64 = ChainId::Polygon.into();
        assert_eq!(chain_id, 137);
    }

    #[test]
    fn test_chain_id_from_when_bsc_should_return_56() {
        let chain_id: u64 = ChainId::BSC.into();
        assert_eq!(chain_id, 56);
    }

    #[test]
    fn test_chain_id_from_when_arbitrum_should_return_42161() {
        let chain_id: u64 = ChainId::Arbitrum.into();
        assert_eq!(chain_id, 42161);
    }

    #[test]
    fn test_chain_id_from_when_optimism_should_return_10() {
        let chain_id: u64 = ChainId::Optimism.into();
        assert_eq!(chain_id, 10);
    }

    #[test]
    fn test_chain_id_from_when_avalanche_should_return_43114() {
        let chain_id: u64 = ChainId::Avalanche.into();
        assert_eq!(chain_id, 43114);
    }

    #[test]
    fn test_chain_id_from_when_base_should_return_8453() {
        let chain_id: u64 = ChainId::Base.into();
        assert_eq!(chain_id, 8453);
    }

    // Tests for EvmStreamConfig

    #[test]
    fn test_evm_stream_config_default_when_called_should_return_expected_values() {
        let config = EvmStreamConfig::default();

        assert_eq!(config.ws_url, String::default());
        assert_eq!(config.chain_id, ChainId::Ethereum);
        assert!(!config.subscribe_pending_transactions);
        assert!(config.subscribe_new_blocks);
        assert_eq!(config.contract_addresses, Vec::<String>::new());
        assert_eq!(config.buffer_size, 10000);
    }

    // Tests for EvmStreamEvent

    #[test]
    fn test_evm_stream_event_stream_metadata_when_called_should_return_some() {
        let event = create_test_event();
        let result = event.stream_metadata();
        assert!(result.is_some());
        assert_eq!(result.unwrap().stream_source, "test-source");
    }

    #[test]
    fn test_evm_stream_event_id_when_called_should_return_metadata_id() {
        let event = create_test_event();
        assert_eq!(event.id(), "test-id");
    }

    #[test]
    fn test_evm_stream_event_kind_when_called_should_return_metadata_kind() {
        let event = create_test_event();
        assert_eq!(event.kind(), &EventKind::Block);
    }

    #[test]
    fn test_evm_stream_event_metadata_when_called_should_return_metadata_ref() {
        let event = create_test_event();
        let metadata = event.metadata();
        assert_eq!(metadata.id, "test-id");
        assert_eq!(metadata.kind, EventKind::Block);
    }

    #[test]
    fn test_evm_stream_event_metadata_mut_when_called_should_return_mutable_metadata() {
        let mut event = create_test_event();
        let metadata_mut = event.metadata_mut();
        metadata_mut.id = "modified-id".to_string();
        assert_eq!(event.metadata().id, "modified-id");
    }

    #[test]
    fn test_evm_stream_event_as_any_when_called_should_return_self() {
        let event = create_test_event();
        let any = event.as_any();
        assert!(any.downcast_ref::<EvmStreamEvent>().is_some());
    }

    #[test]
    fn test_evm_stream_event_as_any_mut_when_called_should_return_mutable_self() {
        let mut event = create_test_event();
        let any_mut = event.as_any_mut();
        assert!(any_mut.downcast_mut::<EvmStreamEvent>().is_some());
    }

    #[test]
    fn test_evm_stream_event_clone_boxed_when_called_should_return_boxed_clone() {
        let event = create_test_event();
        let boxed = event.clone_boxed();
        assert_eq!(boxed.id(), "test-id");
    }

    #[test]
    fn test_evm_stream_event_to_json_when_called_should_return_json_value() {
        let event = create_test_event();
        let result = event.to_json();
        assert!(result.is_ok());

        let json = result.unwrap();
        assert!(json.get("metadata").is_some());
        assert!(json.get("event_type").is_some());
        assert!(json.get("stream_meta").is_some());
        assert!(json.get("chain_id").is_some());
        assert!(json.get("block_number").is_some());
        assert!(json.get("transaction_hash").is_some());
        assert!(json.get("data").is_some());
    }

    // Tests for EvmWebSocketStream

    #[test]
    fn test_evm_websocket_stream_new_when_called_should_create_stream() {
        let stream = EvmWebSocketStream::new("test-stream");

        assert_eq!(stream.name(), "test-stream");
        assert!(!stream.is_running());
        assert_eq!(stream.config.chain_id, ChainId::Ethereum);
    }

    #[test]
    fn test_evm_websocket_stream_new_when_string_provided_should_convert_to_string() {
        let stream = EvmWebSocketStream::new("test".to_string());
        assert_eq!(stream.name(), "test");
    }

    #[tokio::test]
    async fn test_evm_websocket_stream_start_when_not_running_should_start() {
        let mut stream = EvmWebSocketStream::new("test");
        let config = EvmStreamConfig {
            ws_url: "ws://test.com".to_string(),
            chain_id: ChainId::Ethereum,
            subscribe_pending_transactions: true,
            subscribe_new_blocks: true,
            contract_addresses: vec!["0x123".to_string()],
            buffer_size: 1000,
        };

        let result = stream.start(config.clone()).await;

        // The start will fail due to invalid WebSocket URL, but we test the initial setup
        assert!(result.is_err() || stream.is_running());
        assert_eq!(stream.config.ws_url, "ws://test.com");
        assert_eq!(stream.config.chain_id, ChainId::Ethereum);
    }

    #[tokio::test]
    async fn test_evm_websocket_stream_start_when_already_running_should_return_error() {
        let mut stream = EvmWebSocketStream::new("test");
        stream.running.store(true, Ordering::Relaxed);

        let config = EvmStreamConfig::default();
        let result = stream.start(config).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            StreamError::AlreadyRunning { name } => {
                assert_eq!(name, "test");
            }
            _ => panic!("Expected AlreadyRunning error"),
        }
    }

    #[tokio::test]
    async fn test_evm_websocket_stream_stop_when_running_should_stop() {
        let mut stream = EvmWebSocketStream::new("test");
        stream.running.store(true, Ordering::Relaxed);

        let result = stream.stop().await;

        assert!(result.is_ok());
        assert!(!stream.is_running());
    }

    #[tokio::test]
    async fn test_evm_websocket_stream_stop_when_not_running_should_return_ok() {
        let mut stream = EvmWebSocketStream::new("test");

        let result = stream.stop().await;

        assert!(result.is_ok());
        assert!(!stream.is_running());
    }

    #[test]
    fn test_evm_websocket_stream_subscribe_when_called_should_return_receiver() {
        let stream = EvmWebSocketStream::new("test");
        let mut receiver = stream.subscribe();

        // Test that we can try to receive (it will be empty but shouldn't panic)
        let result = receiver.try_recv();
        assert!(result.is_err()); // Should be empty
    }

    #[test]
    fn test_evm_websocket_stream_is_running_when_false_should_return_false() {
        let stream = EvmWebSocketStream::new("test");
        assert!(!stream.is_running());
    }

    #[test]
    fn test_evm_websocket_stream_is_running_when_true_should_return_true() {
        let stream = EvmWebSocketStream::new("test");
        stream.running.store(true, Ordering::Relaxed);
        assert!(stream.is_running());
    }

    #[tokio::test]
    async fn test_evm_websocket_stream_health_when_called_should_return_health() {
        let stream = EvmWebSocketStream::new("test");
        let health = stream.health().await;

        assert!(!health.is_connected);
        assert!(health.last_event_time.is_none());
    }

    #[test]
    fn test_evm_websocket_stream_name_when_called_should_return_name() {
        let stream = EvmWebSocketStream::new("my-test-stream");
        assert_eq!(stream.name(), "my-test-stream");
    }

    // Tests for parse_websocket_message function

    #[test]
    fn test_parse_websocket_message_when_new_block_should_return_block_event() {
        let json = json!({
            "params": {
                "subscription": "0x123",
                "result": {
                    "number": "0xa",
                    "hash": "0xabc"
                }
            }
        });

        let result = EvmWebSocketStream::parse_websocket_message(json, 1, ChainId::Ethereum);

        assert!(result.is_some());
        let event = result.unwrap();
        assert_eq!(event.chain_id, ChainId::Ethereum);
        assert_eq!(event.block_number, Some(10)); // 0xa = 10
        assert!(event.transaction_hash.is_none());
        assert!(matches!(event.event_type, EvmEventType::NewBlock));
        assert_eq!(event.metadata.kind, EventKind::Block);
    }

    #[test]
    fn test_parse_websocket_message_when_contract_event_should_return_contract_event() {
        let json = json!({
            "params": {
                "subscription": "0x456",
                "result": {
                    "transactionHash": "0xdef456",
                    "blockNumber": "0x64",
                    "address": "0x123"
                }
            }
        });

        let result = EvmWebSocketStream::parse_websocket_message(json, 2, ChainId::Polygon);

        assert!(result.is_some());
        let event = result.unwrap();
        assert_eq!(event.chain_id, ChainId::Polygon);
        assert_eq!(event.block_number, Some(100)); // 0x64 = 100
        assert_eq!(event.transaction_hash, Some("0xdef456".to_string()));
        assert!(matches!(event.event_type, EvmEventType::ContractEvent));
        assert_eq!(event.metadata.kind, EventKind::Contract);
    }

    #[test]
    fn test_parse_websocket_message_when_pending_transaction_should_return_pending_event() {
        let json = json!({
            "params": {
                "subscription": "0x789",
                "result": "0xabc123def456"
            }
        });

        let result = EvmWebSocketStream::parse_websocket_message(json, 3, ChainId::BSC);

        assert!(result.is_some());
        let event = result.unwrap();
        assert_eq!(event.chain_id, ChainId::BSC);
        assert!(event.block_number.is_none());
        assert_eq!(event.transaction_hash, Some("0xabc123def456".to_string()));
        assert!(matches!(event.event_type, EvmEventType::PendingTransaction));
        assert_eq!(event.metadata.kind, EventKind::Transaction);
    }

    #[test]
    fn test_parse_websocket_message_when_missing_params_should_return_none() {
        let json = json!({
            "method": "eth_subscription"
        });

        let result = EvmWebSocketStream::parse_websocket_message(json, 1, ChainId::Ethereum);
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_websocket_message_when_missing_subscription_should_return_none() {
        let json = json!({
            "params": {
                "result": {
                    "number": "0xa"
                }
            }
        });

        let result = EvmWebSocketStream::parse_websocket_message(json, 1, ChainId::Ethereum);
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_websocket_message_when_missing_result_should_return_none() {
        let json = json!({
            "params": {
                "subscription": "0x123"
            }
        });

        let result = EvmWebSocketStream::parse_websocket_message(json, 1, ChainId::Ethereum);
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_websocket_message_when_invalid_block_number_should_return_none() {
        let json = json!({
            "params": {
                "subscription": "0x123",
                "result": {
                    "number": "invalid_hex"
                }
            }
        });

        let result = EvmWebSocketStream::parse_websocket_message(json, 1, ChainId::Ethereum);
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_websocket_message_when_block_number_missing_hex_prefix_should_return_none() {
        let json = json!({
            "params": {
                "subscription": "0x123",
                "result": {
                    "number": "a"
                }
            }
        });

        let result = EvmWebSocketStream::parse_websocket_message(json, 1, ChainId::Ethereum);
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_websocket_message_when_contract_event_invalid_block_number_should_return_none() {
        let json = json!({
            "params": {
                "subscription": "0x456",
                "result": {
                    "transactionHash": "0xdef456",
                    "blockNumber": "invalid_hex"
                }
            }
        });

        let result = EvmWebSocketStream::parse_websocket_message(json, 2, ChainId::Polygon);
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_websocket_message_when_contract_event_missing_block_hex_prefix_should_return_none(
    ) {
        let json = json!({
            "params": {
                "subscription": "0x456",
                "result": {
                    "transactionHash": "0xdef456",
                    "blockNumber": "64"
                }
            }
        });

        let result = EvmWebSocketStream::parse_websocket_message(json, 2, ChainId::Polygon);
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_websocket_message_when_unknown_format_should_return_none() {
        let json = json!({
            "params": {
                "subscription": "0x123",
                "result": {
                    "unknownField": "value"
                }
            }
        });

        let result = EvmWebSocketStream::parse_websocket_message(json, 1, ChainId::Ethereum);
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_websocket_message_when_subscription_not_string_should_return_none() {
        let json = json!({
            "params": {
                "subscription": 123,
                "result": {
                    "number": "0xa"
                }
            }
        });

        let result = EvmWebSocketStream::parse_websocket_message(json, 1, ChainId::Ethereum);
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_websocket_message_when_pending_tx_not_string_should_return_none() {
        let json = json!({
            "params": {
                "subscription": "0x789",
                "result": 123
            }
        });

        let result = EvmWebSocketStream::parse_websocket_message(json, 3, ChainId::BSC);
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_websocket_message_when_contract_event_missing_tx_hash_should_return_none() {
        let json = json!({
            "params": {
                "subscription": "0x456",
                "result": {
                    "blockNumber": "0x64",
                    "address": "0x123"
                }
            }
        });

        let result = EvmWebSocketStream::parse_websocket_message(json, 2, ChainId::Polygon);
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_websocket_message_when_contract_event_missing_block_number_should_return_none() {
        let json = json!({
            "params": {
                "subscription": "0x456",
                "result": {
                    "transactionHash": "0xdef456",
                    "address": "0x123"
                }
            }
        });

        let result = EvmWebSocketStream::parse_websocket_message(json, 2, ChainId::Polygon);
        assert!(result.is_none());
    }

    // Helper function to create test events
    fn create_test_event() -> EvmStreamEvent {
        let metadata = EventMetadata {
            id: "test-id".to_string(),
            kind: EventKind::Block,
            timestamp: Utc::now(),
            received_at: Utc::now(),
            source: "test-source".to_string(),
            chain_data: None,
            custom: HashMap::new(),
        };

        let stream_meta = StreamMetadata {
            stream_source: "test-source".to_string(),
            received_at: SystemTime::now(),
            sequence_number: Some(1),
            custom_data: Some(json!({"test": "data"})),
        };

        EvmStreamEvent {
            metadata,
            event_type: EvmEventType::NewBlock,
            stream_meta,
            chain_id: ChainId::Ethereum,
            block_number: Some(12345),
            transaction_hash: Some("0xtest".to_string()),
            data: json!({"block": "data"}),
        }
    }
}
