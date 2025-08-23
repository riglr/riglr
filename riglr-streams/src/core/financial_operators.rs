//! Financial and blockchain-specific stream operators
//!
//! Specialized operators for common DeFi and trading patterns

use async_trait::async_trait;
use dashmap::DashMap;
use std::any::Any;
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::{broadcast, RwLock};

use crate::core::operators::ComposableStream;
use crate::core::{Stream, StreamError, StreamHealth};
use riglr_events_core::prelude::{Event, EventKind};

/// Trait for extracting numeric data from events for financial analysis
pub trait AsNumeric {
    /// Extract price data from the event
    fn as_price(&self) -> Option<f64>;

    /// Extract volume data from the event
    fn as_volume(&self) -> Option<f64>;

    /// Extract market cap or total value if available
    fn as_market_cap(&self) -> Option<f64> {
        None
    }

    /// Extract timestamp as Unix milliseconds for time-series analysis
    fn as_timestamp_ms(&self) -> Option<i64> {
        None
    }

    /// Extract custom numeric field by name (for extensibility)
    fn as_custom_numeric(&self, _field: &str) -> Option<f64> {
        None
    }
}

/// Financial indicator event wrapper
#[derive(Clone, Debug)]
pub struct FinancialEvent<T, E> {
    /// Event metadata containing ID, kind, and source information
    pub metadata: riglr_events_core::EventMetadata,
    /// The calculated indicator value (e.g., VWAP, RSI, etc.)
    pub indicator_value: T,
    /// Reference to the original event that triggered this indicator
    pub original_event: Arc<E>,
    /// Type of financial indicator (e.g., "VWAP", "RSI", "Bollinger")
    pub indicator_type: String,
    /// Timestamp when this indicator was calculated
    pub timestamp: SystemTime,
}

impl<T, E> Event for FinancialEvent<T, E>
where
    T: Clone + Send + Sync + 'static + std::fmt::Debug,
    E: Event + Clone + 'static,
{
    fn id(&self) -> &str {
        &self.metadata.id
    }

    fn kind(&self) -> &riglr_events_core::EventKind {
        &self.metadata.kind
    }

    fn metadata(&self) -> &riglr_events_core::EventMetadata {
        &self.metadata
    }

    fn metadata_mut(&mut self) -> &mut riglr_events_core::EventMetadata {
        &mut self.metadata
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

    fn to_json(&self) -> riglr_events_core::EventResult<serde_json::Value> {
        Ok(serde_json::json!({
            "metadata": self.metadata,
            "indicator_value": format!("{:?}", self.indicator_value),
            "indicator_type": self.indicator_type,
            "timestamp": self.timestamp
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis()
        }))
    }
}

/// Volume-Weighted Average Price (VWAP) calculation
pub struct VwapStream<S> {
    /// The underlying stream to process
    inner: S,
    /// Time window for VWAP calculation
    window: Duration,
    /// Sliding window of (price, volume, timestamp) tuples
    price_volume_pairs: Arc<RwLock<VecDeque<(f64, f64, SystemTime)>>>,
    /// Name of this stream for identification
    name: String,
}

impl<S: Stream> VwapStream<S> {
    /// Creates a new VWAP stream with the specified time window
    pub fn new(inner: S, window: Duration) -> Self {
        let name = format!("vwap({})", inner.name());
        Self {
            inner,
            window,
            price_volume_pairs: Arc::new(RwLock::new(VecDeque::new())),
            name,
        }
    }

    /// Calculates the current VWAP based on stored price-volume pairs
    #[allow(dead_code)]
    async fn calculate_vwap(&self) -> f64 {
        let mut pairs = self.price_volume_pairs.write().await;
        let now = SystemTime::now();

        // Remove old entries
        pairs.retain(|(_, _, time)| now.duration_since(*time).unwrap_or_default() < self.window);

        // Calculate VWAP
        let (total_value, total_volume) = pairs
            .iter()
            .fold((0.0, 0.0), |(val, vol), (price, volume, _)| {
                (val + price * volume, vol + volume)
            });

        if total_volume > 0.0 {
            total_value / total_volume
        } else {
            0.0
        }
    }

    /// Extract price/volume using AsNumeric trait
    fn extract_price_volume(event: &S::Event) -> Option<(f64, f64)>
    where
        S::Event: AsNumeric,
    {
        match (event.as_price(), event.as_volume()) {
            (Some(price), Some(volume)) => Some((price, volume)),
            _ => None,
        }
    }
}

#[async_trait]
impl<S> Stream for VwapStream<S>
where
    S: Stream + Send + Sync + 'static,
    S::Event: AsNumeric,
{
    type Event = FinancialEvent<f64, S::Event>;
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
        let price_volume_pairs = self.price_volume_pairs.clone();
        let window = self.window;

        tokio::spawn(async move {
            while let Ok(event) = inner_rx.recv().await {
                if let Some((price, volume)) = Self::extract_price_volume(&event) {
                    // Add to price/volume pairs
                    {
                        let mut pairs = price_volume_pairs.write().await;
                        pairs.push_back((price, volume, SystemTime::now()));
                    }

                    // Calculate VWAP
                    let vwap = {
                        let mut pairs = price_volume_pairs.write().await;
                        let now = SystemTime::now();

                        // Remove old entries
                        pairs.retain(|(_, _, time)| {
                            now.duration_since(*time).unwrap_or_default() < window
                        });

                        // Calculate VWAP
                        let (total_value, total_volume) = pairs
                            .iter()
                            .fold((0.0, 0.0), |(val, vol), (price, volume, _)| {
                                (val + price * volume, vol + volume)
                            });

                        if total_volume > 0.0 {
                            total_value / total_volume
                        } else {
                            0.0
                        }
                    };

                    let metadata = riglr_events_core::EventMetadata::new(
                        format!("vwap-{}", event.id()),
                        riglr_events_core::EventKind::Price,
                        "financial-vwap".to_string(),
                    );

                    let financial_event = FinancialEvent {
                        metadata,
                        indicator_value: vwap,
                        original_event: event,
                        indicator_type: "VWAP".to_string(),
                        timestamp: SystemTime::now(),
                    };

                    let _ = tx.send(Arc::new(financial_event));
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

/// Moving average calculation stream
#[allow(dead_code)]
pub struct MovingAverageStream<S> {
    /// The underlying stream to process
    inner: S,
    /// Number of values to include in the moving average
    window_size: usize,
    /// Sliding window of values for calculation
    values: Arc<RwLock<VecDeque<f64>>>,
}

impl<S: Stream> MovingAverageStream<S> {
    /// Creates a new moving average stream with the specified window size
    pub fn new(inner: S, window_size: usize) -> Self {
        Self {
            inner,
            window_size,
            values: Arc::new(RwLock::new(VecDeque::with_capacity(window_size))),
        }
    }

    /// Adds a new value to the moving average window and returns the updated average
    #[allow(dead_code)]
    async fn add_value(&self, value: f64) -> f64 {
        let mut values = self.values.write().await;

        values.push_back(value);
        if values.len() > self.window_size {
            values.pop_front();
        }

        let sum: f64 = values.iter().sum();
        sum / values.len() as f64
    }
}

/// Exponential moving average stream
#[allow(dead_code)]
pub struct EmaStream<S> {
    /// The underlying stream to process
    inner: S,
    /// Smoothing factor (alpha) for the EMA calculation
    alpha: f64,
    /// Current EMA value, None if no values processed yet
    current_ema: Arc<RwLock<Option<f64>>>,
}

impl<S: Stream> EmaStream<S> {
    /// Creates a new EMA stream with the specified number of periods
    pub fn new(inner: S, periods: usize) -> Self {
        let alpha = 2.0 / (periods as f64 + 1.0);
        Self {
            inner,
            alpha,
            current_ema: Arc::new(RwLock::new(None)),
        }
    }

    /// Updates the EMA with a new value and returns the calculated EMA
    #[allow(dead_code)]
    async fn update(&self, value: f64) -> f64 {
        let mut ema = self.current_ema.write().await;

        let new_ema = match *ema {
            Some(prev) => self.alpha * value + (1.0 - self.alpha) * prev,
            None => value,
        };

        *ema = Some(new_ema);
        new_ema
    }
}

/// Bollinger Bands calculation stream
#[allow(dead_code)]
pub struct BollingerBandsStream<S> {
    /// The underlying stream to process
    inner: S,
    /// Number of periods for the moving average and standard deviation
    window: usize,
    /// Multiplier for standard deviation to create upper/lower bands
    std_dev_multiplier: f64,
    /// Sliding window of values for calculation
    values: Arc<RwLock<VecDeque<f64>>>,
}

impl<S: Stream> BollingerBandsStream<S> {
    /// Creates a new Bollinger Bands stream with specified window and standard deviation multiplier
    pub fn new(inner: S, window: usize, std_dev_multiplier: f64) -> Self {
        Self {
            inner,
            window,
            std_dev_multiplier,
            values: Arc::new(RwLock::new(VecDeque::with_capacity(window))),
        }
    }

    /// Calculates Bollinger Bands for the given value
    #[allow(dead_code)]
    async fn calculate_bands(&self, value: f64) -> BollingerBands {
        let mut values = self.values.write().await;

        values.push_back(value);
        if values.len() > self.window {
            values.pop_front();
        }

        let mean: f64 = values.iter().sum::<f64>() / values.len() as f64;
        let variance: f64 =
            values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64;
        let std_dev: f64 = variance.sqrt();

        BollingerBands {
            upper: mean + self.std_dev_multiplier * std_dev,
            middle: mean,
            lower: mean - self.std_dev_multiplier * std_dev,
            current_value: value,
        }
    }
}

/// Bollinger Bands calculation result
#[derive(Debug, Clone)]
pub struct BollingerBands {
    /// Upper band (mean + std_dev_multiplier * std_dev)
    pub upper: f64,
    /// Middle band (simple moving average)
    pub middle: f64,
    /// Lower band (mean - std_dev_multiplier * std_dev)
    pub lower: f64,
    /// Current price value
    pub current_value: f64,
}

/// RSI (Relative Strength Index) calculation stream
#[allow(dead_code)]
pub struct RsiStream<S> {
    /// The underlying stream to process
    inner: S,
    /// Period for RSI calculation (typically 14)
    period: usize,
    /// Window of gains for average gain calculation
    gains: Arc<RwLock<VecDeque<f64>>>,
    /// Window of losses for average loss calculation
    losses: Arc<RwLock<VecDeque<f64>>>,
    /// Last processed value for calculating price changes
    last_value: Arc<RwLock<Option<f64>>>,
}

impl<S: Stream> RsiStream<S> {
    /// Creates a new RSI stream with the specified period
    pub fn new(inner: S, period: usize) -> Self {
        Self {
            inner,
            period,
            gains: Arc::new(RwLock::new(VecDeque::with_capacity(period))),
            losses: Arc::new(RwLock::new(VecDeque::with_capacity(period))),
            last_value: Arc::new(RwLock::new(None)),
        }
    }

    /// Calculates RSI for the given value, returns None until enough data is collected
    #[allow(dead_code)]
    async fn calculate_rsi(&self, value: f64) -> Option<f64> {
        let mut last = self.last_value.write().await;

        if let Some(prev) = *last {
            let change = value - prev;
            let gain = if change > 0.0 { change } else { 0.0 };
            let loss = if change < 0.0 { -change } else { 0.0 };

            let mut gains = self.gains.write().await;
            let mut losses = self.losses.write().await;

            gains.push_back(gain);
            losses.push_back(loss);

            if gains.len() > self.period {
                gains.pop_front();
                losses.pop_front();
            }

            if gains.len() == self.period {
                let avg_gain = gains.iter().sum::<f64>() / self.period as f64;
                let avg_loss = losses.iter().sum::<f64>() / self.period as f64;

                if avg_loss == 0.0 {
                    Some(100.0)
                } else {
                    let rs = avg_gain / avg_loss;
                    Some(100.0 - (100.0 / (1.0 + rs)))
                }
            } else {
                None
            }
        } else {
            *last = Some(value);
            None
        }
    }
}

/// Order book imbalance calculator stream
#[allow(dead_code)]
pub struct OrderBookImbalanceStream<S> {
    /// The underlying stream to process
    inner: S,
    /// Number of order book depth levels to consider
    depth_levels: usize,
}

impl<S: Stream> OrderBookImbalanceStream<S> {
    /// Creates a new order book imbalance stream with specified depth levels
    pub fn new(inner: S, depth_levels: usize) -> Self {
        Self {
            inner,
            depth_levels,
        }
    }

    /// Calculates order book imbalance from bid/ask data
    pub fn calculate_imbalance(bids: &[(f64, f64)], asks: &[(f64, f64)]) -> f64 {
        let bid_volume: f64 = bids.iter().take(5).map(|(_, vol)| vol).sum();
        let ask_volume: f64 = asks.iter().take(5).map(|(_, vol)| vol).sum();

        if bid_volume + ask_volume > 0.0 {
            (bid_volume - ask_volume) / (bid_volume + ask_volume)
        } else {
            0.0
        }
    }
}

/// Price momentum indicator stream
#[allow(dead_code)]
pub struct MomentumStream<S> {
    /// The underlying stream to process
    inner: S,
    /// Number of periods to look back for momentum calculation
    lookback_period: usize,
    /// Historical price data for momentum calculation
    price_history: Arc<RwLock<VecDeque<f64>>>,
}

impl<S: Stream> MomentumStream<S> {
    /// Creates a new momentum stream with specified lookback period
    pub fn new(inner: S, lookback_period: usize) -> Self {
        Self {
            inner,
            lookback_period,
            price_history: Arc::new(RwLock::new(VecDeque::with_capacity(lookback_period + 1))),
        }
    }

    /// Calculates price momentum as percentage change from lookback period
    #[allow(dead_code)]
    async fn calculate_momentum(&self, current_price: f64) -> Option<f64> {
        let mut history = self.price_history.write().await;

        history.push_back(current_price);
        if history.len() > self.lookback_period + 1 {
            history.pop_front();
        }

        if history.len() > self.lookback_period {
            let old_price = history[0];
            Some(((current_price - old_price) / old_price) * 100.0)
        } else {
            None
        }
    }
}

/// Liquidity pool balance tracker stream
#[allow(dead_code)]
pub struct LiquidityPoolStream<S> {
    /// The underlying stream to process
    inner: S,
    /// Map of pool ID to pool state using high-performance concurrent map
    pools: Arc<DashMap<String, PoolState>>,
}

/// State of a liquidity pool
#[derive(Debug, Clone)]
pub struct PoolState {
    /// Reserve amount of token A
    pub token_a_reserve: f64,
    /// Reserve amount of token B
    pub token_b_reserve: f64,
    /// Constant product (k = x * y)
    pub k_constant: f64,
    /// Last time this pool state was updated
    pub last_updated: SystemTime,
}

impl<S: Stream> LiquidityPoolStream<S> {
    /// Creates a new liquidity pool tracker stream
    pub fn new(inner: S) -> Self {
        Self {
            inner,
            pools: Arc::new(DashMap::new()),
        }
    }

    /// Updates the state of a liquidity pool
    #[allow(dead_code)]
    async fn update_pool(&self, pool_id: String, token_a: f64, token_b: f64) {
        self.pools.insert(
            pool_id,
            PoolState {
                token_a_reserve: token_a,
                token_b_reserve: token_b,
                k_constant: token_a * token_b,
                last_updated: SystemTime::now(),
            },
        );
    }

    /// Calculates the price impact of a trade on the pool
    #[allow(dead_code)]
    async fn calculate_price_impact(&self, pool_id: &str, trade_amount: f64) -> Option<f64> {
        if let Some(pool) = self.pools.get(pool_id) {
            let new_reserve_a = pool.token_a_reserve + trade_amount;
            let new_reserve_b = pool.k_constant / new_reserve_a;
            let amount_out = pool.token_b_reserve - new_reserve_b;
            let price_impact = (amount_out / pool.token_b_reserve) * 100.0;
            Some(price_impact)
        } else {
            None
        }
    }
}

/// MEV (Maximum Extractable Value) detection stream
#[allow(dead_code)]
pub struct MevDetectionStream<S> {
    /// The underlying stream to process
    inner: S,
    /// Time window for MEV pattern detection
    window: Duration,
    /// Transaction patterns within the detection window
    transactions: Arc<RwLock<VecDeque<TransactionPattern>>>,
}

/// Transaction pattern for MEV detection
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct TransactionPattern {
    /// Transaction hash identifier
    pub tx_hash: String,
    /// When this transaction was observed
    pub timestamp: SystemTime,
    /// Type of MEV pattern detected
    pub pattern_type: MevType,
}

/// Types of MEV (Maximum Extractable Value) patterns
#[derive(Debug, Clone, PartialEq)]
pub enum MevType {
    /// Sandwich attack - placing transactions before and after a target
    Sandwich,
    /// Front-running - placing a transaction before a target transaction
    Frontrun,
    /// Back-running - placing a transaction after a target transaction
    Backrun,
    /// Arbitrage opportunity - price differences across markets
    Arbitrage,
}

impl<S: Stream> MevDetectionStream<S> {
    /// Creates a new MEV detection stream with specified time window
    pub fn new(inner: S, window: Duration) -> Self {
        Self {
            inner,
            window,
            transactions: Arc::new(RwLock::new(VecDeque::new())),
        }
    }

    /// Detects MEV patterns from transaction events
    #[allow(dead_code)]
    async fn detect_mev(&self, event: &dyn Event) -> Option<MevType> {
        // Simplified MEV detection logic
        let mut txs = self.transactions.write().await;
        let now = SystemTime::now();

        // Clean old transactions
        txs.retain(|tx| now.duration_since(tx.timestamp).unwrap_or_default() < self.window);

        // Look for patterns
        if matches!(event.kind(), EventKind::Swap) {
            // Check for sandwich attacks (simplified)
            let recent_swaps: Vec<_> = txs
                .iter()
                .filter(|tx| matches!(tx.pattern_type, MevType::Sandwich))
                .collect();

            if recent_swaps.len() >= 2 {
                return Some(MevType::Sandwich);
            }
        }

        None
    }
}

/// Gas price oracle stream for tracking network gas prices
#[allow(dead_code)]
pub struct GasPriceOracleStream<S> {
    /// The underlying stream to process
    inner: S,
    /// Percentiles to calculate for gas price estimates
    percentiles: Vec<usize>,
    /// Historical gas prices for percentile calculation
    gas_prices: Arc<RwLock<VecDeque<f64>>>,
    /// Number of historical prices to maintain
    window_size: usize,
}

impl<S: Stream> GasPriceOracleStream<S> {
    /// Creates a new gas price oracle stream with specified window size
    pub fn new(inner: S, window_size: usize) -> Self {
        Self {
            inner,
            percentiles: vec![25, 50, 75, 95],
            gas_prices: Arc::new(RwLock::new(VecDeque::with_capacity(window_size))),
            window_size,
        }
    }

    /// Updates the gas price history and returns current estimates
    #[allow(dead_code)]
    async fn update_gas_price(&self, gas_price: f64) -> GasPriceEstimate {
        let mut prices = self.gas_prices.write().await;

        prices.push_back(gas_price);
        if prices.len() > self.window_size {
            prices.pop_front();
        }

        let mut sorted: Vec<f64> = prices.iter().copied().collect();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

        GasPriceEstimate {
            slow: sorted[sorted.len() * 25 / 100],
            standard: sorted[sorted.len() * 50 / 100],
            fast: sorted[sorted.len() * 75 / 100],
            instant: sorted[sorted.len() * 95 / 100],
        }
    }
}

/// Gas price estimates at different priority levels
#[derive(Debug, Clone)]
pub struct GasPriceEstimate {
    /// 25th percentile gas price (slow transactions)
    pub slow: f64,
    /// 50th percentile gas price (standard transactions)
    pub standard: f64,
    /// 75th percentile gas price (fast transactions)
    pub fast: f64,
    /// 95th percentile gas price (instant transactions)
    pub instant: f64,
}

/// Extension trait for adding financial operators
pub trait FinancialStreamExt: ComposableStream {
    /// Calculate VWAP over a time window
    fn vwap(self, window: Duration) -> VwapStream<Self>
    where
        Self: Sized,
        Self::Event: AsNumeric,
    {
        VwapStream::new(self, window)
    }

    /// Calculate moving average
    fn moving_average(self, window_size: usize) -> MovingAverageStream<Self>
    where
        Self: Sized,
    {
        MovingAverageStream::new(self, window_size)
    }

    /// Calculate exponential moving average
    fn ema(self, periods: usize) -> EmaStream<Self>
    where
        Self: Sized,
    {
        EmaStream::new(self, periods)
    }

    /// Calculate Bollinger Bands
    fn bollinger_bands(self, window: usize, std_dev: f64) -> BollingerBandsStream<Self>
    where
        Self: Sized,
    {
        BollingerBandsStream::new(self, window, std_dev)
    }

    /// Calculate RSI
    fn rsi(self, period: usize) -> RsiStream<Self>
    where
        Self: Sized,
    {
        RsiStream::new(self, period)
    }

    /// Calculate momentum
    fn momentum(self, lookback: usize) -> MomentumStream<Self>
    where
        Self: Sized,
    {
        MomentumStream::new(self, lookback)
    }

    /// Track liquidity pools
    fn liquidity_pools(self) -> LiquidityPoolStream<Self>
    where
        Self: Sized,
    {
        LiquidityPoolStream::new(self)
    }

    /// Detect MEV
    fn mev_detection(self, window: Duration) -> MevDetectionStream<Self>
    where
        Self: Sized,
    {
        MevDetectionStream::new(self, window)
    }

    /// Track gas prices
    fn gas_oracle(self, window_size: usize) -> GasPriceOracleStream<Self>
    where
        Self: Sized,
    {
        GasPriceOracleStream::new(self, window_size)
    }
}

/// Implement the extension trait for all composable streams
impl<T> FinancialStreamExt for T where T: ComposableStream {}

#[cfg(test)]
mod tests {
    use super::*;
    use riglr_events_core::{EventKind, EventMetadata};
    use std::time::{Duration, SystemTime, UNIX_EPOCH};
    use tokio::time::timeout;

    // Mock event for testing AsNumeric trait
    #[derive(Clone, Debug)]
    struct MockEvent {
        id: String,
        kind: EventKind,
        price: Option<f64>,
        volume: Option<f64>,
        market_cap: Option<f64>,
        timestamp_ms: Option<i64>,
        metadata: EventMetadata,
    }

    impl Event for MockEvent {
        fn id(&self) -> &str {
            &self.id
        }

        fn kind(&self) -> &EventKind {
            &self.kind
        }

        fn metadata(&self) -> &EventMetadata {
            &self.metadata
        }

        fn metadata_mut(&mut self) -> &mut EventMetadata {
            &mut self.metadata
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

        fn to_json(&self) -> riglr_events_core::EventResult<serde_json::Value> {
            Ok(serde_json::json!({
                "id": self.id,
                "kind": self.kind,
                "price": self.price,
                "volume": self.volume
            }))
        }
    }

    impl AsNumeric for MockEvent {
        fn as_price(&self) -> Option<f64> {
            self.price
        }

        fn as_volume(&self) -> Option<f64> {
            self.volume
        }

        fn as_market_cap(&self) -> Option<f64> {
            self.market_cap
        }

        fn as_timestamp_ms(&self) -> Option<i64> {
            self.timestamp_ms
        }

        fn as_custom_numeric(&self, field: &str) -> Option<f64> {
            match field {
                "test_field" => Some(42.0),
                _ => None,
            }
        }
    }

    // Mock stream for testing
    struct MockStream {
        name: String,
        running: bool,
    }

    impl MockStream {
        fn new(name: &str) -> Self {
            Self {
                name: name.to_string(),
                running: false,
            }
        }
    }

    #[async_trait]
    impl Stream for MockStream {
        type Event = MockEvent;
        type Config = ();

        async fn start(&mut self, _config: Self::Config) -> Result<(), StreamError> {
            self.running = true;
            Ok(())
        }

        async fn stop(&mut self) -> Result<(), StreamError> {
            self.running = false;
            Ok(())
        }

        fn subscribe(&self) -> broadcast::Receiver<Arc<Self::Event>> {
            let (tx, rx) = broadcast::channel(10);

            let event = MockEvent {
                id: "test".to_string(),
                kind: EventKind::Price,
                price: Some(100.0),
                volume: Some(1000.0),
                market_cap: Some(1_000_000.0),
                timestamp_ms: Some(1234567890),
                metadata: EventMetadata::new(
                    "test".to_string(),
                    EventKind::Price,
                    "test".to_string(),
                ),
            };

            tokio::spawn(async move {
                let _ = tx.send(Arc::new(event));
            });

            rx
        }

        fn is_running(&self) -> bool {
            self.running
        }

        async fn health(&self) -> StreamHealth {
            StreamHealth::Healthy
        }

        fn name(&self) -> &str {
            &self.name
        }
    }

    // Tests for AsNumeric trait default implementations
    #[test]
    fn test_as_numeric_default_market_cap_should_return_none() {
        struct TestEvent;
        impl AsNumeric for TestEvent {
            fn as_price(&self) -> Option<f64> {
                Some(100.0)
            }
            fn as_volume(&self) -> Option<f64> {
                Some(1000.0)
            }
        }

        let event = TestEvent;
        assert_eq!(event.as_market_cap(), None);
    }

    #[test]
    fn test_as_numeric_default_timestamp_ms_should_return_none() {
        struct TestEvent;
        impl AsNumeric for TestEvent {
            fn as_price(&self) -> Option<f64> {
                Some(100.0)
            }
            fn as_volume(&self) -> Option<f64> {
                Some(1000.0)
            }
        }

        let event = TestEvent;
        assert_eq!(event.as_timestamp_ms(), None);
    }

    #[test]
    fn test_as_numeric_default_custom_numeric_should_return_none() {
        struct TestEvent;
        impl AsNumeric for TestEvent {
            fn as_price(&self) -> Option<f64> {
                Some(100.0)
            }
            fn as_volume(&self) -> Option<f64> {
                Some(1000.0)
            }
        }

        let event = TestEvent;
        assert_eq!(event.as_custom_numeric("any_field"), None);
    }

    #[test]
    fn test_as_numeric_custom_implementations_should_return_values() {
        let event = MockEvent {
            id: "test".to_string(),
            kind: EventKind::Price,
            price: Some(150.0),
            volume: Some(2000.0),
            market_cap: Some(5_000_000.0),
            timestamp_ms: Some(9876543210),
            metadata: EventMetadata::new("test".to_string(), EventKind::Price, "test".to_string()),
        };

        assert_eq!(event.as_price(), Some(150.0));
        assert_eq!(event.as_volume(), Some(2000.0));
        assert_eq!(event.as_market_cap(), Some(5_000_000.0));
        assert_eq!(event.as_timestamp_ms(), Some(9876543210));
        assert_eq!(event.as_custom_numeric("test_field"), Some(42.0));
        assert_eq!(event.as_custom_numeric("unknown_field"), None);
    }

    // Tests for FinancialEvent
    #[test]
    fn test_financial_event_creation_should_initialize_correctly() {
        let metadata = EventMetadata::new(
            "test-id".to_string(),
            EventKind::Price,
            "test-source".to_string(),
        );
        let original_event = Arc::new(MockEvent {
            id: "orig".to_string(),
            kind: EventKind::Swap,
            price: Some(100.0),
            volume: Some(1000.0),
            market_cap: None,
            timestamp_ms: None,
            metadata: EventMetadata::new("orig".to_string(), EventKind::Swap, "test".to_string()),
        });
        let timestamp = SystemTime::now();

        let financial_event = FinancialEvent {
            metadata: metadata.clone(),
            indicator_value: 42.5,
            original_event: original_event.clone(),
            indicator_type: "TEST".to_string(),
            timestamp,
        };

        assert_eq!(financial_event.id(), "test-id");
        assert_eq!(financial_event.kind(), &EventKind::Price);
        assert_eq!(financial_event.indicator_value, 42.5);
        assert_eq!(financial_event.indicator_type, "TEST");
        assert_eq!(financial_event.original_event.id(), "orig");
    }

    #[test]
    fn test_financial_event_metadata_mut_should_allow_modification() {
        let metadata = EventMetadata::new(
            "test-id".to_string(),
            EventKind::Price,
            "test-source".to_string(),
        );
        let original_event = Arc::new(MockEvent {
            id: "orig".to_string(),
            kind: EventKind::Swap,
            price: Some(100.0),
            volume: Some(1000.0),
            market_cap: None,
            timestamp_ms: None,
            metadata: EventMetadata::new("orig".to_string(), EventKind::Swap, "test".to_string()),
        });

        let mut financial_event = FinancialEvent {
            metadata,
            indicator_value: 42.5,
            original_event,
            indicator_type: "TEST".to_string(),
            timestamp: SystemTime::now(),
        };

        let metadata_mut = financial_event.metadata_mut();
        metadata_mut.id = "new-id".to_string();
        assert_eq!(financial_event.id(), "new-id");
    }

    #[test]
    fn test_financial_event_as_any_should_return_self() {
        let metadata = EventMetadata::new(
            "test-id".to_string(),
            EventKind::Price,
            "test-source".to_string(),
        );
        let original_event = Arc::new(MockEvent {
            id: "orig".to_string(),
            kind: EventKind::Swap,
            price: Some(100.0),
            volume: Some(1000.0),
            market_cap: None,
            timestamp_ms: None,
            metadata: EventMetadata::new("orig".to_string(), EventKind::Swap, "test".to_string()),
        });

        let financial_event = FinancialEvent {
            metadata,
            indicator_value: 42.5,
            original_event,
            indicator_type: "TEST".to_string(),
            timestamp: SystemTime::now(),
        };

        let any_ref = financial_event.as_any();
        assert!(any_ref
            .downcast_ref::<FinancialEvent<f64, MockEvent>>()
            .is_some());
    }

    #[test]
    fn test_financial_event_as_any_mut_should_return_mutable_self() {
        let metadata = EventMetadata::new(
            "test-id".to_string(),
            EventKind::Price,
            "test-source".to_string(),
        );
        let original_event = Arc::new(MockEvent {
            id: "orig".to_string(),
            kind: EventKind::Swap,
            price: Some(100.0),
            volume: Some(1000.0),
            market_cap: None,
            timestamp_ms: None,
            metadata: EventMetadata::new("orig".to_string(), EventKind::Swap, "test".to_string()),
        });

        let mut financial_event = FinancialEvent {
            metadata,
            indicator_value: 42.5,
            original_event,
            indicator_type: "TEST".to_string(),
            timestamp: SystemTime::now(),
        };

        let any_mut = financial_event.as_any_mut();
        assert!(any_mut
            .downcast_mut::<FinancialEvent<f64, MockEvent>>()
            .is_some());
    }

    #[test]
    fn test_financial_event_clone_boxed_should_create_boxed_clone() {
        let metadata = EventMetadata::new(
            "test-id".to_string(),
            EventKind::Price,
            "test-source".to_string(),
        );
        let original_event = Arc::new(MockEvent {
            id: "orig".to_string(),
            kind: EventKind::Swap,
            price: Some(100.0),
            volume: Some(1000.0),
            market_cap: None,
            timestamp_ms: None,
            metadata: EventMetadata::new("orig".to_string(), EventKind::Swap, "test".to_string()),
        });

        let financial_event = FinancialEvent {
            metadata,
            indicator_value: 42.5,
            original_event,
            indicator_type: "TEST".to_string(),
            timestamp: SystemTime::now(),
        };

        let boxed_clone = financial_event.clone_boxed();
        assert_eq!(boxed_clone.id(), "test-id");
    }

    #[test]
    fn test_financial_event_to_json_should_serialize_correctly() {
        let metadata = EventMetadata::new(
            "test-id".to_string(),
            EventKind::Price,
            "test-source".to_string(),
        );
        let original_event = Arc::new(MockEvent {
            id: "orig".to_string(),
            kind: EventKind::Swap,
            price: Some(100.0),
            volume: Some(1000.0),
            market_cap: None,
            timestamp_ms: None,
            metadata: EventMetadata::new("orig".to_string(), EventKind::Swap, "test".to_string()),
        });
        let timestamp = UNIX_EPOCH + Duration::from_millis(1234567890);

        let financial_event = FinancialEvent {
            metadata,
            indicator_value: 42.5,
            original_event,
            indicator_type: "TEST".to_string(),
            timestamp,
        };

        let json = financial_event.to_json().unwrap();
        assert!(json["metadata"].is_object());
        assert_eq!(json["indicator_value"], "42.5");
        assert_eq!(json["indicator_type"], "TEST");
        assert_eq!(json["timestamp"], 1234567890);
    }

    #[test]
    fn test_financial_event_to_json_when_timestamp_before_epoch_should_use_default() {
        let metadata = EventMetadata::new(
            "test-id".to_string(),
            EventKind::Price,
            "test-source".to_string(),
        );
        let original_event = Arc::new(MockEvent {
            id: "orig".to_string(),
            kind: EventKind::Swap,
            price: Some(100.0),
            volume: Some(1000.0),
            market_cap: None,
            timestamp_ms: None,
            metadata: EventMetadata::new("orig".to_string(), EventKind::Swap, "test".to_string()),
        });
        // Set timestamp before epoch
        let timestamp = UNIX_EPOCH - Duration::from_secs(1);

        let financial_event = FinancialEvent {
            metadata,
            indicator_value: 42.5,
            original_event,
            indicator_type: "TEST".to_string(),
            timestamp,
        };

        let json = financial_event.to_json().unwrap();
        assert_eq!(json["timestamp"], 0);
    }

    // Tests for VwapStream
    #[test]
    fn test_vwap_stream_new_should_create_correctly() {
        let mock_stream = MockStream::new("test");
        let window = Duration::from_secs(60);
        let vwap_stream = VwapStream::new(mock_stream, window);

        assert_eq!(vwap_stream.name(), "vwap(test)");
        assert_eq!(vwap_stream.window, window);
    }

    #[tokio::test]
    async fn test_vwap_stream_calculate_vwap_when_empty_should_return_zero() {
        let mock_stream = MockStream::new("test");
        let window = Duration::from_secs(60);
        let vwap_stream = VwapStream::new(mock_stream, window);

        let vwap = vwap_stream.calculate_vwap().await;
        assert_eq!(vwap, 0.0);
    }

    #[tokio::test]
    async fn test_vwap_stream_calculate_vwap_with_data_should_return_weighted_average() {
        let mock_stream = MockStream::new("test");
        let window = Duration::from_secs(60);
        let vwap_stream = VwapStream::new(mock_stream, window);

        // Add some price-volume pairs
        {
            let mut pairs = vwap_stream.price_volume_pairs.write().await;
            pairs.push_back((100.0, 1000.0, SystemTime::now()));
            pairs.push_back((110.0, 500.0, SystemTime::now()));
        }

        let vwap = vwap_stream.calculate_vwap().await;
        // VWAP = (100*1000 + 110*500) / (1000+500) = 155000 / 1500 = 103.33...
        assert!((vwap - 103.33333333333333).abs() < 0.0001);
    }

    #[tokio::test]
    async fn test_vwap_stream_calculate_vwap_should_remove_old_entries() {
        let mock_stream = MockStream::new("test");
        let window = Duration::from_millis(100);
        let vwap_stream = VwapStream::new(mock_stream, window);

        // Add old entry
        {
            let mut pairs = vwap_stream.price_volume_pairs.write().await;
            pairs.push_back((50.0, 1000.0, SystemTime::now() - Duration::from_secs(1)));
            pairs.push_back((100.0, 1000.0, SystemTime::now()));
        }

        let vwap = vwap_stream.calculate_vwap().await;
        // Should only consider the recent entry
        assert_eq!(vwap, 100.0);
    }

    #[test]
    fn test_vwap_stream_extract_price_volume_when_both_present_should_return_some() {
        let event = MockEvent {
            id: "test".to_string(),
            kind: EventKind::Price,
            price: Some(100.0),
            volume: Some(1000.0),
            market_cap: None,
            timestamp_ms: None,
            metadata: EventMetadata::new("test".to_string(), EventKind::Price, "test".to_string()),
        };

        let result = VwapStream::<MockStream>::extract_price_volume(&event);
        assert_eq!(result, Some((100.0, 1000.0)));
    }

    #[test]
    fn test_vwap_stream_extract_price_volume_when_price_missing_should_return_none() {
        let event = MockEvent {
            id: "test".to_string(),
            kind: EventKind::Price,
            price: None,
            volume: Some(1000.0),
            market_cap: None,
            timestamp_ms: None,
            metadata: EventMetadata::new("test".to_string(), EventKind::Price, "test".to_string()),
        };

        let result = VwapStream::<MockStream>::extract_price_volume(&event);
        assert_eq!(result, None);
    }

    #[test]
    fn test_vwap_stream_extract_price_volume_when_volume_missing_should_return_none() {
        let event = MockEvent {
            id: "test".to_string(),
            kind: EventKind::Price,
            price: Some(100.0),
            volume: None,
            market_cap: None,
            timestamp_ms: None,
            metadata: EventMetadata::new("test".to_string(), EventKind::Price, "test".to_string()),
        };

        let result = VwapStream::<MockStream>::extract_price_volume(&event);
        assert_eq!(result, None);
    }

    #[test]
    fn test_vwap_stream_extract_price_volume_when_both_missing_should_return_none() {
        let event = MockEvent {
            id: "test".to_string(),
            kind: EventKind::Price,
            price: None,
            volume: None,
            market_cap: None,
            timestamp_ms: None,
            metadata: EventMetadata::new("test".to_string(), EventKind::Price, "test".to_string()),
        };

        let result = VwapStream::<MockStream>::extract_price_volume(&event);
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn test_vwap_stream_start_should_delegate_to_inner() {
        let mock_stream = MockStream::new("test");
        let mut vwap_stream = VwapStream::new(mock_stream, Duration::from_secs(60));

        let result = vwap_stream.start(()).await;
        assert!(result.is_ok());
        assert!(vwap_stream.inner.is_running());
    }

    #[tokio::test]
    async fn test_vwap_stream_stop_should_delegate_to_inner() {
        let mut mock_stream = MockStream::new("test");
        mock_stream.running = true;
        let mut vwap_stream = VwapStream::new(mock_stream, Duration::from_secs(60));

        let result = vwap_stream.stop().await;
        assert!(result.is_ok());
        assert!(!vwap_stream.inner.is_running());
    }

    #[test]
    fn test_vwap_stream_is_running_should_delegate_to_inner() {
        let mut mock_stream = MockStream::new("test");
        mock_stream.running = true;
        let vwap_stream = VwapStream::new(mock_stream, Duration::from_secs(60));

        assert!(vwap_stream.is_running());
    }

    #[tokio::test]
    async fn test_vwap_stream_health_should_delegate_to_inner() {
        let mock_stream = MockStream::new("test");
        let vwap_stream = VwapStream::new(mock_stream, Duration::from_secs(60));

        let health = vwap_stream.health().await;
        assert_eq!(health, StreamHealth::Healthy);
    }

    #[tokio::test]
    async fn test_vwap_stream_subscribe_should_process_events() {
        let mock_stream = MockStream::new("test");
        let vwap_stream = VwapStream::new(mock_stream, Duration::from_secs(60));

        let mut rx = vwap_stream.subscribe();

        // Wait for event with timeout
        let result = timeout(Duration::from_millis(100), rx.recv()).await;
        assert!(result.is_ok());

        let event = result.unwrap().unwrap();
        assert_eq!(event.indicator_type, "VWAP");
        assert_eq!(event.indicator_value, 100.0); // Single event with price 100, volume 1000
    }

    // Tests for MovingAverageStream
    #[test]
    fn test_moving_average_stream_new_should_create_correctly() {
        let mock_stream = MockStream::new("test");
        let window_size = 5;
        let ma_stream = MovingAverageStream::new(mock_stream, window_size);

        assert_eq!(ma_stream.window_size, window_size);
    }

    #[tokio::test]
    async fn test_moving_average_stream_add_value_when_empty_should_return_value() {
        let mock_stream = MockStream::new("test");
        let ma_stream = MovingAverageStream::new(mock_stream, 3);

        let avg = ma_stream.add_value(100.0).await;
        assert_eq!(avg, 100.0);
    }

    #[tokio::test]
    async fn test_moving_average_stream_add_value_should_calculate_average() {
        let mock_stream = MockStream::new("test");
        let ma_stream = MovingAverageStream::new(mock_stream, 3);

        let avg1 = ma_stream.add_value(100.0).await;
        assert_eq!(avg1, 100.0);

        let avg2 = ma_stream.add_value(200.0).await;
        assert_eq!(avg2, 150.0);

        let avg3 = ma_stream.add_value(300.0).await;
        assert_eq!(avg3, 200.0);
    }

    #[tokio::test]
    async fn test_moving_average_stream_add_value_should_maintain_window_size() {
        let mock_stream = MockStream::new("test");
        let ma_stream = MovingAverageStream::new(mock_stream, 2);

        ma_stream.add_value(100.0).await;
        ma_stream.add_value(200.0).await;
        let avg = ma_stream.add_value(300.0).await; // Should drop 100.0

        assert_eq!(avg, 250.0); // (200 + 300) / 2
    }

    // Tests for EmaStream
    #[test]
    fn test_ema_stream_new_should_calculate_alpha_correctly() {
        let mock_stream = MockStream::new("test");
        let periods = 10;
        let ema_stream = EmaStream::new(mock_stream, periods);

        let expected_alpha = 2.0 / (periods as f64 + 1.0);
        assert_eq!(ema_stream.alpha, expected_alpha);
    }

    #[tokio::test]
    async fn test_ema_stream_update_when_first_value_should_return_value() {
        let mock_stream = MockStream::new("test");
        let ema_stream = EmaStream::new(mock_stream, 10);

        let ema = ema_stream.update(100.0).await;
        assert_eq!(ema, 100.0);
    }

    #[tokio::test]
    async fn test_ema_stream_update_should_calculate_ema() {
        let mock_stream = MockStream::new("test");
        let ema_stream = EmaStream::new(mock_stream, 10);
        let alpha = ema_stream.alpha;

        let ema1 = ema_stream.update(100.0).await;
        assert_eq!(ema1, 100.0);

        let ema2 = ema_stream.update(110.0).await;
        let expected = alpha * 110.0 + (1.0 - alpha) * 100.0;
        assert_eq!(ema2, expected);
    }

    // Tests for BollingerBandsStream and BollingerBands
    #[test]
    fn test_bollinger_bands_stream_new_should_create_correctly() {
        let mock_stream = MockStream::new("test");
        let window = 20;
        let std_dev_multiplier = 2.0;
        let bb_stream = BollingerBandsStream::new(mock_stream, window, std_dev_multiplier);

        assert_eq!(bb_stream.window, window);
        assert_eq!(bb_stream.std_dev_multiplier, std_dev_multiplier);
    }

    #[tokio::test]
    async fn test_bollinger_bands_stream_calculate_bands_with_single_value() {
        let mock_stream = MockStream::new("test");
        let bb_stream = BollingerBandsStream::new(mock_stream, 3, 2.0);

        let bands = bb_stream.calculate_bands(100.0).await;

        assert_eq!(bands.middle, 100.0);
        assert_eq!(bands.upper, 100.0); // No deviation with single value
        assert_eq!(bands.lower, 100.0);
        assert_eq!(bands.current_value, 100.0);
    }

    #[tokio::test]
    async fn test_bollinger_bands_stream_calculate_bands_with_multiple_values() {
        let mock_stream = MockStream::new("test");
        let bb_stream = BollingerBandsStream::new(mock_stream, 3, 2.0);

        bb_stream.calculate_bands(100.0).await;
        bb_stream.calculate_bands(110.0).await;
        let bands = bb_stream.calculate_bands(90.0).await;

        // Mean = (100 + 110 + 90) / 3 = 100
        assert_eq!(bands.middle, 100.0);
        assert_eq!(bands.current_value, 90.0);

        // Variance = ((100-100)^2 + (110-100)^2 + (90-100)^2) / 3 = (0 + 100 + 100) / 3 = 66.666...
        // Std dev = sqrt(66.666...) ≈ 8.164
        // Upper = 100 + 2 * 8.164 ≈ 116.33
        // Lower = 100 - 2 * 8.164 ≈ 83.67
        let expected_variance: f64 = 200.0 / 3.0;
        let expected_std_dev: f64 = expected_variance.sqrt();
        assert!((bands.upper - (100.0 + 2.0 * expected_std_dev)).abs() < 0.0001);
        assert!((bands.lower - (100.0 - 2.0 * expected_std_dev)).abs() < 0.0001);
    }

    #[tokio::test]
    async fn test_bollinger_bands_stream_should_maintain_window_size() {
        let mock_stream = MockStream::new("test");
        let bb_stream = BollingerBandsStream::new(mock_stream, 2, 2.0);

        bb_stream.calculate_bands(100.0).await;
        bb_stream.calculate_bands(110.0).await;
        let bands = bb_stream.calculate_bands(120.0).await; // Should drop 100.0

        // Mean should be (110 + 120) / 2 = 115
        assert_eq!(bands.middle, 115.0);
    }

    // Tests for RsiStream
    #[test]
    fn test_rsi_stream_new_should_create_correctly() {
        let mock_stream = MockStream::new("test");
        let period = 14;
        let rsi_stream = RsiStream::new(mock_stream, period);

        assert_eq!(rsi_stream.period, period);
    }

    #[tokio::test]
    async fn test_rsi_stream_calculate_rsi_when_first_value_should_return_none() {
        let mock_stream = MockStream::new("test");
        let rsi_stream = RsiStream::new(mock_stream, 14);

        let rsi = rsi_stream.calculate_rsi(100.0).await;
        assert_eq!(rsi, None);
    }

    #[tokio::test]
    async fn test_rsi_stream_calculate_rsi_insufficient_data_should_return_none() {
        let mock_stream = MockStream::new("test");
        let rsi_stream = RsiStream::new(mock_stream, 3);

        rsi_stream.calculate_rsi(100.0).await;
        let rsi = rsi_stream.calculate_rsi(110.0).await;
        assert_eq!(rsi, None); // Only 1 price change, need 3
    }

    #[tokio::test]
    async fn test_rsi_stream_calculate_rsi_with_sufficient_data_should_return_value() {
        let mock_stream = MockStream::new("test");
        let rsi_stream = RsiStream::new(mock_stream, 2);

        rsi_stream.calculate_rsi(100.0).await;
        rsi_stream.calculate_rsi(110.0).await; // +10 gain
        let rsi = rsi_stream.calculate_rsi(105.0).await; // -5 loss

        // Gains: [10, 0], Losses: [0, 5]
        // Avg gain = 5, Avg loss = 2.5
        // RS = 5 / 2.5 = 2
        // RSI = 100 - (100 / (1 + 2)) = 100 - 33.333... = 66.666...
        assert!(rsi.is_some());
        let rsi_value = rsi.unwrap();
        assert!((rsi_value - 66.66666666666667).abs() < 0.0001);
    }

    #[tokio::test]
    async fn test_rsi_stream_calculate_rsi_when_no_losses_should_return_100() {
        let mock_stream = MockStream::new("test");
        let rsi_stream = RsiStream::new(mock_stream, 2);

        rsi_stream.calculate_rsi(100.0).await;
        rsi_stream.calculate_rsi(110.0).await; // +10 gain
        let rsi = rsi_stream.calculate_rsi(120.0).await; // +10 gain

        // All gains, no losses
        assert_eq!(rsi, Some(100.0));
    }

    #[tokio::test]
    async fn test_rsi_stream_should_maintain_period_window() {
        let mock_stream = MockStream::new("test");
        let rsi_stream = RsiStream::new(mock_stream, 2);

        rsi_stream.calculate_rsi(100.0).await;
        rsi_stream.calculate_rsi(110.0).await;
        rsi_stream.calculate_rsi(105.0).await;
        let rsi = rsi_stream.calculate_rsi(115.0).await; // Should drop first gain

        // Latest changes: 105->115 (+10), previous: 110->105 (-5)
        // Gains: [0, 10], Losses: [5, 0]
        // Avg gain = 5, Avg loss = 2.5
        assert!(rsi.is_some());
    }

    // Tests for OrderBookImbalanceStream
    #[test]
    fn test_order_book_imbalance_stream_new_should_create_correctly() {
        let mock_stream = MockStream::new("test");
        let depth_levels = 5;
        let imbalance_stream = OrderBookImbalanceStream::new(mock_stream, depth_levels);

        assert_eq!(imbalance_stream.depth_levels, depth_levels);
    }

    #[test]
    fn test_order_book_imbalance_calculate_imbalance_balanced_should_return_zero() {
        let bids = vec![(100.0, 1000.0), (99.0, 1000.0)];
        let asks = vec![(101.0, 1000.0), (102.0, 1000.0)];

        let imbalance = OrderBookImbalanceStream::<MockStream>::calculate_imbalance(&bids, &asks);
        assert_eq!(imbalance, 0.0);
    }

    #[test]
    fn test_order_book_imbalance_calculate_imbalance_bid_heavy_should_return_positive() {
        let bids = vec![(100.0, 2000.0), (99.0, 1000.0)];
        let asks = vec![(101.0, 500.0), (102.0, 500.0)];

        let imbalance = OrderBookImbalanceStream::<MockStream>::calculate_imbalance(&bids, &asks);
        // Bid volume = 3000, Ask volume = 1000
        // Imbalance = (3000 - 1000) / (3000 + 1000) = 2000 / 4000 = 0.5
        assert_eq!(imbalance, 0.5);
    }

    #[test]
    fn test_order_book_imbalance_calculate_imbalance_ask_heavy_should_return_negative() {
        let bids = vec![(100.0, 500.0), (99.0, 500.0)];
        let asks = vec![(101.0, 2000.0), (102.0, 1000.0)];

        let imbalance = OrderBookImbalanceStream::<MockStream>::calculate_imbalance(&bids, &asks);
        // Bid volume = 1000, Ask volume = 3000
        // Imbalance = (1000 - 3000) / (1000 + 3000) = -2000 / 4000 = -0.5
        assert_eq!(imbalance, -0.5);
    }

    #[test]
    fn test_order_book_imbalance_calculate_imbalance_empty_books_should_return_zero() {
        let bids = vec![];
        let asks = vec![];

        let imbalance = OrderBookImbalanceStream::<MockStream>::calculate_imbalance(&bids, &asks);
        assert_eq!(imbalance, 0.0);
    }

    #[test]
    fn test_order_book_imbalance_calculate_imbalance_should_limit_to_five_levels() {
        let bids = vec![(100.0, 100.0); 10]; // 10 levels
        let asks = vec![(101.0, 100.0); 10]; // 10 levels

        let imbalance = OrderBookImbalanceStream::<MockStream>::calculate_imbalance(&bids, &asks);
        // Should only consider first 5 levels: 5*100 vs 5*100
        assert_eq!(imbalance, 0.0);
    }

    // Tests for MomentumStream
    #[test]
    fn test_momentum_stream_new_should_create_correctly() {
        let mock_stream = MockStream::new("test");
        let lookback_period = 10;
        let momentum_stream = MomentumStream::new(mock_stream, lookback_period);

        assert_eq!(momentum_stream.lookback_period, lookback_period);
    }

    #[tokio::test]
    async fn test_momentum_stream_calculate_momentum_insufficient_data_should_return_none() {
        let mock_stream = MockStream::new("test");
        let momentum_stream = MomentumStream::new(mock_stream, 2);

        let momentum = momentum_stream.calculate_momentum(100.0).await;
        assert_eq!(momentum, None);

        let momentum = momentum_stream.calculate_momentum(110.0).await;
        assert_eq!(momentum, None);
    }

    #[tokio::test]
    async fn test_momentum_stream_calculate_momentum_with_sufficient_data_should_return_percentage()
    {
        let mock_stream = MockStream::new("test");
        let momentum_stream = MomentumStream::new(mock_stream, 2);

        momentum_stream.calculate_momentum(100.0).await;
        momentum_stream.calculate_momentum(105.0).await;
        let momentum = momentum_stream.calculate_momentum(110.0).await;

        // Momentum = ((110 - 100) / 100) * 100 = 10%
        assert_eq!(momentum, Some(10.0));
    }

    #[tokio::test]
    async fn test_momentum_stream_calculate_momentum_negative_change_should_return_negative() {
        let mock_stream = MockStream::new("test");
        let momentum_stream = MomentumStream::new(mock_stream, 2);

        momentum_stream.calculate_momentum(100.0).await;
        momentum_stream.calculate_momentum(105.0).await;
        let momentum = momentum_stream.calculate_momentum(90.0).await;

        // Momentum = ((90 - 100) / 100) * 100 = -10%
        assert_eq!(momentum, Some(-10.0));
    }

    #[tokio::test]
    async fn test_momentum_stream_should_maintain_lookback_window() {
        let mock_stream = MockStream::new("test");
        let momentum_stream = MomentumStream::new(mock_stream, 2);

        momentum_stream.calculate_momentum(100.0).await;
        momentum_stream.calculate_momentum(105.0).await;
        momentum_stream.calculate_momentum(110.0).await;
        let momentum = momentum_stream.calculate_momentum(120.0).await;

        // Should compare 120 with 105 (dropped 100)
        // Momentum = ((120 - 105) / 105) * 100 ≈ 14.29%
        let expected = ((120.0 - 105.0) / 105.0) * 100.0;
        assert_eq!(momentum, Some(expected));
    }

    // Tests for LiquidityPoolStream and PoolState
    #[test]
    fn test_liquidity_pool_stream_new_should_create_correctly() {
        let mock_stream = MockStream::new("test");
        let pool_stream = LiquidityPoolStream::new(mock_stream);

        assert!(pool_stream.pools.is_empty());
    }

    #[tokio::test]
    async fn test_liquidity_pool_stream_update_pool_should_create_pool_state() {
        let mock_stream = MockStream::new("test");
        let pool_stream = LiquidityPoolStream::new(mock_stream);

        pool_stream
            .update_pool("pool1".to_string(), 1000.0, 2000.0)
            .await;

        let pool = pool_stream.pools.get("pool1").unwrap();
        assert_eq!(pool.token_a_reserve, 1000.0);
        assert_eq!(pool.token_b_reserve, 2000.0);
        assert_eq!(pool.k_constant, 2_000_000.0);
    }

    #[tokio::test]
    async fn test_liquidity_pool_stream_calculate_price_impact_existing_pool_should_return_impact()
    {
        let mock_stream = MockStream::new("test");
        let pool_stream = LiquidityPoolStream::new(mock_stream);

        pool_stream
            .update_pool("pool1".to_string(), 1000.0, 2000.0)
            .await;
        let impact = pool_stream.calculate_price_impact("pool1", 100.0).await;

        // k = 1000 * 2000 = 2,000,000
        // new_reserve_a = 1000 + 100 = 1100
        // new_reserve_b = 2,000,000 / 1100 ≈ 1818.18
        // amount_out = 2000 - 1818.18 = 181.82
        // price_impact = (181.82 / 2000) * 100 = 9.091%
        assert!(impact.is_some());
        let impact_value = impact.unwrap();
        assert!((impact_value - 9.090909090909092).abs() < 0.0001);
    }

    #[tokio::test]
    async fn test_liquidity_pool_stream_calculate_price_impact_nonexistent_pool_should_return_none()
    {
        let mock_stream = MockStream::new("test");
        let pool_stream = LiquidityPoolStream::new(mock_stream);

        let impact = pool_stream
            .calculate_price_impact("nonexistent", 100.0)
            .await;
        assert_eq!(impact, None);
    }

    // Tests for MevDetectionStream and related types
    #[test]
    fn test_mev_detection_stream_new_should_create_correctly() {
        let mock_stream = MockStream::new("test");
        let window = Duration::from_secs(60);
        let mev_stream = MevDetectionStream::new(mock_stream, window);

        assert_eq!(mev_stream.window, window);
    }

    #[tokio::test]
    async fn test_mev_detection_stream_detect_mev_non_swap_event_should_return_none() {
        let mock_stream = MockStream::new("test");
        let mev_stream = MevDetectionStream::new(mock_stream, Duration::from_secs(60));

        let event = MockEvent {
            id: "test".to_string(),
            kind: EventKind::Price, // Not a swap
            price: Some(100.0),
            volume: Some(1000.0),
            market_cap: None,
            timestamp_ms: None,
            metadata: EventMetadata::new("test".to_string(), EventKind::Price, "test".to_string()),
        };

        let mev_type = mev_stream.detect_mev(&event).await;
        assert_eq!(mev_type, None);
    }

    #[tokio::test]
    async fn test_mev_detection_stream_detect_mev_swap_insufficient_patterns_should_return_none() {
        let mock_stream = MockStream::new("test");
        let mev_stream = MevDetectionStream::new(mock_stream, Duration::from_secs(60));

        let event = MockEvent {
            id: "test".to_string(),
            kind: EventKind::Swap,
            price: Some(100.0),
            volume: Some(1000.0),
            market_cap: None,
            timestamp_ms: None,
            metadata: EventMetadata::new("test".to_string(), EventKind::Swap, "test".to_string()),
        };

        let mev_type = mev_stream.detect_mev(&event).await;
        assert_eq!(mev_type, None);
    }

    #[tokio::test]
    async fn test_mev_detection_stream_detect_mev_should_clean_old_transactions() {
        let mock_stream = MockStream::new("test");
        let mev_stream = MevDetectionStream::new(mock_stream, Duration::from_millis(100));

        // Add old transaction
        {
            let mut txs = mev_stream.transactions.write().await;
            txs.push_back(TransactionPattern {
                tx_hash: "old".to_string(),
                timestamp: SystemTime::now() - Duration::from_secs(1),
                pattern_type: MevType::Sandwich,
            });
        }

        let event = MockEvent {
            id: "test".to_string(),
            kind: EventKind::Swap,
            price: Some(100.0),
            volume: Some(1000.0),
            market_cap: None,
            timestamp_ms: None,
            metadata: EventMetadata::new("test".to_string(), EventKind::Swap, "test".to_string()),
        };

        mev_stream.detect_mev(&event).await;

        // Old transaction should be removed
        let txs = mev_stream.transactions.read().await;
        assert!(txs.is_empty());
    }

    // Tests for MevType enum (Debug trait coverage)
    #[test]
    fn test_mev_type_debug_should_format_correctly() {
        assert_eq!(format!("{:?}", MevType::Sandwich), "Sandwich");
        assert_eq!(format!("{:?}", MevType::Frontrun), "Frontrun");
        assert_eq!(format!("{:?}", MevType::Backrun), "Backrun");
        assert_eq!(format!("{:?}", MevType::Arbitrage), "Arbitrage");
    }

    #[test]
    fn test_mev_type_clone_should_create_copy() {
        let original = MevType::Sandwich;
        let cloned = original.clone();
        matches!(cloned, MevType::Sandwich);
    }

    // Tests for GasPriceOracleStream and GasPriceEstimate
    #[test]
    fn test_gas_price_oracle_stream_new_should_create_correctly() {
        let mock_stream = MockStream::new("test");
        let window_size = 100;
        let gas_stream = GasPriceOracleStream::new(mock_stream, window_size);

        assert_eq!(gas_stream.window_size, window_size);
        assert_eq!(gas_stream.percentiles, vec![25, 50, 75, 95]);
    }

    #[tokio::test]
    async fn test_gas_price_oracle_stream_update_gas_price_single_value() {
        let mock_stream = MockStream::new("test");
        let gas_stream = GasPriceOracleStream::new(mock_stream, 100);

        let estimate = gas_stream.update_gas_price(100.0).await;

        // With single value, all percentiles should be the same
        assert_eq!(estimate.slow, 100.0);
        assert_eq!(estimate.standard, 100.0);
        assert_eq!(estimate.fast, 100.0);
        assert_eq!(estimate.instant, 100.0);
    }

    #[tokio::test]
    async fn test_gas_price_oracle_stream_update_gas_price_multiple_values() {
        let mock_stream = MockStream::new("test");
        let gas_stream = GasPriceOracleStream::new(mock_stream, 100);

        // Add values: 10, 20, 30, 40, 50, 60, 70, 80, 90, 100
        for i in 1..=10 {
            gas_stream.update_gas_price(i as f64 * 10.0).await;
        }

        let estimate = gas_stream.update_gas_price(100.0).await;

        // Sorted: [10, 20, 30, 40, 50, 60, 70, 80, 90, 100]
        // 25th percentile (index 2): 30
        // 50th percentile (index 5): 60
        // 75th percentile (index 7): 80
        // 95th percentile (index 9): 100
        assert_eq!(estimate.slow, 30.0);
        assert_eq!(estimate.standard, 60.0);
        assert_eq!(estimate.fast, 80.0);
        assert_eq!(estimate.instant, 100.0);
    }

    #[tokio::test]
    async fn test_gas_price_oracle_stream_should_maintain_window_size() {
        let mock_stream = MockStream::new("test");
        let gas_stream = GasPriceOracleStream::new(mock_stream, 2);

        gas_stream.update_gas_price(10.0).await;
        gas_stream.update_gas_price(20.0).await;
        let estimate = gas_stream.update_gas_price(30.0).await; // Should drop 10.0

        // Should only have [20.0, 30.0]
        // All percentiles will be either 20 or 30
        assert!(estimate.slow >= 20.0 && estimate.slow <= 30.0);
        assert!(estimate.instant >= 20.0 && estimate.instant <= 30.0);
    }

    // Tests for GasPriceEstimate (Debug and Clone traits)
    #[test]
    fn test_gas_price_estimate_debug_should_format_correctly() {
        let estimate = GasPriceEstimate {
            slow: 10.0,
            standard: 20.0,
            fast: 30.0,
            instant: 40.0,
        };

        let debug_str = format!("{:?}", estimate);
        assert!(debug_str.contains("slow: 10.0"));
        assert!(debug_str.contains("standard: 20.0"));
        assert!(debug_str.contains("fast: 30.0"));
        assert!(debug_str.contains("instant: 40.0"));
    }

    #[test]
    fn test_gas_price_estimate_clone_should_create_copy() {
        let original = GasPriceEstimate {
            slow: 10.0,
            standard: 20.0,
            fast: 30.0,
            instant: 40.0,
        };

        let cloned = original.clone();
        assert_eq!(cloned.slow, 10.0);
        assert_eq!(cloned.standard, 20.0);
        assert_eq!(cloned.fast, 30.0);
        assert_eq!(cloned.instant, 40.0);
    }

    // Tests for PoolState (Debug and Clone traits)
    #[test]
    fn test_pool_state_debug_should_format_correctly() {
        let pool_state = PoolState {
            token_a_reserve: 1000.0,
            token_b_reserve: 2000.0,
            k_constant: 2_000_000.0,
            last_updated: SystemTime::UNIX_EPOCH,
        };

        let debug_str = format!("{:?}", pool_state);
        assert!(debug_str.contains("token_a_reserve: 1000.0"));
        assert!(debug_str.contains("token_b_reserve: 2000.0"));
        assert!(debug_str.contains("k_constant: 2000000.0"));
    }

    #[test]
    fn test_pool_state_clone_should_create_copy() {
        let original = PoolState {
            token_a_reserve: 1000.0,
            token_b_reserve: 2000.0,
            k_constant: 2_000_000.0,
            last_updated: SystemTime::UNIX_EPOCH,
        };

        let cloned = original.clone();
        assert_eq!(cloned.token_a_reserve, 1000.0);
        assert_eq!(cloned.token_b_reserve, 2000.0);
        assert_eq!(cloned.k_constant, 2_000_000.0);
        assert_eq!(cloned.last_updated, SystemTime::UNIX_EPOCH);
    }

    // Tests for BollingerBands (Debug and Clone traits)
    #[test]
    fn test_bollinger_bands_debug_should_format_correctly() {
        let bands = BollingerBands {
            upper: 110.0,
            middle: 100.0,
            lower: 90.0,
            current_value: 105.0,
        };

        let debug_str = format!("{:?}", bands);
        assert!(debug_str.contains("upper: 110.0"));
        assert!(debug_str.contains("middle: 100.0"));
        assert!(debug_str.contains("lower: 90.0"));
        assert!(debug_str.contains("current_value: 105.0"));
    }

    #[test]
    fn test_bollinger_bands_clone_should_create_copy() {
        let original = BollingerBands {
            upper: 110.0,
            middle: 100.0,
            lower: 90.0,
            current_value: 105.0,
        };

        let cloned = original.clone();
        assert_eq!(cloned.upper, 110.0);
        assert_eq!(cloned.middle, 100.0);
        assert_eq!(cloned.lower, 90.0);
        assert_eq!(cloned.current_value, 105.0);
    }

    // Tests for FinancialStreamExt trait methods
    #[test]
    fn test_financial_stream_ext_vwap_should_create_vwap_stream() {
        let mock_stream = MockStream::new("test");
        let window = Duration::from_secs(60);
        let vwap_stream = mock_stream.vwap(window);

        assert_eq!(vwap_stream.name(), "vwap(test)");
    }

    #[test]
    fn test_financial_stream_ext_moving_average_should_create_ma_stream() {
        let mock_stream = MockStream::new("test");
        let ma_stream = mock_stream.moving_average(10);

        assert_eq!(ma_stream.window_size, 10);
    }

    #[test]
    fn test_financial_stream_ext_ema_should_create_ema_stream() {
        let mock_stream = MockStream::new("test");
        let ema_stream = mock_stream.ema(14);

        let expected_alpha = 2.0 / 15.0;
        assert_eq!(ema_stream.alpha, expected_alpha);
    }

    #[test]
    fn test_financial_stream_ext_bollinger_bands_should_create_bb_stream() {
        let mock_stream = MockStream::new("test");
        let bb_stream = mock_stream.bollinger_bands(20, 2.0);

        assert_eq!(bb_stream.window, 20);
        assert_eq!(bb_stream.std_dev_multiplier, 2.0);
    }

    #[test]
    fn test_financial_stream_ext_rsi_should_create_rsi_stream() {
        let mock_stream = MockStream::new("test");
        let rsi_stream = mock_stream.rsi(14);

        assert_eq!(rsi_stream.period, 14);
    }

    #[test]
    fn test_financial_stream_ext_momentum_should_create_momentum_stream() {
        let mock_stream = MockStream::new("test");
        let momentum_stream = mock_stream.momentum(10);

        assert_eq!(momentum_stream.lookback_period, 10);
    }

    #[test]
    fn test_financial_stream_ext_liquidity_pools_should_create_pool_stream() {
        let mock_stream = MockStream::new("test");
        let pool_stream = mock_stream.liquidity_pools();

        assert!(pool_stream.pools.is_empty());
    }

    #[test]
    fn test_financial_stream_ext_mev_detection_should_create_mev_stream() {
        let mock_stream = MockStream::new("test");
        let window = Duration::from_secs(30);
        let mev_stream = mock_stream.mev_detection(window);

        assert_eq!(mev_stream.window, window);
    }

    #[test]
    fn test_financial_stream_ext_gas_oracle_should_create_gas_stream() {
        let mock_stream = MockStream::new("test");
        let gas_stream = mock_stream.gas_oracle(100);

        assert_eq!(gas_stream.window_size, 100);
    }
}
