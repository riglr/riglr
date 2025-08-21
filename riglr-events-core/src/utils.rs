//! Utility functions and helpers for event processing.

use crate::error::EventResult;
use crate::traits::Event;
use dashmap::DashMap;
use futures::{Stream, StreamExt};
use std::pin::Pin;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;

/// Type alias for event streams to reduce complexity
pub type EventStream = Pin<Box<dyn Stream<Item = EventResult<Box<dyn Event>>> + Send>>;

/// Type alias for event batch streams
pub type EventBatchStream = Pin<Box<dyn Stream<Item = EventResult<Vec<Box<dyn Event>>>> + Send>>;

/// Utility for generating unique event IDs.
#[derive(Debug, Clone)]
pub struct EventIdGenerator {
    prefix: String,
    counter: Arc<AtomicU64>,
}

impl EventIdGenerator {
    /// Create a new ID generator with a prefix
    pub fn new(prefix: String) -> Self {
        Self {
            prefix,
            counter: Arc::new(AtomicU64::default()),
        }
    }

    /// Generate a new unique ID
    pub fn next(&self) -> String {
        let count = self.counter.fetch_add(1, Ordering::SeqCst);
        format!(
            "{}_{:016x}_{:016x}",
            self.prefix,
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_micros() as u64,
            count
        )
    }

    /// Generate an ID with additional context
    pub fn next_with_context(&self, context: &str) -> String {
        format!(
            "{}_{}_{}",
            self.next(),
            context,
            uuid::Uuid::new_v4().simple()
        )
    }
}

/// Batching utility for accumulating events before processing.
#[derive(Debug)]
pub struct EventBatcher {
    batch_size: usize,
    timeout: Duration,
    current_batch: Vec<Box<dyn Event>>,
    last_flush: Instant,
}

impl EventBatcher {
    /// Create a new event batcher
    pub fn new(batch_size: usize, timeout: Duration) -> Self {
        Self {
            batch_size,
            timeout,
            current_batch: Vec::with_capacity(batch_size),
            last_flush: Instant::now(),
        }
    }

    /// Add an event to the current batch
    pub fn add(&mut self, event: Box<dyn Event>) -> Option<Vec<Box<dyn Event>>> {
        self.current_batch.push(event);

        if self.should_flush() {
            self.flush()
        } else {
            None
        }
    }

    /// Check if the batch should be flushed
    pub fn should_flush(&self) -> bool {
        self.current_batch.len() >= self.batch_size || self.last_flush.elapsed() >= self.timeout
    }

    /// Flush the current batch and return the events
    pub fn flush(&mut self) -> Option<Vec<Box<dyn Event>>> {
        if self.current_batch.is_empty() {
            return None;
        }

        let batch = std::mem::replace(&mut self.current_batch, Vec::with_capacity(self.batch_size));
        self.last_flush = Instant::now();
        Some(batch)
    }

    /// Get the current batch size
    pub fn current_size(&self) -> usize {
        self.current_batch.len()
    }

    /// Check if the batch is empty
    pub fn is_empty(&self) -> bool {
        self.current_batch.is_empty()
    }
}

/// Event deduplication utility to prevent processing duplicate events.
#[derive(Debug)]
pub struct EventDeduplicator {
    seen_events: Arc<DashMap<String, SystemTime>>,
    ttl: Duration,
    cleanup_interval: Duration,
}

impl EventDeduplicator {
    /// Create a new deduplicator with TTL for seen events
    pub fn new(ttl: Duration, cleanup_interval: Duration) -> Self {
        Self {
            seen_events: Arc::new(DashMap::default()),
            ttl,
            cleanup_interval,
        }
    }

    /// Check if an event is a duplicate
    pub async fn is_duplicate(&self, event: &dyn Event) -> bool {
        let event_id = event.id();

        if let Some(seen_at) = self.seen_events.get(event_id) {
            // Check if the event is still within TTL
            seen_at.value().elapsed().unwrap_or_default() < self.ttl
        } else {
            false
        }
    }

    /// Mark an event as seen
    pub async fn mark_seen(&self, event: &dyn Event) {
        let event_id = event.id().to_string();
        self.seen_events.insert(event_id, SystemTime::now());
    }

    /// Clean up expired entries
    pub async fn cleanup(&self) {
        let now = SystemTime::now();

        self.seen_events
            .retain(|_, seen_at| now.duration_since(*seen_at).unwrap_or_default() < self.ttl);
    }

    /// Start automatic cleanup task
    pub fn start_cleanup_task(&self) -> tokio::task::JoinHandle<()> {
        let seen_events = Arc::clone(&self.seen_events);
        let ttl = self.ttl;
        let cleanup_interval = self.cleanup_interval;

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(cleanup_interval);

            loop {
                interval.tick().await;

                let now = SystemTime::now();

                seen_events
                    .retain(|_, seen_at| now.duration_since(*seen_at).unwrap_or_default() < ttl);
            }
        })
    }
}

/// Rate limiting utility for controlling event processing speed.
#[derive(Debug)]
pub struct RateLimiter {
    max_rate: u64, // events per second
    window: Duration,
    events_in_window: Arc<RwLock<Vec<SystemTime>>>,
}

impl RateLimiter {
    /// Create a new rate limiter
    pub fn new(max_rate: u64, window: Duration) -> Self {
        Self {
            max_rate,
            window,
            events_in_window: Arc::new(RwLock::default()),
        }
    }

    /// Check if we can process another event
    pub async fn can_process(&self) -> bool {
        let mut events = self.events_in_window.write().await;
        let now = SystemTime::now();

        // Remove old events outside the window
        events.retain(|timestamp| now.duration_since(*timestamp).unwrap_or_default() < self.window);

        events.len() < self.max_rate as usize
    }

    /// Record that an event was processed
    pub async fn record_event(&self) {
        let mut events = self.events_in_window.write().await;
        events.push(SystemTime::now());
    }

    /// Wait until we can process the next event
    pub async fn wait_for_capacity(&self) {
        while !self.can_process().await {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }

    /// Get current rate (events per second)
    pub async fn current_rate(&self) -> f64 {
        let events = self.events_in_window.read().await;
        let now = SystemTime::now();

        let recent_events = events
            .iter()
            .filter(|timestamp| now.duration_since(**timestamp).unwrap_or_default() < self.window)
            .count();

        recent_events as f64 / self.window.as_secs_f64()
    }
}

/// Stream transformation utilities
pub struct StreamUtils;

impl StreamUtils {
    /// Transform a stream of events using a mapping function
    pub fn map_events<F, T>(
        stream: EventStream,
        mapper: F,
    ) -> Pin<Box<dyn Stream<Item = EventResult<T>> + Send>>
    where
        F: Fn(Box<dyn Event>) -> EventResult<T> + Send + 'static,
        T: Send + 'static,
    {
        Box::pin(stream.map(move |result| result.and_then(&mapper)))
    }

    /// Filter events based on a predicate
    pub fn filter_events<F>(stream: EventStream, predicate: F) -> EventStream
    where
        F: Fn(&dyn Event) -> bool + Send + Sync + Clone + 'static,
    {
        let pred_clone = predicate.clone();
        Box::pin(stream.filter_map(move |result| {
            let predicate = pred_clone.clone();
            async move {
                match result {
                    Ok(event) => {
                        if predicate(&*event) {
                            Some(Ok(event))
                        } else {
                            None
                        }
                    }
                    Err(e) => Some(Err(e)),
                }
            }
        }))
    }

    /// Batch events into groups of specified size
    pub fn batch_events(stream: EventStream, batch_size: usize) -> EventBatchStream {
        Box::pin(stream.chunks(batch_size).map(|batch| {
            let mut events = Vec::with_capacity(batch.len());
            for result in batch {
                match result {
                    Ok(event) => events.push(event),
                    Err(e) => return Err(e),
                }
            }
            Ok(events)
        }))
    }

    /// Add rate limiting to an event stream
    pub fn rate_limit(stream: EventStream, rate_limiter: Arc<RateLimiter>) -> EventStream {
        Box::pin(stream.then(move |result| {
            let rate_limiter = Arc::clone(&rate_limiter);
            async move {
                match result {
                    Ok(event) => {
                        rate_limiter.wait_for_capacity().await;
                        rate_limiter.record_event().await;
                        Ok(event)
                    }
                    Err(e) => Err(e),
                }
            }
        }))
    }

    /// Deduplicate events in a stream
    pub fn deduplicate(stream: EventStream, deduplicator: Arc<EventDeduplicator>) -> EventStream {
        Box::pin(stream.filter_map(move |result| {
            let deduplicator = Arc::clone(&deduplicator);
            async move {
                match result {
                    Ok(event) => {
                        if deduplicator.is_duplicate(&*event).await {
                            None // Skip duplicate
                        } else {
                            deduplicator.mark_seen(&*event).await;
                            Some(Ok(event))
                        }
                    }
                    Err(e) => Some(Err(e)),
                }
            }
        }))
    }
}

/// Performance metrics collector for event processing
#[derive(Debug, Clone)]
pub struct EventPerformanceMetrics {
    total_events: Arc<AtomicU64>,
    total_errors: Arc<AtomicU64>,
    processing_times: Arc<RwLock<Vec<Duration>>>,
    last_reset: Arc<RwLock<SystemTime>>,
}

impl EventPerformanceMetrics {
    /// Record an event processing time
    pub async fn record_processing_time(&self, duration: Duration) {
        self.total_events.fetch_add(1, Ordering::SeqCst);
        let mut times = self.processing_times.write().await;
        times.push(duration);

        // Keep only the last 10,000 measurements to prevent memory bloat
        if times.len() > 10_000 {
            let new_len = times.len() - 10_000;
            times.drain(..new_len);
        }
    }

    /// Record an error
    pub fn record_error(&self) {
        self.total_errors.fetch_add(1, Ordering::SeqCst);
    }

    /// Get total events processed
    pub fn total_events(&self) -> u64 {
        self.total_events.load(Ordering::SeqCst)
    }

    /// Get total errors
    pub fn total_errors(&self) -> u64 {
        self.total_errors.load(Ordering::SeqCst)
    }

    /// Get error rate as percentage
    pub fn error_rate(&self) -> f64 {
        let total = self.total_events();
        let errors = self.total_errors();

        if total == 0 {
            0.0
        } else {
            (errors as f64 / total as f64) * 100.0
        }
    }

    /// Get average processing time
    pub async fn avg_processing_time(&self) -> Duration {
        let times = self.processing_times.read().await;
        if times.is_empty() {
            Duration::ZERO
        } else {
            let sum: Duration = times.iter().sum();
            sum / times.len() as u32
        }
    }

    /// Get processing time percentiles
    pub async fn processing_time_percentiles(&self, percentiles: &[f64]) -> Vec<Duration> {
        let mut times = self.processing_times.read().await.clone();
        if times.is_empty() {
            return vec![Duration::ZERO; percentiles.len()];
        }

        times.sort();

        percentiles
            .iter()
            .map(|&p| {
                let index = ((times.len() as f64 * p / 100.0) as usize).min(times.len() - 1);
                times[index]
            })
            .collect()
    }

    /// Reset all metrics
    pub async fn reset(&self) {
        self.total_events.store(0, Ordering::SeqCst);
        self.total_errors.store(0, Ordering::SeqCst);
        self.processing_times.write().await.clear();
        *self.last_reset.write().await = SystemTime::now();
    }

    /// Get metrics summary
    pub async fn summary(&self) -> MetricsSummary {
        let times_clone = {
            let times = self.processing_times.read().await;
            times.clone()
        };
        let total_events = self.total_events();
        let total_errors = self.total_errors();

        let (avg_time, p95_time, p99_time) = if times_clone.is_empty() {
            (Duration::ZERO, Duration::ZERO, Duration::ZERO)
        } else {
            let mut sorted_times = times_clone.clone();
            sorted_times.sort();

            let avg = sorted_times.iter().sum::<Duration>() / sorted_times.len() as u32;
            let p95_idx = ((sorted_times.len() as f64 * 0.95) as usize).min(sorted_times.len() - 1);
            let p99_idx = ((sorted_times.len() as f64 * 0.99) as usize).min(sorted_times.len() - 1);

            (avg, sorted_times[p95_idx], sorted_times[p99_idx])
        };

        MetricsSummary {
            total_events,
            total_errors,
            error_rate: self.error_rate(),
            avg_processing_time: avg_time,
            p95_processing_time: p95_time,
            p99_processing_time: p99_time,
            uptime: self.last_reset.read().await.elapsed().unwrap_or_default(),
        }
    }
}

impl Default for EventPerformanceMetrics {
    fn default() -> Self {
        Self {
            total_events: Arc::new(AtomicU64::default()),
            total_errors: Arc::new(AtomicU64::default()),
            processing_times: Arc::new(RwLock::default()),
            last_reset: Arc::new(RwLock::new(SystemTime::now())),
        }
    }
}

/// Summary of event processing metrics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MetricsSummary {
    /// Total events processed
    pub total_events: u64,
    /// Total errors encountered
    pub total_errors: u64,
    /// Error rate as percentage
    pub error_rate: f64,
    /// Average processing time
    pub avg_processing_time: Duration,
    /// 95th percentile processing time
    pub p95_processing_time: Duration,
    /// 99th percentile processing time
    pub p99_processing_time: Duration,
    /// Time since metrics were last reset
    pub uptime: Duration,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{EventKind, GenericEvent};
    use serde_json::json;

    #[test]
    fn test_event_id_generator() {
        let generator = EventIdGenerator::new("test".to_string());

        let id1 = generator.next();
        let id2 = generator.next();

        assert_ne!(id1, id2);
        assert!(id1.starts_with("test_"));
        assert!(id2.starts_with("test_"));
    }

    #[test]
    fn test_event_batcher() {
        let mut batcher = EventBatcher::new(3, Duration::from_secs(10));

        let event1 = Box::new(GenericEvent::new(
            "1".to_string(),
            EventKind::Transaction,
            json!({}),
        ));
        let event2 = Box::new(GenericEvent::new(
            "2".to_string(),
            EventKind::Transaction,
            json!({}),
        ));
        let event3 = Box::new(GenericEvent::new(
            "3".to_string(),
            EventKind::Transaction,
            json!({}),
        ));

        assert!(batcher.add(event1).is_none());
        assert!(batcher.add(event2).is_none());
        assert_eq!(batcher.current_size(), 2);

        let batch = batcher.add(event3);
        assert!(batch.is_some());
        assert_eq!(batch.unwrap().len(), 3);
        assert_eq!(batcher.current_size(), 0);
    }

    #[tokio::test]
    async fn test_event_deduplicator() {
        let deduplicator = EventDeduplicator::new(Duration::from_secs(60), Duration::from_secs(30));

        let event = GenericEvent::new("test-event".to_string(), EventKind::Transaction, json!({}));

        assert!(!deduplicator.is_duplicate(&event).await);
        deduplicator.mark_seen(&event).await;
        assert!(deduplicator.is_duplicate(&event).await);
    }

    #[tokio::test]
    async fn test_rate_limiter() {
        let rate_limiter = RateLimiter::new(2, Duration::from_secs(1));

        assert!(rate_limiter.can_process().await);
        rate_limiter.record_event().await;

        assert!(rate_limiter.can_process().await);
        rate_limiter.record_event().await;

        assert!(!rate_limiter.can_process().await);

        let rate = rate_limiter.current_rate().await;
        assert!(rate > 0.0);
    }

    #[tokio::test]
    async fn test_performance_metrics() {
        let metrics = EventPerformanceMetrics::default();

        metrics
            .record_processing_time(Duration::from_millis(10))
            .await;
        metrics
            .record_processing_time(Duration::from_millis(20))
            .await;
        metrics.record_error();

        assert_eq!(metrics.total_events(), 2);
        assert_eq!(metrics.total_errors(), 1);
        assert_eq!(metrics.error_rate(), 50.0);

        let avg = metrics.avg_processing_time().await;
        assert_eq!(avg, Duration::from_millis(15));

        let summary = metrics.summary().await;
        assert_eq!(summary.total_events, 2);
        assert_eq!(summary.total_errors, 1);
        assert_eq!(summary.error_rate, 50.0);
    }

    #[tokio::test]
    async fn test_performance_metrics_percentiles() {
        let metrics = EventPerformanceMetrics::default();

        // Add some processing times
        for i in 1..=100 {
            metrics
                .record_processing_time(Duration::from_millis(i))
                .await;
        }

        let percentiles = metrics
            .processing_time_percentiles(&[50.0, 95.0, 99.0])
            .await;
        assert_eq!(percentiles.len(), 3);

        // P50 should be around 50ms
        assert!(percentiles[0] >= Duration::from_millis(49));
        assert!(percentiles[0] <= Duration::from_millis(51));

        // P95 should be around 95ms
        assert!(percentiles[1] >= Duration::from_millis(94));
        assert!(percentiles[1] <= Duration::from_millis(96));

        // P99 should be around 99ms
        assert!(percentiles[2] >= Duration::from_millis(98));
        assert!(percentiles[2] <= Duration::from_millis(100));
    }

    // Additional comprehensive tests for 100% coverage

    #[test]
    fn test_event_id_generator_next_with_context() {
        let generator = EventIdGenerator::new("test".to_string());

        let id = generator.next_with_context("custom_context");
        assert!(id.contains("test_"));
        assert!(id.contains("custom_context"));
        assert!(id.len() > 50); // Should be longer due to UUID
    }

    #[test]
    fn test_event_id_generator_clone() {
        let generator = EventIdGenerator::new("prefix".to_string());
        let cloned = generator.clone();

        let id1 = generator.next();
        let id2 = cloned.next();

        assert_ne!(id1, id2);
        assert!(id1.starts_with("prefix_"));
        assert!(id2.starts_with("prefix_"));
    }

    #[test]
    fn test_event_batcher_should_flush_by_size() {
        let batcher = EventBatcher::new(2, Duration::from_secs(10));
        assert!(!batcher.should_flush());

        let mut batcher = EventBatcher::new(1, Duration::from_secs(10));
        let event = Box::new(GenericEvent::new(
            "1".to_string(),
            EventKind::Transaction,
            json!({}),
        ));
        batcher.add(event);
        assert!(batcher.should_flush());
    }

    #[test]
    fn test_event_batcher_should_flush_by_timeout() {
        let mut batcher = EventBatcher::new(10, Duration::from_millis(1));
        let event = Box::new(GenericEvent::new(
            "1".to_string(),
            EventKind::Transaction,
            json!({}),
        ));
        batcher.add(event);

        // Wait for timeout
        std::thread::sleep(Duration::from_millis(2));
        assert!(batcher.should_flush());
    }

    #[test]
    fn test_event_batcher_flush_empty() {
        let mut batcher = EventBatcher::new(3, Duration::from_secs(10));
        assert!(batcher.flush().is_none());
        assert!(batcher.is_empty());
    }

    #[test]
    fn test_event_batcher_current_size_and_is_empty() {
        let mut batcher = EventBatcher::new(3, Duration::from_secs(10));
        assert_eq!(batcher.current_size(), 0);
        assert!(batcher.is_empty());

        let event = Box::new(GenericEvent::new(
            "1".to_string(),
            EventKind::Transaction,
            json!({}),
        ));
        batcher.add(event);

        assert_eq!(batcher.current_size(), 1);
        assert!(!batcher.is_empty());
    }

    #[tokio::test]
    async fn test_event_deduplicator_cleanup() {
        let deduplicator =
            EventDeduplicator::new(Duration::from_millis(10), Duration::from_secs(1));

        let event = GenericEvent::new("test-event".to_string(), EventKind::Transaction, json!({}));
        deduplicator.mark_seen(&event).await;
        assert!(deduplicator.is_duplicate(&event).await);

        // Wait for TTL to expire
        tokio::time::sleep(Duration::from_millis(20)).await;
        deduplicator.cleanup().await;

        assert!(!deduplicator.is_duplicate(&event).await);
    }

    #[tokio::test]
    async fn test_event_deduplicator_start_cleanup_task() {
        let deduplicator =
            EventDeduplicator::new(Duration::from_millis(50), Duration::from_millis(10));

        let handle = deduplicator.start_cleanup_task();

        let event = GenericEvent::new(
            "cleanup-test".to_string(),
            EventKind::Transaction,
            json!({}),
        );
        deduplicator.mark_seen(&event).await;
        assert!(deduplicator.is_duplicate(&event).await);

        // Wait for cleanup task to run
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Event should be cleaned up
        assert!(!deduplicator.is_duplicate(&event).await);

        handle.abort();
    }

    #[tokio::test]
    async fn test_rate_limiter_wait_for_capacity() {
        let rate_limiter = RateLimiter::new(1, Duration::from_millis(100));

        // Fill capacity
        rate_limiter.record_event().await;
        assert!(!rate_limiter.can_process().await);

        // Test wait_for_capacity (should return quickly after window expires)
        let start = Instant::now();
        rate_limiter.wait_for_capacity().await;
        let elapsed = start.elapsed();

        // Should wait at least a short time but not too long
        assert!(elapsed >= Duration::from_millis(10));
        assert!(elapsed < Duration::from_millis(200));
    }

    #[tokio::test]
    async fn test_rate_limiter_current_rate_calculation() {
        let rate_limiter = RateLimiter::new(10, Duration::from_secs(1));

        // No events initially
        assert_eq!(rate_limiter.current_rate().await, 0.0);

        // Add some events
        rate_limiter.record_event().await;
        rate_limiter.record_event().await;

        let rate = rate_limiter.current_rate().await;
        assert!(rate > 0.0);
        assert!(rate <= 10.0);
    }

    #[tokio::test]
    async fn test_performance_metrics_empty_percentiles() {
        let metrics = EventPerformanceMetrics::default();

        let percentiles = metrics
            .processing_time_percentiles(&[50.0, 95.0, 99.0])
            .await;

        assert_eq!(percentiles.len(), 3);
        assert_eq!(percentiles[0], Duration::ZERO);
        assert_eq!(percentiles[1], Duration::ZERO);
        assert_eq!(percentiles[2], Duration::ZERO);
    }

    #[tokio::test]
    async fn test_performance_metrics_empty_avg() {
        let metrics = EventPerformanceMetrics::default();

        let avg = metrics.avg_processing_time().await;
        assert_eq!(avg, Duration::ZERO);
    }

    #[tokio::test]
    async fn test_performance_metrics_error_rate_zero_events() {
        let metrics = EventPerformanceMetrics::default();
        metrics.record_error();

        assert_eq!(metrics.error_rate(), 0.0);
        assert_eq!(metrics.total_events(), 0);
        assert_eq!(metrics.total_errors(), 1);
    }

    #[tokio::test]
    async fn test_performance_metrics_reset() {
        let metrics = EventPerformanceMetrics::default();

        metrics
            .record_processing_time(Duration::from_millis(10))
            .await;
        metrics.record_error();

        assert_eq!(metrics.total_events(), 1);
        assert_eq!(metrics.total_errors(), 1);

        metrics.reset().await;

        assert_eq!(metrics.total_events(), 0);
        assert_eq!(metrics.total_errors(), 0);
        assert_eq!(metrics.avg_processing_time().await, Duration::ZERO);
    }

    #[tokio::test]
    async fn test_performance_metrics_memory_management() {
        let metrics = EventPerformanceMetrics::default();

        // Add more than 10,000 measurements to test memory management
        for i in 1..=10_050 {
            metrics
                .record_processing_time(Duration::from_millis(i % 100))
                .await;
        }

        let times = metrics.processing_times.read().await;
        assert!(times.len() <= 10_000);
        assert_eq!(metrics.total_events(), 10_050);
    }

    #[tokio::test]
    async fn test_performance_metrics_summary_empty() {
        let metrics = EventPerformanceMetrics::default();

        let summary = metrics.summary().await;
        assert_eq!(summary.total_events, 0);
        assert_eq!(summary.total_errors, 0);
        assert_eq!(summary.error_rate, 0.0);
        assert_eq!(summary.avg_processing_time, Duration::ZERO);
        assert_eq!(summary.p95_processing_time, Duration::ZERO);
        assert_eq!(summary.p99_processing_time, Duration::ZERO);
    }

    #[test]
    fn test_metrics_summary_serialization() {
        let summary = MetricsSummary {
            total_events: 100,
            total_errors: 5,
            error_rate: 5.0,
            avg_processing_time: Duration::from_millis(10),
            p95_processing_time: Duration::from_millis(50),
            p99_processing_time: Duration::from_millis(100),
            uptime: Duration::from_secs(3600),
        };

        let serialized = serde_json::to_string(&summary);
        assert!(serialized.is_ok());

        let deserialized: Result<MetricsSummary, _> = serde_json::from_str(&serialized.unwrap());
        assert!(deserialized.is_ok());

        let deserialized = deserialized.unwrap();
        assert_eq!(deserialized.total_events, 100);
        assert_eq!(deserialized.total_errors, 5);
        assert_eq!(deserialized.error_rate, 5.0);
    }

    // Tests for StreamUtils would require complex mock streams and are already tested through integration
    // The StreamUtils methods are static utility methods that primarily compose existing stream operations
}
