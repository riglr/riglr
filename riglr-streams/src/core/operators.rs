//! Stream composition operators for building reactive pipelines
//!
//! This module provides a rich set of operators for composing, transforming,
//! and combining streams in a declarative, functional style.
//!
//! ## Backpressure Behavior
//!
//! The operators in this module use `tokio::sync::broadcast` channels by default,
//! which implement a **drop-oldest backpressure strategy**

use async_trait::async_trait;
use futures::future::BoxFuture;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::broadcast;
use tokio::time::interval;

use super::streamed_event::DynamicStreamedEvent;
use crate::core::{Stream, StreamError, StreamHealth};

/// Type alias for boxed futures
#[allow(dead_code)]
type BoxedFuture<T> = BoxFuture<'static, T>;

/// Trait for composable streams
#[async_trait]
pub trait ComposableStream: Stream {
    /// Map events through a transformation function
    fn map<F>(self, f: F) -> MappedStream<Self, F>
    where
        Self: Sized,
        F: Fn(Arc<DynamicStreamedEvent>) -> DynamicStreamedEvent + Send + Sync + 'static,
    {
        MappedStream::new(self, f)
    }

    /// Filter events based on a predicate
    fn filter<F>(self, predicate: F) -> FilteredStream<Self, F>
    where
        Self: Sized,
        F: Fn(&DynamicStreamedEvent) -> bool + Send + Sync + 'static,
    {
        FilteredStream::new(self, predicate)
    }

    /// Merge with another stream
    fn merge<S>(self, other: S) -> MergedStream<Self, S>
    where
        Self: Sized,
        S: Stream,
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
}

/// Implement ComposableStream for all Stream types
impl<T> ComposableStream for T where T: Stream {}

/// Stream that maps events through a transformation
#[derive(Debug)]
pub struct MappedStream<S, F> {
    inner: S,
    transform: Arc<F>,
}

impl<S, F> MappedStream<S, F>
where
    S: Stream,
    F: Fn(Arc<DynamicStreamedEvent>) -> DynamicStreamedEvent + Send + Sync + 'static,
{
    /// Creates a new mapped stream that applies a transformation function to each event
    pub fn new(inner: S, transform: F) -> Self {
        Self {
            inner,
            transform: Arc::new(transform),
        }
    }
}

#[async_trait]
impl<S, F> Stream for MappedStream<S, F>
where
    S: Stream + Send + Sync + 'static,
    F: Fn(Arc<DynamicStreamedEvent>) -> DynamicStreamedEvent + Send + Sync + 'static,
{
    type Config = S::Config;

    async fn start(&mut self, config: Self::Config) -> Result<(), StreamError> {
        let start_result = self.inner.start(config).await;
        start_result
    }

    async fn stop(&mut self) -> Result<(), StreamError> {
        let stop_result = self.inner.stop().await;
        stop_result
    }

    fn subscribe(&self) -> broadcast::Receiver<Arc<DynamicStreamedEvent>> {
        let (tx, rx) = broadcast::channel(10000);
        let mut inner_rx = self.inner.subscribe();
        let transform = self.transform.clone();

        tokio::spawn(async move {
            loop {
                let recv_result = inner_rx.recv().await;
                if let Ok(event) = recv_result {
                    let transformed = (transform)(event);
                    let _ = tx.send(Arc::new(transformed));
                } else {
                    break;
                }
            }
        });

        rx
    }

    fn is_running(&self) -> bool {
        self.inner.is_running()
    }

    async fn health(&self) -> StreamHealth {
        let health_result = self.inner.health().await;
        health_result
    }

    fn name(&self) -> &str {
        self.inner.name()
    }
}

/// Stream that filters events based on a predicate
#[derive(Debug)]
pub struct FilteredStream<S, F> {
    inner: S,
    predicate: Arc<F>,
}

impl<S, F> FilteredStream<S, F>
where
    S: Stream,
    F: Fn(&DynamicStreamedEvent) -> bool + Send + Sync + 'static,
{
    /// Creates a new filtered stream that only emits events matching the predicate
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
    F: Fn(&DynamicStreamedEvent) -> bool + Send + Sync + 'static,
{
    type Config = S::Config;

    async fn start(&mut self, config: Self::Config) -> Result<(), StreamError> {
        let start_result = self.inner.start(config).await;
        start_result
    }

    async fn stop(&mut self) -> Result<(), StreamError> {
        let stop_result = self.inner.stop().await;
        stop_result
    }

    fn subscribe(&self) -> broadcast::Receiver<Arc<DynamicStreamedEvent>> {
        let (tx, rx) = broadcast::channel(10000);
        let mut inner_rx = self.inner.subscribe();
        let predicate = self.predicate.clone();

        tokio::spawn(async move {
            loop {
                let recv_result = inner_rx.recv().await;
                if let Ok(event) = recv_result {
                    if (predicate)(event.as_ref()) {
                        let _ = tx.send(event);
                    }
                } else {
                    break;
                }
            }
        });

        rx
    }

    fn is_running(&self) -> bool {
        self.inner.is_running()
    }

    async fn health(&self) -> StreamHealth {
        let health_result = self.inner.health().await;
        health_result
    }

    fn name(&self) -> &str {
        self.inner.name()
    }
}

/// Stream that merges events from two streams
#[derive(Debug)]
pub struct MergedStream<S1, S2> {
    inner1: S1,
    inner2: S2,
}

impl<S1, S2> MergedStream<S1, S2>
where
    S1: Stream,
    S2: Stream,
{
    /// Creates a new merged stream that combines events from two source streams
    pub fn new(inner1: S1, inner2: S2) -> Self {
        Self { inner1, inner2 }
    }
}

#[async_trait]
impl<S1, S2> Stream for MergedStream<S1, S2>
where
    S1: Stream + Send + Sync + 'static,
    S2: Stream + Send + Sync + 'static,
{
    type Config = (S1::Config, S2::Config);

    async fn start(&mut self, config: Self::Config) -> Result<(), StreamError> {
        self.inner1.start(config.0).await?;
        let start_result = self.inner2.start(config.1).await;
        start_result
    }

    async fn stop(&mut self) -> Result<(), StreamError> {
        let r1 = self.inner1.stop().await;
        let r2_result = self.inner2.stop().await;
        r1?;
        r2_result
    }

    fn subscribe(&self) -> broadcast::Receiver<Arc<DynamicStreamedEvent>> {
        let (tx, rx) = broadcast::channel(10000);
        let mut rx1 = self.inner1.subscribe();
        let mut rx2 = self.inner2.subscribe();

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    event1 = rx1.recv() => {
                        match event1 {
                            Ok(event) => {
                                let _ = tx.send(event);
                            }
                            Err(_) => break,
                        }
                    }
                    event2 = rx2.recv() => {
                        match event2 {
                            Ok(event) => {
                                let _ = tx.send(event);
                            }
                            Err(_) => break,
                        }
                    }
                }
            }
        });

        rx
    }

    fn is_running(&self) -> bool {
        self.inner1.is_running() || self.inner2.is_running()
    }

    async fn health(&self) -> StreamHealth {
        let h1 = self.inner1.health().await;
        let h2 = self.inner2.health().await;

        // Combine health from both streams
        StreamHealth {
            is_connected: h1.is_connected && h2.is_connected,
            last_event_time: match (h1.last_event_time, h2.last_event_time) {
                (Some(t1), Some(t2)) => Some(t1.max(t2)),
                (Some(t), None) | (None, Some(t)) => Some(t),
                _ => None,
            },
            error_count: h1.error_count + h2.error_count,
            events_processed: h1.events_processed + h2.events_processed,
            backlog_size: match (h1.backlog_size, h2.backlog_size) {
                (Some(b1), Some(b2)) => Some(b1 + b2),
                (Some(b), None) | (None, Some(b)) => Some(b),
                _ => None,
            },
            custom_metrics: None,
            stream_info: None,
        }
    }

    fn name(&self) -> &str {
        self.inner1.name()
    }
}

/// Stream that batches events
#[derive(Debug)]
pub struct BatchedStream<S> {
    inner: S,
    batch_size: usize,
    timeout: Duration,
}

impl<S> BatchedStream<S>
where
    S: Stream,
{
    /// Creates a new batched stream that groups events by size or timeout
    pub fn new(inner: S, batch_size: usize, timeout: Duration) -> Self {
        Self {
            inner,
            batch_size,
            timeout,
        }
    }
}

#[async_trait]
impl<S> Stream for BatchedStream<S>
where
    S: Stream + Send + Sync + 'static,
{
    type Config = S::Config;

    async fn start(&mut self, config: Self::Config) -> Result<(), StreamError> {
        let start_result = self.inner.start(config).await;
        start_result
    }

    async fn stop(&mut self) -> Result<(), StreamError> {
        let stop_result = self.inner.stop().await;
        stop_result
    }

    fn subscribe(&self) -> broadcast::Receiver<Arc<DynamicStreamedEvent>> {
        let (tx, rx) = broadcast::channel(10000);
        let mut inner_rx = self.inner.subscribe();
        let batch_size = self.batch_size;
        let timeout_duration = self.timeout;

        tokio::spawn(async move {
            let mut batch = Vec::new();
            let mut interval = interval(timeout_duration);

            loop {
                tokio::select! {
                    event = inner_rx.recv() => {
                        match event {
                            Ok(event) => {
                                batch.push(event);
                                if batch.len() >= batch_size {
                                    // For now, just send events individually
                                    // In production, you'd wrap them in a batch event
                                    for evt in batch.drain(..) {
                                        let _ = tx.send(evt);
                                    }
                                }
                            }
                            Err(_) => break,
                        }
                    }
                    _ = interval.tick() => {
                        if !batch.is_empty() {
                            for evt in batch.drain(..) {
                                let _ = tx.send(evt);
                            }
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
        let health_result = self.inner.health().await;
        health_result
    }

    fn name(&self) -> &str {
        self.inner.name()
    }
}

/// Stream that debounces events
#[derive(Debug)]
pub struct DebouncedStream<S> {
    inner: S,
    duration: Duration,
}

impl<S> DebouncedStream<S>
where
    S: Stream,
{
    /// Creates a new debounced stream that only emits events after a quiet period
    pub fn new(inner: S, duration: Duration) -> Self {
        Self { inner, duration }
    }
}

#[async_trait]
impl<S> Stream for DebouncedStream<S>
where
    S: Stream + Send + Sync + 'static,
{
    type Config = S::Config;

    async fn start(&mut self, config: Self::Config) -> Result<(), StreamError> {
        let start_result = self.inner.start(config).await;
        start_result
    }

    async fn stop(&mut self) -> Result<(), StreamError> {
        let stop_result = self.inner.stop().await;
        stop_result
    }

    fn subscribe(&self) -> broadcast::Receiver<Arc<DynamicStreamedEvent>> {
        let (tx, rx) = broadcast::channel(10000);
        let mut inner_rx = self.inner.subscribe();
        let duration = self.duration;

        tokio::spawn(async move {
            let mut last_event = None;
            let mut timer = tokio::time::interval(duration);

            loop {
                tokio::select! {
                    event = inner_rx.recv() => {
                        match event {
                            Ok(event) => {
                                last_event = Some(event);
                                timer.reset();
                            }
                            Err(_) => break,
                        }
                    }
                    _ = timer.tick() => {
                        let taken_event = last_event.take();
                        if let Some(event) = taken_event {
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
        let health_result = self.inner.health().await;
        health_result
    }

    fn name(&self) -> &str {
        self.inner.name()
    }
}

/// Stream that throttles events
#[derive(Debug)]
pub struct ThrottledStream<S> {
    inner: S,
    duration: Duration,
}

impl<S> ThrottledStream<S>
where
    S: Stream,
{
    /// Creates a new throttled stream that limits the rate of events
    pub fn new(inner: S, duration: Duration) -> Self {
        Self { inner, duration }
    }
}

#[async_trait]
impl<S> Stream for ThrottledStream<S>
where
    S: Stream + Send + Sync + 'static,
{
    type Config = S::Config;

    async fn start(&mut self, config: Self::Config) -> Result<(), StreamError> {
        let start_result = self.inner.start(config).await;
        start_result
    }

    async fn stop(&mut self) -> Result<(), StreamError> {
        let stop_result = self.inner.stop().await;
        stop_result
    }

    fn subscribe(&self) -> broadcast::Receiver<Arc<DynamicStreamedEvent>> {
        let (tx, rx) = broadcast::channel(10000);
        let mut inner_rx = self.inner.subscribe();
        let duration = self.duration;

        tokio::spawn(async move {
            let mut last_send = Instant::now();

            loop {
                let recv_result = inner_rx.recv().await;
                if let Ok(event) = recv_result {
                    let now = Instant::now();
                    if now.duration_since(last_send) >= duration {
                        let _ = tx.send(event);
                        last_send = now;
                    }
                } else {
                    break;
                }
            }
        });

        rx
    }

    fn is_running(&self) -> bool {
        self.inner.is_running()
    }

    async fn health(&self) -> StreamHealth {
        let health_result = self.inner.health().await;
        health_result
    }

    fn name(&self) -> &str {
        self.inner.name()
    }
}

/// Stream that takes only N events
#[derive(Debug)]
pub struct TakeStream<S> {
    inner: S,
    count: Arc<AtomicUsize>,
    max_count: usize,
}

impl<S> TakeStream<S>
where
    S: Stream,
{
    /// Creates a new take stream that emits only the first N events
    pub fn new(inner: S, count: usize) -> Self {
        Self {
            inner,
            count: Arc::new(AtomicUsize::new(0)),
            max_count: count,
        }
    }
}

#[async_trait]
impl<S> Stream for TakeStream<S>
where
    S: Stream + Send + Sync + 'static,
{
    type Config = S::Config;

    async fn start(&mut self, config: Self::Config) -> Result<(), StreamError> {
        let start_result = self.inner.start(config).await;
        start_result
    }

    async fn stop(&mut self) -> Result<(), StreamError> {
        let stop_result = self.inner.stop().await;
        stop_result
    }

    fn subscribe(&self) -> broadcast::Receiver<Arc<DynamicStreamedEvent>> {
        let (tx, rx) = broadcast::channel(10000);
        let mut inner_rx = self.inner.subscribe();
        let count = self.count.clone();
        let max_count = self.max_count;

        tokio::spawn(async move {
            loop {
                let recv_result = inner_rx.recv().await;
                if let Ok(event) = recv_result {
                    let current = count.fetch_add(1, Ordering::SeqCst);
                    if current < max_count {
                        let _ = tx.send(event);
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }
        });

        rx
    }

    fn is_running(&self) -> bool {
        self.inner.is_running()
    }

    async fn health(&self) -> StreamHealth {
        let health_result = self.inner.health().await;
        health_result
    }

    fn name(&self) -> &str {
        self.inner.name()
    }
}

/// Stream that skips N events
#[derive(Debug)]
pub struct SkipStream<S> {
    inner: S,
    skip_count: Arc<AtomicUsize>,
    target_skip: usize,
}

impl<S> SkipStream<S>
where
    S: Stream,
{
    /// Creates a new skip stream that ignores the first N events
    pub fn new(inner: S, count: usize) -> Self {
        Self {
            inner,
            skip_count: Arc::new(AtomicUsize::new(0)),
            target_skip: count,
        }
    }
}

#[async_trait]
impl<S> Stream for SkipStream<S>
where
    S: Stream + Send + Sync + 'static,
{
    type Config = S::Config;

    async fn start(&mut self, config: Self::Config) -> Result<(), StreamError> {
        let start_result = self.inner.start(config).await;
        start_result
    }

    async fn stop(&mut self) -> Result<(), StreamError> {
        let stop_result = self.inner.stop().await;
        stop_result
    }

    fn subscribe(&self) -> broadcast::Receiver<Arc<DynamicStreamedEvent>> {
        let (tx, rx) = broadcast::channel(10000);
        let mut inner_rx = self.inner.subscribe();
        let skip_count = self.skip_count.clone();
        let target_skip = self.target_skip;

        tokio::spawn(async move {
            loop {
                let recv_result = inner_rx.recv().await;
                if let Ok(event) = recv_result {
                    let current = skip_count.load(Ordering::SeqCst);
                    if current >= target_skip {
                        let _ = tx.send(event);
                    } else {
                        skip_count.fetch_add(1, Ordering::SeqCst);
                    }
                } else {
                    break;
                }
            }
        });

        rx
    }

    fn is_running(&self) -> bool {
        self.inner.is_running()
    }

    async fn health(&self) -> StreamHealth {
        let health_result = self.inner.health().await;
        health_result
    }

    fn name(&self) -> &str {
        self.inner.name()
    }
}
