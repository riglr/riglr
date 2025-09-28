//! Utility functions and helpers for event processing.

extern crate alloc;
use crate::error::EventResult;
use crate::traits::Event;
use alloc::sync::Arc;
use core::mem;
use core::pin::Pin;
use core::sync::atomic::{AtomicU64, Ordering};
use core::time::Duration;
use dashmap::DashMap;
use futures::{Stream, StreamExt as _};
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tokio::time::{interval, sleep};

/// Type alias for event streams to reduce complexity
pub type EventStream = Pin<Box<dyn Stream<Item = EventResult<Box<dyn Event>>> + Send>>;

/// Type alias for event batch streams
pub type EventBatchStream = Pin<Box<dyn Stream<Item = EventResult<Vec<Box<dyn Event>>>> + Send>>;

/// Utility for generating unique event IDs.
#[derive(Debug, Clone)]
pub struct EventIdGenerator {
    /// The counter for generating unique sequential numbers
    counter: Arc<AtomicU64>,
    /// The prefix to use for generated IDs
    prefix: String,
}

impl EventIdGenerator {
    /// Create a new ID generator with a prefix
    #[must_use]
    #[inline]
    pub fn new(prefix: String) -> Self {
        Self {
            prefix,
            counter: Arc::new(AtomicU64::default()),
        }
    }

    /// Generate a new unique ID
    #[must_use]
    #[inline]
    pub fn next(&self) -> String {
        let count = self.counter.fetch_add(1, Ordering::SeqCst);
        {
            format!(
                "{}_{:016x}_{:016x}",
                self.prefix,
                u64::try_from(
                    SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_micros()
                )
                .unwrap_or(0),
                count
            )
        }
    }

    /// Generate an ID with additional context
    #[must_use]
    #[inline]
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
    /// Maximum number of events before flushing
    batch_size: usize,
    /// Current batch of accumulated events
    current_batch: Vec<Box<dyn Event>>,
    /// Time when the last flush occurred
    last_flush: Instant,
    /// Maximum time to wait before flushing
    timeout: Duration,
}

impl EventBatcher {
    /// Add an event to the current batch
    #[inline]
    pub fn add(&mut self, event: Box<dyn Event>) -> Option<Vec<Box<dyn Event>>> {
        self.current_batch.push(event);

        if self.should_flush() {
            return self.flush();
        }
        None
    }

    /// Get the current batch size
    #[must_use]
    #[inline]
    pub fn current_size(&self) -> usize {
        self.current_batch.len()
    }

    /// Flush the current batch and return the events
    #[inline]
    pub fn flush(&mut self) -> Option<Vec<Box<dyn Event>>> {
        if self.current_batch.is_empty() {
            return None;
        }
        let batch = mem::replace(&mut self.current_batch, Vec::with_capacity(self.batch_size));
        self.last_flush = Instant::now();
        Some(batch)
    }

    /// Check if the batch is empty
    #[must_use]
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.current_batch.is_empty()
    }

    /// Create a new event batcher
    #[must_use]
    #[inline]
    pub fn new(batch_size: usize, timeout: Duration) -> Self {
        Self {
            batch_size,
            timeout,
            current_batch: Vec::with_capacity(batch_size),
            last_flush: Instant::now(),
        }
    }

    /// Check if the batch should be flushed
    #[must_use]
    #[inline]
    pub fn should_flush(&self) -> bool {
        self.current_batch.len() >= self.batch_size || self.last_flush.elapsed() >= self.timeout
    }
}

/// Event deduplication utility to prevent processing duplicate events.
#[derive(Debug)]
pub struct EventDeduplicator {
    /// Interval for automatic cleanup of expired entries
    cleanup_interval: Duration,
    /// Map of seen event IDs and when they were first seen
    seen_events: Arc<DashMap<String, SystemTime>>,
    /// Time-to-live for seen events
    ttl: Duration,
}

impl EventDeduplicator {
    /// Clean up expired entries
    #[inline]
    pub fn cleanup(&self) {
        let now = SystemTime::now();

        self.seen_events
            .retain(|_, seen_at| now.duration_since(*seen_at).unwrap_or_default() < self.ttl);
    }

    /// Check if an event is a duplicate
    #[inline]
    pub fn is_duplicate(&self, event: &dyn Event) -> bool {
        let event_id = event.id();
        let seen_entry = self.seen_events.get(event_id);

        seen_entry.is_some_and(|seen_at| seen_at.value().elapsed().unwrap_or_default() < self.ttl)
    }

    /// Mark an event as seen
    #[inline]
    pub fn mark_seen(&self, event: &dyn Event) {
        let event_id = event.id().to_owned();
        self.seen_events.insert(event_id, SystemTime::now());
    }

    /// Create a new deduplicator with TTL for seen events
    #[must_use]
    #[inline]
    pub fn new(ttl: Duration, cleanup_interval: Duration) -> Self {
        Self {
            seen_events: Arc::new(DashMap::default()),
            ttl,
            cleanup_interval,
        }
    }

    /// Start automatic cleanup task
    #[must_use]
    #[inline]
    pub fn start_cleanup_task(&self) -> JoinHandle<()> {
        let seen_events = Arc::clone(&self.seen_events);
        let ttl = self.ttl;
        let cleanup_interval = self.cleanup_interval;

        tokio::spawn(async move {
            let mut interval = interval(cleanup_interval);

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
    /// Timestamps of events processed within the current window
    events_in_window: Arc<RwLock<Vec<SystemTime>>>,
    /// Maximum number of events per time window
    max_rate: u64,
    /// Time window for rate limiting
    window: Duration,
}

impl RateLimiter {
    /// Check if we can process another event
    #[inline]
    pub async fn can_process(&self) -> bool {
        let mut events = self.events_in_window.write().await;
        let now = SystemTime::now();

        // Remove old events outside the window
        events.retain(|timestamp| now.duration_since(*timestamp).unwrap_or_default() < self.window);

        events.len() < usize::try_from(self.max_rate).unwrap_or(usize::MAX)
    }

    /// Get current rate (events per second)
    #[inline]
    pub async fn current_rate(&self) -> f64 {
        let now = SystemTime::now();
        let recent_events = self
            .events_in_window
            .read()
            .await
            .iter()
            .filter(|timestamp| now.duration_since(**timestamp).unwrap_or_default() < self.window)
            .count();

        #[expect(clippy::cast_precision_loss)]
        {
            recent_events as f64 / self.window.as_secs_f64()
        }
    }

    /// Create a new rate limiter
    #[must_use]
    #[inline]
    pub fn new(max_rate: u64, window: Duration) -> Self {
        Self {
            max_rate,
            window,
            events_in_window: Arc::new(RwLock::default()),
        }
    }

    /// Record that an event was processed
    #[inline]
    pub async fn record_event(&self) {
        let mut events = self.events_in_window.write().await;
        events.push(SystemTime::now());
    }

    /// Wait until we can process the next event
    #[inline]
    pub async fn wait_for_capacity(&self) {
        while !self.can_process().await {
            sleep(Duration::from_millis(10)).await;
        }
    }
}

/// Stream transformation utilities
#[derive(Debug)]
#[non_exhaustive]
pub struct StreamOps;

impl StreamOps {
    /// Batch events into groups of specified size
    #[must_use]
    #[inline]
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

    /// Deduplicate events in a stream
    #[must_use]
    #[inline]
    pub fn deduplicate(stream: EventStream, deduplicator: Arc<EventDeduplicator>) -> EventStream {
        Box::pin(stream.filter_map(move |result| {
            let deduplicator = Arc::clone(&deduplicator);
            async move {
                match result {
                    Ok(event) => {
                        if deduplicator.is_duplicate(&*event) {
                            return None; // Skip duplicate
                        }
                        deduplicator.mark_seen(&*event);
                        Some(Ok(event))
                    }
                    Err(e) => Some(Err(e)),
                }
            }
        }))
    }

    /// Filter events based on a predicate
    #[inline]
    pub fn filter_events<F>(stream: EventStream, predicate: F) -> EventStream
    where
        F: Fn(&dyn Event) -> bool + Send + Sync + Clone + 'static,
    {
        let pred_clone = predicate;
        Box::pin(stream.filter_map(move |result| {
            let predicate = pred_clone.clone();
            async move {
                match result {
                    Ok(event) => {
                        if predicate(&*event) {
                            return Some(Ok(event));
                        }
                        None
                    }
                    Err(e) => Some(Err(e)),
                }
            }
        }))
    }

    /// Transform a stream of events using a mapping function
    #[inline]
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

    /// Add rate limiting to an event stream
    #[must_use]
    #[inline]
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
}

/// Event performance metrics for tracking processing statistics.
#[derive(Debug)]
pub struct EventPerformanceMetrics {
    /// Time when metrics were last reset
    last_reset: Arc<RwLock<SystemTime>>,
    /// Individual processing times for calculating percentiles
    processing_times: Arc<RwLock<Vec<Duration>>>,
    /// Counter for total errors encountered
    total_errors: Arc<AtomicU64>,
    /// Counter for total events processed
    total_events: Arc<AtomicU64>,
}

impl EventPerformanceMetrics {
    /// Get average processing time
    #[inline]
    pub async fn avg_processing_time(&self) -> Duration {
        let times = self.processing_times.read().await;
        if times.is_empty() {
            return Duration::ZERO;
        }
        let sum: Duration = times.iter().sum();
        sum.checked_div(u32::try_from(times.len()).unwrap_or(1))
            .unwrap_or(Duration::ZERO)
    }

    /// Get error rate as percentage
    #[must_use]
    #[inline]
    pub fn error_rate(&self) -> f64 {
        let total = self.total_events();
        let errors = self.total_errors();

        if total == 0 {
            return 0.0_f64;
        }
        #[expect(clippy::cast_precision_loss)]
        {
            (errors as f64 / total as f64) * 100.0_f64
        }
    }

    /// Get processing time percentiles
    #[inline]
    pub async fn processing_time_percentiles(&self, percentiles: &[f64]) -> Vec<Duration> {
        let mut times = self.processing_times.read().await.clone();
        if times.is_empty() {
            return vec![Duration::ZERO; percentiles.len()];
        }
        times.sort();

        percentiles
            .iter()
            .map(|&p| {
                #[expect(
                    clippy::cast_precision_loss,
                    clippy::cast_possible_truncation,
                    clippy::cast_sign_loss
                )]
                let index = {
                    let float_index = times.len() as f64 * p / 100.0_f64;
                    if float_index < 0.0 {
                        0
                    } else {
                        (float_index as usize).min(times.len().saturating_sub(1))
                    }
                };
                times.get(index).copied().unwrap_or(Duration::ZERO)
            })
            .collect()
    }

    /// Record an error
    #[inline]
    pub fn record_error(&self) {
        self.total_errors.fetch_add(1, Ordering::SeqCst);
    }

    /// Record an event processing time
    #[inline]
    pub async fn record_processing_time(&self, duration: Duration) {
        self.total_events.fetch_add(1, Ordering::SeqCst);
        let mut times = self.processing_times.write().await;
        times.push(duration);

        // Keep only the last 10,000 measurements to prevent memory bloat
        if times.len() > 10_000 {
            let new_len = times.len().saturating_sub(10_000);
            times.drain(..new_len);
        }
    }

    /// Reset all metrics
    #[inline]
    pub async fn reset(&self) {
        self.total_events.store(0, Ordering::SeqCst);
        self.total_errors.store(0, Ordering::SeqCst);
        self.processing_times.write().await.clear();
        *self.last_reset.write().await = SystemTime::now();
    }

    /// Get metrics summary
    #[inline]
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

            let avg = sorted_times
                .iter()
                .sum::<Duration>()
                .checked_div(u32::try_from(sorted_times.len()).unwrap_or(1))
                .unwrap_or(Duration::ZERO);
            #[expect(
                clippy::cast_precision_loss,
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss
            )]
            let p95_idx = {
                let float_idx = sorted_times.len() as f64 * 0.95_f64;
                (float_idx as usize).min(sorted_times.len().saturating_sub(1))
            };
            #[expect(
                clippy::cast_precision_loss,
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss
            )]
            let p99_idx = {
                let float_idx = sorted_times.len() as f64 * 0.99_f64;
                (float_idx as usize).min(sorted_times.len().saturating_sub(1))
            };

            (
                avg,
                sorted_times.get(p95_idx).copied().unwrap_or(Duration::ZERO),
                sorted_times.get(p99_idx).copied().unwrap_or(Duration::ZERO),
            )
        };

        return MetricsSummary {
            avg_processing_time: avg_time,
            error_rate: self.error_rate(),
            p95_processing_time: p95_time,
            p99_processing_time: p99_time,
            total_errors,
            total_events,
            uptime: self.last_reset.read().await.elapsed().unwrap_or_default(),
        };
    }

    /// Get total errors
    #[must_use]
    #[inline]
    pub fn total_errors(&self) -> u64 {
        self.total_errors.load(Ordering::SeqCst)
    }

    /// Get total events processed
    #[must_use]
    #[inline]
    pub fn total_events(&self) -> u64 {
        self.total_events.load(Ordering::SeqCst)
    }
}

impl Default for EventPerformanceMetrics {
    #[inline]
    fn default() -> Self {
        Self {
            last_reset: Arc::new(RwLock::new(SystemTime::now())),
            processing_times: Arc::new(RwLock::default()),
            total_errors: Arc::new(AtomicU64::default()),
            total_events: Arc::new(AtomicU64::default()),
        }
    }
}

/// Summary of event processing metrics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct MetricsSummary {
    /// Average processing time
    pub avg_processing_time: Duration,
    /// Error rate as percentage
    pub error_rate: f64,
    /// 95th percentile processing time
    pub p95_processing_time: Duration,
    /// 99th percentile processing time
    pub p99_processing_time: Duration,
    /// Total errors encountered
    pub total_errors: u64,
    /// Total events processed
    pub total_events: u64,
    /// Time since metrics were last reset
    pub uptime: Duration,
}

#[cfg(test)]
#[expect(clippy::expect_used)]
mod tests {
    use super::*;
    use crate::types::{EventKind, GenericEvent};
    use serde_json::json;
    use std::thread;
    use tokio::time;

    #[test]
    fn event_id_generator() {
        let generator = EventIdGenerator::new("test".to_owned());

        let id1 = generator.next();
        let id2 = generator.next();

        assert_ne!(id1, id2);
        assert!(id1.starts_with("test_"));
        assert!(id2.starts_with("test_"));
    }

    #[test]
    fn event_batcher() {
        let mut batcher = EventBatcher::new(3, Duration::from_secs(10));

        let event1 = Box::new(GenericEvent::new(
            "1".to_owned(),
            EventKind::Transaction,
            json!({}),
        ));
        let event2 = Box::new(GenericEvent::new(
            "2".to_owned(),
            EventKind::Transaction,
            json!({}),
        ));
        let event3 = Box::new(GenericEvent::new(
            "3".to_owned(),
            EventKind::Transaction,
            json!({}),
        ));

        assert!(batcher.add(event1).is_none());
        assert!(batcher.add(event2).is_none());
        assert_eq!(batcher.current_size(), 2);

        let batch = batcher.add(event3);
        assert!(batch.is_some());

        let batch = batch.expect("batch should exist");
        assert_eq!(batch.len(), 3);
    }

    // Additional comprehensive tests for 100% coverage

    #[test]
    fn event_id_generator_next_with_context() {
        let generator = EventIdGenerator::new("test".to_owned());

        let id = generator.next_with_context("custom_context");
        assert!(id.contains("test_"));
        assert!(id.contains("custom_context"));
        assert!(id.len() > 50); // Should be longer due to UUID
    }

    #[test]
    fn event_id_generator_clone() {
        let generator = EventIdGenerator::new("prefix".to_owned());
        let cloned = generator.clone();

        let id1 = generator.next();
        let id2 = cloned.next();

        assert_ne!(id1, id2);
        assert!(id1.starts_with("prefix_"));
        assert!(id2.starts_with("prefix_"));
    }

    #[test]
    fn event_batcher_should_flush_by_size() {
        let batcher = EventBatcher::new(2, Duration::from_secs(10));
        assert!(!batcher.should_flush());

        let mut batcher = EventBatcher::new(1, Duration::from_secs(10));
        let event = Box::new(GenericEvent::new(
            "1".to_owned(),
            EventKind::Transaction,
            json!({}),
        ));
        let result = batcher.add(event);
        // When batch size is 1, add() should return the batch immediately
        assert!(result.is_some());
        assert_eq!(
            result
                .expect("Batch should be Some when batch size is 1")
                .len(),
            1
        );
        // After flushing, should_flush should be false since batch is empty
        assert!(!batcher.should_flush());
    }

    #[test]
    fn event_batcher_should_flush_by_timeout() {
        let mut batcher = EventBatcher::new(10, Duration::from_millis(1));
        let event = Box::new(GenericEvent::new(
            "1".to_owned(),
            EventKind::Transaction,
            json!({}),
        ));
        batcher.add(event);

        // Wait for timeout
        thread::sleep(Duration::from_millis(2));
        assert!(batcher.should_flush());
    }

    #[test]
    fn event_batcher_flush_empty() {
        let mut batcher = EventBatcher::new(3, Duration::from_secs(10));
        assert!(batcher.flush().is_none());
        assert!(batcher.is_empty());
    }

    #[test]
    fn event_batcher_current_size_and_is_empty() {
        let mut batcher = EventBatcher::new(3, Duration::from_secs(10));
        assert_eq!(batcher.current_size(), 0);
        assert!(batcher.is_empty());

        let event = Box::new(GenericEvent::new(
            "1".to_owned(),
            EventKind::Transaction,
            json!({}),
        ));
        batcher.add(event);

        assert_eq!(batcher.current_size(), 1);
        assert!(!batcher.is_empty());
    }

    #[tokio::test]
    async fn event_deduplicator_cleanup() {
        let deduplicator =
            EventDeduplicator::new(Duration::from_millis(10), Duration::from_secs(1));

        let event = GenericEvent::new("test-event".to_owned(), EventKind::Transaction, json!({}));
        deduplicator.mark_seen(&event);
        assert!(deduplicator.is_duplicate(&event));

        // Wait for TTL to expire
        time::sleep(Duration::from_millis(20)).await;
        deduplicator.cleanup();

        assert!(!deduplicator.is_duplicate(&event));
    }

    #[tokio::test]
    async fn event_deduplicator_start_cleanup_task() {
        let deduplicator =
            EventDeduplicator::new(Duration::from_millis(50), Duration::from_millis(10));

        let handle = deduplicator.start_cleanup_task();

        let event = GenericEvent::new("cleanup-test".to_owned(), EventKind::Transaction, json!({}));
        deduplicator.mark_seen(&event);
        assert!(deduplicator.is_duplicate(&event));

        // Wait for cleanup task to run
        time::sleep(Duration::from_millis(100)).await;

        // Event should be cleaned up
        assert!(!deduplicator.is_duplicate(&event));

        handle.abort();
    }

    #[tokio::test]
    async fn rate_limiter_wait_for_capacity() {
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
    async fn rate_limiter_current_rate_calculation() {
        let rate_limiter = RateLimiter::new(10, Duration::from_secs(1));

        // No events initially
        assert!((rate_limiter.current_rate().await - 0.0_f64).abs() < f64::EPSILON);

        // Add some events
        rate_limiter.record_event().await;
        rate_limiter.record_event().await;

        let rate = rate_limiter.current_rate().await;
        assert!(rate > 0.0_f64);
        assert!(rate <= 10.0_f64);
    }

    #[tokio::test]
    async fn performance_metrics_empty_percentiles() {
        let metrics = EventPerformanceMetrics::default();

        let percentiles = metrics
            .processing_time_percentiles(&[50.0_f64, 95.0_f64, 99.0_f64])
            .await;

        assert_eq!(percentiles.len(), 3);
        // All percentiles should be zero for empty metrics
        for &percentile in &percentiles {
            assert_eq!(percentile, Duration::ZERO);
        }
    }

    // Tests for StreamOps would require complex mock streams and are already tested through integration
    // The StreamOps methods are static utility methods that primarily compose existing stream operations
}
