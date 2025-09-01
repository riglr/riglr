//! Enhanced stream operators for advanced data processing
//!
//! This module provides additional stream operators for more complex transformations,
//! filtering, and aggregation operations beyond the basic operators.

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{broadcast, RwLock};
use tokio::time::interval;

use super::streamed_event::DynamicStreamedEvent;
use crate::core::{Stream, StreamError, StreamHealth};

/// Stream that groups events by a key function
pub struct GroupedStream<S, F, K> {
    inner: S,
    key_fn: Arc<F>,
    groups: Arc<RwLock<HashMap<K, broadcast::Sender<Arc<DynamicStreamedEvent>>>>>,
}

impl<S, F, K> GroupedStream<S, F, K>
where
    S: Stream,
    F: Fn(&DynamicStreamedEvent) -> K + Send + Sync + 'static,
    K: Clone + Eq + std::hash::Hash + Send + Sync + 'static,
{
    /// Creates a new grouped stream with the given inner stream and key function.
    ///
    /// The key function is used to determine which group each event belongs to.
    /// Events with the same key will be grouped together and can be accessed
    /// via the `get_group` method.
    pub fn new(inner: S, key_fn: F) -> Self {
        Self {
            inner,
            key_fn: Arc::new(key_fn),
            groups: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Gets a receiver for events in the specified group.
    ///
    /// Returns `None` if no group with the given key exists yet.
    /// Once events start flowing through the stream, groups will be created
    /// automatically based on the key function provided during construction.
    pub async fn get_group(
        &self,
        key: K,
    ) -> Option<broadcast::Receiver<Arc<DynamicStreamedEvent>>> {
        let groups = self.groups.read().await;
        groups.get(&key).map(|sender| sender.subscribe())
    }
}

#[async_trait]
impl<S, F, K> Stream for GroupedStream<S, F, K>
where
    S: Stream + Send + Sync + 'static,
    F: Fn(&DynamicStreamedEvent) -> K + Send + Sync + 'static,
    K: Clone + Eq + std::hash::Hash + Send + Sync + 'static,
{
    type Config = S::Config;

    async fn start(&mut self, config: Self::Config) -> Result<(), StreamError> {
        self.inner.start(config).await?;

        // Start forwarding events to groups
        let mut inner_rx = self.inner.subscribe();
        let key_fn = self.key_fn.clone();
        let groups = self.groups.clone();

        tokio::spawn(async move {
            while let Ok(event) = inner_rx.recv().await {
                let key = (key_fn)(event.as_ref());

                let mut groups_guard = groups.write().await;
                let sender = groups_guard.entry(key).or_insert_with(|| {
                    let (tx, _) = broadcast::channel(1000);
                    tx
                });

                let _ = sender.send(event);
            }
        });

        Ok(())
    }

    async fn stop(&mut self) -> Result<(), StreamError> {
        self.inner.stop().await
    }

    fn subscribe(&self) -> broadcast::Receiver<Arc<DynamicStreamedEvent>> {
        self.inner.subscribe()
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

/// Stream that windows events by time
pub struct WindowedStream<S> {
    inner: S,
    window_duration: Duration,
    windows: Arc<RwLock<Vec<Arc<DynamicStreamedEvent>>>>,
}

impl<S> WindowedStream<S>
where
    S: Stream,
{
    /// Creates a new windowed stream with the given inner stream and window duration.
    ///
    /// Events will be collected in a sliding window that resets every `window_duration`.
    /// The window duration determines how long events are kept before being cleared.
    pub fn new(inner: S, window_duration: Duration) -> Self {
        Self {
            inner,
            window_duration,
            windows: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

#[async_trait]
impl<S> Stream for WindowedStream<S>
where
    S: Stream + Send + Sync + 'static,
{
    type Config = S::Config;

    async fn start(&mut self, config: Self::Config) -> Result<(), StreamError> {
        self.inner.start(config).await?;

        // Start windowing
        let mut inner_rx = self.inner.subscribe();
        let windows = self.windows.clone();
        let window_duration = self.window_duration;

        tokio::spawn(async move {
            let mut interval = interval(window_duration);

            loop {
                tokio::select! {
                    event = inner_rx.recv() => {
                        match event {
                            Ok(event) => {
                                let mut windows_guard = windows.write().await;
                                windows_guard.push(event);
                            }
                            Err(_) => break,
                        }
                    }
                    _ = interval.tick() => {
                        // Clear window
                        let mut windows_guard = windows.write().await;
                        windows_guard.clear();
                    }
                }
            }
        });

        Ok(())
    }

    async fn stop(&mut self) -> Result<(), StreamError> {
        self.inner.stop().await
    }

    fn subscribe(&self) -> broadcast::Receiver<Arc<DynamicStreamedEvent>> {
        self.inner.subscribe()
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
