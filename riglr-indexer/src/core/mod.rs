//! Core indexer service components

use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tracing::{error, info, warn};

use crate::config::IndexerConfig;
use crate::error::{IndexerError, IndexerResult};
use crate::metrics::MetricsCollector;
use crate::storage::DataStore;

pub mod indexer;
pub mod ingester;
pub mod pipeline;
pub mod processor;

pub use indexer::IndexerService;
pub use ingester::{EventIngester, IngesterConfig};
pub use pipeline::{PipelineResult, PipelineStage, ProcessingPipeline as Pipeline};
pub use processor::{EventProcessor, ProcessingPipeline, ProcessorConfig};

/// Service state enumeration
#[derive(Debug, Clone, PartialEq)]
pub enum ServiceState {
    /// Service is initializing
    Starting,
    /// Service is running normally
    Running,
    /// Service is shutting down gracefully
    Stopping,
    /// Service has stopped
    Stopped,
    /// Service encountered an error
    Error(String),
}

/// Health check status
#[derive(Debug, Clone)]
pub struct HealthStatus {
    /// Overall health status
    pub healthy: bool,
    /// Individual component statuses
    pub components: std::collections::HashMap<String, ComponentHealth>,
    /// Last check timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Component health information
#[derive(Debug, Clone)]
pub struct ComponentHealth {
    /// Component is healthy
    pub healthy: bool,
    /// Status message
    pub message: String,
    /// Last successful operation timestamp
    pub last_success: Option<chrono::DateTime<chrono::Utc>>,
    /// Error count since last success
    pub error_count: u64,
}

impl ComponentHealth {
    /// Create a healthy component status
    pub fn healthy(message: &str) -> Self {
        Self {
            healthy: true,
            message: message.to_string(),
            last_success: Some(chrono::Utc::now()),
            error_count: 0,
        }
    }

    /// Create an unhealthy component status
    pub fn unhealthy(message: &str) -> Self {
        Self {
            healthy: false,
            message: message.to_string(),
            last_success: None,
            error_count: 1,
        }
    }

    /// Record a successful operation
    pub fn record_success(&mut self) {
        self.healthy = true;
        self.last_success = Some(chrono::Utc::now());
        self.error_count = 0;
    }

    /// Record an error
    pub fn record_error(&mut self, message: &str) {
        self.healthy = false;
        self.message = message.to_string();
        self.error_count += 1;
    }
}

/// Shared service context
pub struct ServiceContext {
    /// Service configuration
    pub config: Arc<IndexerConfig>,
    /// Service state
    pub state: Arc<RwLock<ServiceState>>,
    /// Data store
    pub store: Arc<dyn DataStore>,
    /// Metrics collector
    pub metrics: Arc<MetricsCollector>,
    /// Shutdown signal broadcaster
    pub shutdown_tx: broadcast::Sender<()>,
    /// Health status
    pub health: Arc<RwLock<HealthStatus>>,
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
            healthy: true,
            components: std::collections::HashMap::new(),
            timestamp: chrono::Utc::now(),
        };

        Self {
            config: Arc::new(config),
            state: Arc::new(RwLock::new(ServiceState::Starting)),
            store,
            metrics,
            shutdown_tx,
            health: Arc::new(RwLock::new(health)),
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
    pub fn is_shutdown_requested(&self) -> bool {
        self.shutdown_tx.receiver_count() == 0
    }

    /// Request shutdown
    pub fn request_shutdown(&self) -> IndexerResult<()> {
        match self.shutdown_tx.send(()) {
            Ok(_) => {
                info!("Shutdown signal sent");
                Ok(())
            }
            Err(_) => {
                warn!("No shutdown receivers listening");
                Ok(())
            }
        }
    }

    /// Subscribe to shutdown signals
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
    pub async fn health_check(&self) -> IndexerResult<HealthStatus> {
        let mut status = self.health.write().await;

        // Check data store health
        match self.store.health_check().await {
            Ok(_) => {
                status.components.insert(
                    "storage".to_string(),
                    ComponentHealth::healthy("Storage connection OK"),
                );
            }
            Err(e) => {
                status.components.insert(
                    "storage".to_string(),
                    ComponentHealth::unhealthy(&format!("Storage error: {}", e)),
                );
            }
        }

        // Update overall health
        status.healthy = status.components.values().all(|c| c.healthy);
        status.timestamp = chrono::Utc::now();

        Ok(status.clone())
    }
}

/// Service lifecycle trait
#[async_trait::async_trait]
pub trait ServiceLifecycle {
    /// Start the service
    async fn start(&mut self) -> IndexerResult<()>;

    /// Stop the service gracefully
    async fn stop(&mut self) -> IndexerResult<()>;

    /// Get current health status
    async fn health(&self) -> IndexerResult<HealthStatus>;

    /// Check if service is running
    fn is_running(&self) -> bool;
}

/// Event processing trait
#[async_trait::async_trait]
pub trait EventProcessing {
    /// Process a single event
    async fn process_event(&self, event: Box<dyn riglr_events_core::Event>) -> IndexerResult<()>;

    /// Process a batch of events
    async fn process_batch(
        &self,
        events: Vec<Box<dyn riglr_events_core::Event>>,
    ) -> IndexerResult<()>;

    /// Get processing statistics
    async fn processing_stats(&self) -> ProcessingStats;
}

/// Processing statistics
#[derive(Debug, Clone)]
pub struct ProcessingStats {
    /// Total events processed
    pub total_processed: u64,
    /// Events processed per second (current rate)
    pub events_per_second: f64,
    /// Average processing latency in milliseconds
    pub avg_latency_ms: f64,
    /// Error rate (errors per total events)
    pub error_rate: f64,
    /// Queue depth
    pub queue_depth: usize,
    /// Active workers
    pub active_workers: usize,
}

impl Default for ProcessingStats {
    fn default() -> Self {
        Self {
            total_processed: 0,
            events_per_second: 0.0,
            avg_latency_ms: 0.0,
            error_rate: 0.0,
            queue_depth: 0,
            active_workers: 0,
        }
    }
}

/// Graceful shutdown coordinator
pub struct ShutdownCoordinator {
    /// Timeout for graceful shutdown
    timeout: std::time::Duration,
    /// Services to shutdown
    services: Vec<Box<dyn ServiceLifecycle + Send + Sync>>,
}

impl ShutdownCoordinator {
    /// Create a new shutdown coordinator
    pub fn new(timeout: std::time::Duration) -> Self {
        Self {
            timeout,
            services: Vec::new(),
        }
    }

    /// Add a service to coordinate
    pub fn add_service(&mut self, service: Box<dyn ServiceLifecycle + Send + Sync>) {
        self.services.push(service);
    }

    /// Shutdown all services gracefully
    pub async fn shutdown_all(&mut self) -> IndexerResult<()> {
        info!(
            "Starting graceful shutdown of {} services",
            self.services.len()
        );

        let mut results = Vec::new();
        let timeout = self.timeout;

        for (i, service) in self.services.iter_mut().enumerate() {
            let result = match tokio::time::timeout(timeout, service.stop()).await {
                Ok(result) => {
                    match &result {
                        Ok(_) => info!("Service {} shut down successfully", i),
                        Err(ref e) => error!("Service {} shutdown error: {}", i, e),
                    }
                    result
                }
                Err(_) => {
                    error!("Service {} shutdown timeout", i);
                    Err(IndexerError::Service(
                        crate::error::ServiceError::ShutdownTimeout {
                            seconds: timeout.as_secs(),
                        },
                    ))
                }
            };
            results.push((i, result));
        }

        let mut errors = Vec::new();
        for (i, result) in results.into_iter() {
            if let Err(e) = result {
                errors.push((i, e));
            }
        }

        if errors.is_empty() {
            info!("All services shut down successfully");
            Ok(())
        } else {
            error!("Some services failed to shut down: {:?}", errors);
            Err(errors.into_iter().next().unwrap().1)
        }
    }
}
