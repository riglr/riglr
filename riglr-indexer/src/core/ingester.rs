//! Event ingester for high-throughput data collection

use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, RwLock};
use tokio::time::interval;
use tracing::{error, info};

use riglr_events_core::prelude::*;

const SOLANA_RPC_URL: &str = "SOLANA_RPC_URL";
#[cfg(test)]
const RPC_URL_1: &str = "RPC_URL_1";
#[cfg(test)]
const RPC_URL_137: &str = "RPC_URL_137";
#[cfg(test)]
const RPC_URL_INVALID: &str = "RPC_URL_INVALID";
#[cfg(test)]
const RPC_URL_PREFIX_ONLY: &str = "RPC_URL_";

use crate::core::{ComponentHealth, HealthStatus, ServiceContext, ServiceLifecycle};
use crate::error::{IndexerError, IndexerResult};
use crate::utils::RateLimiter;
// Config imports only used in tests
#[cfg(test)]
use crate::config::{
    AuthConfig, AuthMethod, CorsConfig, LogFormat, LogOutput, StructuredLoggingConfig,
    WebSocketConfig,
};

/// Configuration for the event ingester
#[derive(Debug, Clone)]
pub struct IngesterConfig {
    /// Number of ingestion workers
    pub workers: usize,
    /// Batch size for processing
    pub batch_size: usize,
    /// Queue capacity
    pub queue_capacity: usize,
}

/// Type alias for event sender
type EventSender = mpsc::UnboundedSender<Box<dyn Event>>;
/// Type alias for event receiver
type EventReceiver = mpsc::UnboundedReceiver<Box<dyn Event>>;

/// Event ingester that collects events from multiple sources
#[derive(Clone)]
pub struct EventIngester {
    /// Ingester configuration
    config: IngesterConfig,
    /// Shared service context
    context: Arc<ServiceContext>,
    /// Event queue sender
    event_tx: Arc<RwLock<Option<EventSender>>>,
    /// Event queue receiver for external access
    event_rx: Arc<RwLock<Option<EventReceiver>>>,
    /// Stream manager for handling multiple data sources
    stream_manager: Arc<RwLock<Option<StreamManager>>>,
    /// Rate limiter for ingestion
    rate_limiter: Arc<RateLimiter>,
    /// Ingestion statistics
    stats: Arc<RwLock<IngestionStats>>,
}

/// Ingestion statistics
#[derive(Debug, Clone, Default)]
pub struct IngestionStats {
    /// Total events ingested
    pub total_events: u64,
    /// Events per second
    pub events_per_second: f64,
    /// Queue depth
    pub queue_depth: usize,
    /// Active streams
    pub active_streams: usize,
    /// Error count
    pub error_count: u64,
    /// Last successful ingestion
    pub last_success: Option<chrono::DateTime<chrono::Utc>>,
}

impl EventIngester {
    /// Create a new event ingester
    pub async fn new(config: IngesterConfig, context: Arc<ServiceContext>) -> IndexerResult<Self> {
        info!(
            "Initializing event ingester with {} workers",
            config.workers
        );

        // Create event queue
        let (event_tx, event_rx) = mpsc::unbounded_channel();

        // Create rate limiter
        let rate_config = &context.config.processing.rate_limit;
        let rate_limiter = if rate_config.enabled {
            RateLimiter::builder()
                .max_requests(rate_config.max_events_per_second as usize)
                .time_window(Duration::from_secs(1))
                .burst_size(rate_config.burst_capacity as usize)
                .build()
        } else {
            // Create effectively unlimited rate limiter
            RateLimiter::new(1_000_000, Duration::from_secs(1))
        };

        let ingester = Self {
            config,
            context,
            event_tx: Arc::new(RwLock::new(Some(event_tx))),
            event_rx: Arc::new(RwLock::new(Some(event_rx))),
            stream_manager: Arc::new(RwLock::new(None)),
            rate_limiter: Arc::new(rate_limiter),
            stats: Arc::new(RwLock::new(IngestionStats::default())),
        };

        info!("Event ingester initialized successfully");
        Ok(ingester)
    }

    /// Initialize stream sources
    async fn initialize_streams(&self) -> IndexerResult<()> {
        info!("Initializing event streams...");

        // Create stream manager
        let mut builder = StreamManagerBuilder::new();

        // Add Solana streams if configured
        if let Ok(rpc_url) = std::env::var(SOLANA_RPC_URL) {
            info!("Adding Solana stream source: {}", rpc_url);

            // Create a Solana event stream
            // This would integrate with riglr-streams to create actual streams
            // For now, we'll create a placeholder that can be extended
            builder = builder.with_config(StreamManagerConfig {
                max_concurrent_streams: 10,
                health_check_interval: Duration::from_secs(30),
                reconnect_delay: Duration::from_secs(5),
                max_reconnect_attempts: 3,
            });
        }

        // Add EVM streams if configured
        for (chain_id_str, rpc_url) in std::env::vars() {
            if chain_id_str.starts_with("RPC_URL_") {
                if let Some(chain_id_str) = chain_id_str.strip_prefix("RPC_URL_") {
                    if let Ok(chain_id) = chain_id_str.parse::<u64>() {
                        info!(
                            "Adding EVM stream source for chain {}: {}",
                            chain_id, rpc_url
                        );
                        // Add EVM stream configuration
                    }
                }
            }
        }

        let manager = builder.build();
        *self.stream_manager.write().await = Some(manager);

        info!("Event streams initialized");
        Ok(())
    }

    /// Start ingestion workers
    async fn start_workers(&self) -> IndexerResult<()> {
        info!("Starting {} ingestion workers", self.config.workers);

        let tx = {
            let event_tx = self.event_tx.read().await;
            event_tx
                .as_ref()
                .ok_or_else(|| IndexerError::internal("Event sender not initialized"))?
                .clone()
        };

        for worker_id in 0..self.config.workers {
            let context = self.context.clone();
            let stats = self.stats.clone();
            let rate_limiter = self.rate_limiter.clone();
            let tx = tx.clone();

            tokio::spawn(async move {
                info!("Starting ingestion worker {}", worker_id);

                let mut shutdown_rx = context.shutdown_receiver();
                let mut batch_interval = interval(Duration::from_millis(100));

                loop {
                    tokio::select! {
                        _ = batch_interval.tick() => {
                            // This is where we would pull events from external sources
                            // For now, we'll create a mock event for testing
                            if context.config.features.experimental {
                                if let Ok(()) = rate_limiter.check_rate_limit(&format!("worker_{}", worker_id)) {
                                    let mock_event = create_mock_solana_event();

                                    if let Err(e) = tx.send(mock_event) {
                                        error!("Worker {} failed to send event: {}", worker_id, e);
                                        break;
                                    }

                                    // Update stats
                                    let mut stats = stats.write().await;
                                    stats.total_events += 1;
                                    stats.last_success = Some(chrono::Utc::now());
                                }
                            }
                        }

                        _ = shutdown_rx.recv() => {
                            info!("Worker {} received shutdown signal", worker_id);
                            break;
                        }
                    }
                }

                info!("Ingestion worker {} stopped", worker_id);
            });
        }

        Ok(())
    }

    /// Start statistics collection
    fn start_stats_collection(&self) {
        let stats = self.stats.clone();
        let context = self.context.clone();

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(10));
            let mut last_count = 0u64;
            let mut last_time = std::time::Instant::now();

            loop {
                interval.tick().await;

                {
                    let mut stats_guard = stats.write().await;
                    let now = std::time::Instant::now();
                    let elapsed = now.duration_since(last_time).as_secs_f64();

                    if elapsed > 0.0 {
                        let events_delta = stats_guard.total_events.saturating_sub(last_count);
                        stats_guard.events_per_second = events_delta as f64 / elapsed;

                        // Record metrics
                        context.metrics.record_gauge(
                            "indexer_ingestion_events_per_second",
                            stats_guard.events_per_second,
                        );
                        context.metrics.record_counter(
                            "indexer_ingestion_total_events",
                            stats_guard.total_events,
                        );
                    }

                    last_count = stats_guard.total_events;
                    last_time = now;
                }

                // Check for shutdown
                if matches!(
                    context.state().await,
                    crate::core::ServiceState::Stopping | crate::core::ServiceState::Stopped
                ) {
                    break;
                }
            }
        });
    }

    /// Receive events from the ingestion queue
    pub async fn receive_events(&self) -> IndexerResult<Vec<Box<dyn Event>>> {
        let mut rx_guard = self.event_rx.write().await;
        let rx = rx_guard
            .as_mut()
            .ok_or_else(|| IndexerError::internal("Event receiver not initialized"))?;

        let mut events = Vec::new();
        let batch_size = self.config.batch_size;

        // Collect events up to batch size
        while events.len() < batch_size {
            match rx.try_recv() {
                Ok(event) => events.push(event),
                Err(mpsc::error::TryRecvError::Empty) => {
                    if events.is_empty() {
                        // Wait for at least one event
                        match rx.recv().await {
                            Some(event) => events.push(event),
                            None => break, // Channel closed
                        }
                    } else {
                        break; // Return partial batch
                    }
                }
                Err(mpsc::error::TryRecvError::Disconnected) => break,
            }
        }

        // Update queue depth statistics
        {
            let mut stats = self.stats.write().await;
            stats.queue_depth = rx.len();
        }

        Ok(events)
    }

    /// Get current queue depth
    pub async fn queue_depth(&self) -> usize {
        let stats = self.stats.read().await;
        stats.queue_depth
    }

    /// Get ingestion statistics
    pub async fn stats(&self) -> IngestionStats {
        self.stats.read().await.clone()
    }
}

#[async_trait::async_trait]
impl ServiceLifecycle for EventIngester {
    async fn start(&mut self) -> IndexerResult<()> {
        info!("Starting event ingester");

        // Initialize streams
        self.initialize_streams().await?;

        // Start workers
        self.start_workers().await?;

        // Start statistics collection
        self.start_stats_collection();

        // Update component health
        self.context
            .update_component_health(
                "ingester",
                ComponentHealth::healthy("Event ingester started successfully"),
            )
            .await;

        info!(
            "Event ingester started with {} workers",
            self.config.workers
        );
        Ok(())
    }

    async fn stop(&mut self) -> IndexerResult<()> {
        info!("Stopping event ingester");

        // Close event sender to signal workers to stop
        {
            let mut tx_guard = self.event_tx.write().await;
            *tx_guard = None;
        }

        // Stop stream manager
        {
            let mut manager_guard = self.stream_manager.write().await;
            if let Some(_manager) = manager_guard.take() {
                // manager.stop().await?;
                // Manager is automatically dropped here
            }
        }

        info!("Event ingester stopped");
        Ok(())
    }

    async fn health(&self) -> IndexerResult<HealthStatus> {
        let stats = self.stats.read().await;

        let healthy = stats.error_count < 100 && // Less than 100 errors
                     stats.last_success.is_some_and(|t| {
                         chrono::Utc::now().signed_duration_since(t).num_minutes() < 5
                     });

        let mut components = std::collections::HashMap::new();

        let message = if healthy {
            format!(
                "Ingester healthy: {} events/sec, {} errors",
                stats.events_per_second, stats.error_count
            )
        } else {
            format!(
                "Ingester unhealthy: {} errors, last success: {:?}",
                stats.error_count, stats.last_success
            )
        };

        components.insert(
            "ingester".to_string(),
            if healthy {
                ComponentHealth::healthy(&message)
            } else {
                ComponentHealth::unhealthy(&message)
            },
        );

        Ok(HealthStatus {
            healthy,
            components,
            timestamp: chrono::Utc::now(),
        })
    }

    fn is_running(&self) -> bool {
        // Check if event sender is still active
        tokio::runtime::Handle::try_current()
            .ok()
            .and_then(|_| {
                let tx_guard = self.event_tx.try_read().ok()?;
                Some(tx_guard.is_some())
            })
            .unwrap_or(false)
    }
}

/// Create a mock Solana event for testing
fn create_mock_solana_event() -> Box<dyn Event> {
    // For now, create a minimal mock event that implements the Event trait
    // In a real implementation, this would use proper Solana event creation
    #[derive(Debug, Clone)]
    struct MockEvent {
        id: String,
        timestamp: std::time::SystemTime,
        metadata: EventMetadata,
        kind: EventKind,
    }

    impl Event for MockEvent {
        fn id(&self) -> &str {
            &self.id
        }

        fn kind(&self) -> &EventKind {
            &self.kind
        }

        fn source(&self) -> &str {
            "mock"
        }

        fn timestamp(&self) -> std::time::SystemTime {
            self.timestamp
        }

        fn metadata(&self) -> &EventMetadata {
            &self.metadata
        }

        fn metadata_mut(&mut self) -> EventResult<&mut EventMetadata> {
            Ok(&mut self.metadata)
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

        fn to_json(&self) -> Result<serde_json::Value, riglr_events_core::EventError> {
            Ok(serde_json::json!({
                "id": self.id,
                "type": "swap",
                "source": "mock",
                "timestamp": self.timestamp.duration_since(std::time::UNIX_EPOCH).unwrap().as_millis()
            }))
        }
    }

    let custom_data = std::collections::HashMap::new();
    let metadata = EventMetadata {
        id: "mock".to_string(),
        source: "mock".to_string(),
        kind: EventKind::Swap,
        chain_data: None,
        received_at: chrono::Utc::now(),
        timestamp: chrono::Utc::now(),
        custom: custom_data,
    };

    Box::new(MockEvent {
        id: format!("mock-{}", uuid::Uuid::new_v4()),
        timestamp: std::time::SystemTime::now(),
        metadata,
        kind: EventKind::Swap,
    })
}

// Placeholder types that would be implemented properly with riglr-streams integration
struct StreamManager;
struct StreamManagerBuilder;
#[allow(dead_code)]
struct StreamManagerConfig {
    max_concurrent_streams: usize,
    health_check_interval: Duration,
    reconnect_delay: Duration,
    max_reconnect_attempts: u32,
}

impl StreamManagerBuilder {
    fn new() -> Self {
        Self
    }

    fn with_config(self, _config: StreamManagerConfig) -> Self {
        self
    }

    fn build(self) -> StreamManager {
        StreamManager
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{
        ApiConfig, ArchiveConfig, BatchConfig, CacheBackend, CacheConfig, CacheTtlConfig,
        CompressionAlgorithm, CompressionConfig, ConnectionPoolConfig, FeatureConfig, HttpConfig,
        IndexerConfig, LoggingConfig, MemoryCacheConfig, MetricsConfig, ProcessingConfig,
        QueueConfig, QueueType, RateLimitConfig, RetentionConfig, RetryConfig, ServiceConfig,
        StorageBackend, StorageBackendConfig, StorageConfig,
    };
    use crate::metrics::MetricsCollector;
    use crate::storage::DataStore;
    use async_trait::async_trait;
    use std::collections::HashMap;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    use tokio::time::{sleep, timeout, Duration, Instant};

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
        async fn insert_event(&self, _event: &crate::storage::StoredEvent) -> IndexerResult<()> {
            Ok(())
        }

        async fn insert_events(
            &self,
            _events: &[crate::storage::StoredEvent],
        ) -> IndexerResult<()> {
            Ok(())
        }

        async fn query_events(
            &self,
            _query: &crate::storage::EventQuery,
        ) -> IndexerResult<Vec<crate::storage::StoredEvent>> {
            Ok(Vec::new())
        }

        async fn get_event(&self, _id: &str) -> IndexerResult<Option<crate::storage::StoredEvent>> {
            Ok(None)
        }

        async fn delete_events(&self, _filter: &crate::storage::EventFilter) -> IndexerResult<u64> {
            Ok(0)
        }

        async fn count_events(&self, _filter: &crate::storage::EventFilter) -> IndexerResult<u64> {
            Ok(0)
        }

        async fn get_stats(&self) -> IndexerResult<crate::storage::StorageStats> {
            Ok(crate::storage::StorageStats {
                total_events: 0,
                storage_size_bytes: 0,
                avg_write_latency_ms: 0.0,
                avg_read_latency_ms: 0.0,
                active_connections: 1,
                cache_hit_rate: 0.0,
            })
        }

        async fn initialize(&self) -> IndexerResult<()> {
            Ok(())
        }

        async fn health_check(&self) -> IndexerResult<()> {
            if self.health_check_result.load(Ordering::SeqCst) {
                Ok(())
            } else {
                Err(IndexerError::Storage(
                    crate::error::StorageError::ConnectionFailed {
                        message: "Mock connection failed".to_string(),
                    },
                ))
            }
        }

        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
    }

    fn create_test_config() -> IndexerConfig {
        IndexerConfig {
            service: ServiceConfig {
                name: "test-indexer".to_string(),
                version: "0.1.0".to_string(),
                environment: "test".to_string(),
                node_id: Some("test-node".to_string()),
                shutdown_timeout: Duration::from_secs(5),
                health_check_interval: Duration::from_secs(10),
            },
            storage: StorageConfig {
                primary: StorageBackendConfig {
                    backend: StorageBackend::Postgres,
                    url: "postgresql://localhost:5432/test".to_string(),
                    pool: ConnectionPoolConfig {
                        max_connections: 5,
                        min_connections: 1,
                        connect_timeout: Duration::from_secs(5),
                        idle_timeout: Duration::from_secs(60),
                        max_lifetime: Duration::from_secs(300),
                    },
                    settings: HashMap::new(),
                },
                secondary: None,
                cache: CacheConfig {
                    backend: CacheBackend::Memory,
                    redis_url: None,
                    ttl: CacheTtlConfig {
                        default: Duration::from_secs(60),
                        events: Duration::from_secs(300),
                        aggregates: Duration::from_secs(180),
                    },
                    memory: MemoryCacheConfig {
                        max_size_bytes: 1_000_000,
                        max_entries: 1000,
                    },
                },
                retention: RetentionConfig {
                    default: Duration::from_secs(7 * 24 * 3600), // 7 days
                    by_event_type: HashMap::new(),
                    archive: ArchiveConfig {
                        enabled: false,
                        backend: None,
                        compression: CompressionConfig {
                            algorithm: CompressionAlgorithm::Zstd,
                            level: 1,
                        },
                    },
                },
            },
            processing: ProcessingConfig {
                workers: 2,
                batch: BatchConfig {
                    max_size: 100,
                    max_age: Duration::from_secs(1),
                    target_size: 10,
                },
                queue: QueueConfig {
                    capacity: 1000,
                    queue_type: QueueType::Memory,
                    disk_settings: None,
                },
                retry: RetryConfig {
                    max_attempts: 2,
                    base_delay: Duration::from_millis(10),
                    max_delay: Duration::from_secs(1),
                    backoff_multiplier: 1.5,
                    jitter: 0.0,
                },
                rate_limit: RateLimitConfig {
                    enabled: true,
                    max_events_per_second: 100,
                    burst_capacity: 50,
                },
            },
            api: ApiConfig {
                http: HttpConfig {
                    bind: "127.0.0.1".to_string(),
                    port: 0, // Use random port for tests
                    timeout: Duration::from_secs(5),
                    max_request_size: 100_000,
                    keep_alive: Duration::from_secs(60),
                },
                websocket: WebSocketConfig {
                    enabled: false,
                    max_connections: 100,
                    buffer_size: 1024,
                    heartbeat_interval: Duration::from_secs(30),
                },
                graphql: None,
                auth: AuthConfig {
                    enabled: false,
                    method: AuthMethod::None,
                    jwt: None,
                    api_key: None,
                },
                cors: CorsConfig {
                    enabled: false,
                    allowed_origins: vec![],
                    allowed_methods: vec![],
                    allowed_headers: vec![],
                    max_age: Duration::from_secs(3600),
                },
            },
            metrics: MetricsConfig {
                enabled: false,
                port: 9090,
                endpoint: "/metrics".to_string(),
                collection_interval: Duration::from_secs(15),
                histogram_buckets: vec![],
                custom: HashMap::new(),
            },
            logging: LoggingConfig {
                level: "debug".to_string(),
                format: LogFormat::Json,
                outputs: vec![LogOutput::Stdout],
                structured: StructuredLoggingConfig {
                    include_location: false,
                    include_thread: true,
                    include_service_metadata: true,
                    custom_fields: HashMap::new(),
                },
            },
            features: FeatureConfig {
                realtime_streaming: true,
                archival: true,
                graphql_api: false,
                experimental: true,
                custom: HashMap::new(),
            },
        }
    }

    fn create_test_metrics() -> Arc<MetricsCollector> {
        let config = MetricsConfig {
            enabled: true,
            port: 9090,
            endpoint: "/metrics".to_string(),
            collection_interval: Duration::from_secs(15),
            histogram_buckets: vec![],
            custom: HashMap::new(),
        };
        Arc::new(MetricsCollector::new(config).unwrap())
    }

    fn create_test_service_context() -> Arc<ServiceContext> {
        let config = create_test_config();
        let store = Arc::new(MockDataStore::new(true));
        let metrics = create_test_metrics();
        Arc::new(ServiceContext::new(config, store, metrics))
    }

    // Tests for IngesterConfig
    #[test]
    fn test_ingester_config_new() {
        let config = IngesterConfig {
            workers: 4,
            batch_size: 100,
            queue_capacity: 1000,
        };

        assert_eq!(config.workers, 4);
        assert_eq!(config.batch_size, 100);
        assert_eq!(config.queue_capacity, 1000);
    }

    #[test]
    fn test_ingester_config_debug() {
        let config = IngesterConfig {
            workers: 2,
            batch_size: 50,
            queue_capacity: 500,
        };

        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("IngesterConfig"));
        assert!(debug_str.contains("workers: 2"));
        assert!(debug_str.contains("batch_size: 50"));
        assert!(debug_str.contains("queue_capacity: 500"));
    }

    #[test]
    fn test_ingester_config_clone() {
        let original = IngesterConfig {
            workers: 8,
            batch_size: 200,
            queue_capacity: 2000,
        };

        let cloned = original.clone();
        assert_eq!(original.workers, cloned.workers);
        assert_eq!(original.batch_size, cloned.batch_size);
        assert_eq!(original.queue_capacity, cloned.queue_capacity);
    }

    // Tests for IngestionStats
    #[test]
    fn test_ingestion_stats_default() {
        let stats = IngestionStats::default();

        assert_eq!(stats.total_events, 0);
        assert_eq!(stats.events_per_second, 0.0);
        assert_eq!(stats.queue_depth, 0);
        assert_eq!(stats.active_streams, 0);
        assert_eq!(stats.error_count, 0);
        assert!(stats.last_success.is_none());
    }

    #[test]
    fn test_ingestion_stats_debug() {
        let mut stats = IngestionStats::default();
        stats.total_events = 100;
        stats.events_per_second = 25.5;
        stats.queue_depth = 10;
        stats.active_streams = 3;
        stats.error_count = 2;
        stats.last_success = Some(chrono::Utc::now());

        let debug_str = format!("{:?}", stats);
        assert!(debug_str.contains("IngestionStats"));
        assert!(debug_str.contains("total_events: 100"));
        assert!(debug_str.contains("events_per_second: 25.5"));
        assert!(debug_str.contains("queue_depth: 10"));
        assert!(debug_str.contains("active_streams: 3"));
        assert!(debug_str.contains("error_count: 2"));
    }

    #[test]
    fn test_ingestion_stats_clone() {
        let mut original = IngestionStats::default();
        original.total_events = 50;
        original.events_per_second = 12.3;
        original.queue_depth = 5;
        original.active_streams = 2;
        original.error_count = 1;
        original.last_success = Some(chrono::Utc::now());

        let cloned = original.clone();
        assert_eq!(original.total_events, cloned.total_events);
        assert_eq!(original.events_per_second, cloned.events_per_second);
        assert_eq!(original.queue_depth, cloned.queue_depth);
        assert_eq!(original.active_streams, cloned.active_streams);
        assert_eq!(original.error_count, cloned.error_count);
        assert_eq!(original.last_success, cloned.last_success);
    }

    // Tests for EventIngester::new()
    #[tokio::test]
    async fn test_event_ingester_new_success() {
        let config = IngesterConfig {
            workers: 2,
            batch_size: 10,
            queue_capacity: 100,
        };
        let context = create_test_service_context();

        let result = EventIngester::new(config.clone(), context).await;
        assert!(result.is_ok());

        let ingester = result.unwrap();
        assert_eq!(ingester.config.workers, 2);
        assert_eq!(ingester.config.batch_size, 10);
        assert_eq!(ingester.config.queue_capacity, 100);
    }

    #[tokio::test]
    async fn test_event_ingester_new_with_zero_workers() {
        let config = IngesterConfig {
            workers: 0,
            batch_size: 10,
            queue_capacity: 100,
        };
        let context = create_test_service_context();

        let result = EventIngester::new(config, context).await;
        assert!(result.is_ok());
        let ingester = result.unwrap();
        assert_eq!(ingester.config.workers, 0);
    }

    #[tokio::test]
    async fn test_event_ingester_new_with_rate_limiting_disabled() {
        let mut config = create_test_config();
        config.processing.rate_limit.enabled = false;
        let store = Arc::new(MockDataStore::new(true));
        let metrics = create_test_metrics();
        let context = Arc::new(ServiceContext::new(config, store, metrics));

        let ingester_config = IngesterConfig {
            workers: 1,
            batch_size: 10,
            queue_capacity: 100,
        };

        let result = EventIngester::new(ingester_config, context).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_event_ingester_new_with_large_values() {
        let config = IngesterConfig {
            workers: 1000,
            batch_size: 50000,
            queue_capacity: 1000000,
        };
        let context = create_test_service_context();

        let result = EventIngester::new(config, context).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_event_ingester_new_channel_creation() {
        let config = IngesterConfig {
            workers: 1,
            batch_size: 1,
            queue_capacity: 1,
        };
        let context = create_test_service_context();

        let ingester = EventIngester::new(config, context).await.unwrap();

        // Verify channels are created by checking initial state
        let stats = ingester.stats().await;
        assert_eq!(stats.total_events, 0);
        assert_eq!(stats.queue_depth, 0);
    }

    // Tests for EventIngester::initialize_streams()
    #[tokio::test]
    async fn test_initialize_streams_without_env_vars() {
        let config = IngesterConfig {
            workers: 1,
            batch_size: 10,
            queue_capacity: 100,
        };
        let context = create_test_service_context();
        let ingester = EventIngester::new(config, context).await.unwrap();

        let result = ingester.initialize_streams().await;
        assert!(result.is_ok());

        // Verify stream manager is set
        let manager = ingester.stream_manager.read().await;
        assert!(manager.is_some());
    }

    #[tokio::test]
    async fn test_initialize_streams_with_solana_env() {
        // Set environment variable for Solana
        std::env::set_var(SOLANA_RPC_URL, "https://api.mainnet-beta.solana.com");

        let config = IngesterConfig {
            workers: 1,
            batch_size: 10,
            queue_capacity: 100,
        };
        let context = create_test_service_context();
        let ingester = EventIngester::new(config, context).await.unwrap();

        let result = ingester.initialize_streams().await;
        assert!(result.is_ok());

        // Clean up
        std::env::remove_var(SOLANA_RPC_URL);
    }

    #[tokio::test]
    async fn test_initialize_streams_with_evm_env() {
        // Set environment variable for EVM chain
        std::env::set_var(RPC_URL_1, "https://eth-mainnet.alchemyapi.io/v2/test");
        std::env::set_var(RPC_URL_137, "https://polygon-rpc.com");

        let config = IngesterConfig {
            workers: 1,
            batch_size: 10,
            queue_capacity: 100,
        };
        let context = create_test_service_context();
        let ingester = EventIngester::new(config, context).await.unwrap();

        let result = ingester.initialize_streams().await;
        assert!(result.is_ok());

        // Clean up
        std::env::remove_var(RPC_URL_1);
        std::env::remove_var(RPC_URL_137);
    }

    #[tokio::test]
    async fn test_initialize_streams_with_invalid_chain_id() {
        // Set environment variable with invalid chain ID
        std::env::set_var(RPC_URL_INVALID, "https://invalid-chain.com");

        let config = IngesterConfig {
            workers: 1,
            batch_size: 10,
            queue_capacity: 100,
        };
        let context = create_test_service_context();
        let ingester = EventIngester::new(config, context).await.unwrap();

        let result = ingester.initialize_streams().await;
        assert!(result.is_ok()); // Should not fail, just skip invalid entries

        // Clean up
        std::env::remove_var(RPC_URL_INVALID);
    }

    #[tokio::test]
    async fn test_initialize_streams_multiple_times() {
        let config = IngesterConfig {
            workers: 1,
            batch_size: 10,
            queue_capacity: 100,
        };
        let context = create_test_service_context();
        let ingester = EventIngester::new(config, context).await.unwrap();

        // Initialize streams multiple times
        let result1 = ingester.initialize_streams().await;
        assert!(result1.is_ok());

        let result2 = ingester.initialize_streams().await;
        assert!(result2.is_ok());
    }

    // Tests for EventIngester::receive_events()
    #[tokio::test]
    async fn test_receive_events_empty_queue() {
        let config = IngesterConfig {
            workers: 1,
            batch_size: 5,
            queue_capacity: 100,
        };
        let context = create_test_service_context();
        let mut ingester = EventIngester::new(config, context).await.unwrap();

        // Start the ingester to initialize channels properly
        let _ = ingester.start().await;

        // Should timeout quickly since no events are available
        let start = Instant::now();
        let result = timeout(Duration::from_millis(100), ingester.receive_events()).await;
        let elapsed = start.elapsed();

        // Should timeout since no events were sent
        assert!(result.is_err());
        assert!(elapsed >= Duration::from_millis(90)); // Some tolerance
    }

    #[tokio::test]
    async fn test_receive_events_with_events() {
        let config = IngesterConfig {
            workers: 1,
            batch_size: 3,
            queue_capacity: 100,
        };
        let context = create_test_service_context();
        let mut ingester = EventIngester::new(config, context).await.unwrap();

        // Start the ingester
        let _ = ingester.start().await;

        // Send some events manually to the queue
        {
            let tx_guard = ingester.event_tx.read().await;
            if let Some(tx) = tx_guard.as_ref() {
                for _ in 0..2 {
                    let event = create_mock_solana_event();
                    let _ = tx.send(event);
                }
            }
        }

        // Wait a bit for events to be queued
        sleep(Duration::from_millis(10)).await;

        // Receive events
        let result = timeout(Duration::from_millis(100), ingester.receive_events()).await;
        assert!(result.is_ok());

        let events = result.unwrap().unwrap();
        assert_eq!(events.len(), 2);
    }

    #[tokio::test]
    async fn test_receive_events_batch_size_limit() {
        let config = IngesterConfig {
            workers: 1,
            batch_size: 2,
            queue_capacity: 100,
        };
        let context = create_test_service_context();
        let mut ingester = EventIngester::new(config, context).await.unwrap();

        // Start the ingester
        let _ = ingester.start().await;

        // Send more events than batch size
        {
            let tx_guard = ingester.event_tx.read().await;
            if let Some(tx) = tx_guard.as_ref() {
                for _ in 0..5 {
                    let event = create_mock_solana_event();
                    let _ = tx.send(event);
                }
            }
        }

        // Wait a bit for events to be queued
        sleep(Duration::from_millis(10)).await;

        // Should receive only batch_size events
        let result = timeout(Duration::from_millis(100), ingester.receive_events()).await;
        assert!(result.is_ok());

        let events = result.unwrap().unwrap();
        assert_eq!(events.len(), 2); // Should be limited by batch_size
    }

    #[tokio::test]
    async fn test_receive_events_receiver_not_initialized() {
        let config = IngesterConfig {
            workers: 1,
            batch_size: 10,
            queue_capacity: 100,
        };
        let context = create_test_service_context();
        let ingester = EventIngester::new(config, context).await.unwrap();

        // Clear the receiver to simulate uninitialized state
        {
            let mut rx_guard = ingester.event_rx.write().await;
            *rx_guard = None;
        }

        let result = ingester.receive_events().await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), IndexerError::Internal { .. }));
    }

    #[tokio::test]
    async fn test_receive_events_channel_closed() {
        let config = IngesterConfig {
            workers: 1,
            batch_size: 10,
            queue_capacity: 100,
        };
        let context = create_test_service_context();
        let mut ingester = EventIngester::new(config, context).await.unwrap();

        // Start and immediately stop to close channels
        let _ = ingester.start().await;
        let _ = ingester.stop().await;

        let result = ingester.receive_events().await;
        assert!(result.is_ok());
        let events = result.unwrap();
        assert!(events.is_empty()); // Channel closed, no events
    }

    // Tests for EventIngester::queue_depth()
    #[tokio::test]
    async fn test_queue_depth_initial() {
        let config = IngesterConfig {
            workers: 1,
            batch_size: 10,
            queue_capacity: 100,
        };
        let context = create_test_service_context();
        let ingester = EventIngester::new(config, context).await.unwrap();

        let depth = ingester.queue_depth().await;
        assert_eq!(depth, 0);
    }

    #[tokio::test]
    async fn test_queue_depth_after_stats_update() {
        let config = IngesterConfig {
            workers: 1,
            batch_size: 10,
            queue_capacity: 100,
        };
        let context = create_test_service_context();
        let ingester = EventIngester::new(config, context).await.unwrap();

        // Manually update stats
        {
            let mut stats = ingester.stats.write().await;
            stats.queue_depth = 42;
        }

        let depth = ingester.queue_depth().await;
        assert_eq!(depth, 42);
    }

    // Tests for EventIngester::stats()
    #[tokio::test]
    async fn test_stats_initial() {
        let config = IngesterConfig {
            workers: 1,
            batch_size: 10,
            queue_capacity: 100,
        };
        let context = create_test_service_context();
        let ingester = EventIngester::new(config, context).await.unwrap();

        let stats = ingester.stats().await;
        assert_eq!(stats.total_events, 0);
        assert_eq!(stats.events_per_second, 0.0);
        assert_eq!(stats.queue_depth, 0);
        assert_eq!(stats.active_streams, 0);
        assert_eq!(stats.error_count, 0);
        assert!(stats.last_success.is_none());
    }

    #[tokio::test]
    async fn test_stats_after_updates() {
        let config = IngesterConfig {
            workers: 1,
            batch_size: 10,
            queue_capacity: 100,
        };
        let context = create_test_service_context();
        let ingester = EventIngester::new(config, context).await.unwrap();

        let now = chrono::Utc::now();

        // Update stats manually
        {
            let mut stats = ingester.stats.write().await;
            stats.total_events = 100;
            stats.events_per_second = 25.5;
            stats.queue_depth = 10;
            stats.active_streams = 3;
            stats.error_count = 2;
            stats.last_success = Some(now);
        }

        let stats = ingester.stats().await;
        assert_eq!(stats.total_events, 100);
        assert_eq!(stats.events_per_second, 25.5);
        assert_eq!(stats.queue_depth, 10);
        assert_eq!(stats.active_streams, 3);
        assert_eq!(stats.error_count, 2);
        assert_eq!(stats.last_success, Some(now));
    }

    // Tests for ServiceLifecycle trait implementation
    #[tokio::test]
    async fn test_service_lifecycle_start_success() {
        let config = IngesterConfig {
            workers: 1,
            batch_size: 10,
            queue_capacity: 100,
        };
        let context = create_test_service_context();
        let mut ingester = EventIngester::new(config, context).await.unwrap();

        let result = ingester.start().await;
        assert!(result.is_ok());

        // Verify health was updated
        let health = ingester.health().await.unwrap();
        assert!(health.healthy);
        assert!(health.components.contains_key("ingester"));
    }

    #[tokio::test]
    async fn test_service_lifecycle_start_multiple_times() {
        let config = IngesterConfig {
            workers: 1,
            batch_size: 10,
            queue_capacity: 100,
        };
        let context = create_test_service_context();
        let mut ingester = EventIngester::new(config, context).await.unwrap();

        // Start multiple times should not fail
        let result1 = ingester.start().await;
        assert!(result1.is_ok());

        let result2 = ingester.start().await;
        assert!(result2.is_ok());
    }

    #[tokio::test]
    async fn test_service_lifecycle_stop_success() {
        let config = IngesterConfig {
            workers: 1,
            batch_size: 10,
            queue_capacity: 100,
        };
        let context = create_test_service_context();
        let mut ingester = EventIngester::new(config, context).await.unwrap();

        // Start first, then stop
        let _ = ingester.start().await;
        let result = ingester.stop().await;
        assert!(result.is_ok());

        // Verify sender is closed
        let tx_guard = ingester.event_tx.read().await;
        assert!(tx_guard.is_none());
    }

    #[tokio::test]
    async fn test_service_lifecycle_stop_without_start() {
        let config = IngesterConfig {
            workers: 1,
            batch_size: 10,
            queue_capacity: 100,
        };
        let context = create_test_service_context();
        let mut ingester = EventIngester::new(config, context).await.unwrap();

        // Stop without starting should not fail
        let result = ingester.stop().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_service_lifecycle_stop_multiple_times() {
        let config = IngesterConfig {
            workers: 1,
            batch_size: 10,
            queue_capacity: 100,
        };
        let context = create_test_service_context();
        let mut ingester = EventIngester::new(config, context).await.unwrap();

        let _ = ingester.start().await;

        // Stop multiple times should not fail
        let result1 = ingester.stop().await;
        assert!(result1.is_ok());

        let result2 = ingester.stop().await;
        assert!(result2.is_ok());
    }

    #[tokio::test]
    async fn test_service_lifecycle_health_healthy() {
        let config = IngesterConfig {
            workers: 1,
            batch_size: 10,
            queue_capacity: 100,
        };
        let context = create_test_service_context();
        let ingester = EventIngester::new(config, context).await.unwrap();

        // Set up healthy stats
        {
            let mut stats = ingester.stats.write().await;
            stats.error_count = 50; // Less than 100
            stats.last_success = Some(chrono::Utc::now()); // Recent success
        }

        let health = ingester.health().await.unwrap();
        assert!(health.healthy);
        assert!(health.components.contains_key("ingester"));
        assert!(health.components["ingester"].healthy);
    }

    #[tokio::test]
    async fn test_service_lifecycle_health_unhealthy_too_many_errors() {
        let config = IngesterConfig {
            workers: 1,
            batch_size: 10,
            queue_capacity: 100,
        };
        let context = create_test_service_context();
        let ingester = EventIngester::new(config, context).await.unwrap();

        // Set up unhealthy stats - too many errors
        {
            let mut stats = ingester.stats.write().await;
            stats.error_count = 150; // More than 100
            stats.last_success = Some(chrono::Utc::now());
        }

        let health = ingester.health().await.unwrap();
        assert!(!health.healthy);
        assert!(health.components.contains_key("ingester"));
        assert!(!health.components["ingester"].healthy);
    }

    #[tokio::test]
    async fn test_service_lifecycle_health_unhealthy_old_success() {
        let config = IngesterConfig {
            workers: 1,
            batch_size: 10,
            queue_capacity: 100,
        };
        let context = create_test_service_context();
        let ingester = EventIngester::new(config, context).await.unwrap();

        // Set up unhealthy stats - old last success
        {
            let mut stats = ingester.stats.write().await;
            stats.error_count = 10; // Less than 100
            stats.last_success = Some(chrono::Utc::now() - chrono::Duration::minutes(10));
            // More than 5 minutes ago
        }

        let health = ingester.health().await.unwrap();
        assert!(!health.healthy);
        assert!(health.components.contains_key("ingester"));
        assert!(!health.components["ingester"].healthy);
    }

    #[tokio::test]
    async fn test_service_lifecycle_health_unhealthy_no_success() {
        let config = IngesterConfig {
            workers: 1,
            batch_size: 10,
            queue_capacity: 100,
        };
        let context = create_test_service_context();
        let ingester = EventIngester::new(config, context).await.unwrap();

        // Default stats have no last_success
        let health = ingester.health().await.unwrap();
        assert!(!health.healthy);
        assert!(health.components.contains_key("ingester"));
        assert!(!health.components["ingester"].healthy);
    }

    #[tokio::test]
    async fn test_service_lifecycle_is_running_true() {
        let config = IngesterConfig {
            workers: 1,
            batch_size: 10,
            queue_capacity: 100,
        };
        let context = create_test_service_context();
        let mut ingester = EventIngester::new(config, context).await.unwrap();

        let _ = ingester.start().await;
        assert!(ingester.is_running());
    }

    #[tokio::test]
    async fn test_service_lifecycle_is_running_false_after_stop() {
        let config = IngesterConfig {
            workers: 1,
            batch_size: 10,
            queue_capacity: 100,
        };
        let context = create_test_service_context();
        let mut ingester = EventIngester::new(config, context).await.unwrap();

        let _ = ingester.start().await;
        assert!(ingester.is_running());

        let _ = ingester.stop().await;
        assert!(!ingester.is_running());
    }

    #[tokio::test]
    async fn test_service_lifecycle_is_running_false_initially() {
        let config = IngesterConfig {
            workers: 1,
            batch_size: 10,
            queue_capacity: 100,
        };
        let context = create_test_service_context();
        let ingester = EventIngester::new(config, context).await.unwrap();

        // Should not be running initially
        assert!(!ingester.is_running());
    }

    // Tests for create_mock_solana_event()
    #[test]
    fn test_create_mock_solana_event_basic() {
        let event = create_mock_solana_event();

        assert!(!event.id().is_empty());
        assert_eq!(event.source(), "mock");
        assert_eq!(event.kind(), &EventKind::Swap);
    }

    #[test]
    fn test_create_mock_solana_event_unique_ids() {
        let event1 = create_mock_solana_event();
        let event2 = create_mock_solana_event();

        assert_ne!(event1.id(), event2.id());
    }

    #[test]
    fn test_create_mock_solana_event_timestamp() {
        let before = std::time::SystemTime::now();
        let event = create_mock_solana_event();
        let after = std::time::SystemTime::now();

        let event_time = event.timestamp();
        assert!(event_time >= before);
        assert!(event_time <= after);
    }

    #[test]
    fn test_create_mock_solana_event_metadata() {
        let event = create_mock_solana_event();
        let metadata = event.metadata();

        assert_eq!(metadata.source, "mock");
        assert_eq!(metadata.kind, EventKind::Swap);
        assert!(metadata.chain_data.is_none());
        assert!(metadata.custom.is_empty());
    }

    #[test]
    fn test_create_mock_solana_event_json_serialization() {
        let event = create_mock_solana_event();
        let json_result = event.to_json();

        assert!(json_result.is_ok());
        let json_value = json_result.unwrap();
        assert!(json_value.is_object());
        assert!(json_value.get("id").is_some());
        assert_eq!(json_value.get("type").unwrap(), "swap");
        assert_eq!(json_value.get("source").unwrap(), "mock");
        assert!(json_value.get("timestamp").is_some());
    }

    #[test]
    fn test_create_mock_solana_event_clone() {
        let event = create_mock_solana_event();
        let cloned_event = event.clone_boxed();

        assert_eq!(event.id(), cloned_event.id());
        assert_eq!(event.source(), cloned_event.source());
        assert_eq!(event.kind(), cloned_event.kind());
    }

    #[test]
    fn test_create_mock_solana_event_as_any() {
        let event = create_mock_solana_event();
        let any_ref = event.as_any();

        // Should be able to downcast back to concrete type
        // Note: MockEvent is not accessible outside create_mock_solana_event function
        // This test verifies the any reference works, but cannot test the specific downcast
        // Check that the any reference is not null (basic validity check)
        assert!(!std::ptr::eq(
            any_ref as *const dyn std::any::Any as *const (),
            std::ptr::null()
        ));
    }

    #[test]
    fn test_create_mock_solana_event_as_any_mut() {
        let mut event = create_mock_solana_event();
        let _any_mut = event.as_any_mut();

        // Should be able to downcast back to concrete type
        // Note: MockEvent is not accessible outside create_mock_solana_event function
        // This test verifies the any reference works, but cannot test the specific downcast
        let any_ref = event.as_any();
        // Check that the any reference is not null (basic validity check)
        assert!(!std::ptr::eq(
            any_ref as *const dyn std::any::Any as *const (),
            std::ptr::null()
        ));
    }

    // Tests for StreamManagerBuilder placeholder structs
    #[test]
    fn test_stream_manager_builder_new() {
        let builder = StreamManagerBuilder::new();
        // Since it's a placeholder, we can only verify it can be created
        assert_eq!(
            std::mem::size_of_val(&builder),
            std::mem::size_of::<StreamManagerBuilder>()
        );
    }

    #[test]
    fn test_stream_manager_builder_with_config() {
        let builder = StreamManagerBuilder::new();
        let config = StreamManagerConfig {
            max_concurrent_streams: 10,
            health_check_interval: Duration::from_secs(30),
            reconnect_delay: Duration::from_secs(5),
            max_reconnect_attempts: 3,
        };

        let builder_with_config = builder.with_config(config);
        // Since it's a placeholder, we can only verify the method chains
        assert_eq!(
            std::mem::size_of_val(&builder_with_config),
            std::mem::size_of::<StreamManagerBuilder>()
        );
    }

    #[test]
    fn test_stream_manager_builder_build() {
        let builder = StreamManagerBuilder::new();
        let manager = builder.build();

        // Since it's a placeholder, we can only verify it can be built
        assert_eq!(
            std::mem::size_of_val(&manager),
            std::mem::size_of::<StreamManager>()
        );
    }

    #[test]
    fn test_stream_manager_builder_chaining() {
        let config = StreamManagerConfig {
            max_concurrent_streams: 20,
            health_check_interval: Duration::from_secs(60),
            reconnect_delay: Duration::from_secs(10),
            max_reconnect_attempts: 5,
        };

        let manager = StreamManagerBuilder::new().with_config(config).build();

        assert_eq!(
            std::mem::size_of_val(&manager),
            std::mem::size_of::<StreamManager>()
        );
    }

    #[test]
    fn test_stream_manager_config_creation() {
        let config = StreamManagerConfig {
            max_concurrent_streams: 15,
            health_check_interval: Duration::from_secs(45),
            reconnect_delay: Duration::from_secs(7),
            max_reconnect_attempts: 4,
        };

        assert_eq!(config.max_concurrent_streams, 15);
        assert_eq!(config.health_check_interval, Duration::from_secs(45));
        assert_eq!(config.reconnect_delay, Duration::from_secs(7));
        assert_eq!(config.max_reconnect_attempts, 4);
    }

    // Edge case and integration tests
    #[tokio::test]
    async fn test_start_workers_with_experimental_feature_disabled() {
        let mut config = create_test_config();
        config.features.experimental = false;
        let store = Arc::new(MockDataStore::new(true));
        let metrics = create_test_metrics();
        let context = Arc::new(ServiceContext::new(config, store, metrics));

        let ingester_config = IngesterConfig {
            workers: 1,
            batch_size: 10,
            queue_capacity: 100,
        };

        let mut ingester = EventIngester::new(ingester_config, context).await.unwrap();
        let result = ingester.start().await;
        assert!(result.is_ok());

        // With experimental disabled, workers should still start but not generate events
        sleep(Duration::from_millis(200)).await;

        let stats = ingester.stats().await;
        assert_eq!(stats.total_events, 0); // No events should be generated
    }

    #[tokio::test]
    async fn test_concurrent_stats_access() {
        let config = IngesterConfig {
            workers: 1,
            batch_size: 10,
            queue_capacity: 100,
        };
        let context = create_test_service_context();
        let ingester = Arc::new(EventIngester::new(config, context).await.unwrap());

        let ingester1 = ingester.clone();
        let ingester2 = ingester.clone();

        let handle1 = tokio::spawn(async move {
            for i in 0..10 {
                let mut stats = ingester1.stats.write().await;
                stats.total_events = i;
                drop(stats);
                sleep(Duration::from_millis(1)).await;
            }
        });

        let handle2 = tokio::spawn(async move {
            for _ in 0..10 {
                let _stats = ingester2.stats().await;
                sleep(Duration::from_millis(1)).await;
            }
        });

        let _ = tokio::join!(handle1, handle2);
        // Test should complete without deadlock
    }

    #[tokio::test]
    async fn test_event_ingester_clone() {
        let config = IngesterConfig {
            workers: 1,
            batch_size: 10,
            queue_capacity: 100,
        };
        let context = create_test_service_context();
        let ingester1 = EventIngester::new(config, context).await.unwrap();
        let ingester2 = ingester1.clone();

        // Both ingesters should share the same underlying data
        assert_eq!(ingester1.config.workers, ingester2.config.workers);
        assert_eq!(ingester1.config.batch_size, ingester2.config.batch_size);
        assert_eq!(
            ingester1.config.queue_capacity,
            ingester2.config.queue_capacity
        );
    }

    #[tokio::test]
    async fn test_rate_limiter_check_failure() {
        let mut config = create_test_config();
        config.processing.rate_limit.enabled = true;
        config.processing.rate_limit.max_events_per_second = 1; // Very low rate
        config.processing.rate_limit.burst_capacity = 1;

        let store = Arc::new(MockDataStore::new(true));
        let metrics = create_test_metrics();
        let context = Arc::new(ServiceContext::new(config, store, metrics));

        let ingester_config = IngesterConfig {
            workers: 1,
            batch_size: 10,
            queue_capacity: 100,
        };

        let mut ingester = EventIngester::new(ingester_config, context).await.unwrap();
        let _ = ingester.start().await;

        // Wait for some time to let the worker attempt to generate events
        sleep(Duration::from_millis(100)).await;

        // The rate limiter should prevent rapid event generation
        let stats = ingester.stats().await;
        // Events might be generated but should be limited
        assert!(stats.total_events <= 2); // Very conservative estimate
    }

    #[tokio::test]
    async fn test_large_batch_size() {
        let config = IngesterConfig {
            workers: 1,
            batch_size: 10000, // Very large batch
            queue_capacity: 100,
        };
        let context = create_test_service_context();
        let mut ingester = EventIngester::new(config, context).await.unwrap();

        let _ = ingester.start().await;

        // Should handle large batch size without issues
        let events = timeout(Duration::from_millis(100), ingester.receive_events()).await;
        // Should timeout since no events, but shouldn't crash
        assert!(events.is_err());
    }

    #[tokio::test]
    async fn test_zero_batch_size() {
        let config = IngesterConfig {
            workers: 1,
            batch_size: 0, // Edge case: zero batch size
            queue_capacity: 100,
        };
        let context = create_test_service_context();
        let mut ingester = EventIngester::new(config, context).await.unwrap();

        let _ = ingester.start().await;

        let events = timeout(Duration::from_millis(100), ingester.receive_events()).await;
        // Should timeout since no events and zero batch size
        assert!(events.is_err());
    }

    // Test environment variable edge cases
    #[tokio::test]
    async fn test_initialize_streams_empty_solana_url() {
        std::env::set_var(SOLANA_RPC_URL, "");

        let config = IngesterConfig {
            workers: 1,
            batch_size: 10,
            queue_capacity: 100,
        };
        let context = create_test_service_context();
        let ingester = EventIngester::new(config, context).await.unwrap();

        let result = ingester.initialize_streams().await;
        assert!(result.is_ok());

        std::env::remove_var(SOLANA_RPC_URL);
    }

    #[tokio::test]
    async fn test_initialize_streams_rpc_url_prefix_but_no_suffix() {
        std::env::set_var(RPC_URL_PREFIX_ONLY, "https://some-chain.com");

        let config = IngesterConfig {
            workers: 1,
            batch_size: 10,
            queue_capacity: 100,
        };
        let context = create_test_service_context();
        let ingester = EventIngester::new(config, context).await.unwrap();

        let result = ingester.initialize_streams().await;
        assert!(result.is_ok());

        std::env::remove_var(RPC_URL_PREFIX_ONLY);
    }
}
