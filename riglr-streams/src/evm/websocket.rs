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
    Ethereum = 1,
    Polygon = 137,
    BSC = 56,
    Arbitrum = 42161,
    Optimism = 10,
    Avalanche = 43114,
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
            ws_url: String::new(),
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
            running: Arc::new(AtomicBool::new(false)),
            health: Arc::new(RwLock::new(StreamHealth::default())),
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
            .as_ref()
            .unwrap_or(&subscription.to_string())
            .clone();

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
