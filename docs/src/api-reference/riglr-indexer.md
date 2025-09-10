# riglr-indexer

{{#include ../../../riglr-indexer/README.md}}

## API Reference

## Key Components

> The most important types and functions in this crate.

### `IndexerService`

Main indexer service that orchestrates all components

[→ Full documentation](#structs)

### `ProcessingPipeline`

Processing pipeline stage for complex event processing

[→ Full documentation](#structs)

### `DataStore`

Main data store trait

[→ Full documentation](#traits)

---

### Contents

- [Structs](#structs)
- [Enums](#enums)
- [Traits](#traits)
- [Functions](#functions)
- [Type Aliases](#type-aliases)
- [Constants](#constants)

### Structs

> Core data structures and types.

#### `ApiConfig`

API server configuration

---

#### `ApiKeyConfig`

API key configuration

---

#### `ApiResponse`

API response wrapper

---

#### `ApiServer`

Main API server

---

#### `ArchiveConfig`

Archive configuration

---

#### `AuthConfig`

Authentication configuration

---

#### `BatchConfig`

Batch processing configuration

---

#### `BatchProcessor`

Batch processor for collecting items and processing them in batches

---

#### `CacheConfig`

Cache configuration

---

#### `CacheTtlConfig`

Cache TTL configuration

---

#### `ComponentHealth`

Component health information

---

#### `ComponentStatus`

Component status

---

#### `CompressionConfig`

Compression configuration

---

#### `ConnectionHealthCheck`

Simple health check implementation for testing connections

---

#### `ConnectionPoolConfig`

Connection pool configuration

---

#### `ConsistentHash`

Consistent hash ring for distributing work across nodes

---

#### `ConsistentHashBuilder`

Builder for consistent hash ring

---

#### `CorsConfig`

CORS configuration

---

#### `CustomMetricConfig`

Custom metric configuration

---

#### `DiskQueueConfig`

Disk queue configuration

---

#### `EnrichmentStage`

Enrichment pipeline stage

---

#### `ErrorMetrics`

Error metrics

---

#### `EventAggregator`

Event aggregation helpers

---

#### `EventFilter`

Query filter for retrieving events

---

#### `EventIngester`

Event ingester that collects events from multiple sources

---

#### `EventProcessor`

Event processor that handles parallel processing with worker pools

---

#### `EventQuery`

Query parameters for event retrieval

---

#### `EventStatsResponse`

Event statistics response

---

#### `EventSubscriptionFilter`

Event subscription filters

---

#### `EventsQueryParams`

Query parameters for events endpoint

---

#### `FeatureConfig`

Feature flags configuration

---

#### `GraphQLConfig`

GraphQL configuration

---

#### `HealthCheckCoordinator`

Health check coordinator that manages multiple health checks

---

#### `HealthCheckResult`

Result of a health check

---

#### `HealthResponse`

Service health response

---

#### `HealthStatus`

Health check status

---

#### `HealthSummary`

Overall health summary

---

#### `HttpConfig`

HTTP server configuration

---

#### `HttpHealthCheck`

HTTP endpoint health check

---

#### `IndexerConfig`

Main configuration for the indexer service

---

#### `IndexerMetrics`

Indexer-specific metrics

---

#### `IngesterConfig`

Configuration for the event ingester

---

#### `IngestionStats`

Ingestion statistics

---

#### `JwtConfig`

JWT configuration

---

#### `LatencyMetrics`

Latency metrics

---

#### `LoggingConfig`

Logging configuration

---

#### `MemoryCache`

In-memory cache implementation

---

#### `MemoryCacheConfig`

Memory cache settings

---

#### `Metric`

Individual metric data

---

#### `MetricsCollector`

Main metrics collector

---

#### `MetricsConfig`

Metrics configuration

---

#### `MetricsRegistry`

Metrics registry for storing and managing metrics

---

#### `NodeInfo`

Information about a node

---

#### `PerformanceMetrics`

Performance metrics summary

---

#### `PostgresStore`

PostgreSQL storage implementation

---

#### `ProcessingConfig`

Processing configuration

---

#### `ProcessingStats`

Processing statistics

---

#### `ProcessorConfig`

Configuration for the event processor

---

#### `ProcessorStats`

Internal processor statistics

---

#### `PrometheusState`

Prometheus metrics server state

---

#### `QueueConfig`

Event queue configuration

---

#### `RateLimitConfig`

Rate limiting configuration

---

#### `RedisCache`

Redis-based cache implementation

---

#### `ResourceMetrics`

Resource utilization metrics

---

#### `RestHandler`

REST API handler

---

#### `RetentionConfig`

Data retention configuration

---

#### `RetryConfig`

Retry configuration

---

#### `ServiceConfig`

Service-level configuration

---

#### `ServiceContext`

Shared service context

---

#### `ShutdownCoordinator`

Graceful shutdown coordinator

---

#### `StorageBackendConfig`

Storage backend configuration

---

#### `StorageConfig`

Storage configuration

---

#### `StorageStats`

Storage statistics

---

#### `StoredEvent`

Stored event data structure

---

#### `StructuredLoggingConfig`

Structured logging configuration

---

#### `SubscriptionRequest`

WebSocket subscription request

---

#### `SummaryValue`

Summary metric value

---

#### `ThroughputMetrics`

Throughput metrics

---

#### `ValidationStage`

Validation pipeline stage

---

#### `WebSocketConfig`

WebSocket configuration

---

#### `WebSocketStreamer`

WebSocket streaming handler

---

### Enums

> Enumeration types for representing variants.

#### `AuthMethod`

Authentication methods

**Variants:**

- `None`
  - No authentication required
- `ApiKey`
  - API key-based authentication
- `Jwt`
  - JSON Web Token authentication
- `OAuth2`
  - OAuth 2.0 authentication

---

#### `BatchProcessorError`

Errors that can occur during batch processing

**Variants:**

- `SendFailed`
  - Failed to send batch through channel

---

#### `CacheBackend`

Cache backend types

**Variants:**

- `Redis`
  - Redis distributed cache for shared caching across instances
- `Memory`
  - In-memory cache for single-instance deployments
- `Disabled`
  - No caching enabled

---

#### `CompressionAlgorithm`

Compression algorithms

**Variants:**

- `Zstd`
  - Zstandard compression - best compression ratio
- `Gzip`
  - Gzip compression - widely compatible
- `Lz4`
  - LZ4 compression - fastest compression/decompression
- `None`
  - No compression

---

#### `ConfigError`

Configuration-related errors

**Variants:**

- `MissingField`
  - Missing required configuration field
- `InvalidValue`
  - Invalid configuration value
- `LoadFailed`
  - Failed to load configuration from source
- `ValidationFailed`
  - Configuration validation failed

---

#### `IndexerError`

Main error type for the indexer service

**Variants:**

- `Config`
  - Configuration errors
- `Storage`
  - Storage layer errors
- `Network`
  - Network/API errors
- `Processing`
  - Event processing errors
- `Metrics`
  - Metrics collection errors
- `Service`
  - Service lifecycle errors
- `Dependency`
  - External dependency errors
- `Validation`
  - Validation errors
- `Internal`
  - Internal errors that should not occur in normal operation

---

#### `LogFormat`

Log format options

**Variants:**

- `Json`
  - JSON structured logging format
- `Text`
  - Plain text logging format
- `Pretty`
  - Pretty-printed format for development

---

#### `LogOutput`

Log output destinations

**Variants:**

- `Stdout`
  - Standard output stream
- `Stderr`
  - Standard error stream
- `File`
  - Log to specified file path
- `Syslog`
  - System log daemon

---

#### `MetricType`

Metric types

**Variants:**

- `Counter`
  - Counter metric that only increases
- `Gauge`
  - Gauge metric that can go up or down
- `Histogram`
  - Histogram for observing value distributions
- `Summary`
  - Summary for calculating quantiles

---

#### `MetricValue`

Metric value

**Variants:**

- `Counter`
  - Counter value
- `Gauge`
  - Gauge value
- `Histogram`
  - Histogram observations
- `Summary`
  - Summary statistics

---

#### `MetricsError`

Metrics collection errors

**Variants:**

- `RegistryError`
  - Metrics registry error
- `RecordingFailed`
  - Metric recording failed
- `ExportFailed`
  - Metrics export failed
- `InvalidValue`
  - Invalid metric value

---

#### `NetworkError`

Network and API errors

**Variants:**

- `HttpFailed`
  - HTTP request failed
- `WebSocketFailed`
  - WebSocket connection failed
- `Timeout`
  - Request timeout
- `RateLimited`
  - Rate limit exceeded
- `DnsResolutionFailed`
  - DNS resolution failed
- `TlsError`
  - TLS/SSL error

---

#### `ProcessingError`

Event processing errors

**Variants:**

- `ParseFailed`
  - Failed to parse event
- `ValidationFailed`
  - Event validation failed
- `WorkerPoolExhausted`
  - Worker pool exhausted
- `QueueOverflow`
  - Event queue overflow
- `SerializationFailed`
  - Serialization failed
- `DeserializationFailed`
  - Deserialization failed
- `PipelineStalled`
  - Pipeline stalled

---

#### `QueueType`

Queue type options

**Variants:**

- `Memory`
  - In-memory queue for maximum performance
- `Disk`
  - Disk-backed queue for durability
- `Hybrid`
  - Hybrid queue using memory with disk overflow

---

#### `ServiceError`

Service lifecycle errors

**Variants:**

- `StartupFailed`
  - Service failed to start
- `ShutdownTimeout`
  - Service shutdown timeout
- `HealthCheckFailed`
  - Service health check failed
- `ResourceExhausted`
  - Resource exhausted
- `DependencyUnavailable`
  - Dependency unavailable

---

#### `ServiceState`

Service state enumeration

**Variants:**

- `Starting`
  - Service is initializing
- `Running`
  - Service is running normally
- `Stopping`
  - Service is shutting down gracefully
- `Stopped`
  - Service has stopped
- `Error`
  - Service encountered an error

---

#### `StorageBackend`

Storage backend types

**Variants:**

- `Postgres`
  - PostgreSQL database backend for general-purpose storage
- `ClickHouse`
  - ClickHouse columnar database for high-performance analytics
- `TimescaleDB`
  - TimescaleDB time-series database for temporal data
- `MongoDB`
  - MongoDB document database for flexible schema storage

---

#### `StorageError`

Storage layer errors

**Variants:**

- `ConnectionFailed`
  - Database connection failed
- `QueryFailed`
  - Database query failed
- `MigrationFailed`
  - Migration failed
- `DataCorruption`
  - Data corruption detected
- `CapacityExceeded`
  - Storage capacity exceeded
- `TransactionFailed`
  - Transaction failed
- `SchemaValidationFailed`
  - Schema validation failed

---

#### `StreamMessage`

Message types for WebSocket streaming

**Variants:**

- `Event`
  - New event notification
- `Health`
  - Health status update
- `Metrics`
  - Metrics update
- `Status`
  - Service status update
- `Error`
  - Error notification
- `Ping`
  - Heartbeat/keepalive
- `Pong`
  - Pong response

---

#### `SyncStrategy`

Disk sync strategies

**Variants:**

- `Always`
  - Sync to disk after every write for maximum durability
- `Periodic`
  - Sync to disk periodically for balanced performance
- `Never`
  - Never explicitly sync, rely on OS buffering

---

### Traits

> Trait definitions for implementing common behaviors.

#### `Cache`

Cache trait for storing and retrieving events

**Methods:**

- `set()`
  - Store an event in cache
- `get()`
  - Retrieve an event from cache
- `delete()`
  - Delete an event from cache
- `health_check()`
  - Check if cache is healthy
- `clear()`
  - Clear all cached data

---

#### `EventProcessing`

Event processing trait

**Methods:**

- `process_event()`
  - Process a single event
- `process_batch()`
  - Process a batch of events
- `processing_stats()`
  - Get processing statistics

---

#### `HealthCheck`

Health check trait for services and components

**Methods:**

- `health_check()`
  - Perform a health check
- `component_name()`
  - Get component name

---

#### `PipelineStage`

Pipeline stage trait for processing events

**Methods:**

- `process()`
  - Process an event and return the modified event
- `name()`
  - Get stage name for debugging

---

#### `ServiceLifecycle`

Service lifecycle trait

**Methods:**

- `start()`
  - Start the service
- `stop()`
  - Stop the service gracefully
- `health()`
  - Get current health status
- `is_running()`
  - Check if service is running

---

### Functions

> Standalone functions and utilities.

#### `calculate_percentile`

Calculate percentile from a sorted vector of values

---

#### `create_metrics_router`

Create Prometheus metrics router

---

#### `create_store`

Create a data store based on configuration

---

#### `format_bytes`

Format bytes in human-readable format

---

#### `format_duration`

Format duration in human-readable format

---

#### `generate_id`

Generate a unique ID

---

#### `get_event`

Get single event endpoint

---

#### `get_event_stats`

Get event statistics endpoint

---

#### `get_events`

Get events endpoint

---

#### `get_health`

Get service health endpoint

---

#### `get_metrics`

Get metrics endpoint

---

#### `hash_string`

Simple hash function for consistent hashing

---

#### `retry_with_backoff`

Retry helper with exponential backoff

---

#### `start_metrics_server`

Start Prometheus metrics server

---

#### `validate_event_id`

Validate event ID format

---

#### `validate_event_type`

Validate event type format

---

#### `validate_source`

Validate source format

---

#### `websocket_handler`

WebSocket upgrade handler

---

### Type Aliases

#### `IndexerResult`

Result type used throughout the indexer

**Type:** `<T, >`

---

#### `PipelineResult`

Result of pipeline processing

**Type:** `<T>`

---

### Constants

#### `POSTGRES_SCHEMA`

SQL schema for PostgreSQL

**Type:** `&str`

---

#### `VERSION_TABLE`

Migration version tracking

**Type:** `&str`

---
