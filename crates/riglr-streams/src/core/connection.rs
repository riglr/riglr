//! Advanced connection management with circuit breaker and reconnection strategies
//!
//! This module provides production-grade connection handling including:
//! - Automatic reconnection with exponential backoff
//! - Circuit breaker pattern for failing connections
//! - Health monitoring and failover capabilities
//! - Connection pooling for multiple simultaneous streams

use crate::core::config::Connection;
use crate::core::error::{StreamError, StreamResult};
use core::cmp::min;
use core::future::Future;
use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use core::time::Duration;
use futures::future::join_all;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::{Mutex, RwLock};
use tokio::time::{sleep, timeout};
use tracing::{debug, error, info, warn};

/// Connection state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[expect(clippy::module_name_repetitions)]
pub enum ConnectionState {
    /// Connection is healthy and active
    Connected,
    /// Connection is being established
    Connecting,
    /// Connection is explicitly disconnected
    Disconnected,
    /// Connection is permanently failed (circuit breaker open)
    Failed,
    /// Connection failed, attempting to reconnect
    Reconnecting,
}

/// Connection health metrics
#[derive(Debug, Clone)]
#[expect(clippy::module_name_repetitions)]
pub struct ConnectionHealth {
    /// Timestamp when the connection was established
    pub connected_at: Option<Instant>,
    /// Number of consecutive failures experienced
    pub consecutive_failures: usize,
    /// Timestamp of the last activity on this connection
    pub last_activity: Option<Instant>,
    /// Current latency in milliseconds, if available
    pub latency_ms: Option<u64>,
    /// Current state of the connection
    pub state: ConnectionState,
    /// Total number of reconnection attempts made
    pub total_reconnects: usize,
}

/// Circuit breaker for connection management
#[derive(Debug)]
pub struct CircuitBreaker {
    /// Configuration for connection behavior
    config: Connection,
    /// Atomic counter for consecutive failures
    failure_count: AtomicUsize,
    /// Timestamp of the last connection failure
    last_failure: Mutex<Option<Instant>>,
    /// Timestamp of the last successful connection
    last_success: Mutex<Option<Instant>>,
    /// Current atomic state of the circuit breaker
    state: AtomicConnectionState,
}

#[derive(Debug)]
struct AtomicConnectionState {
    state: AtomicUsize,
}

impl AtomicConnectionState {
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
            Err(actual) => {
                let state = match actual {
                    0 => ConnectionState::Connected,
                    1 => ConnectionState::Connecting,
                    2 => ConnectionState::Reconnecting,
                    4 => ConnectionState::Disconnected,
                    _ => ConnectionState::Failed,
                };
                Err(state)
            }
        }
    }

    /// Loads the current connection state atomically
    fn load(&self) -> ConnectionState {
        match self.state.load(Ordering::Acquire) {
            0 => ConnectionState::Connected,
            1 => ConnectionState::Connecting,
            2 => ConnectionState::Reconnecting,
            4 => ConnectionState::Disconnected,
            _ => ConnectionState::Failed,
        }
    }

    /// Creates a new atomic connection state with the given initial state
    const fn new(state: ConnectionState) -> Self {
        Self {
            state: AtomicUsize::new(state as usize),
        }
    }

    /// Stores a new connection state atomically
    fn store(&self, state: ConnectionState) {
        self.state.store(state as usize, Ordering::Release);
    }
}

impl CircuitBreaker {
    /// Attempts to establish a connection using the provided function
    ///
    /// Respects circuit breaker state and implements backoff logic
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Circuit breaker is open and hasn't cooled down
    /// - Connection function fails after retries
    /// - Connection timeout is exceeded
    pub async fn attempt_connect<F, Fut, T>(&self, connect_fn: F) -> StreamResult<T>
    where
        F: Fn() -> Fut,
        Fut: Future<Output = StreamResult<T>>,
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
                        cooldown.saturating_sub(elapsed)
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
                    "Connection timeout after {connect_timeout:?}"
                )))
            }
        }
    }

    fn calculate_backoff_delay(&self, attempt: usize) -> Duration {
        let base_delay = self.config.retry_base_delay();
        let max_delay = self.config.retry_max_delay();

        let multiplier = 1u64 << attempt.min(10); // Cap at 2^10
        let exponential_delay = Duration::from_millis(
            u64::try_from(base_delay.as_millis())
                .unwrap_or(u64::MAX)
                .saturating_mul(multiplier),
        );
        min(exponential_delay, max_delay)
    }

    /// Marks the connection as explicitly disconnected
    pub fn mark_disconnected(&self) {
        self.state.store(ConnectionState::Disconnected);
    }

    /// Creates a new circuit breaker with the given configuration
    #[must_use]
    pub fn new(config: Connection) -> Self {
        Self {
            config,
            failure_count: AtomicUsize::new(0),
            last_failure: Mutex::new(None),
            last_success: Mutex::new(None),
            state: AtomicConnectionState::new(ConnectionState::Disconnected),
        }
    }

    async fn on_failure(&self) {
        let failures = self
            .failure_count
            .fetch_add(1, Ordering::AcqRel)
            .saturating_add(1);
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

    async fn on_success(&self) {
        self.state.store(ConnectionState::Connected);
        self.failure_count.store(0, Ordering::Release);
        *self.last_success.lock().await = Some(Instant::now());
        debug!("Connection established successfully");
    }

    /// Returns the current state of the circuit breaker
    pub fn state(&self) -> ConnectionState {
        self.state.load()
    }
}

/// Connection manager with automatic reconnection
#[derive(Debug)]
#[expect(clippy::module_name_repetitions)]
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
    /// Establishes a connection using the provided function
    ///
    /// Starts background monitoring for automatic reconnection
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Initial connection establishment fails
    /// - Circuit breaker prevents connection
    /// - Background monitoring task fails to start
    pub async fn connect<F, Fut>(&self, connect_fn: F) -> StreamResult<()>
    where
        F: Fn() -> Fut + Send + Sync + 'static + Clone,
        Fut: Future<Output = StreamResult<T>> + Send,
    {
        let connection = self
            .circuit_breaker
            .attempt_connect(connect_fn.clone())
            .await?;

        *self.connection.write().await = Some(connection);
        self.update_health_on_connect().await;

        // Start background reconnection monitoring
        self.start_reconnect_monitor(connect_fn);

        Ok(())
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

    /// Returns a clone of the current connection, if available
    pub async fn get_connection(&self) -> Option<T>
    where
        T: Clone,
    {
        let connection_guard = self.connection.read().await;
        connection_guard.as_ref().cloned()
    }

    /// Returns the current health status of the connection
    pub async fn health(&self) -> ConnectionHealth {
        self.health.read().await.clone()
    }

    /// Creates a new connection manager with the given configuration
    pub fn new(config: Connection) -> Self {
        Self {
            circuit_breaker: Arc::new(CircuitBreaker::new(config)),
            connection: Arc::new(RwLock::new(None)),
            health: Arc::new(RwLock::new(ConnectionHealth {
                connected_at: None,
                consecutive_failures: 0,
                last_activity: None,
                latency_ms: None,
                state: ConnectionState::Disconnected,
                total_reconnects: 0,
            })),
            reconnect_task: Arc::new(AtomicBool::new(false)),
        }
    }

    async fn update_health_on_connect(&self) {
        let mut health = self.health.write().await;
        let now = Instant::now();

        health.state = ConnectionState::Connected;
        health.connected_at = Some(now);
        health.last_activity = Some(now);
        health.total_reconnects = health.total_reconnects.saturating_add(1);
        health.consecutive_failures = 0;
    }

    async fn update_activity(&self) {
        let mut health = self.health.write().await;
        health.last_activity = Some(Instant::now());
    }

    /// Executes a function with the current connection
    ///
    /// Updates activity tracking when the connection is accessed
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - No active connection is available
    /// - Connection is in a failed state
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

    fn start_reconnect_monitor<F, Fut>(&self, connect_fn: F)
    where
        F: Fn() -> Fut + Send + Sync + 'static + Clone,
        Fut: Future<Output = StreamResult<T>> + Send,
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

                    let connection_result =
                        circuit_breaker.attempt_connect(connect_fn.clone()).await;
                    match connection_result {
                        Ok(new_connection) => {
                            *connection.write().await = Some(new_connection);

                            let mut health_guard = health.write().await;
                            let now = Instant::now();
                            health_guard.state = ConnectionState::Connected;
                            health_guard.connected_at = Some(now);
                            health_guard.last_activity = Some(now);
                            health_guard.total_reconnects =
                                health_guard.total_reconnects.saturating_add(1);
                            health_guard.consecutive_failures = 0;
                            drop(health_guard);

                            info!("Automatic reconnection successful");
                        }
                        Err(e) => {
                            let mut health_guard = health.write().await;
                            health_guard.consecutive_failures =
                                health_guard.consecutive_failures.saturating_add(1);

                            debug!("Reconnection attempt failed: {}", e);

                            // Wait before next attempt
                            let delay = circuit_breaker
                                .calculate_backoff_delay(health_guard.consecutive_failures);
                            drop(health_guard);
                            let () = sleep(delay).await;
                        }
                    }
                }
            }
        });
    }
}

/// Connection pool for managing multiple connections
#[derive(Debug)]
#[expect(clippy::module_name_repetitions)]
pub struct ConnectionPool<T> {
    /// Configuration applied to all connections in the pool
    config: Connection,
    /// Collection of managed connections in the pool
    connections: Vec<ConnectionManager<T>>,
    /// Current index for round-robin connection selection
    current_index: AtomicUsize,
}

impl<T> ConnectionPool<T>
where
    T: Send + Sync + 'static + Clone,
{
    /// Attempts to connect all connections in the pool concurrently
    ///
    /// # Errors
    /// Returns an error if:
    /// - Any connection in the pool fails to establish
    /// - Connection timeout is reached
    /// - Circuit breaker is open
    /// - Connection configuration is invalid
    pub async fn connect_all<F, Fut>(&self, connect_fn: F) -> StreamResult<()>
    where
        F: Fn() -> Fut + Send + Sync + 'static + Clone,
        Fut: Future<Output = StreamResult<T>> + Send,
    {
        let mut tasks = Vec::new();

        for manager in &self.connections {
            let connect_fn_clone = connect_fn.clone();
            let manager_ref = manager;

            tasks.push(async move { manager_ref.connect(connect_fn_clone).await });
        }

        let results = join_all(tasks).await;

        for (index, result) in results.into_iter().enumerate() {
            if let Err(e) = result {
                warn!("Failed to connect to pool connection {}: {}", index, e);
            }
        }

        Ok(())
    }

    /// Returns a healthy connection from the pool using round-robin selection
    ///
    /// # Errors
    ///
    /// Returns error if no healthy connections are available in the pool
    ///
    /// # Panics
    ///
    /// Panics if the computed index is out of bounds of the connections vector.
    /// This should not happen in normal operation due to modulo operation.
    pub async fn get_healthy_connection(&self) -> StreamResult<T> {
        let start_index = self.current_index.load(Ordering::Acquire);

        for i in 0..self.connections.len() {
            #[expect(clippy::arithmetic_side_effects)]
            let index = start_index.saturating_add(i) % self.connections.len();
            let Some(manager) = self.connections.get(index) else {
                // This should not happen due to modulo operation, but handle it gracefully
                continue;
            };

            let health = manager.health().await;
            if health.state == ConnectionState::Connected {
                let connection_option = manager.get_connection().await;
                if let Some(connection) = connection_option {
                    // Update round-robin index
                    #[expect(clippy::arithmetic_side_effects)]
                    let next_index = index.saturating_add(1) % self.connections.len();
                    self.current_index.store(next_index, Ordering::Release);
                    return Ok(connection);
                }
            }
        }

        Err(StreamError::retriable_connection(
            "No healthy connections available",
        ))
    }

    /// Creates a new connection pool with the specified size and configuration
    #[must_use]
    pub fn new(config: Connection, pool_size: usize) -> Self {
        let connections = (0..pool_size)
            .map(|_| ConnectionManager::new(config.clone()))
            .collect();

        Self {
            config,
            connections,
            current_index: AtomicUsize::new(0),
        }
    }

    /// Returns the configuration used by this connection pool
    #[must_use]
    pub const fn config(&self) -> &Connection {
        &self.config
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
        id: u32,
    }
    async fn create_mock_connection() -> StreamResult<MockConnection> {
        Ok(MockConnection { id: 42 })
    }
    async fn create_failing_connection() -> StreamResult<MockConnection> {
        Err(StreamError::Connection {
            message: "Mock failure".to_string(),
            retriable: true,
        })
    }

    #[tokio::test]
    async fn test_circuit_breaker_success() {
        let config = Connection::default();
        let breaker = CircuitBreaker::new(config);

        let result = breaker.attempt_connect(create_mock_connection).await;
        assert!(result.is_ok());
        assert_eq!(breaker.state(), ConnectionState::Connected);
    }

    #[tokio::test]
    async fn test_circuit_breaker_failure() {
        let config = Connection {
            max_retries: 1,
            ..Connection::default()
        };
        let breaker = CircuitBreaker::new(config);

        let result = breaker.attempt_connect(create_failing_connection).await;
        assert!(result.is_err());
        assert_eq!(breaker.state(), ConnectionState::Reconnecting);
    }

    #[tokio::test]
    #[expect(clippy::unwrap_used)]
    async fn test_connection_manager() {
        let config = Connection::default();
        let manager = ConnectionManager::new(config);

        manager.connect(create_mock_connection).await.unwrap();

        let connection = manager.get_connection().await;
        assert!(connection.is_some());

        let health = manager.health().await;
        assert_eq!(health.state, ConnectionState::Connected);
        assert!(health.connected_at.is_some());
    }

    #[tokio::test]
    #[expect(clippy::unwrap_used)]
    async fn test_connection_pool() {
        let config = Connection::default();
        let pool = ConnectionPool::new(config, 3);

        pool.connect_all(create_mock_connection).await.unwrap();

        let connection = pool.get_healthy_connection().await;
        assert!(connection.is_ok());

        let health_reports = pool.pool_health().await;
        assert_eq!(health_reports.len(), 3);
    }

    // Comprehensive tests for AtomicConnectionState
    #[test]
    fn test_atomic_connection_state_new() {
        let atomic_state = AtomicConnectionState::new(ConnectionState::Connected);
        assert_eq!(atomic_state.load(), ConnectionState::Connected);
    }

    #[test]
    fn test_atomic_connection_state_load_all_variants() {
        // Test all valid connection states
        let states = [
            ConnectionState::Connected,
            ConnectionState::Connecting,
            ConnectionState::Reconnecting,
            ConnectionState::Failed,
            ConnectionState::Disconnected,
        ];

        for state in states {
            let atomic_state = AtomicConnectionState::new(state);
            assert_eq!(atomic_state.load(), state);
        }
    }

    #[test]
    fn test_atomic_connection_state_load_invalid_value_returns_failed() {
        let atomic_state = AtomicConnectionState::new(ConnectionState::Connected);
        // Manually set an invalid value
        atomic_state.state.store(99, Ordering::Release);
        assert_eq!(atomic_state.load(), ConnectionState::Failed);
    }

    #[test]
    fn test_atomic_connection_state_store() {
        let atomic_state = AtomicConnectionState::new(ConnectionState::Disconnected);
        atomic_state.store(ConnectionState::Connected);
        assert_eq!(atomic_state.load(), ConnectionState::Connected);
    }

    #[test]
    fn test_atomic_connection_state_compare_exchange_success() {
        let atomic_state = AtomicConnectionState::new(ConnectionState::Disconnected);
        let result = atomic_state
            .compare_exchange(ConnectionState::Disconnected, ConnectionState::Connecting);
        assert_eq!(result, Ok(ConnectionState::Disconnected));
        assert_eq!(atomic_state.load(), ConnectionState::Connecting);
    }

    #[test]
    fn test_atomic_connection_state_compare_exchange_failure() {
        let atomic_state = AtomicConnectionState::new(ConnectionState::Connected);
        let result = atomic_state
            .compare_exchange(ConnectionState::Disconnected, ConnectionState::Connecting);
        assert_eq!(result, Err(ConnectionState::Connected));
        assert_eq!(atomic_state.load(), ConnectionState::Connected);
    }

    #[test]
    fn test_atomic_connection_state_compare_exchange_error_invalid_value_returns_failed() {
        let atomic_state = AtomicConnectionState::new(ConnectionState::Connected);
        // Manually set an invalid value
        atomic_state.state.store(99, Ordering::Release);
        let result =
            atomic_state.compare_exchange(ConnectionState::Connected, ConnectionState::Connecting);
        assert_eq!(result, Err(ConnectionState::Failed));
    }

    // Comprehensive tests for CircuitBreaker
    #[tokio::test]
    async fn test_circuit_breaker_new() {
        let config = Connection::default();
        let breaker = CircuitBreaker::new(config);
        assert_eq!(breaker.state(), ConnectionState::Disconnected);
    }

    #[tokio::test]
    async fn test_circuit_breaker_attempt_connect_when_failed_within_cooldown() {
        let config = Connection {
            max_retries: 1,
            retry_base_delay_ms: 1000,
            ..Connection::default()
        };
        let breaker = CircuitBreaker::new(config);

        // Force failure to set circuit breaker to Failed state
        let _ = breaker.attempt_connect(create_failing_connection).await;
        breaker.on_failure().await; // Force to Failed state with max retries exceeded

        assert_eq!(breaker.state(), ConnectionState::Failed);

        // Attempt connection while in cooldown
        let result = breaker.attempt_connect(create_mock_connection).await;
        assert!(result.is_err());
        if let Err(StreamError::Connection { message, retriable }) = result {
            assert!(retriable);
            assert!(message.contains("Circuit breaker open"));
        }
    }

    #[tokio::test]
    async fn test_circuit_breaker_attempt_connect_when_already_connecting() {
        let config = Connection::default();
        let breaker = CircuitBreaker::new(config);

        // Manually set state to Connecting
        breaker.state.store(ConnectionState::Connecting);

        let result = breaker.attempt_connect(create_mock_connection).await;
        assert!(result.is_err());
        if let Err(StreamError::Connection { message, retriable }) = result {
            assert!(!retriable);
            assert_eq!(message, "Connection attempt already in progress");
        }
    }
    async fn create_timeout_connection() -> StreamResult<MockConnection> {
        sleep(Duration::from_secs(10)).await;
        Ok(MockConnection { id: 42 })
    }

    #[tokio::test]
    async fn test_circuit_breaker_attempt_connect_timeout() {
        let config = Connection {
            connect_timeout_secs: 1, // 1 second minimum
            ..Connection::default()
        };
        let breaker = CircuitBreaker::new(config);

        let result = breaker.attempt_connect(create_timeout_connection).await;
        assert!(result.is_err());
        if let Err(StreamError::Connection { message, retriable }) = result {
            assert!(retriable);
            assert!(message.contains("Connection timeout"));
        }
        assert_eq!(breaker.state(), ConnectionState::Reconnecting);
    }

    #[tokio::test]
    async fn test_circuit_breaker_on_success() {
        let config = Connection::default();
        let breaker = CircuitBreaker::new(config);

        // Set some failures first
        breaker.failure_count.store(5, Ordering::Release);

        breaker.on_success().await;

        assert_eq!(breaker.state(), ConnectionState::Connected);
        assert_eq!(breaker.failure_count.load(Ordering::Acquire), 0);
        assert!(breaker.last_success.lock().await.is_some());
    }

    #[tokio::test]
    async fn test_circuit_breaker_on_failure_below_max_retries() {
        let config = Connection {
            max_retries: 5,
            ..Connection::default()
        };
        let breaker = CircuitBreaker::new(config);

        breaker.on_failure().await;

        assert_eq!(breaker.state(), ConnectionState::Reconnecting);
        assert_eq!(breaker.failure_count.load(Ordering::Acquire), 1);
        assert!(breaker.last_failure.lock().await.is_some());
    }

    #[tokio::test]
    async fn test_circuit_breaker_on_failure_exceeds_max_retries() {
        let config = Connection {
            max_retries: 3,
            ..Connection::default()
        };
        let breaker = CircuitBreaker::new(config);

        // Simulate multiple failures
        for _ in 0..3 {
            breaker.on_failure().await;
        }

        assert_eq!(breaker.state(), ConnectionState::Failed);
        assert_eq!(breaker.failure_count.load(Ordering::Acquire), 3);
    }

    #[tokio::test]
    async fn test_circuit_breaker_mark_disconnected() {
        let config = Connection::default();
        let breaker = CircuitBreaker::new(config);

        breaker.state.store(ConnectionState::Connected);
        breaker.mark_disconnected();

        assert_eq!(breaker.state(), ConnectionState::Disconnected);
    }

    #[test]
    fn test_circuit_breaker_calculate_backoff_delay() {
        let config = Connection {
            retry_base_delay_ms: 100,
            retry_max_delay_ms: 30_000,
            ..Connection::default()
        };
        let breaker = CircuitBreaker::new(config);

        // Test exponential backoff
        let initial_delay = breaker.calculate_backoff_delay(0);
        let first_retry_delay = breaker.calculate_backoff_delay(1);
        let second_retry_delay = breaker.calculate_backoff_delay(2);

        assert_eq!(initial_delay, Duration::from_millis(100));
        assert_eq!(first_retry_delay, Duration::from_millis(200));
        assert_eq!(second_retry_delay, Duration::from_millis(400));

        // Test capping at max delay
        let max_delay_test = breaker.calculate_backoff_delay(20);
        assert_eq!(max_delay_test, Duration::from_secs(30));

        // Test capping at attempt 10
        let tenth_attempt = breaker.calculate_backoff_delay(10);
        let eleventh_attempt = breaker.calculate_backoff_delay(11);
        assert_eq!(tenth_attempt, eleventh_attempt);
    }

    // Comprehensive tests for ConnectionManager
    #[tokio::test]
    async fn test_connection_manager_new() {
        let config = Connection::default();
        let manager = ConnectionManager::<MockConnection>::new(config);

        let health = manager.health().await;
        assert_eq!(health.state, ConnectionState::Disconnected);
        assert!(health.connected_at.is_none());
        assert!(health.last_activity.is_none());
        assert_eq!(health.total_reconnects, 0);
        assert_eq!(health.consecutive_failures, 0);
        assert!(health.latency_ms.is_none());
    }

    #[tokio::test]
    async fn test_connection_manager_connect_failure() {
        let config = Connection::default();
        let manager = ConnectionManager::new(config);

        let result = manager.connect(create_failing_connection).await;
        assert!(result.is_err());

        let connection = manager.get_connection().await;
        assert!(connection.is_none());
    }

    #[tokio::test]
    async fn test_connection_manager_get_connection_none() {
        let config = Connection::default();
        let manager = ConnectionManager::<MockConnection>::new(config);

        let connection = manager.get_connection().await;
        assert!(connection.is_none());
    }

    #[tokio::test]
    async fn test_connection_manager_with_connection_no_active_connection() {
        let config = Connection::default();
        let manager = ConnectionManager::<MockConnection>::new(config);

        let result = manager.with_connection(|_conn| 42).await;
        assert!(result.is_err());
        if let Err(StreamError::Connection { message, retriable }) = result {
            assert!(!retriable);
            assert_eq!(message, "No active connection");
        }
    }

    #[tokio::test]
    #[expect(clippy::unwrap_used)]
    async fn test_connection_manager_with_connection_success() {
        let config = Connection::default();
        let manager = ConnectionManager::new(config);

        manager.connect(create_mock_connection).await.unwrap();

        let result = manager
            .with_connection(|conn| conn.id.saturating_mul(2))
            .await;
        assert_eq!(result.unwrap(), 84);

        // Verify activity was updated
        let health = manager.health().await;
        assert!(health.last_activity.is_some());
    }

    #[tokio::test]
    #[expect(clippy::unwrap_used)]
    async fn test_connection_manager_disconnect() {
        let config = Connection::default();
        let manager = ConnectionManager::new(config);

        manager.connect(create_mock_connection).await.unwrap();
        assert!(manager.get_connection().await.is_some());

        manager.disconnect().await;

        let connection = manager.get_connection().await;
        assert!(connection.is_none());

        let health = manager.health().await;
        assert_eq!(health.state, ConnectionState::Disconnected);
        assert!(health.connected_at.is_none());
    }

    #[tokio::test]
    async fn test_connection_manager_update_health_on_connect() {
        let config = Connection::default();
        let manager = ConnectionManager::<MockConnection>::new(config);

        // Set some initial failure state
        {
            let mut health = manager.health.write().await;
            health.consecutive_failures = 5;
            health.total_reconnects = 10;
        }

        manager.update_health_on_connect().await;

        let health = manager.health().await;
        assert_eq!(health.state, ConnectionState::Connected);
        assert!(health.connected_at.is_some());
        assert!(health.last_activity.is_some());
        assert_eq!(health.total_reconnects, 11);
        assert_eq!(health.consecutive_failures, 0);
    }

    #[tokio::test]
    #[expect(clippy::unwrap_used)]
    async fn test_connection_manager_update_activity() {
        let config = Connection::default();
        let manager = ConnectionManager::<MockConnection>::new(config);

        let before = Instant::now();
        manager.update_activity().await;

        let health = manager.health().await;
        assert!(health.last_activity.is_some());
        assert!(health.last_activity.unwrap() >= before);
    }

    #[tokio::test]
    async fn test_connection_manager_start_reconnect_monitor_already_running() {
        let config = Connection::default();
        let manager = ConnectionManager::new(config);

        // Set monitor as already running
        manager.reconnect_task.store(true, Ordering::Release);

        // This should return immediately without starting a new monitor
        manager.start_reconnect_monitor(create_mock_connection);

        // The task should still be marked as running
        assert!(manager.reconnect_task.load(Ordering::Acquire));
    }

    #[tokio::test]
    async fn test_connection_manager_reconnect_monitor_success() {
        let config = Connection {
            max_retries: 2,
            ..Connection::default()
        };
        let manager = ConnectionManager::new(config);

        // Start with a failed connection
        let _ = manager.connect(create_failing_connection).await;

        // Wait a bit for the reconnect monitor to attempt reconnection
        sleep(Duration::from_millis(100)).await;

        // Stop the monitor
        manager.disconnect().await;
    }

    // Comprehensive tests for ConnectionPool
    #[tokio::test]
    async fn test_connection_pool_new() {
        let config = Connection::default();
        let pool = ConnectionPool::<MockConnection>::new(config, 5);

        assert_eq!(pool.connections.len(), 5);
        assert_eq!(pool.current_index.load(Ordering::Acquire), 0);
    }

    #[tokio::test]
    async fn test_connection_pool_new_zero_size() {
        let config = Connection::default();
        let pool = ConnectionPool::<MockConnection>::new(config, 0);

        assert_eq!(pool.connections.len(), 0);
    }

    #[tokio::test]
    async fn test_connection_pool_connect_all_with_failures() {
        let config = Connection::default();
        let pool = ConnectionPool::new(config, 3);

        // This should succeed even if some connections fail
        let result = pool.connect_all(create_failing_connection).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_connection_pool_get_healthy_connection_no_healthy_connections() {
        let config = Connection::default();
        let pool = ConnectionPool::<MockConnection>::new(config, 3);

        // Don't connect anything
        let result = pool.get_healthy_connection().await;
        assert!(result.is_err());
        if let Err(StreamError::Connection { message, retriable }) = result {
            assert!(retriable);
            assert_eq!(message, "No healthy connections available");
        }
    }

    #[tokio::test]
    #[expect(clippy::unwrap_used)]
    async fn test_connection_pool_get_healthy_connection_round_robin() {
        let config = Connection::default();
        let pool = ConnectionPool::new(config, 3);

        pool.connect_all(create_mock_connection).await.unwrap();

        // Get several connections to test round-robin
        for i in 0_usize..6 {
            let result = pool.get_healthy_connection().await;
            assert!(result.is_ok());

            // Verify round-robin progression
            let expected_next_index = i.saturating_add(1) % 3;
            assert_eq!(
                pool.current_index.load(Ordering::Acquire),
                expected_next_index
            );
        }
    }

    #[tokio::test]
    #[expect(clippy::unwrap_used)]
    async fn test_connection_pool_get_healthy_connection_skip_unhealthy() {
        let config = Connection::default();
        let pool = ConnectionPool::new(config, 3);

        // Connect only the first connection
        pool.connections
            .first()
            .unwrap()
            .connect(create_mock_connection)
            .await
            .unwrap();

        // Disconnect the second connection to make it unhealthy
        pool.connections.get(1).unwrap().disconnect().await;

        let result = pool.get_healthy_connection().await;
        assert!(result.is_ok());

        // Should have found the healthy connection at index 0
        assert_eq!(pool.current_index.load(Ordering::Acquire), 1);
    }

    #[tokio::test]
    #[expect(clippy::unwrap_used)]
    async fn test_connection_pool_pool_health() {
        let config = Connection::default();
        let pool = ConnectionPool::new(config, 2);

        // Connect first connection
        pool.connections
            .first()
            .unwrap()
            .connect(create_mock_connection)
            .await
            .unwrap();

        let health_reports = pool.pool_health().await;
        assert_eq!(health_reports.len(), 2);
        assert_eq!(
            health_reports.first().unwrap().state,
            ConnectionState::Connected
        );
        assert_eq!(
            health_reports.get(1).unwrap().state,
            ConnectionState::Disconnected
        );
    }

    #[tokio::test]
    async fn test_connection_pool_empty_pool_health() {
        let config = Connection::default();
        let pool = ConnectionPool::<MockConnection>::new(config, 0);

        let health_reports = pool.pool_health().await;
        assert_eq!(health_reports.len(), 0);
    }

    // Test ConnectionHealth Clone and Debug
    #[test]
    fn test_connection_health_clone_and_debug() {
        let health = ConnectionHealth {
            state: ConnectionState::Connected,
            connected_at: Some(Instant::now()),
            last_activity: Some(Instant::now()),
            total_reconnects: 5,
            consecutive_failures: 2,
            latency_ms: Some(42),
        };

        let cloned = health.clone();
        assert_eq!(health.state, cloned.state);
        assert_eq!(health.total_reconnects, cloned.total_reconnects);
        assert_eq!(health.consecutive_failures, cloned.consecutive_failures);
        assert_eq!(health.latency_ms, cloned.latency_ms);

        // Test Debug implementation
        let debug_str = format!("{health:?}");
        assert!(debug_str.contains("Connected"));
        assert!(debug_str.contains("total_reconnects: 5"));
    }

    // Test ConnectionState PartialEq and Eq
    #[test]
    fn test_connection_state_equality() {
        assert_eq!(ConnectionState::Connected, ConnectionState::Connected);
        assert_ne!(ConnectionState::Connected, ConnectionState::Disconnected);
    }

    // Test ConnectionState Debug
    #[test]
    fn test_connection_state_debug() {
        let debug_str = format!("{:?}", ConnectionState::Connected);
        assert_eq!(debug_str, "Connected");
    }

    // Test CircuitBreaker Debug
    #[test]
    fn test_circuit_breaker_debug() {
        let config = Connection::default();
        let breaker = CircuitBreaker::new(config);
        let debug_str = format!("{breaker:?}");
        assert!(debug_str.contains("CircuitBreaker"));
    }

    // Edge case: Test with very large attempt numbers
    #[test]
    fn test_circuit_breaker_calculate_backoff_delay_overflow_protection() {
        let config = Connection {
            retry_base_delay_ms: 100,
            retry_max_delay_ms: 30_000,
            ..Connection::default()
        };
        let breaker = CircuitBreaker::new(config);

        // Test with very large attempt number
        let delay = breaker.calculate_backoff_delay(usize::MAX);
        assert_eq!(delay, Duration::from_secs(30));
    }

    // Test circuit breaker with failed state and no last failure time
    #[tokio::test]
    async fn test_circuit_breaker_failed_state_no_last_failure() {
        let config = Connection::default();
        let breaker = CircuitBreaker::new(config);

        // Manually set to Failed state without setting last_failure
        breaker.state.store(ConnectionState::Failed);

        let result = breaker.attempt_connect(create_mock_connection).await;
        // Should succeed because there's no last_failure time to check cooldown
        assert!(result.is_ok());
    }

    // Test circuit breaker cooldown period passed
    #[tokio::test]
    async fn test_circuit_breaker_cooldown_period_passed() {
        let config = Connection {
            max_retries: 1,
            retry_base_delay_ms: 1, // Very short cooldown
            ..Connection::default()
        };
        let breaker = CircuitBreaker::new(config);

        // Force failure to set circuit breaker to Failed state
        let _ = breaker.attempt_connect(create_failing_connection).await;
        breaker.on_failure().await; // Force to Failed state

        // Wait for cooldown to pass
        sleep(Duration::from_millis(10)).await;

        // Should be able to attempt connection again
        let result = breaker.attempt_connect(create_mock_connection).await;
        assert!(result.is_ok());
    }
}
