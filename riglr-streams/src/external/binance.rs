use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::{broadcast, RwLock};
use tokio_tungstenite::connect_async;
use tracing::info;

use crate::core::StreamMetadata;
use crate::core::{Stream, StreamError, StreamEvent, StreamHealth};
use chrono::Utc;
use riglr_events_core::prelude::{Event, EventKind, EventMetadata};

/// Binance WebSocket stream implementation
pub struct BinanceStream {
    /// Stream configuration
    config: BinanceConfig,
    /// Event broadcast channel
    event_tx: broadcast::Sender<Arc<BinanceStreamEvent>>,
    /// Running state
    running: Arc<AtomicBool>,
    /// Health metrics
    health: Arc<RwLock<StreamHealth>>,
    /// Stream name
    name: String,
}

/// Binance stream configuration
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct BinanceConfig {
    /// Streams to subscribe to (e.g., ["btcusdt@ticker", "ethusdt@depth"])
    pub streams: Vec<String>,
    /// Use testnet
    pub testnet: bool,
    /// Buffer size for event channel
    pub buffer_size: usize,
}

impl Default for BinanceConfig {
    fn default() -> Self {
        Self {
            streams: Vec::new(),
            testnet: false,
            buffer_size: 10000,
        }
    }
}

/// Binance streaming event
#[derive(Debug, Clone)]
pub struct BinanceStreamEvent {
    /// Event metadata for riglr-events-core compatibility
    pub metadata: EventMetadata,
    /// Event data
    pub data: BinanceEventData,
    /// Stream metadata (legacy)
    pub stream_meta: StreamMetadata,
}

/// Binance event data types
#[derive(Debug, Clone, Serialize)]
pub enum BinanceEventData {
    /// 24hr ticker statistics
    Ticker(TickerData),
    /// Order book update
    OrderBook(OrderBookData),
    /// Individual trade
    Trade(TradeData),
    /// K-line/candlestick
    Kline(KlineData),
    /// Unknown event type
    Unknown(serde_json::Value),
}

/// 24-hour ticker price change statistics data
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TickerData {
    /// Trading pair symbol (e.g., "BTCUSDT")
    #[serde(rename = "s")]
    pub symbol: String,
    /// Last price
    #[serde(rename = "c")]
    pub close_price: String,
    /// Total traded base asset volume
    #[serde(rename = "v")]
    pub volume: String,
    /// Price change percent
    #[serde(rename = "P")]
    pub price_change_percent: String,
    /// Event time timestamp
    #[serde(rename = "E")]
    pub event_time: u64,
}

/// Order book depth update data
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OrderBookData {
    /// Trading pair symbol (e.g., "BTCUSDT")
    #[serde(rename = "s")]
    pub symbol: String,
    /// Bids to be updated (price, quantity pairs)
    #[serde(rename = "b")]
    pub bids: Vec<Vec<String>>,
    /// Asks to be updated (price, quantity pairs)
    #[serde(rename = "a")]
    pub asks: Vec<Vec<String>>,
    /// Event time timestamp
    #[serde(rename = "E")]
    pub event_time: u64,
}

/// Individual trade execution data
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TradeData {
    /// Trading pair symbol (e.g., "BTCUSDT")
    #[serde(rename = "s")]
    pub symbol: String,
    /// Trade price
    #[serde(rename = "p")]
    pub price: String,
    /// Trade quantity
    #[serde(rename = "q")]
    pub quantity: String,
    /// Trade time timestamp
    #[serde(rename = "T")]
    pub trade_time: u64,
    /// Whether the buyer is the market maker
    #[serde(rename = "m")]
    pub is_buyer_maker: bool,
}

/// Kline/candlestick stream data
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct KlineData {
    /// Trading pair symbol (e.g., "BTCUSDT")
    #[serde(rename = "s")]
    pub symbol: String,
    /// Kline details
    #[serde(rename = "k")]
    pub kline: KlineDetails,
}

/// Detailed kline/candlestick data
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct KlineDetails {
    /// Kline start time timestamp
    #[serde(rename = "t")]
    pub open_time: u64,
    /// Open price
    #[serde(rename = "o")]
    pub open: String,
    /// High price
    #[serde(rename = "h")]
    pub high: String,
    /// Low price
    #[serde(rename = "l")]
    pub low: String,
    /// Close price
    #[serde(rename = "c")]
    pub close: String,
    /// Volume
    #[serde(rename = "v")]
    pub volume: String,
}

impl StreamEvent for BinanceStreamEvent {
    fn stream_metadata(&self) -> Option<&StreamMetadata> {
        Some(&self.stream_meta)
    }
}

impl Event for BinanceStreamEvent {
    fn id(&self) -> &str {
        &self.metadata.id
    }

    fn kind(&self) -> &EventKind {
        &self.metadata.kind
    }

    fn metadata(&self) -> &EventMetadata {
        &self.metadata
    }

    fn metadata_mut(&mut self) -> &mut EventMetadata {
        &mut self.metadata
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn clone_boxed(&self) -> Box<dyn Event> {
        Box::new(self.clone())
    }

    fn to_json(&self) -> riglr_events_core::EventResult<serde_json::Value> {
        Ok(serde_json::json!({
            "metadata": self.metadata,
            "data": self.data,
            "stream_meta": self.stream_meta
        }))
    }
}

#[async_trait::async_trait]
impl Stream for BinanceStream {
    type Event = BinanceStreamEvent;
    type Config = BinanceConfig;

    async fn start(&mut self, config: Self::Config) -> Result<(), StreamError> {
        if self.running.load(Ordering::Relaxed) {
            return Err(StreamError::AlreadyRunning {
                name: self.name.clone(),
            });
        }

        info!("Starting Binance WebSocket stream");

        self.config = config;
        self.running.store(true, Ordering::Relaxed);

        // Update health status
        {
            let mut health = self.health.write().await;
            health.is_connected = true;
            health.last_event_time = Some(SystemTime::now());
        }

        // Start WebSocket connection
        self.start_websocket().await?;

        Ok(())
    }

    async fn stop(&mut self) -> Result<(), StreamError> {
        if !self.running.load(Ordering::Relaxed) {
            return Ok(());
        }

        info!("Stopping Binance WebSocket stream");
        self.running.store(false, Ordering::Relaxed);

        // Update health status
        {
            let mut health = self.health.write().await;
            health.is_connected = false;
        }

        Ok(())
    }

    fn subscribe(&self) -> broadcast::Receiver<Arc<Self::Event>> {
        self.event_tx.subscribe()
    }

    fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }

    async fn health(&self) -> StreamHealth {
        let health = self.health.read().await;
        health.clone()
    }

    fn name(&self) -> &str {
        &self.name
    }
}

impl BinanceStream {
    /// Create a new Binance stream
    pub fn new(name: impl Into<String>) -> Self {
        let (event_tx, _) = broadcast::channel(10000);

        Self {
            config: BinanceConfig::default(),
            event_tx,
            running: Arc::new(AtomicBool::new(false)),
            health: Arc::new(RwLock::new(StreamHealth::default())),
            name: name.into(),
        }
    }

    /// Start the WebSocket connection with resilience
    async fn start_websocket(&self) -> Result<(), StreamError> {
        let base_url = if self.config.testnet {
            "wss://testnet.binance.vision/ws"
        } else {
            "wss://stream.binance.com:9443/ws"
        };

        let streams_param = self.config.streams.join("/");
        let url = if streams_param.is_empty() {
            base_url.to_string()
        } else {
            format!("{}/{}", base_url, streams_param)
        };

        let event_tx = self.event_tx.clone();
        let running = self.running.clone();
        let health = self.health.clone();
        let stream_name = self.name.clone();

        tokio::spawn(async move {
            crate::impl_resilient_websocket!(
                stream_name,
                running,
                health,
                event_tx,
                Option::<crate::core::MetricsCollector>::None, // No metrics for now
                // Connect function
                || {
                    let url = url.clone();
                    async move {
                        connect_async(&url)
                            .await
                            .map(|(ws, _)| ws)
                            .map_err(|e| format!("Failed to connect to Binance: {}", e))
                    }
                },
                // Subscribe function
                move |_write: &mut futures::stream::SplitSink<_, Message>| {
                    std::future::ready(Ok::<(), String>(()))
                },
                // Parse function
                |text: String, sequence_number: u64| {
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                        Self::parse_message(json, sequence_number)
                    } else {
                        None
                    }
                }
            );
        });

        Ok(())
    }

    /// Parse a Binance message into an event
    fn parse_message(json: serde_json::Value, sequence_number: u64) -> Option<BinanceStreamEvent> {
        // Check event type
        let event_type = json.get("e")?.as_str()?;

        let data = match event_type {
            "24hrTicker" => {
                let ticker: TickerData = serde_json::from_value(json.clone()).ok()?;
                BinanceEventData::Ticker(ticker)
            }
            "depthUpdate" => {
                let order_book: OrderBookData = serde_json::from_value(json.clone()).ok()?;
                BinanceEventData::OrderBook(order_book)
            }
            "trade" => {
                let trade: TradeData = serde_json::from_value(json.clone()).ok()?;
                BinanceEventData::Trade(trade)
            }
            "kline" => {
                let kline: KlineData = serde_json::from_value(json.clone()).ok()?;
                BinanceEventData::Kline(kline)
            }
            _ => BinanceEventData::Unknown(json.clone()),
        };

        // Extract symbol for ID
        let symbol = match &data {
            BinanceEventData::Ticker(t) => &t.symbol,
            BinanceEventData::OrderBook(o) => &o.symbol,
            BinanceEventData::Trade(t) => &t.symbol,
            BinanceEventData::Kline(k) => &k.symbol,
            BinanceEventData::Unknown(_) => "unknown",
        };

        // Create unique ID
        let id = format!("{}-{}-{}", symbol, event_type, sequence_number);

        // Determine event kind
        let kind = match event_type {
            "24hrTicker" | "kline" => EventKind::Price,
            "depthUpdate" => EventKind::External,
            "trade" => EventKind::Transaction,
            _ => EventKind::External,
        };

        // Create event metadata
        let now = Utc::now();
        let metadata = EventMetadata {
            id: id.clone(),
            kind,
            timestamp: now,
            received_at: now,
            source: "binance-ws".to_string(),
            chain_data: None,
            custom: std::collections::HashMap::new(),
        };

        // Create stream metadata (legacy)
        let stream_meta = StreamMetadata {
            stream_source: "binance".to_string(),
            received_at: SystemTime::now(),
            sequence_number: Some(sequence_number),
            custom_data: Some(json.clone()),
        };

        Some(BinanceStreamEvent {
            metadata,
            data,
            stream_meta,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use riglr_events_core::prelude::EventKind;
    use serde_json::json;
    use std::collections::HashMap;

    #[test]
    fn test_binance_config_default() {
        let config = BinanceConfig::default();
        assert_eq!(config.streams, Vec::<String>::new());
        assert!(!config.testnet);
        assert_eq!(config.buffer_size, 10000);
    }

    #[test]
    fn test_binance_config_custom() {
        let config = BinanceConfig {
            streams: vec!["btcusdt@ticker".to_string(), "ethusdt@depth".to_string()],
            testnet: true,
            buffer_size: 5000,
        };
        assert_eq!(config.streams.len(), 2);
        assert_eq!(config.streams[0], "btcusdt@ticker");
        assert_eq!(config.streams[1], "ethusdt@depth");
        assert!(config.testnet);
        assert_eq!(config.buffer_size, 5000);
    }

    #[test]
    fn test_binance_stream_new() {
        let stream = BinanceStream::new("test-stream");
        assert_eq!(stream.name(), "test-stream");
        assert!(!stream.is_running());
        assert_eq!(stream.config.streams.len(), 0);
        assert!(!stream.config.testnet);
        assert_eq!(stream.config.buffer_size, 10000);
    }

    #[test]
    fn test_binance_stream_new_with_string() {
        let stream = BinanceStream::new("binance-stream".to_string());
        assert_eq!(stream.name(), "binance-stream");
    }

    #[tokio::test]
    async fn test_binance_stream_start_when_not_running_should_succeed() {
        let mut stream = BinanceStream::new("test");
        let config = BinanceConfig {
            streams: vec!["btcusdt@ticker".to_string()],
            testnet: true,
            buffer_size: 1000,
        };

        let result = stream.start(config.clone()).await;
        assert!(result.is_ok());
        assert!(stream.is_running());
        assert_eq!(stream.config.streams, config.streams);
        assert_eq!(stream.config.testnet, config.testnet);
        assert_eq!(stream.config.buffer_size, config.buffer_size);

        // Cleanup
        let _ = stream.stop().await;
    }

    #[tokio::test]
    async fn test_binance_stream_start_when_already_running_should_error() {
        let mut stream = BinanceStream::new("test");
        let config = BinanceConfig::default();

        // Start once
        stream.running.store(true, Ordering::Relaxed);

        // Try to start again
        let result = stream.start(config).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            StreamError::AlreadyRunning { name } => {
                assert_eq!(name, "test");
            }
            _ => panic!("Expected AlreadyRunning error"),
        }
    }

    #[tokio::test]
    async fn test_binance_stream_stop_when_running_should_succeed() {
        let mut stream = BinanceStream::new("test");
        stream.running.store(true, Ordering::Relaxed);

        let result = stream.stop().await;
        assert!(result.is_ok());
        assert!(!stream.is_running());
    }

    #[tokio::test]
    async fn test_binance_stream_stop_when_not_running_should_succeed() {
        let mut stream = BinanceStream::new("test");
        assert!(!stream.is_running());

        let result = stream.stop().await;
        assert!(result.is_ok());
        assert!(!stream.is_running());
    }

    #[test]
    fn test_binance_stream_subscribe() {
        let stream = BinanceStream::new("test");
        let receiver = stream.subscribe();
        assert!(receiver.is_empty());
    }

    #[tokio::test]
    async fn test_binance_stream_health() {
        let stream = BinanceStream::new("test");
        let health = stream.health().await;
        assert!(!health.is_connected);
        assert_eq!(health.error_count, 0);
        assert!(health.last_event_time.is_none());
    }

    #[test]
    fn test_binance_stream_name() {
        let stream = BinanceStream::new("my-stream");
        assert_eq!(stream.name(), "my-stream");
    }

    #[test]
    fn test_ticker_data_serialization() {
        let ticker = TickerData {
            symbol: "BTCUSDT".to_string(),
            close_price: "50000.00".to_string(),
            volume: "1000.5".to_string(),
            price_change_percent: "2.5".to_string(),
            event_time: 1234567890,
        };

        let serialized = serde_json::to_string(&ticker).unwrap();
        assert!(serialized.contains("BTCUSDT"));
        assert!(serialized.contains("50000.00"));

        let deserialized: TickerData = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.symbol, ticker.symbol);
        assert_eq!(deserialized.close_price, ticker.close_price);
        assert_eq!(deserialized.volume, ticker.volume);
        assert_eq!(
            deserialized.price_change_percent,
            ticker.price_change_percent
        );
        assert_eq!(deserialized.event_time, ticker.event_time);
    }

    #[test]
    fn test_order_book_data_serialization() {
        let order_book = OrderBookData {
            symbol: "ETHUSDT".to_string(),
            bids: vec![vec!["3000.0".to_string(), "10.0".to_string()]],
            asks: vec![vec!["3001.0".to_string(), "5.0".to_string()]],
            event_time: 1234567890,
        };

        let serialized = serde_json::to_string(&order_book).unwrap();
        let deserialized: OrderBookData = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.symbol, order_book.symbol);
        assert_eq!(deserialized.bids, order_book.bids);
        assert_eq!(deserialized.asks, order_book.asks);
        assert_eq!(deserialized.event_time, order_book.event_time);
    }

    #[test]
    fn test_trade_data_serialization() {
        let trade = TradeData {
            symbol: "ADAUSDT".to_string(),
            price: "1.25".to_string(),
            quantity: "100.0".to_string(),
            trade_time: 1234567890,
            is_buyer_maker: true,
        };

        let serialized = serde_json::to_string(&trade).unwrap();
        let deserialized: TradeData = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.symbol, trade.symbol);
        assert_eq!(deserialized.price, trade.price);
        assert_eq!(deserialized.quantity, trade.quantity);
        assert_eq!(deserialized.trade_time, trade.trade_time);
        assert_eq!(deserialized.is_buyer_maker, trade.is_buyer_maker);
    }

    #[test]
    fn test_kline_data_serialization() {
        let kline = KlineData {
            symbol: "DOTUSDT".to_string(),
            kline: KlineDetails {
                open_time: 1234567890,
                open: "25.0".to_string(),
                high: "26.0".to_string(),
                low: "24.5".to_string(),
                close: "25.5".to_string(),
                volume: "1000.0".to_string(),
            },
        };

        let serialized = serde_json::to_string(&kline).unwrap();
        let deserialized: KlineData = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.symbol, kline.symbol);
        assert_eq!(deserialized.kline.open_time, kline.kline.open_time);
        assert_eq!(deserialized.kline.open, kline.kline.open);
        assert_eq!(deserialized.kline.high, kline.kline.high);
        assert_eq!(deserialized.kline.low, kline.kline.low);
        assert_eq!(deserialized.kline.close, kline.kline.close);
        assert_eq!(deserialized.kline.volume, kline.kline.volume);
    }

    #[test]
    fn test_binance_event_data_ticker_serialization() {
        let ticker = TickerData {
            symbol: "BTCUSDT".to_string(),
            close_price: "50000.00".to_string(),
            volume: "1000.5".to_string(),
            price_change_percent: "2.5".to_string(),
            event_time: 1234567890,
        };
        let event_data = BinanceEventData::Ticker(ticker);

        let serialized = serde_json::to_value(&event_data).unwrap();
        assert!(serialized.get("Ticker").is_some());
    }

    #[test]
    fn test_binance_event_data_unknown_serialization() {
        let unknown_json = json!({"unknown_field": "value"});
        let event_data = BinanceEventData::Unknown(unknown_json.clone());

        let serialized = serde_json::to_value(&event_data).unwrap();
        assert!(serialized.get("Unknown").is_some());
    }

    #[test]
    fn test_parse_message_ticker_event() {
        let json = json!({
            "e": "24hrTicker",
            "s": "BTCUSDT",
            "c": "50000.00",
            "v": "1000.5",
            "P": "2.5",
            "E": 1234567890
        });

        let event = BinanceStream::parse_message(json, 1).unwrap();
        assert_eq!(event.metadata.id, "BTCUSDT-24hrTicker-1");
        assert_eq!(event.metadata.kind, EventKind::Price);
        assert_eq!(event.metadata.source, "binance-ws");

        match event.data {
            BinanceEventData::Ticker(ticker) => {
                assert_eq!(ticker.symbol, "BTCUSDT");
                assert_eq!(ticker.close_price, "50000.00");
                assert_eq!(ticker.volume, "1000.5");
                assert_eq!(ticker.price_change_percent, "2.5");
                assert_eq!(ticker.event_time, 1234567890);
            }
            _ => panic!("Expected Ticker event"),
        }

        assert_eq!(event.stream_meta.stream_source, "binance");
        assert_eq!(event.stream_meta.sequence_number, Some(1));
    }

    #[test]
    fn test_parse_message_depth_update_event() {
        let json = json!({
            "e": "depthUpdate",
            "s": "ETHUSDT",
            "b": [["3000.0", "10.0"]],
            "a": [["3001.0", "5.0"]],
            "E": 1234567890
        });

        let event = BinanceStream::parse_message(json, 2).unwrap();
        assert_eq!(event.metadata.id, "ETHUSDT-depthUpdate-2");
        assert_eq!(event.metadata.kind, EventKind::External);

        match event.data {
            BinanceEventData::OrderBook(order_book) => {
                assert_eq!(order_book.symbol, "ETHUSDT");
                assert_eq!(order_book.bids.len(), 1);
                assert_eq!(order_book.asks.len(), 1);
                assert_eq!(order_book.event_time, 1234567890);
            }
            _ => panic!("Expected OrderBook event"),
        }
    }

    #[test]
    fn test_parse_message_trade_event() {
        let json = json!({
            "e": "trade",
            "s": "ADAUSDT",
            "p": "1.25",
            "q": "100.0",
            "T": 1234567890,
            "m": true
        });

        let event = BinanceStream::parse_message(json, 3).unwrap();
        assert_eq!(event.metadata.id, "ADAUSDT-trade-3");
        assert_eq!(event.metadata.kind, EventKind::Transaction);

        match event.data {
            BinanceEventData::Trade(trade) => {
                assert_eq!(trade.symbol, "ADAUSDT");
                assert_eq!(trade.price, "1.25");
                assert_eq!(trade.quantity, "100.0");
                assert_eq!(trade.trade_time, 1234567890);
                assert!(trade.is_buyer_maker);
            }
            _ => panic!("Expected Trade event"),
        }
    }

    #[test]
    fn test_parse_message_kline_event() {
        let json = json!({
            "e": "kline",
            "s": "DOTUSDT",
            "k": {
                "t": 1234567890,
                "o": "25.0",
                "h": "26.0",
                "l": "24.5",
                "c": "25.5",
                "v": "1000.0"
            }
        });

        let event = BinanceStream::parse_message(json, 4).unwrap();
        assert_eq!(event.metadata.id, "DOTUSDT-kline-4");
        assert_eq!(event.metadata.kind, EventKind::Price);

        match event.data {
            BinanceEventData::Kline(kline) => {
                assert_eq!(kline.symbol, "DOTUSDT");
                assert_eq!(kline.kline.open_time, 1234567890);
                assert_eq!(kline.kline.open, "25.0");
                assert_eq!(kline.kline.high, "26.0");
                assert_eq!(kline.kline.low, "24.5");
                assert_eq!(kline.kline.close, "25.5");
                assert_eq!(kline.kline.volume, "1000.0");
            }
            _ => panic!("Expected Kline event"),
        }
    }

    #[test]
    fn test_parse_message_unknown_event() {
        let json = json!({
            "e": "unknownEvent",
            "randomField": "value"
        });

        let event = BinanceStream::parse_message(json.clone(), 5).unwrap();
        assert_eq!(event.metadata.id, "unknown-unknownEvent-5");
        assert_eq!(event.metadata.kind, EventKind::External);

        match event.data {
            BinanceEventData::Unknown(data) => {
                assert_eq!(data, json);
            }
            _ => panic!("Expected Unknown event"),
        }
    }

    #[test]
    fn test_parse_message_missing_event_type_should_return_none() {
        let json = json!({
            "s": "BTCUSDT",
            "c": "50000.00"
        });

        let result = BinanceStream::parse_message(json, 1);
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_message_invalid_event_type_should_return_none() {
        let json = json!({
            "e": 123, // Not a string
            "s": "BTCUSDT"
        });

        let result = BinanceStream::parse_message(json, 1);
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_message_invalid_ticker_data_should_return_none() {
        let json = json!({
            "e": "24hrTicker",
            "s": "BTCUSDT"
            // Missing required fields
        });

        let result = BinanceStream::parse_message(json, 1);
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_message_invalid_order_book_data_should_return_none() {
        let json = json!({
            "e": "depthUpdate",
            "s": "ETHUSDT"
            // Missing required fields
        });

        let result = BinanceStream::parse_message(json, 1);
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_message_invalid_trade_data_should_return_none() {
        let json = json!({
            "e": "trade",
            "s": "ADAUSDT"
            // Missing required fields
        });

        let result = BinanceStream::parse_message(json, 1);
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_message_invalid_kline_data_should_return_none() {
        let json = json!({
            "e": "kline",
            "s": "DOTUSDT"
            // Missing required kline field
        });

        let result = BinanceStream::parse_message(json, 1);
        assert!(result.is_none());
    }

    #[test]
    fn test_binance_stream_event_stream_metadata() {
        let event = create_test_event();
        let metadata = event.stream_metadata().unwrap();
        assert_eq!(metadata.stream_source, "binance");
        assert_eq!(metadata.sequence_number, Some(1));
    }

    #[test]
    fn test_binance_stream_event_id() {
        let event = create_test_event();
        assert_eq!(event.id(), "test-id");
    }

    #[test]
    fn test_binance_stream_event_kind() {
        let event = create_test_event();
        assert_eq!(*event.kind(), EventKind::Price);
    }

    #[test]
    fn test_binance_stream_event_metadata() {
        let event = create_test_event();
        let metadata = event.metadata();
        assert_eq!(metadata.id, "test-id");
        assert_eq!(metadata.kind, EventKind::Price);
        assert_eq!(metadata.source, "binance-ws");
    }

    #[test]
    fn test_binance_stream_event_metadata_mut() {
        let mut event = create_test_event();
        let metadata = event.metadata_mut();
        metadata.source = "modified-source".to_string();
        assert_eq!(event.metadata().source, "modified-source");
    }

    #[test]
    fn test_binance_stream_event_as_any() {
        let event = create_test_event();
        let any_ref = event.as_any();
        assert!(any_ref.is::<BinanceStreamEvent>());
    }

    #[test]
    fn test_binance_stream_event_as_any_mut() {
        let mut event = create_test_event();
        let any_mut = event.as_any_mut();
        assert!(any_mut.is::<BinanceStreamEvent>());
    }

    #[test]
    fn test_binance_stream_event_clone_boxed() {
        let event = create_test_event();
        let cloned = event.clone_boxed();
        assert_eq!(cloned.id(), event.id());
        assert_eq!(*cloned.kind(), *event.kind());
    }

    #[test]
    fn test_binance_stream_event_to_json() {
        let event = create_test_event();
        let json = event.to_json().unwrap();

        assert!(json.get("metadata").is_some());
        assert!(json.get("data").is_some());
        assert!(json.get("stream_meta").is_some());

        let metadata = json.get("metadata").unwrap();
        assert_eq!(metadata.get("id").unwrap().as_str().unwrap(), "test-id");
        assert_eq!(
            metadata.get("source").unwrap().as_str().unwrap(),
            "binance-ws"
        );
    }

    fn create_test_event() -> BinanceStreamEvent {
        let metadata = EventMetadata {
            id: "test-id".to_string(),
            kind: EventKind::Price,
            timestamp: Utc::now(),
            received_at: Utc::now(),
            source: "binance-ws".to_string(),
            chain_data: None,
            custom: HashMap::new(),
        };

        let ticker = TickerData {
            symbol: "BTCUSDT".to_string(),
            close_price: "50000.00".to_string(),
            volume: "1000.5".to_string(),
            price_change_percent: "2.5".to_string(),
            event_time: 1234567890,
        };

        let stream_meta = StreamMetadata {
            stream_source: "binance".to_string(),
            received_at: SystemTime::now(),
            sequence_number: Some(1),
            custom_data: Some(json!({"test": "data"})),
        };

        BinanceStreamEvent {
            metadata,
            data: BinanceEventData::Ticker(ticker),
            stream_meta,
        }
    }

    #[test]
    fn test_binance_config_serialization() {
        let config = BinanceConfig {
            streams: vec!["btcusdt@ticker".to_string()],
            testnet: true,
            buffer_size: 5000,
        };

        let serialized = serde_json::to_string(&config).unwrap();
        let deserialized: BinanceConfig = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.streams, config.streams);
        assert_eq!(deserialized.testnet, config.testnet);
        assert_eq!(deserialized.buffer_size, config.buffer_size);
    }

    #[test]
    fn test_empty_streams_configuration() {
        let config = BinanceConfig {
            streams: vec![],
            testnet: false,
            buffer_size: 1000,
        };

        let stream = BinanceStream::new("test");
        assert_eq!(stream.config.streams.len(), 0);

        // Test the config properties
        assert_eq!(config.streams.len(), 0);
        assert!(!config.testnet);
        assert_eq!(config.buffer_size, 1000);
    }

    #[test]
    fn test_multiple_streams_configuration() {
        let streams = vec![
            "btcusdt@ticker".to_string(),
            "ethusdt@depth".to_string(),
            "adausdt@trade".to_string(),
            "dotusdt@kline_1m".to_string(),
        ];

        let config = BinanceConfig {
            streams: streams.clone(),
            testnet: false,
            buffer_size: 1000,
        };

        assert_eq!(config.streams.len(), 4);
        assert_eq!(config.streams, streams);
    }

    #[test]
    fn test_edge_case_empty_strings_in_data() {
        let ticker = TickerData {
            symbol: "".to_string(),
            close_price: "".to_string(),
            volume: "".to_string(),
            price_change_percent: "".to_string(),
            event_time: 0,
        };

        let serialized = serde_json::to_string(&ticker).unwrap();
        let deserialized: TickerData = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.symbol, "");
        assert_eq!(deserialized.close_price, "");
    }

    #[test]
    fn test_edge_case_empty_vectors_in_order_book() {
        let order_book = OrderBookData {
            symbol: "BTCUSDT".to_string(),
            bids: vec![],
            asks: vec![],
            event_time: 1234567890,
        };

        let serialized = serde_json::to_string(&order_book).unwrap();
        let deserialized: OrderBookData = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.bids.len(), 0);
        assert_eq!(deserialized.asks.len(), 0);
    }

    #[test]
    fn test_edge_case_max_values() {
        let ticker = TickerData {
            symbol: "BTCUSDT".to_string(),
            close_price: "999999999.99999999".to_string(),
            volume: "999999999.99999999".to_string(),
            price_change_percent: "999.99".to_string(),
            event_time: u64::MAX,
        };

        let serialized = serde_json::to_string(&ticker).unwrap();
        let deserialized: TickerData = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.event_time, u64::MAX);
    }

    #[test]
    fn test_trade_data_with_false_is_buyer_maker() {
        let trade = TradeData {
            symbol: "BTCUSDT".to_string(),
            price: "50000.00".to_string(),
            quantity: "1.0".to_string(),
            trade_time: 1234567890,
            is_buyer_maker: false,
        };

        let serialized = serde_json::to_string(&trade).unwrap();
        let deserialized: TradeData = serde_json::from_str(&serialized).unwrap();
        assert!(!deserialized.is_buyer_maker);
    }

    #[test]
    fn test_kline_details_with_zero_values() {
        let kline_details = KlineDetails {
            open_time: 0,
            open: "0.0".to_string(),
            high: "0.0".to_string(),
            low: "0.0".to_string(),
            close: "0.0".to_string(),
            volume: "0.0".to_string(),
        };

        let serialized = serde_json::to_string(&kline_details).unwrap();
        let deserialized: KlineDetails = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.open_time, 0);
        assert_eq!(deserialized.open, "0.0");
    }
}
