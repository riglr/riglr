//! Data storage abstractions and implementations

use std::collections::HashMap;
use std::time::Duration;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::config::{StorageConfig, StorageBackend};
use crate::error::{IndexerError, IndexerResult};

pub mod postgres;
pub mod cache;
pub mod schema;

pub use postgres::PostgresStore;

/// Stored event data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredEvent {
    /// Unique event identifier
    pub id: String,
    /// Event type/category
    pub event_type: String,
    /// Event source (chain, protocol, etc.)
    pub source: String,
    /// Event data as JSON
    pub data: serde_json::Value,
    /// Event timestamp
    pub timestamp: DateTime<Utc>,
    /// Block height (if applicable)
    pub block_height: Option<u64>,
    /// Transaction hash (if applicable)
    pub transaction_hash: Option<String>,
}

/// Query filter for retrieving events
#[derive(Debug, Clone, Default)]
pub struct EventFilter {
    /// Filter by event types
    pub event_types: Option<Vec<String>>,
    /// Filter by sources
    pub sources: Option<Vec<String>>,
    /// Filter by time range
    pub time_range: Option<(DateTime<Utc>, DateTime<Utc>)>,
    /// Filter by block height range
    pub block_height_range: Option<(u64, u64)>,
    /// Filter by transaction hash
    pub transaction_hash: Option<String>,
    /// Additional custom filters
    pub custom_filters: HashMap<String, serde_json::Value>,
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
pub struct StorageStats {
    /// Total number of events stored
    pub total_events: u64,
    /// Storage size in bytes
    pub storage_size_bytes: u64,
    /// Average write latency in milliseconds
    pub avg_write_latency_ms: f64,
    /// Average read latency in milliseconds
    pub avg_read_latency_ms: f64,
    /// Number of active connections
    pub active_connections: u32,
    /// Cache hit rate (0.0 to 1.0)
    pub cache_hit_rate: f64,
}

/// Main data store trait
#[async_trait]
pub trait DataStore: Send + Sync {
    /// Insert a single event
    async fn insert_event(&self, event: &StoredEvent) -> IndexerResult<()>;

    /// Insert multiple events in a batch
    async fn insert_events(&self, events: &[StoredEvent]) -> IndexerResult<()>;

    /// Query events with filters
    async fn query_events(&self, query: &EventQuery) -> IndexerResult<Vec<StoredEvent>>;

    /// Get a specific event by ID
    async fn get_event(&self, id: &str) -> IndexerResult<Option<StoredEvent>>;

    /// Delete events matching filter
    async fn delete_events(&self, filter: &EventFilter) -> IndexerResult<u64>;

    /// Count events matching filter
    async fn count_events(&self, filter: &EventFilter) -> IndexerResult<u64>;

    /// Get storage statistics
    async fn get_stats(&self) -> IndexerResult<StorageStats>;

    /// Health check
    async fn health_check(&self) -> IndexerResult<()>;

    /// Cache an event (if caching is supported)
    async fn cache_event(&self, key: &str, event: &StoredEvent, ttl: Option<Duration>) -> IndexerResult<()> {
        // Default implementation is a no-op
        let _ = (key, event, ttl);
        Ok(())
    }

    /// Get cached event (if caching is supported)
    async fn get_cached_event(&self, key: &str) -> IndexerResult<Option<StoredEvent>> {
        // Default implementation returns None
        let _ = key;
        Ok(None)
    }

    /// Initialize storage (create tables, indexes, etc.)
    async fn initialize(&self) -> IndexerResult<()>;

    /// Perform maintenance operations
    async fn maintenance(&self) -> IndexerResult<()> {
        // Default implementation is a no-op
        Ok(())
    }
}

/// Create a data store based on configuration
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
                let store = crate::storage::clickhouse::ClickHouseStore::new(&config.primary).await?;
                store.initialize().await?;
                Ok(Box::new(store))
            }
            #[cfg(not(feature = "clickhouse"))]
            {
                Err(IndexerError::Config(crate::error::ConfigError::ValidationFailed {
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
        StorageBackend::MongoDB => {
            Err(IndexerError::Config(crate::error::ConfigError::ValidationFailed {
                message: "MongoDB support not yet implemented".to_string(),
            }))
        }
    }
}

/// Event aggregation helpers
pub struct EventAggregator;

impl EventAggregator {
    /// Aggregate events by time period
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
            let period_secs = period.as_secs() as i64;
            let timestamp_secs = event.timestamp.timestamp();
            let rounded_secs = (timestamp_secs / period_secs) * period_secs;
            let rounded_timestamp = DateTime::from_timestamp(rounded_secs, 0)
                .unwrap_or(event.timestamp);

            *aggregates.entry(rounded_timestamp).or_insert(0) += 1;
        }

        Ok(aggregates)
    }

    /// Aggregate events by type
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
            *aggregates.entry(event.event_type).or_insert(0) += 1;
        }

        Ok(aggregates)
    }

    /// Aggregate events by source
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
            *aggregates.entry(event.source).or_insert(0) += 1;
        }

        Ok(aggregates)
    }
}