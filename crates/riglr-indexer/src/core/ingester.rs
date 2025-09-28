//! Event ingester for high-throughput data collection

use core::any::Any;
use core::time::Duration;
use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use tokio::runtime::Handle;
use tokio::sync::{mpsc, RwLock};
use tokio::time::interval;
use tracing::{error, info};

use riglr_events_core::prelude::*;

use crate::config::FeatureState;
use crate::core::{ComponentHealth, HealthStatus, ServiceContext, ServiceLifecycle, ServiceState};
use crate::error::{IndexerError, IndexerResult};

const SOLANA_RPC_URL: &str = "SOLANA_RPC_URL";
#[cfg(test)]
const RPC_URL_1: &str = "RPC_URL_1";
#[cfg(test)]
const RPC_URL_137: &str = "RPC_URL_137";
#[cfg(test)]
const RPC_URL_INVALID: &str = "RPC_URL_INVALID";
#[cfg(test)]
const RPC_URL_PREFIX_ONLY: &str = "RPC_URL_;";
use crate::utils::{safe_cast_u64_to_f64, RateLimiter};
// Config imports only used in tests

/// Configuration for the event ingester
#[derive(Debug, Clone)]
pub struct Config {
    /// Batch size for processing
    pub batch_size: usize,
    /// Queue capacity
    pub queue_capacity: usize,
    /// Number of ingestion workers
    pub workers: usize,
}

/// Type alias for event sender
type EventSender = mpsc::UnboundedSender<Box<dyn Event>>;
/// Type alias for event receiver
type EventReceiver = mpsc::UnboundedReceiver<Box<dyn Event>>;

/// Event ingester that collects events from multiple sources
#[derive(Debug, Clone)]
pub struct Ingester {
    /// Ingester configuration
    config: Config,
    /// Shared service context
    context: Arc<ServiceContext>,
    /// Event queue receiver for external access
    event_rx: Arc<RwLock<Option<EventReceiver>>>,
    /// Event queue sender
    event_tx: Arc<RwLock<Option<EventSender>>>,
    /// Rate limiter for ingestion
    rate_limiter: Arc<RateLimiter>,
    /// Ingestion statistics
    stats: Arc<RwLock<IngestionStats>>,
    /// Stream manager for handling multiple data sources
    stream_manager: Arc<RwLock<Option<StreamManager>>>,
}

/// Ingestion statistics
#[derive(Debug, Clone, Default)]
pub struct IngestionStats {
    /// Active streams
    pub active_streams: usize,
    /// Error count
    pub error_count: u64,
    /// Events per second
    pub events_per_second: f64,
    /// Last successful ingestion
    pub last_success: Option<chrono::DateTime<chrono::Utc>>,
    /// Queue depth
    pub queue_depth: usize,
    /// Total events ingested
    pub total_events: u64,
}

impl Ingester {
    /// Create a new event ingester
    ///
    /// # Errors
    ///
    /// Returns error if worker thread pool initialization fails
    pub fn new(config: Config, context: Arc<ServiceContext>) -> IndexerResult<Self> {
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

    /// Add EVM stream configurations for any configured chains
    fn add_evm_streams_if_configured(builder: StreamManagerBuilder) -> StreamManagerBuilder {
        // Add EVM streams if configured
        for (chain_id_str, rpc_url) in env::vars() {
            if let Some(parsed_chain_id) = Self::parse_evm_chain_id(&chain_id_str) {
                info!(
                    "Adding EVM stream source for chain {}: {}",
                    parsed_chain_id, rpc_url
                );
                // Add EVM stream configuration - placeholder for future implementation
                // This will be extended with actual stream configuration when implemented
            }
        }

        builder
    }

    /// Add Solana stream configuration if environment variable is set
    fn add_solana_stream_if_configured(builder: StreamManagerBuilder) -> StreamManagerBuilder {
        if let Ok(rpc_url) = env::var(SOLANA_RPC_URL) {
            info!("Adding Solana stream source: {}", rpc_url);

            // Create a Solana event stream
            // This would integrate with riglr-streams to create actual streams
            // For now, we'll create a placeholder that can be extended
            builder.with_config(StreamManagerConfig {
                // Placeholder config - will be populated when stream manager is implemented
            })
        } else {
            builder
        }
    }

    /// Initialize stream sources
    async fn initialize_streams(&self) -> IndexerResult<()> {
        info!("Initializing event streams...");

        // Create stream manager
        let builder = StreamManagerBuilder::new();

        // Add Solana streams if configured
        let builder = Self::add_solana_stream_if_configured(builder);

        // Add EVM streams if configured
        let builder = Self::add_evm_streams_if_configured(builder);

        let manager = builder.build();
        *self.stream_manager.write().await = Some(manager);

        info!("Event streams initialized");
        Ok(())
    }

    /// Parse chain ID from environment variable name
    fn parse_evm_chain_id(chain_id_str: &str) -> Option<u64> {
        if chain_id_str.starts_with("RPC_URL_") {
            chain_id_str
                .strip_prefix("RPC_URL_")
                .and_then(|chain_id_str| chain_id_str.parse::<u64>().ok())
        } else {
            None
        }
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
                            if context.config.features.dev_features.experimental == FeatureState::Enabled {
                                let rate_limit_result = rate_limiter.check_rate_limit(&format!("worker_{worker_id}"));
                                if rate_limit_result == Ok(()) {
                                    let mock_event = create_mock_solana_event();

                                    if let Err(e) = tx.send(mock_event) {
                                        error!("Worker {} failed to send event: {}", worker_id, e);
                                        break;
                                    }

                                    // Update stats
                                    {
                                        let mut stats = stats.write().await;
                                        stats.total_events = stats.total_events.saturating_add(1);
                                        stats.last_success = Some(chrono::Utc::now());
                                    }
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

    /// Get current queue depth
    pub async fn queue_depth(&self) -> usize {
        let stats = self.stats.read().await;
        stats.queue_depth
    }

    /// Receive events from the ingestion queue
    ///
    /// # Errors
    ///
    /// Returns error if event deserialization fails or queue access fails
    pub async fn receive_events(&self) -> IndexerResult<Vec<Box<dyn Event>>> {
        let mut events = Vec::new();
        let batch_size = self.config.batch_size;

        // Collect events up to batch size
        while events.len() < batch_size {
            let mut lock_guard = self.event_rx.write().await;
            if lock_guard.is_none() {
                return Err(IndexerError::internal("Event receiver not initialized"));
            }
            let rx = lock_guard
                .as_mut()
                .ok_or_else(|| IndexerError::internal("Event receiver not initialized"))?;

            let recv_result = rx.try_recv();
            match recv_result {
                Ok(event) => events.push(event),
                Err(mpsc::error::TryRecvError::Empty) => {
                    if events.is_empty() {
                        // Wait for at least one event
                        let recv_result = rx.recv().await;
                        match recv_result {
                            Some(event) => events.push(event),
                            None => break, // Channel closed
                        }
                    } else {
                        break; // Return partial batch
                    }
                }
                Err(mpsc::error::TryRecvError::Disconnected) => break,
            }

            // Update queue depth statistics
            let queue_len = rx.len();
            drop(lock_guard);

            // Update stats without holding the receiver lock
            {
                let mut stats = self.stats.write().await;
                stats.queue_depth = queue_len;
            }
        }

        Ok(events)
    }

    /// Start statistics collection
    fn start_stats_collection(&self) {
        let stats = self.stats.clone();
        let context = self.context.clone();

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(10));
            let mut last_count = 0u64;
            let mut last_time = Instant::now();

            loop {
                interval.tick().await;

                {
                    let mut stats_guard = stats.write().await;
                    let now = Instant::now();
                    let elapsed = now.duration_since(last_time).as_secs_f64();

                    if elapsed > 0.0 {
                        let events_delta = stats_guard.total_events.saturating_sub(last_count);
                        stats_guard.events_per_second =
                            safe_cast_u64_to_f64(events_delta) / elapsed;

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
                    drop(stats_guard);
                    last_time = now;
                }

                // Check for shutdown
                let state_result = context.state().await;
                if matches!(state_result, ServiceState::Stopping | ServiceState::Stopped) {
                    break;
                }
            }
        });
    }

    /// Get ingestion statistics
    pub async fn stats(&self) -> IngestionStats {
        let stats = self.stats.read().await;
        stats.clone()
    }
}

#[async_trait::async_trait]
impl ServiceLifecycle for Ingester {
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

    async fn health(&self) -> IndexerResult<HealthStatus> {
        let stats = self.stats.read().await;

        let healthy = stats.error_count < 100 && // Less than 100 errors
                     stats.last_success.is_some_and(|t| {
                         chrono::Utc::now().signed_duration_since(t).num_minutes() < 5
                     });

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
        drop(stats);

        let mut components = HashMap::new();

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

    fn is_running(&self) -> bool {
        // Check if event sender is still active
        Handle::try_current()
            .ok()
            .and_then(|_| {
                let tx_guard = self.event_tx.try_read().ok()?;
                Some(tx_guard.is_some())
            })
            .unwrap_or(false)
    }
}

// Placeholder types that would be implemented properly with riglr-streams integration
#[derive(Debug)]
struct StreamManager;
#[derive(Debug)]
struct StreamManagerBuilder;
struct StreamManagerConfig {
    // Placeholder config - fields will be added when stream manager is implemented
}

impl StreamManagerBuilder {
    #[expect(clippy::unused_self)]
    const fn build(self) -> StreamManager {
        StreamManager
    }

    const fn new() -> Self {
        Self
    }

    const fn with_config(self, _config: StreamManagerConfig) -> Self {
        self
    }
}

/// Create a mock Solana event for testing
fn create_mock_solana_event() -> Box<dyn Event> {
    // For now, create a minimal mock event that implements the Event trait
    // In a real implementation, this would use proper Solana event creation
    #[derive(Debug, Clone)]
    struct MockEvent {
        id: String,
        kind: EventKind,
        metadata: EventMetadata,
        timestamp: SystemTime,
    }

    impl Event for MockEvent {
        fn as_any(&self) -> &dyn Any {
            self
        }

        fn as_any_mut(&mut self) -> &mut dyn Any {
            self
        }

        fn clone_boxed(&self) -> Box<dyn Event> {
            Box::new(self.clone())
        }

        fn id(&self) -> &str {
            &self.id
        }

        fn kind(&self) -> &EventKind {
            &self.kind
        }

        fn metadata(&self) -> &EventMetadata {
            &self.metadata
        }

        fn metadata_mut(&mut self) -> EventResult<&mut EventMetadata> {
            Ok(&mut self.metadata)
        }

        fn source(&self) -> &'static str {
            "mock"
        }

        fn timestamp(&self) -> SystemTime {
            self.timestamp
        }

        fn to_json(&self) -> Result<serde_json::Value, riglr_events_core::EventError> {
            Ok(serde_json::json!({
                "id": self.id,
                "type": "swap",
                "source": "mock",
                "timestamp": self.timestamp.duration_since(UNIX_EPOCH)
                    .unwrap_or_else(|_| Duration::from_secs(0))
                    .as_millis()
            }))
        }
    }

    let metadata = EventMetadata::new("mock".to_string(), EventKind::Swap, "mock".to_string());

    Box::new(MockEvent {
        id: format!("mock-{}", uuid::Uuid::new_v4()),
        kind: EventKind::Swap,
        metadata,
        timestamp: SystemTime::now(),
    })
}

#[cfg(test)]
#[expect(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use crate::config::{
        ApiConfig, ApiFeatureSet, ArchiveConfig, AuthConfig, AuthMethod, BatchConfig, CacheBackend,
        CacheConfig, CacheTtlConfig, CompressionAlgorithm, CompressionConfig, ConnectionPoolConfig,
        CoreFeatureSet, CorsConfig, DevFeatureSet, FeatureConfig, FeatureState, HttpConfig,
        IndexerConfig, LogFormat, LogOutput, LoggingConfig, MemoryCacheConfig, MetricsConfig,
        ProcessingConfig, QueueConfig, QueueType, RateLimitConfig, RetentionConfig, RetryConfig,
        ServiceConfig, StorageBackend, StorageBackendConfig, StorageConfig,
        StructuredLoggingConfig, WebSocketConfig,
    };
    use crate::core::IngesterConfig;
    use crate::error::StorageError;
    use crate::metrics::MetricsCollector;
    use crate::prelude::EventIngester;
    use crate::storage::{DataStore, EventFilter, EventQuery, StorageStats, StoredEvent};
    use async_trait::async_trait;
    use core::ptr::{eq, from_ref, null};
    use core::sync::atomic::{AtomicBool, Ordering};
    use std::collections::HashMap;
    use std::sync::Arc;
    use tokio::time::{sleep, timeout, Duration, Instant};

    mod test_env_vars {
        use std::env;

        /// Helper function to set environment variables in tests without using string literals
        #[expect(
            unsafe_code,
            reason = "env::set_var is deprecated safe but will be unsafe in Rust 2024"
        )]
        pub fn set_test_env_var(key: &'static str, value: &str) {
            // TODO: Audit that the environment access only happens in single-threaded code.
            unsafe { env::set_var(key, value) };
        }

        /// Helper function to remove environment variables in tests without using string literals
        #[expect(
            unsafe_code,
            reason = "env::remove_var is deprecated safe but will be unsafe in Rust 2024"
        )]
        pub fn remove_test_env_var(key: &'static str) {
            // TODO: Audit that the environment access only happens in single-threaded code.
            unsafe { env::remove_var(key) };
        }
    }

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
            Ok(StorageStats {
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
                Err(IndexerError::Storage(StorageError::ConnectionFailed {
                    message: "Mock connection failed".to_string(),
                }))
            }
        }

        fn as_any(&self) -> &dyn Any {
            self
        }
    }

    fn create_test_service_config() -> ServiceConfig {
        ServiceConfig {
            name: "test-indexer".to_string(),
            version: "0.1.0".to_string(),
            environment: "test".to_string(),
            node_id: Some("test-node".to_string()),
            shutdown_timeout: Duration::from_secs(5),
            health_check_interval: Duration::from_secs(10),
        }
    }

    fn create_test_storage_config() -> StorageConfig {
        StorageConfig {
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
        }
    }

    fn create_test_processing_config() -> ProcessingConfig {
        ProcessingConfig {
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
        }
    }

    fn create_test_api_config() -> ApiConfig {
        ApiConfig {
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
        }
    }

    fn create_test_config() -> IndexerConfig {
        IndexerConfig {
            service: create_test_service_config(),
            storage: create_test_storage_config(),
            processing: create_test_processing_config(),
            api: create_test_api_config(),
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
                core_features: CoreFeatureSet {
                    streaming: FeatureState::Enabled,
                    archival: FeatureState::Enabled,
                },
                api_features: ApiFeatureSet {
                    graphql: FeatureState::Disabled,
                },
                dev_features: DevFeatureSet {
                    experimental: FeatureState::Enabled,
                },
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
        Arc::new(
            MetricsCollector::new(config).unwrap(), // Test assertion - safe to unwrap
        )
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

        let debug_str = format!("{config:?}");
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
        assert!(stats.events_per_second.abs() < f64::EPSILON);
        assert_eq!(stats.queue_depth, 0);
        assert_eq!(stats.active_streams, 0);
        assert_eq!(stats.error_count, 0);
        assert!(stats.last_success.is_none());
    }

    #[test]
    fn test_ingestion_stats_debug() {
        let stats = IngestionStats {
            total_events: 100,
            events_per_second: 25.5,
            queue_depth: 10,
            active_streams: 3,
            error_count: 2,
            last_success: Some(chrono::Utc::now()),
        };

        let debug_str = format!("{stats:?}");
        assert!(debug_str.contains("IngestionStats"));
        assert!(debug_str.contains("total_events: 100"));
        assert!(debug_str.contains("events_per_second: 25.5"));
        assert!(debug_str.contains("queue_depth: 10"));
        assert!(debug_str.contains("active_streams: 3"));
        assert!(debug_str.contains("error_count: 2"));
    }

    #[test]
    fn test_ingestion_stats_clone() {
        let original = IngestionStats {
            total_events: 50,
            events_per_second: 12.3,
            queue_depth: 5,
            active_streams: 2,
            error_count: 1,
            last_success: Some(chrono::Utc::now()),
        };

        let cloned = original.clone();
        assert_eq!(original.total_events, cloned.total_events);
        assert!((original.events_per_second - cloned.events_per_second).abs() < f64::EPSILON);
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

        let result = EventIngester::new(config, context);
        assert!(result.is_ok());

        let ingester = result.unwrap(); // Test assertion - safe to unwrap
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

        let result = EventIngester::new(config, context);
        assert!(result.is_ok());
        let ingester = result.unwrap(); // Test assertion - safe to unwrap
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

        let result = EventIngester::new(ingester_config, context);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_event_ingester_new_with_large_values() {
        let config = IngesterConfig {
            workers: 1000,
            batch_size: 50000,
            queue_capacity: 1_000_000,
        };
        let context = create_test_service_context();

        let result = EventIngester::new(config, context);
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

        let ingester = EventIngester::new(config, context).unwrap(); // Test assertion - safe to unwrap

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
        let ingester = EventIngester::new(config, context).unwrap(); // Test assertion - safe to unwrap

        let result = ingester.initialize_streams().await;
        assert!(result.is_ok());

        // Verify stream manager is set
        assert!(ingester.stream_manager.read().await.is_some());
    }

    #[tokio::test]
    async fn test_initialize_streams_with_solana_env() {
        // Set environment variable for Solana
        test_env_vars::set_test_env_var(SOLANA_RPC_URL, "https://api.mainnet-beta.solana.com");

        let config = IngesterConfig {
            workers: 1,
            batch_size: 10,
            queue_capacity: 100,
        };
        let context = create_test_service_context();
        let ingester = EventIngester::new(config, context).unwrap(); // Test assertion - safe to unwrap

        let result = ingester.initialize_streams().await;
        assert!(result.is_ok());

        // Clean up
        test_env_vars::remove_test_env_var(SOLANA_RPC_URL);
    }

    #[tokio::test]
    async fn test_initialize_streams_with_evm_env() {
        // Set environment variable for EVM chain
        test_env_vars::set_test_env_var(RPC_URL_1, "https://eth-mainnet.alchemyapi.io/v2/test");
        test_env_vars::set_test_env_var(RPC_URL_137, "https://polygon-rpc.com");

        let config = IngesterConfig {
            workers: 1,
            batch_size: 10,
            queue_capacity: 100,
        };
        let context = create_test_service_context();
        let ingester = EventIngester::new(config, context).unwrap(); // Test assertion - safe to unwrap

        let result = ingester.initialize_streams().await;
        assert!(result.is_ok());

        // Clean up
        test_env_vars::remove_test_env_var(RPC_URL_1);
        test_env_vars::remove_test_env_var(RPC_URL_137);
    }

    #[tokio::test]
    async fn test_initialize_streams_with_invalid_chain_id() {
        // Set environment variable with invalid chain ID
        test_env_vars::set_test_env_var(RPC_URL_INVALID, "https://invalid-chain.com");

        let config = IngesterConfig {
            workers: 1,
            batch_size: 10,
            queue_capacity: 100,
        };
        let context = create_test_service_context();
        let ingester = EventIngester::new(config, context).unwrap(); // Test assertion - safe to unwrap

        let result = ingester.initialize_streams().await;
        assert!(result.is_ok()); // Should not fail, just skip invalid entries

        // Clean up
        test_env_vars::remove_test_env_var(RPC_URL_INVALID);
    }

    #[tokio::test]
    async fn test_initialize_streams_multiple_times() {
        let config = IngesterConfig {
            workers: 1,
            batch_size: 10,
            queue_capacity: 100,
        };
        let context = create_test_service_context();
        let ingester = EventIngester::new(config, context).unwrap(); // Test assertion - safe to unwrap

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
        let mut ingester = EventIngester::new(config, context).unwrap(); // Test assertion - safe to unwrap

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
        let mut ingester = EventIngester::new(config, context).unwrap(); // Test assertion - safe to unwrap

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

        let events = result
            .unwrap() // Test assertion - safe to unwrap
            .unwrap(); // Test assertion - safe to unwrap
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
        let mut ingester = EventIngester::new(config, context).unwrap(); // Test assertion - safe to unwrap

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

        let events = result
            .unwrap() // Test assertion - safe to unwrap
            .unwrap(); // Test assertion - safe to unwrap
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
        let ingester = EventIngester::new(config, context).unwrap(); // Test assertion - safe to unwrap

        // Clear the receiver to simulate uninitialized state
        {
            let mut rx_guard = ingester.event_rx.write().await;
            *rx_guard = None;
        }

        let result = ingester.receive_events().await;
        assert!(result.is_err());
        let error = result.expect_err("Expected error when receiver not initialized");
        assert!(matches!(error, IndexerError::Internal { .. }));
    }

    #[tokio::test]
    async fn test_receive_events_channel_closed() {
        let config = IngesterConfig {
            workers: 1,
            batch_size: 10,
            queue_capacity: 100,
        };
        let context = create_test_service_context();
        let mut ingester = EventIngester::new(config, context).unwrap(); // Test assertion - safe to unwrap

        // Start and immediately stop to close channels
        let _ = ingester.start().await;
        let _ = ingester.stop().await;

        let result = ingester.receive_events().await;
        assert!(result.is_ok());
        let events = result.unwrap(); // Test assertion - safe to unwrap
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
        let ingester = EventIngester::new(config, context).unwrap(); // Test assertion - safe to unwrap

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
        let ingester = EventIngester::new(config, context).unwrap(); // Test assertion - safe to unwrap

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
        let ingester = EventIngester::new(config, context).unwrap(); // Test assertion - safe to unwrap

        let stats = ingester.stats().await;
        assert_eq!(stats.total_events, 0);
        assert!(stats.events_per_second.abs() < f64::EPSILON);
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
        let ingester = EventIngester::new(config, context).unwrap(); // Test assertion - safe to unwrap

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
        assert!((stats.events_per_second - 25.5).abs() < f64::EPSILON);
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
        let mut ingester = EventIngester::new(config, context).unwrap(); // Test assertion - safe to unwrap

        let result = ingester.start().await;
        assert!(result.is_ok());

        // Verify health was updated
        let health = ingester.health().await.unwrap(); // Test assertion - safe to unwrap
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
        let mut ingester = EventIngester::new(config, context).unwrap(); // Test assertion - safe to unwrap

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
        let mut ingester = EventIngester::new(config, context).unwrap(); // Test assertion - safe to unwrap

        // Start first, then stop
        let _ = ingester.start().await;
        let result = ingester.stop().await;
        assert!(result.is_ok());

        // Verify sender is closed
        assert!(ingester.event_tx.read().await.is_none());
    }

    #[tokio::test]
    async fn test_service_lifecycle_stop_without_start() {
        let config = IngesterConfig {
            workers: 1,
            batch_size: 10,
            queue_capacity: 100,
        };
        let context = create_test_service_context();
        let mut ingester = EventIngester::new(config, context).unwrap(); // Test assertion - safe to unwrap

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
        let mut ingester = EventIngester::new(config, context).unwrap(); // Test assertion - safe to unwrap

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
        let ingester = EventIngester::new(config, context).unwrap(); // Test assertion - safe to unwrap

        // Set up healthy stats
        {
            let mut stats = ingester.stats.write().await;
            stats.error_count = 50; // Less than 100
            stats.last_success = Some(chrono::Utc::now()); // Recent success
        }

        let health = ingester.health().await.unwrap(); // Test assertion - safe to unwrap
        assert!(health.healthy);
        assert!(health.components.contains_key("ingester"));
        assert!(
            health
                .components
                .get("ingester")
                .unwrap() // Test assertion - safe to unwrap
                .healthy
        );
    }

    #[tokio::test]
    async fn test_service_lifecycle_health_unhealthy_too_many_errors() {
        let config = IngesterConfig {
            workers: 1,
            batch_size: 10,
            queue_capacity: 100,
        };
        let context = create_test_service_context();
        let ingester = EventIngester::new(config, context).unwrap(); // Test assertion - safe to unwrap

        // Set up unhealthy stats - too many errors
        {
            let mut stats = ingester.stats.write().await;
            stats.error_count = 150; // More than 100
            stats.last_success = Some(chrono::Utc::now());
        }

        let health = ingester.health().await.unwrap(); // Test assertion - safe to unwrap
        assert!(!health.healthy);
        assert!(health.components.contains_key("ingester"));
        assert!(
            !health
                .components
                .get("ingester")
                .unwrap() // Test assertion - safe to unwrap
                .healthy
        );
    }

    #[tokio::test]
    async fn test_service_lifecycle_health_unhealthy_old_success() {
        let config = IngesterConfig {
            workers: 1,
            batch_size: 10,
            queue_capacity: 100,
        };
        let context = create_test_service_context();
        let ingester = EventIngester::new(config, context).unwrap(); // Test assertion - safe to unwrap

        // Set up unhealthy stats - old last success
        {
            let mut stats = ingester.stats.write().await;
            stats.error_count = 10; // Less than 100
            stats.last_success = Some(chrono::Utc::now() - chrono::Duration::minutes(10));
            // More than 5 minutes ago
        }

        let health = ingester.health().await.unwrap(); // Test assertion - safe to unwrap
        assert!(!health.healthy);
        assert!(health.components.contains_key("ingester"));
        assert!(
            !health
                .components
                .get("ingester")
                .unwrap() // Test assertion - safe to unwrap
                .healthy
        );
    }

    #[tokio::test]
    async fn test_service_lifecycle_health_unhealthy_no_success() {
        let config = IngesterConfig {
            workers: 1,
            batch_size: 10,
            queue_capacity: 100,
        };
        let context = create_test_service_context();
        let ingester = EventIngester::new(config, context).unwrap(); // Test assertion - safe to unwrap

        // Default stats have no last_success
        let health = ingester.health().await.unwrap(); // Test assertion - safe to unwrap
        assert!(!health.healthy);
        assert!(health.components.contains_key("ingester"));
        assert!(
            !health
                .components
                .get("ingester")
                .unwrap() // Test assertion - safe to unwrap
                .healthy
        );
    }

    #[tokio::test]
    async fn test_service_lifecycle_is_running_true() {
        let config = IngesterConfig {
            workers: 1,
            batch_size: 10,
            queue_capacity: 100,
        };
        let context = create_test_service_context();
        let mut ingester = EventIngester::new(config, context).unwrap(); // Test assertion - safe to unwrap

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
        let mut ingester = EventIngester::new(config, context).unwrap(); // Test assertion - safe to unwrap

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
        let ingester = EventIngester::new(config, context).unwrap(); // Test assertion - safe to unwrap

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
        let before = SystemTime::now();
        let event = create_mock_solana_event();
        let after = SystemTime::now();

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
        let json_value = json_result.unwrap(); // Test assertion - safe to unwrap
        assert!(json_value.is_object());
        assert!(json_value.get("id").is_some());
        assert_eq!(json_value.get("type").unwrap(), "swap"); // Test assertion - safe to unwrap
        assert_eq!(
            json_value.get("source").unwrap(), // Test assertion - safe to unwrap
            "mock"
        );
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
        assert!(!eq(from_ref::<dyn Any>(any_ref).cast::<()>(), null()));
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
        assert!(!eq(from_ref::<dyn Any>(any_ref).cast::<()>(), null()));
    }

    // Tests for StreamManagerBuilder placeholder structs
    #[test]
    fn test_stream_manager_builder_new() {
        let builder = StreamManagerBuilder::new();
        // Since it's a placeholder, we can only verify it can be created
        assert_eq!(size_of_val(&builder), size_of::<StreamManagerBuilder>());
    }

    #[test]
    fn test_stream_manager_builder_with_config() {
        let builder = StreamManagerBuilder::new();
        let config = StreamManagerConfig {
            // Placeholder config - will be populated when stream manager is implemented
        };

        let builder_with_config = builder.with_config(config);
        // Since it's a placeholder, we can only verify the method chains
        assert_eq!(
            size_of_val(&builder_with_config),
            size_of::<StreamManagerBuilder>()
        );
    }

    #[test]
    fn test_stream_manager_builder_build() {
        let builder = StreamManagerBuilder::new();
        let manager = builder.build();

        // Since it's a placeholder, we can only verify it can be built
        assert_eq!(size_of_val(&manager), size_of::<StreamManager>());
    }

    #[test]
    fn test_stream_manager_builder_chaining() {
        let config = StreamManagerConfig {
            // Placeholder config - will be populated when stream manager is implemented
        };

        let builder = StreamManagerBuilder::new().with_config(config);
        let manager = builder.build();

        assert_eq!(size_of_val(&manager), size_of::<StreamManager>());
    }

    #[test]
    fn test_stream_manager_config_creation() {
        let config = StreamManagerConfig {
            // Placeholder config - will be populated when stream manager is implemented
        };

        // Since this is a placeholder, we can only verify it can be created
        assert_eq!(size_of_val(&config), size_of::<StreamManagerConfig>());
    }

    // Edge case and integration tests
    #[tokio::test]
    async fn test_start_workers_with_experimental_feature_disabled() {
        let mut config = create_test_config();
        config.features.dev_features.experimental = FeatureState::Disabled;
        let store = Arc::new(MockDataStore::new(true));
        let metrics = create_test_metrics();
        let context = Arc::new(ServiceContext::new(config, store, metrics));

        let ingester_config = IngesterConfig {
            workers: 1,
            batch_size: 10,
            queue_capacity: 100,
        };

        let mut ingester = EventIngester::new(ingester_config, context).unwrap(); // Test assertion - safe to unwrap
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
        let ingester = Arc::new(
            EventIngester::new(config, context).unwrap(), // Test assertion - safe to unwrap
        );

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
        let ingester1 = EventIngester::new(config, context).unwrap(); // Test assertion - safe to unwrap
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

        let mut ingester = EventIngester::new(ingester_config, context).unwrap(); // Test assertion - safe to unwrap
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
        let mut ingester = EventIngester::new(config, context).unwrap(); // Test assertion - safe to unwrap

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
        let mut ingester = EventIngester::new(config, context).unwrap(); // Test assertion - safe to unwrap

        let _ = ingester.start().await;

        let events = timeout(Duration::from_millis(100), ingester.receive_events()).await;
        // Should timeout since no events and zero batch size
        assert!(events.is_err());
    }

    // Test environment variable edge cases
    #[tokio::test]
    async fn test_initialize_streams_empty_solana_url() {
        test_env_vars::set_test_env_var(SOLANA_RPC_URL, "");

        let config = IngesterConfig {
            workers: 1,
            batch_size: 10,
            queue_capacity: 100,
        };
        let context = create_test_service_context();
        let ingester = EventIngester::new(config, context).unwrap(); // Test assertion - safe to unwrap

        let result = ingester.initialize_streams().await;
        assert!(result.is_ok());

        test_env_vars::remove_test_env_var(SOLANA_RPC_URL);
    }

    #[tokio::test]
    async fn test_initialize_streams_rpc_url_prefix_but_no_suffix() {
        test_env_vars::set_test_env_var(RPC_URL_PREFIX_ONLY, "https://some-chain.com");

        let config = IngesterConfig {
            workers: 1,
            batch_size: 10,
            queue_capacity: 100,
        };
        let context = create_test_service_context();
        let ingester = EventIngester::new(config, context).unwrap(); // Test assertion - safe to unwrap

        let result = ingester.initialize_streams().await;
        assert!(result.is_ok());

        test_env_vars::remove_test_env_var(RPC_URL_PREFIX_ONLY);
    }
}
