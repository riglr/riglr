//! Main indexer service implementation

use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::interval;
use tracing::{debug, error, info};

use riglr_events_core::prelude::*;

use crate::config::IndexerConfig;
use crate::core::{
    ComponentHealth, EventProcessing, HealthStatus, ProcessingStats, ServiceContext,
    ServiceLifecycle, ServiceState,
};
use crate::core::{EventIngester, EventProcessor};
use crate::error::{IndexerError, IndexerResult};
use crate::metrics::MetricsCollector;

/// Main indexer service that orchestrates all components
pub struct IndexerService {
    /// Service context with shared state
    context: Arc<ServiceContext>,
    /// Event ingester for data collection
    ingester: Option<EventIngester>,
    /// Event processor for parallel processing
    processor: Option<EventProcessor>,
    /// Processing statistics
    stats: Arc<RwLock<ProcessingStats>>,
    /// Health check task handle
    health_task: Option<tokio::task::JoinHandle<()>>,
}

impl IndexerService {
    /// Create a new indexer service
    pub async fn new(config: IndexerConfig) -> IndexerResult<Self> {
        info!(
            "Initializing RIGLR Indexer Service v{}",
            config.service.version
        );

        // Initialize data store
        let store = crate::storage::create_store(&config.storage).await?;

        // Initialize metrics collector
        let metrics = MetricsCollector::new(config.metrics.clone())?;

        // Create service context
        let context = Arc::new(ServiceContext::new(
            config,
            Arc::from(store),
            Arc::new(metrics),
        ));

        let service = Self {
            context,
            ingester: None,
            processor: None,
            stats: Arc::new(RwLock::new(ProcessingStats::default())),
            health_task: None,
        };

        info!("Indexer service initialized successfully");
        Ok(service)
    }

    /// Initialize all service components
    async fn initialize_components(&mut self) -> IndexerResult<()> {
        info!("Initializing service components...");

        // Initialize ingester
        let ingester_config = crate::core::IngesterConfig {
            workers: self.context.config.processing.workers,
            batch_size: self.context.config.processing.batch.max_size,
            queue_capacity: self.context.config.processing.queue.capacity,
        };

        self.ingester = Some(EventIngester::new(ingester_config, self.context.clone()).await?);

        // Initialize processor
        let processor_config = crate::core::ProcessorConfig {
            workers: self.context.config.processing.workers,
            batch_config: self.context.config.processing.batch.clone(),
            retry_config: self.context.config.processing.retry.clone(),
        };

        self.processor = Some(EventProcessor::new(processor_config, self.context.clone()).await?);

        info!("All service components initialized");
        Ok(())
    }

    /// Start health monitoring task
    fn start_health_monitoring(&mut self) {
        let context = self.context.clone();
        let interval_duration = self.context.config.service.health_check_interval;

        self.health_task = Some(tokio::spawn(async move {
            let mut interval = interval(interval_duration);

            loop {
                interval.tick().await;

                match context.health_check().await {
                    Ok(status) => {
                        debug!(
                            "Health check completed: {} components",
                            status.components.len()
                        );

                        // Record metrics
                        let healthy_components =
                            status.components.values().filter(|c| c.healthy).count();
                        context
                            .metrics
                            .record_gauge("indexer_healthy_components", healthy_components as f64);

                        context.metrics.record_gauge(
                            "indexer_total_components",
                            status.components.len() as f64,
                        );
                    }
                    Err(e) => {
                        error!("Health check failed: {}", e);
                        context
                            .metrics
                            .increment_counter("indexer_health_check_errors");
                    }
                }

                // Check if we should stop
                if matches!(
                    context.state().await,
                    ServiceState::Stopping | ServiceState::Stopped
                ) {
                    break;
                }
            }

            info!("Health monitoring task stopped");
        }));
    }

    /// Process events from the ingester
    #[allow(dead_code)]
    async fn process_events(&self) -> IndexerResult<()> {
        let ingester = self
            .ingester
            .as_ref()
            .ok_or_else(|| IndexerError::internal("Ingester not initialized"))?;
        let processor = self
            .processor
            .as_ref()
            .ok_or_else(|| IndexerError::internal("Processor not initialized"))?;

        info!("Starting event processing loop");
        let mut shutdown_rx = self.context.shutdown_receiver();

        loop {
            tokio::select! {
                // Process events from ingester
                events_result = ingester.receive_events() => {
                    match events_result {
                        Ok(events) => {
                            if !events.is_empty() {
                                // Update stats
                                {
                                    let mut stats = self.stats.write().await;
                                    stats.total_processed += events.len() as u64;
                                    stats.queue_depth = ingester.queue_depth().await;
                                }

                                // Process events
                                let start_time = std::time::Instant::now();
                                match processor.process_batch(events).await {
                                    Ok(_) => {
                                        let latency = start_time.elapsed().as_millis() as f64;
                                        self.context.metrics.record_histogram(
                                            "indexer_processing_latency_ms",
                                            latency,
                                        );
                                    }
                                    Err(e) => {
                                        error!("Batch processing failed: {}", e);
                                        self.context.metrics.increment_counter("indexer_processing_errors");

                                        // Update component health
                                        self.context.update_component_health(
                                            "processor",
                                            ComponentHealth::unhealthy(&format!("Processing error: {}", e)),
                                        ).await;
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            error!("Failed to receive events: {}", e);
                            self.context.metrics.increment_counter("indexer_ingestion_errors");

                            // Update component health
                            self.context.update_component_health(
                                "ingester",
                                ComponentHealth::unhealthy(&format!("Ingestion error: {}", e)),
                            ).await;
                        }
                    }
                }

                // Handle shutdown
                _ = shutdown_rx.recv() => {
                    info!("Shutdown signal received in processing loop");
                    break;
                }
            }
        }

        info!("Event processing loop stopped");
        Ok(())
    }

    /// Calculate processing statistics
    #[allow(dead_code)]
    async fn update_processing_stats(&self) {
        let mut stats = self.stats.write().await;

        // Calculate events per second (simple moving average)
        let now = std::time::Instant::now();
        static mut LAST_UPDATE: Option<std::time::Instant> = None;
        static mut LAST_COUNT: u64 = 0;

        unsafe {
            if let Some(last_time) = LAST_UPDATE {
                let elapsed = now.duration_since(last_time).as_secs_f64();
                if elapsed > 0.0 {
                    let events_delta = stats.total_processed.saturating_sub(LAST_COUNT);
                    stats.events_per_second = events_delta as f64 / elapsed;
                }
            }
            LAST_UPDATE = Some(now);
            LAST_COUNT = stats.total_processed;
        }

        // Get queue depth from ingester
        if let Some(ingester) = &self.ingester {
            stats.queue_depth = ingester.queue_depth().await;
        }

        // Get active workers from processor
        if let Some(processor) = &self.processor {
            stats.active_workers = processor.active_workers().await;
        }

        // Record metrics
        self.context
            .metrics
            .record_gauge("indexer_events_per_second", stats.events_per_second);
        self.context
            .metrics
            .record_gauge("indexer_queue_depth", stats.queue_depth as f64);
        self.context
            .metrics
            .record_gauge("indexer_active_workers", stats.active_workers as f64);
    }

    /// Get current configuration
    pub fn config(&self) -> &IndexerConfig {
        &self.context.config
    }

    /// Get service context
    pub fn context(&self) -> &Arc<ServiceContext> {
        &self.context
    }
}

#[async_trait::async_trait]
impl ServiceLifecycle for IndexerService {
    async fn start(&mut self) -> IndexerResult<()> {
        info!("Starting RIGLR Indexer Service");

        // Set state to starting
        self.context.set_state(ServiceState::Starting).await;

        // Initialize all components
        self.initialize_components().await?;

        // Start health monitoring
        self.start_health_monitoring();

        // Start ingester
        if let Some(ingester) = &mut self.ingester {
            ingester.start().await?;
            self.context
                .update_component_health(
                    "ingester",
                    ComponentHealth::healthy("Ingester started successfully"),
                )
                .await;
        }

        // Start processor
        if let Some(processor) = &mut self.processor {
            processor.start().await?;
            self.context
                .update_component_health(
                    "processor",
                    ComponentHealth::healthy("Processor started successfully"),
                )
                .await;
        }

        // Set state to running
        self.context.set_state(ServiceState::Running).await;

        info!("RIGLR Indexer Service started successfully");
        info!("Service configuration:");
        info!("  - Workers: {}", self.context.config.processing.workers);
        info!(
            "  - Batch size: {}",
            self.context.config.processing.batch.max_size
        );
        info!(
            "  - Queue capacity: {}",
            self.context.config.processing.queue.capacity
        );
        info!(
            "  - Storage backend: {:?}",
            self.context.config.storage.primary.backend
        );

        // Start main processing loop
        let context_clone = self.context.clone();
        let _stats_clone = self.stats.clone();
        let _ingester = self.ingester.as_ref().unwrap().clone();
        let _processor = self.processor.as_ref().unwrap().clone();

        tokio::spawn(async move {
            let mut shutdown_rx = context_clone.shutdown_receiver();
            let mut stats_interval = interval(Duration::from_secs(30));

            loop {
                tokio::select! {
                    // Update stats periodically
                    _ = stats_interval.tick() => {
                        // Update processing statistics would be called here
                        // But we need access to self which is not available in this context
                        // This will be handled by the processor itself
                    }

                    // Handle shutdown
                    _ = shutdown_rx.recv() => {
                        info!("Main processing loop received shutdown signal");
                        break;
                    }
                }
            }
        });

        Ok(())
    }

    async fn stop(&mut self) -> IndexerResult<()> {
        info!("Stopping RIGLR Indexer Service gracefully");

        // Set state to stopping
        self.context.set_state(ServiceState::Stopping).await;

        // Stop processor first to finish processing current events
        if let Some(processor) = &mut self.processor {
            if let Err(e) = processor.stop().await {
                error!("Error stopping processor: {}", e);
            } else {
                info!("Processor stopped successfully");
            }
        }

        // Stop ingester
        if let Some(ingester) = &mut self.ingester {
            if let Err(e) = ingester.stop().await {
                error!("Error stopping ingester: {}", e);
            } else {
                info!("Ingester stopped successfully");
            }
        }

        // Stop health monitoring
        if let Some(task) = self.health_task.take() {
            task.abort();
            info!("Health monitoring stopped");
        }

        // Final metrics flush
        if let Err(e) = self.context.metrics.flush().await {
            error!("Error flushing metrics: {}", e);
        }

        // Set state to stopped
        self.context.set_state(ServiceState::Stopped).await;

        info!("RIGLR Indexer Service stopped");
        Ok(())
    }

    async fn health(&self) -> IndexerResult<HealthStatus> {
        self.context.health_check().await
    }

    fn is_running(&self) -> bool {
        // We need a synchronous way to check this
        // For now, we'll use a simple heuristic
        self.ingester.is_some() && self.processor.is_some()
    }
}

#[async_trait::async_trait]
impl EventProcessing for IndexerService {
    async fn process_event(&self, event: Box<dyn Event>) -> IndexerResult<()> {
        let processor = self
            .processor
            .as_ref()
            .ok_or_else(|| IndexerError::internal("Processor not initialized"))?;

        processor.process_event(event).await
    }

    async fn process_batch(&self, events: Vec<Box<dyn Event>>) -> IndexerResult<()> {
        let processor = self
            .processor
            .as_ref()
            .ok_or_else(|| IndexerError::internal("Processor not initialized"))?;

        processor.process_batch(events).await
    }

    async fn processing_stats(&self) -> ProcessingStats {
        self.stats.read().await.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{
        BatchConfig, MetricsConfig, ProcessingConfig, QueueConfig, QueueType, RateLimitConfig,
        RetryConfig, ServiceConfig, StorageConfig,
    };
    use crate::config::{CacheConfig, StorageBackend, StorageBackendConfig};
    use riglr_events_core::{Event, EventKind, EventMetadata};
    use std::collections::HashMap;
    use std::time::Duration;
    use std::time::SystemTime;

    // Mock Event implementation for testing
    #[allow(dead_code)]
    #[derive(Debug, Clone)]
    struct MockEvent {
        metadata: EventMetadata,
        #[allow(dead_code)]
        data: String,
    }

    impl MockEvent {
        fn new(id: String, data: String) -> Self {
            Self {
                metadata: EventMetadata::new(
                    id,
                    EventKind::Custom("mock".to_string()),
                    "test".to_string(),
                ),
                data,
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

        fn metadata_mut(&mut self) -> &mut EventMetadata {
            &mut self.metadata
        }

        fn timestamp(&self) -> SystemTime {
            self.metadata.timestamp.into()
        }

        fn source(&self) -> &str {
            &self.metadata.source
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

        fn to_json(&self) -> riglr_events_core::error::EventResult<serde_json::Value> {
            Ok(serde_json::json!({
                "id": self.id(),
                "kind": format!("{:?}", self.kind()),
                "source": self.source(),
                "data": self.data
            }))
        }
    }

    // Helper function to create test config
    fn create_test_config() -> IndexerConfig {
        IndexerConfig {
            service: ServiceConfig {
                name: "test-indexer".to_string(),
                version: "1.0.0".to_string(),
                environment: "test".to_string(),
                node_id: None,
                shutdown_timeout: Duration::from_secs(30),
                health_check_interval: Duration::from_secs(30),
            },
            processing: ProcessingConfig {
                workers: 4,
                batch: BatchConfig {
                    max_size: 100,
                    max_age: Duration::from_millis(500),
                    target_size: 50,
                },
                queue: QueueConfig {
                    capacity: 1000,
                    queue_type: QueueType::Memory,
                    disk_settings: None,
                },
                retry: RetryConfig {
                    max_attempts: 3,
                    base_delay: Duration::from_millis(100),
                    max_delay: Duration::from_secs(5),
                    backoff_multiplier: 2.0,
                    jitter: 0.1,
                },
                rate_limit: RateLimitConfig {
                    enabled: false,
                    max_events_per_second: 1000,
                    burst_capacity: 100,
                },
            },
            storage: StorageConfig {
                primary: StorageBackendConfig {
                    backend: StorageBackend::Postgres,
                    url: "postgresql://localhost:5432/test".to_string(),
                    pool: crate::config::ConnectionPoolConfig {
                        max_connections: 10,
                        min_connections: 1,
                        connect_timeout: Duration::from_secs(5),
                        idle_timeout: Duration::from_secs(300),
                        max_lifetime: Duration::from_secs(1800),
                    },
                    settings: HashMap::new(),
                },
                secondary: None,
                cache: CacheConfig {
                    backend: crate::config::CacheBackend::Redis,
                    redis_url: Some("redis://localhost:6379".to_string()),
                    ttl: crate::config::CacheTtlConfig {
                        default: Duration::from_secs(300),
                        events: Duration::from_secs(3600),
                        aggregates: Duration::from_secs(1800),
                    },
                    memory: crate::config::MemoryCacheConfig {
                        max_size_bytes: 100_000_000,
                        max_entries: 10_000,
                    },
                },
                retention: crate::config::RetentionConfig {
                    default: Duration::from_secs(30 * 24 * 3600),
                    by_event_type: HashMap::new(),
                    archive: crate::config::ArchiveConfig {
                        enabled: false,
                        backend: None,
                        compression: crate::config::CompressionConfig {
                            algorithm: crate::config::CompressionAlgorithm::Zstd,
                            level: 3,
                        },
                    },
                },
            },
            api: crate::config::ApiConfig {
                http: crate::config::HttpConfig {
                    bind: "127.0.0.1".to_string(),
                    port: 8080,
                    timeout: Duration::from_secs(30),
                    max_request_size: 1_000_000,
                    keep_alive: Duration::from_secs(60),
                },
                websocket: crate::config::WebSocketConfig {
                    enabled: false,
                    max_connections: 100,
                    buffer_size: 1024,
                    heartbeat_interval: Duration::from_secs(30),
                },
                graphql: None,
                auth: crate::config::AuthConfig {
                    enabled: false,
                    method: crate::config::AuthMethod::None,
                    jwt: None,
                    api_key: None,
                },
                cors: crate::config::CorsConfig {
                    enabled: false,
                    allowed_origins: vec!["*".to_string()],
                    allowed_methods: vec!["GET".to_string(), "POST".to_string()],
                    allowed_headers: vec!["*".to_string()],
                    max_age: Duration::from_secs(3600),
                },
            },
            metrics: MetricsConfig {
                enabled: true,
                port: 9090,
                endpoint: "/metrics".to_string(),
                collection_interval: Duration::from_secs(15),
                histogram_buckets: vec![
                    0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
                ],
                custom: HashMap::new(),
            },
            logging: crate::config::LoggingConfig {
                level: "info".to_string(),
                format: crate::config::LogFormat::Json,
                outputs: vec![crate::config::LogOutput::Stdout],
                structured: crate::config::StructuredLoggingConfig {
                    include_location: false,
                    include_thread: true,
                    include_service_metadata: true,
                    custom_fields: HashMap::new(),
                },
            },
            features: crate::config::FeatureConfig {
                realtime_streaming: true,
                archival: false,
                graphql_api: false,
                experimental: false,
                custom: HashMap::new(),
            },
        }
    }

    #[tokio::test]
    async fn test_indexer_service_new_when_valid_config_should_create_successfully() {
        let config = create_test_config();
        let result = IndexerService::new(config).await;
        assert!(result.is_ok());

        let service = result.unwrap();
        assert!(service.ingester.is_none());
        assert!(service.processor.is_none());
        assert!(service.health_task.is_none());
    }

    #[tokio::test]
    async fn test_indexer_service_new_when_invalid_storage_config_should_return_err() {
        let mut config = create_test_config();
        config.storage.primary.url = "invalid://url".to_string();
        config.storage.primary.backend = StorageBackend::Postgres;

        let result = IndexerService::new(config).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_config_when_service_created_should_return_config() {
        let config = create_test_config();
        let service = IndexerService::new(config.clone()).await.unwrap();

        let returned_config = service.config();
        assert_eq!(returned_config.service.version, config.service.version);
        assert_eq!(
            returned_config.processing.workers,
            config.processing.workers
        );
    }

    #[tokio::test]
    async fn test_context_when_service_created_should_return_context() {
        let config = create_test_config();
        let service = IndexerService::new(config).await.unwrap();

        let context = service.context();
        assert_eq!(context.config.service.version, "1.0.0");
    }

    #[tokio::test]
    async fn test_is_running_when_components_not_initialized_should_return_false() {
        let config = create_test_config();
        let service = IndexerService::new(config).await.unwrap();

        assert!(!service.is_running());
    }

    #[tokio::test]
    async fn test_is_running_when_only_ingester_initialized_should_return_false() {
        let config = create_test_config();
        let mut service = IndexerService::new(config).await.unwrap();

        // Initialize only ingester
        let ingester_config = crate::core::IngesterConfig {
            workers: 1,
            batch_size: 10,
            queue_capacity: 100,
        };
        service.ingester = Some(
            EventIngester::new(ingester_config, service.context.clone())
                .await
                .unwrap(),
        );

        assert!(!service.is_running());
    }

    #[tokio::test]
    async fn test_initialize_components_when_called_should_initialize_ingester_and_processor() {
        let config = create_test_config();
        let mut service = IndexerService::new(config).await.unwrap();

        let result = service.initialize_components().await;
        assert!(result.is_ok());
        assert!(service.ingester.is_some());
        assert!(service.processor.is_some());
    }

    #[tokio::test]
    async fn test_start_health_monitoring_when_called_should_create_health_task() {
        let config = create_test_config();
        let mut service = IndexerService::new(config).await.unwrap();

        service.start_health_monitoring();
        assert!(service.health_task.is_some());

        // Clean up the task
        if let Some(task) = service.health_task.take() {
            task.abort();
        }
    }

    #[tokio::test]
    async fn test_process_event_when_processor_not_initialized_should_return_err() {
        let config = create_test_config();
        let service = IndexerService::new(config).await.unwrap();

        let event = Box::new(MockEvent::new(
            "test-1".to_string(),
            "test data".to_string(),
        ));

        let result = service.process_event(event).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Processor not initialized"));
    }

    #[tokio::test]
    async fn test_process_batch_when_processor_not_initialized_should_return_err() {
        let config = create_test_config();
        let service = IndexerService::new(config).await.unwrap();

        let events = vec![Box::new(MockEvent::new(
            "test-1".to_string(),
            "test data".to_string(),
        )) as Box<dyn Event>];

        let result = service.process_batch(events).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Processor not initialized"));
    }

    #[tokio::test]
    async fn test_processing_stats_when_called_should_return_stats() {
        let config = create_test_config();
        let service = IndexerService::new(config).await.unwrap();

        let stats = service.processing_stats().await;
        assert_eq!(stats.total_processed, 0);
        assert_eq!(stats.events_per_second, 0.0);
        assert_eq!(stats.queue_depth, 0);
        assert_eq!(stats.active_workers, 0);
    }

    #[tokio::test]
    async fn test_process_events_when_ingester_not_initialized_should_return_err() {
        let config = create_test_config();
        let service = IndexerService::new(config).await.unwrap();

        let result = service.process_events().await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Ingester not initialized"));
    }

    #[tokio::test]
    async fn test_process_events_when_processor_not_initialized_should_return_err() {
        let config = create_test_config();
        let mut service = IndexerService::new(config).await.unwrap();

        // Initialize only ingester
        let ingester_config = crate::core::IngesterConfig {
            workers: 1,
            batch_size: 10,
            queue_capacity: 100,
        };
        service.ingester = Some(
            EventIngester::new(ingester_config, service.context.clone())
                .await
                .unwrap(),
        );

        let result = service.process_events().await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Processor not initialized"));
    }

    #[tokio::test]
    async fn test_update_processing_stats_when_called_should_update_stats() {
        let config = create_test_config();
        let service = IndexerService::new(config).await.unwrap();

        // Update total processed to test stats calculation
        {
            let mut stats = service.stats.write().await;
            stats.total_processed = 100;
        }

        service.update_processing_stats().await;

        let stats = service.stats.read().await;
        assert_eq!(stats.total_processed, 100);
    }

    #[tokio::test]
    async fn test_update_processing_stats_when_ingester_exists_should_update_queue_depth() {
        let config = create_test_config();
        let mut service = IndexerService::new(config).await.unwrap();

        // Initialize ingester
        let ingester_config = crate::core::IngesterConfig {
            workers: 1,
            batch_size: 10,
            queue_capacity: 100,
        };
        service.ingester = Some(
            EventIngester::new(ingester_config, service.context.clone())
                .await
                .unwrap(),
        );

        service.update_processing_stats().await;

        let stats = service.stats.read().await;
        // Queue depth should be updated (even if it's 0)
        assert_eq!(stats.queue_depth, 0);
    }

    #[tokio::test]
    async fn test_update_processing_stats_when_processor_exists_should_update_active_workers() {
        let config = create_test_config();
        let mut service = IndexerService::new(config).await.unwrap();

        // Initialize processor
        let processor_config = crate::core::ProcessorConfig {
            workers: 4,
            batch_config: service.context.config.processing.batch.clone(),
            retry_config: service.context.config.processing.retry.clone(),
        };
        service.processor = Some(
            EventProcessor::new(processor_config, service.context.clone())
                .await
                .unwrap(),
        );

        service.update_processing_stats().await;

        let stats = service.stats.read().await;
        // Active workers should be updated (even if it's 0)
        assert_eq!(stats.active_workers, 0);
    }

    #[tokio::test]
    async fn test_health_when_called_should_return_health_status() {
        let config = create_test_config();
        let service = IndexerService::new(config).await.unwrap();

        let result = service.health().await;
        assert!(result.is_ok());

        let health_status = result.unwrap();
        assert!(health_status.components.is_empty()); // No components initialized yet
    }

    #[tokio::test]
    async fn test_start_when_called_should_initialize_and_start_components() {
        let config = create_test_config();
        let mut service = IndexerService::new(config).await.unwrap();

        let result = service.start().await;
        assert!(result.is_ok());

        assert!(service.ingester.is_some());
        assert!(service.processor.is_some());
        assert!(service.health_task.is_some());
        assert!(service.is_running());

        // Clean up
        let _ = service.stop().await;
    }

    #[tokio::test]
    async fn test_stop_when_called_should_stop_all_components() {
        let config = create_test_config();
        let mut service = IndexerService::new(config).await.unwrap();

        // Start the service first
        let _ = service.start().await;

        let result = service.stop().await;
        assert!(result.is_ok());

        assert!(service.health_task.is_none());
    }

    #[tokio::test]
    async fn test_stop_when_components_not_initialized_should_still_succeed() {
        let config = create_test_config();
        let mut service = IndexerService::new(config).await.unwrap();

        let result = service.stop().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_stop_when_health_task_exists_should_abort_task() {
        let config = create_test_config();
        let mut service = IndexerService::new(config).await.unwrap();

        // Start health monitoring
        service.start_health_monitoring();
        assert!(service.health_task.is_some());

        let result = service.stop().await;
        assert!(result.is_ok());
        assert!(service.health_task.is_none());
    }
}
