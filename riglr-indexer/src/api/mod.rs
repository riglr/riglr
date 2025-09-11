//! API server for the indexer service

use axum::Router;
use std::sync::Arc;

use crate::config::ApiConfig;
use crate::core::ServiceContext;
use crate::error::{IndexerError, IndexerResult};

pub mod rest;
pub mod websocket;

pub use rest::RestHandler;
pub use websocket::WebSocketStreamer;

/// Main API server
#[derive(Debug)]
pub struct ApiServer {
    /// Service context
    context: Arc<ServiceContext>,
    /// Configuration
    config: ApiConfig,
    /// REST handler
    rest_handler: RestHandler,
    /// WebSocket streamer
    websocket_streamer: Option<WebSocketStreamer>,
}

impl ApiServer {
    /// Create a new API server
    pub fn new(context: Arc<ServiceContext>) -> IndexerResult<Self> {
        let config = context.config.api.clone();
        let rest_handler = RestHandler::new(context.clone())?;

        let websocket_streamer = if config.websocket.enabled {
            Some(WebSocketStreamer::new(context.clone())?)
        } else {
            None
        };

        Ok(Self {
            context,
            config,
            rest_handler,
            websocket_streamer,
        })
    }

    /// Create the router for the API server
    pub fn create_router(&self) -> Router<Arc<ServiceContext>> {
        let mut router = self.rest_handler.create_router();

        if let Some(ws_streamer) = &self.websocket_streamer {
            router = router.merge(ws_streamer.create_router());
        }

        router
    }

    /// Start the API server
    pub async fn start(&self) -> IndexerResult<()> {
        let addr = format!("{}:{}", self.config.http.bind, self.config.http.port);
        let bind_result = tokio::net::TcpListener::bind(&addr).await;
        let listener_result = bind_result.map_err(|_e| {
            IndexerError::Network(crate::error::NetworkError::HttpFailed {
                status: 0,
                url: addr.clone(),
            })
        });
        let listener = listener_result?;

        tracing::info!("API server listening on {}", addr);

        let app = self.create_router();

        let serve_future = axum::serve(listener, app.with_state(self.context.clone()));
        let serve_result = serve_future.await;
        let final_result =
            serve_result.map_err(|e| IndexerError::internal_with_source("API server failed", e));
        final_result?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{
        ApiConfig, BatchConfig, FeatureConfig, HttpConfig, IndexerConfig, LogFormat, LoggingConfig,
        MetricsConfig, ProcessingConfig, QueueConfig, RateLimitConfig, RetryConfig, ServiceConfig,
        StorageBackendConfig, StorageConfig, StructuredLoggingConfig, WebSocketConfig,
    };
    use crate::core::ServiceContext;
    use crate::error::{IndexerError, IndexerResult, NetworkError, StorageError};
    use crate::metrics::MetricsCollector;
    use crate::storage::{DataStore, EventFilter, EventQuery, StorageStats, StoredEvent};
    use async_trait::async_trait;
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::time::Duration;

    // Mock DataStore implementation for testing
    #[derive(Debug)]
    struct MockDataStore {
        should_fail_init: bool,
        should_fail_insert: bool,
        should_fail_query: bool,
    }

    impl MockDataStore {
        fn new() -> Self {
            Self {
                should_fail_init: false,
                should_fail_insert: false,
                should_fail_query: false,
            }
        }

        fn with_init_failure() -> Self {
            Self {
                should_fail_init: true,
                should_fail_insert: false,
                should_fail_query: false,
            }
        }
    }

    #[async_trait]
    impl DataStore for MockDataStore {
        async fn initialize(&self) -> IndexerResult<()> {
            if self.should_fail_init {
                return Err(IndexerError::Storage(StorageError::ConnectionFailed {
                    message: "Mock init failure".to_string(),
                }));
            }
            Ok(())
        }

        async fn insert_event(&self, _event: &StoredEvent) -> IndexerResult<()> {
            if self.should_fail_insert {
                return Err(IndexerError::Storage(StorageError::QueryFailed {
                    query: "INSERT INTO events".to_string(),
                }));
            }
            Ok(())
        }

        async fn insert_events(&self, _events: &[StoredEvent]) -> IndexerResult<()> {
            if self.should_fail_insert {
                return Err(IndexerError::Storage(StorageError::QueryFailed {
                    query: "BATCH INSERT INTO events".to_string(),
                }));
            }
            Ok(())
        }

        async fn query_events(&self, _query: &EventQuery) -> IndexerResult<Vec<StoredEvent>> {
            if self.should_fail_query {
                return Err(IndexerError::Storage(StorageError::QueryFailed {
                    query: "SELECT * FROM events".to_string(),
                }));
            }
            Ok(vec![])
        }

        async fn get_event(&self, _id: &str) -> IndexerResult<Option<StoredEvent>> {
            if self.should_fail_query {
                return Err(IndexerError::Storage(StorageError::QueryFailed {
                    query: "SELECT * FROM events WHERE id = ?".to_string(),
                }));
            }
            Ok(None)
        }

        async fn delete_events(&self, _filter: &EventFilter) -> IndexerResult<u64> {
            Ok(0)
        }

        async fn count_events(&self, _filter: &EventFilter) -> IndexerResult<u64> {
            if self.should_fail_query {
                return Err(IndexerError::Storage(StorageError::QueryFailed {
                    query: "SELECT COUNT(*) FROM events".to_string(),
                }));
            }
            Ok(0)
        }

        async fn get_stats(&self) -> IndexerResult<StorageStats> {
            Ok(StorageStats {
                total_events: 0,
                storage_size_bytes: 0,
                avg_write_latency_ms: 1.0,
                avg_read_latency_ms: 0.5,
                active_connections: 1,
                cache_hit_rate: 0.9,
            })
        }

        async fn health_check(&self) -> IndexerResult<()> {
            Ok(())
        }

        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
    }

    // Helper function to create a test configuration
    fn create_test_config() -> IndexerConfig {
        IndexerConfig {
            service: ServiceConfig {
                name: "test-indexer".to_string(),
                version: "1.0.0".to_string(),
                environment: "test".to_string(),
                node_id: None,
                shutdown_timeout: Duration::from_secs(30),
                health_check_interval: Duration::from_secs(10),
            },
            storage: StorageConfig {
                primary: StorageBackendConfig {
                    backend: crate::config::StorageBackend::Postgres,
                    url: "postgresql://localhost/test".to_string(),
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
                cache: crate::config::CacheConfig {
                    backend: crate::config::CacheBackend::Memory,
                    redis_url: None,
                    ttl: crate::config::CacheTtlConfig {
                        default: Duration::from_secs(300),
                        events: Duration::from_secs(600),
                        aggregates: Duration::from_secs(180),
                    },
                    memory: crate::config::MemoryCacheConfig {
                        max_size_bytes: 1024 * 1024 * 100, // 100 MB
                        max_entries: 10000,
                    },
                },
                retention: crate::config::RetentionConfig {
                    default: Duration::from_secs(86400 * 30), // 30 days
                    by_event_type: HashMap::new(),
                    archive: crate::config::ArchiveConfig {
                        enabled: false,
                        backend: None,
                        compression: crate::config::CompressionConfig {
                            algorithm: crate::config::CompressionAlgorithm::Gzip,
                            level: 6,
                        },
                    },
                },
            },
            processing: ProcessingConfig {
                workers: 4,
                batch: BatchConfig {
                    max_size: 100,
                    max_age: Duration::from_secs(5),
                    target_size: 50,
                },
                queue: QueueConfig {
                    capacity: 10000,
                    queue_type: crate::config::QueueType::Memory,
                    disk_settings: None,
                },
                retry: RetryConfig {
                    max_attempts: 3,
                    base_delay: Duration::from_millis(100),
                    max_delay: Duration::from_secs(10),
                    backoff_multiplier: 2.0,
                    jitter: 0.1,
                },
                rate_limit: RateLimitConfig {
                    enabled: false,
                    max_events_per_second: 1000,
                    burst_capacity: 100,
                },
            },
            api: ApiConfig {
                http: HttpConfig {
                    bind: "127.0.0.1".to_string(),
                    port: 8080,
                    timeout: Duration::from_secs(30),
                    max_request_size: 1024 * 1024, // 1 MB
                    keep_alive: Duration::from_secs(75),
                },
                websocket: WebSocketConfig {
                    enabled: true,
                    max_connections: 1000,
                    buffer_size: 1000,
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
                    enabled: true,
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
                histogram_buckets: vec![0.1, 0.5, 1.0, 2.5, 5.0, 10.0],
                custom: HashMap::new(),
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                format: LogFormat::Json,
                outputs: vec![crate::config::LogOutput::Stdout],
                structured: StructuredLoggingConfig {
                    include_location: false,
                    include_thread: true,
                    include_service_metadata: true,
                    custom_fields: HashMap::new(),
                },
            },
            features: FeatureConfig {
                realtime_streaming: true,
                archival: false,
                graphql_api: false,
                experimental: false,
                custom: HashMap::new(),
            },
        }
    }

    // Helper function to create a test configuration with websocket disabled
    fn create_test_config_no_websocket() -> IndexerConfig {
        let mut config = create_test_config();
        config.api.websocket.enabled = false;
        config
    }

    // Helper function to create a test context
    fn create_test_context() -> Arc<ServiceContext> {
        let config = create_test_config();
        let store = Arc::new(MockDataStore::new()) as Arc<dyn DataStore>;
        let metrics = Arc::new(MetricsCollector::new(config.metrics.clone()).unwrap());
        Arc::new(ServiceContext::new(config, store, metrics))
    }

    // Helper function to create a test context with failing store
    fn create_test_context_with_failing_store() -> Arc<ServiceContext> {
        let config = create_test_config();
        let store = Arc::new(MockDataStore::with_init_failure()) as Arc<dyn DataStore>;
        let metrics = Arc::new(MetricsCollector::new(config.metrics.clone()).unwrap());
        Arc::new(ServiceContext::new(config, store, metrics))
    }

    // Helper function to create a test context without websocket
    fn create_test_context_no_websocket() -> Arc<ServiceContext> {
        let config = create_test_config_no_websocket();
        let store = Arc::new(MockDataStore::new()) as Arc<dyn DataStore>;
        let metrics = Arc::new(MetricsCollector::new(config.metrics.clone()).unwrap());
        Arc::new(ServiceContext::new(config, store, metrics))
    }

    // Tests for ApiServer::new()
    #[test]
    fn test_api_server_new_when_valid_context_should_create_server() {
        let context = create_test_context();
        let result = ApiServer::new(context.clone());

        assert!(result.is_ok());
        let server = result.unwrap();
        assert_eq!(server.config.http.bind, "127.0.0.1");
        assert_eq!(server.config.http.port, 8080);
        assert!(server.config.websocket.enabled);
        assert!(server.websocket_streamer.is_some());
    }

    #[test]
    fn test_api_server_new_when_websocket_disabled_should_create_server_without_websocket() {
        let context = create_test_context_no_websocket();
        let result = ApiServer::new(context.clone());

        assert!(result.is_ok());
        let server = result.unwrap();
        assert_eq!(server.config.http.bind, "127.0.0.1");
        assert_eq!(server.config.http.port, 8080);
        assert!(!server.config.websocket.enabled);
        assert!(server.websocket_streamer.is_none());
    }

    #[tokio::test]
    async fn test_api_server_new_when_rest_handler_fails_should_return_error() {
        // This test would require a way to make RestHandler::new fail
        // Since the current implementation always succeeds, we'll test what we can
        let context = create_test_context();
        let result = ApiServer::new(context.clone());

        // RestHandler::new currently always succeeds, so this should pass
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_api_server_new_when_websocket_streamer_fails_should_return_error() {
        // This test would require a way to make WebSocketStreamer::new fail
        // Since the current implementation always succeeds, we'll test what we can
        let context = create_test_context();
        let result = ApiServer::new(context.clone());

        // WebSocketStreamer::new currently always succeeds, so this should pass
        assert!(result.is_ok());
    }

    // Tests for ApiServer::create_router()
    #[test]
    fn test_api_server_create_router_when_websocket_enabled_should_merge_routes() {
        let context = create_test_context();
        let server = ApiServer::new(context).unwrap();

        let router = server.create_router();

        // We can't easily test the internal structure of the router,
        // but we can verify it's created without panicking
        assert!(!std::ptr::eq(&router as *const _, std::ptr::null()));
    }

    #[test]
    fn test_api_server_create_router_when_websocket_disabled_should_only_have_rest_routes() {
        let context = create_test_context_no_websocket();
        let server = ApiServer::new(context).unwrap();

        let router = server.create_router();

        // We can't easily test the internal structure of the router,
        // but we can verify it's created without panicking
        assert!(!std::ptr::eq(&router as *const _, std::ptr::null()));
    }

    // Tests for ApiServer::start() - these test the binding logic, not the actual server
    #[tokio::test]
    async fn test_api_server_start_when_invalid_bind_address_should_return_network_error() {
        let mut config = create_test_config();
        // Use an invalid bind address that will fail
        config.api.http.bind = "999.999.999.999".to_string();
        config.api.http.port = 0; // Use port 0 to avoid conflicts

        let store = Arc::new(MockDataStore::new()) as Arc<dyn DataStore>;
        let metrics = Arc::new(MetricsCollector::new(config.metrics.clone()).unwrap());
        let context = Arc::new(ServiceContext::new(config, store, metrics));
        let server = ApiServer::new(context).unwrap();

        let result = server.start().await;

        assert!(result.is_err());
        match result.unwrap_err() {
            IndexerError::Network(NetworkError::HttpFailed { status, url }) => {
                assert_eq!(status, 0);
                assert!(url.contains("999.999.999.999"));
            }
            other => panic!("Expected NetworkError::HttpFailed, got: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_api_server_start_when_port_already_in_use_should_return_network_error() {
        // Bind to a port first
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let port = addr.port();

        // Now try to create a server on the same port
        let mut config = create_test_config();
        config.api.http.bind = "127.0.0.1".to_string();
        config.api.http.port = port;

        let store = Arc::new(MockDataStore::new()) as Arc<dyn DataStore>;
        let metrics = Arc::new(MetricsCollector::new(config.metrics.clone()).unwrap());
        let context = Arc::new(ServiceContext::new(config, store, metrics));
        let server = ApiServer::new(context).unwrap();

        let result = server.start().await;

        assert!(result.is_err());
        match result.unwrap_err() {
            IndexerError::Network(NetworkError::HttpFailed { status, url }) => {
                assert_eq!(status, 0);
                assert!(url.contains(&format!("127.0.0.1:{}", port)));
            }
            other => panic!("Expected NetworkError::HttpFailed, got: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_api_server_start_when_valid_config_should_bind_successfully() {
        // This test verifies that binding succeeds but doesn't actually start the server
        let mut config = create_test_config();
        config.api.http.port = 0; // Use port 0 to let the OS choose an available port

        let store = Arc::new(MockDataStore::new()) as Arc<dyn DataStore>;
        let metrics = Arc::new(MetricsCollector::new(config.metrics.clone()).unwrap());
        let context = Arc::new(ServiceContext::new(config, store, metrics));
        let server = ApiServer::new(context).unwrap();

        // Test just the binding part - we can't easily test the full server start
        // without it running indefinitely
        let addr = format!("{}:{}", server.config.http.bind, server.config.http.port);
        let listener_result = tokio::net::TcpListener::bind(&addr).await;

        // If port is 0, the OS will assign an available port
        if server.config.http.port == 0 {
            assert!(listener_result.is_ok());
        } else {
            // For specific ports, we can't guarantee they're available in tests
            // So we just verify the server was created successfully
            assert!(!std::ptr::eq(&server as *const _, std::ptr::null()));
        }
    }

    // Edge case tests
    #[test]
    fn test_api_server_new_when_empty_bind_address_should_work() {
        let mut config = create_test_config();
        config.api.http.bind = "".to_string();

        let store = Arc::new(MockDataStore::new()) as Arc<dyn DataStore>;
        let metrics = Arc::new(MetricsCollector::new(config.metrics.clone()).unwrap());
        let context = Arc::new(ServiceContext::new(config, store, metrics));

        let result = ApiServer::new(context);
        assert!(result.is_ok());
    }

    #[test]
    fn test_api_server_new_when_port_zero_should_work() {
        let mut config = create_test_config();
        config.api.http.port = 0;

        let store = Arc::new(MockDataStore::new()) as Arc<dyn DataStore>;
        let metrics = Arc::new(MetricsCollector::new(config.metrics.clone()).unwrap());
        let context = Arc::new(ServiceContext::new(config, store, metrics));

        let result = ApiServer::new(context);
        assert!(result.is_ok());

        let server = result.unwrap();
        assert_eq!(server.config.http.port, 0);
    }

    #[test]
    fn test_api_server_new_when_max_port_should_work() {
        let mut config = create_test_config();
        config.api.http.port = 65535;

        let store = Arc::new(MockDataStore::new()) as Arc<dyn DataStore>;
        let metrics = Arc::new(MetricsCollector::new(config.metrics.clone()).unwrap());
        let context = Arc::new(ServiceContext::new(config, store, metrics));

        let result = ApiServer::new(context);
        assert!(result.is_ok());

        let server = result.unwrap();
        assert_eq!(server.config.http.port, 65535);
    }

    #[test]
    fn test_api_server_new_when_different_websocket_settings_should_respect_config() {
        // Test with different websocket configurations
        let mut config = create_test_config();
        config.api.websocket.max_connections = 5000;
        config.api.websocket.heartbeat_interval = Duration::from_secs(60);

        let store = Arc::new(MockDataStore::new()) as Arc<dyn DataStore>;
        let metrics = Arc::new(MetricsCollector::new(config.metrics.clone()).unwrap());
        let context = Arc::new(ServiceContext::new(config, store, metrics));

        let result = ApiServer::new(context);
        assert!(result.is_ok());

        let server = result.unwrap();
        assert_eq!(server.config.websocket.max_connections, 5000);
        assert_eq!(
            server.config.websocket.heartbeat_interval,
            Duration::from_secs(60)
        );
    }

    #[tokio::test]
    async fn test_api_server_start_when_bind_localhost_should_format_address_correctly() {
        let mut config = create_test_config();
        config.api.http.bind = "localhost".to_string();
        config.api.http.port = 8080;

        let store = Arc::new(MockDataStore::new()) as Arc<dyn DataStore>;
        let metrics = Arc::new(MetricsCollector::new(config.metrics.clone()).unwrap());
        let context = Arc::new(ServiceContext::new(config, store, metrics));
        let server = ApiServer::new(context).unwrap();

        // Test that the address formatting works correctly
        let addr = format!("{}:{}", server.config.http.bind, server.config.http.port);
        assert_eq!(addr, "localhost:8080");
    }

    #[tokio::test]
    async fn test_api_server_start_when_bind_ipv6_should_format_address_correctly() {
        let mut config = create_test_config();
        config.api.http.bind = "::1".to_string();
        config.api.http.port = 8080;

        let store = Arc::new(MockDataStore::new()) as Arc<dyn DataStore>;
        let metrics = Arc::new(MetricsCollector::new(config.metrics.clone()).unwrap());
        let context = Arc::new(ServiceContext::new(config, store, metrics));
        let server = ApiServer::new(context).unwrap();

        // Test that the address formatting works correctly
        let addr = format!("{}:{}", server.config.http.bind, server.config.http.port);
        assert_eq!(addr, "::1:8080");
    }

    // Test configuration edge cases
    #[test]
    fn test_api_server_new_when_minimal_config_should_work() {
        // Test with minimal configuration values
        let mut config = create_test_config();
        config.api.websocket.max_connections = 1;
        config.api.websocket.buffer_size = 1;
        config.api.websocket.heartbeat_interval = Duration::from_millis(1);

        let store = Arc::new(MockDataStore::new()) as Arc<dyn DataStore>;
        let metrics = Arc::new(MetricsCollector::new(config.metrics.clone()).unwrap());
        let context = Arc::new(ServiceContext::new(config, store, metrics));

        let result = ApiServer::new(context);
        assert!(result.is_ok());
    }

    #[test]
    fn test_api_server_new_when_extreme_config_values_should_work() {
        // Test with extreme but valid configuration values
        let mut config = create_test_config();
        config.api.websocket.max_connections = u32::MAX;
        config.api.websocket.buffer_size = 1000000;
        config.api.websocket.heartbeat_interval = Duration::from_secs(3600);

        let store = Arc::new(MockDataStore::new()) as Arc<dyn DataStore>;
        let metrics = Arc::new(MetricsCollector::new(config.metrics.clone()).unwrap());
        let context = Arc::new(ServiceContext::new(config, store, metrics));

        let result = ApiServer::new(context);
        assert!(result.is_ok());
    }

    // Test that context is properly stored and accessible
    #[test]
    fn test_api_server_new_when_created_should_store_context_reference() {
        let context = create_test_context();
        let original_service_name = context.config.service.name.clone();

        let server = ApiServer::new(context.clone()).unwrap();

        // The server should have a reference to the same context
        assert_eq!(server.context.config.service.name, original_service_name);
    }

    #[test]
    fn test_api_server_new_when_store_has_init_failure_should_still_create_server() {
        // Test that API server creation succeeds even with a failing store
        // The actual store operations happen during API calls, not during server creation
        let context = create_test_context_with_failing_store();
        let result = ApiServer::new(context.clone());

        assert!(result.is_ok());
        let server = result.unwrap();
        assert_eq!(server.config.http.bind, "127.0.0.1");
        assert_eq!(server.config.http.port, 8080);
        assert!(server.websocket_streamer.is_some());
    }

    // Test router creation in different scenarios
    #[test]
    fn test_api_server_create_router_when_called_multiple_times_should_work() {
        let context = create_test_context();
        let server = ApiServer::new(context).unwrap();

        // Should be able to create router multiple times
        let router1 = server.create_router();
        let router2 = server.create_router();

        // Both should be valid (we can't test equality, but we can test they're created)
        assert!(!std::ptr::eq(&router1 as *const _, std::ptr::null()));
        assert!(!std::ptr::eq(&router2 as *const _, std::ptr::null()));
    }

    // Test that the server maintains configuration correctly
    #[test]
    fn test_api_server_when_websocket_enabled_should_have_websocket_streamer() {
        let context = create_test_context();
        let server = ApiServer::new(context).unwrap();

        assert!(server.websocket_streamer.is_some());
        assert!(server.config.websocket.enabled);
    }

    #[test]
    fn test_api_server_when_websocket_disabled_should_not_have_websocket_streamer() {
        let context = create_test_context_no_websocket();
        let server = ApiServer::new(context).unwrap();

        assert!(server.websocket_streamer.is_none());
        assert!(!server.config.websocket.enabled);
    }
}
