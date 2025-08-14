use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::SystemTime;
use tokio::sync::{broadcast, RwLock};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use serde::{Deserialize, Serialize};
use tracing::{info, warn, error, debug};
use futures::{StreamExt, SinkExt, FutureExt};

use riglr_solana_events::{UnifiedEvent, EventType, ProtocolType};
use crate::core::StreamMetadata;
use crate::core::{Stream, StreamError, StreamEvent, StreamHealth};

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
    /// Event ID
    pub id: String,
    /// Event data
    pub data: BinanceEventData,
    /// Stream metadata
    pub stream_meta: StreamMetadata,
}

/// Binance event data types
#[derive(Debug, Clone)]
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

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TickerData {
    #[serde(rename = "s")]
    pub symbol: String,
    #[serde(rename = "c")]
    pub close_price: String,
    #[serde(rename = "v")]
    pub volume: String,
    #[serde(rename = "P")]
    pub price_change_percent: String,
    #[serde(rename = "E")]
    pub event_time: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OrderBookData {
    #[serde(rename = "s")]
    pub symbol: String,
    #[serde(rename = "b")]
    pub bids: Vec<Vec<String>>,
    #[serde(rename = "a")]
    pub asks: Vec<Vec<String>>,
    #[serde(rename = "E")]
    pub event_time: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TradeData {
    #[serde(rename = "s")]
    pub symbol: String,
    #[serde(rename = "p")]
    pub price: String,
    #[serde(rename = "q")]
    pub quantity: String,
    #[serde(rename = "T")]
    pub trade_time: u64,
    #[serde(rename = "m")]
    pub is_buyer_maker: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct KlineData {
    #[serde(rename = "s")]
    pub symbol: String,
    #[serde(rename = "k")]
    pub kline: KlineDetails,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct KlineDetails {
    #[serde(rename = "t")]
    pub open_time: u64,
    #[serde(rename = "o")]
    pub open: String,
    #[serde(rename = "h")]
    pub high: String,
    #[serde(rename = "l")]
    pub low: String,
    #[serde(rename = "c")]
    pub close: String,
    #[serde(rename = "v")]
    pub volume: String,
}

impl StreamEvent for BinanceStreamEvent {
    fn stream_metadata(&self) -> Option<&StreamMetadata> {
        Some(&self.stream_meta)
    }
}

impl UnifiedEvent for BinanceStreamEvent {
    fn id(&self) -> &str {
        &self.id
    }
    
    fn event_type(&self) -> EventType {
        match &self.data {
            BinanceEventData::Ticker(_) => EventType::PriceUpdate,
            BinanceEventData::OrderBook(_) => EventType::OrderBook,
            BinanceEventData::Trade(_) => EventType::Trade,
            BinanceEventData::Kline(_) => EventType::PriceUpdate,
            BinanceEventData::Unknown(_) => EventType::Unknown,
        }
    }
    
    fn signature(&self) -> &str {
        &self.id
    }
    
    fn slot(&self) -> u64 {
        // Use event time as "slot" for compatibility
        match &self.data {
            BinanceEventData::Ticker(t) => t.event_time,
            BinanceEventData::OrderBook(o) => o.event_time,
            BinanceEventData::Trade(t) => t.trade_time,
            BinanceEventData::Kline(k) => k.kline.open_time,
            BinanceEventData::Unknown(_) => 0,
        }
    }
    
    fn program_received_time_ms(&self) -> i64 {
        self.stream_meta.received_at
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as i64
    }
    
    fn program_handle_time_consuming_ms(&self) -> i64 {
        0
    }
    
    fn set_program_handle_time_consuming_ms(&mut self, _time: i64) {}
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
    
    fn clone_boxed(&self) -> Box<dyn UnifiedEvent> {
        Box::new(self.clone())
    }
    
    fn set_transfer_data(
        &mut self,
        _transfer_data: Vec<riglr_solana_events::TransferData>,
        _swap_data: Option<riglr_solana_events::SwapData>,
    ) {}
    
    fn index(&self) -> String {
        "0".to_string()
    }
    
    fn protocol_type(&self) -> ProtocolType {
        ProtocolType::Other("Binance".to_string())
    }
    
    fn timestamp(&self) -> SystemTime {
        self.stream_meta.received_at
    }
    
    fn transaction_hash(&self) -> Option<String> {
        None
    }
    
    fn block_number(&self) -> Option<u64> {
        Some(self.slot())
    }
}

#[async_trait::async_trait]
impl Stream for BinanceStream {
    type Event = BinanceStreamEvent;
    type Config = BinanceConfig;
    
    async fn start(&mut self, config: Self::Config) -> Result<(), StreamError> {
        if self.running.load(Ordering::Relaxed) {
            return Err(StreamError::AlreadyRunning { 
                name: self.name.clone() 
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
        let config = self.config.clone();
        
        tokio::spawn(async move {
            crate::impl_resilient_websocket!(
                stream_name,
                url.clone(),
                running,
                health,
                event_tx,
                // Connect function
                || {
                    let url = url.clone();
                    async move {
                        connect_async(&url).await
                            .map(|(ws, _)| ws)
                            .map_err(|e| format!("Failed to connect to Binance: {}", e))
                    }
                },
                // Subscribe function
                {
                    let _config = config.clone();
                    move |_write: &mut futures::stream::SplitSink<_, Message>| {
                        std::future::ready(Ok::<(), String>(()))
                    }
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
        
        // Create stream metadata
        let stream_meta = StreamMetadata {
            stream_source: "binance".to_string(),
            received_at: SystemTime::now(),
            sequence_number: Some(sequence_number),
            custom_data: Some(json.clone()),
        };
        
        // Create unique ID
        let id = format!("{}-{}-{}", symbol, event_type, sequence_number);
        
        Some(BinanceStreamEvent {
            id,
            data,
            stream_meta,
        })
    }
}