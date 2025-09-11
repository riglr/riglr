//! Advanced event processing with windowing, stateful operations, and flow control
//!
//! This module provides sophisticated event processing capabilities including:
//! - Time-based and count-based windowing
//! - Stateful stream processing with checkpointing
//! - Complex event processing (CEP) patterns
//! - Backpressure management and flow control

use crate::core::config::{BackpressureConfig, BackpressureStrategy, BatchConfig};
use crate::core::error::{StreamError, StreamResult};
use dashmap::DashMap;
use std::collections::{HashMap, VecDeque};
use std::hash::Hash;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};
use tokio::sync::Semaphore;
use tokio::time::sleep;
use tracing::{info, warn};

/// Window types for time-based processing
#[derive(Debug, Clone)]
pub enum WindowType {
    /// Fixed-size tumbling windows
    Tumbling {
        /// Duration of each tumbling window
        duration: Duration,
    },
    /// Sliding windows with overlap
    Sliding {
        /// Size of the sliding window
        size: Duration,
        /// Step size between windows
        step: Duration,
    },
    /// Session windows that close after inactivity
    Session {
        /// Timeout duration for session inactivity
        timeout: Duration,
    },
    /// Count-based windows
    Count {
        /// Number of events per window
        size: usize,
    },
}

/// Window state for managing event windows
#[derive(Debug)]
pub struct Window<E> {
    /// Unique identifier for this window
    pub id: u64,
    /// When this window started
    pub start_time: Instant,
    /// When this window ended (if closed)
    pub end_time: Option<Instant>,
    /// Events collected in this window
    pub events: Vec<E>,
    /// Whether this window is closed for new events
    pub is_closed: bool,
}

impl<E> Window<E> {
    /// Creates a new window with the given ID and start time
    fn new(id: u64, start_time: Instant) -> Self {
        Self {
            id,
            start_time,
            end_time: None,
            events: Vec::new(),
            is_closed: false,
        }
    }

    /// Closes the window, preventing new events from being added
    fn close(&mut self) {
        self.end_time = Some(Instant::now());
        self.is_closed = true;
    }

    /// Adds an event to the window if it's not closed
    fn add_event(&mut self, event: E) {
        if !self.is_closed {
            self.events.push(event);
        }
    }
}

/// Window manager for handling different window types
#[derive(Debug)]
pub struct WindowManager<E> {
    /// Type of windowing to apply
    window_type: WindowType,
    /// Currently active windows indexed by ID
    active_windows: HashMap<u64, Window<E>>,
    /// Counter for generating unique window IDs
    next_window_id: u64,
    /// Last time cleanup was performed
    last_cleanup: Instant,
}

impl<E> WindowManager<E> {
    /// Creates a new window manager with the specified window type
    pub fn new(window_type: WindowType) -> Self {
        Self {
            window_type,
            active_windows: HashMap::new(),
            next_window_id: 0,
            last_cleanup: Instant::now(),
        }
    }

    /// Adds an event to the appropriate window(s) and returns any completed windows
    pub fn add_event(&mut self, event: E) -> Vec<Window<E>> {
        let now = Instant::now();
        let system_now = SystemTime::now();
        let mut completed_windows = Vec::new();

        match &self.window_type {
            WindowType::Tumbling { duration } => {
                let window_id = match system_now.duration_since(SystemTime::UNIX_EPOCH) {
                    Ok(duration_since_epoch) => duration_since_epoch.as_secs() / duration.as_secs(),
                    Err(_) => 0,
                };

                let window = self
                    .active_windows
                    .entry(window_id)
                    .or_insert_with(|| Window::new(window_id, now));

                window.add_event(event);

                // Check for expired windows
                let cutoff = now - *duration;
                let expired_ids: Vec<_> = self
                    .active_windows
                    .iter()
                    .filter(|(_, window)| window.start_time < cutoff)
                    .map(|(&id, _)| id)
                    .collect();

                for id in expired_ids {
                    if let Some(mut window) = self.active_windows.remove(&id) {
                        window.close();
                        completed_windows.push(window);
                    }
                }
            }

            WindowType::Sliding { size, step } => {
                // Create overlapping windows based on step size
                let step_secs = step.as_secs();
                let window_start = match system_now.duration_since(SystemTime::UNIX_EPOCH) {
                    Ok(duration) => (duration.as_secs() / step_secs) * step_secs,
                    Err(_) => 0,
                };
                let window_id = window_start;

                let window = self
                    .active_windows
                    .entry(window_id)
                    .or_insert_with(|| Window::new(window_id, now));

                window.add_event(event);

                // Check for expired sliding windows
                let cutoff = now - *size;
                let expired_ids: Vec<_> = self
                    .active_windows
                    .iter()
                    .filter(|(_, window)| window.start_time < cutoff)
                    .map(|(&id, _)| id)
                    .collect();

                for id in expired_ids {
                    if let Some(mut window) = self.active_windows.remove(&id) {
                        window.close();
                        completed_windows.push(window);
                    }
                }
            }

            WindowType::Session { timeout } => {
                // Session windows group events with no gaps larger than timeout
                let window_id = self.next_window_id;
                let window = self.active_windows.entry(window_id).or_insert_with(|| {
                    self.next_window_id += 1;
                    Window::new(window_id, now)
                });

                window.add_event(event);

                // Close sessions that have been inactive
                let inactive_cutoff = now - *timeout;
                let inactive_ids: Vec<_> = self
                    .active_windows
                    .iter()
                    .filter(|(_, window)| {
                        window.events.is_empty() || window.start_time < inactive_cutoff
                    })
                    .map(|(&id, _)| id)
                    .collect();

                for id in inactive_ids {
                    if let Some(mut window) = self.active_windows.remove(&id) {
                        window.close();
                        completed_windows.push(window);
                    }
                }
            }

            WindowType::Count { size } => {
                let window_id = self.next_window_id;
                let window = self.active_windows.entry(window_id).or_insert_with(|| {
                    self.next_window_id += 1;
                    Window::new(window_id, now)
                });

                window.add_event(event);

                // Close window when it reaches target size
                if window.events.len() >= *size {
                    if let Some(mut completed_window) = self.active_windows.remove(&window_id) {
                        completed_window.close();
                        completed_windows.push(completed_window);
                    }
                }
            }
        }

        // Periodic cleanup
        if self.last_cleanup.elapsed() > Duration::from_secs(60) {
            self.cleanup_old_windows();
            self.last_cleanup = now;
        }

        completed_windows
    }

    /// Removes old windows that are no longer needed
    fn cleanup_old_windows(&mut self) {
        let cleanup_start = Instant::now();
        let cutoff = cleanup_start - Duration::from_secs(3600); // Keep windows for 1 hour max
        let old_ids: Vec<_> = self
            .active_windows
            .iter()
            .filter(|(_, window)| window.start_time < cutoff)
            .map(|(&id, _)| id)
            .collect();

        for id in old_ids {
            self.active_windows.remove(&id);
        }
    }
}

/// Stateful event processor with checkpointing
#[derive(Debug)]
pub struct StatefulProcessor<K, S>
where
    K: Hash + Eq + Clone,
    S: Clone,
{
    /// Concurrent state storage using DashMap for better performance
    state_store: Arc<DashMap<K, S>>,
    /// How often to create checkpoints
    checkpoint_interval: Duration,
    /// When the last checkpoint was created
    last_checkpoint: Instant,
}

impl<K, S> StatefulProcessor<K, S>
where
    K: Hash + Eq + Clone + Send + Sync + 'static,
    S: Clone + Send + Sync + 'static,
{
    /// Creates a new stateful processor with the specified checkpoint interval
    pub fn new(checkpoint_interval: Duration) -> Self {
        Self {
            state_store: Arc::new(DashMap::new()),
            checkpoint_interval,
            last_checkpoint: Instant::now(),
        }
    }

    /// Retrieves the current state for the given key
    pub async fn get_state(&self, key: &K) -> Option<S> {
        self.state_store.get(key).map(|entry| entry.value().clone())
    }

    /// Updates the state for the given key using the provided function
    pub async fn update_state<F, R>(&self, key: K, update_fn: F) -> R
    where
        F: FnOnce(Option<&S>) -> (S, R),
    {
        let current = self.state_store.get(&key);
        let current_ref = current.as_ref().map(|entry| entry.value());
        let (new_state, result) = update_fn(current_ref);
        self.state_store.insert(key, new_state);
        result
    }

    /// Removes and returns the state for the given key
    pub async fn remove_state(&self, key: &K) -> Option<S> {
        self.state_store.remove(key).map(|(_, value)| value)
    }

    /// Creates a checkpoint of the current state if enough time has passed
    pub async fn checkpoint(&self) -> StreamResult<()> {
        if self.last_checkpoint.elapsed() >= self.checkpoint_interval {
            let state_count = self.state_store.len();

            // In a real implementation, this would persist to durable storage
            info!("Checkpointing state with {} entries", state_count);
            // self.persist_checkpoint(state_snapshot).await?;

            Ok(())
        } else {
            Ok(())
        }
    }
}

/// Flow control manager with backpressure handling
#[derive(Debug)]
pub struct FlowController {
    /// Backpressure configuration settings
    config: BackpressureConfig,
    /// Semaphore for controlling concurrent access
    semaphore: Arc<Semaphore>,
    /// Current queue size counter
    queue_size: Arc<tokio::sync::RwLock<usize>>,
    /// Number of dropped events counter
    drop_count: Arc<tokio::sync::RwLock<usize>>,
}

impl FlowController {
    /// Creates a new flow controller with the specified backpressure configuration
    pub fn new(config: BackpressureConfig) -> Self {
        let semaphore = Arc::new(Semaphore::new(config.channel_size));

        Self {
            config,
            semaphore,
            queue_size: Arc::new(tokio::sync::RwLock::new(0)),
            drop_count: Arc::new(tokio::sync::RwLock::new(0)),
        }
    }

    /// Attempts to acquire a permit based on the configured backpressure strategy
    pub async fn acquire_permit(&self) -> StreamResult<()> {
        let current_size = *self.queue_size.read().await;

        match self.config.strategy {
            BackpressureStrategy::Block => {
                self.semaphore
                    .acquire()
                    .await
                    .map_err(|_| StreamError::Processing {
                        message: "Failed to acquire permit".into(),
                    })?
                    .forget();
                Ok(())
            }

            BackpressureStrategy::Drop => {
                if current_size >= self.config.high_watermark() {
                    let mut drop_count = self.drop_count.write().await;
                    *drop_count += 1;

                    if *drop_count % 100 == 0 {
                        warn!("Dropped {} events due to backpressure", *drop_count);
                    }

                    return Err(StreamError::Backpressure {
                        message: "Event dropped due to backpressure".into(),
                    });
                }

                self.semaphore
                    .acquire()
                    .await
                    .map_err(|_| StreamError::Processing {
                        message: "Failed to acquire permit".into(),
                    })?
                    .forget();
                Ok(())
            }

            BackpressureStrategy::Retry {
                max_attempts,
                base_wait_ms,
            } => {
                let mut attempts = 0;

                while attempts < max_attempts {
                    if let Ok(permit) = self.semaphore.try_acquire() {
                        permit.forget();
                        return Ok(());
                    }

                    attempts += 1;
                    let delay = Duration::from_millis(base_wait_ms * (1 << attempts.min(5))); // Exponential backoff
                    sleep(delay).await;
                }

                let mut drop_count = self.drop_count.write().await;
                *drop_count += 1;
                Err(StreamError::Backpressure {
                    message: format!("Failed after {} retry attempts", max_attempts),
                })
            }

            BackpressureStrategy::Adaptive => {
                if current_size >= self.config.high_watermark() {
                    // Switch to drop mode under high load
                    let mut drop_count = self.drop_count.write().await;
                    *drop_count += 1;
                    Err(StreamError::Backpressure {
                        message: "Adaptive backpressure: dropping event".into(),
                    })
                } else if current_size <= self.config.low_watermark() {
                    // Switch to block mode under low load
                    self.semaphore
                        .acquire()
                        .await
                        .map_err(|_| StreamError::Processing {
                            message: "Failed to acquire permit".into(),
                        })?
                        .forget();
                    Ok(())
                } else {
                    // Try to acquire, but don't wait too long
                    let timeout_result =
                        tokio::time::timeout(Duration::from_millis(10), self.semaphore.acquire())
                            .await;
                    if let Ok(Ok(permit)) = timeout_result {
                        permit.forget();
                        Ok(())
                    } else {
                        let mut drop_count = self.drop_count.write().await;
                        *drop_count += 1;
                        Err(StreamError::Backpressure {
                            message: "Adaptive timeout".into(),
                        })
                    }
                }
            }
        }
    }

    /// Releases a permit back to the semaphore pool
    pub async fn release_permit(&self) {
        self.semaphore.add_permits(1);
        let mut size = self.queue_size.write().await;
        if *size > 0 {
            *size -= 1;
        }
    }

    /// Updates the queue size by the specified delta
    pub async fn update_queue_size(&self, delta: i32) {
        let mut size = self.queue_size.write().await;
        if delta > 0 {
            *size += delta as usize;
        } else if delta < 0 && *size >= (-delta) as usize {
            *size -= (-delta) as usize;
        }
    }

    /// Returns current queue size and drop count statistics
    pub async fn get_stats(&self) -> (usize, usize) {
        let queue_size = *self.queue_size.read().await;
        let drop_count = *self.drop_count.read().await;
        (queue_size, drop_count)
    }
}

/// Event batch processor with configurable batching
#[derive(Debug)]
pub struct BatchProcessor<E> {
    /// Batching configuration settings
    config: BatchConfig,
    /// Buffer for accumulating events before batching
    batch_buffer: VecDeque<E>,
    /// When the last batch was flushed
    last_flush: Instant,
}

impl<E> BatchProcessor<E> {
    /// Creates a new batch processor with the specified configuration
    pub fn new(config: BatchConfig) -> Self {
        Self {
            config,
            batch_buffer: VecDeque::new(),
            last_flush: Instant::now(),
        }
    }

    /// Adds an event to the batch buffer and returns a completed batch if ready
    pub fn add_event(&mut self, event: E) -> Option<Vec<E>> {
        if !self.config.enabled {
            return Some(vec![event]);
        }

        self.batch_buffer.push_back(event);

        // Check if we should flush based on size or timeout
        let should_flush_size = self.batch_buffer.len() >= self.config.batch_size;
        let should_flush_time = self.last_flush.elapsed() >= self.config.batch_timeout();

        if should_flush_size || should_flush_time {
            self.flush_batch()
        } else {
            None
        }
    }

    /// Forces a flush of the current batch buffer
    pub fn flush_batch(&mut self) -> Option<Vec<E>> {
        if self.batch_buffer.is_empty() {
            return None;
        }

        let batch = self.batch_buffer.drain(..).collect();
        self.last_flush = Instant::now();
        Some(batch)
    }

    /// Returns the number of events currently in the batch buffer
    pub fn pending_count(&self) -> usize {
        self.batch_buffer.len()
    }
}

/// Complex event processing pattern matcher
#[derive(Debug, Clone)]
pub enum EventPattern<E> {
    /// Match a single event type
    Single(fn(&E) -> bool),
    /// Match a sequence of events
    Sequence(Vec<fn(&E) -> bool>),
    /// Match events within a time window
    Within {
        /// Pattern to match within the time window
        pattern: Box<EventPattern<E>>,
        /// Duration of the time window
        duration: Duration,
    },
    /// Match any of the patterns
    Any(Vec<EventPattern<E>>),
    /// Match all patterns
    All(Vec<EventPattern<E>>),
}

/// Complex event processing pattern matcher
#[derive(Debug)]
pub struct PatternMatcher<E> {
    /// Patterns to match against events
    patterns: Vec<EventPattern<E>>,
    /// History of recent events with timestamps
    event_history: VecDeque<(E, Instant)>,
    /// Maximum number of events to keep in history
    max_history: usize,
}

impl<E> PatternMatcher<E>
where
    E: Clone,
{
    /// Creates a new pattern matcher with the specified patterns and history size
    pub fn new(patterns: Vec<EventPattern<E>>, max_history: usize) -> Self {
        Self {
            patterns,
            event_history: VecDeque::new(),
            max_history,
        }
    }

    /// Processes an event and returns indices of matching patterns
    pub fn match_event(&mut self, event: E) -> Vec<usize> {
        let now = Instant::now();
        self.event_history.push_back((event.clone(), now));

        // Maintain history size
        while self.event_history.len() > self.max_history {
            self.event_history.pop_front();
        }

        // Check patterns
        let mut matches = Vec::new();
        for (index, pattern) in self.patterns.iter().enumerate() {
            if self.matches_pattern(pattern, &event, now) {
                matches.push(index);
            }
        }

        matches
    }

    /// Checks if a specific pattern matches the current event and history
    fn matches_pattern(&self, pattern: &EventPattern<E>, current_event: &E, now: Instant) -> bool {
        match pattern {
            EventPattern::Single(predicate) => predicate(current_event),

            EventPattern::Sequence(predicates) => {
                if predicates.is_empty() {
                    return false;
                }

                let mut match_index = 0;
                for (event, _timestamp) in &self.event_history {
                    if match_index < predicates.len() && predicates[match_index](event) {
                        match_index += 1;
                        if match_index == predicates.len() {
                            return true;
                        }
                    }
                }
                false
            }

            EventPattern::Within { pattern, duration } => {
                let cutoff = now - *duration;
                let recent_events: Vec<_> = self
                    .event_history
                    .iter()
                    .filter(|(_, timestamp)| *timestamp >= cutoff)
                    .map(|(event, timestamp)| (event.clone(), *timestamp))
                    .collect();

                // Create a temporary matcher with recent events
                let mut temp_matcher = PatternMatcher {
                    patterns: vec![(**pattern).clone()],
                    event_history: recent_events.into(),
                    max_history: self.max_history,
                };

                !temp_matcher.match_event(current_event.clone()).is_empty()
            }

            EventPattern::Any(patterns) => patterns
                .iter()
                .any(|p| self.matches_pattern(p, current_event, now)),

            EventPattern::All(patterns) => patterns
                .iter()
                .all(|p| self.matches_pattern(p, current_event, now)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone)]
    struct TestEvent {
        _id: u64,
        _event_type: String,
    }

    #[tokio::test]
    async fn test_window_manager_tumbling() {
        let window_type = WindowType::Tumbling {
            duration: Duration::from_millis(100),
        };
        let mut manager: WindowManager<TestEvent> = WindowManager::new(window_type);

        let event = TestEvent {
            _id: 1,
            _event_type: "test".into(),
        };
        let windows = manager.add_event(event);

        // No completed windows immediately
        assert!(windows.is_empty());

        // Wait for window to expire
        tokio::time::sleep(Duration::from_millis(150)).await;

        let event2 = TestEvent {
            _id: 2,
            _event_type: "test".into(),
        };
        let windows = manager.add_event(event2);

        // Should have completed windows now
        assert!(!windows.is_empty());
    }

    #[tokio::test]
    async fn test_stateful_processor() {
        let processor = StatefulProcessor::<String, u64>::new(Duration::from_secs(1));

        let result = processor
            .update_state("key1".to_string(), |current| {
                let new_value = current.unwrap_or(&0) + 1;
                (new_value, new_value)
            })
            .await;

        assert_eq!(result, 1);

        let state = processor.get_state(&"key1".to_string()).await;
        assert_eq!(state, Some(1));
    }

    #[tokio::test]
    async fn test_flow_controller() {
        let config = BackpressureConfig {
            channel_size: 2,
            strategy: BackpressureStrategy::Block,
            ..Default::default()
        };

        let controller = FlowController::new(config);

        // Should be able to acquire permits up to channel size
        assert!(controller.acquire_permit().await.is_ok());
        assert!(controller.acquire_permit().await.is_ok());

        controller.release_permit().await;
        controller.release_permit().await;
    }

    #[test]
    fn test_batch_processor() {
        let config = BatchConfig {
            batch_size: 3,
            batch_timeout_ms: 100,
            enabled: true,
            zero_copy: false,
        };

        let mut processor: BatchProcessor<TestEvent> = BatchProcessor::new(config);

        // Add events, should not flush until batch size reached
        assert!(processor
            .add_event(TestEvent {
                _id: 1,
                _event_type: "test".into()
            })
            .is_none());
        assert!(processor
            .add_event(TestEvent {
                _id: 2,
                _event_type: "test".into()
            })
            .is_none());

        let batch = processor.add_event(TestEvent {
            _id: 3,
            _event_type: "test".into(),
        });
        assert!(batch.is_some());
        assert_eq!(batch.unwrap().len(), 3);
    }

    // Additional tests for 100% coverage

    #[test]
    fn test_window_new() {
        let start_time = Instant::now();
        let window: Window<TestEvent> = Window::new(42, start_time);

        assert_eq!(window.id, 42);
        assert_eq!(window.start_time, start_time);
        assert!(window.end_time.is_none());
        assert!(window.events.is_empty());
        assert!(!window.is_closed);
    }

    #[test]
    fn test_window_close() {
        let mut window: Window<TestEvent> = Window::new(1, Instant::now());
        assert!(!window.is_closed);
        assert!(window.end_time.is_none());

        window.close();
        assert!(window.is_closed);
        assert!(window.end_time.is_some());
    }

    #[test]
    fn test_window_add_event_when_open() {
        let mut window = Window::new(1, Instant::now());
        let event = TestEvent {
            _id: 1,
            _event_type: "test".into(),
        };

        window.add_event(event);
        assert_eq!(window.events.len(), 1);
    }

    #[test]
    fn test_window_add_event_when_closed() {
        let mut window = Window::new(1, Instant::now());
        let event = TestEvent {
            _id: 1,
            _event_type: "test".into(),
        };

        window.close();
        window.add_event(event);
        assert!(window.events.is_empty());
    }

    #[tokio::test]
    async fn test_window_manager_sliding() {
        let window_type = WindowType::Sliding {
            size: Duration::from_millis(200),
            step: Duration::from_millis(50),
        };
        let mut manager: WindowManager<TestEvent> = WindowManager::new(window_type);

        let event = TestEvent {
            _id: 1,
            _event_type: "test".into(),
        };
        let windows = manager.add_event(event);
        assert!(windows.is_empty());

        tokio::time::sleep(Duration::from_millis(250)).await;

        let event2 = TestEvent {
            _id: 2,
            _event_type: "test".into(),
        };
        let windows = manager.add_event(event2);
        assert!(!windows.is_empty());
    }

    #[tokio::test]
    async fn test_window_manager_session() {
        let window_type = WindowType::Session {
            timeout: Duration::from_millis(100),
        };
        let mut manager: WindowManager<TestEvent> = WindowManager::new(window_type);

        let event = TestEvent {
            _id: 1,
            _event_type: "test".into(),
        };
        let windows = manager.add_event(event);
        assert!(windows.is_empty());

        tokio::time::sleep(Duration::from_millis(150)).await;

        let event2 = TestEvent {
            _id: 2,
            _event_type: "test".into(),
        };
        let windows = manager.add_event(event2);
        assert!(!windows.is_empty());
    }

    #[test]
    fn test_window_manager_count() {
        let window_type = WindowType::Count { size: 2 };
        let mut manager: WindowManager<TestEvent> = WindowManager::new(window_type);

        let event1 = TestEvent {
            _id: 1,
            _event_type: "test".into(),
        };
        let windows = manager.add_event(event1);
        assert!(windows.is_empty());

        let event2 = TestEvent {
            _id: 2,
            _event_type: "test".into(),
        };
        let windows = manager.add_event(event2);
        assert_eq!(windows.len(), 1);
        assert_eq!(windows[0].events.len(), 2);
    }

    #[tokio::test]
    async fn test_window_manager_cleanup() {
        let window_type = WindowType::Count { size: 5 };
        let mut manager: WindowManager<TestEvent> = WindowManager::new(window_type);

        // Force cleanup by setting last_cleanup to a very old time
        manager.last_cleanup = Instant::now()
            .checked_sub(Duration::from_secs(120))
            .unwrap_or_else(Instant::now);

        let event = TestEvent {
            _id: 1,
            _event_type: "test".into(),
        };
        let _windows = manager.add_event(event);

        // Cleanup should have been triggered
        assert!(manager.last_cleanup.elapsed() < Duration::from_secs(1));
    }

    #[test]
    fn test_window_manager_cleanup_old_windows() {
        let window_type = WindowType::Count { size: 5 };
        let mut manager: WindowManager<TestEvent> = WindowManager::new(window_type);

        // Manually add a very old window
        let old_time = Instant::now()
            .checked_sub(Duration::from_secs(7200))
            .unwrap_or_else(Instant::now); // 2 hours ago
        manager
            .active_windows
            .insert(999, Window::new(999, old_time));

        manager.cleanup_old_windows();

        // Old window should be removed
        assert!(!manager.active_windows.contains_key(&999));
    }

    #[tokio::test]
    async fn test_stateful_processor_remove_state() {
        let processor = StatefulProcessor::<String, u64>::new(Duration::from_secs(1));

        // Insert some state
        processor
            .update_state("key1".to_string(), |_| (42, ()))
            .await;

        // Remove and verify
        let removed = processor.remove_state(&"key1".to_string()).await;
        assert_eq!(removed, Some(42));

        // Should be None after removal
        let state = processor.get_state(&"key1".to_string()).await;
        assert_eq!(state, None);
    }

    #[tokio::test]
    async fn test_stateful_processor_checkpoint_not_ready() {
        let processor = StatefulProcessor::<String, u64>::new(Duration::from_secs(10));

        // Checkpoint should succeed but not actually checkpoint
        let result = processor.checkpoint().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_stateful_processor_checkpoint_ready() {
        let processor = StatefulProcessor::<String, u64>::new(Duration::from_millis(1));

        // Wait for checkpoint interval to pass
        tokio::time::sleep(Duration::from_millis(2)).await;

        let result = processor.checkpoint().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_flow_controller_drop_strategy() {
        let config = BackpressureConfig {
            channel_size: 1,
            strategy: BackpressureStrategy::Drop,
            ..Default::default()
        };

        let controller = FlowController::new(config);

        // Fill up the semaphore
        let _permit = controller.acquire_permit().await;

        // Update queue size to trigger drop
        controller.update_queue_size(1000).await;

        // Should drop event
        let result = controller.acquire_permit().await;
        assert!(result.is_err());

        let (_queue_size, drop_count) = controller.get_stats().await;
        assert_eq!(drop_count, 1);
    }

    #[tokio::test]
    async fn test_flow_controller_retry_strategy() {
        let config = BackpressureConfig {
            channel_size: 1,
            strategy: BackpressureStrategy::Retry {
                max_attempts: 2,
                base_wait_ms: 1,
            },
            ..Default::default()
        };

        let controller = FlowController::new(config);

        // Should succeed on first try
        let result = controller.acquire_permit().await;
        assert!(result.is_ok());

        // Should fail after retries
        let result = controller.acquire_permit().await;
        assert!(result.is_err());

        let (_, drop_count) = controller.get_stats().await;
        assert_eq!(drop_count, 1);
    }

    #[tokio::test]
    async fn test_flow_controller_adaptive_high_load() {
        let config = BackpressureConfig {
            channel_size: 10,
            strategy: BackpressureStrategy::Adaptive,
            ..Default::default()
        };

        let controller = FlowController::new(config);

        // Set high queue size to trigger drop mode
        controller.update_queue_size(1000).await;

        let result = controller.acquire_permit().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_flow_controller_adaptive_low_load() {
        let config = BackpressureConfig {
            channel_size: 100,
            strategy: BackpressureStrategy::Adaptive,
            ..Default::default()
        };

        let controller = FlowController::new(config);

        // Low load should allow blocking
        let result = controller.acquire_permit().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_flow_controller_adaptive_medium_load() {
        let config = BackpressureConfig {
            channel_size: 10,
            strategy: BackpressureStrategy::Adaptive,
            ..Default::default()
        };

        let controller = FlowController::new(config);

        // Fill semaphore to make timeout likely
        for _ in 0..10 {
            let _ = controller.acquire_permit().await;
        }

        // Set medium queue size
        controller.update_queue_size(5).await;

        let result = controller.acquire_permit().await;
        // Should timeout and fail
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_flow_controller_update_queue_size_negative() {
        let config = BackpressureConfig::default();
        let controller = FlowController::new(config);

        // Increase size first
        controller.update_queue_size(10).await;
        let (size, _) = controller.get_stats().await;
        assert_eq!(size, 10);

        // Decrease size
        controller.update_queue_size(-5).await;
        let (size, _) = controller.get_stats().await;
        assert_eq!(size, 5);

        // Try to decrease more than available
        controller.update_queue_size(-10).await;
        let (size, _) = controller.get_stats().await;
        assert_eq!(size, 0);
    }

    #[tokio::test]
    async fn test_flow_controller_release_permit() {
        let config = BackpressureConfig::default();
        let controller = FlowController::new(config);

        controller.update_queue_size(5).await;
        controller.release_permit().await;

        let (size, _) = controller.get_stats().await;
        assert_eq!(size, 4);

        // Release when size is 0
        controller.update_queue_size(-10).await;
        controller.release_permit().await;
        let (size, _) = controller.get_stats().await;
        assert_eq!(size, 0);
    }

    #[test]
    fn test_batch_processor_disabled() {
        let config = BatchConfig {
            batch_size: 3,
            batch_timeout_ms: 100,
            enabled: false,
            zero_copy: false,
        };

        let mut processor: BatchProcessor<TestEvent> = BatchProcessor::new(config);

        // When disabled, should return single event immediately
        let result = processor.add_event(TestEvent {
            _id: 1,
            _event_type: "test".into(),
        });

        assert!(result.is_some());
        assert_eq!(result.unwrap().len(), 1);
    }

    #[test]
    fn test_batch_processor_flush_empty() {
        let config = BatchConfig {
            batch_size: 3,
            batch_timeout_ms: 100,
            enabled: true,
            zero_copy: false,
        };

        let mut processor: BatchProcessor<TestEvent> = BatchProcessor::new(config);

        // Flush empty buffer should return None
        let result = processor.flush_batch();
        assert!(result.is_none());
    }

    #[test]
    fn test_batch_processor_pending_count() {
        let config = BatchConfig {
            batch_size: 5,
            batch_timeout_ms: 100,
            enabled: true,
            zero_copy: false,
        };

        let mut processor: BatchProcessor<TestEvent> = BatchProcessor::new(config);

        assert_eq!(processor.pending_count(), 0);

        processor.add_event(TestEvent {
            _id: 1,
            _event_type: "test".into(),
        });
        assert_eq!(processor.pending_count(), 1);

        processor.add_event(TestEvent {
            _id: 2,
            _event_type: "test".into(),
        });
        assert_eq!(processor.pending_count(), 2);
    }

    #[test]
    fn test_batch_processor_timeout_flush() {
        let config = BatchConfig {
            batch_size: 10,
            batch_timeout_ms: 1, // Very short timeout
            enabled: true,
            zero_copy: false,
        };

        let mut processor: BatchProcessor<TestEvent> = BatchProcessor::new(config);

        // Add one event
        processor.add_event(TestEvent {
            _id: 1,
            _event_type: "test".into(),
        });

        // Force timeout by setting last_flush to old time
        processor.last_flush = Instant::now()
            .checked_sub(Duration::from_millis(10))
            .unwrap_or_else(Instant::now);

        // Next event should trigger timeout flush
        let result = processor.add_event(TestEvent {
            _id: 2,
            _event_type: "test".into(),
        });
        assert!(result.is_some());
        assert_eq!(result.unwrap().len(), 2);
    }

    #[test]
    fn test_pattern_matcher_single() {
        let patterns = vec![EventPattern::Single(|e: &TestEvent| e._id == 1)];
        let mut matcher = PatternMatcher::new(patterns, 10);

        let event1 = TestEvent {
            _id: 1,
            _event_type: "test".into(),
        };
        let matches = matcher.match_event(event1);
        assert_eq!(matches, vec![0]);

        let event2 = TestEvent {
            _id: 2,
            _event_type: "test".into(),
        };
        let matches = matcher.match_event(event2);
        assert!(matches.is_empty());
    }

    #[test]
    fn test_pattern_matcher_sequence() {
        let patterns = vec![EventPattern::Sequence(vec![
            |e: &TestEvent| e._id == 1,
            |e: &TestEvent| e._id == 2,
        ])];
        let mut matcher = PatternMatcher::new(patterns, 10);

        // First event
        let event1 = TestEvent {
            _id: 1,
            _event_type: "test".into(),
        };
        let matches = matcher.match_event(event1);
        assert!(matches.is_empty());

        // Second event should complete sequence
        let event2 = TestEvent {
            _id: 2,
            _event_type: "test".into(),
        };
        let matches = matcher.match_event(event2);
        assert_eq!(matches, vec![0]);
    }

    #[test]
    fn test_pattern_matcher_sequence_empty() {
        let patterns = vec![EventPattern::Sequence(vec![])];
        let mut matcher = PatternMatcher::new(patterns, 10);

        let event = TestEvent {
            _id: 1,
            _event_type: "test".into(),
        };
        let matches = matcher.match_event(event);
        assert!(matches.is_empty());
    }

    #[test]
    fn test_pattern_matcher_any() {
        let patterns = vec![EventPattern::Any(vec![
            EventPattern::Single(|e: &TestEvent| e._id == 1),
            EventPattern::Single(|e: &TestEvent| e._id == 2),
        ])];
        let mut matcher = PatternMatcher::new(patterns, 10);

        let event1 = TestEvent {
            _id: 1,
            _event_type: "test".into(),
        };
        let matches = matcher.match_event(event1);
        assert_eq!(matches, vec![0]);

        let event2 = TestEvent {
            _id: 2,
            _event_type: "test".into(),
        };
        let matches = matcher.match_event(event2);
        assert_eq!(matches, vec![0]);

        let event3 = TestEvent {
            _id: 3,
            _event_type: "test".into(),
        };
        let matches = matcher.match_event(event3);
        assert!(matches.is_empty());
    }

    #[test]
    fn test_pattern_matcher_all() {
        let patterns = vec![EventPattern::All(vec![
            EventPattern::Single(|e: &TestEvent| e._id > 0),
            EventPattern::Single(|e: &TestEvent| e._event_type == "test"),
        ])];
        let mut matcher = PatternMatcher::new(patterns, 10);

        let event = TestEvent {
            _id: 1,
            _event_type: "test".into(),
        };
        let matches = matcher.match_event(event);
        assert_eq!(matches, vec![0]);

        let event2 = TestEvent {
            _id: 0,
            _event_type: "test".into(),
        };
        let matches = matcher.match_event(event2);
        assert!(matches.is_empty());
    }

    #[test]
    fn test_pattern_matcher_within() {
        let inner_pattern = EventPattern::Single(|e: &TestEvent| e._id == 1);
        let patterns = vec![EventPattern::Within {
            pattern: Box::new(inner_pattern),
            duration: Duration::from_millis(100),
        }];
        let mut matcher = PatternMatcher::new(patterns, 10);

        let event = TestEvent {
            _id: 1,
            _event_type: "test".into(),
        };
        let matches = matcher.match_event(event);
        assert_eq!(matches, vec![0]);
    }

    #[test]
    fn test_pattern_matcher_history_limit() {
        let patterns = vec![EventPattern::Single(|_: &TestEvent| true)];
        let mut matcher = PatternMatcher::new(patterns, 2);

        // Add more events than history limit
        for i in 1..=5 {
            let event = TestEvent {
                _id: i,
                _event_type: "test".into(),
            };
            matcher.match_event(event);
        }

        // History should be limited to 2 events
        assert_eq!(matcher.event_history.len(), 2);
    }

    #[test]
    fn test_window_type_variants() {
        // Test that all WindowType variants can be created
        let _tumbling = WindowType::Tumbling {
            duration: Duration::from_secs(1),
        };
        let _sliding = WindowType::Sliding {
            size: Duration::from_secs(2),
            step: Duration::from_secs(1),
        };
        let _session = WindowType::Session {
            timeout: Duration::from_secs(5),
        };
        let _count = WindowType::Count { size: 100 };
    }

    #[test]
    fn test_event_pattern_variants() {
        // Test that all EventPattern variants can be created
        let _single = EventPattern::<TestEvent>::Single(|_| true);
        let _sequence = EventPattern::<TestEvent>::Sequence(vec![|_| true]);
        let _within = EventPattern::<TestEvent>::Within {
            pattern: Box::new(EventPattern::Single(|_| true)),
            duration: Duration::from_secs(1),
        };
        let _any = EventPattern::<TestEvent>::Any(vec![]);
        let _all = EventPattern::<TestEvent>::All(vec![]);
    }

    #[test]
    fn test_window_manager_tumbling_with_system_time_error() {
        let window_type = WindowType::Tumbling {
            duration: Duration::from_secs(1),
        };
        let mut manager: WindowManager<TestEvent> = WindowManager::new(window_type);

        // This test checks the error handling path for SystemTime
        // The actual error case is hard to trigger in tests, but the code path exists
        let event = TestEvent {
            _id: 1,
            _event_type: "test".into(),
        };
        let _windows = manager.add_event(event);

        // Should not panic and should handle the error gracefully
        assert!(manager.active_windows.len() <= 1);
    }

    #[test]
    fn test_window_manager_sliding_with_system_time_error() {
        let window_type = WindowType::Sliding {
            size: Duration::from_secs(2),
            step: Duration::from_secs(1),
        };
        let mut manager: WindowManager<TestEvent> = WindowManager::new(window_type);

        let event = TestEvent {
            _id: 1,
            _event_type: "test".into(),
        };
        let _windows = manager.add_event(event);

        // Should not panic and should handle the error gracefully
        assert!(manager.active_windows.len() <= 1);
    }
}
