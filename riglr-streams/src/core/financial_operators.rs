//! Financial stream operators for market data processing
//!
//! This module provides specialized stream operators for financial data,
//! including technical indicators, price aggregation, and market analysis.

use async_trait::async_trait;
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{broadcast, RwLock};

use super::streamed_event::DynamicStreamedEvent;
use crate::core::{Stream, StreamError, StreamHealth};

/// Moving average calculator for financial streams
pub struct MovingAverageStream<S> {
    inner: S,
    window_size: usize,
    values: Arc<RwLock<VecDeque<f64>>>,
}

impl<S> MovingAverageStream<S>
where
    S: Stream,
{
    /// Creates a new moving average stream with the specified window size.
    ///
    /// # Arguments
    /// * `inner` - The underlying stream to wrap
    /// * `window_size` - Number of values to include in the moving average calculation
    ///
    /// # Example
    /// ```
    /// use riglr_streams::core::financial_operators::MovingAverageStream;
    /// let ma_stream = MovingAverageStream::new(base_stream, 20);
    /// ```
    pub fn new(inner: S, window_size: usize) -> Self {
        Self {
            inner,
            window_size,
            values: Arc::new(RwLock::new(VecDeque::with_capacity(window_size))),
        }
    }

    /// Returns the current moving average value.
    ///
    /// Returns `None` if no values have been collected yet, otherwise returns
    /// the arithmetic mean of all values in the current window.
    ///
    /// # Returns
    /// * `Some(f64)` - The current moving average
    /// * `None` - If no values are available for calculation
    pub async fn current_average(&self) -> Option<f64> {
        let values = self.values.read().await;
        if values.is_empty() {
            None
        } else {
            Some(values.iter().sum::<f64>() / values.len() as f64)
        }
    }
}

#[async_trait]
impl<S> Stream for MovingAverageStream<S>
where
    S: Stream + Send + Sync + 'static,
{
    type Config = S::Config;

    async fn start(&mut self, config: Self::Config) -> Result<(), StreamError> {
        self.inner.start(config).await
    }

    async fn stop(&mut self) -> Result<(), StreamError> {
        self.inner.stop().await
    }

    fn subscribe(&self) -> broadcast::Receiver<Arc<DynamicStreamedEvent>> {
        let (tx, rx) = broadcast::channel(10000);
        let mut inner_rx = self.inner.subscribe();
        let values = self.values.clone();
        let window_size = self.window_size;

        tokio::spawn(async move {
            while let Ok(event) = inner_rx.recv().await {
                // Extract value from event if it contains a price
                // This is simplified - in production you'd parse the event properly
                if let Ok(json) = event.inner().to_json() {
                    if let Some(price) = json.get("price").and_then(|p| p.as_f64()) {
                        let mut values_guard = values.write().await;
                        values_guard.push_back(price);
                        if values_guard.len() > window_size {
                            values_guard.pop_front();
                        }
                    }
                }

                // Forward the original event
                let _ = tx.send(event);
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

/// Volume-weighted average price (VWAP) stream
pub struct VWAPStream<S> {
    inner: S,
    time_window: Duration,
    price_volume_pairs: Arc<RwLock<Vec<(f64, f64, std::time::Instant)>>>,
}

impl<S> VWAPStream<S>
where
    S: Stream,
{
    /// Creates a new VWAP stream with the specified time window.
    ///
    /// # Arguments
    /// * `inner` - The underlying stream to wrap
    /// * `time_window` - Duration for which price-volume pairs are considered valid
    ///
    /// # Example
    /// ```
    /// use std::time::Duration;
    /// use riglr_streams::core::financial_operators::VWAPStream;
    /// let vwap_stream = VWAPStream::new(base_stream, Duration::from_secs(300)); // 5 minutes
    /// ```
    pub fn new(inner: S, time_window: Duration) -> Self {
        Self {
            inner,
            time_window,
            price_volume_pairs: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Returns the current volume-weighted average price (VWAP).
    ///
    /// Calculates VWAP using only price-volume pairs within the configured time window.
    /// VWAP is computed as the sum of (price × volume) divided by total volume.
    ///
    /// # Returns
    /// * `Some(f64)` - The current VWAP value
    /// * `None` - If no valid price-volume pairs exist within the time window or total volume is zero
    pub async fn current_vwap(&self) -> Option<f64> {
        let pairs = self.price_volume_pairs.read().await;
        let now = std::time::Instant::now();

        let valid_pairs: Vec<_> = pairs
            .iter()
            .filter(|(_, _, time)| now.duration_since(*time) <= self.time_window)
            .collect();

        if valid_pairs.is_empty() {
            return None;
        }

        let total_volume: f64 = valid_pairs.iter().map(|(_, v, _)| v).sum();
        let weighted_sum: f64 = valid_pairs.iter().map(|(p, v, _)| p * v).sum();

        if total_volume > 0.0 {
            Some(weighted_sum / total_volume)
        } else {
            None
        }
    }
}

#[async_trait]
impl<S> Stream for VWAPStream<S>
where
    S: Stream + Send + Sync + 'static,
{
    type Config = S::Config;

    async fn start(&mut self, config: Self::Config) -> Result<(), StreamError> {
        self.inner.start(config).await
    }

    async fn stop(&mut self) -> Result<(), StreamError> {
        self.inner.stop().await
    }

    fn subscribe(&self) -> broadcast::Receiver<Arc<DynamicStreamedEvent>> {
        let (tx, rx) = broadcast::channel(10000);
        let mut inner_rx = self.inner.subscribe();
        let pairs = self.price_volume_pairs.clone();
        let time_window = self.time_window;

        tokio::spawn(async move {
            while let Ok(event) = inner_rx.recv().await {
                // Extract price and volume from event if available
                if let Ok(json) = event.inner().to_json() {
                    if let (Some(price), Some(volume)) = (
                        json.get("price").and_then(|p| p.as_f64()),
                        json.get("volume").and_then(|v| v.as_f64()),
                    ) {
                        let now = std::time::Instant::now();
                        let mut pairs_guard = pairs.write().await;

                        // Add new pair
                        pairs_guard.push((price, volume, now));

                        // Remove old pairs outside time window
                        pairs_guard.retain(|(_, _, time)| now.duration_since(*time) <= time_window);
                    }
                }

                // Forward the original event
                let _ = tx.send(event);
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
