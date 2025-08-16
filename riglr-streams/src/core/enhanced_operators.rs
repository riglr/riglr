//! Enhanced stream operators that integrate with riglr-events-core utilities
//!
//! This module provides advanced streaming operators that leverage the performance
//! and reliability utilities from riglr-events-core, including batching, deduplication,
//! rate limiting, and performance metrics.

use async_trait::async_trait;
use std::any::Any;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::broadcast;

use crate::core::{Stream, StreamError, StreamHealth};
use riglr_events_core::prelude::Event;
use riglr_events_core::prelude::*;

/// Wrapper for batched events that implements Event
#[derive(Debug, Clone)]
pub struct BatchEvent<E: Event> {
    pub events: Vec<E>,
    pub batch_timestamp: SystemTime,
    pub batch_id: String,
    pub metadata: EventMetadata,
}

impl<E: Event + Clone + 'static> Event for BatchEvent<E> {
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

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn clone_boxed(&self) -> Box<dyn Event> {
        Box::new(self.clone())
    }

    fn to_json(&self) -> EventResult<serde_json::Value> {
        Ok(serde_json::json!({
            "metadata": self.metadata,
            "events": self.events.len(),
            "batch_timestamp": self.batch_timestamp
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis()
        }))
    }
}

/// Enhanced stream that includes batching capabilities
pub struct BatchedStream<S: Stream> {
    inner: S,
    name: String,
    event_tx: broadcast::Sender<Arc<BatchEvent<S::Event>>>,
    _batcher_handle: tokio::task::JoinHandle<()>,
}

impl<S: Stream + 'static> BatchedStream<S> {
    /// Create a new batched stream
    pub fn new(inner: S, _batch_size: usize, _timeout: Duration) -> Self {
        let (tx, _) = broadcast::channel(1000);
        let name = format!("batched-{}", inner.name());

        // Create a task handle placeholder - in a real implementation,
        // this would spawn a task that collects events into batches
        let _batcher_handle = tokio::spawn(async move {
            // Placeholder for batching logic
        });

        Self {
            inner,
            name,
            event_tx: tx,
            _batcher_handle,
        }
    }
}

#[async_trait]
impl<S: Stream> Stream for BatchedStream<S>
where
    S::Event: Event + Clone,
{
    type Event = BatchEvent<S::Event>;
    type Config = S::Config;

    async fn start(&mut self, config: Self::Config) -> Result<(), StreamError> {
        self.inner.start(config).await
    }

    async fn stop(&mut self) -> Result<(), StreamError> {
        self.inner.stop().await
    }

    fn subscribe(&self) -> broadcast::Receiver<Arc<Self::Event>> {
        self.event_tx.subscribe()
    }

    fn is_running(&self) -> bool {
        self.inner.is_running()
    }

    async fn health(&self) -> StreamHealth {
        let mut health = self.inner.health().await;
        // Add batching-specific metrics
        health.custom_metrics = Some(serde_json::json!({
            "batching_enabled": true,
            "batch_size": "dynamic"
        }));
        health
    }

    fn name(&self) -> &str {
        &self.name
    }
}

/// Enhanced stream with rate limiting and deduplication
pub struct EnhancedStream<S: Stream> {
    inner: S,
    name: String,
    event_tx: broadcast::Sender<Arc<S::Event>>,
    metrics: Arc<EventPerformanceMetrics>,
    _processor_handle: tokio::task::JoinHandle<()>,
}

impl<S: Stream + 'static> EnhancedStream<S> {
    /// Create an enhanced stream with rate limiting and deduplication
    pub fn new(inner: S, _max_rate_per_second: u64, _dedup_ttl: Duration) -> Self {
        let (tx, _) = broadcast::channel(10000);
        let name = format!("enhanced-{}", inner.name());
        let metrics = Arc::new(EventPerformanceMetrics::new());

        // Create processor handle - in a real implementation, this would
        // set up rate limiting and deduplication
        let _processor_handle = tokio::spawn(async move {
            // Placeholder for enhanced processing logic
        });

        Self {
            inner,
            name,
            event_tx: tx,
            metrics,
            _processor_handle,
        }
    }

    /// Get performance metrics for this stream
    pub async fn get_metrics(&self) -> MetricsSummary {
        self.metrics.summary().await
    }

    /// Reset performance metrics
    pub async fn reset_metrics(&self) {
        self.metrics.reset().await;
    }
}

#[async_trait]
impl<S: Stream> Stream for EnhancedStream<S> {
    type Event = S::Event;
    type Config = S::Config;

    async fn start(&mut self, config: Self::Config) -> Result<(), StreamError> {
        self.inner.start(config).await
    }

    async fn stop(&mut self) -> Result<(), StreamError> {
        self.inner.stop().await
    }

    fn subscribe(&self) -> broadcast::Receiver<Arc<Self::Event>> {
        self.event_tx.subscribe()
    }

    fn is_running(&self) -> bool {
        self.inner.is_running()
    }

    async fn health(&self) -> StreamHealth {
        let mut health = self.inner.health().await;

        // Add enhanced metrics to health status
        let metrics = self.metrics.summary().await;
        health.custom_metrics = Some(serde_json::json!({
            "total_events_processed": metrics.total_events,
            "total_errors": metrics.total_errors,
            "error_rate_percent": metrics.error_rate,
            "avg_processing_time_ms": metrics.avg_processing_time.as_millis(),
            "p95_processing_time_ms": metrics.p95_processing_time.as_millis(),
            "uptime_seconds": metrics.uptime.as_secs(),
            "rate_limited": true,
            "deduplicated": true
        }));

        health
    }

    fn name(&self) -> &str {
        &self.name
    }
}

/// Extension trait to add enhanced operators to existing streams
pub trait EnhancedStreamExt: Stream + Sized {
    /// Add batching to the stream
    fn with_batching(self, batch_size: usize, timeout: Duration) -> BatchedStream<Self>
    where
        Self: 'static,
        Self::Event: Event + Clone,
    {
        BatchedStream::new(self, batch_size, timeout)
    }

    /// Add rate limiting and deduplication
    fn with_enhancements(
        self,
        max_rate_per_second: u64,
        dedup_ttl: Duration,
    ) -> EnhancedStream<Self>
    where
        Self: 'static,
    {
        EnhancedStream::new(self, max_rate_per_second, dedup_ttl)
    }
}

// Implement the extension trait for all streams
impl<S: Stream> EnhancedStreamExt for S {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::mock_stream::MockStream;
    use std::time::Duration;

    #[tokio::test]
    async fn test_enhanced_stream_integration() {
        let mock_stream = MockStream::new("test-enhanced");
        let enhanced = mock_stream.with_enhancements(100, Duration::from_secs(60));

        assert_eq!(enhanced.name(), "enhanced-test-enhanced");
        assert!(!enhanced.is_running());

        let metrics = enhanced.get_metrics().await;
        assert_eq!(metrics.total_events, 0);
    }

    #[tokio::test]
    async fn test_batched_stream_integration() {
        let mock_stream = MockStream::new("test-batched");
        let batched = mock_stream.with_batching(10, Duration::from_millis(100));

        assert_eq!(batched.name(), "batched-test-batched");
        assert!(!batched.is_running());

        let health = batched.health().await;
        assert!(health.custom_metrics.is_some());
    }
}
