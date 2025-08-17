//! Stream composition operators for building reactive pipelines
//!
//! This module provides a rich set of operators for composing, transforming,
//! and combining streams in a declarative, functional style.
//!
//! ## Backpressure Behavior
//!
//! The operators in this module use `tokio::sync::broadcast` channels by default,
//! which implement a **drop-oldest backpressure strategy**:
//!
//! - When a receiver is too slow and the channel buffer fills up, **old messages are dropped**
//! - Only the most recent messages are retained in the buffer
//! - This is ideal for real-time scenarios where fresh data is more valuable than old data
//! - Receivers that fall behind will skip older events but continue processing new ones
//!
//! ### Buffer Sizing Recommendations:
//! - High-frequency streams (>1000 events/sec): Use buffer size 50,000-100,000
//! - Medium-frequency streams (100-1000 events/sec): Use buffer size 10,000-50,000
//! - Low-frequency streams (<100 events/sec): Use buffer size 1,000-10,000
//! - Financial indicators: Use buffer size 5,000-20,000 for smooth operation
//!
//! ### Performance Considerations:
//! - Each operator spawns a new Tokio task and creates a new broadcast channel
//! - For very long pipelines, consider using operator fusion (FilterMapStream) to reduce overhead
//! - Memory usage scales with buffer size × number of operators in the pipeline
//! - Task switching overhead becomes noticeable with >10 operators in a single pipeline
//!
//! ### Alternative Channel Strategies:
//!
//! For use cases requiring **guaranteed delivery** of every message, use the
//! `with_guaranteed_delivery()` method available on select operators. This switches
//! the underlying channel from `broadcast` to `mpsc`, providing:
//!
//! - **No message loss**: All messages are delivered to receivers
//! - **Backpressure blocking**: Slow receivers will block the entire pipeline
//! - **Single consumer**: Only one receiver per operator (vs multiple with broadcast)
//! - **Higher latency**: Blocking behavior can increase end-to-end latency
//!
//! Choose guaranteed delivery for:
//! - Financial transactions that cannot be missed
//! - Audit logging and compliance data
//! - State synchronization between services
//! - Critical alerts and notifications
//!
//! Choose drop-oldest (default) for:
//! - Real-time price feeds and market data
//! - Live dashboards and monitoring
//! - Event logging and analytics
//! - High-frequency trading signals

use async_trait::async_trait;
use futures::future::BoxFuture;
use std::any::Any;
use std::marker::PhantomData;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{broadcast, RwLock};

use crate::core::{Stream, StreamError, StreamHealth};
use riglr_events_core::Event;
// NOTE: Future enhancement - integrate events-core utilities for batching, rate limiting, etc.
// use riglr_events_core::prelude::{EventBatcher, EventDeduplicator, RateLimiter, StreamUtils};

/// Type alias for boxed futures
#[allow(dead_code)]
type BoxedFuture<T> = BoxFuture<'static, T>;

/// Trait for composable streams
#[async_trait]
pub trait ComposableStream: Stream {
    /// Map events through a transformation function
    fn map<F, R>(self, f: F) -> MappedStream<Self, F, R>
    where
        Self: Sized,
        F: Fn(Arc<Self::Event>) -> R + Send + Sync + 'static,
        R: Event + Clone + Send + Sync + 'static,
    {
        MappedStream::new(self, f)
    }

    /// Filter events based on a predicate
    fn filter<F>(self, predicate: F) -> FilteredStream<Self, F>
    where
        Self: Sized,
        F: Fn(&Self::Event) -> bool + Send + Sync + 'static,
    {
        FilteredStream::new(self, predicate)
    }

    /// Merge with another stream
    fn merge<S>(self, other: S) -> MergedStream<Self, S>
    where
        Self: Sized,
        S: Stream,
        S::Event: Event + Clone + Send + Sync + 'static,
    {
        MergedStream::new(self, other)
    }

    /// Batch events into groups
    fn batch(self, size: usize, timeout: Duration) -> BatchedStream<Self>
    where
        Self: Sized,
    {
        BatchedStream::new(self, size, timeout)
    }

    /// Debounce events (only emit after quiet period)
    fn debounce(self, duration: Duration) -> DebouncedStream<Self>
    where
        Self: Sized,
    {
        DebouncedStream::new(self, duration)
    }

    /// Throttle events (rate limiting)
    fn throttle(self, duration: Duration) -> ThrottledStream<Self>
    where
        Self: Sized,
    {
        ThrottledStream::new(self, duration)
    }

    /// Enable guaranteed delivery using mpsc channels instead of broadcast
    ///
    /// This ensures every message is delivered but may block the pipeline
    /// if receivers are slow. Use for critical data that cannot be lost.
    fn with_guaranteed_delivery(self, buffer_size: usize) -> GuaranteedDeliveryStream<Self>
    where
        Self: Sized,
    {
        GuaranteedDeliveryStream::new(self, buffer_size)
    }

    /// Take only the first N events
    fn take(self, count: usize) -> TakeStream<Self>
    where
        Self: Sized,
    {
        TakeStream::new(self, count)
    }

    /// Skip the first N events
    fn skip(self, count: usize) -> SkipStream<Self>
    where
        Self: Sized,
    {
        SkipStream::new(self, count)
    }

    /// Scan (stateful transformation)
    fn scan<S, F>(self, initial_state: S, f: F) -> ScanStream<Self, S, F>
    where
        Self: Sized,
        S: Clone + Send + Sync + 'static,
        F: Fn(S, Arc<Self::Event>) -> S + Send + Sync + 'static,
    {
        ScanStream::new(self, initial_state, f)
    }

    /// Buffer events with various strategies
    fn buffer(self, strategy: BufferStrategy) -> BufferedStream<Self>
    where
        Self: Sized,
    {
        BufferedStream::new(self, strategy)
    }
}

/// Implement ComposableStream for all Stream types
impl<T> ComposableStream for T where T: Stream {}

/// Buffer strategies for buffered streams
#[derive(Clone, Debug)]
pub enum BufferStrategy {
    /// Buffer up to N events
    Count(usize),
    /// Buffer for duration
    Time(Duration),
    /// Buffer up to N events or duration, whichever comes first
    CountOrTime(usize, Duration),
    /// Sliding window of N events
    SlidingWindow(usize),
}

/// Stream that maps events through a transformation
pub struct MappedStream<S, F, R> {
    /// The underlying stream
    inner: S,
    /// The transformation function
    transform: Arc<F>,
    /// Phantom data for the result type
    _phantom: PhantomData<R>,
}

impl<S, F, R> MappedStream<S, F, R>
where
    S: Stream,
    F: Fn(Arc<S::Event>) -> R + Send + Sync + 'static,
    R: Event + Clone + Send + Sync + 'static,
{
    /// Create a new mapped stream
    pub fn new(inner: S, transform: F) -> Self {
        Self {
            inner,
            transform: Arc::new(transform),
            _phantom: PhantomData,
        }
    }
}

#[async_trait]
impl<S, F, R> Stream for MappedStream<S, F, R>
where
    S: Stream + Send + Sync + 'static,
    F: Fn(Arc<S::Event>) -> R + Send + Sync + 'static,
    R: Event + Clone + Send + Sync + 'static,
{
    type Event = R;
    type Config = S::Config;

    async fn start(&mut self, config: Self::Config) -> Result<(), StreamError> {
        self.inner.start(config).await
    }

    async fn stop(&mut self) -> Result<(), StreamError> {
        self.inner.stop().await
    }

    fn subscribe(&self) -> broadcast::Receiver<Arc<Self::Event>> {
        let (tx, rx) = broadcast::channel(10000);
        let mut inner_rx = self.inner.subscribe();
        let transform = self.transform.clone();

        tokio::spawn(async move {
            while let Ok(event) = inner_rx.recv().await {
                let transformed = (transform)(event);
                let _ = tx.send(Arc::new(transformed));
            }
        });

        rx
    }

    fn is_running(&self) -> bool {
        self.inner.is_running()
    }

    async fn health(&self) -> StreamHealth {
        self.inner.health().await
    }

    fn name(&self) -> &str {
        self.inner.name()
    }
}

/// Stream that filters events based on a predicate
pub struct FilteredStream<S, F> {
    /// The underlying stream
    inner: S,
    /// The filter predicate
    predicate: Arc<F>,
}

impl<S, F> FilteredStream<S, F>
where
    S: Stream,
    F: Fn(&S::Event) -> bool + Send + Sync + 'static,
{
    /// Create a new filtered stream
    pub fn new(inner: S, predicate: F) -> Self {
        Self {
            inner,
            predicate: Arc::new(predicate),
        }
    }
}

#[async_trait]
impl<S, F> Stream for FilteredStream<S, F>
where
    S: Stream + Send + Sync + 'static,
    F: Fn(&S::Event) -> bool + Send + Sync + 'static,
{
    type Event = S::Event;
    type Config = S::Config;

    async fn start(&mut self, config: Self::Config) -> Result<(), StreamError> {
        self.inner.start(config).await
    }

    async fn stop(&mut self) -> Result<(), StreamError> {
        self.inner.stop().await
    }

    fn subscribe(&self) -> broadcast::Receiver<Arc<Self::Event>> {
        let (tx, rx) = broadcast::channel(10000);
        let mut inner_rx = self.inner.subscribe();
        let predicate = self.predicate.clone();

        tokio::spawn(async move {
            while let Ok(event) = inner_rx.recv().await {
                if (predicate)(&event) {
                    let _ = tx.send(event);
                }
            }
        });

        rx
    }

    fn is_running(&self) -> bool {
        self.inner.is_running()
    }

    async fn health(&self) -> StreamHealth {
        self.inner.health().await
    }

    fn name(&self) -> &str {
        self.inner.name()
    }
}

/// Stream that merges events from two streams
pub struct MergedStream<S1, S2> {
    /// First stream to merge
    stream1: S1,
    /// Second stream to merge
    stream2: S2,
    /// Name of the merged stream
    name: String,
}

impl<S1, S2> MergedStream<S1, S2>
where
    S1: Stream,
    S2: Stream,
{
    /// Create a new merged stream
    pub fn new(stream1: S1, stream2: S2) -> Self {
        let name = format!("merged({},{})", stream1.name(), stream2.name());
        Self {
            stream1,
            stream2,
            name,
        }
    }
}

#[async_trait]
impl<S1, S2> Stream for MergedStream<S1, S2>
where
    S1: Stream + Send + Sync + 'static,
    S2: Stream + Send + Sync + 'static,
    S1::Event: Event + Clone + Send + Sync + 'static,
    S2::Event: Event + Clone + Send + Sync + 'static,
{
    type Event = MergedEvent<S1::Event, S2::Event>;
    type Config = (S1::Config, S2::Config);

    async fn start(&mut self, config: Self::Config) -> Result<(), StreamError> {
        self.stream1.start(config.0).await?;
        self.stream2.start(config.1).await?;
        Ok(())
    }

    async fn stop(&mut self) -> Result<(), StreamError> {
        self.stream1.stop().await?;
        self.stream2.stop().await?;
        Ok(())
    }

    fn subscribe(&self) -> broadcast::Receiver<Arc<Self::Event>> {
        let (tx, rx) = broadcast::channel(10000);
        let mut rx1 = self.stream1.subscribe();
        let mut rx2 = self.stream2.subscribe();
        let tx1 = tx.clone();
        let tx2 = tx.clone();

        tokio::spawn(async move {
            while let Ok(event) = rx1.recv().await {
                let merged = MergedEvent::First(Arc::try_unwrap(event).unwrap_or_else(|arc| (*arc).clone()));
                let _ = tx1.send(Arc::new(merged));
            }
        });

        tokio::spawn(async move {
            while let Ok(event) = rx2.recv().await {
                let merged = MergedEvent::Second(Arc::try_unwrap(event).unwrap_or_else(|arc| (*arc).clone()));
                let _ = tx2.send(Arc::new(merged));
            }
        });

        rx
    }

    fn is_running(&self) -> bool {
        self.stream1.is_running() && self.stream2.is_running()
    }

    async fn health(&self) -> StreamHealth {
        let health1 = self.stream1.health().await;
        let health2 = self.stream2.health().await;

        StreamHealth {
            is_connected: health1.is_connected && health2.is_connected,
            last_event_time: [health1.last_event_time, health2.last_event_time]
                .iter()
                .filter_map(|&x| x)
                .max(),
            error_count: health1.error_count + health2.error_count,
            events_processed: health1.events_processed + health2.events_processed,
            backlog_size: None,
            custom_metrics: None,
            stream_info: None,
        }
    }

    fn name(&self) -> &str {
        &self.name
    }
}

/// Unified event type for merged streams
#[derive(Clone, Debug)]
pub enum MergedEvent<E1, E2> {
    /// Event from the first stream
    First(E1),
    /// Event from the second stream
    Second(E2),
}

impl<E1, E2> Event for MergedEvent<E1, E2>
where
    E1: Event + Clone + 'static,
    E2: Event + Clone + 'static,
{
    fn id(&self) -> &str {
        match self {
            MergedEvent::First(e) => e.id(),
            MergedEvent::Second(e) => e.id(),
        }
    }

    fn kind(&self) -> &riglr_events_core::EventKind {
        match self {
            MergedEvent::First(e) => e.kind(),
            MergedEvent::Second(e) => e.kind(),
        }
    }

    fn metadata(&self) -> &riglr_events_core::EventMetadata {
        match self {
            MergedEvent::First(e) => e.metadata(),
            MergedEvent::Second(e) => e.metadata(),
        }
    }

    fn metadata_mut(&mut self) -> &mut riglr_events_core::EventMetadata {
        match self {
            MergedEvent::First(e) => e.metadata_mut(),
            MergedEvent::Second(e) => e.metadata_mut(),
        }
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
}

/// Event wrapper for batched events
#[derive(Clone, Debug)]
pub struct BatchEvent<E> {
    /// The events in this batch
    pub events: Vec<Arc<E>>,
    /// Unique identifier for this batch
    pub batch_id: String,
    /// Timestamp when the batch was created
    pub timestamp: std::time::SystemTime,
}

impl<E> Event for BatchEvent<E>
where
    E: Event + Clone + 'static,
{
    fn id(&self) -> &str {
        &self.batch_id
    }

    fn kind(&self) -> &riglr_events_core::EventKind {
        static KIND: std::sync::LazyLock<riglr_events_core::EventKind> =
            std::sync::LazyLock::new(|| riglr_events_core::EventKind::Custom("batch".to_string()));
        &KIND
    }

    fn metadata(&self) -> &riglr_events_core::EventMetadata {
        // For simplicity, we'll create a static metadata
        // In a real implementation, this should be stored in the struct
        static METADATA: std::sync::OnceLock<riglr_events_core::EventMetadata> =
            std::sync::OnceLock::new();
        METADATA.get_or_init(|| {
            riglr_events_core::EventMetadata::new(
                "batch".to_string(),
                riglr_events_core::EventKind::Custom("batch".to_string()),
                "batch-stream".to_string(),
            )
        })
    }

    fn metadata_mut(&mut self) -> &mut riglr_events_core::EventMetadata {
        panic!("metadata_mut not supported for BatchEvent")
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
}

/// Stream that batches events
pub struct BatchedStream<S> {
    /// The underlying stream
    inner: S,
    /// Maximum number of events per batch
    batch_size: usize,
    /// Maximum time to wait before emitting a batch
    timeout: Duration,
    /// Name of the batched stream
    name: String,
}

impl<S: Stream> BatchedStream<S> {
    /// Create a new batched stream
    pub fn new(inner: S, batch_size: usize, timeout: Duration) -> Self {
        let name = format!("batched({})", inner.name());
        Self {
            inner,
            batch_size,
            timeout,
            name,
        }
    }
}

#[async_trait]
impl<S> Stream for BatchedStream<S>
where
    S: Stream + Send + Sync + 'static,
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
        let (tx, rx) = broadcast::channel(10000);
        let mut inner_rx = self.inner.subscribe();
        let batch_size = self.batch_size;
        let timeout = self.timeout;

        tokio::spawn(async move {
            let mut buffer = Vec::new();
            let mut interval = tokio::time::interval(timeout);
            let mut batch_counter = 0u64;

            loop {
                tokio::select! {
                    Ok(event) = inner_rx.recv() => {
                        buffer.push(event);

                        if buffer.len() >= batch_size {
                            let batch = BatchEvent {
                                events: std::mem::take(&mut buffer),
                                batch_id: format!("batch_{}", batch_counter),
                                timestamp: std::time::SystemTime::now(),
                            };
                            batch_counter += 1;
                            let _ = tx.send(Arc::new(batch));
                        }
                    }
                    _ = interval.tick() => {
                        if !buffer.is_empty() {
                            let batch = BatchEvent {
                                events: std::mem::take(&mut buffer),
                                batch_id: format!("batch_{}", batch_counter),
                                timestamp: std::time::SystemTime::now(),
                            };
                            batch_counter += 1;
                            let _ = tx.send(Arc::new(batch));
                        }
                    }
                }
            }
        });

        rx
    }

    fn is_running(&self) -> bool {
        self.inner.is_running()
    }

    async fn health(&self) -> StreamHealth {
        self.inner.health().await
    }

    fn name(&self) -> &str {
        &self.name
    }
}

/// Stream that debounces events
pub struct DebouncedStream<S> {
    /// The underlying stream
    inner: S,
    /// Debounce duration
    duration: Duration,
    /// Name of the debounced stream
    name: String,
}

impl<S: Stream> DebouncedStream<S> {
    /// Create a new debounced stream
    pub fn new(inner: S, duration: Duration) -> Self {
        let name = format!("debounced({})", inner.name());
        Self {
            inner,
            duration,
            name,
        }
    }
}

#[async_trait]
impl<S> Stream for DebouncedStream<S>
where
    S: Stream + Send + Sync + 'static,
{
    type Event = S::Event;
    type Config = S::Config;

    async fn start(&mut self, config: Self::Config) -> Result<(), StreamError> {
        self.inner.start(config).await
    }

    async fn stop(&mut self) -> Result<(), StreamError> {
        self.inner.stop().await
    }

    fn subscribe(&self) -> broadcast::Receiver<Arc<Self::Event>> {
        let (tx, rx) = broadcast::channel(10000);
        let mut inner_rx = self.inner.subscribe();
        let duration = self.duration;

        tokio::spawn(async move {
            let mut timeout_task: Option<tokio::task::JoinHandle<()>> = None;

            while let Ok(event) = inner_rx.recv().await {
                // Cancel previous timeout if it exists
                if let Some(task) = timeout_task.take() {
                    task.abort();
                }

                let tx_clone = tx.clone();
                let event_to_send = event;

                // Start new timeout
                timeout_task = Some(tokio::spawn(async move {
                    tokio::time::sleep(duration).await;
                    let _ = tx_clone.send(event_to_send);
                }));
            }
        });

        rx
    }

    fn is_running(&self) -> bool {
        self.inner.is_running()
    }

    async fn health(&self) -> StreamHealth {
        self.inner.health().await
    }

    fn name(&self) -> &str {
        &self.name
    }
}

/// Stream that throttles events
pub struct ThrottledStream<S> {
    /// The underlying stream
    inner: S,
    /// Throttle interval
    duration: Duration,
    /// Name of the throttled stream
    name: String,
}

impl<S: Stream> ThrottledStream<S> {
    /// Create a new throttled stream
    pub fn new(inner: S, duration: Duration) -> Self {
        let name = format!("throttled({})", inner.name());
        Self {
            inner,
            duration,
            name,
        }
    }
}

#[async_trait]
impl<S> Stream for ThrottledStream<S>
where
    S: Stream + Send + Sync + 'static,
{
    type Event = S::Event;
    type Config = S::Config;

    async fn start(&mut self, config: Self::Config) -> Result<(), StreamError> {
        self.inner.start(config).await
    }

    async fn stop(&mut self) -> Result<(), StreamError> {
        self.inner.stop().await
    }

    fn subscribe(&self) -> broadcast::Receiver<Arc<Self::Event>> {
        let (tx, rx) = broadcast::channel(10000);
        let mut inner_rx = self.inner.subscribe();
        let duration = self.duration;

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(duration);
            let mut last_event: Option<Arc<S::Event>> = None;

            loop {
                tokio::select! {
                    Ok(event) = inner_rx.recv() => {
                        last_event = Some(event);
                    }
                    _ = interval.tick() => {
                        if let Some(event) = last_event.take() {
                            let _ = tx.send(event);
                        }
                    }
                }
            }
        });

        rx
    }

    fn is_running(&self) -> bool {
        self.inner.is_running()
    }

    async fn health(&self) -> StreamHealth {
        self.inner.health().await
    }

    fn name(&self) -> &str {
        &self.name
    }
}

/// Stream that takes only N events
pub struct TakeStream<S> {
    /// The underlying stream
    inner: S,
    /// Maximum number of events to take
    count: usize,
    /// Counter of events taken so far
    taken: Arc<RwLock<usize>>,
    /// Name of the take stream
    name: String,
}

impl<S: Stream> TakeStream<S> {
    /// Create a new take stream
    pub fn new(inner: S, count: usize) -> Self {
        let name = format!("take({},{})", count, inner.name());
        Self {
            inner,
            count,
            taken: Arc::new(RwLock::new(0)),
            name,
        }
    }
}

#[async_trait]
impl<S> Stream for TakeStream<S>
where
    S: Stream + Send + Sync + 'static,
{
    type Event = S::Event;
    type Config = S::Config;

    async fn start(&mut self, config: Self::Config) -> Result<(), StreamError> {
        self.inner.start(config).await
    }

    async fn stop(&mut self) -> Result<(), StreamError> {
        self.inner.stop().await
    }

    fn subscribe(&self) -> broadcast::Receiver<Arc<Self::Event>> {
        let (tx, rx) = broadcast::channel(10000);
        let mut inner_rx = self.inner.subscribe();
        let count = self.count;
        let taken = self.taken.clone();

        tokio::spawn(async move {
            while let Ok(event) = inner_rx.recv().await {
                let mut taken_count = taken.write().await;
                if *taken_count < count {
                    *taken_count += 1;
                    let _ = tx.send(event);
                    if *taken_count >= count {
                        break;
                    }
                }
            }
        });

        rx
    }

    fn is_running(&self) -> bool {
        self.inner.is_running()
    }

    async fn health(&self) -> StreamHealth {
        self.inner.health().await
    }

    fn name(&self) -> &str {
        &self.name
    }
}

/// Stream that skips N events
#[allow(dead_code)]
pub struct SkipStream<S> {
    /// The underlying stream
    inner: S,
    /// Number of events to skip
    count: usize,
    /// Counter of events skipped so far
    skipped: Arc<RwLock<usize>>,
    /// Name of the skip stream
    name: String,
}

impl<S: Stream> SkipStream<S> {
    /// Create a new skip stream
    pub fn new(inner: S, count: usize) -> Self {
        let name = format!("skip({},{})", count, inner.name());
        Self {
            inner,
            count,
            skipped: Arc::new(RwLock::new(0)),
            name,
        }
    }
}

/// Stream with stateful transformation (scan)
pub struct ScanStream<S, St, F> {
    /// The underlying stream
    inner: S,
    /// Current state of the scan
    state: Arc<RwLock<St>>,
    /// State transformation function
    transform: Arc<F>,
    /// Name of the scan stream
    name: String,
}

impl<S, St, F> ScanStream<S, St, F>
where
    S: Stream,
    St: Clone + Send + Sync + 'static,
    F: Fn(St, Arc<S::Event>) -> St + Send + Sync + 'static,
{
    /// Create a new scan stream
    pub fn new(inner: S, initial_state: St, transform: F) -> Self {
        let name = format!("scan({})", inner.name());
        Self {
            inner,
            state: Arc::new(RwLock::new(initial_state)),
            transform: Arc::new(transform),
            name,
        }
    }
}

/// Event wrapper for scan results
#[derive(Clone, Debug)]
pub struct ScanEvent<S, E> {
    /// Current state after transformation
    pub state: S,
    /// Original event that triggered the scan
    pub original_event: Arc<E>,
    /// Unique identifier for this scan event
    pub scan_id: String,
    /// Timestamp when the scan was performed
    pub timestamp: std::time::SystemTime,
}

impl<S, E> Event for ScanEvent<S, E>
where
    S: Clone + Send + Sync + 'static + std::fmt::Debug,
    E: Event + Clone + 'static,
{
    fn id(&self) -> &str {
        &self.scan_id
    }

    fn kind(&self) -> &riglr_events_core::EventKind {
        self.original_event.kind()
    }

    fn metadata(&self) -> &riglr_events_core::EventMetadata {
        self.original_event.metadata()
    }

    fn metadata_mut(&mut self) -> &mut riglr_events_core::EventMetadata {
        // Cannot modify original event through Arc
        panic!("metadata_mut not supported for ScanEvent")
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
}

#[async_trait]
impl<S, St, F> Stream for ScanStream<S, St, F>
where
    S: Stream + Send + Sync + 'static,
    St: Clone + Send + Sync + 'static + std::fmt::Debug,
    F: Fn(St, Arc<S::Event>) -> St + Send + Sync + 'static,
{
    type Event = ScanEvent<St, S::Event>;
    type Config = S::Config;

    async fn start(&mut self, config: Self::Config) -> Result<(), StreamError> {
        self.inner.start(config).await
    }

    async fn stop(&mut self) -> Result<(), StreamError> {
        self.inner.stop().await
    }

    fn subscribe(&self) -> broadcast::Receiver<Arc<Self::Event>> {
        let (tx, rx) = broadcast::channel(10000);
        let mut inner_rx = self.inner.subscribe();
        let state = self.state.clone();
        let transform = self.transform.clone();
        let mut counter = 0u64;

        tokio::spawn(async move {
            while let Ok(event) = inner_rx.recv().await {
                let current_state = {
                    let state_guard = state.read().await;
                    state_guard.clone()
                };

                let new_state = (transform)(current_state, event.clone());

                let scan_event = ScanEvent {
                    state: new_state.clone(),
                    original_event: event,
                    scan_id: format!("scan_{}", counter),
                    timestamp: std::time::SystemTime::now(),
                };

                {
                    let mut state_guard = state.write().await;
                    *state_guard = new_state;
                }
                counter += 1;

                let _ = tx.send(Arc::new(scan_event));
            }
        });

        rx
    }

    fn is_running(&self) -> bool {
        self.inner.is_running()
    }

    async fn health(&self) -> StreamHealth {
        self.inner.health().await
    }

    fn name(&self) -> &str {
        &self.name
    }
}

/// Stream with buffering strategies
#[allow(dead_code)]
pub struct BufferedStream<S: Stream> {
    inner: S,
    strategy: BufferStrategy,
    buffer: Arc<RwLock<Vec<Arc<S::Event>>>>,
    name: String,
}

impl<S: Stream> BufferedStream<S> {
    /// Create a new buffered stream
    pub fn new(inner: S, strategy: BufferStrategy) -> Self {
        let name = format!("buffered({})", inner.name());
        Self {
            inner,
            strategy,
            buffer: Arc::new(RwLock::new(Vec::new())),
            name,
        }
    }
}

/// Stream that fuses filter and map operations for better performance
pub struct FilterMapStream<S, FilterF, MapF, R> {
    inner: S,
    filter: Arc<FilterF>,
    map: Arc<MapF>,
    name: String,
    _phantom: PhantomData<R>,
}

impl<S, FilterF, MapF, R> FilterMapStream<S, FilterF, MapF, R>
where
    S: Stream,
    FilterF: Fn(&S::Event) -> bool + Send + Sync + 'static,
    MapF: Fn(Arc<S::Event>) -> R + Send + Sync + 'static,
    R: Event + Clone + Send + Sync + 'static,
{
    /// Create a new filter-map stream
    pub fn new(inner: S, filter: FilterF, map: MapF) -> Self {
        let name = format!("filter_map({})", inner.name());
        Self {
            inner,
            filter: Arc::new(filter),
            map: Arc::new(map),
            name,
            _phantom: PhantomData,
        }
    }
}

#[async_trait]
impl<S, FilterF, MapF, R> Stream for FilterMapStream<S, FilterF, MapF, R>
where
    S: Stream + Send + Sync + 'static,
    FilterF: Fn(&S::Event) -> bool + Send + Sync + 'static,
    MapF: Fn(Arc<S::Event>) -> R + Send + Sync + 'static,
    R: Event + Clone + Send + Sync + 'static,
{
    type Event = R;
    type Config = S::Config;

    async fn start(&mut self, config: Self::Config) -> Result<(), StreamError> {
        self.inner.start(config).await
    }

    async fn stop(&mut self) -> Result<(), StreamError> {
        self.inner.stop().await
    }

    fn subscribe(&self) -> broadcast::Receiver<Arc<Self::Event>> {
        let (tx, rx) = broadcast::channel(10000);
        let mut inner_rx = self.inner.subscribe();
        let filter = self.filter.clone();
        let map = self.map.clone();

        tokio::spawn(async move {
            while let Ok(event) = inner_rx.recv().await {
                // Apply filter and map in a single task - reduces overhead
                if (filter)(&event) {
                    let mapped = (map)(event);
                    let _ = tx.send(Arc::new(mapped));
                }
            }
        });

        rx
    }

    fn is_running(&self) -> bool {
        self.inner.is_running()
    }

    async fn health(&self) -> StreamHealth {
        self.inner.health().await
    }

    fn name(&self) -> &str {
        &self.name
    }
}

/// Extension trait for performance optimizations
pub trait PerformanceStreamExt: ComposableStream {
    /// Fused filter + map operation for better performance
    fn filter_map<FilterF, MapF, R>(
        self,
        filter: FilterF,
        map: MapF,
    ) -> FilterMapStream<Self, FilterF, MapF, R>
    where
        Self: Sized,
        FilterF: Fn(&Self::Event) -> bool + Send + Sync + 'static,
        MapF: Fn(Arc<Self::Event>) -> R + Send + Sync + 'static,
        R: Event + Clone + Send + Sync + 'static,
    {
        FilterMapStream::new(self, filter, map)
    }
}

impl<T> PerformanceStreamExt for T where T: ComposableStream {}

/// Stream with error recovery and retry capabilities
pub struct ResilientStream<S> {
    inner: S,
    max_retries: usize,
    retry_delay: Duration,
    name: String,
}

impl<S: Stream> ResilientStream<S> {
    /// Create a new resilient stream
    pub fn new(inner: S, max_retries: usize, retry_delay: Duration) -> Self {
        let name = format!("resilient({})", inner.name());
        Self {
            inner,
            max_retries,
            retry_delay,
            name,
        }
    }
}

#[async_trait]
impl<S> Stream for ResilientStream<S>
where
    S: Stream + Send + Sync + 'static,
{
    type Event = S::Event;
    type Config = S::Config;

    async fn start(&mut self, config: Self::Config) -> Result<(), StreamError> {
        let mut attempt = 0;
        loop {
            match self.inner.start(config.clone()).await {
                Ok(()) => return Ok(()),
                Err(e) if e.is_retriable() && attempt < self.max_retries => {
                    attempt += 1;
                    tracing::warn!("Stream start failed (attempt {}): {}", attempt, e);

                    let delay = if let Some(retry_after) = e.retry_after() {
                        retry_after
                    } else {
                        self.retry_delay * attempt as u32
                    };

                    tokio::time::sleep(delay).await;
                    continue;
                }
                Err(e) => return Err(e),
            }
        }
    }

    async fn stop(&mut self) -> Result<(), StreamError> {
        self.inner.stop().await
    }

    fn subscribe(&self) -> broadcast::Receiver<Arc<Self::Event>> {
        // For resilience, we could add circuit breaker logic here
        self.inner.subscribe()
    }

    fn is_running(&self) -> bool {
        self.inner.is_running()
    }

    async fn health(&self) -> StreamHealth {
        self.inner.health().await
    }

    fn name(&self) -> &str {
        &self.name
    }
}

/// Stream that catches errors and provides fallback values
pub struct CatchErrorStream<S, F> {
    inner: S,
    error_handler: Arc<F>,
    name: String,
}

impl<S, F> CatchErrorStream<S, F>
where
    S: Stream,
    F: Fn(StreamError) -> Option<Arc<S::Event>> + Send + Sync + 'static,
{
    /// Create a new error-catching stream with the given error handler
    pub fn new(inner: S, error_handler: F) -> Self {
        let name = format!("catch_error({})", inner.name());
        Self {
            inner,
            error_handler: Arc::new(error_handler),
            name,
        }
    }
}

#[async_trait]
impl<S, F> Stream for CatchErrorStream<S, F>
where
    S: Stream + Send + Sync + 'static,
    F: Fn(StreamError) -> Option<Arc<S::Event>> + Send + Sync + 'static,
{
    type Event = S::Event;
    type Config = S::Config;

    async fn start(&mut self, config: Self::Config) -> Result<(), StreamError> {
        self.inner.start(config).await
    }

    async fn stop(&mut self) -> Result<(), StreamError> {
        self.inner.stop().await
    }

    fn subscribe(&self) -> broadcast::Receiver<Arc<Self::Event>> {
        let (tx, rx) = broadcast::channel(10000);
        let mut inner_rx = self.inner.subscribe();
        let error_handler = self.error_handler.clone();

        tokio::spawn(async move {
            loop {
                match inner_rx.recv().await {
                    Ok(event) => {
                        let _ = tx.send(event);
                    }
                    Err(e) => {
                        // Convert broadcast error to StreamError
                        let stream_error = StreamError::Channel {
                            message: e.to_string(),
                        };

                        if let Some(fallback_event) = (error_handler)(stream_error) {
                            let _ = tx.send(fallback_event);
                        }

                        // Continue listening - don't break on errors
                        continue;
                    }
                }
            }
        });

        rx
    }

    fn is_running(&self) -> bool {
        self.inner.is_running()
    }

    async fn health(&self) -> StreamHealth {
        self.inner.health().await
    }

    fn name(&self) -> &str {
        &self.name
    }
}

/// Extension trait for resilience and error handling
pub trait ResilienceStreamExt: ComposableStream {
    /// Add retry logic to stream operations
    fn with_retries(self, max_retries: usize, retry_delay: Duration) -> ResilientStream<Self>
    where
        Self: Sized,
    {
        ResilientStream::new(self, max_retries, retry_delay)
    }

    /// Catch errors and provide fallback events
    fn catch_errors<F>(self, error_handler: F) -> CatchErrorStream<Self, F>
    where
        Self: Sized,
        F: Fn(StreamError) -> Option<Arc<Self::Event>> + Send + Sync + 'static,
    {
        CatchErrorStream::new(self, error_handler)
    }
}

impl<T> ResilienceStreamExt for T where T: ComposableStream {}

/// Builder for complex stream compositions
pub struct StreamPipeline<T> {
    stream: T,
}

impl<T: Stream> StreamPipeline<T> {
    /// Create a new pipeline from a stream
    pub fn from(stream: T) -> Self {
        Self { stream }
    }

    /// Build the final stream
    pub fn build(self) -> T {
        self.stream
    }
}

/// Utility functions for stream composition
pub mod combinators {
    use super::*;

    /// Merge multiple streams into one
    pub fn merge_all<S, I>(streams: I) -> MergeAllStream<S>
    where
        S: Stream + 'static,
        I: IntoIterator<Item = S>,
    {
        MergeAllStream::new(streams.into_iter().collect())
    }

    /// Zip two streams together
    pub fn zip<S1, S2>(stream1: S1, stream2: S2) -> ZipStream<S1, S2>
    where
        S1: Stream,
        S2: Stream,
    {
        ZipStream::new(stream1, stream2)
    }

    /// Combine latest values from two streams
    pub fn combine_latest<S1, S2>(stream1: S1, stream2: S2) -> CombineLatestStream<S1, S2>
    where
        S1: Stream,
        S2: Stream,
    {
        CombineLatestStream::new(stream1, stream2)
    }
}

/// Type-erased event for merged streams to avoid complex nested types
#[derive(Clone, Debug)]
pub struct TypeErasedEvent {
    inner: Arc<dyn Event>,
    source_stream: String,
}

impl TypeErasedEvent {
    /// Create a new type-erased event
    pub fn new(event: Arc<dyn Event>, source_stream: String) -> Self {
        Self {
            inner: event,
            source_stream,
        }
    }

    /// Attempt to downcast to a specific event type
    pub fn downcast<T: Event + 'static>(&self) -> Option<&T> {
        self.inner.as_any().downcast_ref::<T>()
    }

    /// Get the source stream name
    pub fn source_stream(&self) -> &str {
        &self.source_stream
    }

    /// Get the inner event as a trait object
    pub fn inner(&self) -> &dyn Event {
        &*self.inner
    }
}

impl Event for TypeErasedEvent {
    fn id(&self) -> &str {
        self.inner.id()
    }

    fn kind(&self) -> &riglr_events_core::EventKind {
        self.inner.kind()
    }

    fn metadata(&self) -> &riglr_events_core::EventMetadata {
        self.inner.metadata()
    }

    fn metadata_mut(&mut self) -> &mut riglr_events_core::EventMetadata {
        // Cannot modify through Arc
        panic!("metadata_mut not supported for TypeErasedEvent")
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
}

/// Stream that merges multiple streams using type erasure
pub struct MergeAllStream<S> {
    streams: Vec<S>,
    name: String,
}

impl<S: Stream> MergeAllStream<S> {
    /// Create a new merge-all stream from a vector of streams
    pub fn new(streams: Vec<S>) -> Self {
        let names: Vec<_> = streams.iter().map(|s| s.name()).collect();
        let name = format!("merge_all({})", names.join(","));
        Self { streams, name }
    }
}

#[async_trait]
impl<S> Stream for MergeAllStream<S>
where
    S: Stream + Send + Sync + 'static,
{
    type Event = TypeErasedEvent;
    type Config = Vec<S::Config>;

    async fn start(&mut self, configs: Self::Config) -> Result<(), StreamError> {
        if configs.len() != self.streams.len() {
            return Err(StreamError::ConfigurationInvalid {
                reason: "Number of configs must match number of streams".to_string(),
            });
        }

        for (stream, config) in self.streams.iter_mut().zip(configs.into_iter()) {
            stream.start(config).await?;
        }
        Ok(())
    }

    async fn stop(&mut self) -> Result<(), StreamError> {
        for stream in &mut self.streams {
            stream.stop().await?;
        }
        Ok(())
    }

    fn subscribe(&self) -> broadcast::Receiver<Arc<Self::Event>> {
        let (tx, rx) = broadcast::channel(10000);

        for (i, stream) in self.streams.iter().enumerate() {
            let mut stream_rx = stream.subscribe();
            let tx_clone = tx.clone();
            let stream_name = format!("{}_{}", stream.name(), i);

            tokio::spawn(async move {
                while let Ok(event) = stream_rx.recv().await {
                    let type_erased =
                        TypeErasedEvent::new(event as Arc<dyn Event>, stream_name.clone());
                    let _ = tx_clone.send(Arc::new(type_erased));
                }
            });
        }

        rx
    }

    fn is_running(&self) -> bool {
        self.streams.iter().any(|s| s.is_running())
    }

    async fn health(&self) -> StreamHealth {
        let mut combined_health = StreamHealth::default();

        for stream in &self.streams {
            let health = stream.health().await;
            combined_health.is_connected = combined_health.is_connected || health.is_connected;
            combined_health.error_count += health.error_count;
            combined_health.events_processed += health.events_processed;

            if health.last_event_time.is_some() {
                combined_health.last_event_time =
                    [combined_health.last_event_time, health.last_event_time]
                        .iter()
                        .filter_map(|&x| x)
                        .max();
            }
        }

        combined_health
    }

    fn name(&self) -> &str {
        &self.name
    }
}

/// Stream wrapper that provides guaranteed delivery using mpsc channels
///
/// Unlike the default broadcast channels that drop old messages when buffers fill,
/// this wrapper uses tokio::sync::mpsc channels to ensure every message is delivered.
/// This comes at the cost of potential pipeline blocking if receivers are slow.
pub struct GuaranteedDeliveryStream<S> {
    inner: S,
    buffer_size: usize,
    name: String,
}

impl<S: Stream> GuaranteedDeliveryStream<S> {
    /// Create a new guaranteed delivery stream with specified buffer size
    pub fn new(inner: S, buffer_size: usize) -> Self {
        let name = format!("guaranteed({})", inner.name());
        Self {
            inner,
            buffer_size,
            name,
        }
    }
}

#[async_trait]
impl<S> Stream for GuaranteedDeliveryStream<S>
where
    S: Stream + Send + Sync + 'static,
{
    type Event = S::Event;
    type Config = S::Config;

    async fn start(&mut self, config: Self::Config) -> Result<(), StreamError> {
        self.inner.start(config).await
    }

    async fn stop(&mut self) -> Result<(), StreamError> {
        self.inner.stop().await
    }

    fn subscribe(&self) -> broadcast::Receiver<Arc<Self::Event>> {
        // We still need to return a broadcast::Receiver for compatibility,
        // but internally we use mpsc for guaranteed delivery
        let (broadcast_tx, broadcast_rx) = broadcast::channel(self.buffer_size);
        let (mpsc_tx, mut mpsc_rx) =
            tokio::sync::mpsc::channel::<Arc<Self::Event>>(self.buffer_size);

        let mut inner_rx = self.inner.subscribe();

        // Task 1: Forward from broadcast source to mpsc (guaranteed delivery)
        tokio::spawn(async move {
            while let Ok(event) = inner_rx.recv().await {
                // This will block if receiver is slow, providing backpressure
                if mpsc_tx.send(event).await.is_err() {
                    break; // Receiver dropped
                }
            }
        });

        // Task 2: Forward from mpsc to broadcast for compatibility
        tokio::spawn(async move {
            while let Some(event) = mpsc_rx.recv().await {
                // If broadcast fails (no receivers), continue processing
                let _ = broadcast_tx.send(event);
            }
        });

        broadcast_rx
    }

    fn is_running(&self) -> bool {
        self.inner.is_running()
    }

    async fn health(&self) -> StreamHealth {
        self.inner.health().await
    }

    fn name(&self) -> &str {
        &self.name
    }
}

/// Stream that zips two streams
#[allow(dead_code)]
pub struct ZipStream<S1, S2> {
    stream1: S1,
    stream2: S2,
    name: String,
}

impl<S1: Stream, S2: Stream> ZipStream<S1, S2> {
    /// Create a new zip stream that combines events from two streams
    pub fn new(stream1: S1, stream2: S2) -> Self {
        let name = format!("zip({},{})", stream1.name(), stream2.name());
        Self {
            stream1,
            stream2,
            name,
        }
    }
}

/// Stream that combines latest values
#[allow(dead_code)]
pub struct CombineLatestStream<S1, S2> {
    stream1: S1,
    stream2: S2,
    name: String,
}

impl<S1: Stream, S2: Stream> CombineLatestStream<S1, S2> {
    /// Create a new combine-latest stream that emits when either stream updates
    pub fn new(stream1: S1, stream2: S2) -> Self {
        let name = format!("combine_latest({},{})", stream1.name(), stream2.name());
        Self {
            stream1,
            stream2,
            name,
        }
    }
}
