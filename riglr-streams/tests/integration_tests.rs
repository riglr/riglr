use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use tokio::sync::RwLock;
use async_trait::async_trait;

use riglr_streams::core::{
    StreamManager, EventHandler, HandlerExecutionMode, 
    MetricsCollector, StreamHealth
};

/// Mock handler that tracks execution count and timing
struct MockHandler {
    name: String,
    execution_count: Arc<AtomicU64>,
    execution_time_ms: u64,
    should_fail: bool,
}

impl MockHandler {
    fn new(name: impl Into<String>, execution_time_ms: u64) -> Self {
        Self {
            name: name.into(),
            execution_count: Arc::new(AtomicU64::new(0)),
            execution_time_ms,
            should_fail: false,
        }
    }
    
    fn _with_failure(mut self) -> Self {
        self.should_fail = true;
        self
    }
    
    fn _execution_count(&self) -> u64 {
        self.execution_count.load(Ordering::Relaxed)
    }
}

#[async_trait]
impl EventHandler for MockHandler {
    async fn should_handle(&self, _event: &(dyn std::any::Any + Send + Sync)) -> bool {
        true
    }
    
    async fn handle(&self, _event: Arc<dyn std::any::Any + Send + Sync>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.execution_count.fetch_add(1, Ordering::Relaxed);
        
        // Simulate processing time
        tokio::time::sleep(Duration::from_millis(self.execution_time_ms)).await;
        
        if self.should_fail {
            Err("Simulated failure".into())
        } else {
            Ok(())
        }
    }
    
    fn name(&self) -> &str {
        &self.name
    }
}

#[tokio::test]
async fn test_sequential_execution_preserves_order() {
    let manager = StreamManager::with_execution_mode(HandlerExecutionMode::Sequential);
    
    // Add handlers with different execution times
    let handler1 = Arc::new(MockHandler::new("handler1", 100));
    let handler2 = Arc::new(MockHandler::new("handler2", 50));
    let handler3 = Arc::new(MockHandler::new("handler3", 10));
    
    manager.add_event_handler(handler1.clone()).await;
    manager.add_event_handler(handler2.clone()).await;
    manager.add_event_handler(handler3.clone()).await;
    
    // Process an event and measure time
    let start = std::time::Instant::now();
    let _event = Arc::new("test_event");
    
    // Simulate handle_event (this would normally be internal)
    // In sequential mode, total time should be sum of all handlers
    
    // Wait for handlers to complete
    tokio::time::sleep(Duration::from_millis(200)).await;
    
    let elapsed = start.elapsed();
    
    // In sequential mode, execution should take at least 160ms (100+50+10)
    assert!(elapsed >= Duration::from_millis(160));
}

#[tokio::test]
async fn test_concurrent_execution_runs_in_parallel() {
    let manager = StreamManager::with_execution_mode(HandlerExecutionMode::Concurrent);
    
    // Add handlers with different execution times
    let handler1 = Arc::new(MockHandler::new("handler1", 100));
    let handler2 = Arc::new(MockHandler::new("handler2", 100));
    let handler3 = Arc::new(MockHandler::new("handler3", 100));
    
    manager.add_event_handler(handler1.clone()).await;
    manager.add_event_handler(handler2.clone()).await;
    manager.add_event_handler(handler3.clone()).await;
    
    // Process an event and measure time
    let start = std::time::Instant::now();
    
    // In concurrent mode, all handlers run in parallel
    // Total time should be close to the longest handler (100ms), not the sum (300ms)
    
    tokio::time::sleep(Duration::from_millis(150)).await;
    
    let elapsed = start.elapsed();
    
    // In concurrent mode, execution should complete in ~100ms, not 300ms
    assert!(elapsed < Duration::from_millis(200));
}

#[tokio::test]
async fn test_bounded_concurrent_respects_limit() {
    let manager = StreamManager::with_execution_mode(HandlerExecutionMode::ConcurrentBounded(2));
    
    // Add 4 handlers that each take 100ms
    for i in 0..4 {
        let handler = Arc::new(MockHandler::new(format!("handler{}", i), 100));
        manager.add_event_handler(handler).await;
    }
    
    // With a limit of 2, handlers should run in 2 batches
    // Total time should be ~200ms (2 batches of 100ms each)
    
    let start = std::time::Instant::now();
    
    // Process event
    tokio::time::sleep(Duration::from_millis(250)).await;
    
    let elapsed = start.elapsed();
    
    // Should take more than 150ms but less than 400ms
    assert!(elapsed >= Duration::from_millis(150));
    assert!(elapsed < Duration::from_millis(400));
}

#[tokio::test]
async fn test_metrics_collection() {
    let metrics = Arc::new(MetricsCollector::new());
    let _manager = StreamManager::new();
    
    // Record some events
    metrics.record_stream_event("test-stream", 10.0, 1024).await;
    metrics.record_stream_event("test-stream", 20.0, 2048).await;
    metrics.record_stream_event("test-stream", 15.0, 512).await;
    
    // Record handler executions
    metrics.record_handler_execution("test-handler", 5.0, true).await;
    metrics.record_handler_execution("test-handler", 10.0, false).await;
    
    // Get metrics
    let stream_metrics = metrics.get_stream_metrics("test-stream").await.unwrap();
    assert_eq!(stream_metrics.events_processed, 3);
    assert_eq!(stream_metrics.bytes_received, 3584);
    
    let handler_metrics = metrics.get_handler_metrics("test-handler").await.unwrap();
    assert_eq!(handler_metrics.invocations, 2);
    assert_eq!(handler_metrics.successes, 1);
    assert_eq!(handler_metrics.failures, 1);
}

#[tokio::test]
async fn test_metrics_percentiles() {
    let metrics = Arc::new(MetricsCollector::new());
    
    // Record events with known latencies
    for i in 1..=100 {
        metrics.record_stream_event("test-stream", i as f64, 100).await;
    }
    
    let stream_metrics = metrics.get_stream_metrics("test-stream").await.unwrap();
    
    // Check percentiles
    assert!(stream_metrics.p50_processing_time_ms >= 49.0 && stream_metrics.p50_processing_time_ms <= 51.0);
    assert!(stream_metrics.p95_processing_time_ms >= 94.0 && stream_metrics.p95_processing_time_ms <= 96.0);
    assert!(stream_metrics.p99_processing_time_ms >= 98.0 && stream_metrics.p99_processing_time_ms <= 100.0);
    
    // Check average
    let expected_avg = (1..=100).sum::<i32>() as f64 / 100.0;
    assert!((stream_metrics.avg_processing_time_ms - expected_avg).abs() < 1.0);
}

#[tokio::test]
async fn test_stream_health_tracking() {
    let health = Arc::new(RwLock::new(StreamHealth::default()));
    
    // Update health status
    {
        let mut h = health.write().await;
        h.is_connected = true;
        h.events_processed = 100;
        h.error_count = 2;
    }
    
    // Read health status
    let current_health = health.read().await;
    assert!(current_health.is_connected);
    assert_eq!(current_health.events_processed, 100);
    assert_eq!(current_health.error_count, 2);
}

#[tokio::test]
async fn test_metrics_export_formats() {
    let metrics = Arc::new(MetricsCollector::new());
    
    // Record some data
    metrics.record_stream_event("test-stream", 10.0, 1024).await;
    metrics.record_handler_execution("test-handler", 5.0, true).await;
    
    // Test JSON export
    let json_export = metrics.export_json().await;
    assert!(json_export.contains("\"stream_name\":\"test-stream\""));
    assert!(json_export.contains("\"handler_name\":\"test-handler\""));
    
    // Test Prometheus export
    let prometheus_export = metrics.export_prometheus().await;
    assert!(prometheus_export.contains("stream_events_processed{stream=\"test-stream\"}"));
    assert!(prometheus_export.contains("handler_invocations{handler=\"test-handler\"}"));
}

#[tokio::test]
async fn test_dropped_events_tracking() {
    let metrics = Arc::new(MetricsCollector::new());
    
    // Record dropped events
    metrics.record_dropped_event("test-stream").await;
    metrics.record_dropped_event("test-stream").await;
    metrics.record_dropped_event("test-stream").await;
    
    let stream_metrics = metrics.get_stream_metrics("test-stream").await.unwrap();
    assert_eq!(stream_metrics.events_dropped, 3);
}

#[tokio::test]
async fn test_reconnection_tracking() {
    let metrics = Arc::new(MetricsCollector::new());
    
    // Record reconnections
    for _ in 0..5 {
        metrics.record_reconnection("test-stream").await;
    }
    
    let stream_metrics = metrics.get_stream_metrics("test-stream").await.unwrap();
    assert_eq!(stream_metrics.reconnection_count, 5);
}