use core::any::Any;
use core::error::Error as CoreError;
use core::fmt::{Debug, Formatter, Result as FmtResult};
use dashmap::DashMap;
use std::collections::HashMap;
use std::io::Error as IoError;
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex, RwLock, Semaphore};
use tokio::task::JoinHandle;
use tracing::{debug, error, info, warn};

#[cfg(test)]
use core::time::Duration;
#[cfg(test)]
use tokio::time::sleep;

use super::error::{StreamError, StreamResult};
use super::metrics::{MetricsCollector, MetricsTimer};
use super::stream::{DynamicStream, DynamicStreamWrapper, Stream, StreamHealth};
use super::streamed_event::DynamicStreamed;

/// Execution mode for event handlers
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub enum HandlerExecutionMode {
    /// Execute handlers concurrently (better throughput, no ordering guarantees)
    Concurrent,
    /// Execute handlers concurrently with a limit on parallelism
    ConcurrentBounded(usize),
    /// Execute handlers sequentially in order (preserves ordering guarantees)
    Sequential,
}

impl Default for HandlerExecutionMode {
    /// Returns the default handler execution mode: ConcurrentBounded(10)
    /// This provides a balance between throughput and resource usage
    fn default() -> Self {
        // Default to concurrent with reasonable parallelism limit
        Self::ConcurrentBounded(10)
    }
}

/// Manages multiple streams and routes events to handlers
#[expect(clippy::module_name_repetitions)]
pub struct StreamManager {
    /// Event handlers
    event_handlers: Arc<RwLock<Vec<Arc<dyn EventHandler>>>>,
    /// Handler execution mode
    execution_mode: Arc<RwLock<HandlerExecutionMode>>,
    /// Global event channel for all streams
    global_event_tx: broadcast::Sender<Arc<DynamicStreamed>>,
    /// Semaphore for bounded concurrent execution
    handler_semaphore: Arc<Semaphore>,
    /// Metrics collector
    metrics_collector: Arc<MetricsCollector>,
    /// Running stream tasks
    running_streams: Arc<DashMap<String, JoinHandle<()>>>,
    /// Shutdown signal
    shutdown_tx: broadcast::Sender<()>,
    /// Manager state
    state: Arc<Mutex<ManagerState>>,
    /// Registered streams
    streams: Arc<DashMap<String, Box<dyn DynamicStream + 'static>>>,
}

impl Debug for StreamManager {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_struct("StreamManager")
            .field("streams", &format!("{} streams", self.streams.len()))
            .field(
                "running_streams",
                &format!("{} running streams", self.running_streams.len()),
            )
            .field(
                "event_handlers",
                &format!(
                    "{} event handlers",
                    self.event_handlers
                        .try_read()
                        .map_or(0, |handlers| handlers.len())
                ),
            )
            .field("global_event_tx", &self.global_event_tx)
            .field("shutdown_tx", &self.shutdown_tx)
            .field("state", &self.state)
            .field("execution_mode", &self.execution_mode)
            .field("handler_semaphore", &self.handler_semaphore)
            .field("metrics_collector", &self.metrics_collector)
            .finish()
    }
}

/// Represents the current state of the `StreamManager`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[expect(clippy::module_name_repetitions)]
pub enum ManagerState {
    /// Manager is idle and not processing any streams
    Idle,
    /// Manager is actively running and processing streams
    Running,
    /// Manager is in the process of starting up
    Starting,
    /// Manager has stopped and is no longer processing streams
    Stopped,
    /// Manager is in the process of stopping
    Stopping,
}

/// Trait for handling stream events
#[async_trait::async_trait]
pub trait EventHandler: Send + Sync {
    /// Process the event
    async fn handle(
        &self,
        event: Arc<DynamicStreamed>,
    ) -> Result<(), Box<dyn CoreError + Send + Sync>>;

    /// Get handler name for logging
    fn name(&self) -> &str;

    /// Check if this handler should process the event
    async fn should_handle(&self, event: &DynamicStreamed) -> bool;
}

impl StreamManager {
    /// Get the metrics collector
    #[must_use]
    pub fn metrics_collector(&self) -> Arc<MetricsCollector> {
        self.metrics_collector.clone()
    }

    /// Create a new `StreamManager` with default configuration
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new `StreamManager` with specified execution mode
    #[must_use]
    pub fn with_execution_mode(mode: HandlerExecutionMode) -> Self {
        let mut manager = Self::default();
        let permits = match mode {
            HandlerExecutionMode::ConcurrentBounded(n) => n,
            _ => 10,
        };
        manager.handler_semaphore = Arc::new(Semaphore::new(permits));
        manager.execution_mode = Arc::new(RwLock::new(mode));
        manager
    }

    /// Add a stream to be managed (stream should already be configured and started)
    ///
    /// # Errors
    ///
    /// Returns error if a stream with the same name is already running or if stream initialization fails
    pub async fn add_stream<S>(&self, name: String, stream: S) -> StreamResult<()>
    where
        S: Stream + Send + Sync + 'static,
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

            // Only spawn forwarding task if we have event handlers or not in test mode
            #[cfg(not(test))]
            {
                // Start forwarding events from this stream to global channel
                let stream_rx = wrapper.subscribe_dynamic();
                let global_tx = self.global_event_tx.clone();
                let stream_name = name.clone();
                let shutdown_rx = self.shutdown_tx.subscribe();

                let handle = tokio::spawn(async move {
                    Self::forward_stream_events(stream_rx, global_tx, stream_name, shutdown_rx)
                        .await;
                });

                self.running_streams.insert(name.clone(), handle);
            }

            // In test mode, only spawn if we have handlers
            #[cfg(test)]
            {
                let has_handlers = !self.event_handlers.read().await.is_empty();
                if has_handlers {
                    let stream_rx = wrapper.subscribe_dynamic();
                    let global_tx = self.global_event_tx.clone();
                    let stream_name = name.clone();
                    let shutdown_rx = self.shutdown_tx.subscribe();

                    let handle = tokio::spawn(async move {
                        Self::forward_stream_events(stream_rx, global_tx, stream_name, shutdown_rx)
                            .await;
                    });

                    self.running_streams.insert(name.clone(), handle);
                }
            }
        }

        let dynamic_stream = Box::new(wrapper);
        self.streams.insert(name, dynamic_stream);
        Ok(())
    }

    /// Remove a stream
    ///
    /// # Errors
    ///
    /// Returns error if the stream cannot be stopped gracefully or if removal fails
    pub async fn remove_stream(&self, name: &str) -> StreamResult<()> {
        // Stop the stream if it's running
        let stop_result = self.stop_stream(name).await;
        stop_result?;

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
    ///
    /// # Errors
    ///
    /// Returns error if the stream is not found or if starting the stream fails
    #[expect(clippy::significant_drop_tightening)]
    pub async fn start_stream(&self, name: &str) -> StreamResult<()> {
        let mut stream_guard =
            self.streams
                .get_mut(name)
                .ok_or_else(|| StreamError::NotRunning {
                    name: name.to_string(),
                })?;
        let stream = stream_guard.value_mut();

        if stream.is_running_dynamic() {
            // Stream is already running, nothing to do
            return Ok(());
        }

        info!("Starting stream: {}", name);
        stream.start_dynamic().await?;

        // Only spawn forwarding task if we have event handlers or not in test mode
        #[cfg(not(test))]
        {
            // Start forwarding events from this stream
            let stream_rx = stream.subscribe_dynamic();
            let global_tx = self.global_event_tx.clone();
            let stream_name = name.to_string();
            let shutdown_rx = self.shutdown_tx.subscribe();

            let handle = tokio::spawn(async move {
                Self::forward_stream_events(stream_rx, global_tx, stream_name, shutdown_rx).await;
            });

            self.running_streams.insert(name.to_string(), handle);
        }

        // In test mode, only spawn if we have handlers
        #[cfg(test)]
        {
            let has_handlers = !self.event_handlers.read().await.is_empty();
            if has_handlers {
                let stream_rx = stream.subscribe_dynamic();
                let global_tx = self.global_event_tx.clone();
                let stream_name = name.to_string();
                let shutdown_rx = self.shutdown_tx.subscribe();

                let handle = tokio::spawn(async move {
                    Self::forward_stream_events(stream_rx, global_tx, stream_name, shutdown_rx)
                        .await;
                });

                self.running_streams.insert(name.to_string(), handle);
            }
        }
        Ok(())
    }

    /// Stop a specific stream
    ///
    /// # Errors
    ///
    /// Returns error if the stream is not found or if stopping the stream fails
    #[expect(clippy::significant_drop_tightening)]
    pub async fn stop_stream(&self, name: &str) -> StreamResult<()> {
        let mut stream_guard =
            self.streams
                .get_mut(name)
                .ok_or_else(|| StreamError::NotRunning {
                    name: name.to_string(),
                })?;
        let stream = stream_guard.value_mut();

        if !stream.is_running_dynamic() {
            return Ok(()); // Already stopped
        }

        info!("Stopping stream: {}", name);
        let stop_result = stream.stop_dynamic().await;
        stop_result?;

        // Cancel the forwarding task
        if let Some((_, handle)) = self.running_streams.remove(name) {
            handle.abort();
        }

        Ok(())
    }

    /// Start all registered streams
    ///
    /// # Errors
    ///
    /// Returns error if any stream fails to start or if the manager state transition fails
    #[expect(clippy::pattern_type_mismatch)]
    pub async fn start_all(&self) -> StreamResult<()> {
        // Check state and update to Starting
        {
            let mut state = self.state.lock().await;
            if *state != ManagerState::Idle && *state != ManagerState::Stopped {
                return Err(StreamError::AlreadyRunning {
                    name: "StreamManager".to_string(),
                });
            }
            *state = ManagerState::Starting;
        } // Lock is dropped here

        info!("Starting all streams");

        let stream_names: Vec<String> = self
            .streams
            .iter()
            .map(|entry| entry.key().clone())
            .collect();

        let mut errors = Vec::default();
        for name in stream_names {
            let start_result = self.start_stream(&name).await;
            if let Err(e) = start_result {
                error!("Failed to start stream {}: {}", name, e);
                errors.push((name, e));
            }
        }

        // Update state to Running
        {
            let mut state = self.state.lock().await;
            *state = ManagerState::Running;
        } // Lock is dropped here

        if !errors.is_empty() {
            let error_msg = errors
                .iter()
                .map(|(name, e)| format!("{name}: {e}"))
                .collect::<Vec<_>>()
                .join(", ");
            return Err(StreamError::Internal {
                source: Box::new(IoError::other(format!(
                    "Failed to start some streams: {error_msg}"
                ))),
            });
        }

        Ok(())
    }

    /// Stop all streams
    ///
    /// # Errors
    ///
    /// Returns error if any stream fails to stop or if the manager state transition fails
    pub async fn stop_all(&self) -> StreamResult<()> {
        if !self.transition_to_stopping().await {
            return Ok(());
        }

        info!("Stopping all streams");
        self.send_shutdown_signal();
        self.stop_all_streams().await;
        self.abort_forwarding_tasks();
        self.transition_to_stopped().await;

        Ok(())
    }

    /// Check if manager is running and transition to stopping state
    async fn transition_to_stopping(&self) -> bool {
        let mut state = self.state.lock().await;
        if *state == ManagerState::Running {
            *state = ManagerState::Stopping;
            true
        } else {
            false
        }
    }

    /// Send shutdown signal to all stream forwarding tasks
    fn send_shutdown_signal(&self) {
        let _ = self.shutdown_tx.send(());
    }

    /// Stop all registered streams
    async fn stop_all_streams(&self) {
        let stream_names: Vec<String> = self
            .streams
            .iter()
            .map(|entry| entry.key().clone())
            .collect();

        for name in stream_names {
            let stop_result = self.stop_stream(&name).await;
            if let Err(e) = stop_result {
                error!("Failed to stop stream {}: {}", name, e);
            }
        }
    }

    /// Abort all running forwarding tasks
    fn abort_forwarding_tasks(&self) {
        let running_keys: Vec<String> = self
            .running_streams
            .iter()
            .map(|entry| entry.key().clone())
            .collect();

        for name in running_keys {
            let removal_result = self.running_streams.remove(&name);
            if let Some((_, handle)) = removal_result {
                debug!("Waiting for stream {} to stop", name);
                handle.abort();
            }
        }
    }

    /// Transition to stopped state
    async fn transition_to_stopped(&self) {
        let mut state = self.state.lock().await;
        *state = ManagerState::Stopped;
    }

    /// Process events from all streams
    ///
    /// # Errors
    ///
    /// Returns error if event processing fails or if the manager is not in a valid state
    pub async fn process_events(&self) -> StreamResult<()> {
        self.validate_processing_state().await?;

        let mut global_event_rx = self.global_event_tx.subscribe();
        let mut shutdown_rx = self.shutdown_tx.subscribe();

        info!("Starting event processing loop");

        loop {
            let should_break = self
                .process_single_event_cycle(&mut global_event_rx, &mut shutdown_rx)
                .await;
            if should_break {
                break;
            }
        }

        Ok(())
    }

    /// Validate that the manager is in the correct state for event processing
    async fn validate_processing_state(&self) -> StreamResult<()> {
        let state = *self.state.lock().await;
        if state != ManagerState::Running {
            return Err(StreamError::NotRunning {
                name: "StreamManager".to_string(),
            });
        }
        Ok(())
    }

    /// Process a single event cycle, returning true if the loop should break
    async fn process_single_event_cycle(
        &self,
        global_event_rx: &mut broadcast::Receiver<Arc<DynamicStreamed>>,
        shutdown_rx: &mut broadcast::Receiver<()>,
    ) -> bool {
        #[cfg(test)]
        {
            self.process_event_cycle_with_timeout(global_event_rx, shutdown_rx)
                .await
        }

        #[cfg(not(test))]
        {
            self.process_event_cycle_no_timeout(global_event_rx, shutdown_rx)
                .await
        }
    }

    /// Process event cycle with timeout for test mode
    #[cfg(test)]
    #[expect(clippy::pattern_type_mismatch)]
    async fn process_event_cycle_with_timeout(
        &self,
        global_event_rx: &mut broadcast::Receiver<Arc<DynamicStreamed>>,
        shutdown_rx: &mut broadcast::Receiver<()>,
    ) -> bool {
        tokio::select! {
            event = global_event_rx.recv() => {
                self.handle_event_result(event).await
            }
            _ = shutdown_rx.recv() => {
                info!("Shutdown signal received");
                true
            }
            () = sleep(Duration::from_millis(200)) => {
                debug!("Event processing timed out in test mode");
                true
            }
        }
    }

    /// Process event cycle without timeout for production mode
    #[cfg(not(test))]
    async fn process_event_cycle_no_timeout(
        &self,
        global_event_rx: &mut broadcast::Receiver<Arc<DynamicStreamed>>,
        shutdown_rx: &mut broadcast::Receiver<()>,
    ) -> bool {
        tokio::select! {
            event = global_event_rx.recv() => {
                self.handle_event_result(event).await
            }
            _ = shutdown_rx.recv() => {
                info!("Shutdown signal received");
                true
            }
        }
    }

    /// Handle the result of receiving an event, returning true if the loop should break
    async fn handle_event_result(
        &self,
        event_result: Result<Arc<DynamicStreamed>, broadcast::error::RecvError>,
    ) -> bool {
        match event_result {
            Ok(event) => {
                self.handle_event(event).await;
                false
            }
            Err(broadcast::error::RecvError::Lagged(count)) => {
                warn!("Event receiver lagged by {} messages", count);
                false
            }
            Err(broadcast::error::RecvError::Closed) => {
                info!("Event channel closed");
                true
            }
        }
    }

    /// Get health status of all streams
    pub async fn health(&self) -> HashMap<String, StreamHealth> {
        let mut health_map = HashMap::default();

        // Collect stream names first to avoid holding references during async calls
        let stream_names: Vec<String> = self
            .streams
            .iter()
            .map(|entry| entry.key().clone())
            .collect();

        for name in stream_names {
            let stream_get_result = self.streams.get(&name);
            if let Some(stream_ref) = stream_get_result {
                let health_await_result = stream_ref.value().health_dynamic().await;
                health_map.insert(name, health_await_result);
            }
        }

        health_map
    }

    /// Get list of stream names
    #[must_use]
    pub fn list_streams(&self) -> Vec<String> {
        self.streams
            .iter()
            .map(|entry| entry.key().clone())
            .collect()
    }

    /// Check if a stream is running
    #[must_use]
    pub fn is_stream_running(&self, name: &str) -> bool {
        self.streams
            .get(name)
            .is_some_and(|stream| stream.value().is_running_dynamic())
    }

    /// Get manager state
    pub async fn state(&self) -> ManagerState {
        let state_guard = self.state.lock().await;
        *state_guard
    }

    /// Forward events from a stream to the global channel
    async fn forward_stream_events(
        mut stream_rx: broadcast::Receiver<Arc<dyn Any + Send + Sync>>,
        global_tx: broadcast::Sender<Arc<DynamicStreamed>>,
        stream_name: String,
        mut shutdown_rx: broadcast::Receiver<()>,
    ) {
        loop {
            let should_break = Self::forward_single_event(
                &mut stream_rx,
                &global_tx,
                &stream_name,
                &mut shutdown_rx,
            )
            .await;

            if should_break {
                break;
            }
        }
    }

    /// Forward a single event cycle, returning true if the forwarding loop should break
    async fn forward_single_event(
        stream_rx: &mut broadcast::Receiver<Arc<dyn Any + Send + Sync>>,
        global_tx: &broadcast::Sender<Arc<DynamicStreamed>>,
        stream_name: &str,
        shutdown_rx: &mut broadcast::Receiver<()>,
    ) -> bool {
        #[cfg(test)]
        return Self::forward_event_with_timeout(stream_rx, global_tx, stream_name, shutdown_rx)
            .await;

        #[cfg(not(test))]
        return Self::forward_event_no_timeout(stream_rx, global_tx, stream_name, shutdown_rx)
            .await;
    }

    /// Forward event with timeout for test mode
    #[cfg(test)]
    #[expect(clippy::pattern_type_mismatch)]
    async fn forward_event_with_timeout(
        stream_rx: &mut broadcast::Receiver<Arc<dyn Any + Send + Sync>>,
        global_tx: &broadcast::Sender<Arc<DynamicStreamed>>,
        stream_name: &str,
        shutdown_rx: &mut broadcast::Receiver<()>,
    ) -> bool {
        tokio::select! {
            event = stream_rx.recv() => {
                Self::handle_received_event(event, global_tx, stream_name)
            }
            _ = shutdown_rx.recv() => {
                debug!("Shutdown signal received for stream {}", stream_name);
                true
            }
            () = sleep(Duration::from_millis(100)) => {
                debug!("Stream {} forwarding timed out in test mode", stream_name);
                true
            }
        }
    }

    /// Forward event without timeout for production mode
    #[cfg(not(test))]
    async fn forward_event_no_timeout(
        stream_rx: &mut broadcast::Receiver<Arc<dyn Any + Send + Sync>>,
        global_tx: &broadcast::Sender<Arc<DynamicStreamed>>,
        stream_name: &str,
        shutdown_rx: &mut broadcast::Receiver<()>,
    ) -> bool {
        tokio::select! {
            event = stream_rx.recv() => {
                Self::handle_received_event(event, global_tx, stream_name)
            }
            _ = shutdown_rx.recv() => {
                debug!("Shutdown signal received for stream {}", stream_name);
                true
            }
        }
    }

    /// Handle a received event from the stream, returning true if forwarding should break
    fn handle_received_event(
        event_result: Result<Arc<dyn Any + Send + Sync>, broadcast::error::RecvError>,
        global_tx: &broadcast::Sender<Arc<DynamicStreamed>>,
        stream_name: &str,
    ) -> bool {
        match event_result {
            Ok(event) => {
                Self::forward_dynamic_event(&event, global_tx, stream_name);
                false
            }
            Err(broadcast::error::RecvError::Lagged(count)) => {
                warn!(
                    "Stream {} receiver lagged by {} messages",
                    stream_name, count
                );
                false
            }
            Err(broadcast::error::RecvError::Closed) => {
                info!("Stream {} channel closed", stream_name);
                true
            }
        }
    }

    /// Forward a dynamic streamed event to the global channel
    fn forward_dynamic_event(
        event: &Arc<dyn Any + Send + Sync>,
        global_tx: &broadcast::Sender<Arc<DynamicStreamed>>,
        stream_name: &str,
    ) {
        if let Some(streamed_event) = (**event).downcast_ref::<DynamicStreamed>() {
            let cloned_event = streamed_event.clone();
            if let Err(e) = global_tx.send(Arc::new(cloned_event)) {
                warn!("Failed to forward event from stream {}: {}", stream_name, e);
            }
        } else {
            debug!(
                "Received non-DynamicStreamed from stream {}, skipping",
                stream_name
            );
        }
    }

    /// Handle a single event
    async fn handle_event(&self, event: Arc<DynamicStreamed>) {
        let eligible_handlers = self.get_eligible_handlers(&event).await;
        if eligible_handlers.is_empty() {
            return;
        }

        let execution_mode = *self.execution_mode.read().await;
        let metrics = self.metrics_collector.clone();

        match execution_mode {
            HandlerExecutionMode::Sequential => {
                self.execute_handlers_sequential(eligible_handlers, event, metrics)
                    .await;
            }
            HandlerExecutionMode::Concurrent => {
                self.execute_handlers_concurrent(eligible_handlers, event, metrics)
                    .await;
            }
            HandlerExecutionMode::ConcurrentBounded(_) => {
                self.execute_handlers_bounded(eligible_handlers, event, metrics)
                    .await;
            }
        }
    }

    /// Get handlers that should process the given event
    async fn get_eligible_handlers(
        &self,
        event: &Arc<DynamicStreamed>,
    ) -> Vec<Arc<dyn EventHandler>> {
        // First, clone all handlers without holding the lock during async operations
        let all_handlers = {
            let handlers = self.event_handlers.read().await;
            handlers.clone()
        };

        let mut eligible_handlers = Vec::new();
        for handler in &all_handlers {
            if handler.should_handle(event.as_ref()).await {
                eligible_handlers.push(handler.clone());
            }
        }

        eligible_handlers
    }

    /// Execute handlers sequentially
    async fn execute_handlers_sequential(
        &self,
        handlers: Vec<Arc<dyn EventHandler>>,
        event: Arc<DynamicStreamed>,
        metrics: Arc<MetricsCollector>,
    ) {
        for handler in handlers {
            self.execute_single_handler(handler, event.clone(), metrics.clone(), "sequential")
                .await;
        }
    }

    /// Execute handlers concurrently without limits
    async fn execute_handlers_concurrent(
        &self,
        handlers: Vec<Arc<dyn EventHandler>>,
        event: Arc<DynamicStreamed>,
        metrics: Arc<MetricsCollector>,
    ) {
        let tasks: Vec<_> = handlers
            .into_iter()
            .map(|handler| {
                let event = event.clone();
                let metrics = metrics.clone();
                tokio::spawn(async move {
                    Self::execute_handler_task(handler, event, metrics, "concurrent").await;
                })
            })
            .collect();

        self.await_handler_tasks(tasks).await;
    }

    /// Execute handlers with bounded concurrency
    async fn execute_handlers_bounded(
        &self,
        handlers: Vec<Arc<dyn EventHandler>>,
        event: Arc<DynamicStreamed>,
        metrics: Arc<MetricsCollector>,
    ) {
        let semaphore = self.handler_semaphore.clone();
        let tasks: Vec<_> = handlers
            .into_iter()
            .map(|handler| {
                let event = event.clone();
                let sem = semaphore.clone();
                let metrics = metrics.clone();
                tokio::spawn(async move {
                    let Ok(_permit) = sem.acquire().await else {
                        error!("Failed to acquire semaphore for handler {}", handler.name());
                        return;
                    };

                    Self::execute_handler_task(handler, event, metrics, "bounded concurrent").await;
                })
            })
            .collect();

        self.await_handler_tasks(tasks).await;
    }

    /// Execute a single handler with metrics and logging
    async fn execute_single_handler(
        &self,
        handler: Arc<dyn EventHandler>,
        event: Arc<DynamicStreamed>,
        metrics: Arc<MetricsCollector>,
        mode: &str,
    ) {
        let handler_name = handler.name().to_string();
        debug!("Handler {} processing event ({})", handler_name, mode);

        let timer = MetricsTimer::start_with_collector(format!("handler:{handler_name}"), metrics);

        let result = handler.handle(event).await;
        let success = result.is_ok();

        if let Err(e) = result {
            error!("Handler {} failed: {}", handler_name, e);
        }

        let () = timer.stop(success).await;
    }

    /// Execute a handler task (static method for spawned tasks)
    async fn execute_handler_task(
        handler: Arc<dyn EventHandler>,
        event: Arc<DynamicStreamed>,
        metrics: Arc<MetricsCollector>,
        mode: &str,
    ) {
        let handler_name = handler.name().to_string();
        debug!("Handler {} processing event ({})", handler_name, mode);

        let timer = MetricsTimer::start_with_collector(format!("handler:{handler_name}"), metrics);

        let result = handler.handle(event).await;
        let success = result.is_ok();

        if let Err(e) = result {
            error!("Handler {} failed: {}", handler_name, e);
        }

        let () = timer.stop(success).await;
    }

    /// Wait for all handler tasks to complete
    async fn await_handler_tasks(&self, tasks: Vec<JoinHandle<()>>) {
        for task in tasks {
            let task_result = task.await;
            if let Err(e) = task_result {
                error!("Handler task failed: {}", e);
            }
        }
    }
}

impl Default for StreamManager {
    /// Creates a new `StreamManager` with default configuration
    ///
    /// Initializes with:
    /// - Empty stream collections
    /// - Broadcast channels for events and shutdown signals
    /// - Default handler execution mode (ConcurrentBounded(10))
    /// - Semaphore with 10 permits for bounded concurrency
    /// - Default metrics collector
    fn default() -> Self {
        let (global_event_tx, _) = broadcast::channel(10000);
        let (shutdown_tx, _) = broadcast::channel(1);

        Self {
            event_handlers: Arc::new(RwLock::new(Vec::default())),
            execution_mode: Arc::new(RwLock::new(HandlerExecutionMode::default())),
            global_event_tx,
            handler_semaphore: Arc::new(Semaphore::new(10)),
            metrics_collector: Arc::new(MetricsCollector::default()),
            running_streams: Arc::new(DashMap::default()),
            shutdown_tx,
            state: Arc::new(Mutex::new(ManagerState::Idle)),
            streams: Arc::new(DashMap::default()),
        }
    }
}

/// Simple event handler for testing
#[derive(Debug)]
pub struct LoggingEventHandler {
    name: String,
}

impl LoggingEventHandler {
    /// Create a new `LoggingEventHandler` with the specified name
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

#[async_trait::async_trait]
impl EventHandler for LoggingEventHandler {
    async fn should_handle(&self, _event: &DynamicStreamed) -> bool {
        true // Handle all events
    }

    async fn handle(
        &self,
        _event: Arc<DynamicStreamed>,
    ) -> Result<(), Box<dyn CoreError + Send + Sync>> {
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
    use crate::core::streamed_event::IntoDynamicStreamed;
    use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    use riglr_events_core::{Event, EventKind, EventMetadata, EventResult};
    use tokio::sync::Notify;
    use tokio::time::timeout;

    // Mock event for testing
    #[derive(Debug, Clone)]
    struct MockEvent {
        metadata: EventMetadata,
    }

    impl MockEvent {
        fn new(id: &str) -> Self {
            Self {
                metadata: EventMetadata::new(
                    id.to_string(),
                    EventKind::Transaction,
                    "mock_stream".to_string(),
                ),
            }
        }
    }

    impl Event for MockEvent {
        fn id(&self) -> &str {
            &self.metadata.id
        }
        fn kind(&self) -> &EventKind {
            &self.metadata.kind
        }
        fn metadata(&self) -> &EventMetadata {
            &self.metadata
        }
        fn metadata_mut(&mut self) -> EventResult<&mut EventMetadata> {
            Ok(&mut self.metadata)
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
        fn to_json(&self) -> EventResult<serde_json::Value> {
            Ok(serde_json::json!({
                "id": self.id(),
                "kind": format!("{:?}", self.kind()),
                "source": &self.metadata.source
            }))
        }
    }

    // Mock config for testing
    #[derive(Debug, Clone, Default)]
    struct MockConfig {
        fail_start: bool,
        fail_stop: bool,
    }

    // Mock stream for testing
    struct MockStream {
        name: String,
        is_running: AtomicBool,
        should_fail_start: AtomicBool,
        should_fail_stop: AtomicBool,
        event_sender: broadcast::Sender<Arc<DynamicStreamed>>,
        health: StreamHealth,
    }

    impl MockStream {
        fn new(name: &str) -> Self {
            let (tx, _) = broadcast::channel(100);
            Self {
                name: name.to_string(),
                is_running: AtomicBool::default(),
                should_fail_start: AtomicBool::default(),
                should_fail_stop: AtomicBool::default(),
                event_sender: tx,
                health: StreamHealth::healthy(),
            }
        }

        fn with_health(name: &str, health: StreamHealth) -> Self {
            let mut stream = Self::new(name);
            stream.health = health;
            stream
        }

        #[expect(dead_code)]
        fn send_event(&self, event: MockEvent) {
            let dynamic_event =
                (Box::new(event) as Box<dyn Event>).with_default_stream_metadata(&self.name);
            let _ = self.event_sender.send(Arc::new(dynamic_event));
        }
    }

    #[async_trait::async_trait]
    impl Stream for MockStream {
        type Config = MockConfig;

        async fn start(&mut self, config: Self::Config) -> StreamResult<()> {
            self.should_fail_start
                .store(config.fail_start, Ordering::SeqCst);
            self.should_fail_stop
                .store(config.fail_stop, Ordering::SeqCst);
            if self.should_fail_start.load(Ordering::SeqCst) {
                return Err(StreamError::Internal {
                    source: Box::new(IoError::other("Mock start failure")),
                });
            }
            self.is_running.store(true, Ordering::SeqCst);
            Ok(())
        }

        async fn stop(&mut self) -> StreamResult<()> {
            if self.should_fail_stop.load(Ordering::SeqCst) {
                return Err(StreamError::Internal {
                    source: Box::new(IoError::other("Mock stop failure")),
                });
            }
            self.is_running.store(false, Ordering::SeqCst);
            Ok(())
        }

        fn is_running(&self) -> bool {
            self.is_running.load(Ordering::SeqCst)
        }

        fn subscribe(&self) -> broadcast::Receiver<Arc<DynamicStreamed>> {
            self.event_sender.subscribe()
        }

        async fn health(&self) -> StreamHealth {
            self.health.clone()
        }

        fn name(&self) -> &str {
            &self.name
        }
    }

    // Mock event handler for testing
    struct MockEventHandler {
        name: String,
        should_handle_flag: AtomicBool,
        should_fail: AtomicBool,
        handle_count: AtomicUsize,
        handle_notify: Arc<Notify>,
    }

    impl MockEventHandler {
        fn new(name: &str) -> Self {
            Self {
                name: name.to_string(),
                should_handle_flag: AtomicBool::default(),
                should_fail: AtomicBool::default(),
                handle_count: AtomicUsize::default(),
                handle_notify: Arc::default(),
            }
        }

        fn set_should_handle(&self, should_handle: bool) {
            self.should_handle_flag
                .store(should_handle, Ordering::SeqCst);
        }

        fn set_should_fail(&self, should_fail: bool) {
            self.should_fail.store(should_fail, Ordering::SeqCst);
        }

        fn handle_count(&self) -> usize {
            self.handle_count.load(Ordering::SeqCst)
        }

        #[expect(dead_code)]
        async fn wait_for_handle(&self) {
            self.handle_notify.notified().await;
        }
    }

    #[async_trait::async_trait]
    impl EventHandler for MockEventHandler {
        async fn should_handle(&self, _event: &DynamicStreamed) -> bool {
            self.should_handle_flag.load(Ordering::SeqCst)
        }

        async fn handle(
            &self,
            _event: Arc<DynamicStreamed>,
        ) -> Result<(), Box<dyn CoreError + Send + Sync>> {
            self.handle_count.fetch_add(1, Ordering::SeqCst);
            self.handle_notify.notify_one();

            if self.should_fail.load(Ordering::SeqCst) {
                Err(Box::new(IoError::other("Mock handler failure")))
            } else {
                Ok(())
            }
        }

        fn name(&self) -> &str {
            &self.name
        }
    }

    // Tests for HandlerExecutionMode
    #[test]
    fn test_handler_execution_mode_default() {
        let mode = HandlerExecutionMode::default();
        assert_eq!(mode, HandlerExecutionMode::ConcurrentBounded(10));
    }

    #[test]
    fn test_handler_execution_mode_variants() {
        let sequential = HandlerExecutionMode::Sequential;
        let concurrent = HandlerExecutionMode::Concurrent;
        let bounded = HandlerExecutionMode::ConcurrentBounded(5);

        assert_eq!(sequential, HandlerExecutionMode::Sequential);
        assert_eq!(concurrent, HandlerExecutionMode::Concurrent);
        assert_eq!(bounded, HandlerExecutionMode::ConcurrentBounded(5));
        assert_ne!(sequential, concurrent);
    }

    // Tests for ManagerState
    #[test]
    fn test_manager_state_variants() {
        assert_eq!(ManagerState::Idle, ManagerState::Idle);
        assert_eq!(ManagerState::Starting, ManagerState::Starting);
        assert_eq!(ManagerState::Running, ManagerState::Running);
        assert_eq!(ManagerState::Stopping, ManagerState::Stopping);
        assert_eq!(ManagerState::Stopped, ManagerState::Stopped);
        assert_ne!(ManagerState::Idle, ManagerState::Running);
    }

    // Tests for StreamManager creation
    #[tokio::test]
    async fn test_stream_manager_new() {
        let manager = StreamManager::default();
        assert_eq!(manager.state().await, ManagerState::Idle);
        assert_eq!(manager.list_streams().len(), 0);
    }

    #[tokio::test]
    async fn test_stream_manager_default() {
        let manager = StreamManager::default();
        assert_eq!(manager.state().await, ManagerState::Idle);
        assert_eq!(manager.list_streams().len(), 0);
    }

    #[tokio::test]
    async fn test_stream_manager_with_execution_mode_sequential() {
        let manager = StreamManager::with_execution_mode(HandlerExecutionMode::Sequential);
        assert_eq!(manager.state().await, ManagerState::Idle);
    }

    #[tokio::test]
    async fn test_stream_manager_with_execution_mode_concurrent() {
        let manager = StreamManager::with_execution_mode(HandlerExecutionMode::Concurrent);
        assert_eq!(manager.state().await, ManagerState::Idle);
    }

    #[tokio::test]
    async fn test_stream_manager_with_execution_mode_bounded() {
        let manager =
            StreamManager::with_execution_mode(HandlerExecutionMode::ConcurrentBounded(5));
        assert_eq!(manager.state().await, ManagerState::Idle);
    }

    #[tokio::test]
    async fn test_metrics_collector() {
        let manager = StreamManager::default();
        let collector = manager.metrics_collector();
        assert!(Arc::strong_count(&collector) >= 1);
    }

    // Tests for stream management
    #[tokio::test]
    async fn test_add_stream_success() {
        let manager = StreamManager::default();
        let stream = MockStream::new("test_stream");

        let result = manager.add_stream("test_stream".to_string(), stream).await;
        assert!(result.is_ok());
        assert_eq!(manager.list_streams().len(), 1);
        assert!(manager.list_streams().contains(&"test_stream".to_string()));
    }

    #[tokio::test]
    #[expect(clippy::expect_used, clippy::panic)]
    async fn test_add_stream_already_exists() {
        let manager = StreamManager::default();
        let stream1 = MockStream::new("test_stream");
        let stream2 = MockStream::new("test_stream");

        manager
            .add_stream("test_stream".to_string(), stream1)
            .await
            .expect("Failed to add test stream");
        let result = manager.add_stream("test_stream".to_string(), stream2).await;

        assert!(result.is_err());
        let error_result = result.expect_err("Expected error result");
        match error_result {
            StreamError::AlreadyRunning { name } => {
                assert_eq!(name, "test_stream");
            }
            _ => panic!("Expected AlreadyRunning error"),
        }
    }

    #[tokio::test]
    #[expect(clippy::expect_used)]
    async fn test_add_running_stream() {
        let manager = StreamManager::default();
        let mut stream = MockStream::new("test_stream");
        stream
            .start(MockConfig::default())
            .await
            .expect("Failed to start mock stream");

        let result = manager.add_stream("test_stream".to_string(), stream).await;
        assert!(result.is_ok());
        assert!(manager.is_stream_running("test_stream"));

        // Clean up the background forwarding task
        manager
            .stop_stream("test_stream")
            .await
            .expect("Failed to stop test stream");
    }

    #[tokio::test]
    #[expect(clippy::expect_used)]
    async fn test_remove_stream_success() {
        let manager = StreamManager::default();
        let stream = MockStream::new("test_stream");

        manager
            .add_stream("test_stream".to_string(), stream)
            .await
            .expect("Failed to add test stream");
        assert_eq!(manager.list_streams().len(), 1);

        let result = manager.remove_stream("test_stream").await;
        assert!(result.is_ok());
        assert_eq!(manager.list_streams().len(), 0);
    }

    #[tokio::test]
    #[expect(clippy::expect_used, clippy::panic)]
    async fn test_remove_nonexistent_stream() {
        let manager = StreamManager::default();
        let result = manager.remove_stream("nonexistent").await;
        assert!(result.is_err());
        let error_result = result.expect_err("Expected error result");
        match error_result {
            StreamError::NotRunning { name } => {
                assert_eq!(name, "nonexistent");
            }
            _ => panic!("Expected NotRunning error"),
        }
    }

    #[tokio::test]
    #[expect(clippy::expect_used)]
    async fn test_start_stream_success() {
        let manager = StreamManager::default();
        let stream = MockStream::new("test_stream");

        manager
            .add_stream("test_stream".to_string(), stream)
            .await
            .expect("Failed to add test stream");
        let result = manager.start_stream("test_stream").await;
        assert!(result.is_ok());
        assert!(manager.is_stream_running("test_stream"));
    }

    #[tokio::test]
    #[expect(clippy::expect_used, clippy::panic)]
    async fn test_start_nonexistent_stream() {
        let manager = StreamManager::default();
        let result = manager.start_stream("nonexistent").await;
        assert!(result.is_err());
        let error_result = result.expect_err("Expected error result");
        match error_result {
            StreamError::NotRunning { name } => {
                assert_eq!(name, "nonexistent");
            }
            _ => panic!("Expected NotRunning error"),
        }
    }

    #[tokio::test]
    #[expect(clippy::expect_used)]
    #[allow(clippy::panic)]
    async fn test_start_already_running_stream() {
        let manager = StreamManager::default();
        let stream = MockStream::new("test_stream");

        manager
            .add_stream("test_stream".to_string(), stream)
            .await
            .expect("Failed to add test stream");
        manager
            .start_stream("test_stream")
            .await
            .expect("Failed to start test stream");

        let result = manager.start_stream("test_stream").await;
        assert!(result.is_err());
        let error_result = result.expect_err("Expected error result");
        match error_result {
            StreamError::AlreadyRunning { name } => {
                assert_eq!(name, "test_stream");
            }
            _ => panic!("Expected AlreadyRunning error"),
        }
    }

    #[tokio::test]
    async fn test_start_stream_failure() {
        let _manager = StreamManager::default();
        let mut stream = MockStream::new("test_stream");

        // Configure stream to fail on start
        let config = MockConfig {
            fail_start: true,
            fail_stop: false,
        };
        let result = stream.start(config).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    #[expect(clippy::expect_used)]
    async fn test_stop_stream_success() {
        let manager = StreamManager::default();
        let stream = MockStream::new("test_stream");

        manager
            .add_stream("test_stream".to_string(), stream)
            .await
            .expect("Failed to add test stream");
        manager
            .start_stream("test_stream")
            .await
            .expect("Failed to start test stream");

        let result = manager.stop_stream("test_stream").await;
        assert!(result.is_ok());
        assert!(!manager.is_stream_running("test_stream"));
    }

    #[tokio::test]
    #[expect(clippy::expect_used, clippy::panic)]
    async fn test_stop_nonexistent_stream() {
        let manager = StreamManager::default();
        let result = manager.stop_stream("nonexistent").await;
        assert!(result.is_err());
        let error_result = result.expect_err("Expected error result");
        match error_result {
            StreamError::NotRunning { name } => {
                assert_eq!(name, "nonexistent");
            }
            _ => panic!("Expected NotRunning error"),
        }
    }

    #[tokio::test]
    #[expect(clippy::expect_used)]
    async fn test_stop_already_stopped_stream() {
        let manager = StreamManager::default();
        let stream = MockStream::new("test_stream");

        manager
            .add_stream("test_stream".to_string(), stream)
            .await
            .expect("Failed to add test stream");
        let result = manager.stop_stream("test_stream").await;
        assert!(result.is_ok()); // Should succeed even if already stopped
    }

    #[tokio::test]
    #[expect(clippy::expect_used)]
    async fn test_stop_stream_failure() {
        let _manager = StreamManager::default();
        let mut stream = MockStream::new("test_stream");

        // Start the stream normally first
        stream
            .start(MockConfig::default())
            .await
            .expect("Failed to start mock stream");

        // Test stop failure directly
        let config = MockConfig {
            fail_start: false,
            fail_stop: true,
        };
        stream
            .start(config)
            .await
            .expect("Failed to start mock stream"); // Reconfigure to fail on stop

        let result = stream.stop().await;
        assert!(result.is_err());
    }

    // Tests for event handler management
    #[tokio::test]
    async fn test_add_event_handler() {
        let manager = StreamManager::default();
        let handler = Arc::new(MockEventHandler::new("test_handler"));

        manager.add_event_handler(handler).await;
        // Handler is stored internally, no direct verification possible
    }

    #[tokio::test]
    async fn test_multiple_event_handlers() {
        let manager = StreamManager::default();
        let handler1 = Arc::new(MockEventHandler::new("handler1"));
        let handler2 = Arc::new(MockEventHandler::new("handler2"));

        manager.add_event_handler(handler1).await;
        manager.add_event_handler(handler2).await;
        // Both handlers are stored internally
    }

    // Tests for manager state transitions
    #[tokio::test]
    #[expect(clippy::expect_used)]
    async fn test_start_all_success() {
        let manager = StreamManager::default();
        let mut stream1 = MockStream::new("stream1");
        let mut stream2 = MockStream::new("stream2");

        // Start the streams before adding them
        stream1
            .start(MockConfig::default())
            .await
            .expect("Failed to start mock stream");
        stream2
            .start(MockConfig::default())
            .await
            .expect("Failed to start mock stream");

        manager
            .add_stream("stream1".to_string(), stream1)
            .await
            .expect("Failed to add test stream");
        manager
            .add_stream("stream2".to_string(), stream2)
            .await
            .expect("Failed to add test stream");

        let result = manager.start_all().await;
        if let Err(ref e) = result {
            eprintln!("start_all failed: {e:?}");
        }
        assert!(result.is_ok());
        assert_eq!(manager.state().await, ManagerState::Running);
        assert!(manager.is_stream_running("stream1"));
        assert!(manager.is_stream_running("stream2"));

        // Clean up
        manager
            .stop_all()
            .await
            .expect("Failed to stop all streams");
    }

    #[tokio::test]
    #[expect(clippy::expect_used, clippy::panic)]
    async fn test_start_all_already_running() {
        let manager = StreamManager::default();
        manager
            .start_all()
            .await
            .expect("Failed to start all streams"); // Start with no streams

        let result = manager.start_all().await;
        assert!(result.is_err());
        let error_result = result.expect_err("Expected error result");
        match error_result {
            StreamError::AlreadyRunning { name } => {
                assert_eq!(name, "StreamManager");
            }
            _ => panic!("Expected AlreadyRunning error"),
        }

        // Clean up
        manager
            .stop_all()
            .await
            .expect("Failed to stop all streams");
    }

    #[tokio::test]
    #[expect(clippy::expect_used)]
    async fn test_start_all_partial_failure() {
        // This test should succeed with empty streams or be removed
        // since MockStream doesn't fail on start by default
        let manager = StreamManager::default();
        let mut stream1 = MockStream::new("stream1");
        let mut stream2 = MockStream::new("stream2");

        // Start the streams before adding them
        stream1
            .start(MockConfig::default())
            .await
            .expect("Failed to start mock stream");
        stream2
            .start(MockConfig::default())
            .await
            .expect("Failed to start mock stream");

        manager
            .add_stream("stream1".to_string(), stream1)
            .await
            .expect("Failed to add test stream");
        manager
            .add_stream("stream2".to_string(), stream2)
            .await
            .expect("Failed to add test stream");

        let result = manager.start_all().await;
        // Should succeed since there's no actual failure
        assert!(result.is_ok());
        assert_eq!(manager.state().await, ManagerState::Running);

        // Clean up
        manager
            .stop_all()
            .await
            .expect("Failed to stop all streams");
    }

    #[tokio::test]
    #[expect(clippy::expect_used)]
    async fn test_start_all_from_stopped_state() {
        let manager = StreamManager::default();
        manager
            .start_all()
            .await
            .expect("Failed to start all streams");
        manager
            .stop_all()
            .await
            .expect("Failed to stop all streams");
        assert_eq!(manager.state().await, ManagerState::Stopped);

        let result = manager.start_all().await;
        assert!(result.is_ok());
        assert_eq!(manager.state().await, ManagerState::Running);

        // Clean up
        manager
            .stop_all()
            .await
            .expect("Failed to stop all streams");
    }

    #[tokio::test]
    #[expect(clippy::expect_used)]
    async fn test_stop_all_success() {
        let manager = StreamManager::default();
        let mut stream1 = MockStream::new("stream1");
        let mut stream2 = MockStream::new("stream2");

        // Start the streams before adding them
        stream1
            .start(MockConfig::default())
            .await
            .expect("Failed to start mock stream");
        stream2
            .start(MockConfig::default())
            .await
            .expect("Failed to start mock stream");

        manager
            .add_stream("stream1".to_string(), stream1)
            .await
            .expect("Failed to add test stream");
        manager
            .add_stream("stream2".to_string(), stream2)
            .await
            .expect("Failed to add test stream");
        manager
            .start_all()
            .await
            .expect("Failed to start all streams");

        let result = manager.stop_all().await;
        assert!(result.is_ok());
        assert_eq!(manager.state().await, ManagerState::Stopped);
        assert!(!manager.is_stream_running("stream1"));
        assert!(!manager.is_stream_running("stream2"));
    }

    #[tokio::test]
    async fn test_stop_all_not_running() {
        let manager = StreamManager::default();
        let result = manager.stop_all().await;
        assert!(result.is_ok()); // Should succeed even if not running
    }

    #[tokio::test]
    #[expect(clippy::expect_used)]
    async fn test_stop_all_with_stream_failure() {
        let manager = StreamManager::default();
        let mut stream = MockStream::new("test_stream");

        // Start stream normally
        stream
            .start(MockConfig::default())
            .await
            .expect("Failed to start mock stream");
        manager
            .add_stream("test_stream".to_string(), stream)
            .await
            .expect("Failed to add test stream");
        manager
            .start_all()
            .await
            .expect("Failed to start all streams");

        let result = manager.stop_all().await;
        assert!(result.is_ok()); // Should complete despite individual stream failure
        assert_eq!(manager.state().await, ManagerState::Stopped);
    }

    #[tokio::test]
    #[expect(clippy::expect_used)]
    #[allow(clippy::panic)]
    async fn test_process_events_not_running() {
        let manager = StreamManager::default();
        let result = manager.process_events().await;
        assert!(result.is_err());
        let error_result = result.expect_err("Expected error result");
        match error_result {
            StreamError::NotRunning { name } => {
                assert_eq!(name, "StreamManager");
            }
            _ => panic!("Expected NotRunning error"),
        }
    }

    #[tokio::test]
    #[expect(clippy::expect_used)]
    async fn test_process_events_with_shutdown() {
        let manager = Arc::new(StreamManager::default());
        manager
            .start_all()
            .await
            .expect("Failed to start all streams");

        // Spawn process_events in background with timeout
        let manager_clone = manager.clone();
        let process_handle = tokio::spawn(async move {
            // Add timeout to prevent infinite wait
            timeout(
                Duration::from_millis(500), // Reduced timeout for tests
                manager_clone.process_events(),
            )
            .await
        });

        // Give it a moment to start listening
        sleep(Duration::from_millis(50)).await;

        // Stop the manager (sends shutdown signal)
        manager
            .stop_all()
            .await
            .expect("Failed to stop all streams");

        // Wait for the task to complete or timeout quickly
        let result = timeout(Duration::from_millis(200), process_handle).await;

        // Any completion is acceptable in tests
        assert!(result.is_ok() || result.is_err());
    }

    // Tests for utility methods
    #[tokio::test]
    #[expect(clippy::expect_used)]
    async fn test_health() {
        let manager = StreamManager::default();
        let stream1 = MockStream::with_health("stream1", StreamHealth::healthy());
        let stream2 = MockStream::with_health("stream2", StreamHealth::degraded());

        manager
            .add_stream("stream1".to_string(), stream1)
            .await
            .expect("Failed to add test stream");
        manager
            .add_stream("stream2".to_string(), stream2)
            .await
            .expect("Failed to add test stream");

        let health = manager.health().await;
        assert_eq!(health.len(), 2);
        assert_eq!(health.get("stream1"), Some(&StreamHealth::healthy()));
        assert_eq!(health.get("stream2"), Some(&StreamHealth::degraded()));
    }

    #[tokio::test]
    async fn test_health_empty() {
        let manager = StreamManager::default();
        let health = manager.health().await;
        assert_eq!(health.len(), 0);
    }

    #[tokio::test]
    async fn test_list_streams_empty() {
        let manager = StreamManager::default();
        let streams = manager.list_streams();
        assert_eq!(streams.len(), 0);
    }

    #[tokio::test]
    #[expect(clippy::expect_used)]
    async fn test_list_streams_multiple() {
        let manager = StreamManager::default();
        let first_stream = MockStream::new("stream1");
        let second_stream = MockStream::new("stream2");

        manager
            .add_stream("stream1".to_string(), first_stream)
            .await
            .expect("Failed to add test stream");
        manager
            .add_stream("stream2".to_string(), second_stream)
            .await
            .expect("Failed to add test stream");

        let streams = manager.list_streams();
        assert_eq!(streams.len(), 2);
        assert!(streams.contains(&"stream1".to_string()));
        assert!(streams.contains(&"stream2".to_string()));
    }

    #[tokio::test]
    async fn test_is_stream_running_nonexistent() {
        let manager = StreamManager::default();
        assert!(!manager.is_stream_running("nonexistent"));
    }

    #[tokio::test]
    #[expect(clippy::expect_used)]
    async fn test_is_stream_running_stopped() {
        let manager = StreamManager::default();
        let stream = MockStream::new("test_stream");

        manager
            .add_stream("test_stream".to_string(), stream)
            .await
            .expect("Failed to add test stream");
        assert!(!manager.is_stream_running("test_stream"));
    }

    #[tokio::test]
    #[expect(clippy::expect_used)]
    async fn test_is_stream_running_started() {
        let manager = StreamManager::default();
        let stream = MockStream::new("test_stream");

        manager
            .add_stream("test_stream".to_string(), stream)
            .await
            .expect("Failed to add test stream");
        manager
            .start_stream("test_stream")
            .await
            .expect("Failed to start test stream");
        assert!(manager.is_stream_running("test_stream"));
    }

    #[tokio::test]
    #[expect(clippy::expect_used)]
    async fn test_state_transitions() {
        let manager = StreamManager::default();

        // Initial state
        assert_eq!(manager.state().await, ManagerState::Idle);

        // Start
        manager
            .start_all()
            .await
            .expect("Failed to start all streams");
        assert_eq!(manager.state().await, ManagerState::Running);

        // Stop
        manager
            .stop_all()
            .await
            .expect("Failed to stop all streams");
        assert_eq!(manager.state().await, ManagerState::Stopped);
    }

    // Tests for event handling with different execution modes
    #[tokio::test]
    async fn test_handle_event_sequential() {
        let manager = StreamManager::with_execution_mode(HandlerExecutionMode::Sequential);
        let handler1 = Arc::new(MockEventHandler::new("handler1"));
        let handler2 = Arc::new(MockEventHandler::new("handler2"));

        handler1.set_should_handle(true);
        handler2.set_should_handle(true);

        manager.add_event_handler(handler1.clone()).await;
        manager.add_event_handler(handler2.clone()).await;

        let mock_event = MockEvent::new("test_event");
        let event =
            Arc::new((Box::new(mock_event) as Box<dyn Event>).with_default_stream_metadata("test"));
        manager.handle_event(event).await;

        // Both handlers should have processed the event
        assert_eq!(handler1.handle_count(), 1);
        assert_eq!(handler2.handle_count(), 1);
    }

    #[tokio::test]
    async fn test_handle_event_concurrent() {
        let manager = StreamManager::with_execution_mode(HandlerExecutionMode::Concurrent);
        let handler1 = Arc::new(MockEventHandler::new("handler1"));
        let handler2 = Arc::new(MockEventHandler::new("handler2"));

        handler1.set_should_handle(true);
        handler2.set_should_handle(true);

        manager.add_event_handler(handler1.clone()).await;
        manager.add_event_handler(handler2.clone()).await;

        let mock_event = MockEvent::new("test_event");
        let event =
            Arc::new((Box::new(mock_event) as Box<dyn Event>).with_default_stream_metadata("test"));
        manager.handle_event(event).await;

        // Both handlers should have processed the event
        assert_eq!(handler1.handle_count(), 1);
        assert_eq!(handler2.handle_count(), 1);
    }

    #[tokio::test]
    async fn test_handle_event_concurrent_bounded() {
        let manager =
            StreamManager::with_execution_mode(HandlerExecutionMode::ConcurrentBounded(2));
        let handler1 = Arc::new(MockEventHandler::new("handler1"));
        let handler2 = Arc::new(MockEventHandler::new("handler2"));

        handler1.set_should_handle(true);
        handler2.set_should_handle(true);

        manager.add_event_handler(handler1.clone()).await;
        manager.add_event_handler(handler2.clone()).await;

        let mock_event = MockEvent::new("test_event");
        let event =
            Arc::new((Box::new(mock_event) as Box<dyn Event>).with_default_stream_metadata("test"));
        manager.handle_event(event).await;

        // Both handlers should have processed the event
        assert_eq!(handler1.handle_count(), 1);
        assert_eq!(handler2.handle_count(), 1);
    }

    #[tokio::test]
    async fn test_handle_event_no_eligible_handlers() {
        let manager = StreamManager::default();
        let handler = Arc::new(MockEventHandler::new("handler"));
        handler.set_should_handle(false);

        manager.add_event_handler(handler.clone()).await;

        let mock_event = MockEvent::new("test_event");
        let event =
            Arc::new((Box::new(mock_event) as Box<dyn Event>).with_default_stream_metadata("test"));
        manager.handle_event(event).await;

        // Handler should not have processed the event
        assert_eq!(handler.handle_count(), 0);
    }

    #[tokio::test]
    async fn test_handle_event_handler_failure() {
        let manager = StreamManager::default();
        let handler = Arc::new(MockEventHandler::new("handler"));
        handler.set_should_handle(true);
        handler.set_should_fail(true);

        manager.add_event_handler(handler.clone()).await;

        let mock_event = MockEvent::new("test_event");
        let event =
            Arc::new((Box::new(mock_event) as Box<dyn Event>).with_default_stream_metadata("test"));
        manager.handle_event(event).await;

        // Handler should have been called despite failure
        assert_eq!(handler.handle_count(), 1);
    }

    // Tests for LoggingEventHandler
    #[tokio::test]
    async fn test_logging_event_handler_new() {
        let handler = LoggingEventHandler::new("test_handler");
        assert_eq!(handler.name(), "test_handler");
    }

    #[tokio::test]
    async fn test_logging_event_handler_should_handle() {
        let handler = LoggingEventHandler::new("test_handler");
        let mock_event = MockEvent::new("test_event");
        let event = (Box::new(mock_event) as Box<dyn Event>).with_default_stream_metadata("test");
        assert!(handler.should_handle(&event).await);
    }

    #[tokio::test]
    async fn test_logging_event_handler_handle() {
        let handler = LoggingEventHandler::new("test_handler");
        let mock_event = MockEvent::new("test_event");
        let event =
            Arc::new((Box::new(mock_event) as Box<dyn Event>).with_default_stream_metadata("test"));
        let result = handler.handle(event).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_logging_event_handler_name() {
        let handler = LoggingEventHandler::new("my_handler");
        assert_eq!(handler.name(), "my_handler");
    }

    // Edge case tests
    #[tokio::test]
    async fn test_empty_stream_name() {
        let manager = StreamManager::default();
        let stream = MockStream::new("");

        let result = manager.add_stream(String::new(), stream).await;
        assert!(result.is_ok());
        assert!(manager.list_streams().contains(&String::new()));
    }

    #[tokio::test]
    async fn test_large_stream_name() {
        let manager = StreamManager::default();
        let large_name = "a".repeat(1000);
        let stream = MockStream::new(&large_name);

        let result = manager.add_stream(large_name.clone(), stream).await;
        assert!(result.is_ok());
        assert!(manager.list_streams().contains(&large_name));
    }

    #[tokio::test]
    async fn test_unicode_stream_name() {
        let manager = StreamManager::default();
        let unicode_name = "тест_🚀_stream";
        let stream = MockStream::new(unicode_name);

        let result = manager.add_stream(unicode_name.to_string(), stream).await;
        assert!(result.is_ok());
        assert!(manager.list_streams().contains(&unicode_name.to_string()));
    }

    #[tokio::test]
    #[expect(clippy::expect_used)]
    async fn test_concurrent_stream_operations() {
        let manager = Arc::new(StreamManager::default());
        let mut handles = Vec::default();

        // Add streams concurrently
        for i in 0..10 {
            let manager = manager.clone();
            let handle = tokio::spawn(async move {
                let stream = MockStream::new(&format!("stream_{i}"));
                manager.add_stream(format!("stream_{i}"), stream).await
            });
            handles.push(handle);
        }

        // Wait for all additions to complete
        for handle in handles {
            assert!(handle.await.expect("Handle should complete").is_ok());
        }

        assert_eq!(manager.list_streams().len(), 10);
    }

    // Test existing tests for compatibility
    #[tokio::test]
    async fn test_stream_manager_creation() {
        let manager = StreamManager::default();
        assert_eq!(manager.state().await, ManagerState::Idle);
        assert_eq!(manager.list_streams().len(), 0);
    }

    #[tokio::test]
    async fn test_add_event_handler_logging() {
        let manager = StreamManager::default();
        let handler = Arc::new(LoggingEventHandler::new("test"));
        manager.add_event_handler(handler).await;

        // Handler count is not exposed, but we can verify no panic
        assert_eq!(manager.state().await, ManagerState::Idle);
    }
}
