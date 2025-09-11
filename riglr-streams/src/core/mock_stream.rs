//! Mock stream implementation for testing and examples
//!
//! This module provides MockStream and TestSource implementations that demonstrate
//! how to use the new StreamedEvent<T> wrapper with protocol-specific events
//! from riglr-solana-events.

use async_trait::async_trait;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::{broadcast, RwLock};
use tokio::time::interval;

use riglr_events_core::prelude::{Event, EventKind, EventMetadata};

use crate::core::{
    DynamicStreamedEvent, IntoDynamicStreamedEvent, Stream, StreamError, StreamHealth,
    StreamMetadata,
};

/// Configuration for mock stream
#[derive(Clone, Debug)]
pub struct MockConfig {
    /// Events per second to generate
    pub events_per_second: f64,
    /// Event kinds to simulate
    pub event_kinds: Vec<EventKind>,
    /// Total events to generate (None for infinite)
    pub max_events: Option<usize>,
}

impl Default for MockConfig {
    fn default() -> Self {
        Self {
            events_per_second: 1.0,
            event_kinds: vec![EventKind::Transaction, EventKind::Swap, EventKind::Price],
            max_events: Some(100),
        }
    }
}

/// Mock stream that generates protocol-specific events for testing
#[derive(Debug)]
pub struct MockStream {
    config: MockConfig,
    event_tx: broadcast::Sender<Arc<DynamicStreamedEvent>>,
    running: Arc<AtomicBool>,
    health: Arc<RwLock<StreamHealth>>,
    name: String,
}

impl MockStream {
    /// Create a new mock stream
    pub fn new(name: impl Into<String>) -> Self {
        let (event_tx, _) = broadcast::channel(1000);

        Self {
            config: MockConfig::default(),
            event_tx,
            running: Arc::new(AtomicBool::new(false)),
            health: Arc::new(RwLock::new(StreamHealth::default())),
            name: name.into(),
        }
    }

    /// Start generating mock events
    async fn start_event_generation(&self) -> Result<(), StreamError> {
        let event_tx = self.event_tx.clone();
        let running = self.running.clone();
        let health = self.health.clone();
        let config = self.config.clone();
        let stream_name = self.name.clone();

        tokio::spawn(async move {
            let mut sequence_number = 0u64;
            let interval_duration =
                Duration::from_millis((1000.0 / config.events_per_second) as u64);
            let mut ticker = interval(interval_duration);
            let mut events_generated = 0;

            while running.load(Ordering::Relaxed) {
                let tick_result = ticker.tick().await;
                let _ = tick_result;

                // Check if we've reached max events
                if let Some(max) = config.max_events {
                    if events_generated >= max {
                        running.store(false, Ordering::Relaxed);
                        break;
                    }
                }

                // Generate a random event from configured event kinds
                let generated_event =
                    Self::generate_mock_event(&config.event_kinds, sequence_number);
                if let Some(event) = generated_event {
                    // Send the event
                    let event_arc = Arc::new(event);
                    let send_result = event_tx.send(event_arc);
                    if let Err(e) = send_result {
                        eprintln!("Failed to send mock event: {e}");
                        break;
                    }

                    sequence_number += 1;
                    events_generated += 1;

                    // Update health (keep it healthy while events are being generated)
                    {
                        let mut h = health.write().await;
                        h.last_event_time = Some(SystemTime::now());
                        h.events_processed += 1;
                    }
                }
            }

            println!(
                "Mock stream '{stream_name}' stopped after generating {events_generated} events"
            );
        });

        Ok(())
    }

    /// Generate a mock event for a random event kind
    fn generate_mock_event(
        event_kinds: &[EventKind],
        sequence_number: u64,
    ) -> Option<DynamicStreamedEvent> {
        if event_kinds.is_empty() {
            return None;
        }

        use rand::Rng;
        let mut rng = rand::rng();
        let event_kind = event_kinds.get(rng.random_range(0..event_kinds.len()))?;

        let stream_metadata = StreamMetadata {
            stream_source: "mock-stream".to_string(),
            received_at: SystemTime::now(),
            sequence_number: Some(sequence_number),
            custom_data: None,
        };

        let event: Box<dyn Event> = Box::new(MockEvent::new(
            format!("mock-{event_kind:?}-{sequence_number}"),
            event_kind.clone(),
        ));

        Some(event.with_stream_metadata(stream_metadata))
    }
}

#[async_trait]
impl Stream for MockStream {
    type Config = MockConfig;

    async fn start(&mut self, config: Self::Config) -> Result<(), StreamError> {
        if self.running.load(Ordering::Relaxed) {
            return Err(StreamError::AlreadyRunning {
                name: self.name.clone(),
            });
        }

        self.config = config;
        self.running.store(true, Ordering::Relaxed);

        // Update health
        {
            let mut health = self.health.write().await;
            health.is_connected = true;
            health.last_event_time = Some(SystemTime::now());
        }

        // Start generating events
        let generation_result = self.start_event_generation().await;
        generation_result?;

        Ok(())
    }

    async fn stop(&mut self) -> Result<(), StreamError> {
        self.running.store(false, Ordering::Relaxed);

        // Update health
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
        self.health.read().await.clone()
    }

    fn name(&self) -> &str {
        &self.name
    }
}

/// Simple mock event for testing
#[derive(Debug, Clone)]
struct MockEvent {
    metadata: EventMetadata,
}

impl MockEvent {
    fn new(id: String, kind: EventKind) -> Self {
        let metadata = EventMetadata::new(id, kind, "mock-stream".to_string());
        Self { metadata }
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

    fn metadata_mut(&mut self) -> riglr_events_core::error::EventResult<&mut EventMetadata> {
        Ok(&mut self.metadata)
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

    fn to_json(&self) -> riglr_events_core::EventResult<serde_json::Value> {
        Ok(serde_json::json!({
            "metadata": self.metadata
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::stream::StreamEvent;
    use tokio::time::{timeout, Duration};

    #[tokio::test]
    async fn test_mock_stream_generation() {
        let mut stream = MockStream::new("test-mock");
        let config = MockConfig {
            events_per_second: 10.0,
            event_kinds: vec![EventKind::Transaction],
            max_events: Some(5),
        };

        stream.start(config).await.unwrap();

        let mut rx = stream.subscribe();
        let mut events_received = 0;

        // Receive events with timeout
        while let Ok(Ok(event)) = timeout(Duration::from_secs(2), rx.recv()).await {
            println!(
                "Received event: {} from kind: {:?}",
                event.id(),
                event.kind()
            );

            // Verify it has streaming metadata
            assert!(event.stream_metadata().is_some());
            assert_eq!(event.stream_source(), Some("mock-stream"));

            events_received += 1;
            if events_received >= 5 {
                break;
            }
        }

        assert_eq!(events_received, 5);
        stream.stop().await.unwrap();
    }

    // Tests for MockConfig
    #[test]
    fn test_mock_config_default() {
        let config = MockConfig::default();
        assert_eq!(config.events_per_second, 1.0);
        assert_eq!(
            config.event_kinds,
            vec![EventKind::Transaction, EventKind::Swap, EventKind::Price]
        );
        assert_eq!(config.max_events, Some(100));
    }

    #[test]
    fn test_mock_config_clone() {
        let config = MockConfig {
            events_per_second: 5.0,
            event_kinds: vec![EventKind::Swap],
            max_events: None,
        };
        let cloned = config.clone();
        assert_eq!(cloned.events_per_second, 5.0);
        assert_eq!(cloned.event_kinds, vec![EventKind::Swap]);
        assert_eq!(cloned.max_events, None);
    }

    // Tests for MockStream constructor
    #[test]
    fn test_mock_stream_new() {
        let stream = MockStream::new("test-stream");
        assert_eq!(stream.name(), "test-stream");
        assert!(!stream.is_running());
        assert_eq!(stream.config.events_per_second, 1.0);
    }

    #[test]
    fn test_mock_stream_new_with_string() {
        let stream = MockStream::new("another-stream".to_string());
        assert_eq!(stream.name(), "another-stream");
    }

    // Tests for Stream trait implementation - start method
    #[tokio::test]
    async fn test_mock_stream_start_when_not_running_should_succeed() {
        let mut stream = MockStream::new("test");
        let config = MockConfig {
            events_per_second: 1.0,
            event_kinds: vec![EventKind::Transaction],
            max_events: Some(1),
        };

        let result = stream.start(config).await;
        assert!(result.is_ok());
        assert!(stream.is_running());

        let health = stream.health().await;
        assert!(health.is_connected);
        assert!(health.last_event_time.is_some());

        stream.stop().await.unwrap();
    }

    #[tokio::test]
    async fn test_mock_stream_start_when_already_running_should_return_error() {
        let mut stream = MockStream::new("test");
        let config = MockConfig::default();

        stream.start(config.clone()).await.unwrap();

        // Try to start again - should fail
        let result = stream.start(config).await;
        assert!(result.is_err());

        if let Err(StreamError::AlreadyRunning { name }) = result {
            assert_eq!(name, "test");
        } else {
            panic!("Expected AlreadyRunning error");
        }

        stream.stop().await.unwrap();
    }

    // Tests for Stream trait implementation - stop method
    #[tokio::test]
    async fn test_mock_stream_stop_should_succeed() {
        let mut stream = MockStream::new("test");
        stream.start(MockConfig::default()).await.unwrap();

        assert!(stream.is_running());

        let result = stream.stop().await;
        assert!(result.is_ok());
        assert!(!stream.is_running());

        let health = stream.health().await;
        assert!(!health.is_connected);
    }

    #[tokio::test]
    async fn test_mock_stream_stop_when_not_running_should_succeed() {
        let mut stream = MockStream::new("test");

        let result = stream.stop().await;
        assert!(result.is_ok());
        assert!(!stream.is_running());
    }

    // Tests for subscribe method
    #[tokio::test]
    async fn test_mock_stream_subscribe() {
        let stream = MockStream::new("test");
        let _rx = stream.subscribe();
        // If we can subscribe without error, the test passes
    }

    #[tokio::test]
    async fn test_mock_stream_multiple_subscribers() {
        let mut stream = MockStream::new("test");
        let _rx1 = stream.subscribe();
        let _rx2 = stream.subscribe();

        let config = MockConfig {
            events_per_second: 100.0,
            event_kinds: vec![EventKind::Transaction],
            max_events: Some(1),
        };

        stream.start(config).await.unwrap();
        stream.stop().await.unwrap();
    }

    // Tests for health method
    #[tokio::test]
    async fn test_mock_stream_health_initial_state() {
        let stream = MockStream::new("test");
        let health = stream.health().await;

        assert!(!health.is_connected);
        assert_eq!(health.events_processed, 0);
        assert!(health.last_event_time.is_none());
    }

    // Tests for name method
    #[test]
    fn test_mock_stream_name() {
        let stream = MockStream::new("my-test-stream");
        assert_eq!(stream.name(), "my-test-stream");
    }

    // Tests for is_running method
    #[tokio::test]
    async fn test_mock_stream_is_running_states() {
        let mut stream = MockStream::new("test");

        // Initially not running
        assert!(!stream.is_running());

        // After start
        stream.start(MockConfig::default()).await.unwrap();
        assert!(stream.is_running());

        // After stop
        stream.stop().await.unwrap();
        assert!(!stream.is_running());
    }

    // Tests for generate_mock_event function
    #[test]
    fn test_generate_mock_event_with_single_kind() {
        let event_kinds = vec![EventKind::Transaction];
        let event = MockStream::generate_mock_event(&event_kinds, 42);

        assert!(event.is_some());
        let event = event.unwrap();
        assert_eq!(event.kind(), &EventKind::Transaction);
        assert!(event.id().contains("Transaction"));
        assert!(event.id().contains("42"));

        let stream_meta = event.stream_metadata().unwrap();
        assert_eq!(stream_meta.stream_source, "mock-stream");
        assert_eq!(stream_meta.sequence_number, Some(42));
        assert!(stream_meta.custom_data.is_none());
    }

    #[test]
    fn test_generate_mock_event_with_multiple_kinds() {
        let event_kinds = vec![EventKind::Transaction, EventKind::Swap, EventKind::Price];
        let event = MockStream::generate_mock_event(&event_kinds, 123);

        assert!(event.is_some());
        let event = event.unwrap();
        assert!(event_kinds.contains(event.kind()));
        assert!(event.id().contains("123"));
    }

    #[test]
    fn test_generate_mock_event_with_empty_kinds_should_return_none() {
        let event_kinds = vec![];
        let event = MockStream::generate_mock_event(&event_kinds, 0);
        assert!(event.is_none());
    }

    // Tests for MockEvent
    #[test]
    fn test_mock_event_new() {
        let event = MockEvent::new("test-id".to_string(), EventKind::Swap);
        assert_eq!(event.id(), "test-id");
        assert_eq!(event.kind(), &EventKind::Swap);
        assert_eq!(event.metadata().source, "mock-stream");
    }

    #[test]
    fn test_mock_event_metadata() {
        let event = MockEvent::new("test-id".to_string(), EventKind::Price);
        let metadata = event.metadata();
        assert_eq!(metadata.id, "test-id");
        assert_eq!(metadata.kind, EventKind::Price);
        assert_eq!(metadata.source, "mock-stream");
    }

    #[test]
    fn test_mock_event_metadata_mut() {
        let mut event = MockEvent::new("test-id".to_string(), EventKind::Transaction);
        {
            let metadata = event.metadata_mut().unwrap();
            metadata.source = "modified-source".to_string();
        }
        assert_eq!(event.metadata().source, "modified-source");
    }

    #[test]
    fn test_mock_event_as_any() {
        let event = MockEvent::new("test".to_string(), EventKind::Swap);
        let any_ref = event.as_any();
        assert!(any_ref.downcast_ref::<MockEvent>().is_some());
    }

    #[test]
    fn test_mock_event_as_any_mut() {
        let mut event = MockEvent::new("test".to_string(), EventKind::Swap);
        let any_ref = event.as_any_mut();
        assert!(any_ref.downcast_mut::<MockEvent>().is_some());
    }

    #[test]
    fn test_mock_event_clone_boxed() {
        let event = MockEvent::new("test".to_string(), EventKind::Transaction);
        let cloned = event.clone_boxed();
        assert_eq!(cloned.id(), "test");
        assert_eq!(cloned.kind(), &EventKind::Transaction);
    }

    #[test]
    fn test_mock_event_to_json() {
        let event = MockEvent::new("test-json".to_string(), EventKind::Price);
        let json_result = event.to_json();

        assert!(json_result.is_ok());
        let json = json_result.unwrap();
        assert!(json.get("metadata").is_some());
    }

    #[test]
    fn test_mock_event_clone() {
        let event = MockEvent::new("clone-test".to_string(), EventKind::Swap);
        let cloned = event.clone();

        assert_eq!(event.id(), cloned.id());
        assert_eq!(event.kind(), cloned.kind());
        assert_eq!(event.metadata().source, cloned.metadata().source);
    }

    // Integration tests for event generation and streaming
    #[tokio::test]
    async fn test_mock_stream_with_zero_max_events() {
        let mut stream = MockStream::new("zero-events");
        let config = MockConfig {
            events_per_second: 100.0,
            event_kinds: vec![EventKind::Transaction],
            max_events: Some(0),
        };

        stream.start(config).await.unwrap();

        let mut rx = stream.subscribe();

        // Should not receive any events
        let result = timeout(Duration::from_millis(100), rx.recv()).await;
        assert!(result.is_err()); // Timeout expected

        stream.stop().await.unwrap();
    }

    #[tokio::test]
    async fn test_mock_stream_with_none_max_events() {
        let mut stream = MockStream::new("infinite-events");
        let config = MockConfig {
            events_per_second: 100.0,
            event_kinds: vec![EventKind::Transaction],
            max_events: None, // Infinite
        };

        stream.start(config).await.unwrap();

        let mut rx = stream.subscribe();
        let mut events_received = 0;

        // Receive a few events then stop
        while let Ok(Ok(_event)) = timeout(Duration::from_millis(50), rx.recv()).await {
            events_received += 1;
            if events_received >= 3 {
                break;
            }
        }

        assert!(events_received >= 1);
        stream.stop().await.unwrap();
    }

    #[tokio::test]
    async fn test_mock_stream_high_event_rate() {
        let mut stream = MockStream::new("high-rate");
        let config = MockConfig {
            events_per_second: 1000.0, // Very high rate
            event_kinds: vec![EventKind::Swap],
            max_events: Some(10),
        };

        stream.start(config).await.unwrap();

        let mut rx = stream.subscribe();
        let mut events_received = 0;

        while let Ok(Ok(_event)) = timeout(Duration::from_secs(1), rx.recv()).await {
            events_received += 1;
            if events_received >= 10 {
                break;
            }
        }

        assert_eq!(events_received, 10);
        stream.stop().await.unwrap();
    }

    #[tokio::test]
    async fn test_mock_stream_low_event_rate() {
        let mut stream = MockStream::new("low-rate");
        let config = MockConfig {
            events_per_second: 0.5, // Very low rate - one event every 2 seconds
            event_kinds: vec![EventKind::Price],
            max_events: Some(1),
        };

        stream.start(config).await.unwrap();

        let mut rx = stream.subscribe();

        // Should receive one event within reasonable time
        let result = timeout(Duration::from_secs(3), rx.recv()).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_ok());

        stream.stop().await.unwrap();
    }

    #[tokio::test]
    async fn test_mock_stream_sequence_numbers() {
        let mut stream = MockStream::new("sequence-test");
        let config = MockConfig {
            events_per_second: 100.0,
            event_kinds: vec![EventKind::Transaction],
            max_events: Some(3),
        };

        stream.start(config).await.unwrap();

        let mut rx = stream.subscribe();
        let mut sequence_numbers = Vec::new();

        while let Ok(Ok(event)) = timeout(Duration::from_secs(1), rx.recv()).await {
            if let Some(seq) = event.stream_metadata().unwrap().sequence_number {
                sequence_numbers.push(seq);
            }
            if sequence_numbers.len() >= 3 {
                break;
            }
        }

        assert_eq!(sequence_numbers, vec![0, 1, 2]);
        stream.stop().await.unwrap();
    }

    #[tokio::test]
    async fn test_mock_stream_health_updates() {
        let mut stream = MockStream::new("health-test");
        let config = MockConfig {
            events_per_second: 50.0,
            event_kinds: vec![EventKind::Swap],
            max_events: Some(2),
        };

        // Initial health
        let initial_health = stream.health().await;
        assert!(!initial_health.is_connected);
        assert_eq!(initial_health.events_processed, 0);

        stream.start(config).await.unwrap();

        // Health after start
        let started_health = stream.health().await;
        assert!(started_health.is_connected);

        // Wait for events to be processed
        let mut rx = stream.subscribe();
        let mut events_received = 0;
        while let Ok(Ok(_event)) = timeout(Duration::from_secs(1), rx.recv()).await {
            events_received += 1;
            if events_received >= 2 {
                break;
            }
        }

        // Health should show processed events
        let processed_health = stream.health().await;
        assert!(processed_health.events_processed >= 2);
        assert!(processed_health.last_event_time.is_some());

        stream.stop().await.unwrap();

        // Health after stop
        let stopped_health = stream.health().await;
        assert!(!stopped_health.is_connected);
    }
}
