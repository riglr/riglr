//! Core indexer service components

use core::fmt::{Debug, Formatter, Result as FmtResult};
use core::time::Duration;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tokio::time;
use tracing::{error, info, warn};

use crate::config::IndexerConfig;
use crate::error::{IndexerError, IndexerResult, ServiceError};
use crate::metrics::MetricsCollector;
use crate::storage::DataStore;

pub mod indexer;
pub mod ingester;
pub mod pipeline;
pub mod processor;

pub use indexer::Service as IndexerService;
pub use ingester::{Config as IngesterConfig, Ingester as EventIngester};
pub use pipeline::{PipelineResult, PipelineStage, ProcessingPipeline as Pipeline};
pub use processor::{Config as ProcessorConfig, ProcessingPipeline, Processor as EventProcessor};

/// Service state enumeration
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServiceState {
    /// Service encountered an error
    Error(String),
    /// Service is running normally
    Running,
    /// Service is initializing
    Starting,
    /// Service has stopped
    Stopped,
    /// Service is shutting down gracefully
    Stopping,
}

/// Health check status
#[derive(Debug, Clone)]
pub struct HealthStatus {
    /// Individual component statuses
    pub components: HashMap<String, ComponentHealth>,
    /// Overall health status
    pub healthy: bool,
    /// Last check timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Component health information
#[derive(Debug, Clone)]
pub struct ComponentHealth {
    /// Error count since last success
    pub error_count: u64,
    /// Component is healthy
    pub healthy: bool,
    /// Last successful operation timestamp
    pub last_success: Option<chrono::DateTime<chrono::Utc>>,
    /// Status message
    pub message: String,
}

impl ComponentHealth {
    /// Create a healthy component status
    #[must_use]
    pub fn healthy(message: &str) -> Self {
        Self {
            error_count: 0,
            healthy: true,
            last_success: Some(chrono::Utc::now()),
            message: message.to_string(),
        }
    }

    /// Create an unhealthy component status
    #[must_use]
    pub fn unhealthy(message: &str) -> Self {
        Self {
            error_count: 1,
            healthy: false,
            last_success: None,
            message: message.to_string(),
        }
    }

    /// Record an error
    pub fn record_error(&mut self, message: &str) {
        self.healthy = false;
        self.message = message.to_string();
        self.error_count = self.error_count.saturating_add(1);
    }

    /// Record a successful operation
    pub fn record_success(&mut self) {
        self.healthy = true;
        self.last_success = Some(chrono::Utc::now());
        self.error_count = 0;
    }
}

/// Shared service context
pub struct ServiceContext {
    /// Service configuration
    pub config: Arc<IndexerConfig>,
    /// Health status
    pub health: Arc<RwLock<HealthStatus>>,
    /// Metrics collector
    pub metrics: Arc<MetricsCollector>,
    /// Shutdown signal broadcaster
    pub shutdown_tx: broadcast::Sender<()>,
    /// Service state
    pub state: Arc<RwLock<ServiceState>>,
    /// Data store
    pub store: Arc<dyn DataStore>,
}

impl ServiceContext {
    /// Create a new service context
    pub fn new(
        config: IndexerConfig,
        store: Arc<dyn DataStore>,
        metrics: Arc<MetricsCollector>,
    ) -> Self {
        let (shutdown_tx, _) = broadcast::channel(1);
        let health = HealthStatus {
            components: HashMap::new(),
            healthy: true,
            timestamp: chrono::Utc::now(),
        };

        Self {
            config: Arc::new(config),
            health: Arc::new(RwLock::new(health)),
            metrics,
            shutdown_tx,
            state: Arc::new(RwLock::new(ServiceState::Starting)),
            store,
        }
    }

    /// Get current service state
    pub async fn state(&self) -> ServiceState {
        self.state.read().await.clone()
    }

    /// Update service state
    pub async fn set_state(&self, state: ServiceState) {
        let mut current_state = self.state.write().await;
        info!(
            "Service state transition: {:?} -> {:?}",
            *current_state, state
        );
        *current_state = state;
    }

    /// Check if shutdown has been requested
    #[must_use]
    pub fn is_shutdown_requested(&self) -> bool {
        self.shutdown_tx.receiver_count() == 0
    }

    /// Request shutdown
    ///
    /// # Errors
    ///
    /// Returns error if shutdown signal cannot be sent to all services
    pub fn request_shutdown(&self) -> IndexerResult<()> {
        if self.shutdown_tx.send(()).is_ok() {
            info!("Shutdown signal sent");
        } else {
            warn!("No shutdown receivers listening");
        }
        Ok(())
    }

    /// Subscribe to shutdown signals
    #[must_use]
    pub fn shutdown_receiver(&self) -> broadcast::Receiver<()> {
        self.shutdown_tx.subscribe()
    }

    /// Update component health
    pub async fn update_component_health(&self, component: &str, health: ComponentHealth) {
        let mut status = self.health.write().await;
        status.components.insert(component.to_string(), health);
        status.timestamp = chrono::Utc::now();

        // Update overall health based on components
        status.healthy = status.components.values().all(|c| c.healthy);
    }

    /// Get current health status
    pub async fn health_status(&self) -> HealthStatus {
        self.health.read().await.clone()
    }

    /// Perform health check on all components
    ///
    /// # Errors
    ///
    /// Returns error if health check operation fails
    pub async fn health_check(&self) -> IndexerResult<HealthStatus> {
        let mut status = self.health.write().await;

        // Check data store health
        match self.store.health_check().await {
            Ok(()) => {
                status.components.insert(
                    "storage".to_string(),
                    ComponentHealth::healthy("Storage connection OK"),
                );
            }
            Err(e) => {
                status.components.insert(
                    "storage".to_string(),
                    ComponentHealth::unhealthy(&format!("Storage error: {e}")),
                );
            }
        }

        // Update overall health
        status.healthy = status.components.values().all(|c| c.healthy);
        status.timestamp = chrono::Utc::now();

        Ok(status.clone())
    }
}

impl Debug for ServiceContext {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_struct("ServiceContext")
            .field("config", &self.config)
            .field("state", &"Arc<RwLock<ServiceState>>")
            .field("store", &"Arc<dyn DataStore>")
            .field("metrics", &"Arc<MetricsCollector>")
            .field("shutdown_tx", &"broadcast::Sender<()>")
            .field("health", &"Arc<RwLock<HealthStatus>>")
            .finish()
    }
}

/// Service lifecycle trait
#[async_trait::async_trait]
pub trait ServiceLifecycle {
    /// Get current health status
    async fn health(&self) -> IndexerResult<HealthStatus>;

    /// Check if service is running
    fn is_running(&self) -> bool;

    /// Start the service
    async fn start(&mut self) -> IndexerResult<()>;

    /// Stop the service gracefully
    async fn stop(&mut self) -> IndexerResult<()>;
}

/// Event processing trait
#[async_trait::async_trait]
pub trait EventProcessing {
    /// Process a batch of events
    async fn process_batch(
        &self,
        events: Vec<Box<dyn riglr_events_core::Event>>,
    ) -> IndexerResult<()>;

    /// Process a single event
    async fn process_event(&self, event: Box<dyn riglr_events_core::Event>) -> IndexerResult<()>;

    /// Get processing statistics
    async fn processing_stats(&self) -> ProcessingStats;
}

/// Processing statistics
#[derive(Debug, Clone)]
pub struct ProcessingStats {
    /// Active workers
    pub active_workers: usize,
    /// Average processing latency in milliseconds
    pub avg_latency_ms: f64,
    /// Error rate (errors per total events)
    pub error_rate: f64,
    /// Events processed per second (current rate)
    pub events_per_second: f64,
    /// Queue depth
    pub queue_depth: usize,
    /// Total events processed
    pub total_processed: u64,
}

impl Default for ProcessingStats {
    fn default() -> Self {
        Self {
            active_workers: 0,
            avg_latency_ms: 0.0,
            error_rate: 0.0,
            events_per_second: 0.0,
            queue_depth: 0,
            total_processed: 0,
        }
    }
}

/// Graceful shutdown coordinator
pub struct ShutdownCoordinator {
    /// Services to shutdown
    services: Vec<Box<dyn ServiceLifecycle + Send + Sync>>,
    /// Timeout for graceful shutdown
    timeout: Duration,
}

impl ShutdownCoordinator {
    /// Create a new shutdown coordinator
    #[must_use]
    pub fn new(timeout: Duration) -> Self {
        Self {
            services: Vec::new(),
            timeout,
        }
    }

    /// Add a service to coordinate
    pub fn add_service(&mut self, service: Box<dyn ServiceLifecycle + Send + Sync>) {
        self.services.push(service);
    }

    /// Shutdown all services gracefully
    ///
    /// # Errors
    ///
    /// Returns error if any service fails to shutdown gracefully
    pub async fn shutdown_all(&mut self) -> IndexerResult<()> {
        info!(
            "Starting graceful shutdown of {} services",
            self.services.len()
        );

        let results = self.shutdown_services().await;
        Self::process_shutdown_results(results)
    }

    /// Shutdown all services and collect results
    async fn shutdown_services(&mut self) -> Vec<(usize, IndexerResult<()>)> {
        let mut results = Vec::new();
        let timeout = self.timeout;

        for (i, service) in self.services.iter_mut().enumerate() {
            let result = Self::shutdown_single_service(i, service, timeout).await;
            results.push((i, result));
        }

        results
    }

    /// Shutdown a single service with timeout handling
    async fn shutdown_single_service(
        service_index: usize,
        service: &mut Box<dyn ServiceLifecycle + Send + Sync>,
        timeout: Duration,
    ) -> IndexerResult<()> {
        let timeout_result = time::timeout(timeout, service.stop()).await;

        timeout_result.map_or_else(
            |_| {
                error!("Service {} shutdown timeout", service_index);
                Err(IndexerError::Service(ServiceError::ShutdownTimeout {
                    seconds: timeout.as_secs(),
                }))
            },
            |result| {
                match result {
                    Ok(()) => info!("Service {} shut down successfully", service_index),
                    Err(ref e) => error!("Service {} shutdown error: {}", service_index, e),
                }
                result
            },
        )
    }

    /// Process shutdown results and return final outcome
    fn process_shutdown_results(results: Vec<(usize, IndexerResult<()>)>) -> IndexerResult<()> {
        let errors: Vec<(usize, IndexerError)> = results
            .into_iter()
            .filter_map(|(i, result)| result.err().map(|e| (i, e)))
            .collect();

        if errors.is_empty() {
            info!("All services shut down successfully");
            return Ok(());
        }
        error!("Some services failed to shut down: {:?}", errors);
        // Return the first error, but safely handle empty case (though it should never happen due to filter_map)
        errors.into_iter().next().map_or_else(
            || {
                // This should never happen, but provides a fallback
                Err(IndexerError::Service(ServiceError::StartupFailed {
                    reason: "Unknown shutdown error".to_string(),
                }))
            },
            |(_, error)| Err(error),
        )
    }
}

impl Debug for ShutdownCoordinator {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_struct("ShutdownCoordinator")
            .field("timeout", &self.timeout)
            .field(
                "services",
                &format!(
                    "Vec<Box<dyn ServiceLifecycle + Send + Sync>> (len: {})",
                    self.services.len()
                ),
            )
            .finish()
    }
}

#[cfg(test)]
#[expect(clippy::expect_used, clippy::panic)]
#[expect(clippy::float_cmp)] // All test float comparisons use exact literal values
mod tests {
    use super::*;
    use async_trait::async_trait;
    use core::any::Any;
    use core::sync::atomic::{AtomicBool, Ordering};
    use std::collections::HashMap;
    use tokio::time::Duration;

    use crate::config::MetricsConfig;
    use crate::error::StorageError;
    use crate::storage::{EventFilter, EventQuery, StorageStats, StoredEvent};

    // Mock DataStore for testing
    struct MockDataStore {
        health_check_result: Arc<AtomicBool>,
    }

    impl MockDataStore {
        fn new(healthy: bool) -> Self {
            Self {
                health_check_result: Arc::new(AtomicBool::new(healthy)),
            }
        }
    }

    #[async_trait]
    impl DataStore for MockDataStore {
        async fn health_check(&self) -> IndexerResult<()> {
            if self.health_check_result.load(Ordering::SeqCst) {
                Ok(())
            } else {
                Err(IndexerError::Storage(StorageError::ConnectionFailed {
                    message: "Mock connection failed".to_string(),
                }))
            }
        }

        async fn insert_event(&self, _event: &StoredEvent) -> IndexerResult<()> {
            Ok(())
        }

        async fn insert_events(&self, _events: &[StoredEvent]) -> IndexerResult<()> {
            Ok(())
        }

        async fn query_events(&self, _query: &EventQuery) -> IndexerResult<Vec<StoredEvent>> {
            Ok(Vec::new())
        }

        async fn get_event(&self, _id: &str) -> IndexerResult<Option<StoredEvent>> {
            Ok(None)
        }

        async fn delete_events(&self, _filter: &EventFilter) -> IndexerResult<u64> {
            Ok(0)
        }

        async fn count_events(&self, _filter: &EventFilter) -> IndexerResult<u64> {
            Ok(0)
        }

        async fn get_stats(&self) -> IndexerResult<StorageStats> {
            return Ok(StorageStats {
                total_events: 0,
                storage_size_bytes: 0,
                avg_write_latency_ms: 0.0,
                avg_read_latency_ms: 0.0,
                active_connections: 0,
                cache_hit_rate: 0.0,
            });
        }

        async fn initialize(&self) -> IndexerResult<()> {
            Ok(())
        }

        fn as_any(&self) -> &dyn Any {
            self
        }
    }

    // Mock ServiceLifecycle implementation for testing
    struct MockService {
        delay: Option<Duration>,
        running: Arc<AtomicBool>,
        should_fail: bool,
    }

    impl MockService {
        fn new(should_fail: bool, delay: Option<Duration>) -> Self {
            Self {
                delay,
                running: Arc::new(AtomicBool::new(false)),
                should_fail,
            }
        }
    }

    #[async_trait]
    impl ServiceLifecycle for MockService {
        async fn start(&mut self) -> IndexerResult<()> {
            if let Some(delay) = self.delay {
                time::sleep(delay).await;
            }
            if self.should_fail {
                return Err(IndexerError::Service(ServiceError::StartupFailed {
                    reason: "Mock startup failed".to_string(),
                }));
            }
            self.running.store(true, Ordering::SeqCst);
            Ok(())
        }

        async fn stop(&mut self) -> IndexerResult<()> {
            if let Some(delay) = self.delay {
                time::sleep(delay).await;
            }
            if self.should_fail {
                return Err(IndexerError::Service(ServiceError::StartupFailed {
                    reason: "Mock shutdown failed".to_string(),
                }));
            }
            self.running.store(false, Ordering::SeqCst);
            Ok(())
        }

        async fn health(&self) -> IndexerResult<HealthStatus> {
            return Ok(HealthStatus {
                components: HashMap::new(),
                healthy: true,
                timestamp: chrono::Utc::now(),
            });
        }
        fn is_running(&self) -> bool {
            self.running.load(Ordering::SeqCst)
        }
    }
    fn create_test_config() -> IndexerConfig {
        IndexerConfig::default()
    }
    fn create_test_metrics() -> Arc<MetricsCollector> {
        let config = MetricsConfig {
            enabled: true,
            port: 9090,
            endpoint: "/metrics".to_string(),
            collection_interval: Duration::from_secs(60),
            histogram_buckets: vec![0.1, 0.5, 1.0, 5.0, 10.0],
            custom: HashMap::new(),
        };
        Arc::new(
            MetricsCollector::new(config)
                .expect("Failed to create metrics collector with valid config"),
        )
    }

    // Tests for ServiceState
    #[test]
    fn test_service_state_equality() {
        assert_eq!(ServiceState::Starting, ServiceState::Starting);
        assert_eq!(ServiceState::Running, ServiceState::Running);
        assert_eq!(ServiceState::Stopping, ServiceState::Stopping);
        assert_eq!(ServiceState::Stopped, ServiceState::Stopped);
        assert_eq!(
            ServiceState::Error("test".to_string()),
            ServiceState::Error("test".to_string())
        );
        assert_ne!(ServiceState::Starting, ServiceState::Running);
        assert_ne!(
            ServiceState::Error("test1".to_string()),
            ServiceState::Error("test2".to_string())
        );
    }

    #[test]
    fn test_service_state_debug() {
        let state = ServiceState::Error("test error".to_string());
        let debug_str = format!("{state:?}");
        assert!(debug_str.contains("Error"));
        assert!(debug_str.contains("test error"));
    }

    #[test]
    fn test_service_state_clone() {
        let original = ServiceState::Error("test".to_string());
        let cloned = original.clone();
        assert_eq!(original, cloned);
    }

    // Tests for ComponentHealth
    #[test]
    fn test_component_health_healthy() {
        let health = ComponentHealth::healthy("All good");
        assert!(health.healthy);
        assert_eq!(health.message, "All good");
        assert!(health.last_success.is_some());
        assert_eq!(health.error_count, 0);
    }

    #[test]
    fn test_component_health_unhealthy() {
        let health = ComponentHealth::unhealthy("Something wrong");
        assert!(!health.healthy);
        assert_eq!(health.message, "Something wrong");
        assert!(health.last_success.is_none());
        assert_eq!(health.error_count, 1);
    }

    #[test]
    fn test_component_health_record_success() {
        let mut health = ComponentHealth::unhealthy("Error message");
        assert!(!health.healthy);
        assert_eq!(health.error_count, 1);

        health.record_success();
        assert!(health.healthy);
        assert!(health.last_success.is_some());
        assert_eq!(health.error_count, 0);
    }

    #[test]
    fn test_component_health_record_error() {
        let mut health = ComponentHealth::healthy("Good");
        assert!(health.healthy);

        health.record_error("New error");
        assert!(!health.healthy);
        assert_eq!(health.message, "New error");
        assert_eq!(health.error_count, 1);

        health.record_error("Another error");
        assert_eq!(health.error_count, 2);
        assert_eq!(health.message, "Another error");
    }

    #[test]
    fn test_component_health_debug() {
        let health = ComponentHealth::healthy("test");
        let debug_str = format!("{health:?}");
        assert!(debug_str.contains("ComponentHealth"));
    }

    #[test]
    fn test_component_health_clone() {
        let original = ComponentHealth::healthy("test");
        let cloned = original.clone();
        assert_eq!(original.healthy, cloned.healthy);
        assert_eq!(original.message, cloned.message);
        assert_eq!(original.error_count, cloned.error_count);
    }

    // Tests for HealthStatus
    #[test]
    fn test_health_status_debug() {
        let status = HealthStatus {
            healthy: true,
            components: HashMap::new(),
            timestamp: chrono::Utc::now(),
        };
        let debug_str = format!("{status:?}");
        assert!(debug_str.contains("HealthStatus"));
    }

    #[test]
    fn test_health_status_clone() {
        let mut components = HashMap::new();
        components.insert("test".to_string(), ComponentHealth::healthy("test"));
        let original = HealthStatus {
            healthy: true,
            components,
            timestamp: chrono::Utc::now(),
        };
        let cloned = original.clone();
        assert_eq!(original.healthy, cloned.healthy);
        assert_eq!(original.components.len(), cloned.components.len());
    }

    // Tests for ProcessingStats
    #[test]
    fn test_processing_stats_default() {
        let stats = ProcessingStats::default();
        assert_eq!(stats.total_processed, 0);
        {
            assert_eq!(stats.events_per_second, 0.0);
            assert_eq!(stats.avg_latency_ms, 0.0);
            assert_eq!(stats.error_rate, 0.0);
        }
        assert_eq!(stats.queue_depth, 0);
        assert_eq!(stats.active_workers, 0);
    }

    #[test]
    fn test_processing_stats_debug() {
        let stats = ProcessingStats::default();
        let debug_str = format!("{stats:?}");
        assert!(debug_str.contains("ProcessingStats"));
    }

    #[test]
    fn test_processing_stats_clone() {
        let original = ProcessingStats {
            total_processed: 100,
            events_per_second: 10.5,
            avg_latency_ms: 5.2,
            error_rate: 0.01,
            queue_depth: 5,
            active_workers: 3,
        };
        let cloned = original.clone();
        assert_eq!(original.total_processed, cloned.total_processed);
        {
            assert_eq!(original.events_per_second, cloned.events_per_second);
            assert_eq!(original.avg_latency_ms, cloned.avg_latency_ms);
            assert_eq!(original.error_rate, cloned.error_rate);
        }
        assert_eq!(original.queue_depth, cloned.queue_depth);
        assert_eq!(original.active_workers, cloned.active_workers);
    }

    // Tests for ServiceContext
    #[tokio::test]
    async fn test_service_context_new() {
        let config = create_test_config();
        let store = Arc::new(MockDataStore::new(true));
        let metrics = create_test_metrics();

        let context = ServiceContext::new(config, store, metrics);

        assert_eq!(context.state().await, ServiceState::Starting);
        let health = context.health_status().await;
        assert!(health.healthy);
        assert!(health.components.is_empty());
    }

    #[tokio::test]
    async fn test_service_context_state_transitions() {
        let config = create_test_config();
        let store = Arc::new(MockDataStore::new(true));
        let metrics = create_test_metrics();
        let context = ServiceContext::new(config, store, metrics);

        assert_eq!(context.state().await, ServiceState::Starting);

        context.set_state(ServiceState::Running).await;
        assert_eq!(context.state().await, ServiceState::Running);

        context.set_state(ServiceState::Stopping).await;
        assert_eq!(context.state().await, ServiceState::Stopping);

        context.set_state(ServiceState::Stopped).await;
        assert_eq!(context.state().await, ServiceState::Stopped);

        context
            .set_state(ServiceState::Error("test error".to_string()))
            .await;
        assert_eq!(
            context.state().await,
            ServiceState::Error("test error".to_string())
        );
    }

    #[tokio::test]
    async fn test_service_context_shutdown_signal() {
        let config = create_test_config();
        let store = Arc::new(MockDataStore::new(true));
        let metrics = create_test_metrics();
        let context = ServiceContext::new(config, store, metrics);

        // Initially no shutdown requested
        assert!(!context.is_shutdown_requested());

        // Subscribe to shutdown signal
        let mut receiver = context.shutdown_receiver();

        // Request shutdown
        assert!(context.request_shutdown().is_ok());

        // Should receive shutdown signal
        assert!(receiver.recv().await.is_ok());
    }

    #[tokio::test]
    async fn test_service_context_shutdown_no_receivers() {
        let config = create_test_config();
        let store = Arc::new(MockDataStore::new(true));
        let metrics = create_test_metrics();
        let context = ServiceContext::new(config, store, metrics);

        // Request shutdown without any receivers
        assert!(context.request_shutdown().is_ok());
    }

    #[tokio::test]
    async fn test_service_context_update_component_health() {
        let config = create_test_config();
        let store = Arc::new(MockDataStore::new(true));
        let metrics = create_test_metrics();
        let context = ServiceContext::new(config, store, metrics);

        let component_health = ComponentHealth::healthy("Test component OK");
        context
            .update_component_health("test_component", component_health)
            .await;

        let health = context.health_status().await;
        assert!(health.healthy);
        assert!(health.components.contains_key("test_component"));
        assert!(
            health
                .components
                .get("test_component")
                .expect("Should have test_component")
                .healthy
        );
    }

    #[tokio::test]
    async fn test_service_context_update_component_health_unhealthy() {
        let config = create_test_config();
        let store = Arc::new(MockDataStore::new(true));
        let metrics = create_test_metrics();
        let context = ServiceContext::new(config, store, metrics);

        // Add healthy component first
        let healthy_component = ComponentHealth::healthy("Healthy component");
        context
            .update_component_health("healthy", healthy_component)
            .await;

        // Add unhealthy component
        let unhealthy_component = ComponentHealth::unhealthy("Component failed");
        context
            .update_component_health("unhealthy", unhealthy_component)
            .await;

        let health = context.health_status().await;
        assert!(!health.healthy); // Overall health should be false
        assert!(health.components.contains_key("healthy"));
        assert!(health.components.contains_key("unhealthy"));
        assert!(
            health
                .components
                .get("healthy")
                .expect("Should have healthy component")
                .healthy
        );
        assert!(
            !health
                .components
                .get("unhealthy")
                .expect("Should have unhealthy component")
                .healthy
        );
    }

    #[tokio::test]
    async fn test_service_context_health_check_storage_healthy() {
        let config = create_test_config();
        let store = Arc::new(MockDataStore::new(true));
        let metrics = create_test_metrics();
        let context = ServiceContext::new(config, store, metrics);

        let health = context
            .health_check()
            .await
            .expect("Health check should succeed with healthy store");
        assert!(health.healthy);
        assert!(health.components.contains_key("storage"));
        assert!(
            health
                .components
                .get("storage")
                .expect("Should have storage component")
                .healthy
        );
        assert_eq!(
            health
                .components
                .get("storage")
                .expect("Should have storage component")
                .message,
            "Storage connection OK"
        );
    }

    #[tokio::test]
    async fn test_service_context_health_check_storage_unhealthy() {
        let config = create_test_config();
        let store = Arc::new(MockDataStore::new(false));
        let metrics = create_test_metrics();
        let context = ServiceContext::new(config, store, metrics);

        let health = context
            .health_check()
            .await
            .expect("Health check should complete even with unhealthy store");
        assert!(!health.healthy);
        assert!(health.components.contains_key("storage"));
        assert!(
            !health
                .components
                .get("storage")
                .expect("Should have storage component")
                .healthy
        );
        assert!(health
            .components
            .get("storage")
            .expect("Should have storage component")
            .message
            .contains("Storage error"));
    }

    #[tokio::test]
    async fn test_service_context_health_check_mixed_components() {
        let config = create_test_config();
        let store = Arc::new(MockDataStore::new(true));
        let metrics = create_test_metrics();
        let context = ServiceContext::new(config, store, metrics);

        // Add an unhealthy component first
        let unhealthy_component = ComponentHealth::unhealthy("Component failed");
        context
            .update_component_health("test_component", unhealthy_component)
            .await;

        // Now run health check which will add healthy storage
        let health = context
            .health_check()
            .await
            .expect("Health check should complete");
        assert!(!health.healthy); // Overall should still be unhealthy
        assert!(health.components.contains_key("storage"));
        assert!(health.components.contains_key("test_component"));
        assert!(
            health
                .components
                .get("storage")
                .expect("Should have storage component")
                .healthy
        );
        assert!(
            !health
                .components
                .get("test_component")
                .expect("Should have test_component")
                .healthy
        );
    }

    // Tests for ShutdownCoordinator
    #[tokio::test]
    async fn test_shutdown_coordinator_new() {
        let timeout = Duration::from_secs(30);
        let coordinator = ShutdownCoordinator::new(timeout);
        assert_eq!(coordinator.timeout, timeout);
        assert_eq!(coordinator.services.len(), 0);
    }

    #[tokio::test]
    async fn test_shutdown_coordinator_add_service() {
        let timeout = Duration::from_secs(30);
        let mut coordinator = ShutdownCoordinator::new(timeout);

        let service = Box::new(MockService::new(false, None));
        coordinator.add_service(service);

        assert_eq!(coordinator.services.len(), 1);
    }

    #[tokio::test]
    async fn test_shutdown_coordinator_shutdown_all_success() {
        let timeout = Duration::from_secs(30);
        let mut coordinator = ShutdownCoordinator::new(timeout);

        let service1 = Box::new(MockService::new(false, None));
        let service2 = Box::new(MockService::new(false, None));
        coordinator.add_service(service1);
        coordinator.add_service(service2);

        let result = coordinator.shutdown_all().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_shutdown_coordinator_shutdown_all_with_failure() {
        let timeout = Duration::from_secs(30);
        let mut coordinator = ShutdownCoordinator::new(timeout);

        let service1 = Box::new(MockService::new(false, None));
        let service2 = Box::new(MockService::new(true, None)); // This will fail
        coordinator.add_service(service1);
        coordinator.add_service(service2);

        let result = coordinator.shutdown_all().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_shutdown_coordinator_shutdown_all_timeout() {
        let timeout = Duration::from_millis(100); // Very short timeout
        let mut coordinator = ShutdownCoordinator::new(timeout);

        // Service with delay longer than timeout
        let service = Box::new(MockService::new(false, Some(Duration::from_millis(200))));
        coordinator.add_service(service);

        let result = coordinator.shutdown_all().await;
        assert!(result.is_err());

        if let Err(IndexerError::Service(ServiceError::ShutdownTimeout { seconds })) = result {
            assert_eq!(seconds, 0); // 100ms rounds down to 0 seconds
        } else {
            panic!("Expected ShutdownTimeout error");
        }
    }

    #[tokio::test]
    async fn test_shutdown_coordinator_shutdown_all_empty() {
        let timeout = Duration::from_secs(30);
        let mut coordinator = ShutdownCoordinator::new(timeout);

        let result = coordinator.shutdown_all().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_shutdown_coordinator_shutdown_all_mixed_results() {
        let timeout = Duration::from_secs(30);
        let mut coordinator = ShutdownCoordinator::new(timeout);

        let service1 = Box::new(MockService::new(false, None)); // Success
        let service2 = Box::new(MockService::new(true, None)); // Failure
        let service3 = Box::new(MockService::new(false, None)); // Success
        coordinator.add_service(service1);
        coordinator.add_service(service2);
        coordinator.add_service(service3);

        let result = coordinator.shutdown_all().await;
        assert!(result.is_err());
    }

    // Edge case tests
    #[tokio::test]
    async fn test_service_context_multiple_receivers() {
        let config = create_test_config();
        let store = Arc::new(MockDataStore::new(true));
        let metrics = create_test_metrics();
        let context = ServiceContext::new(config, store, metrics);

        let mut receiver1 = context.shutdown_receiver();
        let mut receiver2 = context.shutdown_receiver();

        context
            .request_shutdown()
            .expect("Request shutdown should succeed");

        // Both receivers should get the signal
        assert!(receiver1.recv().await.is_ok());
        assert!(receiver2.recv().await.is_ok());
    }

    #[tokio::test]
    async fn test_service_context_receiver_dropped() {
        let config = create_test_config();
        let store = Arc::new(MockDataStore::new(true));
        let metrics = create_test_metrics();
        let context = ServiceContext::new(config, store, metrics);

        {
            let _receiver = context.shutdown_receiver();
            // Receiver is dropped here
        }

        // Should still work even with dropped receiver
        assert!(context.request_shutdown().is_ok());
    }

    #[test]
    fn test_component_health_error_accumulation() {
        let mut health = ComponentHealth::healthy("Initial");
        assert_eq!(health.error_count, 0);

        health.record_error("Error 1");
        assert_eq!(health.error_count, 1);

        health.record_error("Error 2");
        assert_eq!(health.error_count, 2);

        health.record_error("Error 3");
        assert_eq!(health.error_count, 3);

        // Success should reset error count
        health.record_success();
        assert_eq!(health.error_count, 0);
    }

    #[tokio::test]
    async fn test_service_context_concurrent_state_updates() {
        let config = create_test_config();
        let store = Arc::new(MockDataStore::new(true));
        let metrics = create_test_metrics();
        let context = Arc::new(ServiceContext::new(config, store, metrics));

        let context1 = context.clone();
        let context2 = context.clone();

        let handle1 = tokio::spawn(async move {
            context1.set_state(ServiceState::Running).await;
        });

        let handle2 = tokio::spawn(async move {
            context2.set_state(ServiceState::Stopping).await;
        });

        let _ = tokio::join!(handle1, handle2);

        // Final state should be one of the two
        let final_state = context.state().await;
        assert!(final_state == ServiceState::Running || final_state == ServiceState::Stopping);
    }

    #[tokio::test]
    async fn test_service_context_concurrent_health_updates() {
        let config = create_test_config();
        let store = Arc::new(MockDataStore::new(true));
        let metrics = create_test_metrics();
        let context = Arc::new(ServiceContext::new(config, store, metrics));

        let context1 = context.clone();
        let context2 = context.clone();

        let handle1 = tokio::spawn(async move {
            context1
                .update_component_health("component1", ComponentHealth::healthy("OK 1"))
                .await;
        });

        let handle2 = tokio::spawn(async move {
            context2
                .update_component_health("component2", ComponentHealth::healthy("OK 2"))
                .await;
        });

        let _ = tokio::join!(handle1, handle2);

        let health = context.health_status().await;
        assert!(health.components.contains_key("component1"));
        assert!(health.components.contains_key("component2"));
    }
}
