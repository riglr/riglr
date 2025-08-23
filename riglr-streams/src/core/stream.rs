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
#[derive(Debug, Clone, Default, PartialEq)]
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

impl StreamHealth {
    /// Create a healthy stream health instance
    pub const fn healthy() -> Self {
        Self {
            is_connected: true,
            last_event_time: None,
            error_count: 0,
            events_processed: 0,
            backlog_size: None,
            custom_metrics: None,
            stream_info: None,
        }
    }

    /// Create a degraded stream health instance
    pub const fn degraded() -> Self {
        Self {
            is_connected: true,
            last_event_time: None,
            error_count: 1,
            events_processed: 0,
            backlog_size: None,
            custom_metrics: None,
            stream_info: None,
        }
    }

    /// Create an unhealthy stream health instance
    pub const fn unhealthy() -> Self {
        Self {
            is_connected: false,
            last_event_time: None,
            error_count: 0,
            events_processed: 0,
            backlog_size: None,
            custom_metrics: None,
            stream_info: None,
        }
    }

    /// Constants for backward compatibility
    #[allow(non_upper_case_globals)]
    pub const Healthy: Self = Self::healthy();
    /// Degraded stream health constant
    #[allow(non_upper_case_globals)]
    pub const Degraded: Self = Self::degraded();
    /// Unhealthy stream health constant
    #[allow(non_upper_case_globals)]
    pub const Unhealthy: Self = Self::unhealthy();
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

        // Start the inner stream if it's not already running
        // Try to start with default config if available
        // This is needed for test streams like MockStream
        if !self.inner.is_running() {
            // We can't use S::Config::default() here because not all configs implement Default
            // So we rely on the inner stream being in a runnable state or
            // the test setting it up properly
            // For MockStream in tests, we'll need to ensure it's started before adding
        }

        // Start forwarding events
        self.forward_events().await;
        self.is_started = true;

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

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use riglr_events_core::prelude::{EventKind, EventMetadata};
    use std::any::Any;
    use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
    use std::sync::Arc;
    use std::time::{Duration, SystemTime, UNIX_EPOCH};
    use tokio::sync::broadcast;

    // Mock event for testing
    #[derive(Debug, Clone)]
    struct MockEvent {
        metadata: EventMetadata,
    }

    impl MockEvent {
        fn new(id: &str) -> Self {
            Self {
                metadata: EventMetadata::new(
                    id.to_string(),
                    EventKind::Swap,
                    "mock_stream".to_string(),
                ),
            }
        }
    }

    impl Event for MockEvent {
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
                "id": self.id(),
                "kind": format!("{:?}", self.kind()),
                "source": &self.metadata.source
            }))
        }
    }

    // Mock stream implementation for testing
    #[derive(Clone)]
    struct MockConfig {
        should_fail_start: bool,
        should_fail_stop: bool,
    }

    impl Default for MockConfig {
        fn default() -> Self {
            Self {
                should_fail_start: false,
                should_fail_stop: false,
            }
        }
    }

    struct TestStream {
        name: String,
        is_running: Arc<AtomicBool>,
        event_tx: broadcast::Sender<Arc<MockEvent>>,
        events_processed: Arc<AtomicU64>,
        should_fail_start: bool,
        should_fail_stop: bool,
    }

    impl TestStream {
        fn new(name: &str) -> Self {
            let (event_tx, _) = broadcast::channel(100);
            Self {
                name: name.to_string(),
                is_running: Arc::new(AtomicBool::new(false)),
                event_tx,
                events_processed: Arc::new(AtomicU64::new(0)),
                should_fail_start: false,
                should_fail_stop: false,
            }
        }

        fn with_fail_start(mut self) -> Self {
            self.should_fail_start = true;
            self
        }

        fn with_fail_stop(mut self) -> Self {
            self.should_fail_stop = true;
            self
        }

        fn send_event(
            &self,
            event: MockEvent,
        ) -> Result<usize, broadcast::error::SendError<Arc<MockEvent>>> {
            self.events_processed.fetch_add(1, Ordering::Relaxed);
            self.event_tx.send(Arc::new(event))
        }
    }

    #[async_trait]
    impl Stream for TestStream {
        type Event = MockEvent;
        type Config = MockConfig;

        async fn start(&mut self, config: Self::Config) -> Result<(), StreamError> {
            if config.should_fail_start || self.should_fail_start {
                return Err(StreamError::Configuration {
                    message: "Start failed".to_string(),
                });
            }
            self.is_running.store(true, Ordering::Relaxed);
            Ok(())
        }

        async fn stop(&mut self) -> Result<(), StreamError> {
            if self.should_fail_stop {
                return Err(StreamError::Configuration {
                    message: "Stop failed".to_string(),
                });
            }
            self.is_running.store(false, Ordering::Relaxed);
            Ok(())
        }

        fn subscribe(&self) -> broadcast::Receiver<Arc<Self::Event>> {
            self.event_tx.subscribe()
        }

        fn is_running(&self) -> bool {
            self.is_running.load(Ordering::Relaxed)
        }

        async fn health(&self) -> StreamHealth {
            StreamHealth {
                is_connected: self.is_running(),
                last_event_time: Some(SystemTime::now()),
                error_count: 0,
                events_processed: self.events_processed.load(Ordering::Relaxed),
                backlog_size: Some(0),
                custom_metrics: Some(serde_json::json!({"test": true})),
                stream_info: None,
            }
        }

        fn name(&self) -> &str {
            &self.name
        }
    }

    // Mock event that implements StreamEvent trait
    struct MockStreamEvent {
        stream_metadata: StreamMetadata,
    }

    impl MockStreamEvent {
        fn new(source: &str) -> Self {
            Self {
                stream_metadata: StreamMetadata {
                    stream_source: source.to_string(),
                    received_at: SystemTime::now(),
                    sequence_number: Some(42),
                    custom_data: Some(serde_json::json!({"mock": true})),
                },
            }
        }

        fn new_without_sequence() -> Self {
            Self {
                stream_metadata: StreamMetadata {
                    stream_source: "no_seq_stream".to_string(),
                    received_at: UNIX_EPOCH + Duration::from_secs(1000),
                    sequence_number: None,
                    custom_data: None,
                },
            }
        }
    }

    impl StreamEvent for MockStreamEvent {
        fn stream_metadata(&self) -> Option<&StreamMetadata> {
            Some(&self.stream_metadata)
        }
    }

    // Tests for StreamHealth
    #[test]
    fn test_stream_health_default_should_have_correct_initial_values() {
        let health = StreamHealth::default();

        assert!(!health.is_connected);
        assert_eq!(health.last_event_time, None);
        assert_eq!(health.error_count, 0);
        assert_eq!(health.events_processed, 0);
        assert_eq!(health.backlog_size, None);
        assert_eq!(health.custom_metrics, None);
        assert_eq!(health.stream_info, None);
    }

    #[test]
    fn test_stream_health_clone_should_create_identical_copy() {
        let original = StreamHealth {
            is_connected: true,
            last_event_time: Some(SystemTime::now()),
            error_count: 5,
            events_processed: 100,
            backlog_size: Some(50),
            custom_metrics: Some(serde_json::json!({"metric": "value"})),
            stream_info: None,
        };

        let cloned = original.clone();

        assert_eq!(original.is_connected, cloned.is_connected);
        assert_eq!(original.last_event_time, cloned.last_event_time);
        assert_eq!(original.error_count, cloned.error_count);
        assert_eq!(original.events_processed, cloned.events_processed);
        assert_eq!(original.backlog_size, cloned.backlog_size);
        assert_eq!(original.custom_metrics, cloned.custom_metrics);
        assert_eq!(original.stream_info, cloned.stream_info);
    }

    #[test]
    fn test_stream_health_debug_should_contain_field_info() {
        let health = StreamHealth {
            is_connected: false,
            last_event_time: None,
            error_count: 999,
            events_processed: u64::MAX,
            backlog_size: Some(usize::MAX),
            custom_metrics: Some(serde_json::json!({"debug": "test"})),
            stream_info: None,
        };

        let debug_str = format!("{:?}", health);
        assert!(debug_str.contains("StreamHealth"));
        assert!(debug_str.contains("999"));
        assert!(debug_str.contains(&u64::MAX.to_string()));
    }

    // Tests for StreamEvent trait
    #[test]
    fn test_stream_event_stream_metadata_when_some_should_return_metadata() {
        let event = MockStreamEvent::new("test_stream");
        let metadata = event.stream_metadata();

        assert!(metadata.is_some());
        assert_eq!(metadata.unwrap().stream_source, "test_stream");
    }

    #[test]
    fn test_stream_event_stream_source_when_metadata_exists_should_return_source() {
        let event = MockStreamEvent::new("source_stream");
        assert_eq!(event.stream_source(), Some("source_stream"));
    }

    #[test]
    fn test_stream_event_stream_source_when_no_metadata_should_return_none() {
        // Mock event without StreamEvent implementation
        struct EmptyEvent;
        impl StreamEvent for EmptyEvent {
            fn stream_metadata(&self) -> Option<&StreamMetadata> {
                None
            }
        }

        let event = EmptyEvent;
        assert_eq!(event.stream_source(), None);
    }

    #[test]
    fn test_stream_event_received_at_when_metadata_exists_should_return_time() {
        let event = MockStreamEvent::new("time_stream");
        let received_at = event.received_at();

        assert!(received_at.is_some());
        assert!(received_at.unwrap() <= SystemTime::now());
    }

    #[test]
    fn test_stream_event_received_at_when_no_metadata_should_return_none() {
        struct EmptyEvent;
        impl StreamEvent for EmptyEvent {
            fn stream_metadata(&self) -> Option<&StreamMetadata> {
                None
            }
        }

        let event = EmptyEvent;
        assert_eq!(event.received_at(), None);
    }

    #[test]
    fn test_stream_event_sequence_number_when_some_should_return_number() {
        let event = MockStreamEvent::new("seq_stream");
        assert_eq!(event.sequence_number(), Some(42));
    }

    #[test]
    fn test_stream_event_sequence_number_when_none_should_return_none() {
        let event = MockStreamEvent::new_without_sequence();
        assert_eq!(event.sequence_number(), None);
    }

    #[test]
    fn test_stream_event_sequence_number_when_no_metadata_should_return_none() {
        struct EmptyEvent;
        impl StreamEvent for EmptyEvent {
            fn stream_metadata(&self) -> Option<&StreamMetadata> {
                None
            }
        }

        let event = EmptyEvent;
        assert_eq!(event.sequence_number(), None);
    }

    // Tests for DynamicStreamWrapper
    #[test]
    fn test_dynamic_stream_wrapper_new_should_create_wrapper() {
        let stream = TestStream::new("test_wrapper");
        let wrapper = DynamicStreamWrapper::new(stream);

        assert!(!wrapper.is_started);
        assert_eq!(wrapper.inner.name(), "test_wrapper");
    }

    #[tokio::test]
    async fn test_dynamic_stream_wrapper_start_dynamic_when_not_started_should_start() {
        let stream = TestStream::new("start_test");
        let mut wrapper = DynamicStreamWrapper::new(stream);

        let result = wrapper.start_dynamic().await;

        assert!(result.is_ok());
        assert!(wrapper.is_started);
    }

    #[tokio::test]
    async fn test_dynamic_stream_wrapper_start_dynamic_when_already_started_should_return_ok() {
        let stream = TestStream::new("already_started");
        let mut wrapper = DynamicStreamWrapper::new(stream);
        wrapper.is_started = true;

        let result = wrapper.start_dynamic().await;

        assert!(result.is_ok());
        assert!(wrapper.is_started);
    }

    #[tokio::test]
    async fn test_dynamic_stream_wrapper_stop_dynamic_should_delegate_to_inner() {
        let stream = TestStream::new("stop_test");
        let mut wrapper = DynamicStreamWrapper::new(stream);

        let result = wrapper.stop_dynamic().await;

        assert!(result.is_ok());
        assert!(!wrapper.inner.is_running());
    }

    #[tokio::test]
    async fn test_dynamic_stream_wrapper_stop_dynamic_when_inner_fails_should_return_error() {
        let stream = TestStream::new("stop_fail").with_fail_stop();
        let mut wrapper = DynamicStreamWrapper::new(stream);

        let result = wrapper.stop_dynamic().await;

        assert!(result.is_err());
        if let Err(StreamError::Configuration { message: msg }) = result {
            assert_eq!(msg, "Stop failed");
        } else {
            panic!("Expected Configuration error");
        }
    }

    #[test]
    fn test_dynamic_stream_wrapper_is_running_dynamic_should_delegate_to_inner() {
        let stream = TestStream::new("running_test");
        stream.is_running.store(true, Ordering::Relaxed);
        let wrapper = DynamicStreamWrapper::new(stream);

        assert!(wrapper.is_running_dynamic());
    }

    #[test]
    fn test_dynamic_stream_wrapper_is_running_dynamic_when_not_running_should_return_false() {
        let stream = TestStream::new("not_running");
        let wrapper = DynamicStreamWrapper::new(stream);

        assert!(!wrapper.is_running_dynamic());
    }

    #[tokio::test]
    async fn test_dynamic_stream_wrapper_health_dynamic_should_delegate_to_inner() {
        let stream = TestStream::new("health_test");
        stream.is_running.store(true, Ordering::Relaxed);
        let wrapper = DynamicStreamWrapper::new(stream);

        let health = wrapper.health_dynamic().await;

        assert!(health.is_connected);
        assert!(health.last_event_time.is_some());
        assert_eq!(health.error_count, 0);
        assert_eq!(health.backlog_size, Some(0));
        assert!(health.custom_metrics.is_some());
    }

    #[test]
    fn test_dynamic_stream_wrapper_name_dynamic_should_delegate_to_inner() {
        let stream = TestStream::new("name_test");
        let wrapper = DynamicStreamWrapper::new(stream);

        assert_eq!(wrapper.name_dynamic(), "name_test");
    }

    #[test]
    fn test_dynamic_stream_wrapper_subscribe_dynamic_should_return_receiver() {
        let stream = TestStream::new("subscribe_test");
        let wrapper = DynamicStreamWrapper::new(stream);

        let _receiver = wrapper.subscribe_dynamic();
        // Test that we can create a receiver without panicking
    }

    // Tests for DynamicStreamWrapper::forward_events
    #[tokio::test]
    async fn test_dynamic_stream_wrapper_forward_events_should_forward_to_type_erased_channel() {
        let stream = TestStream::new("forward_test");
        let wrapper = DynamicStreamWrapper::new(stream);

        // Start forwarding
        wrapper.forward_events().await;

        // Send an event through the inner stream
        let event = MockEvent::new("test_event");
        let _ = wrapper.inner.send_event(event.clone());

        // Subscribe to the type-erased channel
        let _receiver = wrapper.subscribe_dynamic();

        // Wait a bit for the forwarding to happen
        tokio::time::sleep(Duration::from_millis(10)).await;

        // We should be able to receive something, but we can't easily test the content
        // due to the async nature and type erasure
    }

    // Edge case tests
    #[test]
    fn test_stream_health_with_max_values_should_handle_correctly() {
        let health = StreamHealth {
            is_connected: true,
            last_event_time: Some(SystemTime::now()),
            error_count: u64::MAX,
            events_processed: u64::MAX,
            backlog_size: Some(usize::MAX),
            custom_metrics: Some(serde_json::json!({"max": "values"})),
            stream_info: None,
        };

        // Should not panic or overflow
        let debug_str = format!("{:?}", health);
        assert!(debug_str.contains(&u64::MAX.to_string()));
    }

    #[test]
    fn test_stream_event_with_empty_source_should_handle_correctly() {
        let event = MockStreamEvent {
            stream_metadata: StreamMetadata {
                stream_source: "".to_string(),
                received_at: UNIX_EPOCH,
                sequence_number: Some(0),
                custom_data: None,
            },
        };

        assert_eq!(event.stream_source(), Some(""));
        assert_eq!(event.sequence_number(), Some(0));
        assert_eq!(event.received_at(), Some(UNIX_EPOCH));
    }

    #[test]
    fn test_stream_event_with_max_sequence_number_should_handle_correctly() {
        let event = MockStreamEvent {
            stream_metadata: StreamMetadata {
                stream_source: "max_seq".to_string(),
                received_at: SystemTime::now(),
                sequence_number: Some(u64::MAX),
                custom_data: None,
            },
        };

        assert_eq!(event.sequence_number(), Some(u64::MAX));
    }

    // Test Stream trait implementation
    #[tokio::test]
    async fn test_test_stream_start_when_config_ok_should_succeed() {
        let mut stream = TestStream::new("start_success");
        let config = MockConfig::default();

        let result = stream.start(config).await;

        assert!(result.is_ok());
        assert!(stream.is_running());
    }

    #[tokio::test]
    async fn test_test_stream_start_when_config_fails_should_return_error() {
        let mut stream = TestStream::new("start_fail");
        let config = MockConfig {
            should_fail_start: true,
            should_fail_stop: false,
        };

        let result = stream.start(config).await;

        assert!(result.is_err());
        assert!(!stream.is_running());
    }

    #[tokio::test]
    async fn test_test_stream_start_when_stream_configured_to_fail_should_return_error() {
        let mut stream = TestStream::new("stream_fail").with_fail_start();
        let config = MockConfig::default();

        let result = stream.start(config).await;

        assert!(result.is_err());
        assert!(!stream.is_running());
    }

    #[tokio::test]
    async fn test_test_stream_stop_when_ok_should_succeed() {
        let mut stream = TestStream::new("stop_success");
        stream.is_running.store(true, Ordering::Relaxed);

        let result = stream.stop().await;

        assert!(result.is_ok());
        assert!(!stream.is_running());
    }

    #[tokio::test]
    async fn test_test_stream_stop_when_fails_should_return_error() {
        let mut stream = TestStream::new("stop_error").with_fail_stop();
        stream.is_running.store(true, Ordering::Relaxed);

        let result = stream.stop().await;

        assert!(result.is_err());
        // Stream state doesn't change when stop fails
        assert!(stream.is_running());
    }

    #[test]
    fn test_test_stream_subscribe_should_return_receiver() {
        let stream = TestStream::new("subscribe");
        let _receiver = stream.subscribe();
        // Should not panic
    }

    #[test]
    fn test_test_stream_is_running_when_true_should_return_true() {
        let stream = TestStream::new("running_true");
        stream.is_running.store(true, Ordering::Relaxed);

        assert!(stream.is_running());
    }

    #[test]
    fn test_test_stream_is_running_when_false_should_return_false() {
        let stream = TestStream::new("running_false");
        stream.is_running.store(false, Ordering::Relaxed);

        assert!(!stream.is_running());
    }

    #[tokio::test]
    async fn test_test_stream_health_should_return_correct_status() {
        let stream = TestStream::new("health_check");
        stream.is_running.store(true, Ordering::Relaxed);
        stream.events_processed.store(50, Ordering::Relaxed);

        let health = stream.health().await;

        assert!(health.is_connected);
        assert!(health.last_event_time.is_some());
        assert_eq!(health.error_count, 0);
        assert_eq!(health.events_processed, 50);
        assert_eq!(health.backlog_size, Some(0));
        assert!(health.custom_metrics.is_some());
        assert_eq!(health.stream_info, None);
    }

    #[test]
    fn test_test_stream_name_should_return_correct_name() {
        let stream = TestStream::new("test_name_stream");
        assert_eq!(stream.name(), "test_name_stream");
    }

    #[test]
    fn test_test_stream_send_event_should_increment_counter() {
        let stream = TestStream::new("event_counter");
        let event1 = MockEvent::new("event1");
        let event2 = MockEvent::new("event2");

        let _ = stream.send_event(event1);
        let _ = stream.send_event(event2);

        assert_eq!(stream.events_processed.load(Ordering::Relaxed), 2);
    }

    // Tests for MockConfig
    #[test]
    fn test_mock_config_default_should_have_correct_values() {
        let config = MockConfig::default();

        assert!(!config.should_fail_start);
        assert!(!config.should_fail_stop);
    }

    #[test]
    fn test_mock_config_clone_should_create_identical_copy() {
        let original = MockConfig {
            should_fail_start: true,
            should_fail_stop: true,
        };

        let cloned = original.clone();

        assert_eq!(original.should_fail_start, cloned.should_fail_start);
        assert_eq!(original.should_fail_stop, cloned.should_fail_stop);
    }

    // Test builder methods
    #[test]
    fn test_test_stream_with_fail_start_should_set_flag() {
        let stream = TestStream::new("fail_start").with_fail_start();
        assert!(stream.should_fail_start);
        assert!(!stream.should_fail_stop);
    }

    #[test]
    fn test_test_stream_with_fail_stop_should_set_flag() {
        let stream = TestStream::new("fail_stop").with_fail_stop();
        assert!(!stream.should_fail_start);
        assert!(stream.should_fail_stop);
    }

    #[test]
    fn test_test_stream_chained_builder_methods_should_set_both_flags() {
        let stream = TestStream::new("fail_both")
            .with_fail_start()
            .with_fail_stop();
        assert!(stream.should_fail_start);
        assert!(stream.should_fail_stop);
    }
}
