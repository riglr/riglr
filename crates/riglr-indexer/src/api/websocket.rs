//! WebSocket streaming API

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::Response,
    routing::get,
    Router,
};
use futures::{
    sink::SinkExt,
    stream::{SplitSink, SplitStream, StreamExt},
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt, sync::Arc, time::Duration};
use tokio::{sync::broadcast, time};
use tracing::{debug, error, info, warn};

use crate::core::ServiceContext;
use crate::error::IndexerResult;
use crate::storage::StoredEvent;

/// WebSocket streaming handler
pub struct WebSocketStreamer {
    #[expect(dead_code)] // Used in future integration with the service context
    context: Arc<ServiceContext>,
    event_broadcaster: broadcast::Sender<StreamMessage>,
}

impl fmt::Debug for WebSocketStreamer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WebSocketStreamer")
            .field("context", &"ServiceContext { .. }")
            .field("event_broadcaster", &"broadcast::Sender { .. }")
            .finish()
    }
}

/// Message types for WebSocket streaming
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum StreamMessage {
    /// New event notification
    Event {
        /// The stored event data
        event: StoredEvent,
    },
    /// Error notification
    Error {
        /// Error message
        message: String,
        /// Optional error code
        code: Option<String>,
    },
    /// Health status update
    Health {
        /// Overall health status
        healthy: bool,
        /// Individual component health statuses
        components: HashMap<String, bool>,
    },
    /// Metrics update
    Metrics {
        /// Metrics data as JSON value
        metrics: serde_json::Value,
    },
    /// Heartbeat/keepalive
    Ping,
    /// Service status update
    Status {
        /// Current service state
        state: String,
        /// Status message
        message: String,
    },
    /// Pong response
    Pong,
}

/// WebSocket subscription request
#[derive(Debug, Deserialize)]
pub struct SubscriptionRequest {
    /// Types of messages to subscribe to
    pub message_types: Vec<String>,
    /// Event filters for event messages
    pub event_filters: Option<EventSubscriptionFilter>,
}

/// Event subscription filters
#[derive(Debug, Deserialize)]
pub struct EventSubscriptionFilter {
    /// Event types to include
    pub event_types: Option<Vec<String>>,
    /// Sources to include
    pub sources: Option<Vec<String>>,
    /// Minimum block height
    pub min_block: Option<u64>,
}

impl WebSocketStreamer {
    /// Broadcast new event
    pub fn broadcast_event(&self, event: StoredEvent) {
        self.broadcast_message(StreamMessage::Event { event });
    }

    /// Broadcast health update
    pub fn broadcast_health(&self, healthy: bool, components: HashMap<String, bool>) {
        self.broadcast_message(StreamMessage::Health {
            healthy,
            components,
        });
    }

    /// Broadcast a message to all connected clients
    pub fn broadcast_message(&self, message: StreamMessage) {
        if let Err(e) = self.event_broadcaster.send(message) {
            // Only log if there are receivers (ignore if no one is listening)
            if self.event_broadcaster.receiver_count() > 0 {
                warn!("Failed to broadcast message: {}", e);
            }
        }
    }

    /// Create WebSocket router
    pub fn create_router(&self) -> Router<Arc<ServiceContext>> {
        Router::new().route("/api/v1/ws", get(ws_handler))
    }

    /// Create a new WebSocket streamer
    ///
    /// # Errors
    ///
    /// Currently always succeeds, but marked as Result for future extensibility.
    pub fn new(context: Arc<ServiceContext>) -> IndexerResult<Self> {
        let (event_broadcaster, _) = broadcast::channel(1000);

        Ok(Self {
            context,
            event_broadcaster,
        })
    }

    /// Broadcast metrics update
    pub fn broadcast_metrics(&self, metrics: serde_json::Value) {
        self.broadcast_message(StreamMessage::Metrics { metrics });
    }

    /// Get receiver for messages
    #[must_use]
    pub fn subscribe(&self) -> broadcast::Receiver<StreamMessage> {
        self.event_broadcaster.subscribe()
    }
}

/// WebSocket upgrade handler
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(context): State<Arc<ServiceContext>>,
) -> Response {
    info!("New WebSocket connection request");
    ws.on_upgrade(|socket| handle_websocket(socket, context))
}

/// Handle individual WebSocket connection
#[expect(clippy::cognitive_complexity)]
async fn handle_websocket(socket: WebSocket, context: Arc<ServiceContext>) {
    let (sender, receiver) = socket.split();
    let client_id = generate_client_id();

    info!("WebSocket client {} connected", client_id);

    let message_rx = context.shutdown_receiver();

    let send_task = tokio::spawn(spawn_send_task(sender, message_rx, client_id.clone()));
    let recv_task = tokio::spawn(spawn_receive_task(receiver, client_id.clone()));

    // Wait for either task to complete
    tokio::select! {
        _ = send_task => {
            debug!("Send task completed for client {}", client_id);
        }
        _ = recv_task => {
            debug!("Receive task completed for client {}", client_id);
        }
    }

    info!("WebSocket client {} disconnected", client_id);
}

/// Spawn the WebSocket send task
async fn spawn_send_task(
    mut sender: SplitSink<WebSocket, Message>,
    mut message_rx: broadcast::Receiver<()>,
    client_id: String,
) {
    let mut heartbeat_interval = time::interval(Duration::from_secs(30));

    loop {
        tokio::select! {
            _ = heartbeat_interval.tick() => {
                if send_ping_message(&mut sender, &client_id).await.is_err() {
                    break;
                }
            }

            _ = message_rx.recv() => {
                debug!("Shutdown signal received, closing WebSocket");
                break;
            }
        }
    }

    debug!("Send task for client {} ended", client_id);
}

/// Send a ping message
async fn send_ping_message(
    sender: &mut SplitSink<WebSocket, Message>,
    client_id: &str,
) -> Result<(), ()> {
    let ping_msg = StreamMessage::Ping;
    if let Ok(json) = serde_json::to_string(&ping_msg) {
        if sender.send(Message::Text(json.into())).await.is_err() {
            debug!("Client {} disconnected (send failed)", client_id);
            return Err(());
        }
    }
    Ok(())
}

/// Spawn the WebSocket receive task
async fn spawn_receive_task(mut receiver: SplitStream<WebSocket>, client_id: String) {
    loop {
        let msg_option = receiver.next().await;
        let Some(msg) = msg_option else {
            break;
        };

        if handle_websocket_message(msg, &client_id).is_err() {
            break;
        }
    }

    debug!("Receive task for client {} ended", client_id);
}

/// Handle different message types for WebSocket
#[expect(clippy::cognitive_complexity)]
fn handle_message_by_type(msg: Message, client_id: &str) -> Result<Option<String>, ()> {
    match msg {
        Message::Text(text) => Ok(Some(text.to_string())),
        Message::Binary(data) => {
            debug!(
                "Received binary message from {}: {} bytes",
                client_id,
                data.len()
            );
            Ok(None)
        }
        Message::Close(_) => {
            info!("Client {} closed connection", client_id);
            Err(())
        }
        Message::Ping(_) => {
            debug!("Received ping from client {}", client_id);
            // Axum automatically responds to pings
            Ok(None)
        }
        Message::Pong(_) => {
            debug!("Received pong from client {}", client_id);
            Ok(None)
        }
    }
}

/// Handle a single WebSocket message
fn handle_websocket_message(msg: Result<Message, axum::Error>, client_id: &str) -> Result<(), ()> {
    match msg {
        Ok(message) => {
            if let Some(text) = handle_message_by_type(message, client_id)? {
                handle_text_message(&text, client_id);
            }
            Ok(())
        }
        Err(e) => {
            error!("WebSocket error for client {}: {}", client_id, e);
            Err(())
        }
    }
}

/// Handle a text message from the client
fn handle_text_message(text: &str, client_id: &str) {
    debug!("Received text message from {}: {}", client_id, text);

    // Try to parse as subscription request
    match serde_json::from_str::<SubscriptionRequest>(text) {
        Ok(sub_req) => {
            info!(
                "Client {} subscribed to: {:?}",
                client_id, sub_req.message_types
            );
            // In a real implementation, you'd store the subscription preferences
        }
        Err(_) => {
            handle_generic_message(text, client_id);
        }
    }
}

/// Handle a generic stream message
fn handle_generic_message(text: &str, client_id: &str) {
    match serde_json::from_str::<StreamMessage>(text) {
        Ok(StreamMessage::Pong) => {
            debug!("Received pong from client {}", client_id);
        }
        Ok(msg) => {
            debug!("Received message from {}: {:?}", client_id, msg);
        }
        Err(e) => {
            warn!("Invalid message from client {}: {}", client_id, e);
        }
    }
}

/// Generate a unique client ID
fn generate_client_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::float_eq;

    #[tokio::test]
    #[allow(clippy::unwrap_used, clippy::panic)]
    async fn test_stream_message_serialization() {
        let event = StoredEvent {
            id: "test-1".to_string(),
            event_type: "swap".to_string(),
            source: "jupiter".to_string(),
            data: serde_json::json!({"amount": 1000}),
            timestamp: chrono::Utc::now(),
            block_height: Some(12345),
            transaction_hash: Some("abc123".to_string()),
        };

        let message = StreamMessage::Event { event };
        let json = serde_json::to_string(&message).unwrap(); // Test helper

        let deserialized: StreamMessage = serde_json::from_str(&json).unwrap(); // Test helper

        match deserialized {
            StreamMessage::Event { event } => {
                assert_eq!(event.id, "test-1");
                assert_eq!(event.event_type, "swap");
            }
            _ => panic!("Expected Event message"),
        }
    }

    #[test]
    fn test_broadcast_channel_behavior_when_no_receivers_should_not_error() {
        let (event_broadcaster, _) = broadcast::channel::<StreamMessage>(1000);

        let message = StreamMessage::Ping;
        // Should not panic or error when no receivers
        let result = event_broadcaster.send(message);

        // When no receivers, send returns Err but that's expected behavior
        assert!(result.is_err());
        assert_eq!(event_broadcaster.receiver_count(), 0);
    }

    #[test]
    fn test_broadcast_channel_behavior_when_has_receivers_should_send() {
        let (event_broadcaster, _) = broadcast::channel::<StreamMessage>(1000);

        let _receiver = event_broadcaster.subscribe(); // Create a receiver
        let message = StreamMessage::Ping;

        // Should successfully send when there are receivers
        let result = event_broadcaster.send(message);
        assert!(result.is_ok());

        // Verify receiver count is 1
        assert_eq!(event_broadcaster.receiver_count(), 1);
    }

    #[test]
    #[allow(clippy::unwrap_used, clippy::panic)]
    fn test_stream_message_health_serialization_when_valid_should_serialize() {
        let mut components = HashMap::new();
        components.insert("db".to_string(), true);
        components.insert("cache".to_string(), false);

        let message = StreamMessage::Health {
            healthy: false,
            components,
        };

        let json = serde_json::to_string(&message).unwrap(); // Test helper
        let deserialized: StreamMessage = serde_json::from_str(&json).unwrap(); // Test helper

        match deserialized {
            StreamMessage::Health {
                healthy,
                components,
            } => {
                assert!(!healthy);
                assert_eq!(components.len(), 2);
                assert_eq!(components.get("db"), Some(&true));
                assert_eq!(components.get("cache"), Some(&false));
            }
            _ => panic!("Expected Health message"),
        }
    }

    #[test]
    #[allow(clippy::unwrap_used, clippy::panic)]
    fn test_stream_message_metrics_serialization_when_valid_should_serialize() {
        let metrics = serde_json::json!({
            "cpu_usage": 75.5,
            "memory_usage": 1024
        });

        let message = StreamMessage::Metrics { metrics };

        let json = serde_json::to_string(&message).unwrap(); // Test helper
        let deserialized: StreamMessage = serde_json::from_str(&json).unwrap(); // Test helper

        match deserialized {
            StreamMessage::Metrics { metrics } => {
                assert!(float_eq(
                    metrics.get("cpu_usage").unwrap().as_f64().unwrap(),
                    75.5
                )); // Test assertion
                assert_eq!(metrics.get("memory_usage").unwrap().as_i64().unwrap(), 1024);
                // Test assertion
            }
            _ => panic!("Expected Metrics message"),
        }
    }

    #[test]
    #[allow(clippy::unwrap_used, clippy::panic)]
    fn test_stream_message_status_serialization_when_valid_should_serialize() {
        let message = StreamMessage::Status {
            state: "running".to_string(),
            message: "All systems operational".to_string(),
        };

        let json = serde_json::to_string(&message).unwrap(); // Test helper
        let deserialized: StreamMessage = serde_json::from_str(&json).unwrap(); // Test helper

        match deserialized {
            StreamMessage::Status { state, message } => {
                assert_eq!(state, "running");
                assert_eq!(message, "All systems operational");
            }
            _ => panic!("Expected Status message"),
        }
    }

    #[test]
    #[allow(clippy::unwrap_used, clippy::panic)]
    fn test_stream_message_error_serialization_when_valid_should_serialize() {
        let message = StreamMessage::Error {
            message: "Connection failed".to_string(),
            code: Some("E001".to_string()),
        };

        let json = serde_json::to_string(&message).unwrap(); // Test helper
        let deserialized: StreamMessage = serde_json::from_str(&json).unwrap(); // Test helper

        match deserialized {
            StreamMessage::Error { message, code } => {
                assert_eq!(message, "Connection failed");
                assert_eq!(code, Some("E001".to_string()));
            }
            _ => panic!("Expected Error message"),
        }
    }

    #[test]
    #[allow(clippy::unwrap_used, clippy::panic)]
    fn test_stream_message_error_serialization_when_no_code_should_serialize() {
        let message = StreamMessage::Error {
            message: "Unknown error".to_string(),
            code: None,
        };

        let json = serde_json::to_string(&message).unwrap(); // Test helper
        let deserialized: StreamMessage = serde_json::from_str(&json).unwrap(); // Test helper

        match deserialized {
            StreamMessage::Error { message, code } => {
                assert_eq!(message, "Unknown error");
                assert_eq!(code, None);
            }
            _ => panic!("Expected Error message"),
        }
    }

    #[test]
    #[allow(clippy::unwrap_used, clippy::panic)]
    fn test_stream_message_ping_serialization_when_valid_should_serialize() {
        let message = StreamMessage::Ping;

        let json = serde_json::to_string(&message).unwrap(); // Test helper
        let deserialized: StreamMessage = serde_json::from_str(&json).unwrap(); // Test helper

        match deserialized {
            StreamMessage::Ping => {
                // Success
            }
            _ => panic!("Expected Ping message"),
        }
    }

    #[test]
    #[allow(clippy::unwrap_used, clippy::panic)]
    fn test_stream_message_pong_serialization_when_valid_should_serialize() {
        let message = StreamMessage::Pong;

        let json = serde_json::to_string(&message).unwrap(); // Test helper
        let deserialized: StreamMessage = serde_json::from_str(&json).unwrap(); // Test helper

        match deserialized {
            StreamMessage::Pong => {
                // Success
            }
            _ => panic!("Expected Pong message"),
        }
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_subscription_request_deserialization_when_valid_should_deserialize() {
        let json = r#"{
            "message_types": ["event", "health"],
            "event_filters": {
                "event_types": ["swap", "transfer"],
                "sources": ["jupiter", "uniswap"],
                "min_block": 12345
            }
        }"#;

        let request: SubscriptionRequest = serde_json::from_str(json).unwrap(); // Test helper

        assert_eq!(request.message_types, vec!["event", "health"]);
        assert!(request.event_filters.is_some());

        let filters = request.event_filters.unwrap(); // Test assertion
        assert_eq!(
            filters.event_types,
            Some(vec!["swap".to_string(), "transfer".to_string()])
        );
        assert_eq!(
            filters.sources,
            Some(vec!["jupiter".to_string(), "uniswap".to_string()])
        );
        assert_eq!(filters.min_block, Some(12345));
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_subscription_request_deserialization_when_no_filters_should_deserialize() {
        let json = r#"{
            "message_types": ["ping", "pong"]
        }"#;

        let request: SubscriptionRequest = serde_json::from_str(json).unwrap(); // Test helper

        assert_eq!(request.message_types, vec!["ping", "pong"]);
        assert!(request.event_filters.is_none());
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_subscription_request_deserialization_when_empty_message_types_should_deserialize() {
        let json = r#"{
            "message_types": []
        }"#;

        let request: SubscriptionRequest = serde_json::from_str(json).unwrap(); // Test helper

        assert_eq!(request.message_types.len(), 0);
        assert!(request.event_filters.is_none());
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_event_subscription_filter_deserialization_when_partial_fields_should_deserialize() {
        let json = r#"{
            "event_types": ["swap"]
        }"#;

        let filter: EventSubscriptionFilter = serde_json::from_str(json).unwrap(); // Test helper

        assert_eq!(filter.event_types, Some(vec!["swap".to_string()]));
        assert!(filter.sources.is_none());
        assert!(filter.min_block.is_none());
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_event_subscription_filter_deserialization_when_all_none_should_deserialize() {
        let json = r"{}";

        let filter: EventSubscriptionFilter = serde_json::from_str(json).unwrap(); // Test helper

        assert!(filter.event_types.is_none());
        assert!(filter.sources.is_none());
        assert!(filter.min_block.is_none());
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_event_subscription_filter_deserialization_when_empty_arrays_should_deserialize() {
        let json = r#"{
            "event_types": [],
            "sources": [],
            "min_block": 0
        }"#;

        let filter: EventSubscriptionFilter = serde_json::from_str(json).unwrap(); // Test helper

        assert_eq!(filter.event_types, Some(vec![]));
        assert_eq!(filter.sources, Some(vec![]));
        assert_eq!(filter.min_block, Some(0));
    }

    #[test]
    #[allow(clippy::panic)]
    fn test_stream_message_clone_when_called_should_clone() {
        let message = StreamMessage::Ping;
        let cloned = message.clone();

        match (message, cloned) {
            (StreamMessage::Ping, StreamMessage::Ping) => {
                // Success
            }
            _ => panic!("Clone should preserve variant"),
        }
    }

    #[test]
    fn test_stream_message_debug_when_called_should_format() {
        let message = StreamMessage::Error {
            message: "test".to_string(),
            code: None,
        };

        let debug_str = format!("{message:?}");
        assert!(debug_str.contains("Error"));
        assert!(debug_str.contains("test"));
    }

    #[test]
    fn test_subscription_request_debug_when_called_should_format() {
        let request = SubscriptionRequest {
            message_types: vec!["test".to_string()],
            event_filters: None,
        };

        let debug_str = format!("{request:?}");
        assert!(debug_str.contains("SubscriptionRequest"));
        assert!(debug_str.contains("test"));
    }

    #[test]
    fn test_event_subscription_filter_debug_when_called_should_format() {
        let filter = EventSubscriptionFilter {
            event_types: Some(vec!["test".to_string()]),
            sources: None,
            min_block: Some(100),
        };

        let debug_str = format!("{filter:?}");
        assert!(debug_str.contains("EventSubscriptionFilter"));
        assert!(debug_str.contains("test"));
        assert!(debug_str.contains("100"));
    }
}
