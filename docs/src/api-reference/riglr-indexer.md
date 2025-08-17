# riglr-indexer API Reference

Comprehensive API documentation for the `riglr-indexer` crate.

## Table of Contents

### Enums

- [`AuthMethod`](#authmethod)
- [`BatchProcessorError`](#batchprocessorerror)
- [`CacheBackend`](#cachebackend)
- [`CompressionAlgorithm`](#compressionalgorithm)
- [`ConfigError`](#configerror)
- [`IndexerError`](#indexererror)
- [`LogFormat`](#logformat)
- [`LogOutput`](#logoutput)
- [`MetricType`](#metrictype)
- [`MetricType`](#metrictype)
- [`MetricValue`](#metricvalue)
- [`MetricsError`](#metricserror)
- [`NetworkError`](#networkerror)
- [`ProcessingError`](#processingerror)
- [`QueueType`](#queuetype)
- [`RateLimitError`](#ratelimiterror)
- [`ServiceError`](#serviceerror)
- [`ServiceState`](#servicestate)
- [`StorageBackend`](#storagebackend)
- [`StorageError`](#storageerror)
- [`StreamMessage`](#streammessage)
- [`SyncStrategy`](#syncstrategy)

### Functions

- [`active_workers`](#active_workers)
- [`add_item`](#add_item)
- [`add_node`](#add_node)
- [`add_node`](#add_node)
- [`add_service`](#add_service)
- [`add_simple_node`](#add_simple_node)
- [`add_stage`](#add_stage)
- [`aggregate_by_source`](#aggregate_by_source)
- [`aggregate_by_time`](#aggregate_by_time)
- [`aggregate_by_type`](#aggregate_by_type)
- [`available_tokens`](#available_tokens)
- [`broadcast_event`](#broadcast_event)
- [`broadcast_health`](#broadcast_health)
- [`broadcast_message`](#broadcast_message)
- [`broadcast_metrics`](#broadcast_metrics)
- [`build`](#build)
- [`calculate_percentile`](#calculate_percentile)
- [`category`](#category)
- [`check`](#check)
- [`check_all`](#check_all)
- [`check_and_adapt`](#check_and_adapt)
- [`check_one`](#check_one)
- [`check_tokens`](#check_tokens)
- [`clear`](#clear)
- [`clear_cache`](#clear_cache)
- [`config`](#config)
- [`context`](#context)
- [`create_metrics_router`](#create_metrics_router)
- [`create_router`](#create_router)
- [`create_router`](#create_router)
- [`create_router`](#create_router)
- [`create_store`](#create_store)
- [`current_batch_size`](#current_batch_size)
- [`error`](#error)
- [`export_json`](#export_json)
- [`export_prometheus`](#export_prometheus)
- [`export_prometheus`](#export_prometheus)
- [`feature_enabled`](#feature_enabled)
- [`flush`](#flush)
- [`flush`](#flush)
- [`format_bytes`](#format_bytes)
- [`format_duration`](#format_duration)
- [`from_env`](#from_env)
- [`generate_id`](#generate_id)
- [`get_all_metrics`](#get_all_metrics)
- [`get_all_nodes`](#get_all_nodes)
- [`get_event`](#get_event)
- [`get_event_stats`](#get_event_stats)
- [`get_events`](#get_events)
- [`get_health`](#get_health)
- [`get_indexer_metrics`](#get_indexer_metrics)
- [`get_load_distribution`](#get_load_distribution)
- [`get_metric`](#get_metric)
- [`get_metrics`](#get_metrics)
- [`get_node`](#get_node)
- [`get_nodes`](#get_nodes)
- [`get_performance_metrics`](#get_performance_metrics)
- [`has_node`](#has_node)
- [`hash_string`](#hash_string)
- [`health_check`](#health_check)
- [`health_status`](#health_status)
- [`health_summary`](#health_summary)
- [`healthy`](#healthy)
- [`healthy`](#healthy)
- [`increment_counter`](#increment_counter)
- [`increment_counter`](#increment_counter)
- [`internal`](#internal)
- [`internal_with_source`](#internal_with_source)
- [`is_empty`](#is_empty)
- [`is_healthy`](#is_healthy)
- [`is_retriable`](#is_retriable)
- [`is_retriable`](#is_retriable)
- [`is_retriable`](#is_retriable)
- [`is_retriable`](#is_retriable)
- [`is_retriable`](#is_retriable)
- [`is_shutdown_requested`](#is_shutdown_requested)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`next_batch`](#next_batch)
- [`node_count`](#node_count)
- [`node_id`](#node_id)
- [`oldest_item_age`](#oldest_item_age)
- [`process`](#process)
- [`queue_depth`](#queue_depth)
- [`receive_events`](#receive_events)
- [`record_counter`](#record_counter)
- [`record_error`](#record_error)
- [`record_gauge`](#record_gauge)
- [`record_histogram`](#record_histogram)
- [`record_histogram`](#record_histogram)
- [`record_success`](#record_success)
- [`register`](#register)
- [`register_metric`](#register_metric)
- [`remove_node`](#remove_node)
- [`request_shutdown`](#request_shutdown)
- [`reset`](#reset)
- [`retry_with_backoff`](#retry_with_backoff)
- [`set_gauge`](#set_gauge)
- [`set_state`](#set_state)
- [`shutdown_all`](#shutdown_all)
- [`shutdown_receiver`](#shutdown_receiver)
- [`start`](#start)
- [`start_age_flusher`](#start_age_flusher)
- [`start_background_checks`](#start_background_checks)
- [`start_collection_task`](#start_collection_task)
- [`start_metrics_server`](#start_metrics_server)
- [`state`](#state)
- [`stats`](#stats)
- [`stats`](#stats)
- [`subscribe`](#subscribe)
- [`success`](#success)
- [`try_next_batch`](#try_next_batch)
- [`unhealthy`](#unhealthy)
- [`unhealthy`](#unhealthy)
- [`unlimited`](#unlimited)
- [`unregister`](#unregister)
- [`update_component_health`](#update_component_health)
- [`update_indexer_metrics`](#update_indexer_metrics)
- [`validate`](#validate)
- [`validate_event_id`](#validate_event_id)
- [`validate_event_type`](#validate_event_type)
- [`validate_source`](#validate_source)
- [`validation`](#validation)
- [`virtual_nodes`](#virtual_nodes)
- [`wait_for_tokens`](#wait_for_tokens)
- [`websocket_handler`](#websocket_handler)
- [`with_detail`](#with_detail)
- [`with_response_time`](#with_response_time)

### Structs

- [`AdaptiveRateLimiter`](#adaptiveratelimiter)
- [`ApiConfig`](#apiconfig)
- [`ApiKeyConfig`](#apikeyconfig)
- [`ApiResponse`](#apiresponse)
- [`ApiServer`](#apiserver)
- [`ArchiveConfig`](#archiveconfig)
- [`AuthConfig`](#authconfig)
- [`BatchConfig`](#batchconfig)
- [`BatchProcessor`](#batchprocessor)
- [`CacheConfig`](#cacheconfig)
- [`CacheTtlConfig`](#cachettlconfig)
- [`ComponentHealth`](#componenthealth)
- [`ComponentStatus`](#componentstatus)
- [`CompressionConfig`](#compressionconfig)
- [`ConnectionHealthCheck`](#connectionhealthcheck)
- [`ConnectionPoolConfig`](#connectionpoolconfig)
- [`ConsistentHash`](#consistenthash)
- [`ConsistentHashBuilder`](#consistenthashbuilder)
- [`CorsConfig`](#corsconfig)
- [`CustomMetricConfig`](#custommetricconfig)
- [`DiskQueueConfig`](#diskqueueconfig)
- [`EnrichmentStage`](#enrichmentstage)
- [`ErrorMetrics`](#errormetrics)
- [`EventAggregator`](#eventaggregator)
- [`EventFilter`](#eventfilter)
- [`EventIngester`](#eventingester)
- [`EventProcessor`](#eventprocessor)
- [`EventQuery`](#eventquery)
- [`EventStatsResponse`](#eventstatsresponse)
- [`EventSubscriptionFilter`](#eventsubscriptionfilter)
- [`EventsQueryParams`](#eventsqueryparams)
- [`FeatureConfig`](#featureconfig)
- [`GraphQLConfig`](#graphqlconfig)
- [`HealthCheckCoordinator`](#healthcheckcoordinator)
- [`HealthCheckResult`](#healthcheckresult)
- [`HealthResponse`](#healthresponse)
- [`HealthStatus`](#healthstatus)
- [`HealthSummary`](#healthsummary)
- [`HttpConfig`](#httpconfig)
- [`HttpHealthCheck`](#httphealthcheck)
- [`IndexerConfig`](#indexerconfig)
- [`IndexerMetrics`](#indexermetrics)
- [`IndexerService`](#indexerservice)
- [`IngesterConfig`](#ingesterconfig)
- [`IngestionStats`](#ingestionstats)
- [`JwtConfig`](#jwtconfig)
- [`LatencyMetrics`](#latencymetrics)
- [`LoggingConfig`](#loggingconfig)
- [`MemoryCache`](#memorycache)
- [`MemoryCacheConfig`](#memorycacheconfig)
- [`Metric`](#metric)
- [`MetricsCollector`](#metricscollector)
- [`MetricsConfig`](#metricsconfig)
- [`MetricsRegistry`](#metricsregistry)
- [`NodeInfo`](#nodeinfo)
- [`PerformanceMetrics`](#performancemetrics)
- [`PostgresStore`](#postgresstore)
- [`ProcessingConfig`](#processingconfig)
- [`ProcessingPipeline`](#processingpipeline)
- [`ProcessingStats`](#processingstats)
- [`ProcessorConfig`](#processorconfig)
- [`ProcessorStats`](#processorstats)
- [`PrometheusState`](#prometheusstate)
- [`QueueConfig`](#queueconfig)
- [`RateLimitConfig`](#ratelimitconfig)
- [`RateLimiter`](#ratelimiter)
- [`RateLimiterConfig`](#ratelimiterconfig)
- [`RateLimiterStats`](#ratelimiterstats)
- [`RedisCache`](#rediscache)
- [`ResourceMetrics`](#resourcemetrics)
- [`RestHandler`](#resthandler)
- [`RetentionConfig`](#retentionconfig)
- [`RetryConfig`](#retryconfig)
- [`ServiceConfig`](#serviceconfig)
- [`ServiceContext`](#servicecontext)
- [`ShutdownCoordinator`](#shutdowncoordinator)
- [`StorageBackendConfig`](#storagebackendconfig)
- [`StorageConfig`](#storageconfig)
- [`StorageStats`](#storagestats)
- [`StoredEvent`](#storedevent)
- [`StructuredLoggingConfig`](#structuredloggingconfig)
- [`SubscriptionRequest`](#subscriptionrequest)
- [`SummaryValue`](#summaryvalue)
- [`ThroughputMetrics`](#throughputmetrics)
- [`ValidationStage`](#validationstage)
- [`WebSocketConfig`](#websocketconfig)
- [`WebSocketStreamer`](#websocketstreamer)

### Traits

- [`Cache`](#cache)
- [`DataStore`](#datastore)
- [`EventProcessing`](#eventprocessing)
- [`HealthCheck`](#healthcheck)
- [`PipelineStage`](#pipelinestage)
- [`ServiceLifecycle`](#servicelifecycle)

### Constants

- [`POSTGRES_SCHEMA`](#postgres_schema)
- [`VERSION_TABLE`](#version_table)

## Enums

### AuthMethod

**Source**: `config/mod.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
```

```rust
pub enum AuthMethod { /// No authentication required None, /// API key-based authentication ApiKey, /// JSON Web Token authentication Jwt, /// OAuth 2.0 authentication OAuth2, }
```

Authentication methods

**Variants**:

- `None`
- `ApiKey`
- `Jwt`
- `OAuth2`

---

### BatchProcessorError

**Source**: `utils/batch.rs`

**Attributes**:
```rust
#[derive(Debug, thiserror::Error)]
```

```rust
pub enum BatchProcessorError { /// Failed to send batch through channel #[error("Failed to send batch")] SendFailed, }
```

Errors that can occur during batch processing

**Variants**:

- `SendFailed`

---

### CacheBackend

**Source**: `config/mod.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
```

```rust
pub enum CacheBackend { /// Redis distributed cache for shared caching across instances Redis, /// In-memory cache for single-instance deployments Memory, /// No caching enabled Disabled, }
```

Cache backend types

**Variants**:

- `Redis`
- `Memory`
- `Disabled`

---

### CompressionAlgorithm

**Source**: `config/mod.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
```

```rust
pub enum CompressionAlgorithm { /// Zstandard compression - best compression ratio Zstd, /// Gzip compression - widely compatible Gzip, /// LZ4 compression - fastest compression/decompression Lz4, /// No compression None, }
```

Compression algorithms

**Variants**:

- `Zstd`
- `Gzip`
- `Lz4`
- `None`

---

### ConfigError

**Source**: `src/error.rs`

**Attributes**:
```rust
#[derive(Error, Debug)]
```

```rust
pub enum ConfigError { /// Missing required configuration field #[error("Missing required configuration: {field}")] MissingField { /// The missing field name field: String, }, /// Invalid configuration value #[error("Invalid configuration value for {field}: {value}")] InvalidValue { /// The field with invalid value field: String, /// The invalid value value: String, }, /// Failed to load configuration from source #[error("Failed to load configuration from {path}: {source}")] LoadFailed { /// Path or source of configuration path: String, /// The underlying error #[source] source: Box<dyn std::error::Error + Send + Sync>, }, /// Configuration validation failed #[error("Configuration validation failed: {message}")] ValidationFailed { /// Validation error message message: String, }, }
```

Configuration-related errors

**Variants**:

- `MissingField`
- `field`
- `InvalidValue`
- `field`
- `value`
- `LoadFailed`
- `path`
- `source`
- `ValidationFailed`
- `message`

---

### IndexerError

**Source**: `src/error.rs`

**Attributes**:
```rust
#[derive(Error, Debug)]
```

```rust
pub enum IndexerError { /// Configuration errors #[error("Configuration error: {0}")] Config(#[from] ConfigError), /// Storage layer errors #[error("Storage error: {0}")] Storage(#[from] StorageError), /// Network/API errors #[error("Network error: {0}")] Network(#[from] NetworkError), /// Event processing errors #[error("Processing error: {0}")] Processing(#[from] ProcessingError), /// Metrics collection errors #[error("Metrics error: {0}")] Metrics(#[from] MetricsError), /// Service lifecycle errors #[error("Service error: {0}")] Service(#[from] ServiceError), /// External dependency errors #[error("Dependency error: {source}")] Dependency { /// The underlying error from a dependency #[from] source: Box<dyn std::error::Error + Send + Sync>, }, /// Validation errors #[error("Validation error: {message}")] Validation { /// Human-readable validation error message message: String, }, /// Internal errors that should not occur in normal operation #[error("Internal error: {message}")] Internal { /// Description of the internal error message: String, /// Optional source error source: Option<Box<dyn std::error::Error + Send + Sync>>, }, }
```

Main error type for the indexer service

**Variants**:

- `Config(#[from] ConfigError)`
- `Storage(#[from] StorageError)`
- `Network(#[from] NetworkError)`
- `Processing(#[from] ProcessingError)`
- `Metrics(#[from] MetricsError)`
- `Service(#[from] ServiceError)`
- `Dependency`
- `source`
- `Validation`
- `message`
- `Internal`
- `message`
- `source`

---

### LogFormat

**Source**: `config/mod.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
```

```rust
pub enum LogFormat { /// JSON structured logging format Json, /// Plain text logging format Text, /// Pretty-printed format for development Pretty, }
```

Log format options

**Variants**:

- `Json`
- `Text`
- `Pretty`

---

### LogOutput

**Source**: `config/mod.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
```

```rust
pub enum LogOutput { /// Standard output stream Stdout, /// Standard error stream Stderr, /// Log to specified file path File(PathBuf), /// System log daemon Syslog, }
```

Log output destinations

**Variants**:

- `Stdout`
- `Stderr`
- `File(PathBuf)`
- `Syslog`

---

### MetricType

**Source**: `config/mod.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
```

```rust
pub enum MetricType { /// Counter metric that only increases Counter, /// Gauge metric that can go up or down Gauge, /// Histogram for observing value distributions Histogram, /// Summary for calculating quantiles Summary, }
```

Metric types

**Variants**:

- `Counter`
- `Gauge`
- `Histogram`
- `Summary`

---

### MetricType

**Source**: `metrics/mod.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, PartialEq, Serialize)]
```

```rust
pub enum MetricType { /// Counter metric that only increases Counter, /// Gauge metric that can go up or down Gauge, /// Histogram for observing value distributions Histogram, /// Summary for calculating quantiles Summary, }
```

Metric types

**Variants**:

- `Counter`
- `Gauge`
- `Histogram`
- `Summary`

---

### MetricValue

**Source**: `metrics/mod.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize)]
```

```rust
pub enum MetricValue { /// Counter value Counter(u64), /// Gauge value Gauge(f64), /// Histogram observations Histogram(Vec<f64>), /// Summary statistics Summary(SummaryValue), }
```

Metric value

**Variants**:

- `Counter(u64)`
- `Gauge(f64)`
- `Histogram(Vec<f64>)`
- `Summary(SummaryValue)`

---

### MetricsError

**Source**: `src/error.rs`

**Attributes**:
```rust
#[derive(Error, Debug)]
```

```rust
pub enum MetricsError { /// Metrics registry error #[error("Metrics registry error: {message}")] RegistryError { /// Error message message: String, }, /// Metric recording failed #[error("Metric recording failed: {metric_name}")] RecordingFailed { /// Name of metric that failed to record metric_name: String, }, /// Metrics export failed #[error("Metrics export failed: {endpoint}")] ExportFailed { /// Export endpoint that failed endpoint: String, }, /// Invalid metric value #[error("Invalid metric value: {value}")] InvalidValue { /// The invalid value value: String, }, }
```

Metrics collection errors

**Variants**:

- `RegistryError`
- `message`
- `RecordingFailed`
- `metric_name`
- `ExportFailed`
- `endpoint`
- `InvalidValue`
- `value`

---

### NetworkError

**Source**: `src/error.rs`

**Attributes**:
```rust
#[derive(Error, Debug)]
```

```rust
pub enum NetworkError { /// HTTP request failed #[error("HTTP request failed: {status} - {url}")] HttpFailed { /// HTTP status code status: u16, /// Request URL url: String, }, /// WebSocket connection failed #[error("WebSocket connection failed: {reason}")] WebSocketFailed { /// Failure reason reason: String, }, /// Request timeout #[error("Timeout occurred after {seconds}s")] Timeout { /// Timeout duration in seconds seconds: u64, }, /// Rate limit exceeded #[error("Rate limit exceeded: {limit} requests per {window}s")] RateLimited { /// Rate limit threshold limit: u32, /// Time window in seconds window: u32, }, /// DNS resolution failed #[error("DNS resolution failed for {host}")] DnsResolutionFailed { /// Host that failed to resolve host: String, }, /// TLS/SSL error #[error("TLS/SSL error: {message}")] TlsError { /// Error message message: String, }, }
```

Network and API errors

**Variants**:

- `HttpFailed`
- `status`
- `url`
- `WebSocketFailed`
- `reason`
- `Timeout`
- `seconds`
- `RateLimited`
- `limit`
- `window`
- `DnsResolutionFailed`
- `host`
- `TlsError`
- `message`

---

### ProcessingError

**Source**: `src/error.rs`

**Attributes**:
```rust
#[derive(Error, Debug)]
```

```rust
pub enum ProcessingError { /// Failed to parse event #[error("Failed to parse event: {event_type}")] ParseFailed { /// Type of event that failed to parse event_type: String, }, /// Event validation failed #[error("Event validation failed: {reason}")] ValidationFailed { /// Validation failure reason reason: String, }, /// Worker pool exhausted #[error("Worker pool exhausted: {active}/{max} workers")] WorkerPoolExhausted { /// Number of active workers active: usize, /// Maximum number of workers max: usize, }, /// Event queue overflow #[error("Event queue overflow: {size}/{capacity}")] QueueOverflow { /// Current queue size size: usize, /// Queue capacity capacity: usize, }, /// Serialization failed #[error("Serialization failed for event {event_id}")] SerializationFailed { /// ID of event that failed to serialize event_id: String, }, /// Deserialization failed #[error("Deserialization failed: {format}")] DeserializationFailed { /// Format that failed to deserialize format: String, }, /// Pipeline stalled #[error("Pipeline stalled: no events processed in {seconds}s")] PipelineStalled { /// Seconds since last event processed seconds: u64, }, }
```

Event processing errors

**Variants**:

- `ParseFailed`
- `event_type`
- `ValidationFailed`
- `reason`
- `WorkerPoolExhausted`
- `active`
- `max`
- `QueueOverflow`
- `size`
- `capacity`
- `SerializationFailed`
- `event_id`
- `DeserializationFailed`
- `format`
- `PipelineStalled`
- `seconds`

---

### QueueType

**Source**: `config/mod.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
```

```rust
pub enum QueueType { /// In-memory queue for maximum performance Memory, /// Disk-backed queue for durability Disk, /// Hybrid queue using memory with disk overflow Hybrid, }
```

Queue type options

**Variants**:

- `Memory`
- `Disk`
- `Hybrid`

---

### RateLimitError

**Source**: `utils/rate_limiter.rs`

**Attributes**:
```rust
#[derive(Debug, thiserror::Error)]
```

```rust
pub enum RateLimitError { /// Rate limit exceeded #[error("Rate limited: need {required_tokens} tokens, have {available_tokens}. Retry after {retry_after:?}")] RateLimited { /// Number of available tokens available_tokens: u32, /// Number of required tokens required_tokens: u32, /// Time to wait before retry retry_after: Duration, }, /// Lock contention error #[error("Lock contention")] LockContention, }
```

Rate limiter errors

**Variants**:

- `RateLimited`
- `available_tokens`
- `required_tokens`
- `retry_after`
- `LockContention`

---

### ServiceError

**Source**: `src/error.rs`

**Attributes**:
```rust
#[derive(Error, Debug)]
```

```rust
pub enum ServiceError { /// Service failed to start #[error("Service failed to start: {reason}")] StartupFailed { /// Startup failure reason reason: String, }, /// Service shutdown timeout #[error("Service shutdown timeout after {seconds}s")] ShutdownTimeout { /// Timeout duration in seconds seconds: u64, }, /// Service health check failed #[error("Service unhealthy: {check_name}")] HealthCheckFailed { /// Name of the health check that failed check_name: String, }, /// Resource exhausted #[error("Resource exhausted: {resource}")] ResourceExhausted { /// Name of exhausted resource resource: String, }, /// Dependency unavailable #[error("Dependency unavailable: {service}")] DependencyUnavailable { /// Name of unavailable service service: String, }, }
```

Service lifecycle errors

**Variants**:

- `StartupFailed`
- `reason`
- `ShutdownTimeout`
- `seconds`
- `HealthCheckFailed`
- `check_name`
- `ResourceExhausted`
- `resource`
- `DependencyUnavailable`
- `service`

---

### ServiceState

**Source**: `core/mod.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, PartialEq)]
```

```rust
pub enum ServiceState { /// Service is initializing Starting, /// Service is running normally Running, /// Service is shutting down gracefully Stopping, /// Service has stopped Stopped, /// Service encountered an error Error(String), }
```

Service state enumeration

**Variants**:

- `Starting`
- `Running`
- `Stopping`
- `Stopped`
- `Error(String)`

---

### StorageBackend

**Source**: `config/mod.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
```

```rust
pub enum StorageBackend { /// PostgreSQL database backend for general-purpose storage Postgres, /// ClickHouse columnar database for high-performance analytics ClickHouse, /// TimescaleDB time-series database for temporal data TimescaleDB, /// MongoDB document database for flexible schema storage MongoDB, }
```

Storage backend types

**Variants**:

- `Postgres`
- `ClickHouse`
- `TimescaleDB`
- `MongoDB`

---

### StorageError

**Source**: `src/error.rs`

**Attributes**:
```rust
#[derive(Error, Debug)]
```

```rust
pub enum StorageError { /// Database connection failed #[error("Database connection failed: {message}")] ConnectionFailed { /// Connection failure message message: String, }, /// Database query failed #[error("Database query failed: {query}")] QueryFailed { /// The failed query query: String, }, /// Migration failed #[error("Migration failed: {version}")] MigrationFailed { /// Migration version that failed version: String, }, /// Data corruption detected #[error("Data corruption detected: {details}")] DataCorruption { /// Corruption details details: String, }, /// Storage capacity exceeded #[error("Storage capacity exceeded: {current_size}")] CapacityExceeded { /// Current storage size in bytes current_size: u64, }, /// Transaction failed #[error("Transaction failed: {operation}")] TransactionFailed { /// Failed operation name operation: String, }, /// Schema validation failed #[error("Schema validation failed: {table}")] SchemaValidationFailed { /// Table with schema issues table: String, }, }
```

Storage layer errors

**Variants**:

- `ConnectionFailed`
- `message`
- `QueryFailed`
- `query`
- `MigrationFailed`
- `version`
- `DataCorruption`
- `details`
- `CapacityExceeded`
- `current_size`
- `TransactionFailed`
- `operation`
- `SchemaValidationFailed`
- `table`

---

### StreamMessage

**Source**: `api/websocket.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
```

```rust
pub enum StreamMessage { /// New event notification Event { /// The stored event data event: StoredEvent, }, /// Health status update Health { /// Overall health status healthy: bool, /// Individual component health statuses components: std::collections::HashMap<String, bool>, }, /// Metrics update Metrics { /// Metrics data as JSON value metrics: serde_json::Value, }, /// Service status update Status { /// Current service state state: String, /// Status message message: String, }, /// Error notification Error { /// Error message message: String, /// Optional error code code: Option<String>, }, /// Heartbeat/keepalive Ping, /// Pong response Pong, }
```

Message types for WebSocket streaming

**Variants**:

- `Event`
- `event`
- `Health`
- `healthy`
- `components`
- `Metrics`
- `metrics`
- `Status`
- `state`
- `message`
- `Error`
- `message`
- `code`
- `Ping`
- `Pong`

---

### SyncStrategy

**Source**: `config/mod.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
```

```rust
pub enum SyncStrategy { /// Sync to disk after every write for maximum durability Always, /// Sync to disk periodically for balanced performance Periodic, /// Never explicitly sync, rely on OS buffering Never, }
```

Disk sync strategies

**Variants**:

- `Always`
- `Periodic`
- `Never`

---

## Functions

### active_workers

**Source**: `core/processor.rs`

```rust
pub async fn active_workers(&self) -> usize
```

Get current number of active workers

---

### add_item

**Source**: `utils/batch.rs`

```rust
pub async fn add_item(&self, item: T) -> Result<(), BatchProcessorError>
```

Add an item to the current batch

---

### add_node

**Source**: `utils/consistent_hash.rs`

```rust
pub fn add_node(&mut self, node_id: String, weight: u32, metadata: HashMap<String, String>)
```

Add a node to the hash ring

---

### add_node

**Source**: `utils/consistent_hash.rs`

```rust
pub fn add_node( mut self, node_id: String, weight: u32, metadata: HashMap<String, String>, ) -> Self
```

Add a node

---

### add_service

**Source**: `core/mod.rs`

```rust
pub fn add_service(&mut self, service: Box<dyn ServiceLifecycle + Send + Sync>)
```

Add a service to coordinate

---

### add_simple_node

**Source**: `utils/consistent_hash.rs`

```rust
pub fn add_simple_node(self, node_id: String) -> Self
```

Add a simple node with default weight

---

### add_stage

**Source**: `core/processor.rs`

```rust
pub fn add_stage(mut self, stage: Box<dyn PipelineStage + Send + Sync>) -> Self
```

Add a stage to the pipeline

---

### aggregate_by_source

**Source**: `storage/mod.rs`

```rust
pub async fn aggregate_by_source( store: &dyn DataStore, filter: &EventFilter, ) -> IndexerResult<HashMap<String, u64>>
```

Aggregate events by source

---

### aggregate_by_time

**Source**: `storage/mod.rs`

```rust
pub async fn aggregate_by_time( store: &dyn DataStore, period: Duration, filter: &EventFilter, ) -> IndexerResult<HashMap<DateTime<Utc>, u64>>
```

Aggregate events by time period

---

### aggregate_by_type

**Source**: `storage/mod.rs`

```rust
pub async fn aggregate_by_type( store: &dyn DataStore, filter: &EventFilter, ) -> IndexerResult<HashMap<String, u64>>
```

Aggregate events by type

---

### available_tokens

**Source**: `utils/rate_limiter.rs`

```rust
pub async fn available_tokens(&self) -> u32
```

Get current number of available tokens

---

### broadcast_event

**Source**: `api/websocket.rs`

```rust
pub fn broadcast_event(&self, event: StoredEvent)
```

Broadcast new event

---

### broadcast_health

**Source**: `api/websocket.rs`

```rust
pub fn broadcast_health( &self, healthy: bool, components: std::collections::HashMap<String, bool>, )
```

Broadcast health update

---

### broadcast_message

**Source**: `api/websocket.rs`

```rust
pub fn broadcast_message(&self, message: StreamMessage)
```

Broadcast a message to all connected clients

---

### broadcast_metrics

**Source**: `api/websocket.rs`

```rust
pub fn broadcast_metrics(&self, metrics: serde_json::Value)
```

Broadcast metrics update

---

### build

**Source**: `utils/consistent_hash.rs`

```rust
pub fn build(self) -> ConsistentHash
```

Build the consistent hash ring

---

### calculate_percentile

**Source**: `utils/mod.rs`

```rust
pub fn calculate_percentile(sorted_values: &[f64], percentile: f64) -> Option<f64>
```

Calculate percentile from a sorted vector of values

---

### category

**Source**: `src/error.rs`

```rust
pub fn category(&self) -> &'static str
```

Get error category for metrics and logging

---

### check

**Source**: `utils/rate_limiter.rs`

```rust
pub fn check(&self) -> Result<(), RateLimitError>
```

Check if an operation can proceed (consumes 1 token if available)

---

### check_all

**Source**: `utils/health.rs`

```rust
pub async fn check_all(&self) -> HashMap<String, HealthCheckResult>
```

Perform all health checks

---

### check_and_adapt

**Source**: `utils/rate_limiter.rs`

```rust
pub async fn check_and_adapt(&self) -> Result<(), RateLimitError>
```

Check tokens and potentially adjust rate based on success/failure

---

### check_one

**Source**: `utils/health.rs`

```rust
pub async fn check_one(&self, name: &str) -> Option<HealthCheckResult>
```

Perform a specific health check

---

### check_tokens

**Source**: `utils/rate_limiter.rs`

```rust
pub fn check_tokens(&self, tokens_required: u32) -> Result<(), RateLimitError>
```

Check if an operation requiring multiple tokens can proceed

---

### clear

**Source**: `metrics/mod.rs`

```rust
pub fn clear(&self)
```

Clear all metrics

---

### clear_cache

**Source**: `utils/health.rs`

```rust
pub async fn clear_cache(&self)
```

Clear all cached results

---

### config

**Source**: `core/indexer.rs`

```rust
pub fn config(&self) -> &IndexerConfig
```

Get current configuration

---

### context

**Source**: `core/indexer.rs`

```rust
pub fn context(&self) -> &Arc<ServiceContext>
```

Get service context

---

### create_metrics_router

**Source**: `metrics/prometheus.rs`

```rust
pub fn create_metrics_router(collector: Arc<MetricsCollector>) -> Router
```

Create Prometheus metrics router

---

### create_router

**Source**: `api/mod.rs`

```rust
pub fn create_router(&self) -> Router<Arc<ServiceContext>>
```

Create the router for the API server

---

### create_router

**Source**: `api/rest.rs`

```rust
pub fn create_router(&self) -> Router<Arc<ServiceContext>>
```

Create the router for REST endpoints

---

### create_router

**Source**: `api/websocket.rs`

```rust
pub fn create_router(&self) -> Router<Arc<ServiceContext>>
```

Create WebSocket router

---

### create_store

**Source**: `storage/mod.rs`

```rust
pub async fn create_store(config: &StorageConfig) -> IndexerResult<Box<dyn DataStore>>
```

Create a data store based on configuration

---

### current_batch_size

**Source**: `utils/batch.rs`

```rust
pub async fn current_batch_size(&self) -> usize
```

Get current batch size

---

### error

**Source**: `api/rest.rs`

```rust
pub fn error(message: String) -> Self
```

Create an error response

---

### export_json

**Source**: `metrics/collector.rs`

```rust
pub fn export_json(&self) -> IndexerResult<serde_json::Value>
```

Export metrics as JSON

---

### export_prometheus

**Source**: `metrics/collector.rs`

```rust
pub fn export_prometheus(&self) -> String
```

Export metrics in Prometheus format

---

### export_prometheus

**Source**: `metrics/mod.rs`

```rust
pub fn export_prometheus(&self) -> String
```

Export metrics in Prometheus format

---

### feature_enabled

**Source**: `config/mod.rs`

```rust
pub fn feature_enabled(&self, feature: &str) -> bool
```

Get a feature flag value

---

### flush

**Source**: `metrics/collector.rs`

```rust
pub async fn flush(&self) -> IndexerResult<()>
```

Flush metrics (for graceful shutdown)

---

### flush

**Source**: `utils/batch.rs`

```rust
pub async fn flush(&self) -> Result<(), BatchProcessorError>
```

Flush the current batch immediately

---

### format_bytes

**Source**: `utils/mod.rs`

```rust
pub fn format_bytes(bytes: u64) -> String
```

Format bytes in human-readable format

---

### format_duration

**Source**: `utils/mod.rs`

```rust
pub fn format_duration(duration: Duration) -> String
```

Format duration in human-readable format

---

### from_env

**Source**: `config/mod.rs`

```rust
pub fn from_env() -> IndexerResult<Self>
```

Load configuration from environment variables and optional file

---

### generate_id

**Source**: `utils/mod.rs`

```rust
pub fn generate_id() -> String
```

Generate a unique ID

---

### get_all_metrics

**Source**: `metrics/mod.rs`

```rust
pub fn get_all_metrics(&self) -> Vec<Metric>
```

Get all metrics

---

### get_all_nodes

**Source**: `utils/consistent_hash.rs`

```rust
pub fn get_all_nodes(&self) -> Vec<&NodeInfo>
```

Get all active nodes

---

### get_event

**Source**: `api/rest.rs`

```rust
pub async fn get_event( State(context): State<Arc<ServiceContext>>, Path(id): Path<String>, ) -> Result<Json<ApiResponse<Option<StoredEvent>>>, StatusCode>
```

Get single event endpoint

---

### get_event_stats

**Source**: `api/rest.rs`

```rust
pub async fn get_event_stats( State(context): State<Arc<ServiceContext>>, ) -> Result<Json<ApiResponse<EventStatsResponse>>, StatusCode>
```

Get event statistics endpoint

---

### get_events

**Source**: `api/rest.rs`

```rust
pub async fn get_events( State(context): State<Arc<ServiceContext>>, Query(params): Query<EventsQueryParams>, ) -> Result<Json<ApiResponse<Vec<StoredEvent>>>, StatusCode>
```

Get events endpoint

---

### get_health

**Source**: `api/rest.rs`

```rust
pub async fn get_health( State(context): State<Arc<ServiceContext>>, ) -> Result<Json<ApiResponse<HealthResponse>>, StatusCode>
```

Get service health endpoint

---

### get_indexer_metrics

**Source**: `metrics/collector.rs`

```rust
pub async fn get_indexer_metrics(&self) -> IndexerMetrics
```

Get current indexer metrics

---

### get_load_distribution

**Source**: `utils/consistent_hash.rs`

```rust
pub fn get_load_distribution(&self, sample_keys: &[String]) -> HashMap<String, usize>
```

Get load distribution (for debugging/monitoring)

---

### get_metric

**Source**: `metrics/mod.rs`

```rust
pub fn get_metric(&self, name: &str) -> Option<Metric>
```

Get a metric by name

---

### get_metrics

**Source**: `api/rest.rs`

```rust
pub async fn get_metrics( State(context): State<Arc<ServiceContext>>, ) -> Result<Json<ApiResponse<serde_json::Value>>, StatusCode>
```

Get metrics endpoint

---

### get_node

**Source**: `utils/consistent_hash.rs`

```rust
pub fn get_node(&self, key: &str) -> Option<&NodeInfo>
```

Get the node responsible for a given key

---

### get_nodes

**Source**: `utils/consistent_hash.rs`

```rust
pub fn get_nodes(&self, key: &str, count: usize) -> Vec<&NodeInfo>
```

Get multiple nodes for replication (returns in order of preference)

---

### get_performance_metrics

**Source**: `metrics/collector.rs`

```rust
pub async fn get_performance_metrics(&self) -> PerformanceMetrics
```

Get performance metrics summary

---

### has_node

**Source**: `utils/consistent_hash.rs`

```rust
pub fn has_node(&self, node_id: &str) -> bool
```

Check if a node exists

---

### hash_string

**Source**: `utils/mod.rs`

```rust
pub fn hash_string(s: &str) -> u64
```

Simple hash function for consistent hashing

---

### health_check

**Source**: `core/mod.rs`

```rust
pub async fn health_check(&self) -> IndexerResult<HealthStatus>
```

Perform health check on all components

---

### health_status

**Source**: `core/mod.rs`

```rust
pub async fn health_status(&self) -> HealthStatus
```

Get current health status

---

### health_summary

**Source**: `utils/health.rs`

```rust
pub async fn health_summary(&self) -> HealthSummary
```

Get overall health summary

---

### healthy

**Source**: `core/mod.rs`

```rust
pub fn healthy(message: &str) -> Self
```

Create a healthy component status

---

### healthy

**Source**: `utils/health.rs`

```rust
pub fn healthy(message: &str) -> Self
```

Create a healthy result

---

### increment_counter

**Source**: `metrics/collector.rs`

```rust
pub fn increment_counter(&self, name: &str)
```

Record a counter increment

---

### increment_counter

**Source**: `metrics/mod.rs`

```rust
pub fn increment_counter(&self, name: &str, value: u64) -> IndexerResult<()>
```

Update counter metric

---

### internal

**Source**: `src/error.rs`

```rust
pub fn internal(message: impl Into<String>) -> Self
```

Create an internal error

---

### internal_with_source

**Source**: `src/error.rs`

```rust
pub fn internal_with_source( message: impl Into<String>, source: impl std::error::Error + Send + Sync + 'static, ) -> Self
```

Create an internal error with a source

---

### is_empty

**Source**: `utils/batch.rs`

```rust
pub async fn is_empty(&self) -> bool
```

Check if the current batch is empty

---

### is_healthy

**Source**: `utils/health.rs`

```rust
pub async fn is_healthy(&self) -> bool
```

Check if all components are healthy

---

### is_retriable

**Source**: `src/error.rs`

```rust
pub fn is_retriable(&self) -> bool
```

Check if the error is retriable

---

### is_retriable

**Source**: `src/error.rs`

```rust
pub fn is_retriable(&self) -> bool
```

Check if the storage error is retriable

---

### is_retriable

**Source**: `src/error.rs`

```rust
pub fn is_retriable(&self) -> bool
```

Check if the network error is retriable

---

### is_retriable

**Source**: `src/error.rs`

```rust
pub fn is_retriable(&self) -> bool
```

Check if the processing error is retriable

---

### is_retriable

**Source**: `src/error.rs`

```rust
pub fn is_retriable(&self) -> bool
```

Check if the service error is retriable

---

### is_shutdown_requested

**Source**: `core/mod.rs`

```rust
pub fn is_shutdown_requested(&self) -> bool
```

Check if shutdown has been requested

---

### new

**Source**: `api/mod.rs`

```rust
pub fn new(context: Arc<ServiceContext>) -> IndexerResult<Self>
```

Create a new API server

---

### new

**Source**: `api/rest.rs`

```rust
pub fn new(context: Arc<ServiceContext>) -> IndexerResult<Self>
```

Create a new REST handler

---

### new

**Source**: `api/websocket.rs`

```rust
pub fn new(context: Arc<ServiceContext>) -> IndexerResult<Self>
```

Create a new WebSocket streamer

---

### new

**Source**: `core/mod.rs`

```rust
pub fn new( config: IndexerConfig, store: Arc<dyn DataStore>, metrics: Arc<MetricsCollector>, ) -> Self
```

Create a new service context

---

### new

**Source**: `core/mod.rs`

```rust
pub fn new(timeout: std::time::Duration) -> Self
```

Create a new shutdown coordinator

---

### new

**Source**: `core/pipeline.rs`

```rust
pub fn new() -> Self
```

Create a new validation stage

---

### new

**Source**: `core/pipeline.rs`

```rust
pub fn new() -> Self
```

Create a new enrichment stage

---

### new

**Source**: `core/processor.rs`

```rust
pub async fn new(config: ProcessorConfig, context: Arc<ServiceContext>) -> IndexerResult<Self>
```

Create a new event processor

---

### new

**Source**: `core/processor.rs`

```rust
pub fn new() -> Self
```

Create a new processing pipeline

---

### new

**Source**: `core/ingester.rs`

```rust
pub async fn new(config: IngesterConfig, context: Arc<ServiceContext>) -> IndexerResult<Self>
```

Create a new event ingester

---

### new

**Source**: `core/indexer.rs`

```rust
pub async fn new(config: IndexerConfig) -> IndexerResult<Self>
```

Create a new indexer service

---

### new

**Source**: `metrics/collector.rs`

```rust
pub fn new(config: MetricsConfig) -> IndexerResult<Self>
```

Create a new metrics collector

---

### new

**Source**: `metrics/mod.rs`

```rust
pub fn new(config: MetricsConfig) -> Self
```

Create a new metrics registry

---

### new

**Source**: `storage/postgres.rs`

```rust
pub async fn new(config: &StorageBackendConfig) -> IndexerResult<Self>
```

Create a new PostgreSQL store

---

### new

**Source**: `utils/rate_limiter.rs`

```rust
pub fn new(tokens_per_second: u32, burst_capacity: u32) -> Self
```

Create a new rate limiter

---

### new

**Source**: `utils/rate_limiter.rs`

```rust
pub fn new( initial_tokens_per_second: u32, burst_capacity: u32, adjustment_interval: Duration, ) -> Self
```

Create a new adaptive rate limiter

---

### new

**Source**: `utils/batch.rs`

```rust
pub fn new(max_size: usize, max_age: Duration) -> Self
```

Create a new batch processor

---

### new

**Source**: `utils/consistent_hash.rs`

```rust
pub fn new(virtual_nodes: usize) -> Self
```

Create a new consistent hash ring

---

### new

**Source**: `utils/health.rs`

```rust
pub fn new(cache_ttl: Duration) -> Self
```

Create a new health check coordinator

---

### new

**Source**: `utils/health.rs`

```rust
pub fn new<F, E>(name: String, test_fn: F) -> Self where F: Fn() -> Result<(), E> + Send + Sync + 'static, E: std::error::Error + Send + Sync + 'static,
```

Create a new connection health check

---

### new

**Source**: `utils/health.rs`

```rust
pub fn new(name: String, url: String, timeout: Duration, expected_status: u16) -> Self
```

Create a new HTTP health check

---

### next_batch

**Source**: `utils/batch.rs`

```rust
pub async fn next_batch(&self) -> Option<Vec<T>>
```

Get the next batch (blocks until available)

---

### node_count

**Source**: `utils/consistent_hash.rs`

```rust
pub fn node_count(&self) -> usize
```

Get number of active nodes

---

### node_id

**Source**: `config/mod.rs`

```rust
pub fn node_id(&self) -> String
```

Generate node ID if not provided

---

### oldest_item_age

**Source**: `utils/batch.rs`

```rust
pub async fn oldest_item_age(&self) -> Option<Duration>
```

Get the age of the oldest item in the batch

---

### process

**Source**: `core/processor.rs`

```rust
pub async fn process(&self, mut event: Box<dyn Event>) -> IndexerResult<Box<dyn Event>>
```

Process an event through all stages

---

### queue_depth

**Source**: `core/ingester.rs`

```rust
pub async fn queue_depth(&self) -> usize
```

Get current queue depth

---

### receive_events

**Source**: `core/ingester.rs`

```rust
pub async fn receive_events(&self) -> IndexerResult<Vec<Box<dyn Event>>>
```

Receive events from the ingestion queue

---

### record_counter

**Source**: `metrics/collector.rs`

```rust
pub fn record_counter(&self, name: &str, value: u64)
```

Record a counter value

---

### record_error

**Source**: `core/mod.rs`

```rust
pub fn record_error(&mut self, message: &str)
```

Record an error

---

### record_gauge

**Source**: `metrics/collector.rs`

```rust
pub fn record_gauge(&self, name: &str, value: f64)
```

Record a gauge value

---

### record_histogram

**Source**: `metrics/collector.rs`

```rust
pub fn record_histogram(&self, name: &str, value: f64)
```

Record a histogram value

---

### record_histogram

**Source**: `metrics/mod.rs`

```rust
pub fn record_histogram(&self, name: &str, value: f64) -> IndexerResult<()>
```

Record histogram value

---

### record_success

**Source**: `core/mod.rs`

```rust
pub fn record_success(&mut self)
```

Record a successful operation

---

### register

**Source**: `utils/health.rs`

```rust
pub async fn register(&self, name: String, check: Arc<dyn HealthCheck + 'static>)
```

Register a health check

---

### register_metric

**Source**: `metrics/mod.rs`

```rust
pub fn register_metric(&self, metric: Metric) -> IndexerResult<()>
```

Register a metric

---

### remove_node

**Source**: `utils/consistent_hash.rs`

```rust
pub fn remove_node(&mut self, node_id: &str) -> bool
```

Remove a node from the hash ring

---

### request_shutdown

**Source**: `core/mod.rs`

```rust
pub fn request_shutdown(&self) -> IndexerResult<()>
```

Request shutdown

---

### reset

**Source**: `metrics/collector.rs`

```rust
pub fn reset(&self)
```

Reset all metrics

---

### retry_with_backoff

**Source**: `utils/mod.rs`

```rust
pub async fn retry_with_backoff<F, T, E>( mut operation: F, max_attempts: u32, initial_delay: Duration, max_delay: Duration, multiplier: f64, ) -> Result<T, E> where F: FnMut() -> Result<T, E>, E: std::fmt::Display,
```

Retry helper with exponential backoff

---

### set_gauge

**Source**: `metrics/mod.rs`

```rust
pub fn set_gauge(&self, name: &str, value: f64) -> IndexerResult<()>
```

Update gauge metric

---

### set_state

**Source**: `core/mod.rs`

```rust
pub async fn set_state(&self, state: ServiceState)
```

Update service state

---

### shutdown_all

**Source**: `core/mod.rs`

```rust
pub async fn shutdown_all(&mut self) -> IndexerResult<()>
```

Shutdown all services gracefully

---

### shutdown_receiver

**Source**: `core/mod.rs`

```rust
pub fn shutdown_receiver(&self) -> broadcast::Receiver<()>
```

Subscribe to shutdown signals

---

### start

**Source**: `api/mod.rs`

```rust
pub async fn start(&self) -> IndexerResult<()>
```

Start the API server

---

### start_age_flusher

**Source**: `utils/batch.rs`

```rust
pub fn start_age_flusher(&self)
```

Start the age-based flushing task

---

### start_background_checks

**Source**: `utils/health.rs`

```rust
pub fn start_background_checks(&self, interval: Duration) -> tokio::task::JoinHandle<()>
```

Start background health check task

---

### start_collection_task

**Source**: `metrics/collector.rs`

```rust
pub fn start_collection_task(&self) -> tokio::task::JoinHandle<()>
```

Start metrics collection background task

---

### start_metrics_server

**Source**: `metrics/prometheus.rs`

```rust
pub async fn start_metrics_server( bind_addr: &str, port: u16, collector: Arc<MetricsCollector>, ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
```

Start Prometheus metrics server

---

### state

**Source**: `core/mod.rs`

```rust
pub async fn state(&self) -> ServiceState
```

Get current service state

---

### stats

**Source**: `core/ingester.rs`

```rust
pub async fn stats(&self) -> IngestionStats
```

Get ingestion statistics

---

### stats

**Source**: `utils/rate_limiter.rs`

```rust
pub async fn stats(&self) -> RateLimiterStats
```

Get rate limiter statistics

---

### subscribe

**Source**: `api/websocket.rs`

```rust
pub fn subscribe(&self) -> broadcast::Receiver<StreamMessage>
```

Get receiver for messages

---

### success

**Source**: `api/rest.rs`

```rust
pub fn success(data: T) -> Self
```

Create a successful response

---

### try_next_batch

**Source**: `utils/batch.rs`

```rust
pub async fn try_next_batch(&self) -> Option<Vec<T>>
```

Try to get the next batch (non-blocking)

---

### unhealthy

**Source**: `core/mod.rs`

```rust
pub fn unhealthy(message: &str) -> Self
```

Create an unhealthy component status

---

### unhealthy

**Source**: `utils/health.rs`

```rust
pub fn unhealthy(message: &str) -> Self
```

Create an unhealthy result

---

### unlimited

**Source**: `utils/rate_limiter.rs`

```rust
pub fn unlimited() -> Self
```

Create an unlimited rate limiter (no rate limiting)

---

### unregister

**Source**: `utils/health.rs`

```rust
pub async fn unregister(&self, name: &str) -> bool
```

Remove a health check

---

### update_component_health

**Source**: `core/mod.rs`

```rust
pub async fn update_component_health(&self, component: &str, health: ComponentHealth)
```

Update component health

---

### update_indexer_metrics

**Source**: `metrics/collector.rs`

```rust
pub async fn update_indexer_metrics(&self, metrics: IndexerMetrics)
```

Update indexer-specific metrics

---

### validate

**Source**: `config/mod.rs`

```rust
pub fn validate(&self) -> IndexerResult<()>
```

Validate the configuration

---

### validate_event_id

**Source**: `utils/mod.rs`

```rust
pub fn validate_event_id(id: &str) -> IndexerResult<()>
```

Validate event ID format

---

### validate_event_type

**Source**: `utils/mod.rs`

```rust
pub fn validate_event_type(event_type: &str) -> IndexerResult<()>
```

Validate event type format

---

### validate_source

**Source**: `utils/mod.rs`

```rust
pub fn validate_source(source: &str) -> IndexerResult<()>
```

Validate source format

---

### validation

**Source**: `src/error.rs`

```rust
pub fn validation(message: impl Into<String>) -> Self
```

Create a validation error

---

### virtual_nodes

**Source**: `utils/consistent_hash.rs`

```rust
pub fn virtual_nodes(mut self, count: usize) -> Self
```

Set number of virtual nodes per physical node

---

### wait_for_tokens

**Source**: `utils/rate_limiter.rs`

```rust
pub async fn wait_for_tokens(&self, tokens_required: u32) -> Result<(), RateLimitError>
```

Wait until tokens are available, then consume them

---

### websocket_handler

**Source**: `api/websocket.rs`

```rust
pub async fn websocket_handler( ws: WebSocketUpgrade, State(context): State<Arc<ServiceContext>>, ) -> Response
```

WebSocket upgrade handler

---

### with_detail

**Source**: `utils/health.rs`

```rust
pub fn with_detail(mut self, key: &str, value: &str) -> Self
```

Add detail information

---

### with_response_time

**Source**: `utils/health.rs`

```rust
pub fn with_response_time(mut self, duration: Duration) -> Self
```

Set response time

---

## Structs

### AdaptiveRateLimiter

**Source**: `utils/rate_limiter.rs`

```rust
pub struct AdaptiveRateLimiter { base_limiter: RateLimiter, success_count: Arc<Mutex<u32>>, failure_count: Arc<Mutex<u32>>, last_adjustment: Arc<Mutex<Instant>>, adjustment_interval: Duration, }
```

Adaptive rate limiter that adjusts based on success/failure rates

---

### ApiConfig

**Source**: `config/mod.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Deserialize, Serialize)]
```

```rust
pub struct ApiConfig { /// HTTP server settings pub http: HttpConfig, /// WebSocket settings pub websocket: WebSocketConfig, /// GraphQL settings pub graphql: Option<GraphQLConfig>, /// Authentication settings pub auth: AuthConfig, /// CORS settings pub cors: CorsConfig, }
```

API server configuration

---

### ApiKeyConfig

**Source**: `config/mod.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Deserialize, Serialize)]
```

```rust
pub struct ApiKeyConfig { /// Header name for API key pub header_name: String, /// Valid API keys pub keys: Vec<String>, }
```

API key configuration

---

### ApiResponse

**Source**: `api/rest.rs`

**Attributes**:
```rust
#[derive(Serialize)]
```

```rust
pub struct ApiResponse<T> { /// Whether the request was successful pub success: bool, /// Response data if successful pub data: Option<T>, /// Error message if unsuccessful pub error: Option<String>, /// Response timestamp pub timestamp: DateTime<Utc>, }
```

API response wrapper

---

### ApiServer

**Source**: `api/mod.rs`

```rust
pub struct ApiServer { /// Service context context: Arc<ServiceContext>, /// Configuration config: ApiConfig, /// REST handler rest_handler: RestHandler, /// WebSocket streamer websocket_streamer: Option<WebSocketStreamer>, }
```

Main API server

---

### ArchiveConfig

**Source**: `config/mod.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Deserialize, Serialize)]
```

```rust
pub struct ArchiveConfig { /// Whether archival is enabled pub enabled: bool, /// Archive storage backend pub backend: Option<StorageBackendConfig>, /// Compression settings pub compression: CompressionConfig, }
```

Archive configuration

---

### AuthConfig

**Source**: `config/mod.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Deserialize, Serialize)]
```

```rust
pub struct AuthConfig { /// Whether authentication is enabled pub enabled: bool, /// Authentication method pub method: AuthMethod, /// JWT settings (if using JWT)
```

Authentication configuration

---

### BatchConfig

**Source**: `config/mod.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Deserialize, Serialize)]
```

```rust
pub struct BatchConfig { /// Maximum batch size pub max_size: usize, /// Maximum batch age before forcing flush #[serde(with = "humantime_serde")]
```

Batch processing configuration

---

### BatchProcessor

**Source**: `utils/batch.rs`

```rust
pub struct BatchProcessor<T> { /// Maximum batch size max_size: usize, /// Maximum age before forcing a batch max_age: Duration, /// Current batch current_batch: Arc<RwLock<VecDeque<(T, Instant)>>>,
```

Batch processor for collecting items and processing them in batches

---

### CacheConfig

**Source**: `config/mod.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Deserialize, Serialize)]
```

```rust
pub struct CacheConfig { /// Cache backend (redis, memory)
```

Cache configuration

---

### CacheTtlConfig

**Source**: `config/mod.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Deserialize, Serialize)]
```

```rust
pub struct CacheTtlConfig { /// Default TTL for cached items #[serde(with = "humantime_serde")]
```

Cache TTL configuration

---

### ComponentHealth

**Source**: `core/mod.rs`

**Attributes**:
```rust
#[derive(Debug, Clone)]
```

```rust
pub struct ComponentHealth { /// Component is healthy pub healthy: bool, /// Status message pub message: String, /// Last successful operation timestamp pub last_success: Option<chrono::DateTime<chrono::Utc>>, /// Error count since last success pub error_count: u64, }
```

Component health information

---

### ComponentStatus

**Source**: `api/rest.rs`

**Attributes**:
```rust
#[derive(Serialize)]
```

```rust
pub struct ComponentStatus { /// Whether the component is healthy pub healthy: bool, /// Status message describing component state pub message: String, /// Timestamp of last successful operation pub last_success: Option<DateTime<Utc>>, }
```

Component status

---

### CompressionConfig

**Source**: `config/mod.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Deserialize, Serialize)]
```

```rust
pub struct CompressionConfig { /// Compression algorithm pub algorithm: CompressionAlgorithm, /// Compression level (1-9)
```

Compression configuration

---

### ConnectionHealthCheck

**Source**: `utils/health.rs`

```rust
pub struct ConnectionHealthCheck { name: String, test_fn: Arc<dyn Fn() -> Result<(), Box<dyn std::error::Error + Send + Sync>> + Send + Sync>,
```

Simple health check implementation for testing connections

---

### ConnectionPoolConfig

**Source**: `config/mod.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Deserialize, Serialize)]
```

```rust
pub struct ConnectionPoolConfig { /// Maximum number of connections pub max_connections: u32, /// Minimum number of idle connections pub min_connections: u32, /// Connection timeout #[serde(with = "humantime_serde")]
```

Connection pool configuration

---

### ConsistentHash

**Source**: `utils/consistent_hash.rs`

```rust
pub struct ConsistentHash { /// Virtual nodes on the hash ring ring: BTreeMap<u64, String>, /// Number of virtual nodes per physical node virtual_nodes: usize, /// Active nodes nodes: HashMap<String, NodeInfo>, }
```

Consistent hash ring for distributing work across nodes

---

### ConsistentHashBuilder

**Source**: `utils/consistent_hash.rs`

```rust
pub struct ConsistentHashBuilder { virtual_nodes: usize, nodes: Vec<(String, u32, HashMap<String, String>)>,
```

Builder for consistent hash ring

---

### CorsConfig

**Source**: `config/mod.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Deserialize, Serialize)]
```

```rust
pub struct CorsConfig { /// Whether CORS is enabled pub enabled: bool, /// Allowed origins pub allowed_origins: Vec<String>, /// Allowed methods pub allowed_methods: Vec<String>, /// Allowed headers pub allowed_headers: Vec<String>, /// Max age for preflight requests #[serde(with = "humantime_serde")]
```

CORS configuration

---

### CustomMetricConfig

**Source**: `config/mod.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Deserialize, Serialize)]
```

```rust
pub struct CustomMetricConfig { /// Metric type pub metric_type: MetricType, /// Metric description pub description: String, /// Metric labels pub labels: Vec<String>, }
```

Custom metric configuration

---

### DiskQueueConfig

**Source**: `config/mod.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Deserialize, Serialize)]
```

```rust
pub struct DiskQueueConfig { /// Directory for queue files pub directory: PathBuf, /// Maximum size per file pub max_file_size: u64, /// Sync strategy pub sync_strategy: SyncStrategy, }
```

Disk queue configuration

---

### EnrichmentStage

**Source**: `core/pipeline.rs`

```rust
pub struct EnrichmentStage { name: String, }
```

Enrichment pipeline stage

---

### ErrorMetrics

**Source**: `metrics/mod.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Default)]
```

```rust
pub struct ErrorMetrics { /// Total error count pub total_errors: u64, /// Error rate (errors per total events)
```

Error metrics

---

### EventAggregator

**Source**: `storage/mod.rs`

```rust
pub struct EventAggregator;
```

Event aggregation helpers

---

### EventFilter

**Source**: `storage/mod.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Default)]
```

```rust
pub struct EventFilter { /// Filter by event types pub event_types: Option<Vec<String>>, /// Filter by sources pub sources: Option<Vec<String>>, /// Filter by time range pub time_range: Option<(DateTime<Utc>, DateTime<Utc>)>,
```

Query filter for retrieving events

---

### EventIngester

**Source**: `core/ingester.rs`

**Attributes**:
```rust
#[derive(Clone)]
```

```rust
pub struct EventIngester { /// Ingester configuration config: IngesterConfig, /// Shared service context context: Arc<ServiceContext>, /// Event queue sender event_tx: Arc<RwLock<Option<EventSender>>>, /// Event queue receiver for external access event_rx: Arc<RwLock<Option<EventReceiver>>>, /// Stream manager for handling multiple data sources stream_manager: Arc<RwLock<Option<StreamManager>>>, /// Rate limiter for ingestion rate_limiter: Arc<RateLimiter>, /// Ingestion statistics stats: Arc<RwLock<IngestionStats>>, }
```

Event ingester that collects events from multiple sources

---

### EventProcessor

**Source**: `core/processor.rs`

**Attributes**:
```rust
#[derive(Clone)]
```

```rust
pub struct EventProcessor { /// Processor configuration config: ProcessorConfig, /// Shared service context context: Arc<ServiceContext>, /// Worker pool semaphore for limiting concurrent operations worker_semaphore: Arc<Semaphore>, /// Processing statistics stats: Arc<RwLock<ProcessorStats>>, /// Active workers counter active_workers: Arc<RwLock<usize>>, /// Batch processor #[allow(dead_code)]
```

Event processor that handles parallel processing with worker pools

---

### EventQuery

**Source**: `storage/mod.rs`

**Attributes**:
```rust
#[derive(Debug, Clone)]
```

```rust
pub struct EventQuery { /// Filter criteria pub filter: EventFilter, /// Number of results to return (None = no limit)
```

Query parameters for event retrieval

---

### EventStatsResponse

**Source**: `api/rest.rs`

**Attributes**:
```rust
#[derive(Serialize)]
```

```rust
pub struct EventStatsResponse { /// Total number of events indexed pub total_events: u64, /// Event counts grouped by type pub events_by_type: HashMap<String, u64>, /// Event counts grouped by source pub events_by_source: HashMap<String, u64>, /// Storage layer statistics pub storage_stats: crate::storage::StorageStats, }
```

Event statistics response

---

### EventSubscriptionFilter

**Source**: `api/websocket.rs`

**Attributes**:
```rust
#[derive(Debug, Deserialize)]
```

```rust
pub struct EventSubscriptionFilter { /// Event types to include pub event_types: Option<Vec<String>>, /// Sources to include pub sources: Option<Vec<String>>, /// Minimum block height pub min_block: Option<u64>, }
```

Event subscription filters

---

### EventsQueryParams

**Source**: `api/rest.rs`

**Attributes**:
```rust
#[derive(Debug, Deserialize)]
```

```rust
pub struct EventsQueryParams { /// Event types to filter by pub event_types: Option<String>, // Comma-separated /// Sources to filter by pub sources: Option<String>, // Comma-separated /// Start time for filtering pub start_time: Option<DateTime<Utc>>, /// End time for filtering pub end_time: Option<DateTime<Utc>>, /// Minimum block height pub min_block: Option<u64>, /// Maximum block height pub max_block: Option<u64>, /// Transaction hash pub tx_hash: Option<String>, /// Number of results to return pub limit: Option<usize>, /// Number of results to skip pub offset: Option<usize>, /// Sort field pub sort_by: Option<String>, /// Sort direction (asc/desc)
```

Query parameters for events endpoint

---

### FeatureConfig

**Source**: `config/mod.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Deserialize, Serialize)]
```

```rust
pub struct FeatureConfig { /// Enable real-time streaming pub realtime_streaming: bool, /// Enable event archival pub archival: bool, /// Enable GraphQL API pub graphql_api: bool, /// Enable experimental features pub experimental: bool, /// Custom feature flags pub custom: HashMap<String, bool>, }
```

Feature flags configuration

---

### GraphQLConfig

**Source**: `config/mod.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Deserialize, Serialize)]
```

```rust
pub struct GraphQLConfig { /// Whether GraphQL is enabled pub enabled: bool, /// GraphQL endpoint path pub endpoint: String, /// Playground enabled pub playground: bool, /// Query complexity limit pub complexity_limit: u32, /// Query depth limit pub depth_limit: u32, }
```

GraphQL configuration

---

### HealthCheckCoordinator

**Source**: `utils/health.rs`

```rust
pub struct HealthCheckCoordinator { /// Registered health checks checks: Arc<DashMap<String, Arc<dyn HealthCheck + 'static>>>, /// Cached results cached_results: Arc<DashMap<String, (HealthCheckResult, Instant)>>,
```

Health check coordinator that manages multiple health checks

---

### HealthCheckResult

**Source**: `utils/health.rs`

**Attributes**:
```rust
#[derive(Debug, Clone)]
```

```rust
pub struct HealthCheckResult { /// Whether the component is healthy pub healthy: bool, /// Status message pub message: String, /// Additional details pub details: HashMap<String, String>, /// Response time for the check pub response_time: Duration, /// Timestamp when check was performed pub timestamp: Instant, }
```

Result of a health check

---

### HealthResponse

**Source**: `api/rest.rs`

**Attributes**:
```rust
#[derive(Serialize)]
```

```rust
pub struct HealthResponse { /// Overall health status (healthy/unhealthy/degraded)
```

Service health response

---

### HealthStatus

**Source**: `core/mod.rs`

**Attributes**:
```rust
#[derive(Debug, Clone)]
```

```rust
pub struct HealthStatus { /// Overall health status pub healthy: bool, /// Individual component statuses pub components: std::collections::HashMap<String, ComponentHealth>, /// Last check timestamp pub timestamp: chrono::DateTime<chrono::Utc>, }
```

Health check status

---

### HealthSummary

**Source**: `utils/health.rs`

**Attributes**:
```rust
#[derive(Debug, Clone)]
```

```rust
pub struct HealthSummary { /// Overall system health status pub overall_healthy: bool, /// Human-readable status description pub status: String, /// Total number of components checked pub total_components: usize, /// Number of healthy components pub healthy_components: usize, /// Number of unhealthy components pub unhealthy_components: usize, /// Individual component health check results pub component_results: HashMap<String, HealthCheckResult>, /// Health check timestamp pub timestamp: Instant, }
```

Overall health summary

---

### HttpConfig

**Source**: `config/mod.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Deserialize, Serialize)]
```

```rust
pub struct HttpConfig { /// Server bind address pub bind: String, /// Server port pub port: u16, /// Request timeout #[serde(with = "humantime_serde")]
```

HTTP server configuration

---

### HttpHealthCheck

**Source**: `utils/health.rs`

```rust
pub struct HttpHealthCheck { name: String, url: String, timeout: Duration, expected_status: u16, }
```

HTTP endpoint health check

---

### IndexerConfig

**Source**: `config/mod.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Deserialize, Serialize)]
```

```rust
pub struct IndexerConfig { /// Service configuration pub service: ServiceConfig, /// Storage configuration pub storage: StorageConfig, /// Processing configuration pub processing: ProcessingConfig, /// API server configuration pub api: ApiConfig, /// Metrics configuration pub metrics: MetricsConfig, /// Logging configuration pub logging: LoggingConfig, /// Feature flags pub features: FeatureConfig, }
```

Main configuration for the indexer service

---

### IndexerMetrics

**Source**: `metrics/mod.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Default)]
```

```rust
pub struct IndexerMetrics { /// Total events processed pub events_processed_total: u64, /// Events processing rate (events/second)
```

Indexer-specific metrics

---

### IndexerService

**Source**: `core/indexer.rs`

```rust
pub struct IndexerService { /// Service context with shared state context: Arc<ServiceContext>, /// Event ingester for data collection ingester: Option<EventIngester>, /// Event processor for parallel processing processor: Option<EventProcessor>, /// Processing statistics stats: Arc<RwLock<ProcessingStats>>, /// Health check task handle health_task: Option<tokio::task::JoinHandle<()>>,
```

Main indexer service that orchestrates all components

---

### IngesterConfig

**Source**: `core/ingester.rs`

**Attributes**:
```rust
#[derive(Debug, Clone)]
```

```rust
pub struct IngesterConfig { /// Number of ingestion workers pub workers: usize, /// Batch size for processing pub batch_size: usize, /// Queue capacity pub queue_capacity: usize, }
```

Configuration for the event ingester

---

### IngestionStats

**Source**: `core/ingester.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Default)]
```

```rust
pub struct IngestionStats { /// Total events ingested pub total_events: u64, /// Events per second pub events_per_second: f64, /// Queue depth pub queue_depth: usize, /// Active streams pub active_streams: usize, /// Error count pub error_count: u64, /// Last successful ingestion pub last_success: Option<chrono::DateTime<chrono::Utc>>, }
```

Ingestion statistics

---

### JwtConfig

**Source**: `config/mod.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Deserialize, Serialize)]
```

```rust
pub struct JwtConfig { /// JWT secret key pub secret: String, /// Token expiration time #[serde(with = "humantime_serde")]
```

JWT configuration

---

### LatencyMetrics

**Source**: `metrics/mod.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Default)]
```

```rust
pub struct LatencyMetrics { /// Average latency in milliseconds pub avg_latency_ms: f64, /// 50th percentile latency pub p50_latency_ms: f64, /// 95th percentile latency pub p95_latency_ms: f64, /// 99th percentile latency pub p99_latency_ms: f64, /// Maximum latency pub max_latency_ms: f64, }
```

Latency metrics

---

### LoggingConfig

**Source**: `config/mod.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Deserialize, Serialize)]
```

```rust
pub struct LoggingConfig { /// Log level pub level: String, /// Log format (json, text)
```

Logging configuration

---

### MemoryCache

**Source**: `storage/cache.rs`

```rust
pub struct MemoryCache { // Implementation would go here }
```

In-memory cache implementation

---

### MemoryCacheConfig

**Source**: `config/mod.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Deserialize, Serialize)]
```

```rust
pub struct MemoryCacheConfig { /// Maximum memory usage in bytes pub max_size_bytes: u64, /// Maximum number of entries pub max_entries: u64, }
```

Memory cache settings

---

### Metric

**Source**: `metrics/mod.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize)]
```

```rust
pub struct Metric { /// Metric name pub name: String, /// Type of metric pub metric_type: MetricType, /// Current metric value pub value: MetricValue, /// Metric labels for dimensional data pub labels: HashMap<String, String>, /// Timestamp when metric was recorded pub timestamp: DateTime<Utc>, /// Help text describing the metric pub help: String, }
```

Individual metric data

---

### MetricsCollector

**Source**: `metrics/collector.rs`

```rust
pub struct MetricsCollector { /// Metrics registry registry: Arc<MetricsRegistry>, /// Configuration config: MetricsConfig, /// Current indexer metrics snapshot current_metrics: Arc<RwLock<IndexerMetrics>>, /// Collection start time start_time: std::time::Instant, }
```

Main metrics collector

---

### MetricsConfig

**Source**: `config/mod.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Deserialize, Serialize)]
```

```rust
pub struct MetricsConfig { /// Whether metrics are enabled pub enabled: bool, /// Metrics server port pub port: u16, /// Metrics endpoint path pub endpoint: String, /// Collection interval #[serde(with = "humantime_serde")]
```

Metrics configuration

---

### MetricsRegistry

**Source**: `metrics/mod.rs`

```rust
pub struct MetricsRegistry { /// Stored metrics metrics: Arc<DashMap<String, Metric>>, /// Configuration #[allow(dead_code)]
```

Metrics registry for storing and managing metrics

---

### NodeInfo

**Source**: `utils/consistent_hash.rs`

**Attributes**:
```rust
#[derive(Debug, Clone)]
```

```rust
pub struct NodeInfo { /// Unique node identifier pub id: String, /// Node weight for virtual node allocation pub weight: u32, /// Additional node metadata pub metadata: HashMap<String, String>, }
```

Information about a node

---

### PerformanceMetrics

**Source**: `metrics/mod.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Default)]
```

```rust
pub struct PerformanceMetrics { /// Throughput metrics pub throughput: ThroughputMetrics, /// Latency metrics pub latency: LatencyMetrics, /// Resource utilization metrics pub resources: ResourceMetrics, /// Error metrics pub errors: ErrorMetrics, }
```

Performance metrics summary

---

### PostgresStore

**Source**: `storage/postgres.rs`

```rust
pub struct PostgresStore { /// Database connection pool pool: PgPool, /// Configuration config: StorageBackendConfig, /// Statistics tracking stats: std::sync::RwLock<InternalStats>, }
```

PostgreSQL storage implementation

---

### ProcessingConfig

**Source**: `config/mod.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Deserialize, Serialize)]
```

```rust
pub struct ProcessingConfig { /// Number of worker threads pub workers: usize, /// Batch processing settings pub batch: BatchConfig, /// Event queue configuration pub queue: QueueConfig, /// Retry configuration pub retry: RetryConfig, /// Rate limiting pub rate_limit: RateLimitConfig, }
```

Processing configuration

---

### ProcessingPipeline

**Source**: `core/processor.rs`

```rust
pub struct ProcessingPipeline { stages: Vec<Box<dyn PipelineStage + Send + Sync>>, }
```

Processing pipeline stage for complex event processing

---

### ProcessingStats

**Source**: `core/mod.rs`

**Attributes**:
```rust
#[derive(Debug, Clone)]
```

```rust
pub struct ProcessingStats { /// Total events processed pub total_processed: u64, /// Events processed per second (current rate)
```

Processing statistics

---

### ProcessorConfig

**Source**: `core/processor.rs`

**Attributes**:
```rust
#[derive(Debug, Clone)]
```

```rust
pub struct ProcessorConfig { /// Number of worker threads pub workers: usize, /// Batch processing configuration pub batch_config: BatchConfig, /// Retry configuration pub retry_config: RetryConfig, }
```

Configuration for the event processor

---

### ProcessorStats

**Source**: `core/processor.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Default)]
```

```rust
pub struct ProcessorStats { /// Total events processed total_processed: u64, /// Total errors encountered total_errors: u64, /// Events processed per second events_per_second: f64, /// Average processing latency in milliseconds avg_latency_ms: f64, /// Current batch size #[allow(dead_code)]
```

Internal processor statistics

---

### PrometheusState

**Source**: `metrics/prometheus.rs`

```rust
pub struct PrometheusState { /// Metrics collector instance pub collector: Arc<MetricsCollector>, }
```

Prometheus metrics server state

---

### QueueConfig

**Source**: `config/mod.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Deserialize, Serialize)]
```

```rust
pub struct QueueConfig { /// Maximum queue capacity pub capacity: usize, /// Queue type (memory, disk, hybrid)
```

Event queue configuration

---

### RateLimitConfig

**Source**: `config/mod.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Deserialize, Serialize)]
```

```rust
pub struct RateLimitConfig { /// Whether rate limiting is enabled pub enabled: bool, /// Maximum events per second pub max_events_per_second: u32, /// Burst capacity pub burst_capacity: u32, }
```

Rate limiting configuration

---

### RateLimiter

**Source**: `utils/rate_limiter.rs`

```rust
pub struct RateLimiter { /// Configuration config: RateLimiterConfig, /// Current state state: Arc<Mutex<RateLimiterState>>, }
```

Token bucket rate limiter

---

### RateLimiterConfig

**Source**: `utils/rate_limiter.rs`

**Attributes**:
```rust
#[derive(Debug, Clone)]
```

```rust
pub struct RateLimiterConfig { /// Maximum tokens per second pub tokens_per_second: u32, /// Burst capacity (maximum tokens in bucket)
```

Rate limiter configuration

---

### RateLimiterStats

**Source**: `utils/rate_limiter.rs`

**Attributes**:
```rust
#[derive(Debug, Clone)]
```

```rust
pub struct RateLimiterStats { /// Whether rate limiting is enabled pub enabled: bool, /// Token generation rate per second pub tokens_per_second: u32, /// Maximum burst capacity pub burst_capacity: u32, /// Current available tokens pub current_tokens: f64, /// Last token refill timestamp pub last_refill: Instant, }
```

Rate limiter statistics

---

### RedisCache

**Source**: `storage/cache.rs`

```rust
pub struct RedisCache { // Implementation would go here }
```

Redis-based cache implementation

---

### ResourceMetrics

**Source**: `metrics/mod.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Default)]
```

```rust
pub struct ResourceMetrics { /// CPU usage percentage pub cpu_usage_percent: f64, /// Memory usage in bytes pub memory_usage_bytes: u64, /// Memory usage percentage pub memory_usage_percent: f64, /// Active database connections pub db_connections_active: u32, /// Maximum database connections pub db_connections_max: u32, }
```

Resource utilization metrics

---

### RestHandler

**Source**: `api/rest.rs`

```rust
pub struct RestHandler { context: Arc<ServiceContext>, }
```

REST API handler

---

### RetentionConfig

**Source**: `config/mod.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Deserialize, Serialize)]
```

```rust
pub struct RetentionConfig { /// Default retention period #[serde(with = "humantime_serde")]
```

Data retention configuration

---

### RetryConfig

**Source**: `config/mod.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Deserialize, Serialize)]
```

```rust
pub struct RetryConfig { /// Maximum retry attempts pub max_attempts: u32, /// Base delay between retries #[serde(with = "humantime_serde")]
```

Retry configuration

---

### ServiceConfig

**Source**: `config/mod.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Deserialize, Serialize)]
```

```rust
pub struct ServiceConfig { /// Service name for identification pub name: String, /// Service version pub version: String, /// Environment (development, staging, production)
```

Service-level configuration

---

### ServiceContext

**Source**: `core/mod.rs`

```rust
pub struct ServiceContext { /// Service configuration pub config: Arc<IndexerConfig>, /// Service state pub state: Arc<RwLock<ServiceState>>, /// Data store pub store: Arc<dyn DataStore>, /// Metrics collector pub metrics: Arc<MetricsCollector>, /// Shutdown signal broadcaster pub shutdown_tx: broadcast::Sender<()>,
```

Shared service context

---

### ShutdownCoordinator

**Source**: `core/mod.rs`

```rust
pub struct ShutdownCoordinator { /// Timeout for graceful shutdown timeout: std::time::Duration, /// Services to shutdown services: Vec<Box<dyn ServiceLifecycle + Send + Sync>>, }
```

Graceful shutdown coordinator

---

### StorageBackendConfig

**Source**: `config/mod.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Deserialize, Serialize)]
```

```rust
pub struct StorageBackendConfig { /// Backend type (postgres, clickhouse, etc.)
```

Storage backend configuration

---

### StorageConfig

**Source**: `config/mod.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Deserialize, Serialize)]
```

```rust
pub struct StorageConfig { /// Primary storage backend pub primary: StorageBackendConfig, /// Optional secondary storage for archival pub secondary: Option<StorageBackendConfig>, /// Cache configuration pub cache: CacheConfig, /// Data retention policies pub retention: RetentionConfig, }
```

Storage configuration

---

### StorageStats

**Source**: `storage/mod.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize)]
```

```rust
pub struct StorageStats { /// Total number of events stored pub total_events: u64, /// Storage size in bytes pub storage_size_bytes: u64, /// Average write latency in milliseconds pub avg_write_latency_ms: f64, /// Average read latency in milliseconds pub avg_read_latency_ms: f64, /// Number of active connections pub active_connections: u32, /// Cache hit rate (0.0 to 1.0)
```

Storage statistics

---

### StoredEvent

**Source**: `storage/mod.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct StoredEvent { /// Unique event identifier pub id: String, /// Event type/category pub event_type: String, /// Event source (chain, protocol, etc.)
```

Stored event data structure

---

### StructuredLoggingConfig

**Source**: `config/mod.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Deserialize, Serialize)]
```

```rust
pub struct StructuredLoggingConfig { /// Include source location pub include_location: bool, /// Include thread information pub include_thread: bool, /// Include service metadata pub include_service_metadata: bool, /// Custom fields to include pub custom_fields: HashMap<String, String>, }
```

Structured logging configuration

---

### SubscriptionRequest

**Source**: `api/websocket.rs`

**Attributes**:
```rust
#[derive(Debug, Deserialize)]
```

```rust
pub struct SubscriptionRequest { /// Types of messages to subscribe to pub message_types: Vec<String>, /// Event filters for event messages pub event_filters: Option<EventSubscriptionFilter>, }
```

WebSocket subscription request

---

### SummaryValue

**Source**: `metrics/mod.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize)]
```

```rust
pub struct SummaryValue { /// Number of observations pub count: u64, /// Sum of all observations pub sum: f64, /// Quantile values (e.g., 0.5 -> median)
```

Summary metric value

---

### ThroughputMetrics

**Source**: `metrics/mod.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Default)]
```

```rust
pub struct ThroughputMetrics { /// Events per second (current)
```

Throughput metrics

---

### ValidationStage

**Source**: `core/pipeline.rs`

```rust
pub struct ValidationStage { name: String, }
```

Validation pipeline stage

---

### WebSocketConfig

**Source**: `config/mod.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Deserialize, Serialize)]
```

```rust
pub struct WebSocketConfig { /// Whether WebSocket is enabled pub enabled: bool, /// Maximum connections pub max_connections: u32, /// Message buffer size pub buffer_size: usize, /// Heartbeat interval #[serde(with = "humantime_serde")]
```

WebSocket configuration

---

### WebSocketStreamer

**Source**: `api/websocket.rs`

```rust
pub struct WebSocketStreamer { #[allow(dead_code)]
```

WebSocket streaming handler

---

## Traits

### Cache

**Source**: `storage/cache.rs`

**Attributes**:
```rust
#[async_trait]
```

```rust
pub trait Cache: Send + Sync { ... }
```

Cache trait for storing and retrieving events

**Methods**:

#### `set`

```rust
async fn set(&self, key: &str, event: &StoredEvent, ttl: Option<Duration>) -> IndexerResult<()>;
```

#### `get`

```rust
async fn get(&self, key: &str) -> IndexerResult<Option<StoredEvent>>;
```

#### `delete`

```rust
async fn delete(&self, key: &str) -> IndexerResult<()>;
```

#### `health_check`

```rust
async fn health_check(&self) -> IndexerResult<()>;
```

#### `clear`

```rust
async fn clear(&self) -> IndexerResult<()>;
```

---

### DataStore

**Source**: `storage/mod.rs`

**Attributes**:
```rust
#[async_trait]
```

```rust
pub trait DataStore: Send + Sync { ... }
```

Main data store trait

**Methods**:

#### `insert_event`

```rust
async fn insert_event(&self, event: &StoredEvent) -> IndexerResult<()>;
```

#### `insert_events`

```rust
async fn insert_events(&self, events: &[StoredEvent]) -> IndexerResult<()>;
```

#### `query_events`

```rust
async fn query_events(&self, query: &EventQuery) -> IndexerResult<Vec<StoredEvent>>;
```

#### `get_event`

```rust
async fn get_event(&self, id: &str) -> IndexerResult<Option<StoredEvent>>;
```

#### `delete_events`

```rust
async fn delete_events(&self, filter: &EventFilter) -> IndexerResult<u64>;
```

#### `count_events`

```rust
async fn count_events(&self, filter: &EventFilter) -> IndexerResult<u64>;
```

#### `get_stats`

```rust
async fn get_stats(&self) -> IndexerResult<StorageStats>;
```

#### `health_check`

```rust
async fn health_check(&self) -> IndexerResult<()>;
```

#### `cache_event`

```rust
async fn cache_event( &self, key: &str, event: &StoredEvent, ttl: Option<Duration>, ) -> IndexerResult<()> {;
```

#### `get_cached_event`

```rust
async fn get_cached_event(&self, key: &str) -> IndexerResult<Option<StoredEvent>> {
```

#### `initialize`

```rust
async fn initialize(&self) -> IndexerResult<()>;
```

#### `maintenance`

```rust
async fn maintenance(&self) -> IndexerResult<()> {
```

---

### EventProcessing

**Source**: `core/mod.rs`

**Attributes**:
```rust
#[async_trait::async_trait]
```

```rust
pub trait EventProcessing { ... }
```

Event processing trait

**Methods**:

#### `process_event`

```rust
async fn process_event(&self, event: Box<dyn riglr_events_core::Event>) -> IndexerResult<()>;
```

#### `process_batch`

```rust
async fn process_batch( &self, events: Vec<Box<dyn riglr_events_core::Event>>, ) -> IndexerResult<()>;
```

#### `processing_stats`

```rust
async fn processing_stats(&self) -> ProcessingStats;
```

---

### HealthCheck

**Source**: `utils/health.rs`

**Attributes**:
```rust
#[async_trait::async_trait]
```

```rust
pub trait HealthCheck: Send + Sync { ... }
```

Health check trait for services and components

**Methods**:

#### `health_check`

```rust
async fn health_check( &self, ) -> Result<HealthCheckResult, Box<dyn std::error::Error + Send + Sync>>;
```

#### `component_name`

```rust
fn component_name(&self) -> &str;
```

---

### PipelineStage

**Source**: `core/processor.rs`

**Attributes**:
```rust
#[async_trait::async_trait]
```

```rust
pub trait PipelineStage { ... }
```

Pipeline stage trait for processing events

**Methods**:

#### `process`

```rust
async fn process(&self, event: Box<dyn Event>) -> IndexerResult<Box<dyn Event>>;
```

#### `name`

```rust
fn name(&self) -> &str;
```

---

### ServiceLifecycle

**Source**: `core/mod.rs`

**Attributes**:
```rust
#[async_trait::async_trait]
```

```rust
pub trait ServiceLifecycle { ... }
```

Service lifecycle trait

**Methods**:

#### `start`

```rust
async fn start(&mut self) -> IndexerResult<()>;
```

#### `stop`

```rust
async fn stop(&mut self) -> IndexerResult<()>;
```

#### `health`

```rust
async fn health(&self) -> IndexerResult<HealthStatus>;
```

#### `is_running`

```rust
fn is_running(&self) -> bool;
```

---

## Constants

### POSTGRES_SCHEMA

**Source**: `storage/schema.rs`

```rust
const POSTGRES_SCHEMA: &str
```

Database schema definitions and migrations
SQL schema for PostgreSQL

---

### VERSION_TABLE

**Source**: `storage/schema.rs`

```rust
const VERSION_TABLE: &str
```

Migration version tracking

---


---

*This documentation was automatically generated from the source code.*