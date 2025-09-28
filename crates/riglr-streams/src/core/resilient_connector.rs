//! Resilient WebSocket connector implementation
//!
//! This module provides a struct-based replacement for the `impl_resilient_websocket!` macro,
//! improving maintainability and debuggability while maintaining the same functionality.

use core::error::Error;
use core::fmt::{Debug, Formatter, Result as FmtResult};
use core::future::Future;
use core::pin::Pin;
use core::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Instant, SystemTime};

use tokio::time::error::Elapsed;
use tokio::time::{sleep, timeout, Duration};
use tokio_tungstenite::tungstenite::Error as TungsteniteError;

use futures::stream::{SplitSink, SplitStream};
use futures::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio::sync::{mpsc, RwLock};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};
use tracing::{debug, error, info, warn};

use crate::core::{DynamicStreamed, MetricsCollector, StreamHealth};

/// Configuration for resilient WebSocket connections
#[derive(Debug, Clone)]
pub struct ResilientConfig {
    /// Base delay for exponential backoff (in seconds)
    pub base_retry_delay_secs: u64,
    /// Long retry delay after max retries (in seconds)
    pub long_retry_delay_secs: u64,
    /// Maximum consecutive failures before stopping
    pub max_consecutive_failures: usize,
    /// Maximum idle timeouts before reconnecting
    pub max_idle_timeouts: usize,
    /// Maximum number of connection retry attempts
    pub max_retries: usize,
    /// Timeout duration for reading messages (in seconds)
    pub read_timeout_secs: u64,
}

impl Default for ResilientConfig {
    fn default() -> Self {
        Self {
            base_retry_delay_secs: 2,
            long_retry_delay_secs: 30,
            max_consecutive_failures: 10,
            max_idle_timeouts: 3,
            max_retries: 5,
            read_timeout_secs: 60,
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
type ParseFn = Box<dyn Fn(String, u64) -> Option<DynamicStreamed> + Send + Sync>;

/// Builder for creating resilient WebSocket connectors
pub struct ResilientWebSocketBuilder {
    config: ResilientConfig,
    connect_fn: ConnectFn,
    event_tx: mpsc::UnboundedSender<Arc<DynamicStreamed>>,
    health: Arc<RwLock<StreamHealth>>,
    metrics: Option<Arc<MetricsCollector>>,
    parse_fn: ParseFn,
    running: Arc<AtomicBool>,
    stream_name: String,
    subscribe_fn: SubscribeFn,
}

impl Debug for ResilientWebSocketBuilder {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_struct("ResilientWebSocketBuilder")
            .field("stream_name", &self.stream_name)
            .field("running", &self.running)
            .field("health", &self.health)
            .field("event_tx", &"<UnboundedSender>")
            .field("metrics", &self.metrics)
            .field("connect_fn", &"<ConnectFn>")
            .field("subscribe_fn", &"<SubscribeFn>")
            .field("parse_fn", &"<ParseFn>")
            .field("config", &self.config)
            .finish()
    }
}

impl ResilientWebSocketBuilder {
    /// Create a new builder
    #[must_use]
    pub fn new<C, S, P>(
        stream_name: impl Into<String>,
        running: Arc<AtomicBool>,
        health: Arc<RwLock<StreamHealth>>,
        event_tx: mpsc::UnboundedSender<Arc<DynamicStreamed>>,
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
        P: Fn(String, u64) -> Option<DynamicStreamed> + Send + Sync + 'static,
    {
        Self {
            config: ResilientConfig::default(),
            connect_fn: Box::new(connect_fn),
            event_tx,
            health,
            metrics: None,
            parse_fn: Box::new(parse_fn),
            running,
            stream_name: stream_name.into(),
            subscribe_fn: Box::new(subscribe_fn),
        }
    }

    /// Set custom configuration
    #[must_use]
    pub const fn with_config(mut self, config: ResilientConfig) -> Self {
        self.config = config;
        self
    }

    /// Set metrics collector
    #[must_use]
    pub fn with_metrics(mut self, metrics: Arc<MetricsCollector>) -> Self {
        self.metrics = Some(metrics);
        self
    }

    /// Build and return the connector
    #[must_use]
    pub fn build(self) -> ResilientWebSocketConnector {
        ResilientWebSocketConnector {
            config: self.config,
            connect_fn: self.connect_fn,
            event_tx: self.event_tx,
            health: self.health,
            metrics: self.metrics,
            parse_fn: self.parse_fn,
            running: self.running,
            stream_name: self.stream_name,
            subscribe_fn: self.subscribe_fn,
        }
    }
}

/// Resilient WebSocket connector that handles reconnection and error recovery
pub struct ResilientWebSocketConnector {
    config: ResilientConfig,
    connect_fn: ConnectFn,
    event_tx: mpsc::UnboundedSender<Arc<DynamicStreamed>>,
    health: Arc<RwLock<StreamHealth>>,
    metrics: Option<Arc<MetricsCollector>>,
    parse_fn: ParseFn,
    running: Arc<AtomicBool>,
    stream_name: String,
    subscribe_fn: SubscribeFn,
}

impl Debug for ResilientWebSocketConnector {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_struct("ResilientWebSocketConnector")
            .field("stream_name", &self.stream_name)
            .field("running", &self.running)
            .field("health", &self.health)
            .field("event_tx", &"<UnboundedSender>")
            .field("metrics", &self.metrics)
            .field("connect_fn", &"<ConnectFn>")
            .field("subscribe_fn", &"<SubscribeFn>")
            .field("parse_fn", &"<ParseFn>")
            .field("config", &self.config)
            .finish()
    }
}

impl ResilientWebSocketConnector {
    /// Run the resilient WebSocket connection
    pub async fn run(self) {
        let mut sequence_number = 0u64;
        let mut consecutive_failures: usize = 0;

        while self.running.load(Ordering::Relaxed) {
            let retry_result = self.connect_with_retry(&mut sequence_number).await;
            if retry_result == Ok(()) {
                consecutive_failures = 0;
            } else {
                {
                    consecutive_failures = consecutive_failures.saturating_add(1);
                }
                self.handle_connection_failure(consecutive_failures).await;

                if self.should_stop_after_failures(consecutive_failures) {
                    break;
                }

                self.wait_before_retry().await;
            }
        }

        info!("WebSocket handler exiting for {}", self.stream_name);
    }

    /// Handle connection failure by logging and updating health
    async fn handle_connection_failure(&self, _consecutive_failures: usize) {
        error!(
            "Failed to connect to {} after {} retries",
            self.stream_name, self.config.max_retries
        );

        // Update health
        let mut h = self.health.write().await;
        h.is_connected = false;
        {
            h.error_count = h.error_count.saturating_add(1);
        }
    }

    /// Check if we should stop after too many consecutive failures
    #[must_use]
    fn should_stop_after_failures(&self, consecutive_failures: usize) -> bool {
        if consecutive_failures >= self.config.max_consecutive_failures {
            error!(
                "Too many consecutive failures for {}. Stopping stream.",
                self.stream_name
            );
            self.running.store(false, Ordering::Relaxed);
            return true;
        }
        false
    }

    /// Wait before next connection retry
    async fn wait_before_retry(&self) {
        let sleep_duration = Duration::from_secs(self.config.long_retry_delay_secs);
        sleep(sleep_duration).await;
    }

    /// Connect with retry logic
    async fn connect_with_retry(&self, sequence_number: &mut u64) -> Result<(), ()> {
        let mut retry_count = 0;

        while retry_count < self.config.max_retries {
            let display_count = retry_count.saturating_add(1);
            info!(
                "Attempting to connect to {} (attempt {}/{})",
                self.stream_name, display_count, self.config.max_retries
            );

            let connect_result = (self.connect_fn)().await;
            match connect_result {
                Ok(ws_stream) => {
                    let result = self
                        .handle_successful_connection(ws_stream, sequence_number)
                        .await;
                    if result.is_ok() {
                        return Ok(());
                    }
                    // Connection succeeded but subscription or processing failed
                    {
                        retry_count = retry_count.saturating_add(1);
                    }
                    if retry_count < self.config.max_retries {
                        self.delay_retry(retry_count).await;
                    }
                }
                Err(e) => {
                    self.handle_connection_error(e, &mut retry_count).await;
                }
            }
        }
        Err(())
    }

    /// Handle successful connection and subscription
    async fn handle_successful_connection(
        &self,
        ws_stream: WebSocketType,
        sequence_number: &mut u64,
    ) -> Result<(), ()> {
        let (mut write, read) = ws_stream.split();

        // Send subscription messages
        let subscribe_result = (self.subscribe_fn)(Box::pin(&mut write)).await;
        match subscribe_result {
            Ok(()) => {
                info!(
                    "Successfully connected and subscribed to {}",
                    self.stream_name
                );

                self.update_health_connected().await;

                // Process messages
                self.process_messages(write, read, sequence_number).await;

                self.update_health_disconnected().await;
                Ok(())
            }
            Err(e) => {
                error!("Failed to subscribe: {}", e);
                Err(())
            }
        }
    }

    /// Handle connection error and update retry count
    async fn handle_connection_error(
        &self,
        e: Box<dyn Error + Send + Sync>,
        retry_count: &mut usize,
    ) {
        error!("Failed to connect to {}: {}", self.stream_name, e);
        {
            *retry_count = retry_count.saturating_add(1);
        }
        if *retry_count < self.config.max_retries {
            self.record_reconnection_metric(*retry_count);
            self.delay_retry(*retry_count).await;
        }
    }

    /// Record reconnection metric if appropriate
    fn record_reconnection_metric(&self, retry_count: usize) {
        if retry_count > 1 {
            if let Some(ref metrics) = self.metrics {
                metrics.record_reconnection(&self.stream_name);
            }
        }
    }

    /// Update health status to connected
    async fn update_health_connected(&self) {
        let mut h = self.health.write().await;
        h.is_connected = true;
        h.last_event_time = Some(SystemTime::now());
    }

    /// Update health status to disconnected with error count
    async fn update_health_disconnected(&self) {
        let mut h = self.health.write().await;
        h.is_connected = false;
        {
            h.error_count = h.error_count.saturating_add(1);
        }
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
            let timeout_result = self.read_message_with_timeout(&mut read).await;

            let should_break = self
                .handle_message_result(
                    timeout_result,
                    &mut write,
                    sequence_number,
                    &mut idle_counter,
                )
                .await;

            if should_break {
                break;
            }
        }
    }

    /// Read a message with timeout
    async fn read_message_with_timeout(
        &self,
        read: &mut SplitStream<WebSocketType>,
    ) -> Result<Option<Result<Message, TungsteniteError>>, Elapsed> {
        timeout(
            Duration::from_secs(self.config.read_timeout_secs),
            read.next(),
        )
        .await
    }

    /// Handle the result of reading a message and return whether to break the loop
    async fn handle_message_result(
        &self,
        timeout_result: Result<Option<Result<Message, TungsteniteError>>, Elapsed>,
        write: &mut SplitSink<WebSocketType, Message>,
        sequence_number: &mut u64,
        idle_counter: &mut usize,
    ) -> bool {
        match timeout_result {
            Ok(Some(Ok(message))) => {
                self.handle_websocket_message(message, write, sequence_number, idle_counter)
                    .await
            }
            Ok(Some(Err(e))) => {
                error!("WebSocket error for {}: {}", self.stream_name, e);
                true
            }
            Ok(None) => {
                warn!("WebSocket stream ended for {}", self.stream_name);
                true
            }
            Err(_) => self.handle_timeout(write, idle_counter).await,
        }
    }

    /// Handle a specific WebSocket message type
    async fn handle_websocket_message(
        &self,
        message: Message,
        write: &mut SplitSink<WebSocketType, Message>,
        sequence_number: &mut u64,
        idle_counter: &mut usize,
    ) -> bool {
        match message {
            Message::Text(text) => {
                *idle_counter = 0;
                {
                    *sequence_number = sequence_number.saturating_add(1);
                }
                self.handle_text_message(text.to_string(), *sequence_number)
                    .await;
                false
            }
            Message::Binary(data) => {
                *idle_counter = 0;
                {
                    *sequence_number = sequence_number.saturating_add(1);
                }
                debug!("Received binary message of {} bytes", data.len());
                false
            }
            Message::Ping(data) => {
                self.handle_ping_message(data.to_vec(), write, idle_counter)
                    .await
            }
            Message::Close(_) => {
                info!(
                    "WebSocket connection closed gracefully for {}",
                    self.stream_name
                );
                true
            }
            _ => {
                false // Ignore other message types
            }
        }
    }

    /// Handle ping message and return whether to break the loop
    async fn handle_ping_message(
        &self,
        data: Vec<u8>,
        write: &mut SplitSink<WebSocketType, Message>,
        idle_counter: &mut usize,
    ) -> bool {
        *idle_counter = 0;
        let send_result = write.send(Message::Pong(data.into())).await;
        if let Err(e) = send_result {
            warn!("Failed to send pong: {}", e);
            return true;
        }
        false
    }

    /// Handle timeout by sending ping and checking idle count
    async fn handle_timeout(
        &self,
        write: &mut SplitSink<WebSocketType, Message>,
        idle_counter: &mut usize,
    ) -> bool {
        {
            *idle_counter = idle_counter.saturating_add(1);
        }
        if *idle_counter > self.config.max_idle_timeouts {
            warn!(
                "WebSocket idle timeout for {}. Reconnecting...",
                self.stream_name
            );
            return true;
        }
        self.send_keepalive_ping(write).await
    }

    /// Send a ping message to keep the connection alive
    async fn send_keepalive_ping(&self, write: &mut SplitSink<WebSocketType, Message>) -> bool {
        let ping_result = write.send(Message::Ping(vec![].into())).await;
        if let Err(e) = ping_result {
            warn!("Failed to send ping: {}", e);
            return true;
        }
        false
    }

    /// Handle text message
    async fn handle_text_message(&self, text: String, sequence_number: u64) {
        let start = Instant::now();

        // Parse the message - extract to fix if-let rescope
        let parsed_event = (self.parse_fn)(text.clone(), sequence_number);
        if let Some(event) = parsed_event {
            // Send event
            let send_result = self.event_tx.send(Arc::new(event));
            if let Err(e) = send_result {
                warn!("Failed to send event: {}", e);
                // Record dropped event metric
                if let Some(ref metrics) = self.metrics {
                    metrics.record_dropped_event(&self.stream_name);
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
            {
                h.events_processed = h.events_processed.saturating_add(1);
            }
        }
    }

    /// Delay before retry with exponential backoff
    async fn delay_retry(&self, retry_count: usize) {
        let delay = Duration::from_secs(
            self.config
                .base_retry_delay_secs
                .saturating_pow(u32::try_from(retry_count).unwrap_or(0)),
        );
        info!("Retrying {} in {:?}", self.stream_name, delay);
        sleep(delay).await;
    }
}
