use super::StreamMetadata;
use async_trait::async_trait;
use riglr_events_core::prelude::Event;
use riglr_events_core::StreamInfo;
use std::any::Any;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::broadcast;

use super::error::StreamError;

/// Core trait that all streams must implement
#[async_trait]
pub trait Stream: Send + Sync {
    /// The type of events this stream produces
    type Event: Event + Clone + Send + Sync + 'static;

    /// Configuration type for this stream
    type Config: Clone + Send + Sync;

    /// Start the stream with given configuration
    async fn start(&mut self, config: Self::Config) -> Result<(), StreamError>;

    /// Stop the stream gracefully
    async fn stop(&mut self) -> Result<(), StreamError>;

    /// Get a receiver for events from this stream
    fn subscribe(&self) -> broadcast::Receiver<Arc<Self::Event>>;

    /// Check if stream is currently running
    fn is_running(&self) -> bool;

    /// Get stream health status
    async fn health(&self) -> StreamHealth;

    /// Get stream name/identifier
    fn name(&self) -> &str;
}

/// Health status of a stream
///
/// Enhanced with events-core integration for better observability
#[derive(Debug, Clone, Default)]
pub struct StreamHealth {
    /// Whether the stream is connected
    pub is_connected: bool,
    /// Last time an event was received
    pub last_event_time: Option<SystemTime>,
    /// Number of errors encountered
    pub error_count: u64,
    /// Total number of events processed
    pub events_processed: u64,
    /// Current backlog size (if applicable)
    pub backlog_size: Option<usize>,
    /// Stream-specific health data
    pub custom_metrics: Option<serde_json::Value>,
    /// Integration with events-core StreamInfo (optional)
    pub stream_info: Option<StreamInfo>,
}

/// Extension trait for events with streaming context
pub trait StreamEvent: Send + Sync {
    /// Get stream-specific metadata
    fn stream_metadata(&self) -> Option<&StreamMetadata>;

    /// Get the stream source identifier
    fn stream_source(&self) -> Option<&str> {
        StreamEvent::stream_metadata(self).map(|m| m.stream_source.as_str())
    }

    /// Get when the event was received
    fn received_at(&self) -> Option<SystemTime> {
        StreamEvent::stream_metadata(self).map(|m| m.received_at)
    }

    /// Get the sequence number if available
    fn sequence_number(&self) -> Option<u64> {
        StreamEvent::stream_metadata(self).and_then(|m| m.sequence_number)
    }
}

/// Type-erased stream for heterogeneous stream management
#[async_trait]
pub trait DynamicStream: Send + Sync {
    /// Start the stream
    async fn start_dynamic(&mut self) -> Result<(), StreamError>;

    /// Stop the stream
    async fn stop_dynamic(&mut self) -> Result<(), StreamError>;

    /// Check if running
    fn is_running_dynamic(&self) -> bool;

    /// Get health status
    async fn health_dynamic(&self) -> StreamHealth;

    /// Get stream name
    fn name_dynamic(&self) -> &str;

    /// Subscribe to events as type-erased values
    fn subscribe_dynamic(&self) -> broadcast::Receiver<Arc<dyn Any + Send + Sync>>;
}

/// Wrapper to make any Stream implement DynamicStream
pub struct DynamicStreamWrapper<S: Stream> {
    pub(crate) inner: S,
    event_forwarder: broadcast::Sender<Arc<dyn Any + Send + Sync>>,
    is_started: bool,
}

impl<S: Stream + 'static> DynamicStreamWrapper<S> {
    /// Create a new dynamic stream wrapper around the given stream
    pub fn new(stream: S) -> Self {
        let (tx, _) = broadcast::channel(10000);
        Self {
            inner: stream,
            event_forwarder: tx,
            is_started: false,
        }
    }

    /// Forward events from the inner stream to the type-erased channel
    async fn forward_events(&self) {
        let mut rx = self.inner.subscribe();
        let tx = self.event_forwarder.clone();

        tokio::spawn(async move {
            while let Ok(event) = rx.recv().await {
                let _ = tx.send(event as Arc<dyn Any + Send + Sync>);
            }
        });
    }
}

#[async_trait]
impl<S: Stream + Send + Sync + 'static> DynamicStream for DynamicStreamWrapper<S> {
    async fn start_dynamic(&mut self) -> Result<(), StreamError> {
        if self.is_started {
            return Ok(()); // Already started
        }

        // Start forwarding events
        self.forward_events().await;
        self.is_started = true;

        // Note: The actual stream.start() should be called before wrapping,
        // or we need to store the config in the wrapper.
        // For now, we assume the stream is already configured and just needs
        // the forwarding to be set up.
        Ok(())
    }

    async fn stop_dynamic(&mut self) -> Result<(), StreamError> {
        self.inner.stop().await
    }

    fn is_running_dynamic(&self) -> bool {
        self.inner.is_running()
    }

    async fn health_dynamic(&self) -> StreamHealth {
        self.inner.health().await
    }

    fn name_dynamic(&self) -> &str {
        self.inner.name()
    }

    fn subscribe_dynamic(&self) -> broadcast::Receiver<Arc<dyn Any + Send + Sync>> {
        self.event_forwarder.subscribe()
    }
}
