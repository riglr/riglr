use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::SystemTime;
use tokio::sync::{broadcast, RwLock};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use serde::{Deserialize, Serialize};
use tracing::{info, warn, error, debug};
use futures::{StreamExt, SinkExt};

use riglr_solana_events::{UnifiedEvent, EventType, ProtocolType, MultiEventParser};
use crate::core::{StreamMetadata, DynamicStreamedEvent, IntoDynamicStreamedEvent};
use crate::core::{Stream, StreamError, StreamEvent, StreamHealth};

/// Solana Geyser stream implementation using WebSocket
pub struct SolanaGeyserStream {
    /// Stream configuration
    config: GeyserConfig,
    /// Event broadcast channel for protocol-specific events
    event_tx: broadcast::Sender<Arc<DynamicStreamedEvent>>,
    /// Event parser for converting raw data to structured events
    event_parser: MultiEventParser,
    /// Running state
    running: Arc<AtomicBool>,
    /// Health metrics
    health: Arc<RwLock<StreamHealth>>,
    /// Stream name
    name: String,
}

/// Geyser stream configuration
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GeyserConfig {
    /// WebSocket endpoint URL (e.g., wss://api.mainnet-beta.solana.com)
    pub ws_url: String,
    /// Authentication token (if required)
    pub auth_token: Option<String>,
    /// Programs to monitor
    pub program_ids: Vec<String>,
    /// Buffer size for event channel
    pub buffer_size: usize,
}

impl Default for GeyserConfig {
    fn default() -> Self {
        Self {
            ws_url: String::from("wss://api.mainnet-beta.solana.com"),
            auth_token: None,
            program_ids: Vec::new(),
            buffer_size: 10000,
        }
    }
}

/// Type alias for streamed Solana events using protocol-specific parsing
pub type SolanaStreamEvent = DynamicStreamedEvent;

/// Simple transaction event for fallback cases
#[derive(Debug, Clone)]
pub struct TransactionEvent {
    pub signature: String,
    pub slot: u64,
}

impl UnifiedEvent for TransactionEvent {
    fn id(&self) -> &str { &self.signature }
    fn event_type(&self) -> EventType { EventType::Transaction }
    fn signature(&self) -> &str { &self.signature }
    fn slot(&self) -> u64 { self.slot }
    fn program_received_time_ms(&self) -> i64 { 0 }
    fn program_handle_time_consuming_ms(&self) -> i64 { 0 }
    fn set_program_handle_time_consuming_ms(&mut self, _: i64) {}
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
    fn clone_boxed(&self) -> Box<dyn UnifiedEvent> { Box::new(self.clone()) }
    fn set_transfer_data(&mut self, _: Vec<riglr_solana_events::TransferData>, _: Option<riglr_solana_events::SwapData>) {}
    fn index(&self) -> String { "0".to_string() }
    fn protocol_type(&self) -> ProtocolType { ProtocolType::Other("Solana".to_string()) }
}

#[async_trait::async_trait]
impl Stream for SolanaGeyserStream {
    type Event = DynamicStreamedEvent;
    type Config = GeyserConfig;
    
    async fn start(&mut self, config: Self::Config) -> Result<(), StreamError> {
        if self.running.load(Ordering::Relaxed) {
            return Err(StreamError::AlreadyRunning { 
                name: self.name.clone() 
            });
        }
        
        info!("Starting Solana Geyser stream: {}", config.ws_url);
        
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
        
        info!("Stopping Solana Geyser stream");
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

impl SolanaGeyserStream {
    /// Create a new Solana Geyser stream
    pub fn new(name: impl Into<String>) -> Self {
        let (event_tx, _) = broadcast::channel(10000);
        
        Self {
            config: GeyserConfig::default(),
            event_tx,
            event_parser: MultiEventParser::new(),
            running: Arc::new(AtomicBool::new(false)),
            health: Arc::new(RwLock::new(StreamHealth::default())),
            name: name.into(),
        }
    }
    
    /// Start the WebSocket connection with automatic reconnection
    async fn start_websocket(&self) -> Result<(), StreamError> {
        let url = self.config.ws_url.clone();
        let program_ids = self.config.program_ids.clone();
        let event_tx = self.event_tx.clone();
        let running = self.running.clone();
        let health = self.health.clone();
        let stream_name = self.name.clone();
        
        tokio::spawn(async move {
            let mut sequence_number = 0u64;
            
            while running.load(Ordering::Relaxed) {
                info!("Attempting to connect to Solana WebSocket: {}", url);
                
                match connect_async(&url).await {
                    Ok((ws_stream, _)) => {
                        let (mut write, mut read) = ws_stream.split();
                        info!("Successfully connected to Solana WebSocket for {}", stream_name);
                        
                        // Update health - connected
                        {
                            let mut h = health.write().await;
                            h.is_connected = true;
                            h.last_event_time = Some(SystemTime::now());
                        }
                        
                        // Message reading loop
                        loop {
                            match tokio::time::timeout(
                                tokio::time::Duration::from_secs(60),
                                read.next()
                            ).await {
                                Ok(Some(Ok(Message::Text(text)))) => {
                                    sequence_number += 1;
                                    
                                    // Parse the message using proper event parsers
                                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                                        if let Some(events) = Self::parse_websocket_message_to_events(json, sequence_number) {
                                            for event in events {
                                                // Send event
                                                if let Err(e) = event_tx.send(Arc::new(event)) {
                                                    warn!("Failed to send event: {}", e);
                                                }
                                            }
                                            
                                            // Update health
                                            let mut h = health.write().await;
                                            h.last_event_time = Some(SystemTime::now());
                                            h.events_processed += 1;
                                        }
                                    }
                                }
                                Ok(Some(Ok(Message::Binary(_)))) => {
                                    // Ignore binary messages
                                }
                                Ok(Some(Ok(Message::Ping(_)))) => {
                                    // Ping messages are handled automatically by tungstenite
                                }
                                Ok(Some(Ok(Message::Pong(_)))) => {
                                    // Pong messages are handled automatically by tungstenite
                                }
                                Ok(Some(Ok(Message::Close(_)))) => {
                                    info!("WebSocket connection closed gracefully for {}", stream_name);
                                    break;
                                }
                                Ok(Some(Ok(Message::Frame(_)))) => {
                                    // Raw frames are not typically handled at this level
                                }
                                Ok(Some(Err(e))) => {
                                    error!("WebSocket error for {}: {}", stream_name, e);
                                    break;
                                }
                                Ok(None) => {
                                    warn!("WebSocket stream ended for {}", stream_name);
                                    break;
                                }
                                Err(_) => {
                                    warn!("WebSocket idle timeout for {}. Reconnecting...", stream_name);
                                    break;
                                }
                            }
                        }
                        
                        // Connection lost - will retry
                        let mut h = health.write().await;
                        h.is_connected = false;
                        h.error_count += 1;
                    }
                    Err(e) => {
                        error!("Failed to connect: {}", e);
                        let mut h = health.write().await;
                        h.is_connected = false;
                        h.error_count += 1;
                    }
                }
                
                // Wait before retry
                tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
            }
            
            info!("WebSocket handler exiting for {}", stream_name);
        });
        
        Ok(())
    }
    
    /// Parse a WebSocket message into structured events using riglr-solana-events parsers
    fn parse_websocket_message_to_events(
        json: serde_json::Value, 
        sequence_number: u64
    ) -> Option<Vec<DynamicStreamedEvent>> {
        // Extract basic transaction data
        let signature = json.get("params")?.get("result")?.get("signature")?.as_str()?;
        let slot = json.get("params")?.get("result")?.get("slot")?.as_u64()?.max(1);
        
        // Create stream metadata
        let stream_metadata = StreamMetadata {
            stream_source: "solana-ws".to_string(),
            received_at: SystemTime::now(),
            sequence_number: Some(sequence_number),
            custom_data: Some(json.clone()),
        };
        
        // Try to parse transaction data if available
        if let Some(_transaction_data) = json.get("params")?.get("result")?.get("transaction") {
            // TODO: Parse actual transaction data into structured events
            // For now, create a basic transaction event as a fallback
            
            // Create a simple transaction event as fallback
            let fallback_event = Box::new(TransactionEvent {
                signature: signature.to_string(),
                slot,
            }) as Box<dyn UnifiedEvent>;
            
            let streamed_event = fallback_event.with_stream_metadata(stream_metadata);
            return Some(vec![streamed_event]);
        }
        
        None
    }
}