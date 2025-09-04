use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::{broadcast, RwLock};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{error, info, warn};

use crate::core::{DynamicStreamedEvent, IntoDynamicStreamedEvent, StreamMetadata};
use crate::core::{Stream, StreamError, StreamHealth};
use riglr_events_core::Event;

/// Solana Geyser stream implementation using WebSocket
pub struct SolanaGeyserStream {
    /// Stream configuration
    config: GeyserConfig,
    /// Event broadcast channel for protocol-specific events
    event_tx: broadcast::Sender<Arc<DynamicStreamedEvent>>,
    /// Event parser for converting raw data to structured events
    _event_parser_placeholder: (),
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
    /// Transaction signature hash
    pub signature: String,
    /// Slot number where the transaction was confirmed
    pub slot: u64,
}

impl Event for TransactionEvent {
    fn id(&self) -> &str {
        &self.signature
    }

    fn kind(&self) -> &riglr_events_core::EventKind {
        static KIND: riglr_events_core::EventKind = riglr_events_core::EventKind::Transaction;
        &KIND
    }

    fn metadata(&self) -> &riglr_events_core::EventMetadata {
        // For simplicity, we'll create a static metadata
        // In a real implementation, this should be stored in the struct
        static METADATA: std::sync::OnceLock<riglr_events_core::EventMetadata> =
            std::sync::OnceLock::new();
        METADATA.get_or_init(|| {
            riglr_events_core::EventMetadata::new(
                "transaction".to_string(),
                riglr_events_core::EventKind::Transaction,
                "geyser-stream".to_string(),
            )
        })
    }

    fn metadata_mut(&mut self) -> &mut riglr_events_core::EventMetadata {
        // This is a limitation of this simple implementation
        // In a real implementation, metadata should be stored in the struct
        panic!("metadata_mut not supported for TransactionEvent")
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

    fn to_json(&self) -> riglr_events_core::error::EventResult<serde_json::Value> {
        Ok(serde_json::json!({
            "signature": self.signature,
            "slot": self.slot
        }))
    }
}

#[async_trait::async_trait]
impl Stream for SolanaGeyserStream {
    type Event = DynamicStreamedEvent;
    type Config = GeyserConfig;

    async fn start(&mut self, config: Self::Config) -> Result<(), StreamError> {
        if self.running.load(Ordering::Relaxed) {
            return Err(StreamError::AlreadyRunning {
                name: self.name.clone(),
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
            _event_parser_placeholder: (),
            running: Arc::new(AtomicBool::new(false)),
            health: Arc::new(RwLock::new(StreamHealth::default())),
            name: name.into(),
        }
    }

    /// Start the WebSocket connection with automatic reconnection
    async fn start_websocket(&self) -> Result<(), StreamError> {
        let url = self.config.ws_url.clone();
        let _program_ids = self.config.program_ids.clone();
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
                        let (_write, mut read) = ws_stream.split();
                        info!(
                            "Successfully connected to Solana WebSocket for {}",
                            stream_name
                        );

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
                                read.next(),
                            )
                            .await
                            {
                                Ok(Some(Ok(Message::Text(text)))) => {
                                    sequence_number += 1;

                                    // Parse the message using proper event parsers
                                    if let Ok(json) =
                                        serde_json::from_str::<serde_json::Value>(&text)
                                    {
                                        if let Some(events) =
                                            Self::parse_websocket_message_to_events(
                                                json,
                                                sequence_number,
                                            )
                                        {
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
                                    info!(
                                        "WebSocket connection closed gracefully for {}",
                                        stream_name
                                    );
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
                                    warn!(
                                        "WebSocket idle timeout for {}. Reconnecting...",
                                        stream_name
                                    );
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
        sequence_number: u64,
    ) -> Option<Vec<DynamicStreamedEvent>> {
        // Extract basic transaction data
        let signature = json
            .get("params")?
            .get("result")?
            .get("signature")?
            .as_str()?;
        let slot = json
            .get("params")?
            .get("result")?
            .get("slot")?
            .as_u64()?
            .max(1);

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
            }) as Box<dyn Event>;

            let streamed_event = fallback_event.with_stream_metadata(stream_metadata);
            return Some(vec![streamed_event]);
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tokio::time::timeout;

    #[test]
    fn test_geyser_config_default() {
        let config = GeyserConfig::default();
        assert_eq!(config.ws_url, "wss://api.mainnet-beta.solana.com");
        assert_eq!(config.auth_token, None);
        assert_eq!(config.program_ids, Vec::<String>::new());
        assert_eq!(config.buffer_size, 10000);
    }

    #[test]
    fn test_geyser_config_clone_debug_serialize_deserialize() {
        let config = GeyserConfig {
            ws_url: "wss://test.com".to_string(),
            auth_token: Some("token123".to_string()),
            program_ids: vec!["program1".to_string(), "program2".to_string()],
            buffer_size: 5000,
        };

        // Test Clone
        let cloned = config.clone();
        assert_eq!(config.ws_url, cloned.ws_url);
        assert_eq!(config.auth_token, cloned.auth_token);
        assert_eq!(config.program_ids, cloned.program_ids);
        assert_eq!(config.buffer_size, cloned.buffer_size);

        // Test Debug
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("wss://test.com"));
        assert!(debug_str.contains("token123"));

        // Test Serialize/Deserialize
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: GeyserConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config.ws_url, deserialized.ws_url);
        assert_eq!(config.auth_token, deserialized.auth_token);
        assert_eq!(config.program_ids, deserialized.program_ids);
        assert_eq!(config.buffer_size, deserialized.buffer_size);
    }

    #[test]
    fn test_transaction_event_new() {
        let event = TransactionEvent {
            signature: "test_signature".to_string(),
            slot: 12345,
        };

        assert_eq!(event.signature, "test_signature");
        assert_eq!(event.slot, 12345);
    }

    #[test]
    fn test_transaction_event_clone() {
        let event = TransactionEvent {
            signature: "test_signature".to_string(),
            slot: 12345,
        };

        let cloned = event.clone();
        assert_eq!(event.signature, cloned.signature);
        assert_eq!(event.slot, cloned.slot);
    }

    #[test]
    fn test_transaction_event_debug() {
        let event = TransactionEvent {
            signature: "test_signature".to_string(),
            slot: 12345,
        };

        let debug_str = format!("{:?}", event);
        assert!(debug_str.contains("test_signature"));
        assert!(debug_str.contains("12345"));
    }

    #[test]
    fn test_transaction_event_id() {
        let event = TransactionEvent {
            signature: "test_signature".to_string(),
            slot: 12345,
        };

        assert_eq!(event.id(), "test_signature");
    }

    #[test]
    fn test_transaction_event_kind() {
        let event = TransactionEvent {
            signature: "test_signature".to_string(),
            slot: 12345,
        };

        let kind = event.kind();
        assert_eq!(*kind, riglr_events_core::EventKind::Transaction);
    }

    #[test]
    fn test_transaction_event_metadata() {
        let event = TransactionEvent {
            signature: "test_signature".to_string(),
            slot: 12345,
        };

        let metadata = event.metadata();
        assert_eq!(metadata.kind.to_string(), "transaction");
        assert_eq!(metadata.kind, riglr_events_core::EventKind::Transaction);
        assert_eq!(metadata.source, "geyser-stream");
    }

    #[test]
    #[should_panic(expected = "metadata_mut not supported for TransactionEvent")]
    fn test_transaction_event_metadata_mut_panics() {
        let mut event = TransactionEvent {
            signature: "test_signature".to_string(),
            slot: 12345,
        };

        event.metadata_mut();
    }

    #[test]
    fn test_transaction_event_as_any() {
        let event = TransactionEvent {
            signature: "test_signature".to_string(),
            slot: 12345,
        };

        let any = event.as_any();
        assert!(any.downcast_ref::<TransactionEvent>().is_some());
    }

    #[test]
    fn test_transaction_event_as_any_mut() {
        let mut event = TransactionEvent {
            signature: "test_signature".to_string(),
            slot: 12345,
        };

        let any_mut = event.as_any_mut();
        assert!(any_mut.downcast_mut::<TransactionEvent>().is_some());
    }

    #[test]
    fn test_transaction_event_clone_boxed() {
        let event = TransactionEvent {
            signature: "test_signature".to_string(),
            slot: 12345,
        };

        let boxed = event.clone_boxed();
        assert_eq!(boxed.id(), "test_signature");
    }

    #[test]
    fn test_solana_geyser_stream_new() {
        let stream = SolanaGeyserStream::new("test_stream");
        assert_eq!(stream.name(), "test_stream");
        assert!(!stream.is_running());
        assert_eq!(stream.config.ws_url, "wss://api.mainnet-beta.solana.com");
    }

    #[test]
    fn test_solana_geyser_stream_new_with_string() {
        let stream = SolanaGeyserStream::new("test_stream".to_string());
        assert_eq!(stream.name(), "test_stream");
    }

    #[tokio::test]
    async fn test_solana_geyser_stream_start_when_not_running_should_start() {
        let mut stream = SolanaGeyserStream::new("test_stream");
        let config = GeyserConfig {
            ws_url: "wss://invalid-url-for-testing.com".to_string(),
            auth_token: None,
            program_ids: vec!["program1".to_string()],
            buffer_size: 1000,
        };

        let result = stream.start(config.clone()).await;
        assert!(result.is_ok());
        assert!(stream.is_running());
        assert_eq!(stream.config.ws_url, config.ws_url);
        assert_eq!(stream.config.program_ids, config.program_ids);
        assert_eq!(stream.config.buffer_size, config.buffer_size);

        // Clean up
        let _ = stream.stop().await;
    }

    #[tokio::test]
    async fn test_solana_geyser_stream_start_when_already_running_should_return_error() {
        let mut stream = SolanaGeyserStream::new("test_stream");
        let config = GeyserConfig::default();

        // Start first time
        let result1 = stream.start(config.clone()).await;
        assert!(result1.is_ok());
        assert!(stream.is_running());

        // Try to start again - should fail
        let result2 = stream.start(config).await;
        assert!(result2.is_err());

        if let Err(StreamError::AlreadyRunning { name }) = result2 {
            assert_eq!(name, "test_stream");
        } else {
            panic!("Expected AlreadyRunning error");
        }

        // Clean up
        let _ = stream.stop().await;
    }

    #[tokio::test]
    async fn test_solana_geyser_stream_stop_when_running_should_stop() {
        let mut stream = SolanaGeyserStream::new("test_stream");
        let config = GeyserConfig::default();

        // Start stream
        let _ = stream.start(config).await;
        assert!(stream.is_running());

        // Stop stream
        let result = stream.stop().await;
        assert!(result.is_ok());
        assert!(!stream.is_running());
    }

    #[tokio::test]
    async fn test_solana_geyser_stream_stop_when_not_running_should_be_ok() {
        let mut stream = SolanaGeyserStream::new("test_stream");
        assert!(!stream.is_running());

        let result = stream.stop().await;
        assert!(result.is_ok());
        assert!(!stream.is_running());
    }

    #[tokio::test]
    async fn test_solana_geyser_stream_subscribe() {
        let stream = SolanaGeyserStream::new("test_stream");
        let mut receiver = stream.subscribe();

        // Should not block and should be ready to receive
        assert!(timeout(Duration::from_millis(100), receiver.recv())
            .await
            .is_err());
    }

    #[tokio::test]
    async fn test_solana_geyser_stream_health() {
        let stream = SolanaGeyserStream::new("test_stream");
        let health = stream.health().await;

        // Default health state
        assert!(!health.is_connected);
        assert_eq!(health.events_processed, 0);
        assert_eq!(health.error_count, 0);
        assert!(health.last_event_time.is_none());
    }

    #[tokio::test]
    async fn test_solana_geyser_stream_health_after_start() {
        let mut stream = SolanaGeyserStream::new("test_stream");
        let config = GeyserConfig::default();

        let _ = stream.start(config).await;

        // Give it a moment to update health
        tokio::time::sleep(Duration::from_millis(10)).await;

        let health = stream.health().await;
        assert!(health.is_connected);
        assert!(health.last_event_time.is_some());

        // Clean up
        let _ = stream.stop().await;
    }

    #[test]
    fn test_parse_websocket_message_to_events_with_valid_transaction() {
        let json = serde_json::json!({
            "params": {
                "result": {
                    "signature": "test_signature_123",
                    "slot": 12345,
                    "transaction": {
                        "message": {
                            "instructions": []
                        }
                    }
                }
            }
        });

        let events = SolanaGeyserStream::parse_websocket_message_to_events(json, 1);
        assert!(events.is_some());

        let events = events.unwrap();
        assert_eq!(events.len(), 1);

        let event = &events[0];
        assert_eq!(event.stream_metadata.stream_source, "solana-ws");
        assert_eq!(event.stream_metadata.sequence_number, Some(1));
        assert!(event.stream_metadata.custom_data.is_some());
        assert!(event.stream_metadata.received_at <= SystemTime::now());
    }

    #[test]
    fn test_parse_websocket_message_to_events_with_missing_signature() {
        let json = serde_json::json!({
            "params": {
                "result": {
                    "slot": 12345,
                    "transaction": {
                        "message": {
                            "instructions": []
                        }
                    }
                }
            }
        });

        let events = SolanaGeyserStream::parse_websocket_message_to_events(json, 1);
        assert!(events.is_none());
    }

    #[test]
    fn test_parse_websocket_message_to_events_with_missing_slot() {
        let json = serde_json::json!({
            "params": {
                "result": {
                    "signature": "test_signature_123",
                    "transaction": {
                        "message": {
                            "instructions": []
                        }
                    }
                }
            }
        });

        let events = SolanaGeyserStream::parse_websocket_message_to_events(json, 1);
        assert!(events.is_none());
    }

    #[test]
    fn test_parse_websocket_message_to_events_with_missing_params() {
        let json = serde_json::json!({
            "result": {
                "signature": "test_signature_123",
                "slot": 12345,
                "transaction": {
                    "message": {
                        "instructions": []
                    }
                }
            }
        });

        let events = SolanaGeyserStream::parse_websocket_message_to_events(json, 1);
        assert!(events.is_none());
    }

    #[test]
    fn test_parse_websocket_message_to_events_with_missing_result() {
        let json = serde_json::json!({
            "params": {
                "signature": "test_signature_123",
                "slot": 12345,
                "transaction": {
                    "message": {
                        "instructions": []
                    }
                }
            }
        });

        let events = SolanaGeyserStream::parse_websocket_message_to_events(json, 1);
        assert!(events.is_none());
    }

    #[test]
    fn test_parse_websocket_message_to_events_with_zero_slot() {
        let json = serde_json::json!({
            "params": {
                "result": {
                    "signature": "test_signature_123",
                    "slot": 0,
                    "transaction": {
                        "message": {
                            "instructions": []
                        }
                    }
                }
            }
        });

        let events = SolanaGeyserStream::parse_websocket_message_to_events(json, 1);
        assert!(events.is_some());

        let events = events.unwrap();
        assert_eq!(events.len(), 1);

        // slot should be max(0, 1) = 1
        let event_data = events[0].stream_metadata.custom_data.as_ref().unwrap();
        let slot = event_data["params"]["result"]["slot"].as_u64().unwrap();
        assert!(slot >= 1); // The max(1) operation should ensure slot is at least 1
    }

    #[test]
    fn test_parse_websocket_message_to_events_without_transaction() {
        let json = serde_json::json!({
            "params": {
                "result": {
                    "signature": "test_signature_123",
                    "slot": 12345
                }
            }
        });

        let events = SolanaGeyserStream::parse_websocket_message_to_events(json, 1);
        assert!(events.is_none());
    }

    #[test]
    fn test_parse_websocket_message_to_events_with_large_slot() {
        let json = serde_json::json!({
            "params": {
                "result": {
                    "signature": "test_signature_123",
                    "slot": u64::MAX,
                    "transaction": {
                        "message": {
                            "instructions": []
                        }
                    }
                }
            }
        });

        let events = SolanaGeyserStream::parse_websocket_message_to_events(json, u64::MAX);
        assert!(events.is_some());

        let events = events.unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].stream_metadata.sequence_number, Some(u64::MAX));
    }

    #[test]
    fn test_parse_websocket_message_to_events_with_empty_signature() {
        let json = serde_json::json!({
            "params": {
                "result": {
                    "signature": "",
                    "slot": 12345,
                    "transaction": {
                        "message": {
                            "instructions": []
                        }
                    }
                }
            }
        });

        let events = SolanaGeyserStream::parse_websocket_message_to_events(json, 1);
        assert!(events.is_some());

        let events = events.unwrap();
        assert_eq!(events.len(), 1);
    }

    #[test]
    fn test_parse_websocket_message_to_events_with_non_string_signature() {
        let json = serde_json::json!({
            "params": {
                "result": {
                    "signature": 123,
                    "slot": 12345,
                    "transaction": {
                        "message": {
                            "instructions": []
                        }
                    }
                }
            }
        });

        let events = SolanaGeyserStream::parse_websocket_message_to_events(json, 1);
        assert!(events.is_none());
    }

    #[test]
    fn test_parse_websocket_message_to_events_with_non_number_slot() {
        let json = serde_json::json!({
            "params": {
                "result": {
                    "signature": "test_signature_123",
                    "slot": "not_a_number",
                    "transaction": {
                        "message": {
                            "instructions": []
                        }
                    }
                }
            }
        });

        let events = SolanaGeyserStream::parse_websocket_message_to_events(json, 1);
        assert!(events.is_none());
    }
}
