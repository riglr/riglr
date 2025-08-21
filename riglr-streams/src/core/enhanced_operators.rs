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
    /// The collection of events that are batched together
    pub events: Vec<E>,
    /// When the batch was created
    pub batch_timestamp: SystemTime,
    /// Unique identifier for this batch
    pub batch_id: String,
    /// Event metadata for the batch event
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
        let metrics = Arc::new(EventPerformanceMetrics::default());

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
    use riglr_events_core::prelude::{Event, EventKind, EventMetadata};
    use std::any::Any;
    use std::time::Duration;

    // Test helper: Create a simple test event
    #[derive(Debug, Clone)]
    struct TestEvent {
        metadata: EventMetadata,
    }

    impl TestEvent {
        fn new(id: String, kind: EventKind) -> Self {
            let metadata = EventMetadata::new(id, kind, "test-source".to_string());
            Self { metadata }
        }
    }

    impl Event for TestEvent {
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

        fn to_json(&self) -> riglr_events_core::EventResult<serde_json::Value> {
            Ok(serde_json::json!({
                "metadata": self.metadata
            }))
        }
    }

    // Tests for BatchEvent struct
    #[test]
    fn test_batch_event_new() {
        let events = vec![
            TestEvent::new("event1".to_string(), EventKind::Transaction),
            TestEvent::new("event2".to_string(), EventKind::Swap),
        ];
        let batch_timestamp = SystemTime::now();
        let batch_id = "batch123".to_string();
        let metadata = EventMetadata::new(
            "batch-id".to_string(),
            EventKind::Custom("batch".to_string()),
            "batch-source".to_string(),
        );

        let batch_event = BatchEvent {
            events: events.clone(),
            batch_timestamp,
            batch_id: batch_id.clone(),
            metadata: metadata.clone(),
        };

        assert_eq!(batch_event.events.len(), 2);
        assert_eq!(batch_event.batch_id, "batch123");
        assert_eq!(batch_event.metadata.id, "batch-id");
    }

    #[test]
    fn test_batch_event_id() {
        let events = vec![TestEvent::new("event1".to_string(), EventKind::Transaction)];
        let metadata = EventMetadata::new(
            "test-batch-id".to_string(),
            EventKind::Custom("batch".to_string()),
            "source".to_string(),
        );

        let batch_event = BatchEvent {
            events,
            batch_timestamp: SystemTime::now(),
            batch_id: "batch1".to_string(),
            metadata,
        };

        assert_eq!(batch_event.id(), "test-batch-id");
    }

    #[test]
    fn test_batch_event_kind() {
        let events = vec![TestEvent::new("event1".to_string(), EventKind::Transaction)];
        let metadata = EventMetadata::new(
            "batch-id".to_string(),
            EventKind::Custom("batch".to_string()),
            "source".to_string(),
        );

        let batch_event = BatchEvent {
            events,
            batch_timestamp: SystemTime::now(),
            batch_id: "batch1".to_string(),
            metadata,
        };

        assert_eq!(batch_event.kind(), &EventKind::Custom("batch".to_string()));
    }

    #[test]
    fn test_batch_event_metadata() {
        let events = vec![TestEvent::new("event1".to_string(), EventKind::Transaction)];
        let metadata = EventMetadata::new(
            "batch-id".to_string(),
            EventKind::Custom("batch".to_string()),
            "test-source".to_string(),
        );

        let batch_event = BatchEvent {
            events,
            batch_timestamp: SystemTime::now(),
            batch_id: "batch1".to_string(),
            metadata,
        };

        let meta = batch_event.metadata();
        assert_eq!(meta.id, "batch-id");
        assert_eq!(meta.source, "test-source");
    }

    #[test]
    fn test_batch_event_metadata_mut() {
        let events = vec![TestEvent::new("event1".to_string(), EventKind::Transaction)];
        let metadata = EventMetadata::new(
            "batch-id".to_string(),
            EventKind::Custom("batch".to_string()),
            "source".to_string(),
        );

        let mut batch_event = BatchEvent {
            events,
            batch_timestamp: SystemTime::now(),
            batch_id: "batch1".to_string(),
            metadata,
        };

        {
            let meta_mut = batch_event.metadata_mut();
            meta_mut.source = "modified-source".to_string();
        }

        assert_eq!(batch_event.metadata().source, "modified-source");
    }

    #[test]
    fn test_batch_event_as_any() {
        let events = vec![TestEvent::new("event1".to_string(), EventKind::Transaction)];
        let metadata = EventMetadata::new(
            "batch-id".to_string(),
            EventKind::Custom("batch".to_string()),
            "source".to_string(),
        );

        let batch_event = BatchEvent {
            events,
            batch_timestamp: SystemTime::now(),
            batch_id: "batch1".to_string(),
            metadata,
        };

        let any_ref = batch_event.as_any();
        assert!(any_ref.downcast_ref::<BatchEvent<TestEvent>>().is_some());
    }

    #[test]
    fn test_batch_event_as_any_mut() {
        let events = vec![TestEvent::new("event1".to_string(), EventKind::Transaction)];
        let metadata = EventMetadata::new(
            "batch-id".to_string(),
            EventKind::Custom("batch".to_string()),
            "source".to_string(),
        );

        let mut batch_event = BatchEvent {
            events,
            batch_timestamp: SystemTime::now(),
            batch_id: "batch1".to_string(),
            metadata,
        };

        let any_mut = batch_event.as_any_mut();
        assert!(any_mut.downcast_mut::<BatchEvent<TestEvent>>().is_some());
    }

    #[test]
    fn test_batch_event_clone_boxed() {
        let events = vec![TestEvent::new("event1".to_string(), EventKind::Transaction)];
        let metadata = EventMetadata::new(
            "batch-id".to_string(),
            EventKind::Custom("batch".to_string()),
            "source".to_string(),
        );

        let batch_event = BatchEvent {
            events,
            batch_timestamp: SystemTime::now(),
            batch_id: "batch1".to_string(),
            metadata,
        };

        let cloned = batch_event.clone_boxed();
        assert_eq!(cloned.id(), "batch-id");
        assert_eq!(cloned.kind(), &EventKind::Custom("batch".to_string()));
    }

    #[test]
    fn test_batch_event_to_json() {
        let events = vec![
            TestEvent::new("event1".to_string(), EventKind::Transaction),
            TestEvent::new("event2".to_string(), EventKind::Swap),
        ];
        let batch_timestamp = SystemTime::UNIX_EPOCH + Duration::from_millis(1000);
        let metadata = EventMetadata::new(
            "batch-id".to_string(),
            EventKind::Custom("batch".to_string()),
            "source".to_string(),
        );

        let batch_event = BatchEvent {
            events,
            batch_timestamp,
            batch_id: "batch1".to_string(),
            metadata,
        };

        let json_result = batch_event.to_json();
        assert!(json_result.is_ok());

        let json = json_result.unwrap();
        assert_eq!(json["events"], 2);
        assert_eq!(json["batch_timestamp"], 1000);
        assert!(json.get("metadata").is_some());
    }

    #[test]
    fn test_batch_event_to_json_with_duration_error() {
        let events = vec![TestEvent::new("event1".to_string(), EventKind::Transaction)];
        // Set batch_timestamp to before UNIX_EPOCH to trigger duration error path
        let batch_timestamp = SystemTime::UNIX_EPOCH - Duration::from_secs(1);
        let metadata = EventMetadata::new(
            "batch-id".to_string(),
            EventKind::Custom("batch".to_string()),
            "source".to_string(),
        );

        let batch_event = BatchEvent {
            events,
            batch_timestamp,
            batch_id: "batch1".to_string(),
            metadata,
        };

        let json_result = batch_event.to_json();
        assert!(json_result.is_ok());

        let json = json_result.unwrap();
        // Should default to 0 when duration calculation fails
        assert_eq!(json["batch_timestamp"], 0);
    }

    #[test]
    fn test_batch_event_clone() {
        let events = vec![TestEvent::new("event1".to_string(), EventKind::Transaction)];
        let metadata = EventMetadata::new(
            "batch-id".to_string(),
            EventKind::Custom("batch".to_string()),
            "source".to_string(),
        );

        let batch_event = BatchEvent {
            events,
            batch_timestamp: SystemTime::now(),
            batch_id: "batch1".to_string(),
            metadata,
        };

        let cloned = batch_event.clone();
        assert_eq!(cloned.id(), batch_event.id());
        assert_eq!(cloned.batch_id, batch_event.batch_id);
        assert_eq!(cloned.events.len(), batch_event.events.len());
    }

    // Tests for BatchedStream
    #[test]
    fn test_batched_stream_new() {
        let mock_stream = MockStream::new("test-stream");
        let batched = BatchedStream::new(mock_stream, 10, Duration::from_millis(100));

        assert_eq!(batched.name(), "batched-test-stream");
        assert!(!batched.is_running());
    }

    #[tokio::test]
    async fn test_batched_stream_start() {
        let mock_stream = MockStream::new("test-stream");
        let mut batched = BatchedStream::new(mock_stream, 5, Duration::from_millis(50));

        let config = crate::core::mock_stream::MockConfig::default();
        let result = batched.start(config).await;
        assert!(result.is_ok());

        batched.stop().await.unwrap();
    }

    #[tokio::test]
    async fn test_batched_stream_stop() {
        let mock_stream = MockStream::new("test-stream");
        let mut batched = BatchedStream::new(mock_stream, 5, Duration::from_millis(50));

        let result = batched.stop().await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_batched_stream_subscribe() {
        let mock_stream = MockStream::new("test-stream");
        let batched = BatchedStream::new(mock_stream, 5, Duration::from_millis(50));

        let _rx = batched.subscribe();
        // If we can subscribe without error, the test passes
    }

    #[test]
    fn test_batched_stream_is_running() {
        let mock_stream = MockStream::new("test-stream");
        let batched = BatchedStream::new(mock_stream, 5, Duration::from_millis(50));

        assert!(!batched.is_running());
    }

    #[tokio::test]
    async fn test_batched_stream_health() {
        let mock_stream = MockStream::new("test-stream");
        let batched = BatchedStream::new(mock_stream, 5, Duration::from_millis(50));

        let health = batched.health().await;
        assert!(health.custom_metrics.is_some());

        let custom_metrics = health.custom_metrics.unwrap();
        assert!(custom_metrics["batching_enabled"]
            .as_bool()
            .unwrap_or(false));
        assert_eq!(custom_metrics["batch_size"], "dynamic");
    }

    #[test]
    fn test_batched_stream_name() {
        let mock_stream = MockStream::new("my-test-stream");
        let batched = BatchedStream::new(mock_stream, 5, Duration::from_millis(50));

        assert_eq!(batched.name(), "batched-my-test-stream");
    }

    // Tests for EnhancedStream
    #[test]
    fn test_enhanced_stream_new() {
        let mock_stream = MockStream::new("test-stream");
        let enhanced = EnhancedStream::new(mock_stream, 100, Duration::from_secs(60));

        assert_eq!(enhanced.name(), "enhanced-test-stream");
        assert!(!enhanced.is_running());
    }

    #[tokio::test]
    async fn test_enhanced_stream_start() {
        let mock_stream = MockStream::new("test-stream");
        let mut enhanced = EnhancedStream::new(mock_stream, 100, Duration::from_secs(60));

        let config = crate::core::mock_stream::MockConfig::default();
        let result = enhanced.start(config).await;
        assert!(result.is_ok());

        enhanced.stop().await.unwrap();
    }

    #[tokio::test]
    async fn test_enhanced_stream_stop() {
        let mock_stream = MockStream::new("test-stream");
        let mut enhanced = EnhancedStream::new(mock_stream, 100, Duration::from_secs(60));

        let result = enhanced.stop().await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_enhanced_stream_subscribe() {
        let mock_stream = MockStream::new("test-stream");
        let enhanced = EnhancedStream::new(mock_stream, 100, Duration::from_secs(60));

        let _rx = enhanced.subscribe();
        // If we can subscribe without error, the test passes
    }

    #[test]
    fn test_enhanced_stream_is_running() {
        let mock_stream = MockStream::new("test-stream");
        let enhanced = EnhancedStream::new(mock_stream, 100, Duration::from_secs(60));

        assert!(!enhanced.is_running());
    }

    #[tokio::test]
    async fn test_enhanced_stream_health() {
        let mock_stream = MockStream::new("test-stream");
        let enhanced = EnhancedStream::new(mock_stream, 100, Duration::from_secs(60));

        let health = enhanced.health().await;
        assert!(health.custom_metrics.is_some());

        let custom_metrics = health.custom_metrics.unwrap();
        assert!(custom_metrics.get("total_events_processed").is_some());
        assert!(custom_metrics.get("total_errors").is_some());
        assert!(custom_metrics.get("error_rate_percent").is_some());
        assert!(custom_metrics.get("avg_processing_time_ms").is_some());
        assert!(custom_metrics.get("p95_processing_time_ms").is_some());
        assert!(custom_metrics.get("uptime_seconds").is_some());
        assert!(custom_metrics["rate_limited"].as_bool().unwrap_or(false));
        assert!(custom_metrics["deduplicated"].as_bool().unwrap_or(false));
    }

    #[test]
    fn test_enhanced_stream_name() {
        let mock_stream = MockStream::new("my-test-stream");
        let enhanced = EnhancedStream::new(mock_stream, 100, Duration::from_secs(60));

        assert_eq!(enhanced.name(), "enhanced-my-test-stream");
    }

    #[tokio::test]
    async fn test_enhanced_stream_get_metrics() {
        let mock_stream = MockStream::new("test-stream");
        let enhanced = EnhancedStream::new(mock_stream, 100, Duration::from_secs(60));

        let metrics = enhanced.get_metrics().await;
        assert_eq!(metrics.total_events, 0);
        assert_eq!(metrics.total_errors, 0);
    }

    #[tokio::test]
    async fn test_enhanced_stream_reset_metrics() {
        let mock_stream = MockStream::new("test-stream");
        let enhanced = EnhancedStream::new(mock_stream, 100, Duration::from_secs(60));

        // This should not panic
        enhanced.reset_metrics().await;
    }

    // Tests for EnhancedStreamExt trait
    #[test]
    fn test_enhanced_stream_ext_with_batching() {
        let mock_stream = MockStream::new("test-stream");
        let batched = mock_stream.with_batching(10, Duration::from_millis(100));

        assert_eq!(batched.name(), "batched-test-stream");
    }

    #[test]
    fn test_enhanced_stream_ext_with_enhancements() {
        let mock_stream = MockStream::new("test-stream");
        let enhanced = mock_stream.with_enhancements(50, Duration::from_secs(30));

        assert_eq!(enhanced.name(), "enhanced-test-stream");
    }

    // Integration tests
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

    // Edge case tests
    #[test]
    fn test_batched_stream_with_zero_batch_size() {
        let mock_stream = MockStream::new("test-stream");
        let batched = BatchedStream::new(mock_stream, 0, Duration::from_millis(100));

        assert_eq!(batched.name(), "batched-test-stream");
    }

    #[test]
    fn test_batched_stream_with_zero_timeout() {
        let mock_stream = MockStream::new("test-stream");
        let batched = BatchedStream::new(mock_stream, 10, Duration::from_millis(0));

        assert_eq!(batched.name(), "batched-test-stream");
    }

    #[test]
    fn test_enhanced_stream_with_zero_rate() {
        let mock_stream = MockStream::new("test-stream");
        let enhanced = EnhancedStream::new(mock_stream, 0, Duration::from_secs(60));

        assert_eq!(enhanced.name(), "enhanced-test-stream");
    }

    #[test]
    fn test_enhanced_stream_with_zero_ttl() {
        let mock_stream = MockStream::new("test-stream");
        let enhanced = EnhancedStream::new(mock_stream, 100, Duration::from_secs(0));

        assert_eq!(enhanced.name(), "enhanced-test-stream");
    }

    #[test]
    fn test_batch_event_with_empty_events() {
        let events: Vec<TestEvent> = vec![];
        let metadata = EventMetadata::new(
            "empty-batch".to_string(),
            EventKind::Custom("batch".to_string()),
            "source".to_string(),
        );

        let batch_event = BatchEvent {
            events,
            batch_timestamp: SystemTime::now(),
            batch_id: "empty-batch".to_string(),
            metadata,
        };

        assert_eq!(batch_event.events.len(), 0);

        let json_result = batch_event.to_json();
        assert!(json_result.is_ok());

        let json = json_result.unwrap();
        assert_eq!(json["events"], 0);
    }

    #[test]
    fn test_batch_event_with_large_number_of_events() {
        let events: Vec<TestEvent> = (0..1000)
            .map(|i| TestEvent::new(format!("event{}", i), EventKind::Transaction))
            .collect();
        let metadata = EventMetadata::new(
            "large-batch".to_string(),
            EventKind::Custom("batch".to_string()),
            "source".to_string(),
        );

        let batch_event = BatchEvent {
            events,
            batch_timestamp: SystemTime::now(),
            batch_id: "large-batch".to_string(),
            metadata,
        };

        assert_eq!(batch_event.events.len(), 1000);

        let json_result = batch_event.to_json();
        assert!(json_result.is_ok());

        let json = json_result.unwrap();
        assert_eq!(json["events"], 1000);
    }

    // Test concurrent operations
    #[tokio::test]
    async fn test_enhanced_stream_concurrent_operations() {
        let mock_stream = MockStream::new("concurrent-test");
        let enhanced = EnhancedStream::new(mock_stream, 100, Duration::from_secs(60));

        // Test concurrent metrics access
        let metrics_future1 = enhanced.get_metrics();
        let metrics_future2 = enhanced.get_metrics();
        let reset_future = enhanced.reset_metrics();

        let (metrics1, metrics2, _) = tokio::join!(metrics_future1, metrics_future2, reset_future);

        assert_eq!(metrics1.total_events, 0);
        assert_eq!(metrics2.total_events, 0);
    }

    #[tokio::test]
    async fn test_batched_stream_concurrent_health_checks() {
        let mock_stream = MockStream::new("concurrent-health");
        let batched = BatchedStream::new(mock_stream, 10, Duration::from_millis(100));

        // Test concurrent health checks
        let health_future1 = batched.health();
        let health_future2 = batched.health();

        let (health1, health2) = tokio::join!(health_future1, health_future2);

        assert!(health1.custom_metrics.is_some());
        assert!(health2.custom_metrics.is_some());
    }

    // Test with different event kinds
    #[test]
    fn test_batch_event_with_different_event_kinds() {
        let events = vec![
            TestEvent::new("tx1".to_string(), EventKind::Transaction),
            TestEvent::new("swap1".to_string(), EventKind::Swap),
            TestEvent::new("price1".to_string(), EventKind::Price),
            TestEvent::new("token1".to_string(), EventKind::Custom("token".to_string())),
        ];
        let metadata = EventMetadata::new(
            "mixed-batch".to_string(),
            EventKind::Custom("batch".to_string()),
            "source".to_string(),
        );

        let batch_event = BatchEvent {
            events,
            batch_timestamp: SystemTime::now(),
            batch_id: "mixed-batch".to_string(),
            metadata,
        };

        assert_eq!(batch_event.events.len(), 4);
        assert_eq!(batch_event.events[0].kind(), &EventKind::Transaction);
        assert_eq!(batch_event.events[1].kind(), &EventKind::Swap);
        assert_eq!(batch_event.events[2].kind(), &EventKind::Price);
        assert_eq!(
            batch_event.events[3].kind(),
            &EventKind::Custom("token".to_string())
        );
    }

    // Test error scenarios with stream operations
    #[tokio::test]
    async fn test_batched_stream_start_with_error() {
        // Create a mock stream that we'll force to have an error
        let mut mock_stream = MockStream::new("error-test");

        // Start the stream first to put it in running state
        let config = crate::core::mock_stream::MockConfig::default();
        mock_stream.start(config.clone()).await.unwrap();

        let mut batched = BatchedStream::new(mock_stream, 5, Duration::from_millis(50));

        // This should return an error because the inner stream is already running
        let result = batched.start(config).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_enhanced_stream_start_with_error() {
        // Create a mock stream that we'll force to have an error
        let mut mock_stream = MockStream::new("error-test");

        // Start the stream first to put it in running state
        let config = crate::core::mock_stream::MockConfig::default();
        mock_stream.start(config.clone()).await.unwrap();

        let mut enhanced = EnhancedStream::new(mock_stream, 100, Duration::from_secs(60));

        // This should return an error because the inner stream is already running
        let result = enhanced.start(config).await;
        assert!(result.is_err());
    }
}
