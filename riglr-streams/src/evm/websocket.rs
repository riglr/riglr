use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::SystemTime;
use tokio::sync::{broadcast, RwLock};
use tokio_tungstenite::{connect_async, tungstenite::Message, WebSocketStream, MaybeTlsStream};
use serde::{Deserialize, Serialize};
use tracing::{info, warn, error, debug};
use futures::{StreamExt, SinkExt, FutureExt};

use riglr_solana_events::{UnifiedEvent, EventType, ProtocolType};
use crate::core::StreamMetadata;
use crate::core::{Stream, StreamError, StreamEvent, StreamHealth};

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
    /// Event ID
    pub id: String,
    /// Event type
    pub event_type: EvmEventType,
    /// Stream metadata
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
#[derive(Debug, Clone)]
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

impl UnifiedEvent for EvmStreamEvent {
    fn id(&self) -> &str {
        &self.id
    }
    
    fn event_type(&self) -> EventType {
        match self.event_type {
            EvmEventType::PendingTransaction => EventType::Transaction,
            EvmEventType::NewBlock => EventType::Block,
            EvmEventType::ContractEvent => EventType::ContractEvent,
        }
    }
    
    fn signature(&self) -> &str {
        self.transaction_hash.as_deref().unwrap_or(&self.id)
    }
    
    fn slot(&self) -> u64 {
        self.block_number.unwrap_or(0)
    }
    
    fn program_received_time_ms(&self) -> i64 {
        self.stream_meta.received_at
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as i64
    }
    
    fn program_handle_time_consuming_ms(&self) -> i64 {
        0
    }
    
    fn set_program_handle_time_consuming_ms(&mut self, _time: i64) {}
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
    
    fn clone_boxed(&self) -> Box<dyn UnifiedEvent> {
        Box::new(self.clone())
    }
    
    fn set_transfer_data(
        &mut self,
        _transfer_data: Vec<riglr_solana_events::TransferData>,
        _swap_data: Option<riglr_solana_events::SwapData>,
    ) {}
    
    fn index(&self) -> String {
        "0".to_string()
    }
    
    fn protocol_type(&self) -> ProtocolType {
        ProtocolType::Other("EVM".to_string())
    }
    
    fn timestamp(&self) -> SystemTime {
        self.stream_meta.received_at
    }
    
    fn transaction_hash(&self) -> Option<String> {
        self.transaction_hash.clone()
    }
    
    fn block_number(&self) -> Option<u64> {
        self.block_number
    }
}

#[async_trait::async_trait]
impl Stream for EvmWebSocketStream {
    type Event = EvmStreamEvent;
    type Config = EvmStreamConfig;
    
    async fn start(&mut self, config: Self::Config) -> Result<(), StreamError> {
        if self.running.load(Ordering::Relaxed) {
            return Err(StreamError::AlreadyRunning { 
                name: self.name.clone() 
            });
        }
        
        info!("Starting EVM WebSocket stream for chain {}: {}", config.chain_id, config.ws_url);
        
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
                // Connect function
                || {
                    let url = url.clone();
                    async move {
                        connect_async(&url).await
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
    fn parse_websocket_message(json: serde_json::Value, sequence_number: u64, chain_id: ChainId) -> Option<EvmStreamEvent> {
        // Check if this is a subscription notification
        let params = json.get("params")?;
        let subscription = params.get("subscription")?.as_str()?;
        let result = params.get("result")?;
        
        // Determine event type based on the result structure
        let (event_type, block_number, transaction_hash) = if result.get("number").is_some() {
            // New block
            let block_num = result.get("number")?
                .as_str()?
                .strip_prefix("0x")?;
            let block_number = u64::from_str_radix(block_num, 16).ok()?;
            
            (EvmEventType::NewBlock, Some(block_number), None)
        } else if result.get("transactionHash").is_some() {
            // Contract event
            let tx_hash = result.get("transactionHash")?.as_str()?.to_string();
            let block_num = result.get("blockNumber")?
                .as_str()?
                .strip_prefix("0x")?;
            let block_number = u64::from_str_radix(block_num, 16).ok()?;
            
            (EvmEventType::ContractEvent, Some(block_number), Some(tx_hash))
        } else if result.is_string() {
            // Pending transaction (just a hash)
            let tx_hash = result.as_str()?.to_string();
            
            (EvmEventType::PendingTransaction, None, Some(tx_hash))
        } else {
            return None;
        };
        
        // Create stream metadata
        let stream_meta = StreamMetadata {
            stream_source: format!("evm-ws-{}", chain_id),
            received_at: SystemTime::now(),
            sequence_number: Some(sequence_number),
            custom_data: Some(json.clone()),
        };
        
        // Create unique ID
        let id = transaction_hash.as_ref()
            .unwrap_or(&subscription.to_string())
            .clone();
        
        Some(EvmStreamEvent {
            id,
            event_type,
            stream_meta,
            chain_id,
            block_number,
            transaction_hash,
            data: json,
        })
    }
}