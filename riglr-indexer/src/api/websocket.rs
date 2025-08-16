//! WebSocket streaming API

use std::sync::Arc;
use axum::{
    extract::{ws::{Message, WebSocket, WebSocketUpgrade}, State},
    response::Response,
    routing::get,
    Router,
};
use futures::{sink::SinkExt, stream::StreamExt};
use tokio::sync::broadcast;
use tracing::{info, error, debug, warn};
use serde::{Deserialize, Serialize};

use crate::core::ServiceContext;
use crate::error::IndexerResult;
use crate::storage::StoredEvent;

/// WebSocket streaming handler
pub struct WebSocketStreamer {
    #[allow(dead_code)]
    context: Arc<ServiceContext>,
    event_broadcaster: broadcast::Sender<StreamMessage>,
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
    /// Health status update
    Health {
        /// Overall health status
        healthy: bool,
        /// Individual component health statuses
        components: std::collections::HashMap<String, bool>,
    },
    /// Metrics update
    Metrics {
        /// Metrics data as JSON value
        metrics: serde_json::Value,
    },
    /// Service status update
    Status {
        /// Current service state
        state: String,
        /// Status message
        message: String,
    },
    /// Error notification
    Error {
        /// Error message
        message: String,
        /// Optional error code
        code: Option<String>,
    },
    /// Heartbeat/keepalive
    Ping,
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
    /// Create a new WebSocket streamer
    pub fn new(context: Arc<ServiceContext>) -> IndexerResult<Self> {
        let (event_broadcaster, _) = broadcast::channel(1000);
        
        Ok(Self {
            context,
            event_broadcaster,
        })
    }

    /// Create WebSocket router
    pub fn create_router(&self) -> Router<Arc<ServiceContext>> {
        Router::new()
            .route("/api/v1/ws", get(websocket_handler))
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

    /// Broadcast new event
    pub fn broadcast_event(&self, event: StoredEvent) {
        self.broadcast_message(StreamMessage::Event { event });
    }

    /// Broadcast health update
    pub fn broadcast_health(&self, healthy: bool, components: std::collections::HashMap<String, bool>) {
        self.broadcast_message(StreamMessage::Health { healthy, components });
    }

    /// Broadcast metrics update
    pub fn broadcast_metrics(&self, metrics: serde_json::Value) {
        self.broadcast_message(StreamMessage::Metrics { metrics });
    }

    /// Get receiver for messages
    pub fn subscribe(&self) -> broadcast::Receiver<StreamMessage> {
        self.event_broadcaster.subscribe()
    }
}

/// WebSocket upgrade handler
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(context): State<Arc<ServiceContext>>,
) -> Response {
    info!("New WebSocket connection request");
    ws.on_upgrade(|socket| handle_websocket(socket, context))
}

/// Handle individual WebSocket connection
async fn handle_websocket(socket: WebSocket, context: Arc<ServiceContext>) {
    let (mut sender, mut receiver) = socket.split();
    let client_id = uuid::Uuid::new_v4().to_string();
    let client_id_send = client_id.clone();
    let client_id_recv = client_id.clone();
    
    info!("WebSocket client {} connected", client_id);

    // Subscribe to events (this is a simplified approach)
    // In a real implementation, you'd integrate this with the WebSocketStreamer
    let mut message_rx = context.shutdown_receiver();
    
    let send_task = tokio::spawn(async move {
        let mut heartbeat_interval = tokio::time::interval(std::time::Duration::from_secs(30));
        
        loop {
            tokio::select! {
                _ = heartbeat_interval.tick() => {
                    let ping_msg = StreamMessage::Ping;
                    if let Ok(json) = serde_json::to_string(&ping_msg) {
                        if sender.send(Message::Text(json)).await.is_err() {
                            debug!("Client {} disconnected (send failed)", client_id_send);
                            break;
                        }
                    }
                }
                
                _ = message_rx.recv() => {
                    debug!("Shutdown signal received, closing WebSocket");
                    break;
                }
            }
        }
        
        debug!("Send task for client {} ended", client_id_send);
    });

    let recv_task = tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    debug!("Received text message from {}: {}", client_id_recv, text);
                    
                    // Try to parse as subscription request
                    match serde_json::from_str::<SubscriptionRequest>(&text) {
                        Ok(sub_req) => {
                            info!("Client {} subscribed to: {:?}", client_id_recv, sub_req.message_types);
                            // In a real implementation, you'd store the subscription preferences
                        }
                        Err(_) => {
                            // Try to parse as a generic message
                            match serde_json::from_str::<StreamMessage>(&text) {
                                Ok(StreamMessage::Pong) => {
                                    debug!("Received pong from client {}", client_id_recv);
                                }
                                Ok(msg) => {
                                    debug!("Received message from {}: {:?}", client_id_recv, msg);
                                }
                                Err(e) => {
                                    warn!("Invalid message from client {}: {}", client_id_recv, e);
                                }
                            }
                        }
                    }
                }
                Ok(Message::Binary(data)) => {
                    debug!("Received binary message from {}: {} bytes", client_id_recv, data.len());
                }
                Ok(Message::Close(_)) => {
                    info!("Client {} closed connection", client_id_recv);
                    break;
                }
                Ok(Message::Ping(_)) => {
                    debug!("Received ping from client {}", client_id_recv);
                    // Axum automatically responds to pings
                }
                Ok(Message::Pong(_)) => {
                    debug!("Received pong from client {}", client_id_recv);
                }
                Err(e) => {
                    error!("WebSocket error for client {}: {}", client_id_recv, e);
                    break;
                }
            }
        }
        
        debug!("Receive task for client {} ended", client_id_recv);
    });

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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_stream_message_serialization() {
        let event = crate::storage::StoredEvent {
            id: "test-1".to_string(),
            event_type: "swap".to_string(),
            source: "jupiter".to_string(),
            data: serde_json::json!({"amount": 1000}),
            timestamp: chrono::Utc::now(),
            block_height: Some(12345),
            transaction_hash: Some("abc123".to_string()),
        };

        let message = StreamMessage::Event { event };
        let json = serde_json::to_string(&message).unwrap();
        
        let deserialized: StreamMessage = serde_json::from_str(&json).unwrap();
        
        match deserialized {
            StreamMessage::Event { event } => {
                assert_eq!(event.id, "test-1");
                assert_eq!(event.event_type, "swap");
            }
            _ => panic!("Expected Event message"),
        }
    }
}