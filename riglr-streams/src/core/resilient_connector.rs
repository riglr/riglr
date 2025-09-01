//! Resilient WebSocket connector implementation
//!
//! This module provides a struct-based replacement for the `impl_resilient_websocket!` macro,
//! improving maintainability and debuggability while maintaining the same functionality.

use std::error::Error;
use std::future::Future;
use std::pin::Pin;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::SystemTime;

use futures::stream::{SplitSink, SplitStream};
use futures::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio::sync::{mpsc, RwLock};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};
use tracing::{debug, error, info, warn};

use crate::core::{DynamicStreamedEvent, MetricsCollector, StreamHealth};

/// Configuration for resilient WebSocket connections
#[derive(Debug, Clone)]
pub struct ResilientConfig {
    /// Maximum number of connection retry attempts
    pub max_retries: usize,
    /// Maximum consecutive failures before stopping
    pub max_consecutive_failures: usize,
    /// Maximum idle timeouts before reconnecting
    pub max_idle_timeouts: usize,
    /// Timeout duration for reading messages (in seconds)
    pub read_timeout_secs: u64,
    /// Base delay for exponential backoff (in seconds)
    pub base_retry_delay_secs: u64,
    /// Long retry delay after max retries (in seconds)
    pub long_retry_delay_secs: u64,
}

impl Default for ResilientConfig {
    fn default() -> Self {
        Self {
            max_retries: 5,
            max_consecutive_failures: 10,
            max_idle_timeouts: 3,
            read_timeout_secs: 60,
            base_retry_delay_secs: 2,
            long_retry_delay_secs: 30,
        }
    }
}

type WebSocketType = WebSocketStream<MaybeTlsStream<TcpStream>>;
type ConnectFn = Box<
    dyn Fn() -> Pin<
            Box<dyn Future<Output = Result<WebSocketType, Box<dyn Error + Send + Sync>>> + Send>,
        > + Send
        + Sync,
>;
type SubscribeFn = Box<
    dyn Fn(
            Pin<Box<&mut SplitSink<WebSocketType, Message>>>,
        )
            -> Pin<Box<dyn Future<Output = Result<(), Box<dyn Error + Send + Sync>>> + Send>>
        + Send
        + Sync,
>;
type ParseFn = Box<dyn Fn(String, u64) -> Option<DynamicStreamedEvent> + Send + Sync>;

/// Builder for creating resilient WebSocket connectors
pub struct ResilientWebSocketBuilder {
    stream_name: String,
    running: Arc<AtomicBool>,
    health: Arc<RwLock<StreamHealth>>,
    event_tx: mpsc::UnboundedSender<Arc<DynamicStreamedEvent>>,
    metrics: Option<Arc<MetricsCollector>>,
    connect_fn: ConnectFn,
    subscribe_fn: SubscribeFn,
    parse_fn: ParseFn,
    config: ResilientConfig,
}

impl ResilientWebSocketBuilder {
    /// Create a new builder
    pub fn new<C, S, P>(
        stream_name: impl Into<String>,
        running: Arc<AtomicBool>,
        health: Arc<RwLock<StreamHealth>>,
        event_tx: mpsc::UnboundedSender<Arc<DynamicStreamedEvent>>,
        connect_fn: C,
        subscribe_fn: S,
        parse_fn: P,
    ) -> Self
    where
        C: Fn() -> Pin<
                Box<
                    dyn Future<Output = Result<WebSocketType, Box<dyn Error + Send + Sync>>> + Send,
                >,
            > + Send
            + Sync
            + 'static,
        S: Fn(
                Pin<Box<&mut SplitSink<WebSocketType, Message>>>,
            )
                -> Pin<Box<dyn Future<Output = Result<(), Box<dyn Error + Send + Sync>>> + Send>>
            + Send
            + Sync
            + 'static,
        P: Fn(String, u64) -> Option<DynamicStreamedEvent> + Send + Sync + 'static,
    {
        Self {
            stream_name: stream_name.into(),
            running,
            health,
            event_tx,
            metrics: None,
            connect_fn: Box::new(connect_fn),
            subscribe_fn: Box::new(subscribe_fn),
            parse_fn: Box::new(parse_fn),
            config: ResilientConfig::default(),
        }
    }

    /// Set metrics collector
    pub fn with_metrics(mut self, metrics: Arc<MetricsCollector>) -> Self {
        self.metrics = Some(metrics);
        self
    }

    /// Set custom configuration
    pub fn with_config(mut self, config: ResilientConfig) -> Self {
        self.config = config;
        self
    }

    /// Build and return the connector
    pub fn build(self) -> ResilientWebSocketConnector {
        ResilientWebSocketConnector {
            stream_name: self.stream_name,
            running: self.running,
            health: self.health,
            event_tx: self.event_tx,
            metrics: self.metrics,
            connect_fn: self.connect_fn,
            subscribe_fn: self.subscribe_fn,
            parse_fn: self.parse_fn,
            config: self.config,
        }
    }
}

/// Resilient WebSocket connector that handles reconnection and error recovery
pub struct ResilientWebSocketConnector {
    stream_name: String,
    running: Arc<AtomicBool>,
    health: Arc<RwLock<StreamHealth>>,
    event_tx: mpsc::UnboundedSender<Arc<DynamicStreamedEvent>>,
    metrics: Option<Arc<MetricsCollector>>,
    connect_fn: ConnectFn,
    subscribe_fn: SubscribeFn,
    parse_fn: ParseFn,
    config: ResilientConfig,
}

impl ResilientWebSocketConnector {
    /// Run the resilient WebSocket connection
    pub async fn run(self) {
        let mut sequence_number = 0u64;
        let mut consecutive_failures = 0;

        while self.running.load(Ordering::Relaxed) {
            match self.connect_with_retry(&mut sequence_number).await {
                Ok(()) => {
                    consecutive_failures = 0;
                }
                Err(()) => {
                    consecutive_failures += 1;
                    error!(
                        "Failed to connect to {} after {} retries",
                        self.stream_name, self.config.max_retries
                    );

                    // Update health
                    {
                        let mut h = self.health.write().await;
                        h.is_connected = false;
                        h.error_count += 1;
                    }

                    if consecutive_failures >= self.config.max_consecutive_failures {
                        error!(
                            "Too many consecutive failures for {}. Stopping stream.",
                            self.stream_name
                        );
                        self.running.store(false, Ordering::Relaxed);
                        break;
                    }

                    // Wait before next attempt
                    tokio::time::sleep(tokio::time::Duration::from_secs(
                        self.config.long_retry_delay_secs,
                    ))
                    .await;
                }
            }
        }

        info!("WebSocket handler exiting for {}", self.stream_name);
    }

    /// Connect with retry logic
    async fn connect_with_retry(&self, sequence_number: &mut u64) -> Result<(), ()> {
        let mut retry_count = 0;

        while retry_count < self.config.max_retries {
            info!(
                "Attempting to connect to {} (attempt {}/{})",
                self.stream_name,
                retry_count + 1,
                self.config.max_retries
            );

            match (self.connect_fn)().await {
                Ok(ws_stream) => {
                    let (mut write, read) = ws_stream.split();

                    // Send subscription messages
                    match (self.subscribe_fn)(Box::pin(&mut write)).await {
                        Ok(()) => {
                            info!(
                                "Successfully connected and subscribed to {}",
                                self.stream_name
                            );

                            // Update health - connected
                            {
                                let mut h = self.health.write().await;
                                h.is_connected = true;
                                h.last_event_time = Some(SystemTime::now());
                            }

                            // Process messages
                            self.process_messages(write, read, sequence_number).await;

                            // Connection lost - update health
                            {
                                let mut h = self.health.write().await;
                                h.is_connected = false;
                                h.error_count += 1;
                            }

                            return Ok(());
                        }
                        Err(e) => {
                            error!("Failed to subscribe: {}", e);
                            retry_count += 1;
                            if retry_count < self.config.max_retries {
                                self.delay_retry(retry_count).await;
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to connect to {}: {}", self.stream_name, e);
                    retry_count += 1;
                    if retry_count < self.config.max_retries {
                        // Record reconnection attempt metric
                        if retry_count > 1 {
                            if let Some(ref metrics) = self.metrics {
                                metrics.record_reconnection(&self.stream_name).await;
                            }
                        }
                        self.delay_retry(retry_count).await;
                    }
                }
            }
        }

        Err(())
    }

    /// Process incoming WebSocket messages
    async fn process_messages(
        &self,
        mut write: SplitSink<WebSocketType, Message>,
        mut read: SplitStream<WebSocketType>,
        sequence_number: &mut u64,
    ) {
        let mut idle_counter = 0;

        loop {
            match tokio::time::timeout(
                tokio::time::Duration::from_secs(self.config.read_timeout_secs),
                read.next(),
            )
            .await
            {
                Ok(Some(Ok(Message::Text(text)))) => {
                    idle_counter = 0;
                    *sequence_number += 1;

                    self.handle_text_message(text.to_string(), *sequence_number)
                        .await;
                }
                Ok(Some(Ok(Message::Binary(data)))) => {
                    idle_counter = 0;
                    *sequence_number += 1;
                    debug!("Received binary message of {} bytes", data.len());
                }
                Ok(Some(Ok(Message::Ping(data)))) => {
                    idle_counter = 0;
                    if let Err(e) = write.send(Message::Pong(data)).await {
                        warn!("Failed to send pong: {}", e);
                        break;
                    }
                }
                Ok(Some(Ok(Message::Close(_)))) => {
                    info!(
                        "WebSocket connection closed gracefully for {}",
                        self.stream_name
                    );
                    break;
                }
                Ok(Some(Err(e))) => {
                    error!("WebSocket error for {}: {}", self.stream_name, e);
                    break;
                }
                Ok(None) => {
                    warn!("WebSocket stream ended for {}", self.stream_name);
                    break;
                }
                Err(_) => {
                    idle_counter += 1;
                    if idle_counter > self.config.max_idle_timeouts {
                        warn!(
                            "WebSocket idle timeout for {}. Reconnecting...",
                            self.stream_name
                        );
                        break;
                    }
                    // Send ping to keep connection alive
                    if let Err(e) = write.send(Message::Ping(vec![].into())).await {
                        warn!("Failed to send ping: {}", e);
                        break;
                    }
                }
                _ => {} // Ignore other message types
            }
        }
    }

    /// Handle text message
    async fn handle_text_message(&self, text: String, sequence_number: u64) {
        let start = std::time::Instant::now();

        // Parse the message
        if let Some(event) = (self.parse_fn)(text.clone(), sequence_number) {
            // Send event
            if let Err(e) = self.event_tx.send(std::sync::Arc::new(event)) {
                warn!("Failed to send event: {}", e);
                // Record dropped event metric
                if let Some(ref metrics) = self.metrics {
                    metrics.record_dropped_event(&self.stream_name).await;
                }
            } else {
                // Record successful event processing
                if let Some(ref metrics) = self.metrics {
                    let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;
                    metrics
                        .record_stream_event(&self.stream_name, elapsed_ms, text.len() as u64)
                        .await;
                }
            }

            // Update health
            let mut h = self.health.write().await;
            h.last_event_time = Some(SystemTime::now());
            h.events_processed += 1;
        }
    }

    /// Delay before retry with exponential backoff
    async fn delay_retry(&self, retry_count: usize) {
        let delay = tokio::time::Duration::from_secs(
            self.config.base_retry_delay_secs.pow(retry_count as u32),
        );
        info!("Retrying {} in {:?}", self.stream_name, delay);
        tokio::time::sleep(delay).await;
    }
}
