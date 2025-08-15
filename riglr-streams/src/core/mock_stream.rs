//! Mock stream implementation for testing and examples
//!
//! This module provides MockStream and TestSource implementations that demonstrate
//! how to use the new StreamedEvent<T> wrapper with protocol-specific events
//! from riglr-solana-events.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, SystemTime};
use tokio::sync::{broadcast, RwLock};
use tokio::time::interval;
use async_trait::async_trait;

use riglr_events_core::prelude::{Event, EventKind, EventMetadata};

use crate::core::{
    Stream, StreamError, StreamHealth, StreamMetadata, 
    DynamicStreamedEvent, IntoDynamicStreamedEvent
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
            event_kinds: vec![
                EventKind::Transaction,
                EventKind::Swap,
                EventKind::Price,
            ],
            max_events: Some(100),
        }
    }
}

/// Mock stream that generates protocol-specific events for testing
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
            let interval_duration = Duration::from_millis(
                (1000.0 / config.events_per_second) as u64
            );
            let mut ticker = interval(interval_duration);
            let mut events_generated = 0;
            
            while running.load(Ordering::Relaxed) {
                ticker.tick().await;
                
                // Check if we've reached max events
                if let Some(max) = config.max_events {
                    if events_generated >= max {
                        running.store(false, Ordering::Relaxed);
                        break;
                    }
                }
                
                // Generate a random event from configured event kinds
                if let Some(event) = Self::generate_mock_event(&config.event_kinds, sequence_number) {
                    // Send the event
                    if let Err(e) = event_tx.send(Arc::new(event)) {
                        eprintln!("Failed to send mock event: {}", e);
                        break;
                    }
                    
                    sequence_number += 1;
                    events_generated += 1;
                    
                    // Update health
                    {
                        let mut h = health.write().await;
                        h.last_event_time = Some(SystemTime::now());
                        h.events_processed += 1;
                    }
                }
            }
            
            println!("Mock stream '{}' stopped after generating {} events", stream_name, events_generated);
        });
        
        Ok(())
    }
    
    /// Generate a mock event for a random event kind
    fn generate_mock_event(
        event_kinds: &[EventKind], 
        sequence_number: u64
    ) -> Option<DynamicStreamedEvent> {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let event_kind = event_kinds.get(rng.gen_range(0..event_kinds.len()))?;
        
        let stream_metadata = StreamMetadata {
            stream_source: "mock-stream".to_string(),
            received_at: SystemTime::now(),
            sequence_number: Some(sequence_number),
            custom_data: None,
        };
        
        let event: Box<dyn Event> = Box::new(MockEvent::new(
            format!("mock-{:?}-{}", event_kind, sequence_number),
            event_kind.clone(),
        ));
        
        Some(event.with_stream_metadata(stream_metadata))
    }
}

#[async_trait]
impl Stream for MockStream {
    type Event = DynamicStreamedEvent;
    type Config = MockConfig;
    
    async fn start(&mut self, config: Self::Config) -> Result<(), StreamError> {
        if self.running.load(Ordering::Relaxed) {
            return Err(StreamError::AlreadyRunning { 
                name: self.name.clone() 
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
        self.start_event_generation().await?;
        
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
    
    fn metadata_mut(&mut self) -> &mut EventMetadata {
        &mut self.metadata
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
            println!("Received event: {} from kind: {:?}", 
                event.id(), event.kind());
            
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
}