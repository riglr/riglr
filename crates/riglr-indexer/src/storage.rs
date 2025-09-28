//! Data storage abstractions and implementations

#[path = "storage/cache.rs"]
pub mod cache;
#[path = "storage/postgres.rs"]
pub mod postgres;
#[path = "storage/schema.rs"]
pub mod schema;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use core::any::Any;
use core::time::Duration;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::config::{StorageBackend, StorageConfig};
use crate::error::{ConfigError, IndexerError, IndexerResult};

pub use postgres::Store as PostgresStore;

/// Stored event data structure
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct StoredEvent {
    /// Block height (if applicable)
    pub block_height: Option<i64>,
    /// Event data as JSON
    pub data: serde_json::Value,
    /// Event type/category
    pub event_type: String,
    /// Unique event identifier
    pub id: String,
    /// Event source (chain, protocol, etc.)
    pub source: String,
    /// Event timestamp
    pub timestamp: DateTime<Utc>,
    /// Transaction hash (if applicable)
    pub transaction_hash: Option<String>,
}

/// Query filter for retrieving events
#[derive(Debug, Clone, Default)]
pub struct EventFilter {
    /// Filter by block height range
    pub block_height_range: Option<(i64, i64)>,
    /// Additional custom filters
    pub custom_filters: HashMap<String, serde_json::Value>,
    /// Filter by event types
    pub event_types: Option<Vec<String>>,
    /// Filter by sources
    pub sources: Option<Vec<String>>,
    /// Filter by time range
    pub time_range: Option<(DateTime<Utc>, DateTime<Utc>)>,
    /// Filter by transaction hash
    pub transaction_hash: Option<String>,
}

/// Query parameters for event retrieval
#[derive(Debug, Clone)]
pub struct EventQuery {
    /// Filter criteria
    pub filter: EventFilter,
    /// Number of results to return (None = no limit)
    pub limit: Option<usize>,
    /// Number of results to skip
    pub offset: Option<usize>,
    /// Sort order (field, ascending)
    pub sort: Option<(String, bool)>,
}

impl Default for EventQuery {
    fn default() -> Self {
        Self {
            filter: EventFilter::default(),
            limit: Some(1000), // Default limit
            offset: None,
            sort: Some(("timestamp".to_string(), false)), // Default sort by timestamp DESC
        }
    }
}

/// Storage statistics
#[derive(Debug, Clone, Serialize)]
#[expect(clippy::module_name_repetitions)]
pub struct StorageStats {
    /// Number of active connections
    pub active_connections: u32,
    /// Average read latency in milliseconds
    pub avg_read_latency_ms: f64,
    /// Average write latency in milliseconds
    pub avg_write_latency_ms: f64,
    /// Cache hit rate (0.0 to 1.0)
    pub cache_hit_rate: f64,
    /// Storage size in bytes
    pub storage_size_bytes: u64,
    /// Total number of events stored
    pub total_events: u64,
}

/// Main data store trait
#[async_trait]
pub trait DataStore: Send + Sync {
    /// Get reference to concrete type for downcasting
    fn as_any(&self) -> &dyn Any;

    /// Cache an event (if caching is supported)
    async fn cache_event(
        &self,
        key: &str,
        event: &StoredEvent,
        ttl: Option<Duration>,
    ) -> IndexerResult<()> {
        // Default implementation is a no-op
        let _ = (key, event, ttl);
        Ok(())
    }

    /// Count events matching filter
    async fn count_events(&self, filter: &EventFilter) -> IndexerResult<u64>;

    /// Delete events matching filter
    async fn delete_events(&self, filter: &EventFilter) -> IndexerResult<u64>;

    /// Get cached event (if caching is supported)
    async fn get_cached_event(&self, key: &str) -> IndexerResult<Option<StoredEvent>> {
        // Default implementation returns None
        let _ = key;
        Ok(None)
    }

    /// Get a specific event by ID
    async fn get_event(&self, id: &str) -> IndexerResult<Option<StoredEvent>>;

    /// Get storage statistics
    async fn get_stats(&self) -> IndexerResult<StorageStats>;

    /// Health check
    async fn health_check(&self) -> IndexerResult<()>;

    /// Initialize storage (create tables, indexes, etc.)
    async fn initialize(&self) -> IndexerResult<()>;

    /// Insert a single event
    async fn insert_event(&self, event: &StoredEvent) -> IndexerResult<()>;

    /// Insert multiple events in a batch
    async fn insert_events(&self, events: &[StoredEvent]) -> IndexerResult<()>;

    /// Perform maintenance operations
    async fn maintenance(&self) -> IndexerResult<()> {
        // Default implementation is a no-op
        Ok(())
    }

    /// Query events with filters
    async fn query_events(&self, query: &EventQuery) -> IndexerResult<Vec<StoredEvent>>;
}

/// Event aggregation helpers
#[derive(Debug)]
pub struct EventAggregator;

/// Create a data store based on configuration
///
/// # Errors
///
/// Returns error if database connection fails or unsupported storage backend is specified
pub async fn create_store(config: &StorageConfig) -> IndexerResult<Box<dyn DataStore>> {
    match config.primary.backend {
        StorageBackend::Postgres => {
            let store = PostgresStore::new(&config.primary).await?;
            store.initialize().await?;
            Ok(Box::new(store))
        }
        StorageBackend::ClickHouse => {
            #[cfg(feature = "clickhouse")]
            {
                // Note: ClickHouse implementation would go here
                // For now, fallback to PostgreSQL
                let store = PostgresStore::new(&config.primary).await?;
                store.initialize().await?;
                Ok(Box::new(store))
            }
            #[cfg(not(feature = "clickhouse"))]
            {
                Err(IndexerError::Config(ConfigError::ValidationFailed {
                    message: "ClickHouse support not compiled in".to_string(),
                }))
            }
        }
        StorageBackend::TimescaleDB => {
            // TimescaleDB uses PostgreSQL driver
            let store = PostgresStore::new(&config.primary).await?;
            store.initialize().await?;
            Ok(Box::new(store))
        }
        StorageBackend::MongoDB => Err(IndexerError::Config(ConfigError::ValidationFailed {
            message: "MongoDB support not yet implemented".to_string(),
        })),
    }
}

impl EventAggregator {
    /// Aggregate events by source
    ///
    /// # Errors
    /// Returns an error if:
    /// - Database query fails
    /// - Event deserialization fails
    /// - Source aggregation processing fails
    /// - Memory allocation for results fails
    pub async fn aggregate_by_source(
        store: &dyn DataStore,
        filter: &EventFilter,
    ) -> IndexerResult<HashMap<String, u64>> {
        let query = EventQuery {
            filter: filter.clone(),
            limit: None,
            offset: None,
            sort: None,
        };

        let events = store.query_events(&query).await?;
        let mut aggregates = HashMap::new();

        for event in events {
            #[expect(clippy::arithmetic_side_effects)]
            {
                *aggregates.entry(event.source).or_insert(0) += 1;
            }
        }

        Ok(aggregates)
    }

    /// Aggregate events by time period
    ///
    /// # Errors
    /// Returns an error if:
    /// - Database query fails
    /// - Event deserialization fails
    /// - Timestamp conversion fails
    /// - Aggregation processing fails
    pub async fn aggregate_by_time(
        store: &dyn DataStore,
        period: Duration,
        filter: &EventFilter,
    ) -> IndexerResult<HashMap<DateTime<Utc>, u64>> {
        let query = EventQuery {
            filter: filter.clone(),
            limit: None,
            offset: None,
            sort: Some(("timestamp".to_string(), true)),
        };

        let events = store.query_events(&query).await?;
        let mut aggregates = HashMap::new();

        for event in events {
            // Round timestamp to period boundary
            let period_secs = period.as_secs().try_into().unwrap_or(i64::MAX);
            let timestamp_secs = event.timestamp.timestamp();
            #[expect(clippy::arithmetic_side_effects)]
            let rounded_secs = timestamp_secs
                .saturating_div(period_secs)
                .saturating_mul(period_secs);
            let rounded_timestamp =
                DateTime::from_timestamp(rounded_secs, 0).unwrap_or(event.timestamp);

            #[expect(clippy::arithmetic_side_effects)]
            {
                *aggregates.entry(rounded_timestamp).or_insert(0) += 1;
            }
        }

        Ok(aggregates)
    }

    /// Aggregate events by type
    ///
    /// # Errors
    /// Returns an error if:
    /// - Database query fails
    /// - Event deserialization fails
    /// - Type aggregation processing fails
    /// - Memory allocation for results fails
    pub async fn aggregate_by_type(
        store: &dyn DataStore,
        filter: &EventFilter,
    ) -> IndexerResult<HashMap<String, u64>> {
        let query = EventQuery {
            filter: filter.clone(),
            limit: None,
            offset: None,
            sort: None,
        };

        let events = store.query_events(&query).await?;
        let mut aggregates = HashMap::new();

        for event in events {
            #[expect(clippy::arithmetic_side_effects)]
            {
                *aggregates.entry(event.event_type).or_insert(0) += 1;
            }
        }

        Ok(aggregates)
    }
}

#[cfg(test)]
#[expect(clippy::expect_used, clippy::panic)]
#[expect(clippy::float_cmp)] // All test float comparisons use exact literal values
mod tests {
    use super::*;
    use crate::config::{
        ArchiveConfig, CacheBackend, CacheConfig, CacheTtlConfig, CompressionAlgorithm,
        CompressionConfig, ConnectionPoolConfig, MemoryCacheConfig, RetentionConfig,
        StorageBackendConfig,
    };
    use crate::error::ConfigError;
    use std::sync::{Arc, Mutex};

    // Mock DataStore implementation for testing
    struct MockDataStore {
        events: Arc<Mutex<Vec<StoredEvent>>>,
    }

    impl MockDataStore {
        fn new() -> Self {
            Self {
                events: Arc::new(Mutex::new(Vec::new())),
            }
        }
        fn with_events(events: Vec<StoredEvent>) -> Self {
            Self {
                events: Arc::new(Mutex::new(events)),
            }
        }
    }

    #[async_trait]
    impl DataStore for MockDataStore {
        async fn insert_event(&self, event: &StoredEvent) -> IndexerResult<()> {
            self.events
                .lock()
                .expect("Failed to acquire events mutex lock for insert")
                .push(event.clone());
            Ok(())
        }
        async fn insert_events(&self, events: &[StoredEvent]) -> IndexerResult<()> {
            self.events
                .lock()
                .expect("Failed to acquire events mutex lock for batch insert")
                .extend_from_slice(events);
            Ok(())
        }
        async fn query_events(&self, query: &EventQuery) -> IndexerResult<Vec<StoredEvent>> {
            let filtered = self
                .events
                .lock()
                .expect("Failed to acquire events mutex lock for query")
                .clone();
            let mut filtered: Vec<StoredEvent> = filtered;

            // Apply filter
            if let Some(ref event_types) = query.filter.event_types {
                filtered.retain(|e| event_types.contains(&e.event_type));
            }

            if let Some(ref sources) = query.filter.sources {
                filtered.retain(|e| sources.contains(&e.source));
            }

            if let Some((start, end)) = query.filter.time_range {
                filtered.retain(|e| e.timestamp >= start && e.timestamp <= end);
            }

            if let Some((min, max)) = query.filter.block_height_range {
                filtered.retain(|e| {
                    e.block_height
                        .is_some_and(|height| height >= min && height <= max)
                });
            }

            if let Some(ref tx_hash) = query.filter.transaction_hash {
                filtered.retain(|e| e.transaction_hash.as_ref() == Some(tx_hash));
            }

            // Apply sort
            if let Some((ref field, ascending)) = query.sort {
                if field == "timestamp" {
                    filtered.sort_by(|a, b| {
                        if ascending {
                            a.timestamp.cmp(&b.timestamp)
                        } else {
                            b.timestamp.cmp(&a.timestamp)
                        }
                    });
                }
            }

            // Apply offset and limit
            if let Some(offset) = query.offset {
                if offset < filtered.len() {
                    filtered = filtered
                        .get(offset..)
                        .expect("Offset bounds already checked")
                        .to_vec();
                } else {
                    filtered.clear();
                }
            }

            if let Some(limit) = query.limit {
                filtered.truncate(limit);
            }

            Ok(filtered)
        }
        async fn get_event(&self, id: &str) -> IndexerResult<Option<StoredEvent>> {
            let events = self
                .events
                .lock()
                .expect("Failed to acquire events mutex lock for get_event");
            Ok(events.iter().find(|e| e.id == id).cloned())
        }

        async fn delete_events(&self, filter: &EventFilter) -> IndexerResult<u64> {
            let mut events = self
                .events
                .lock()
                .expect("Failed to acquire events mutex lock for delete");
            let initial_count = events.len();

            events.retain(|e| {
                filter
                    .event_types
                    .as_ref()
                    .is_some_and(|event_types| !event_types.contains(&e.event_type))
            });

            #[expect(clippy::arithmetic_side_effects)]
            Ok((initial_count - events.len()) as u64)
        }
        async fn count_events(&self, _filter: &EventFilter) -> IndexerResult<u64> {
            return Ok(self
                .events
                .lock()
                .expect("Failed to acquire events mutex lock for count")
                .len() as u64);
        }
        async fn get_stats(&self) -> IndexerResult<StorageStats> {
            return Ok(StorageStats {
                total_events: self
                    .events
                    .lock()
                    .expect("Failed to acquire events mutex lock for stats")
                    .len() as u64,
                storage_size_bytes: 1024,
                avg_write_latency_ms: 5.0,
                avg_read_latency_ms: 2.0,
                active_connections: 1,
                cache_hit_rate: 0.8,
            });
        }
        async fn health_check(&self) -> IndexerResult<()> {
            Ok(())
        }
        async fn initialize(&self) -> IndexerResult<()> {
            Ok(())
        }
        fn as_any(&self) -> &dyn Any {
            self
        }
    }
    fn create_test_event(id: &str, event_type: &str, source: &str) -> StoredEvent {
        StoredEvent {
            id: id.to_string(),
            event_type: event_type.to_string(),
            source: source.to_string(),
            data: serde_json::json!({"test": "data"}),
            timestamp: Utc::now(),
            block_height: Some(12345i64),
            transaction_hash: Some("0xabc123".to_string()),
        }
    }

    #[test]
    fn test_stored_event_creation() {
        let event = create_test_event("test_id", "swap", "ethereum");

        assert_eq!(event.id, "test_id");
        assert_eq!(event.event_type, "swap");
        assert_eq!(event.source, "ethereum");
        assert_eq!(event.data, serde_json::json!({"test": "data"}));
        assert_eq!(event.block_height, Some(12345));
        assert_eq!(event.transaction_hash, Some("0xabc123".to_string()));
    }

    #[test]
    fn test_stored_event_debug() {
        let event = create_test_event("test", "test_type", "test_source");
        let debug_str = format!("{event:?}");
        assert!(debug_str.contains("StoredEvent"));
        assert!(debug_str.contains("test"));
    }

    #[test]
    fn test_stored_event_clone() {
        let event = create_test_event("test", "test_type", "test_source");
        let cloned = event.clone();
        assert_eq!(event.id, cloned.id);
        assert_eq!(event.event_type, cloned.event_type);
    }

    #[test]
    fn test_stored_event_serialize_deserialize() {
        let event = create_test_event("test", "test_type", "test_source");
        let serialized = serde_json::to_string(&event).expect("Failed to serialize test event");
        let deserialized: StoredEvent =
            serde_json::from_str(&serialized).expect("Failed to deserialize test event");
        assert_eq!(event.id, deserialized.id);
        assert_eq!(event.event_type, deserialized.event_type);
    }

    #[test]
    fn test_event_filter_default() {
        let filter = EventFilter::default();
        assert!(filter.event_types.is_none());
        assert!(filter.sources.is_none());
        assert!(filter.time_range.is_none());
        assert!(filter.block_height_range.is_none());
        assert!(filter.transaction_hash.is_none());
        assert!(filter.custom_filters.is_empty());
    }

    #[test]
    fn test_event_filter_with_event_types() {
        let filter = EventFilter {
            event_types: Some(vec!["swap".to_string(), "transfer".to_string()]),
            ..Default::default()
        };
        assert_eq!(
            filter
                .event_types
                .expect("Filter should have event_types")
                .len(),
            2
        );
    }

    #[test]
    fn test_event_filter_with_sources() {
        let filter = EventFilter {
            sources: Some(vec!["ethereum".to_string(), "solana".to_string()]),
            ..Default::default()
        };
        assert_eq!(filter.sources.expect("Filter should have sources").len(), 2);
    }

    #[test]
    fn test_event_filter_with_time_range() {
        let start = Utc::now();
        let end = Utc::now();
        let filter = EventFilter {
            time_range: Some((start, end)),
            ..Default::default()
        };
        assert!(filter.time_range.is_some());
    }

    #[test]
    fn test_event_filter_with_block_height_range() {
        let filter = EventFilter {
            block_height_range: Some((100, 200)),
            ..Default::default()
        };
        assert_eq!(
            filter
                .block_height_range
                .expect("Filter should have block_height_range"),
            (100, 200)
        );
    }

    #[test]
    fn test_event_filter_with_transaction_hash() {
        let filter = EventFilter {
            transaction_hash: Some("0xabc123".to_string()),
            ..Default::default()
        };
        assert_eq!(
            filter
                .transaction_hash
                .expect("Filter should have transaction_hash"),
            "0xabc123"
        );
    }

    #[test]
    fn test_event_filter_with_custom_filters() {
        let mut custom_filters = HashMap::new();
        custom_filters.insert("key1".to_string(), serde_json::json!("value1"));
        let filter = EventFilter {
            custom_filters,
            ..Default::default()
        };
        assert_eq!(filter.custom_filters.len(), 1);
    }

    #[test]
    fn test_event_filter_debug() {
        let filter = EventFilter::default();
        let debug_str = format!("{filter:?}");
        assert!(debug_str.contains("EventFilter"));
    }

    #[test]
    fn test_event_filter_clone() {
        let filter = EventFilter {
            event_types: Some(vec!["test".to_string()]),
            ..Default::default()
        };
        let cloned = filter.clone();
        assert_eq!(filter.event_types, cloned.event_types);
    }

    #[test]
    fn test_event_query_default() {
        let query = EventQuery::default();
        assert_eq!(query.limit, Some(1000));
        assert!(query.offset.is_none());
        assert_eq!(query.sort, Some(("timestamp".to_string(), false)));
    }

    #[test]
    fn test_event_query_with_filter() {
        let filter = EventFilter {
            event_types: Some(vec!["test".to_string()]),
            ..Default::default()
        };
        let query = EventQuery {
            filter,
            limit: Some(100),
            offset: Some(10),
            sort: Some(("id".to_string(), true)),
        };
        assert_eq!(query.limit, Some(100));
        assert_eq!(query.offset, Some(10));
        assert_eq!(query.sort, Some(("id".to_string(), true)));
    }

    #[test]
    fn test_event_query_debug() {
        let query = EventQuery::default();
        let debug_str = format!("{query:?}");
        assert!(debug_str.contains("EventQuery"));
    }

    #[test]
    fn test_event_query_clone() {
        let query = EventQuery::default();
        let cloned = query.clone();
        assert_eq!(query.limit, cloned.limit);
        assert_eq!(query.offset, cloned.offset);
    }

    #[test]
    fn test_storage_stats_debug() {
        let stats = StorageStats {
            total_events: 100,
            storage_size_bytes: 1024,
            avg_write_latency_ms: 5.0,
            avg_read_latency_ms: 2.0,
            active_connections: 1,
            cache_hit_rate: 0.8,
        };
        let debug_str = format!("{stats:?}");
        assert!(debug_str.contains("StorageStats"));
        assert!(debug_str.contains("100"));
    }

    #[test]
    fn test_storage_stats_clone() {
        let stats = StorageStats {
            total_events: 100,
            storage_size_bytes: 1024,
            avg_write_latency_ms: 5.0,
            avg_read_latency_ms: 2.0,
            active_connections: 1,
            cache_hit_rate: 0.8,
        };
        let cloned = stats.clone();
        assert_eq!(stats.total_events, cloned.total_events);
        assert_eq!(stats.storage_size_bytes, cloned.storage_size_bytes);
    }

    #[test]
    fn test_storage_stats_serialize() {
        let stats = StorageStats {
            total_events: 100,
            storage_size_bytes: 1024,
            avg_write_latency_ms: 5.0,
            avg_read_latency_ms: 2.0,
            active_connections: 1,
            cache_hit_rate: 0.8,
        };
        let serialized = serde_json::to_string(&stats).expect("Failed to serialize test stats");
        assert!(serialized.contains("100"));
        assert!(serialized.contains("1024"));
    }

    #[tokio::test]
    async fn test_data_store_cache_event_default() {
        let store = MockDataStore::new();
        let event = create_test_event("test", "test_type", "test_source");
        let result = store.cache_event("test_key", &event, None).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_data_store_cache_event_with_ttl() {
        let store = MockDataStore::new();
        let event = create_test_event("test", "test_type", "test_source");
        let ttl = Some(Duration::from_secs(60));
        let result = store.cache_event("test_key", &event, ttl).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_data_store_get_cached_event_default() {
        let store = MockDataStore::new();
        let result = store.get_cached_event("test_key").await;
        assert!(result.is_ok());
        assert!(result.expect("Cache get should succeed").is_none());
    }

    #[tokio::test]
    async fn test_data_store_maintenance_default() {
        let store = MockDataStore::new();
        let result = store.maintenance().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_create_store_postgres() {
        let config = StorageConfig {
            primary: StorageBackendConfig {
                backend: StorageBackend::Postgres,
                url: "postgresql://test:test@localhost:5432/test?sslmode=disable&application_name=test".to_string(),
                pool: ConnectionPoolConfig {
                    max_connections: 10,
                    min_connections: 1,
                    connect_timeout: Duration::from_secs(30),
                    idle_timeout: Duration::from_secs(600),
                    max_lifetime: Duration::from_secs(1800),
                },
                settings: HashMap::default(),
            },
            secondary: None,
            cache: CacheConfig {
                backend: CacheBackend::Disabled,
                redis_url: None,
                ttl: CacheTtlConfig {
                    default: Duration::from_secs(300),
                    events: Duration::from_secs(3600),
                    aggregates: Duration::from_secs(1800),
                },
                memory: MemoryCacheConfig {
                    max_size_bytes: 100_000_000,
                    max_entries: 10_000,
                },
            },
            retention: RetentionConfig {
                default: Duration::from_secs(30 * 24 * 3600),
                by_event_type: HashMap::default(),
                archive: ArchiveConfig {
                    enabled: false,
                    backend: None,
                    compression: CompressionConfig {
                        algorithm: CompressionAlgorithm::Zstd,
                        level: 3,
                    },
                },
            },
        };

        // This will fail because we don't have a real database, but we test the code path
        let result = create_store(&config).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_create_store_timescaledb() {
        let config = StorageConfig {
            primary: StorageBackendConfig {
                backend: StorageBackend::TimescaleDB,
                url: "postgresql://test:test@localhost:5432/test?sslmode=disable&application_name=test".to_string(),
                pool: ConnectionPoolConfig {
                    max_connections: 10,
                    min_connections: 1,
                    connect_timeout: Duration::from_secs(30),
                    idle_timeout: Duration::from_secs(600),
                    max_lifetime: Duration::from_secs(1800),
                },
                settings: HashMap::default(),
            },
            secondary: None,
            cache: CacheConfig {
                backend: CacheBackend::Disabled,
                redis_url: None,
                ttl: CacheTtlConfig {
                    default: Duration::from_secs(300),
                    events: Duration::from_secs(3600),
                    aggregates: Duration::from_secs(1800),
                },
                memory: MemoryCacheConfig {
                    max_size_bytes: 100_000_000,
                    max_entries: 10_000,
                },
            },
            retention: RetentionConfig {
                default: Duration::from_secs(30 * 24 * 3600),
                by_event_type: HashMap::default(),
                archive: ArchiveConfig {
                    enabled: false,
                    backend: None,
                    compression: CompressionConfig {
                        algorithm: CompressionAlgorithm::Zstd,
                        level: 3,
                    },
                },
            },
        };

        // This will fail because we don't have a real database, but we test the code path
        let result = create_store(&config).await;
        assert!(result.is_err());
    }

    #[cfg(not(feature = "clickhouse"))]
    #[tokio::test]
    async fn test_create_store_clickhouse_not_compiled() {
        let config = StorageConfig {
            primary: StorageBackendConfig {
                backend: StorageBackend::ClickHouse,
                url: "http://test:test@localhost:8123/test".to_string(),
                pool: ConnectionPoolConfig {
                    max_connections: 10,
                    min_connections: 1,
                    connect_timeout: Duration::from_secs(30),
                    idle_timeout: Duration::from_secs(600),
                    max_lifetime: Duration::from_secs(1800),
                },
                settings: HashMap::default(),
            },
            secondary: None,
            cache: CacheConfig {
                backend: CacheBackend::Disabled,
                redis_url: None,
                ttl: CacheTtlConfig {
                    default: Duration::from_secs(300),
                    events: Duration::from_secs(3600),
                    aggregates: Duration::from_secs(1800),
                },
                memory: MemoryCacheConfig {
                    max_size_bytes: 100_000_000,
                    max_entries: 10_000,
                },
            },
            retention: RetentionConfig {
                default: Duration::from_secs(30 * 24 * 3600),
                by_event_type: HashMap::default(),
                archive: ArchiveConfig {
                    enabled: false,
                    backend: None,
                    compression: CompressionConfig {
                        algorithm: CompressionAlgorithm::Zstd,
                        level: 3,
                    },
                },
            },
        };

        let result = create_store(&config).await;
        assert!(result.is_err());
        if let Err(IndexerError::Config(ConfigError::ValidationFailed { message })) = result {
            assert_eq!(message, "ClickHouse support not compiled in");
        } else {
            panic!("Expected ConfigError::ValidationFailed");
        }
    }

    #[cfg(feature = "clickhouse")]
    #[tokio::test]
    async fn test_create_store_clickhouse_compiled() {
        let config = StorageConfig {
            primary: StorageBackendConfig {
                backend: StorageBackend::ClickHouse,
                url: "http://test:test@localhost:8123/test".to_string(),
                pool: ConnectionPoolConfig {
                    max_connections: 10,
                    min_connections: 1,
                    connect_timeout: Duration::from_secs(30),
                    idle_timeout: Duration::from_secs(600),
                    max_lifetime: Duration::from_secs(1800),
                },
                settings: HashMap::default(),
            },
            secondary: None,
            cache: CacheConfig {
                backend: CacheBackend::Disabled,
                redis_url: None,
                ttl: CacheTtlConfig {
                    default: Duration::from_secs(300),
                    events: Duration::from_secs(3600),
                    aggregates: Duration::from_secs(1800),
                },
                memory: MemoryCacheConfig {
                    max_size_bytes: 100_000_000,
                    max_entries: 10_000,
                },
            },
            retention: RetentionConfig {
                default: Duration::from_secs(30 * 24 * 3600),
                by_event_type: HashMap::default(),
                archive: ArchiveConfig {
                    enabled: false,
                    backend: None,
                    compression: CompressionConfig {
                        algorithm: CompressionAlgorithm::Zstd,
                        level: 3,
                    },
                },
            },
        };

        // This will fail because we don't have a real database, but we test the code path
        let result = create_store(&config).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_create_store_mongodb_not_implemented() {
        let config = StorageConfig {
            primary: StorageBackendConfig {
                backend: StorageBackend::MongoDB,
                url: "mongodb://test:test@localhost:27017/test".to_string(),
                pool: ConnectionPoolConfig {
                    max_connections: 10,
                    min_connections: 1,
                    connect_timeout: Duration::from_secs(30),
                    idle_timeout: Duration::from_secs(600),
                    max_lifetime: Duration::from_secs(1800),
                },
                settings: HashMap::default(),
            },
            secondary: None,
            cache: CacheConfig {
                backend: CacheBackend::Disabled,
                redis_url: None,
                ttl: CacheTtlConfig {
                    default: Duration::from_secs(300),
                    events: Duration::from_secs(3600),
                    aggregates: Duration::from_secs(1800),
                },
                memory: MemoryCacheConfig {
                    max_size_bytes: 100_000_000,
                    max_entries: 10_000,
                },
            },
            retention: RetentionConfig {
                default: Duration::from_secs(30 * 24 * 3600),
                by_event_type: HashMap::default(),
                archive: ArchiveConfig {
                    enabled: false,
                    backend: None,
                    compression: CompressionConfig {
                        algorithm: CompressionAlgorithm::Zstd,
                        level: 3,
                    },
                },
            },
        };

        let result = create_store(&config).await;
        assert!(result.is_err());
        if let Err(IndexerError::Config(ConfigError::ValidationFailed { message })) = result {
            assert_eq!(message, "MongoDB support not yet implemented");
        } else {
            panic!("Expected ConfigError::ValidationFailed");
        }
    }

    #[tokio::test]
    async fn test_event_aggregator_aggregate_by_time() {
        let event1 = StoredEvent {
            timestamp: DateTime::from_timestamp(1000, 0).expect("Valid timestamp for test"),
            ..create_test_event("1", "swap", "ethereum")
        };
        let event2 = StoredEvent {
            timestamp: DateTime::from_timestamp(1100, 0).expect("Valid timestamp for test"),
            ..create_test_event("2", "transfer", "ethereum")
        };
        let event3 = StoredEvent {
            timestamp: DateTime::from_timestamp(2000, 0).expect("Valid timestamp for test"),
            ..create_test_event("3", "swap", "solana")
        };

        let store = MockDataStore::with_events(vec![event1, event2, event3]);
        let filter = EventFilter::default();
        let period = Duration::from_secs(1000);

        let result = EventAggregator::aggregate_by_time(&store, period, &filter).await;
        assert!(result.is_ok());
        let aggregates = result.expect("Time aggregation should succeed");
        assert!(!aggregates.is_empty());
    }

    #[tokio::test]
    async fn test_event_aggregator_aggregate_by_time_with_filter() {
        let event1 = StoredEvent {
            timestamp: DateTime::from_timestamp(1000, 0).expect("Valid timestamp for test"),
            event_type: "swap".to_string(),
            ..create_test_event("1", "swap", "ethereum")
        };
        let event2 = StoredEvent {
            timestamp: DateTime::from_timestamp(1100, 0).expect("Valid timestamp for test"),
            event_type: "transfer".to_string(),
            ..create_test_event("2", "transfer", "ethereum")
        };

        let store = MockDataStore::with_events(vec![event1, event2]);
        let filter = EventFilter {
            event_types: Some(vec!["swap".to_string()]),
            ..Default::default()
        };
        let period = Duration::from_secs(500);

        let result = EventAggregator::aggregate_by_time(&store, period, &filter).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_event_aggregator_aggregate_by_time_empty_events() {
        let store = MockDataStore::new();
        let filter = EventFilter::default();
        let period = Duration::from_secs(1000);

        let result = EventAggregator::aggregate_by_time(&store, period, &filter).await;
        assert!(result.is_ok());
        let aggregates = result.expect("Time aggregation should succeed");
        assert_eq!(aggregates.len(), 0);
    }

    #[tokio::test]
    async fn test_event_aggregator_aggregate_by_type() {
        let event1 = create_test_event("1", "swap", "ethereum");
        let event2 = create_test_event("2", "transfer", "ethereum");
        let event3 = create_test_event("3", "swap", "solana");

        let store = MockDataStore::with_events(vec![event1, event2, event3]);
        let filter = EventFilter::default();

        let result = EventAggregator::aggregate_by_type(&store, &filter).await;
        assert!(result.is_ok());
        let aggregates = result.expect("Time aggregation should succeed");
        assert_eq!(aggregates.get("swap"), Some(&2));
        assert_eq!(aggregates.get("transfer"), Some(&1));
    }

    #[tokio::test]
    async fn test_event_aggregator_aggregate_by_type_with_filter() {
        let event1 = create_test_event("1", "swap", "ethereum");
        let event2 = create_test_event("2", "transfer", "ethereum");
        let event3 = create_test_event("3", "swap", "solana");

        let store = MockDataStore::with_events(vec![event1, event2, event3]);
        let filter = EventFilter {
            sources: Some(vec!["ethereum".to_string()]),
            ..Default::default()
        };

        let result = EventAggregator::aggregate_by_type(&store, &filter).await;
        assert!(result.is_ok());
        let aggregates = result.expect("Time aggregation should succeed");
        assert_eq!(aggregates.get("swap"), Some(&1));
        assert_eq!(aggregates.get("transfer"), Some(&1));
        assert!(aggregates.contains_key("swap"));
    }

    #[tokio::test]
    async fn test_event_aggregator_aggregate_by_type_empty_events() {
        let store = MockDataStore::new();
        let filter = EventFilter::default();

        let result = EventAggregator::aggregate_by_type(&store, &filter).await;
        assert!(result.is_ok());
        let aggregates = result.expect("Time aggregation should succeed");
        assert_eq!(aggregates.len(), 0);
    }

    #[tokio::test]
    async fn test_event_aggregator_aggregate_by_source() {
        let event1 = create_test_event("1", "swap", "ethereum");
        let event2 = create_test_event("2", "transfer", "ethereum");
        let event3 = create_test_event("3", "swap", "solana");

        let store = MockDataStore::with_events(vec![event1, event2, event3]);
        let filter = EventFilter::default();

        let result = EventAggregator::aggregate_by_source(&store, &filter).await;
        assert!(result.is_ok());
        let aggregates = result.expect("Time aggregation should succeed");
        assert_eq!(aggregates.get("ethereum"), Some(&2));
        assert_eq!(aggregates.get("solana"), Some(&1));
    }

    #[tokio::test]
    async fn test_event_aggregator_aggregate_by_source_with_filter() {
        let event1 = create_test_event("1", "swap", "ethereum");
        let event2 = create_test_event("2", "transfer", "ethereum");
        let event3 = create_test_event("3", "swap", "solana");

        let store = MockDataStore::with_events(vec![event1, event2, event3]);
        let filter = EventFilter {
            event_types: Some(vec!["swap".to_string()]),
            ..Default::default()
        };

        let result = EventAggregator::aggregate_by_source(&store, &filter).await;
        assert!(result.is_ok());
        let aggregates = result.expect("Time aggregation should succeed");
        assert_eq!(aggregates.get("ethereum"), Some(&1));
        assert_eq!(aggregates.get("solana"), Some(&1));
    }

    #[tokio::test]
    async fn test_event_aggregator_aggregate_by_source_empty_events() {
        let store = MockDataStore::new();
        let filter = EventFilter::default();

        let result = EventAggregator::aggregate_by_source(&store, &filter).await;
        assert!(result.is_ok());
        let aggregates = result.expect("Time aggregation should succeed");
        assert_eq!(aggregates.len(), 0);
    }

    #[tokio::test]
    async fn test_mock_data_store_insert_event() {
        let store = MockDataStore::new();
        let event = create_test_event("test", "test_type", "test_source");

        let result = store.insert_event(&event).await;
        assert!(result.is_ok());

        let retrieved = store
            .get_event("test")
            .await
            .expect("Get event should succeed");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.expect("Event should exist").id, "test");
    }

    #[tokio::test]
    async fn test_mock_data_store_insert_events() {
        let store = MockDataStore::new();
        let events = vec![
            create_test_event("1", "type1", "source1"),
            create_test_event("2", "type2", "source2"),
        ];

        let result = store.insert_events(&events).await;
        assert!(result.is_ok());

        let count = store
            .count_events(&EventFilter::default())
            .await
            .expect("Count events should succeed");
        assert_eq!(count, 2);
    }

    #[tokio::test]
    async fn test_mock_data_store_query_events_with_limit() {
        let events = vec![
            create_test_event("1", "type1", "source1"),
            create_test_event("2", "type2", "source2"),
            create_test_event("3", "type3", "source3"),
        ];
        let store = MockDataStore::with_events(events);

        let query = EventQuery {
            filter: EventFilter::default(),
            limit: Some(2),
            offset: None,
            sort: None,
        };

        let result = store
            .query_events(&query)
            .await
            .expect("Query with limit should succeed");
        assert_eq!(result.len(), 2);
    }

    #[tokio::test]
    async fn test_mock_data_store_query_events_with_offset() {
        let events = vec![
            create_test_event("1", "type1", "source1"),
            create_test_event("2", "type2", "source2"),
            create_test_event("3", "type3", "source3"),
        ];
        let store = MockDataStore::with_events(events);

        let query = EventQuery {
            filter: EventFilter::default(),
            limit: None,
            offset: Some(1),
            sort: None,
        };

        let result = store
            .query_events(&query)
            .await
            .expect("Query with limit should succeed");
        assert_eq!(result.len(), 2);
        assert_eq!(
            result
                .first()
                .expect("Result should contain at least one element")
                .id,
            "2"
        );
    }

    #[tokio::test]
    async fn test_mock_data_store_query_events_with_large_offset() {
        let events = vec![create_test_event("1", "type1", "source1")];
        let store = MockDataStore::with_events(events);

        let query = EventQuery {
            filter: EventFilter::default(),
            limit: None,
            offset: Some(10),
            sort: None,
        };

        let result = store
            .query_events(&query)
            .await
            .expect("Query with limit should succeed");
        assert_eq!(result.len(), 0);
    }

    #[tokio::test]
    async fn test_mock_data_store_get_event_not_found() {
        let store = MockDataStore::new();
        let result = store
            .get_event("nonexistent")
            .await
            .expect("Get nonexistent event should succeed");
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_mock_data_store_delete_events() {
        let events = vec![
            create_test_event("1", "type1", "source1"),
            create_test_event("2", "type2", "source2"),
        ];
        let store = MockDataStore::with_events(events);

        let filter = EventFilter {
            event_types: Some(vec!["type1".to_string()]),
            ..Default::default()
        };

        let deleted_count = store
            .delete_events(&filter)
            .await
            .expect("Delete events should succeed");
        assert_eq!(deleted_count, 1);
    }

    #[tokio::test]
    async fn test_mock_data_store_health_check() {
        let store = MockDataStore::new();
        let result = store.health_check().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_mock_data_store_get_stats() {
        let store = MockDataStore::new();
        let stats = store.get_stats().await.expect("Get stats should succeed");
        assert_eq!(stats.total_events, 0);
        assert_eq!(stats.storage_size_bytes, 1024);
        assert_eq!(stats.avg_write_latency_ms, 5.0);
        assert_eq!(stats.avg_read_latency_ms, 2.0);
        assert_eq!(stats.cache_hit_rate, 0.8);
        assert_eq!(stats.active_connections, 1);
    }

    #[tokio::test]
    async fn test_mock_data_store_initialize() {
        let store = MockDataStore::new();
        let result = store.initialize().await;
        assert!(result.is_ok());
    }
}
