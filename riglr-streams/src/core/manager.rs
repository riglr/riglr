use std::any::Any;
use std::sync::Arc;
use dashmap::DashMap;
use tokio::sync::{broadcast, Mutex, RwLock};
use tokio::task::JoinHandle;
use tracing::{debug, error, info, warn};

use super::error::{StreamError, StreamResult};
use super::metrics::{MetricsCollector, MetricsTimer};
use super::stream::{DynamicStream, DynamicStreamWrapper, Stream, StreamHealth};

/// Execution mode for event handlers
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub enum HandlerExecutionMode {
    /// Execute handlers sequentially in order (preserves ordering guarantees)
    Sequential,
    /// Execute handlers concurrently (better throughput, no ordering guarantees)
    Concurrent,
    /// Execute handlers concurrently with a limit on parallelism
    ConcurrentBounded(usize),
}

impl Default for HandlerExecutionMode {
    fn default() -> Self {
        // Default to concurrent with reasonable parallelism limit
        HandlerExecutionMode::ConcurrentBounded(10)
    }
}

/// Manages multiple streams and routes events to handlers
pub struct StreamManager {
    /// Registered streams
    streams: Arc<DashMap<String, Box<dyn DynamicStream + 'static>>>,
    /// Running stream tasks
    running_streams: Arc<DashMap<String, JoinHandle<()>>>,
    /// Event handlers
    event_handlers: Arc<RwLock<Vec<Arc<dyn EventHandler>>>>,
    /// Global event channel for all streams
    global_event_tx: broadcast::Sender<Arc<dyn Any + Send + Sync>>,
    /// Shutdown signal
    shutdown_tx: broadcast::Sender<()>,
    /// Manager state
    state: Arc<Mutex<ManagerState>>,
    /// Handler execution mode
    execution_mode: Arc<RwLock<HandlerExecutionMode>>,
    /// Semaphore for bounded concurrent execution
    handler_semaphore: Arc<tokio::sync::Semaphore>,
    /// Metrics collector
    metrics_collector: Arc<MetricsCollector>,
}

/// Represents the current state of the StreamManager
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManagerState {
    /// Manager is idle and not processing any streams
    Idle,
    /// Manager is in the process of starting up
    Starting,
    /// Manager is actively running and processing streams
    Running,
    /// Manager is in the process of stopping
    Stopping,
    /// Manager has stopped and is no longer processing streams
    Stopped,
}

/// Trait for handling stream events
#[async_trait::async_trait]
pub trait EventHandler: Send + Sync {
    /// Check if this handler should process the event
    async fn should_handle(&self, event: &(dyn Any + Send + Sync)) -> bool;

    /// Process the event
    async fn handle(
        &self,
        event: Arc<dyn Any + Send + Sync>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    /// Get handler name for logging
    fn name(&self) -> &str;
}

impl StreamManager {
    /// Create a new StreamManager with default configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new StreamManager with specified execution mode
    pub fn with_execution_mode(mode: HandlerExecutionMode) -> Self {
        let mut manager = Self::default();
        let permits = match mode {
            HandlerExecutionMode::ConcurrentBounded(n) => n,
            _ => 10,
        };
        manager.handler_semaphore = Arc::new(tokio::sync::Semaphore::new(permits));
        manager.execution_mode = Arc::new(RwLock::new(mode));
        manager
    }

    /// Get the metrics collector
    pub fn metrics_collector(&self) -> Arc<MetricsCollector> {
        self.metrics_collector.clone()
    }

    /// Set the handler execution mode
    pub async fn set_execution_mode(&self, mode: HandlerExecutionMode) {
        let mut exec_mode = self.execution_mode.write().await;
        *exec_mode = mode;
        info!("Handler execution mode set to: {:?}", mode);
    }

    /// Add a stream to be managed (stream should already be configured and started)
    pub async fn add_stream<S>(&self, name: String, stream: S) -> StreamResult<()>
    where
        S: Stream + Send + Sync + 'static,
        S::Event: Send + Sync + 'static,
    {
        if self.streams.contains_key(&name) {
            return Err(StreamError::AlreadyRunning { name });
        }

        info!("Adding stream: {}", name);

        // Wrap the stream in a DynamicStreamWrapper
        let mut wrapper = DynamicStreamWrapper::new(stream);

        // If the stream is already running, set up forwarding
        if wrapper.inner.is_running() {
            wrapper.start_dynamic().await?;

            // Start forwarding events from this stream to global channel
            let stream_rx = wrapper.subscribe_dynamic();
            let global_tx = self.global_event_tx.clone();
            let stream_name = name.clone();
            let shutdown_rx = self.shutdown_tx.subscribe();

            let handle = tokio::spawn(async move {
                Self::forward_stream_events(stream_rx, global_tx, stream_name, shutdown_rx).await;
            });

            self.running_streams.insert(name.clone(), handle);
        }

        let dynamic_stream = Box::new(wrapper);
        self.streams.insert(name, dynamic_stream);
        Ok(())
    }

    /// Remove a stream
    pub async fn remove_stream(&self, name: &str) -> StreamResult<()> {
        // Stop the stream if it's running
        self.stop_stream(name).await?;

        self.streams.remove(name);

        info!("Removed stream: {}", name);
        Ok(())
    }

    /// Add an event handler
    pub async fn add_event_handler(&self, handler: Arc<dyn EventHandler>) {
        let mut handlers = self.event_handlers.write().await;
        info!("Adding event handler: {}", handler.name());
        handlers.push(handler);
    }

    /// Start a specific stream (stream should already be configured)
    pub async fn start_stream(&self, name: &str) -> StreamResult<()> {
        let mut stream_ref = self.streams
            .get_mut(name)
            .ok_or_else(|| StreamError::NotRunning {
                name: name.to_string(),
            })?;
        let stream = stream_ref.value_mut();

        if stream.is_running_dynamic() {
            return Err(StreamError::AlreadyRunning {
                name: name.to_string(),
            });
        }

        info!("Starting stream: {}", name);
        stream.start_dynamic().await?;

        // Start forwarding events from this stream
        let stream_rx = stream.subscribe_dynamic();
        let global_tx = self.global_event_tx.clone();
        let stream_name = name.to_string();
        let shutdown_rx = self.shutdown_tx.subscribe();

        let handle = tokio::spawn(async move {
            Self::forward_stream_events(stream_rx, global_tx, stream_name, shutdown_rx).await;
        });

        self.running_streams.insert(name.to_string(), handle);
        Ok(())
    }

    /// Stop a specific stream
    pub async fn stop_stream(&self, name: &str) -> StreamResult<()> {
        let mut stream_ref = self.streams
            .get_mut(name)
            .ok_or_else(|| StreamError::NotRunning {
                name: name.to_string(),
            })?;
        let stream = stream_ref.value_mut();

        if !stream.is_running_dynamic() {
            return Ok(()); // Already stopped
        }

        info!("Stopping stream: {}", name);
        stream.stop_dynamic().await?;

        // Cancel the forwarding task
        if let Some((_, handle)) = self.running_streams.remove(name) {
            handle.abort();
        }

        Ok(())
    }

    /// Start all registered streams
    pub async fn start_all(&self) -> StreamResult<()> {
        let mut state = self.state.lock().await;
        if *state != ManagerState::Idle && *state != ManagerState::Stopped {
            return Err(StreamError::AlreadyRunning {
                name: "StreamManager".to_string(),
            });
        }
        *state = ManagerState::Starting;

        info!("Starting all streams");

        let stream_names: Vec<String> = self.streams.iter().map(|entry| entry.key().clone()).collect();

        let mut errors = Vec::new();
        for name in stream_names {
            if let Err(e) = self.start_stream(&name).await {
                error!("Failed to start stream {}: {}", name, e);
                errors.push((name, e));
            }
        }

        *self.state.lock().await = ManagerState::Running;

        if !errors.is_empty() {
            let error_msg = errors
                .iter()
                .map(|(name, e)| format!("{}: {}", name, e))
                .collect::<Vec<_>>()
                .join(", ");
            return Err(StreamError::Internal {
                source: Box::new(std::io::Error::other(format!(
                    "Failed to start some streams: {}",
                    error_msg
                ))),
            });
        }

        Ok(())
    }

    /// Stop all streams
    pub async fn stop_all(&self) -> StreamResult<()> {
        let mut state = self.state.lock().await;
        if *state != ManagerState::Running {
            return Ok(());
        }
        *state = ManagerState::Stopping;

        info!("Stopping all streams");

        // Send shutdown signal
        let _ = self.shutdown_tx.send(());

        let stream_names: Vec<String> = self.streams.iter().map(|entry| entry.key().clone()).collect();

        for name in stream_names {
            if let Err(e) = self.stop_stream(&name).await {
                error!("Failed to stop stream {}: {}", name, e);
            }
        }

        // Wait for all forwarding tasks to complete
        let running_keys: Vec<String> = self.running_streams.iter().map(|entry| entry.key().clone()).collect();
        
        for name in running_keys {
            if let Some((_, handle)) = self.running_streams.remove(&name) {
                debug!("Waiting for stream {} to stop", name);
                handle.abort();
            }
        }

        *self.state.lock().await = ManagerState::Stopped;
        Ok(())
    }

    /// Process events from all streams
    pub async fn process_events(&self) -> StreamResult<()> {
        let state = *self.state.lock().await;
        if state != ManagerState::Running {
            return Err(StreamError::NotRunning {
                name: "StreamManager".to_string(),
            });
        }

        let mut global_event_rx = self.global_event_tx.subscribe();
        let mut shutdown_rx = self.shutdown_tx.subscribe();

        info!("Starting event processing loop");

        loop {
            tokio::select! {
                event = global_event_rx.recv() => {
                    match event {
                        Ok(event) => {
                            self.handle_event(event).await;
                        }
                        Err(broadcast::error::RecvError::Lagged(count)) => {
                            warn!("Event receiver lagged by {} messages", count);
                        }
                        Err(broadcast::error::RecvError::Closed) => {
                            info!("Event channel closed");
                            break;
                        }
                    }
                }
                _ = shutdown_rx.recv() => {
                    info!("Shutdown signal received");
                    break;
                }
            }
        }

        Ok(())
    }

    /// Get health status of all streams
    pub async fn health(&self) -> std::collections::HashMap<String, StreamHealth> {
        let mut health_map = std::collections::HashMap::new();

        // Collect stream names first to avoid holding references during async calls
        let stream_names: Vec<String> = self.streams.iter().map(|entry| entry.key().clone()).collect();
        
        for name in stream_names {
            if let Some(stream_ref) = self.streams.get(&name) {
                let health = stream_ref.value().health_dynamic().await;
                health_map.insert(name, health);
            }
        }

        health_map
    }

    /// Get list of stream names
    pub async fn list_streams(&self) -> Vec<String> {
        self.streams.iter().map(|entry| entry.key().clone()).collect()
    }

    /// Check if a stream is running
    pub async fn is_stream_running(&self, name: &str) -> bool {
        if let Some(stream) = self.streams.get(name) {
            stream.value().is_running_dynamic()
        } else {
            false
        }
    }

    /// Get manager state
    pub async fn state(&self) -> ManagerState {
        *self.state.lock().await
    }

    /// Forward events from a stream to the global channel
    async fn forward_stream_events(
        mut stream_rx: broadcast::Receiver<Arc<dyn Any + Send + Sync>>,
        global_tx: broadcast::Sender<Arc<dyn Any + Send + Sync>>,
        stream_name: String,
        mut shutdown_rx: broadcast::Receiver<()>,
    ) {
        loop {
            tokio::select! {
                event = stream_rx.recv() => {
                    match event {
                        Ok(event) => {
                            if let Err(e) = global_tx.send(event) {
                                warn!("Failed to forward event from stream {}: {}", stream_name, e);
                            }
                        }
                        Err(broadcast::error::RecvError::Lagged(count)) => {
                            warn!("Stream {} receiver lagged by {} messages", stream_name, count);
                        }
                        Err(broadcast::error::RecvError::Closed) => {
                            info!("Stream {} channel closed", stream_name);
                            break;
                        }
                    }
                }
                _ = shutdown_rx.recv() => {
                    debug!("Shutdown signal received for stream {}", stream_name);
                    break;
                }
            }
        }
    }

    /// Handle a single event
    async fn handle_event(&self, event: Arc<dyn Any + Send + Sync>) {
        let handlers = self.event_handlers.read().await;
        let execution_mode = *self.execution_mode.read().await;
        let metrics = self.metrics_collector.clone();

        // First, determine which handlers should process this event
        let mut eligible_handlers = Vec::new();
        for handler in handlers.iter() {
            if handler.should_handle(event.as_ref()).await {
                eligible_handlers.push(handler.clone());
            }
        }

        if eligible_handlers.is_empty() {
            return;
        }

        match execution_mode {
            HandlerExecutionMode::Sequential => {
                // Execute handlers sequentially (preserves order)
                for handler in eligible_handlers {
                    let handler_name = handler.name().to_string();
                    debug!("Handler {} processing event (sequential)", handler_name);

                    let timer = MetricsTimer::start_with_collector(
                        format!("handler:{}", handler_name),
                        metrics.clone(),
                    );

                    let result = handler.handle(event.clone()).await;
                    let success = result.is_ok();

                    if let Err(e) = result {
                        error!("Handler {} failed: {}", handler_name, e);
                    }

                    timer.stop(success).await;
                }
            }
            HandlerExecutionMode::Concurrent => {
                // Execute all handlers concurrently (no limit)
                let mut tasks = Vec::new();
                for handler in eligible_handlers {
                    let event = event.clone();
                    let metrics = metrics.clone();
                    let handler_name = handler.name().to_string();

                    tasks.push(tokio::spawn(async move {
                        debug!("Handler {} processing event (concurrent)", handler_name);

                        let timer = MetricsTimer::start_with_collector(
                            format!("handler:{}", handler_name),
                            metrics,
                        );

                        let result = handler.handle(event).await;
                        let success = result.is_ok();

                        if let Err(e) = result {
                            error!("Handler {} failed: {}", handler_name, e);
                        }

                        timer.stop(success).await;
                    }));
                }

                // Wait for all handlers to complete
                for task in tasks {
                    if let Err(e) = task.await {
                        error!("Handler task failed: {}", e);
                    }
                }
            }
            HandlerExecutionMode::ConcurrentBounded(_) => {
                // Execute handlers concurrently with semaphore limiting parallelism
                let semaphore = self.handler_semaphore.clone();
                let mut tasks = Vec::new();

                for handler in eligible_handlers {
                    let event = event.clone();
                    let sem = semaphore.clone();
                    let metrics = metrics.clone();
                    let handler_name = handler.name().to_string();

                    tasks.push(tokio::spawn(async move {
                        // Acquire permit before executing
                        let _permit = sem.acquire().await.expect("Failed to acquire semaphore");
                        debug!(
                            "Handler {} processing event (bounded concurrent)",
                            handler_name
                        );

                        let timer = MetricsTimer::start_with_collector(
                            format!("handler:{}", handler_name),
                            metrics,
                        );

                        let result = handler.handle(event).await;
                        let success = result.is_ok();

                        if let Err(e) = result {
                            error!("Handler {} failed: {}", handler_name, e);
                        }

                        timer.stop(success).await;
                    }));
                }

                // Wait for all handlers to complete
                for task in tasks {
                    if let Err(e) = task.await {
                        error!("Handler task failed: {}", e);
                    }
                }
            }
        }
    }
}

impl Default for StreamManager {
    fn default() -> Self {
        let (global_event_tx, _) = broadcast::channel(10000);
        let (shutdown_tx, _) = broadcast::channel(1);

        Self {
            streams: Arc::new(DashMap::new()),
            running_streams: Arc::new(DashMap::new()),
            event_handlers: Arc::new(RwLock::new(Vec::new())),
            global_event_tx,
            shutdown_tx,
            state: Arc::new(Mutex::new(ManagerState::Idle)),
            execution_mode: Arc::new(RwLock::new(HandlerExecutionMode::default())),
            handler_semaphore: Arc::new(tokio::sync::Semaphore::new(10)),
            metrics_collector: Arc::new(MetricsCollector::default()),
        }
    }
}

/// Simple event handler for testing
pub struct LoggingEventHandler {
    name: String,
}

impl LoggingEventHandler {
    /// Create a new LoggingEventHandler with the specified name
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

#[async_trait::async_trait]
impl EventHandler for LoggingEventHandler {
    async fn should_handle(&self, _event: &(dyn Any + Send + Sync)) -> bool {
        true // Handle all events
    }

    async fn handle(
        &self,
        _event: Arc<dyn Any + Send + Sync>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        debug!("LoggingEventHandler {} received event", self.name);
        Ok(())
    }

    fn name(&self) -> &str {
        &self.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_stream_manager_creation() {
        let manager = StreamManager::default();
        assert_eq!(manager.state().await, ManagerState::Idle);
        assert_eq!(manager.list_streams().await.len(), 0);
    }

    #[tokio::test]
    async fn test_add_event_handler() {
        let manager = StreamManager::default();
        let handler = Arc::new(LoggingEventHandler::new("test"));
        manager.add_event_handler(handler).await;

        // Handler count is not exposed, but we can verify no panic
        assert_eq!(manager.state().await, ManagerState::Idle);
    }
}
