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
        let std_dev = variance.sqrt();

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
#[derive(Debug, Clone)]
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
