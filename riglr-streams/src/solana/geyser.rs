use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::{broadcast, RwLock};
use tokio_tungstenite::connect_async;
use tracing::info;

use crate::core::{DynamicStreamedEvent, IntoDynamicStreamedEvent, StreamMetadata};
use crate::core::{MetricsCollector, Stream, StreamError, StreamHealth};
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
    /// Metrics collector for observability
    metrics_collector: Option<Arc<MetricsCollector>>,
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

    fn metadata_mut(
        &mut self,
    ) -> riglr_events_core::error::EventResult<&mut riglr_events_core::EventMetadata> {
        // TransactionEvent doesn't store mutable metadata, so we return an error
        Err(riglr_events_core::error::EventError::Generic {
            message: "TransactionEvent: mutable metadata access not supported - event is immutable"
                .to_string(),
        })
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

    fn subscribe(&self) -> broadcast::Receiver<Arc<DynamicStreamedEvent>> {
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
        Self::with_metrics(name, None)
    }

    /// Create a new Solana Geyser stream with metrics collector
    pub fn with_metrics(
        name: impl Into<String>,
        metrics_collector: Option<Arc<MetricsCollector>>,
    ) -> Self {
        let (event_tx, _) = broadcast::channel(10000);

        Self {
            config: GeyserConfig::default(),
            event_tx,
            _event_parser_placeholder: (),
            running: Arc::new(AtomicBool::new(false)),
            health: Arc::new(RwLock::new(StreamHealth::default())),
            name: name.into(),
            metrics_collector,
        }
    }

    /// Start the WebSocket connection with automatic reconnection
    async fn start_websocket(&self) -> Result<(), StreamError> {
        let url = self.config.ws_url.clone();
        let metrics = self.metrics_collector.clone();
        let running = self.running.clone();
        let health = self.health.clone();
        let stream_name = self.name.clone();

        // Create mpsc channel for internal event handling
        let (internal_tx, mut internal_rx) =
            tokio::sync::mpsc::unbounded_channel::<Arc<DynamicStreamedEvent>>();
        let event_tx = self.event_tx.clone();

        // Bridge internal events to broadcast channel
        let stream_name_clone = stream_name.clone();
        let metrics_clone = metrics.clone();
        tokio::spawn(async move {
            while let Some(event) = internal_rx.recv().await {
                if let Some(ref metrics) = metrics_clone {
                    metrics
                        .record_stream_event(&stream_name_clone, 0.0, 0)
                        .await;
                }
                let _ = event_tx.send(event);
            }
        });

        let internal_tx_for_connector = internal_tx;
        tokio::spawn(async move {
            let connector = crate::core::ResilientWebSocketBuilder::new(
                stream_name,
                running,
                health,
                internal_tx_for_connector,
                move || {
                    let url = url.clone();
                    Box::pin(async move {
                        connect_async(&url).await.map(|(ws, _)| ws).map_err(|e| {
                            Box::new(std::io::Error::new(
                                std::io::ErrorKind::ConnectionRefused,
                                format!("Failed to connect to Solana Geyser: {}", e),
                            ))
                                as Box<dyn std::error::Error + Send + Sync>
                        })
                    })
                },
                |_write| {
                    Box::pin(async move { Ok::<(), Box<dyn std::error::Error + Send + Sync>>(()) })
                },
                Self::parse_message_to_event,
            )
            .with_metrics(metrics.unwrap_or_else(|| Arc::new(MetricsCollector::default())))
            .build();

            connector.run().await;
        });

        Ok(())
    }

    /// Parse a WebSocket message into an event
    fn parse_message_to_event(data: String, sequence_number: u64) -> Option<DynamicStreamedEvent> {
        // Parse the message using proper event parsers
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&data) {
            if let Some(events) = Self::parse_websocket_message_to_events(json, sequence_number) {
                return events.into_iter().next(); // Return first event
            }
        }
        None
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
    fn test_transaction_event_metadata_mut_returns_error() {
        let mut event = TransactionEvent {
            signature: "test_signature".to_string(),
            slot: 12345,
        };

        let result = event.metadata_mut();
        assert!(result.is_err());
        if let Err(riglr_events_core::error::EventError::Generic { message }) = result {
            assert!(message.contains("mutable metadata access not supported"));
        } else {
            panic!("Expected Generic error");
        }
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
