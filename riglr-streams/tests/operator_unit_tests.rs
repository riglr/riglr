//! Unit tests for stream operators
//! Tests each operator (map, filter, batch) in isolation using MockStream

use std::time::Duration;
use tokio::time::timeout;
use riglr_events_core::{Event, EventKind};
use riglr_streams::core::StreamEvent;
use riglr_streams::core::{
    mock_stream::{MockStream, MockConfig},
    Stream,
};

#[tokio::test]
async fn test_mock_stream_basic() {
    // Create a mock stream that generates 5 events
    let mut stream = MockStream::new("test-basic");
    let config = MockConfig {
        events_per_second: 10.0,
        event_kinds: vec![EventKind::Swap],
        max_events: Some(5),
    };
    
    // Start the stream
    stream.start(config).await.unwrap();
    
    // Check if stream is running
    assert!(stream.is_running(), "Stream should be running");
    
    let mut rx = stream.subscribe();
    let mut events_received = 0;
    
    // Collect events
    while let Ok(Ok(event)) = timeout(Duration::from_secs(2), rx.recv()).await {
        println!("Received event: {}", event.inner.id());
        events_received += 1;
        if events_received >= 5 {
            break;
        }
    }
    
    assert_eq!(events_received, 5, "Should receive all 5 events");
    
    // Check health
    let health = stream.health().await;
    assert!(health.is_connected);
    assert_eq!(health.events_processed, 5);
    
    stream.stop().await.unwrap();
}

#[tokio::test]
async fn test_multiple_protocols() {
    // Create a mock stream with multiple protocols
    let mut stream = MockStream::new("test-multi-protocol");
    let config = MockConfig {
        events_per_second: 20.0,
        event_kinds: vec![
            EventKind::Swap,
            EventKind::Liquidity,
            EventKind::Transfer,
        ],
        max_events: Some(15),
    };
    
    stream.start(config).await.unwrap();
    
    let mut rx = stream.subscribe();
    let mut protocol_counts = std::collections::HashMap::new();
    let mut total_events = 0;
    
    // Collect events and count by protocol
    while let Ok(Ok(event)) = timeout(Duration::from_secs(2), rx.recv()).await {
        let protocol = event.inner.kind();
        *protocol_counts.entry(format!("{:?}", protocol)).or_insert(0) += 1;
        total_events += 1;
        
        if total_events >= 15 {
            break;
        }
    }
    
    assert_eq!(total_events, 15, "Should receive all 15 events");
    
    // Should have events from multiple protocols
    assert!(protocol_counts.len() > 1, "Should have events from multiple protocols");
    
    stream.stop().await.unwrap();
}

#[tokio::test]
async fn test_stream_health_metrics() {
    let mut stream = MockStream::new("test-health");
    let config = MockConfig {
        events_per_second: 10.0,
        event_kinds: vec![EventKind::Swap],
        max_events: Some(10),
    };
    
    // Health before start
    let health_before = stream.health().await;
    assert!(!health_before.is_connected);
    assert_eq!(health_before.events_processed, 0);
    
    stream.start(config).await.unwrap();
    
    // Health after start
    let health_running = stream.health().await;
    assert!(health_running.is_connected);
    
    // Let it generate some events
    tokio::time::sleep(Duration::from_millis(1500)).await;
    
    // Check health metrics
    let health_after = stream.health().await;
    assert!(health_after.is_connected);
    assert!(health_after.events_processed > 0);
    assert!(health_after.last_event_time.is_some());
    
    stream.stop().await.unwrap();
    
    // Health after stop
    let health_stopped = stream.health().await;
    assert!(!health_stopped.is_connected);
}

#[tokio::test]
async fn test_stream_subscribe_multiple() {
    let mut stream = MockStream::new("test-multi-sub");
    let config = MockConfig {
        events_per_second: 10.0,
        event_kinds: vec![EventKind::Swap],
        max_events: Some(5),
    };
    
    stream.start(config).await.unwrap();
    
    // Create multiple subscribers
    let mut rx1 = stream.subscribe();
    let mut rx2 = stream.subscribe();
    
    let mut events1 = 0;
    let mut events2 = 0;
    
    // Collect from first subscriber
    while let Ok(Ok(_event)) = timeout(Duration::from_millis(100), rx1.recv()).await {
        events1 += 1;
        if events1 >= 3 {
            break;
        }
    }
    
    // Collect from second subscriber
    while let Ok(Ok(_event)) = timeout(Duration::from_millis(100), rx2.recv()).await {
        events2 += 1;
        if events2 >= 3 {
            break;
        }
    }
    
    // Both subscribers should receive events
    assert!(events1 > 0, "First subscriber should receive events");
    assert!(events2 > 0, "Second subscriber should receive events");
    
    stream.stop().await.unwrap();
}

#[tokio::test]
async fn test_stream_restart() {
    let mut stream = MockStream::new("test-restart");
    let config1 = MockConfig {
        events_per_second: 5.0,
        event_kinds: vec![EventKind::Swap],
        max_events: Some(3),
    };
    
    // Start stream
    stream.start(config1.clone()).await.unwrap();
    assert!(stream.is_running());
    
    // Wait for completion
    tokio::time::sleep(Duration::from_millis(800)).await;
    
    // Stop stream
    stream.stop().await.unwrap();
    assert!(!stream.is_running());
    
    // Restart with different config
    let config2 = MockConfig {
        events_per_second: 10.0,
        event_kinds: vec![EventKind::Liquidity],
        max_events: Some(5),
    };
    
    stream.start(config2).await.unwrap();
    assert!(stream.is_running());
    
    let mut rx = stream.subscribe();
    let mut events_after_restart = 0;
    
    // Verify we get events with new config
    while let Ok(Ok(event)) = timeout(Duration::from_millis(200), rx.recv()).await {
        assert_eq!(event.inner.kind(), &EventKind::Liquidity);
        events_after_restart += 1;
        if events_after_restart >= 2 {
            break;
        }
    }
    
    assert!(events_after_restart > 0, "Should receive events after restart");
    
    stream.stop().await.unwrap();
}

#[tokio::test]
async fn test_infinite_stream() {
    let mut stream = MockStream::new("test-infinite");
    let config = MockConfig {
        events_per_second: 20.0,
        event_kinds: vec![EventKind::Swap],
        max_events: None, // Infinite stream
    };
    
    stream.start(config).await.unwrap();
    
    let mut rx = stream.subscribe();
    let mut events = 0;
    
    // Collect events for 1 second
    let start = std::time::Instant::now();
    while start.elapsed() < Duration::from_secs(1) {
        if let Ok(Ok(_event)) = timeout(Duration::from_millis(100), rx.recv()).await {
            events += 1;
        }
    }
    
    // Should receive approximately 20 events (allowing for some variance)
    assert!(events >= 15 && events <= 25, 
            "Should receive approximately 20 events, got {}", events);
    
    // Stream should still be running
    assert!(stream.is_running());
    
    stream.stop().await.unwrap();
}

#[tokio::test]
async fn test_event_metadata() {
    let mut stream = MockStream::new("test-metadata");
    let config = MockConfig {
        events_per_second: 5.0,
        event_kinds: vec![EventKind::Swap],
        max_events: Some(2),
    };
    
    stream.start(config).await.unwrap();
    
    let mut rx = stream.subscribe();
    
    // Get an event and check its metadata
    if let Ok(Ok(event)) = timeout(Duration::from_secs(1), rx.recv()).await {
        // Check stream metadata - it's a method on Arc<DynamicStreamedEvent>
        let metadata = event.stream_meta();
        assert_eq!(metadata.stream_source, "mock-stream");
        assert!(metadata.sequence_number.is_some());
        
        // Check event properties
        assert!(event.inner.id().starts_with("mock-"));
        assert_eq!(event.inner.kind(), &EventKind::Swap);
    } else {
        panic!("Failed to receive event");
    }
    
    stream.stop().await.unwrap();
}

#[tokio::test]
async fn test_already_running_error() {
    let mut stream = MockStream::new("test-already-running");
    let config = MockConfig::default();
    
    // Start stream
    stream.start(config.clone()).await.unwrap();
    
    // Try to start again - should fail
    let result = stream.start(config).await;
    assert!(result.is_err());
    
    if let Err(e) = result {
        // Check that it's an AlreadyRunning error
        let error_string = format!("{:?}", e);
        assert!(error_string.contains("AlreadyRunning"));
    }
    
    stream.stop().await.unwrap();
}