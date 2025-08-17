//! Advanced connection management with circuit breaker and reconnection strategies
//!
//! This module provides production-grade connection handling including:
//! - Automatic reconnection with exponential backoff
//! - Circuit breaker pattern for failing connections
//! - Health monitoring and failover capabilities
//! - Connection pooling for multiple simultaneous streams

use crate::core::config::ConnectionConfig;
use crate::core::error::{StreamError, StreamResult};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, RwLock};
use tokio::time::{sleep, timeout};
use tracing::{debug, error, info, warn};

/// Connection state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    /// Connection is healthy and active
    Connected,
    /// Connection is being established
    Connecting,
    /// Connection failed, attempting to reconnect
    Reconnecting,
    /// Connection is permanently failed (circuit breaker open)
    Failed,
    /// Connection is explicitly disconnected
    Disconnected,
}

/// Connection health metrics
#[derive(Debug, Clone)]
pub struct ConnectionHealth {
    /// Current state of the connection
    pub state: ConnectionState,
    /// Timestamp when the connection was established
    pub connected_at: Option<Instant>,
    /// Timestamp of the last activity on this connection
    pub last_activity: Option<Instant>,
    /// Total number of reconnection attempts made
    pub total_reconnects: usize,
    /// Number of consecutive failures experienced
    pub consecutive_failures: usize,
    /// Current latency in milliseconds, if available
    pub latency_ms: Option<u64>,
}

/// Circuit breaker for connection management
#[derive(Debug)]
pub struct CircuitBreaker {
    /// Configuration for connection behavior
    config: ConnectionConfig,
    /// Current atomic state of the circuit breaker
    state: AtomicConnectionState,
    /// Atomic counter for consecutive failures
    failure_count: AtomicUsize,
    /// Timestamp of the last connection failure
    last_failure: Mutex<Option<Instant>>,
    /// Timestamp of the last successful connection
    last_success: Mutex<Option<Instant>>,
}

#[derive(Debug)]
struct AtomicConnectionState {
    state: AtomicUsize,
}

impl AtomicConnectionState {
    /// Creates a new atomic connection state with the given initial state
    fn new(state: ConnectionState) -> Self {
        Self {
            state: AtomicUsize::new(state as usize),
        }
    }

    /// Loads the current connection state atomically
    fn load(&self) -> ConnectionState {
        match self.state.load(Ordering::Acquire) {
            0 => ConnectionState::Connected,
            1 => ConnectionState::Connecting,
            2 => ConnectionState::Reconnecting,
            3 => ConnectionState::Failed,
            4 => ConnectionState::Disconnected,
            _ => ConnectionState::Failed,
        }
    }

    /// Stores a new connection state atomically
    fn store(&self, state: ConnectionState) {
        self.state.store(state as usize, Ordering::Release);
    }

    /// Atomically compares and exchanges the connection state
    fn compare_exchange(
        &self,
        current: ConnectionState,
        new: ConnectionState,
    ) -> Result<ConnectionState, ConnectionState> {
        match self.state.compare_exchange(
            current as usize,
            new as usize,
            Ordering::AcqRel,
            Ordering::Acquire,
        ) {
            Ok(_) => Ok(current),
            Err(actual) => Err(match actual {
                0 => ConnectionState::Connected,
                1 => ConnectionState::Connecting,
                2 => ConnectionState::Reconnecting,
                3 => ConnectionState::Failed,
                4 => ConnectionState::Disconnected,
                _ => ConnectionState::Failed,
            }),
        }
    }
}

impl CircuitBreaker {
    /// Creates a new circuit breaker with the given configuration
    pub fn new(config: ConnectionConfig) -> Self {
        Self {
            config,
            state: AtomicConnectionState::new(ConnectionState::Disconnected),
            failure_count: AtomicUsize::new(0),
            last_failure: Mutex::new(None),
            last_success: Mutex::new(None),
        }
    }

    /// Returns the current state of the circuit breaker
    pub fn state(&self) -> ConnectionState {
        self.state.load()
    }

    /// Attempts to establish a connection using the provided function
    ///
    /// Respects circuit breaker state and implements backoff logic
    pub async fn attempt_connect<F, Fut, T>(&self, connect_fn: F) -> StreamResult<T>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = StreamResult<T>>,
    {
        let current_state = self.state();

        // Check if circuit breaker is open
        if current_state == ConnectionState::Failed {
            let last_failure = *self.last_failure.lock().await;
            if let Some(last_fail_time) = last_failure {
                let elapsed = last_fail_time.elapsed();
                let cooldown =
                    self.calculate_backoff_delay(self.failure_count.load(Ordering::Acquire));

                if elapsed < cooldown {
                    return Err(StreamError::retriable_connection(format!(
                        "Circuit breaker open, retrying in {:?}",
                        cooldown - elapsed
                    )));
                }
            }
        }

        // Transition to connecting state
        if self
            .state
            .compare_exchange(current_state, ConnectionState::Connecting)
            .is_err()
        {
            return Err(StreamError::permanent_connection(
                "Connection attempt already in progress",
            ));
        }

        let connect_timeout = self.config.connect_timeout();
        let result = timeout(connect_timeout, connect_fn()).await;

        match result {
            Ok(Ok(connection)) => {
                self.on_success().await;
                Ok(connection)
            }
            Ok(Err(e)) => {
                self.on_failure().await;
                Err(e)
            }
            Err(_) => {
                self.on_failure().await;
                Err(StreamError::retriable_connection(format!(
                    "Connection timeout after {:?}",
                    connect_timeout
                )))
            }
        }
    }

    async fn on_success(&self) {
        self.state.store(ConnectionState::Connected);
        self.failure_count.store(0, Ordering::Release);
        *self.last_success.lock().await = Some(Instant::now());
        debug!("Connection established successfully");
    }

    async fn on_failure(&self) {
        let failures = self.failure_count.fetch_add(1, Ordering::AcqRel) + 1;
        *self.last_failure.lock().await = Some(Instant::now());

        if failures >= self.config.max_retries {
            self.state.store(ConnectionState::Failed);
            error!("Circuit breaker opened after {} failures", failures);
        } else {
            self.state.store(ConnectionState::Reconnecting);
            warn!(
                "Connection failed ({}/{}), will retry",
                failures, self.config.max_retries
            );
        }
    }

    /// Marks the connection as explicitly disconnected
    pub fn mark_disconnected(&self) {
        self.state.store(ConnectionState::Disconnected);
    }

    fn calculate_backoff_delay(&self, attempt: usize) -> Duration {
        let base_delay = self.config.retry_base_delay();
        let max_delay = self.config.retry_max_delay();

        let multiplier = 1u64 << attempt.min(10); // Cap at 2^10
        let exponential_delay = Duration::from_millis(base_delay.as_millis() as u64 * multiplier);
        std::cmp::min(exponential_delay, max_delay)
    }
}

/// Connection manager with automatic reconnection
pub struct ConnectionManager<T> {
    /// Circuit breaker for connection failure handling
    circuit_breaker: Arc<CircuitBreaker>,
    /// The managed connection, if active
    connection: Arc<RwLock<Option<T>>>,
    /// Health metrics and status of the connection
    health: Arc<RwLock<ConnectionHealth>>,
    /// Flag indicating if reconnection monitoring is active
    reconnect_task: Arc<AtomicBool>,
}

impl<T> ConnectionManager<T>
where
    T: Send + Sync + 'static,
{
    /// Creates a new connection manager with the given configuration
    pub fn new(config: ConnectionConfig) -> Self {
        Self {
            circuit_breaker: Arc::new(CircuitBreaker::new(config)),
            connection: Arc::new(RwLock::new(None)),
            health: Arc::new(RwLock::new(ConnectionHealth {
                state: ConnectionState::Disconnected,
                connected_at: None,
                last_activity: None,
                total_reconnects: 0,
                consecutive_failures: 0,
                latency_ms: None,
            })),
            reconnect_task: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Establishes a connection using the provided function
    ///
    /// Starts background monitoring for automatic reconnection
    pub async fn connect<F, Fut>(&self, connect_fn: F) -> StreamResult<()>
    where
        F: Fn() -> Fut + Send + Sync + 'static + Clone,
        Fut: std::future::Future<Output = StreamResult<T>> + Send,
    {
        let connection = self
            .circuit_breaker
            .attempt_connect(connect_fn.clone())
            .await?;

        *self.connection.write().await = Some(connection);
        self.update_health_on_connect().await;

        // Start background reconnection monitoring
        self.start_reconnect_monitor(connect_fn).await;

        Ok(())
    }

    /// Returns a clone of the current connection, if available
    pub async fn get_connection(&self) -> Option<T>
    where
        T: Clone,
    {
        let connection_guard = self.connection.read().await;
        connection_guard.as_ref().cloned()
    }

    /// Executes a function with the current connection
    ///
    /// Updates activity tracking when the connection is accessed
    pub async fn with_connection<F, R>(&self, f: F) -> StreamResult<R>
    where
        F: FnOnce(&T) -> R,
        T: Clone,
    {
        let connection = self
            .get_connection()
            .await
            .ok_or_else(|| StreamError::permanent_connection("No active connection"))?;

        self.update_activity().await;
        Ok(f(&connection))
    }

    /// Returns the current health status of the connection
    pub async fn health(&self) -> ConnectionHealth {
        self.health.read().await.clone()
    }

    /// Disconnects and stops monitoring the connection
    pub async fn disconnect(&self) {
        *self.connection.write().await = None;
        self.circuit_breaker.mark_disconnected();
        self.reconnect_task.store(false, Ordering::Release);

        let mut health = self.health.write().await;
        health.state = ConnectionState::Disconnected;
        health.connected_at = None;
    }

    async fn update_health_on_connect(&self) {
        let mut health = self.health.write().await;
        let now = Instant::now();

        health.state = ConnectionState::Connected;
        health.connected_at = Some(now);
        health.last_activity = Some(now);
        health.total_reconnects += 1;
        health.consecutive_failures = 0;
    }

    async fn update_activity(&self) {
        let mut health = self.health.write().await;
        health.last_activity = Some(Instant::now());
    }

    async fn start_reconnect_monitor<F, Fut>(&self, connect_fn: F)
    where
        F: Fn() -> Fut + Send + Sync + 'static + Clone,
        Fut: std::future::Future<Output = StreamResult<T>> + Send,
    {
        if self
            .reconnect_task
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
        {
            return; // Monitor already running
        }

        let circuit_breaker = Arc::clone(&self.circuit_breaker);
        let connection = Arc::clone(&self.connection);
        let health = Arc::clone(&self.health);
        let reconnect_task = Arc::clone(&self.reconnect_task);

        tokio::spawn(async move {
            while reconnect_task.load(Ordering::Acquire) {
                sleep(Duration::from_secs(5)).await;

                let state = circuit_breaker.state();
                if state == ConnectionState::Reconnecting || state == ConnectionState::Failed {
                    info!("Attempting automatic reconnection...");

                    match circuit_breaker.attempt_connect(connect_fn.clone()).await {
                        Ok(new_connection) => {
                            *connection.write().await = Some(new_connection);

                            let mut health_guard = health.write().await;
                            let now = Instant::now();
                            health_guard.state = ConnectionState::Connected;
                            health_guard.connected_at = Some(now);
                            health_guard.last_activity = Some(now);
                            health_guard.total_reconnects += 1;
                            health_guard.consecutive_failures = 0;

                            info!("Automatic reconnection successful");
                        }
                        Err(e) => {
                            let mut health_guard = health.write().await;
                            health_guard.consecutive_failures += 1;

                            debug!("Reconnection attempt failed: {}", e);

                            // Wait before next attempt
                            let delay = circuit_breaker
                                .calculate_backoff_delay(health_guard.consecutive_failures);
                            sleep(delay).await;
                        }
                    }
                }
            }
        });
    }
}

/// Connection pool for managing multiple connections
#[allow(dead_code)]
pub struct ConnectionPool<T> {
    /// Collection of managed connections in the pool
    connections: Vec<ConnectionManager<T>>,
    /// Configuration applied to all connections in the pool
    config: ConnectionConfig,
    /// Current index for round-robin connection selection
    current_index: AtomicUsize,
}

impl<T> ConnectionPool<T>
where
    T: Send + Sync + 'static + Clone,
{
    /// Creates a new connection pool with the specified size and configuration
    pub fn new(config: ConnectionConfig, pool_size: usize) -> Self {
        let connections = (0..pool_size)
            .map(|_| ConnectionManager::new(config.clone()))
            .collect();

        Self {
            connections,
            config,
            current_index: AtomicUsize::new(0),
        }
    }

    /// Attempts to connect all connections in the pool concurrently
    pub async fn connect_all<F, Fut>(&self, connect_fn: F) -> StreamResult<()>
    where
        F: Fn() -> Fut + Send + Sync + 'static + Clone,
        Fut: std::future::Future<Output = StreamResult<T>> + Send,
    {
        let mut tasks = Vec::new();

        for manager in &self.connections {
            let connect_fn_clone = connect_fn.clone();
            let manager_ref = manager;

            tasks.push(async move { manager_ref.connect(connect_fn_clone).await });
        }

        let results = futures::future::join_all(tasks).await;

        for (index, result) in results.into_iter().enumerate() {
            if let Err(e) = result {
                warn!("Failed to connect to pool connection {}: {}", index, e);
            }
        }

        Ok(())
    }

    /// Returns a healthy connection from the pool using round-robin selection
    pub async fn get_healthy_connection(&self) -> StreamResult<T> {
        let start_index = self.current_index.load(Ordering::Acquire);

        for i in 0..self.connections.len() {
            let index = (start_index + i) % self.connections.len();
            let manager = &self.connections[index];

            let health = manager.health().await;
            if health.state == ConnectionState::Connected {
                if let Some(connection) = manager.get_connection().await {
                    // Update round-robin index
                    self.current_index
                        .store((index + 1) % self.connections.len(), Ordering::Release);
                    return Ok(connection);
                }
            }
        }

        Err(StreamError::retriable_connection(
            "No healthy connections available",
        ))
    }

    /// Returns health status for all connections in the pool
    pub async fn pool_health(&self) -> Vec<ConnectionHealth> {
        let mut health_reports = Vec::new();

        for manager in &self.connections {
            health_reports.push(manager.health().await);
        }

        health_reports
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone)]
    struct MockConnection {
        _id: u32,
    }

    async fn create_mock_connection() -> StreamResult<MockConnection> {
        Ok(MockConnection { _id: 42 })
    }

    async fn create_failing_connection() -> StreamResult<MockConnection> {
        Err(StreamError::Connection {
            message: "Mock failure".to_string(),
            retriable: true,
        })
    }

    #[tokio::test]
    async fn test_circuit_breaker_success() {
        let config = ConnectionConfig::default();
        let breaker = CircuitBreaker::new(config);

        let result = breaker.attempt_connect(create_mock_connection).await;
        assert!(result.is_ok());
        assert_eq!(breaker.state(), ConnectionState::Connected);
    }

    #[tokio::test]
    async fn test_circuit_breaker_failure() {
        let config = ConnectionConfig {
            max_retries: 1,
            ..ConnectionConfig::default()
        };
        let breaker = CircuitBreaker::new(config);

        let result = breaker.attempt_connect(create_failing_connection).await;
        assert!(result.is_err());
        assert_eq!(breaker.state(), ConnectionState::Reconnecting);
    }

    #[tokio::test]
    async fn test_connection_manager() {
        let config = ConnectionConfig::default();
        let manager = ConnectionManager::new(config);

        manager.connect(create_mock_connection).await.unwrap();

        let connection = manager.get_connection().await;
        assert!(connection.is_some());

        let health = manager.health().await;
        assert_eq!(health.state, ConnectionState::Connected);
        assert!(health.connected_at.is_some());
    }

    #[tokio::test]
    async fn test_connection_pool() {
        let config = ConnectionConfig::default();
        let pool = ConnectionPool::new(config, 3);

        pool.connect_all(create_mock_connection).await.unwrap();

        let connection = pool.get_healthy_connection().await;
        assert!(connection.is_ok());

        let health_reports = pool.pool_health().await;
        assert_eq!(health_reports.len(), 3);
    }
}
