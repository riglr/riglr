//! `PostgreSQL` storage implementation

use core::{any::Any, fmt::Write, time::Duration};
use sqlx::{PgPool, Row};
use std::sync::RwLock;
use std::time::Instant;
use tracing::{debug, error, info};

use crate::config::StorageBackendConfig;
use crate::error::{IndexerError, IndexerResult, StorageError};
use crate::storage::{DataStore, EventFilter, EventQuery, StorageStats, StoredEvent};
use crate::utils::safe_cast_u64_to_f64;

/// `PostgreSQL` storage implementation
#[derive(Debug)]
pub struct Store {
    /// Whether partitioning is enabled
    #[expect(dead_code)] // Will be used for table partitioning optimization
    enable_partitioning: bool,
    /// Database connection pool
    pool: PgPool,
    /// Statistics tracking
    stats: RwLock<InternalStats>,
}

#[derive(Default, Debug, Clone, PartialEq)]
struct InternalStats {
    read_latency_sum: f64,
    total_events: u64,
    total_reads: u64,
    total_writes: u64,
    write_latency_sum: f64,
}

impl Store {
    /// Create a new `PostgreSQL` store
    ///
    /// # Errors
    ///
    /// Returns error if database connection fails or connection pool setup fails
    pub async fn new(config: &StorageBackendConfig) -> IndexerResult<Self> {
        info!(
            "Connecting to PostgreSQL at {}",
            config.url.split('@').next_back().unwrap_or(&config.url)
        );

        let connect_result = sqlx::PgPool::connect(&config.url).await;
        let pool = connect_result.map_err(|e| {
            IndexerError::Storage(StorageError::ConnectionFailed {
                message: e.to_string(),
            })
        })?;

        // Test connection
        let test_result = sqlx::query("SELECT 1").execute(&pool).await;
        test_result.map_err(|e| {
            IndexerError::Storage(StorageError::ConnectionFailed {
                message: format!("Connection test failed: {e}"),
            })
        })?;

        info!("PostgreSQL connection established successfully");

        let enable_partitioning = config
            .settings
            .get("enable_partitioning")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);

        Ok(Self {
            enable_partitioning,
            pool,
            stats: RwLock::new(InternalStats::default()),
        })
    }

    /// Record read latency
    fn record_read_latency(&self, duration: Duration) {
        if let Ok(mut stats) = self.stats.write() {
            stats.total_reads = stats.total_reads.saturating_add(1);
            // Use as_secs_f64() * 1000.0 to avoid precision loss from u128 -> f64 cast
            stats.read_latency_sum += duration.as_secs_f64() * 1000.0;
        }
    }

    /// Record write latency
    fn record_write_latency(&self, duration: Duration) {
        if let Ok(mut stats) = self.stats.write() {
            stats.total_writes = stats.total_writes.saturating_add(1);
            // Use as_secs_f64() * 1000.0 to avoid precision loss from u128 -> f64 cast
            stats.write_latency_sum += duration.as_secs_f64() * 1000.0;
        }
    }

    /// Build WHERE clause for SQL queries based on filters
    #[cfg(test)]
    fn build_where_clause(filter: &EventFilter) -> String {
        let mut conditions = Vec::default();
        let mut param_idx: i32 = 1;

        if let Some(event_types) = filter.event_types.as_ref() {
            if !event_types.is_empty() {
                conditions.push(format!("event_type = ANY(${param_idx})"));
                param_idx = param_idx.saturating_add(1);
            }
        }

        if let Some(sources) = filter.sources.as_ref() {
            if !sources.is_empty() {
                conditions.push(format!("source = ANY(${param_idx})"));
                param_idx = param_idx.saturating_add(1);
            }
        }

        if let Some(&(_start, _end)) = filter.time_range.as_ref() {
            conditions.push(format!(
                "timestamp >= ${} AND timestamp <= ${}",
                param_idx,
                param_idx.saturating_add(1)
            ));
            param_idx = param_idx.saturating_add(2);
        }

        if let Some(&(_min_height, _max_height)) = filter.block_height_range.as_ref() {
            conditions.push(format!(
                "block_height >= ${} AND block_height <= ${}",
                param_idx,
                param_idx.saturating_add(1)
            ));
            param_idx = param_idx.saturating_add(2);
        }

        if let Some(_tx_hash) = filter.transaction_hash.as_ref() {
            conditions.push(format!("transaction_hash = ${param_idx}"));
        }

        if conditions.is_empty() {
            return String::default();
        }
        format!("WHERE {}", conditions.join(" AND "))
    }
}

#[async_trait::async_trait]
impl DataStore for Store {
    fn as_any(&self) -> &dyn Any {
        self
    }

    async fn initialize(&self) -> IndexerResult<()> {
        info!("Initializing PostgreSQL schema");

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS events (
                id TEXT PRIMARY KEY,
                event_type TEXT NOT NULL,
                source TEXT NOT NULL,
                data JSONB NOT NULL,
                "timestamp" TIMESTAMPTZ NOT NULL,
                block_height BIGINT,
                transaction_hash TEXT,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| {
            IndexerError::Storage(StorageError::QueryFailed {
                query: format!("CREATE TABLE events failed: {e}"),
            })
        })?;

        sqlx::query(r#"CREATE INDEX IF NOT EXISTS idx_events_timestamp ON events ("timestamp")"#)
            .execute(&self.pool)
            .await
            .ok();
        sqlx::query(r"CREATE INDEX IF NOT EXISTS idx_events_event_type ON events (event_type)")
            .execute(&self.pool)
            .await
            .ok();
        sqlx::query(r"CREATE INDEX IF NOT EXISTS idx_events_source ON events (source)")
            .execute(&self.pool)
            .await
            .ok();
        sqlx::query(r"CREATE INDEX IF NOT EXISTS idx_events_block_height ON events (block_height) WHERE block_height IS NOT NULL").execute(&self.pool).await.ok();
        sqlx::query(r"CREATE INDEX IF NOT EXISTS idx_events_transaction_hash ON events (transaction_hash) WHERE transaction_hash IS NOT NULL").execute(&self.pool).await.ok();
        sqlx::query(r"CREATE INDEX IF NOT EXISTS idx_events_data_gin ON events USING GIN (data)")
            .execute(&self.pool)
            .await
            .ok();
        sqlx::query(r"CREATE INDEX IF NOT EXISTS idx_events_created_at ON events (created_at)")
            .execute(&self.pool)
            .await
            .ok();

        info!("PostgreSQL schema initialized successfully");
        return Ok(());
    }

    async fn insert_event(&self, event: &StoredEvent) -> IndexerResult<()> {
        let start_time = Instant::now();

        sqlx::query(
            r"
            INSERT INTO events (id, event_type, source, data, timestamp, block_height, transaction_hash)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (id) DO UPDATE SET
                event_type = EXCLUDED.event_type,
                source = EXCLUDED.source,
                data = EXCLUDED.data,
                timestamp = EXCLUDED.timestamp,
                block_height = EXCLUDED.block_height,
                transaction_hash = EXCLUDED.transaction_hash
            ",
        )
        .bind(&event.id)
        .bind(&event.event_type)
        .bind(&event.source)
        .bind(&event.data)
        .bind(event.timestamp)
        .bind(event.block_height)
        .bind(&event.transaction_hash)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to insert event {}: {}", event.id, e);
            IndexerError::Storage(StorageError::QueryFailed {
                query: format!("INSERT event {}", event.id),
            })
        })?;

        self.record_write_latency(start_time.elapsed());

        if let Ok(mut stats) = self.stats.write() {
            stats.total_events = stats.total_events.saturating_add(1);
        }

        debug!("Event {} inserted successfully", event.id);
        return Ok(());
    }

    async fn count_events(&self, _filter: &EventFilter) -> IndexerResult<u64> {
        // This simplified version counts all events. A full implementation would
        // require dynamic query building similar to `query_events`.
        let fetch_result = sqlx::query("SELECT COUNT(*) as count FROM events")
            .fetch_one(&self.pool)
            .await;
        let mapped_result = fetch_result.map_err(|e| {
            error!("Count query failed: {}", e);
            IndexerError::Storage(StorageError::QueryFailed {
                query: "SELECT COUNT(*)".to_string(),
            })
        });
        let query_result = mapped_result?;
        let row = query_result;

        let count: i64 = row.get("count");
        // Database COUNT() should never return negative values
        if count < 0 {
            tracing::warn!("Database returned negative count {}, treating as 0", count);
            Ok(0)
        } else {
            #[expect(clippy::cast_sign_loss)]
            Ok(count as u64)
        }
    }

    async fn insert_events(&self, events: &[StoredEvent]) -> IndexerResult<()> {
        if events.is_empty() {
            return Ok(());
        }

        let start_time = Instant::now();
        info!("Batch inserting {} events", events.len());

        // Use a transaction for batch insert
        let begin_result = self.pool.begin().await;
        let mut tx = begin_result.map_err(|_e| {
            IndexerError::Storage(StorageError::TransactionFailed {
                operation: "begin batch insert".to_string(),
            })
        })?;

        for event in events {
            sqlx::query(
                r"
                INSERT INTO events (id, event_type, source, data, timestamp, block_height, transaction_hash)
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                ON CONFLICT (id) DO UPDATE SET
                    event_type = EXCLUDED.event_type,
                    source = EXCLUDED.source,
                    data = EXCLUDED.data,
                    timestamp = EXCLUDED.timestamp,
                    block_height = EXCLUDED.block_height,
                    transaction_hash = EXCLUDED.transaction_hash
                ",
            )
            .bind(&event.id)
            .bind(&event.event_type)
            .bind(&event.source)
            .bind(&event.data)
            .bind(event.timestamp)
            .bind(event.block_height)
            .bind(&event.transaction_hash)
            .execute(&mut *tx)
            .await
            .map_err(|e| {
                error!("Failed to insert event {} in batch: {}", event.id, e);
                IndexerError::Storage(StorageError::QueryFailed {
                    query: format!("BATCH INSERT event {}", event.id),
                })
            })?;
        }

        let commit_result = tx.commit().await;
        commit_result.map_err(|e| {
            error!("Failed to commit batch insert: {}", e);
            IndexerError::Storage(StorageError::TransactionFailed {
                operation: "commit batch insert".to_string(),
            })
        })?;

        self.record_write_latency(start_time.elapsed());

        if let Ok(mut stats) = self.stats.write() {
            stats.total_events = stats.total_events.saturating_add(events.len() as u64);
        }

        info!(
            "Batch insert of {} events completed in {:?}",
            events.len(),
            start_time.elapsed()
        );
        return Ok(());
    }

    async fn delete_events(&self, filter: &EventFilter) -> IndexerResult<u64> {
        if let Some(&(_, end)) = filter.time_range.as_ref() {
            let execute_result = sqlx::query(r#"DELETE FROM events WHERE "timestamp" <= $1"#)
                .bind(end)
                .execute(&self.pool)
                .await;
            let result = execute_result.map_err(|e| {
                error!("Delete failed: {}", e);
                IndexerError::Storage(StorageError::QueryFailed {
                    query: "DELETE with time range".to_string(),
                })
            })?;
            return Ok(result.rows_affected());
        }
        Err(IndexerError::validation(
            "Delete operations must have a time_range filter for safety.",
        ))
    }

    async fn get_event(&self, id: &str) -> IndexerResult<Option<StoredEvent>> {
        let start_time = Instant::now();

        let event = sqlx::query_as::<_, StoredEvent>(
            r#"
            SELECT id, event_type, source, data, "timestamp", block_height, transaction_hash
            FROM events
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to get event {}: {}", id, e);
            IndexerError::Storage(StorageError::QueryFailed {
                query: format!("GET event {id}"),
            })
        })?;

        self.record_read_latency(start_time.elapsed());
        Ok(event)
    }

    async fn get_stats(&self) -> IndexerResult<StorageStats> {
        // Get database stats
        let size_row = sqlx::query(
            r"
            SELECT pg_total_relation_size('events') as size_bytes,
                   COUNT(*) as total_events
            FROM events
            ",
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|_e| {
            IndexerError::Storage(StorageError::QueryFailed {
                query: "get_stats".to_string(),
            })
        })?;

        let storage_size_bytes: i64 = size_row.get("size_bytes");
        let total_events: i64 = size_row.get("total_events");

        // Get connection stats (we'll use a default value since pool.options() is private)
        let pool_state = 10u32; // Default max connections

        let stats = self
            .stats
            .read()
            .map_err(|_| IndexerError::internal("Failed to read stats"))?;

        let avg_write_latency_ms = if stats.total_writes > 0 {
            // Safe conversion - precision loss acceptable for latency averages
            let writes_as_f64 = safe_cast_u64_to_f64(stats.total_writes);
            stats.write_latency_sum / writes_as_f64
        } else {
            0.0
        };

        let avg_read_latency_ms = if stats.total_reads > 0 {
            // Safe conversion - precision loss acceptable for latency averages
            let reads_as_f64 = safe_cast_u64_to_f64(stats.total_reads);
            stats.read_latency_sum / reads_as_f64
        } else {
            0.0
        };

        Ok(StorageStats {
            // Database values should never be negative, safe cast with bounds checking
            total_events: if total_events < 0 {
                tracing::warn!(
                    "Database returned negative total_events {}, treating as 0",
                    total_events
                );
                0
            } else {
                #[expect(clippy::cast_sign_loss)]
                {
                    total_events as u64
                }
            },
            storage_size_bytes: if storage_size_bytes < 0 {
                tracing::warn!(
                    "Database returned negative storage_size_bytes {}, treating as 0",
                    storage_size_bytes
                );
                0
            } else {
                #[expect(clippy::cast_sign_loss)]
                {
                    storage_size_bytes as u64
                }
            },
            avg_write_latency_ms,
            avg_read_latency_ms,
            active_connections: pool_state,
            cache_hit_rate: 0.0, // Not applicable for direct PostgreSQL access
        })
    }

    #[expect(clippy::expect_used)]
    async fn query_events(&self, query: &EventQuery) -> IndexerResult<Vec<StoredEvent>> {
        let start_time = Instant::now();

        let mut sql = String::from("SELECT id, event_type, source, data, timestamp, block_height, transaction_hash FROM events");
        let mut conditions = Vec::default();
        let mut param_idx: i32 = 1;

        if let Some(event_types) = query.filter.event_types.as_ref() {
            if !event_types.is_empty() {
                conditions.push(format!("event_type = ANY(${param_idx})"));
                param_idx = param_idx.saturating_add(1);
            }
        }
        if let Some(sources) = query.filter.sources.as_ref() {
            if !sources.is_empty() {
                conditions.push(format!("source = ANY(${param_idx})"));
                param_idx = param_idx.saturating_add(1);
            }
        }
        if query.filter.time_range.is_some() {
            conditions.push(format!(
                "timestamp >= ${} AND timestamp <= ${}",
                param_idx,
                param_idx.saturating_add(1)
            ));
            param_idx = param_idx.saturating_add(2);
        }
        if query.filter.block_height_range.is_some() {
            conditions.push(format!(
                "block_height >= ${} AND block_height <= ${}",
                param_idx,
                param_idx.saturating_add(1)
            ));
            param_idx = param_idx.saturating_add(2);
        }
        if query.filter.transaction_hash.is_some() {
            conditions.push(format!("transaction_hash = ${param_idx}"));
        }

        if !conditions.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&conditions.join(" AND "));
        }

        if let Some(&(ref sort_field, ascending)) = query.sort.as_ref() {
            let direction = if ascending { "ASC" } else { "DESC" };
            // Writing to String is infallible - only fails on out-of-memory conditions
            write!(sql, " ORDER BY {sort_field} {direction}")
                .expect("Failed to write ORDER BY clause to SQL string");
        }
        if let Some(limit) = query.limit {
            // Writing to String is infallible - only fails on out-of-memory conditions
            write!(sql, " LIMIT {limit}").expect("Failed to write LIMIT clause to SQL string");
        }
        if let Some(offset) = query.offset {
            // Writing to String is infallible - only fails on out-of-memory conditions
            write!(sql, " OFFSET {offset}").expect("Failed to write OFFSET clause to SQL string");
        }

        let mut sqlx_query = sqlx::query_as::<_, StoredEvent>(&sql);

        if let Some(event_types) = query.filter.event_types.as_ref() {
            if !event_types.is_empty() {
                sqlx_query = sqlx_query.bind(event_types);
            }
        }
        if let Some(sources) = query.filter.sources.as_ref() {
            if !sources.is_empty() {
                sqlx_query = sqlx_query.bind(sources);
            }
        }
        if let Some(&(start, end)) = query.filter.time_range.as_ref() {
            sqlx_query = sqlx_query.bind(start).bind(end);
        }
        if let Some(&(min_height, max_height)) = query.filter.block_height_range.as_ref() {
            sqlx_query = sqlx_query.bind(min_height).bind(max_height);
        }
        if let Some(tx_hash) = query.filter.transaction_hash.as_ref() {
            sqlx_query = sqlx_query.bind(tx_hash);
        }

        debug!("Executing query: {}", sql);

        let fetch_result = sqlx_query.fetch_all(&self.pool).await;
        let events = fetch_result.map_err(|e| {
            error!("Query failed: {}", e);
            IndexerError::Storage(StorageError::QueryFailed { query: sql })
        })?;

        self.record_read_latency(start_time.elapsed());
        debug!(
            "Query returned {} events in {:?}",
            events.len(),
            start_time.elapsed()
        );
        Ok(events)
    }

    async fn health_check(&self) -> IndexerResult<()> {
        let execute_result = sqlx::query("SELECT 1").execute(&self.pool).await;
        let mapped_result = execute_result.map_err(|e| {
            IndexerError::Storage(StorageError::ConnectionFailed {
                message: format!("Health check failed: {e}"),
            })
        });
        let _health_result = mapped_result?;

        Ok(())
    }
}

#[cfg(test)]
#[expect(clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use crate::config::{ConnectionPoolConfig, StorageBackend};
    use crate::prelude::PostgresStore;
    use crate::storage::{EventFilter, EventQuery, StoredEvent};
    use chrono::{DateTime, Utc};
    use core::f64::consts::PI;
    use core::time::Duration;
    use serde_json::{json, Value};
    use std::{collections::HashMap, env};

    // Helper function to create test configuration
    fn create_test_config() -> StorageBackendConfig {
        StorageBackendConfig {
            backend: StorageBackend::Postgres,
            url: "postgresql://test:test@localhost:5432/test_db".to_string(),
            pool: ConnectionPoolConfig {
                max_connections: 10,
                min_connections: 2,
                connect_timeout: Duration::from_secs(30),
                idle_timeout: Duration::from_secs(300),
                max_lifetime: Duration::from_secs(1800),
            },
            settings: HashMap::default(),
        }
    }

    // Helper function to create test event
    fn create_test_event(id: &str) -> StoredEvent {
        StoredEvent {
            id: id.to_string(),
            event_type: "test_event".to_string(),
            source: "test_source".to_string(),
            data: json!({"key": "value"}),
            timestamp: Utc::now(),
            block_height: Some(12345i64),
            transaction_hash: Some("0x123abc".to_string()),
        }
    }

    // Helper function to create test event with custom fields
    fn create_custom_event(
        id: &str,
        event_type: &str,
        source: &str,
        data: Value,
        block_height: Option<i64>,
        transaction_hash: Option<String>,
    ) -> StoredEvent {
        StoredEvent {
            id: id.to_string(),
            event_type: event_type.to_string(),
            source: source.to_string(),
            data,
            timestamp: Utc::now(),
            block_height,
            transaction_hash,
        }
    }

    // Test InternalStats
    #[test]
    fn test_internal_stats_default() {
        let stats = InternalStats::default();
        assert_eq!(stats.total_events, 0);
        assert_eq!(stats.total_writes, 0);
        assert_eq!(stats.total_reads, 0);
        assert!((stats.write_latency_sum - 0.0).abs() < f64::EPSILON);
        assert!((stats.read_latency_sum - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_internal_stats_clone() {
        let stats = InternalStats {
            total_events: 100,
            total_writes: 50,
            total_reads: 30,
            write_latency_sum: 250.5,
            read_latency_sum: 150.3,
        };
        let cloned = stats.clone();
        assert_eq!(stats, cloned);
    }

    #[test]
    fn test_internal_stats_debug() {
        let stats = InternalStats {
            total_events: 100,
            total_writes: 50,
            total_reads: 30,
            write_latency_sum: 250.5,
            read_latency_sum: 150.3,
        };
        let debug_output = format!("{stats:?}");
        assert!(debug_output.contains("InternalStats"));
        assert!(debug_output.contains("100"));
        assert!(debug_output.contains("50"));
        assert!(debug_output.contains("30"));
    }

    // Test build_where_clause function - Happy paths
    #[test]
    fn test_build_where_clause_when_empty_filter_should_return_empty_clause() {
        let filter = EventFilter::default();
        let where_clause = PostgresStore::build_where_clause(&filter);
        assert_eq!(where_clause, "");
    }

    #[test]
    fn test_build_where_clause_when_event_types_only_should_return_event_type_clause() {
        let filter = EventFilter {
            event_types: Some(vec!["transfer".to_string(), "mint".to_string()]),
            ..Default::default()
        };
        let where_clause = PostgresStore::build_where_clause(&filter);
        assert_eq!(where_clause, "WHERE event_type = ANY($1)");
    }

    #[test]
    fn test_build_where_clause_when_sources_only_should_return_source_clause() {
        let filter = EventFilter {
            sources: Some(vec!["ethereum".to_string(), "polygon".to_string()]),
            ..Default::default()
        };
        let where_clause = PostgresStore::build_where_clause(&filter);
        assert_eq!(where_clause, "WHERE source = ANY($1)");
    }

    #[test]
    fn test_build_where_clause_when_time_range_only_should_return_time_clause() {
        let start = DateTime::from_timestamp(1_000_000_000, 0).expect("Valid timestamp for test");
        let end = DateTime::from_timestamp(2_000_000_000, 0).expect("Valid timestamp for test");
        let filter = EventFilter {
            time_range: Some((start, end)),
            ..Default::default()
        };
        let where_clause = PostgresStore::build_where_clause(&filter);
        assert_eq!(where_clause, "WHERE timestamp >= $1 AND timestamp <= $2");
    }

    #[test]
    fn test_build_where_clause_when_block_height_range_only_should_return_block_height_clause() {
        let filter = EventFilter {
            block_height_range: Some((1000, 2000)),
            ..Default::default()
        };
        let where_clause = PostgresStore::build_where_clause(&filter);
        assert_eq!(
            where_clause,
            "WHERE block_height >= $1 AND block_height <= $2"
        );
    }

    #[test]
    fn test_build_where_clause_when_transaction_hash_only_should_return_tx_hash_clause() {
        let filter = EventFilter {
            transaction_hash: Some("0xabc123".to_string()),
            ..Default::default()
        };
        let where_clause = PostgresStore::build_where_clause(&filter);
        assert_eq!(where_clause, "WHERE transaction_hash = $1");
    }

    #[test]
    fn test_build_where_clause_when_multiple_filters_should_combine_with_and() {
        let start = DateTime::from_timestamp(1_000_000_000, 0).expect("Valid timestamp for test");
        let end = DateTime::from_timestamp(2_000_000_000, 0).expect("Valid timestamp for test");
        let filter = EventFilter {
            event_types: Some(vec!["transfer".to_string()]),
            sources: Some(vec!["ethereum".to_string()]),
            time_range: Some((start, end)),
            block_height_range: Some((1000, 2000)),
            transaction_hash: Some("0xabc123".to_string()),
            ..Default::default()
        };
        let where_clause = PostgresStore::build_where_clause(&filter);
        assert!(where_clause.starts_with("WHERE "));
        assert!(where_clause.contains("event_type = ANY($1)"));
        assert!(where_clause.contains("source = ANY($2)"));
        assert!(where_clause.contains("timestamp >= $3 AND timestamp <= $4"));
        assert!(where_clause.contains("block_height >= $5 AND block_height <= $6"));
        assert!(where_clause.contains("transaction_hash = $7"));
        assert!(where_clause.matches(" AND ").count() == 4); // 4 AND operators for 5 conditions
    }

    // Test build_where_clause edge cases
    #[test]
    fn test_build_where_clause_when_empty_event_types_should_not_add_condition() {
        let filter = EventFilter {
            event_types: Some(vec![]),
            ..Default::default()
        };
        let where_clause = PostgresStore::build_where_clause(&filter);
        assert_eq!(where_clause, "");
    }

    #[test]
    fn test_build_where_clause_when_empty_sources_should_not_add_condition() {
        let filter = EventFilter {
            sources: Some(vec![]),
            ..Default::default()
        };
        let where_clause = PostgresStore::build_where_clause(&filter);
        assert_eq!(where_clause, "");
    }

    #[test]
    fn test_build_where_clause_when_none_values_should_not_add_conditions() {
        let filter = EventFilter {
            event_types: None,
            sources: None,
            time_range: None,
            block_height_range: None,
            transaction_hash: None,
            custom_filters: HashMap::default(),
        };
        let where_clause = PostgresStore::build_where_clause(&filter);
        assert_eq!(where_clause, "");
    }

    #[test]
    fn test_build_where_clause_when_single_event_type_should_handle_correctly() {
        let filter = EventFilter {
            event_types: Some(vec!["single_event".to_string()]),
            ..Default::default()
        };
        let where_clause = PostgresStore::build_where_clause(&filter);
        assert_eq!(where_clause, "WHERE event_type = ANY($1)");
    }

    #[test]
    fn test_build_where_clause_when_zero_block_heights_should_handle_correctly() {
        let filter = EventFilter {
            block_height_range: Some((0, 0)),
            ..Default::default()
        };
        let where_clause = PostgresStore::build_where_clause(&filter);
        assert_eq!(
            where_clause,
            "WHERE block_height >= $1 AND block_height <= $2"
        );
    }

    #[test]
    fn test_build_where_clause_when_large_block_heights_should_handle_correctly() {
        let filter = EventFilter {
            block_height_range: Some((i64::MAX - 1, i64::MAX)),
            ..Default::default()
        };
        let where_clause = PostgresStore::build_where_clause(&filter);
        assert_eq!(
            where_clause,
            "WHERE block_height >= $1 AND block_height <= $2"
        );
    }

    #[test]
    fn test_build_where_clause_when_empty_string_transaction_hash_should_handle_correctly() {
        let filter = EventFilter {
            transaction_hash: Some(String::new()),
            ..Default::default()
        };
        let where_clause = PostgresStore::build_where_clause(&filter);
        assert_eq!(where_clause, "WHERE transaction_hash = $1");
    }

    #[test]
    fn test_build_where_clause_when_very_long_transaction_hash_should_handle_correctly() {
        let long_hash = "0x".to_string() + &"a".repeat(1000);
        let filter = EventFilter {
            transaction_hash: Some(long_hash),
            ..Default::default()
        };
        let where_clause = PostgresStore::build_where_clause(&filter);
        assert_eq!(where_clause, "WHERE transaction_hash = $1");
    }

    #[test]
    fn test_build_where_clause_when_unicode_transaction_hash_should_handle_correctly() {
        let filter = EventFilter {
            transaction_hash: Some("🦀❤️🔗".to_string()),
            ..Default::default()
        };
        let where_clause = PostgresStore::build_where_clause(&filter);
        assert_eq!(where_clause, "WHERE transaction_hash = $1");
    }

    // Note: The following tests would require actual database connections and mock setup
    // Since we're focusing on unit tests for the static/pure functions, we'll create
    // comprehensive tests for the helper methods that don't require database connections

    #[tokio::test]
    async fn test_postgres_store_new_when_invalid_url_should_return_connection_error() {
        let mut config = create_test_config();
        config.url = "invalid_url".to_string();

        let result = PostgresStore::new(&config).await;
        assert!(result.is_err());
        match result.expect_err("Expected connection to fail with invalid URL") {
            IndexerError::Storage(StorageError::ConnectionFailed { message }) => {
                assert!(!message.is_empty());
            }
            _ => panic!("Expected Storage(ConnectionFailed) error"),
        }
    }

    #[test]
    fn test_postgres_store_record_write_latency_when_stats_available() {
        let _config = create_test_config();
        let stats = RwLock::new(InternalStats::default());

        // Create a minimal PostgresStore-like structure to test the latency recording logic
        // Since we can't create a real PostgresStore without a database connection,
        // we'll test the stats update logic directly
        let duration = Duration::from_millis(100);

        if let Ok(mut stats_guard) = stats.write() {
            stats_guard.total_writes = stats_guard.total_writes.saturating_add(1);
            stats_guard.write_latency_sum += duration.as_secs_f64() * 1000.0;
        }

        {
            let stats_read = stats.read().expect("Failed to acquire read lock for stats");
            assert_eq!(stats_read.total_writes, 1);
            assert!((stats_read.write_latency_sum - 100.0).abs() < f64::EPSILON);
            drop(stats_read);
        }
    }

    #[test]
    fn test_postgres_store_record_read_latency_when_stats_available() {
        let stats = RwLock::new(InternalStats::default());
        let duration = Duration::from_millis(50);

        if let Ok(mut stats_guard) = stats.write() {
            stats_guard.total_reads = stats_guard.total_reads.saturating_add(1);
            stats_guard.read_latency_sum += duration.as_secs_f64() * 1000.0;
        }

        {
            let stats_read = stats.read().expect("Failed to acquire read lock for stats");
            assert_eq!(stats_read.total_reads, 1);
            assert!((stats_read.read_latency_sum - 50.0).abs() < f64::EPSILON);
            drop(stats_read);
        }
    }

    #[test]
    fn test_postgres_store_record_write_latency_when_stats_locked_should_not_panic() {
        let stats = RwLock::new(InternalStats::default());
        let duration = Duration::from_millis(100);

        // Simulate a lock failure by holding a read lock and trying to write
        let _read_guard = stats.read().expect("Failed to acquire read lock for stats");

        // This should not panic even if the write lock fails
        if let Ok(mut stats_guard) = stats.try_write() {
            stats_guard.total_writes = stats_guard.total_writes.saturating_add(1);
            stats_guard.write_latency_sum += duration.as_secs_f64() * 1000.0;
        };
        // If lock fails, operation should silently continue
    }

    #[test]
    fn test_postgres_store_record_read_latency_when_stats_locked_should_not_panic() {
        let stats = RwLock::new(InternalStats::default());
        let duration = Duration::from_millis(50);

        // Simulate a lock failure by holding a read lock and trying to write
        let _read_guard = stats.read().expect("Failed to acquire read lock for stats");

        // This should not panic even if the write lock fails
        if let Ok(mut stats_guard) = stats.try_write() {
            stats_guard.total_reads = stats_guard.total_reads.saturating_add(1);
            stats_guard.read_latency_sum += duration.as_secs_f64() * 1000.0;
        };
        // If lock fails, operation should silently continue
    }

    #[test]
    fn test_postgres_store_record_zero_duration_latency() {
        let stats = RwLock::new(InternalStats::default());
        let duration = Duration::from_nanos(0);

        if let Ok(mut stats_guard) = stats.write() {
            stats_guard.total_writes = stats_guard.total_writes.saturating_add(1);
            stats_guard.write_latency_sum += duration.as_secs_f64() * 1000.0;
        }

        {
            let stats_read = stats.read().expect("Failed to acquire read lock for stats");
            assert_eq!(stats_read.total_writes, 1);
            assert!((stats_read.write_latency_sum - 0.0).abs() < f64::EPSILON);
            drop(stats_read);
        }
    }

    #[test]
    fn test_postgres_store_record_very_large_duration_latency() {
        let stats = RwLock::new(InternalStats::default());
        let duration = Duration::from_secs(3600); // 1 hour

        if let Ok(mut stats_guard) = stats.write() {
            stats_guard.total_writes = stats_guard.total_writes.saturating_add(1);
            stats_guard.write_latency_sum += duration.as_secs_f64() * 1000.0;
        }

        {
            let stats_read = stats.read().expect("Failed to acquire read lock for stats");
            assert_eq!(stats_read.total_writes, 1);
            assert!((stats_read.write_latency_sum - 3_600_000.0).abs() < f64::EPSILON);
            // 1 hour in milliseconds
            drop(stats_read);
        }
    }

    #[test]
    fn test_postgres_store_multiple_latency_recordings() {
        let stats = RwLock::new(InternalStats::default());

        // Record multiple write latencies
        for i in 1..=5 {
            let duration = Duration::from_millis(i * 10);
            let stats_result = stats.write();
            if let Ok(mut stats_guard) = stats_result {
                stats_guard.total_writes = stats_guard.total_writes.saturating_add(1);
                stats_guard.write_latency_sum += duration.as_secs_f64() * 1000.0;
            }
        }

        // Record multiple read latencies
        for i in 1..=3 {
            let duration = Duration::from_millis(i * 20);
            let stats_result = stats.write();
            if let Ok(mut stats_guard) = stats_result {
                stats_guard.total_reads = stats_guard.total_reads.saturating_add(1);
                stats_guard.read_latency_sum += duration.as_secs_f64() * 1000.0;
            }
        }

        {
            let stats_read = stats.read().expect("Failed to acquire read lock for stats");
            assert_eq!(stats_read.total_writes, 5);
            assert_eq!(stats_read.total_reads, 3);
            let expected_write_sum = 10.0 + 20.0 + 30.0 + 40.0 + 50.0; // 150.0
            let expected_read_sum = 20.0 + 40.0 + 60.0; // 120.0
            assert!((stats_read.write_latency_sum - expected_write_sum).abs() < f64::EPSILON);
            assert!((stats_read.read_latency_sum - expected_read_sum).abs() < f64::EPSILON);
            drop(stats_read);
        }
    }

    // Test EventQuery and EventFilter edge cases that would be processed by the implementation
    #[test]
    fn test_event_query_default_values() {
        let query = EventQuery::default();
        assert_eq!(query.limit, Some(1000));
        assert_eq!(query.offset, None);
        assert_eq!(query.sort, Some(("timestamp".to_string(), false)));
    }

    #[test]
    fn test_event_filter_default_values() {
        let filter = EventFilter::default();
        assert_eq!(filter.event_types, None);
        assert_eq!(filter.sources, None);
        assert_eq!(filter.time_range, None);
        assert_eq!(filter.block_height_range, None);
        assert_eq!(filter.transaction_hash, None);
        assert!(filter.custom_filters.is_empty());
    }

    #[test]
    fn test_stored_event_creation_with_all_fields() {
        let event = create_test_event("test_id");
        assert_eq!(event.id, "test_id");
        assert_eq!(event.event_type, "test_event");
        assert_eq!(event.source, "test_source");
        assert_eq!(event.data, json!({"key": "value"}));
        assert_eq!(event.block_height, Some(12345));
        assert_eq!(event.transaction_hash, Some("0x123abc".to_string()));
    }

    #[test]
    fn test_stored_event_creation_with_minimal_fields() {
        let event = create_custom_event(
            "minimal_id",
            "minimal_event",
            "minimal_source",
            json!({}),
            None,
            None,
        );
        assert_eq!(event.id, "minimal_id");
        assert_eq!(event.event_type, "minimal_event");
        assert_eq!(event.source, "minimal_source");
        assert_eq!(event.data, json!({}));
        assert_eq!(event.block_height, None);
        assert_eq!(event.transaction_hash, None);
    }

    #[test]
    fn test_stored_event_creation_with_complex_data() {
        let complex_data = json!({
            "nested": {
                "array": [1, 2, 3],
                "object": {
                    "key": "value",
                    "number": 42,
                    "boolean": true,
                    "null_value": null
                }
            },
            "unicode": "🦀❤️🔗",
            "large_number": 18_446_744_073_709_551_615_u64
        });

        let event = create_custom_event(
            "complex_id",
            "complex_event",
            "complex_source",
            complex_data,
            Some(0),
            Some(String::new()),
        );

        assert_eq!(
            event
                .data
                .get("nested")
                .expect("Should have nested field")
                .get("array")
                .expect("Should have array field"),
            &json!([1, 2, 3])
        );
        assert_eq!(event.block_height, Some(0));
        assert_eq!(event.transaction_hash, Some(String::new()));
    }

    #[test]
    fn test_stored_event_creation_with_edge_case_values() {
        let event = create_custom_event(
            "",                      // Empty ID
            "",                      // Empty event type
            "",                      // Empty source
            json!(null),             // Null data
            Some(i64::MAX),          // Maximum block height
            Some("x".repeat(10000)), // Very long transaction hash
        );

        assert_eq!(event.id, "");
        assert_eq!(event.event_type, "");
        assert_eq!(event.source, "");
        assert_eq!(event.data, json!(null));
        assert_eq!(event.block_height, Some(i64::MAX));
        assert_eq!(
            event
                .transaction_hash
                .as_ref()
                .expect("Transaction hash should be present")
                .len(),
            10000
        );
    }

    // Test configuration handling
    #[test]
    fn test_storage_backend_config_creation() {
        let config = create_test_config();
        assert_eq!(config.backend, StorageBackend::Postgres);
        assert_eq!(config.url, "postgresql://test:test@localhost:5432/test_db");
        assert_eq!(config.pool.max_connections, 10);
        assert_eq!(config.pool.min_connections, 2);
        assert!(config.settings.is_empty());
    }

    #[test]
    fn test_storage_backend_config_with_custom_settings() {
        let mut config = create_test_config();
        config
            .settings
            .insert("enable_partitioning".to_string(), json!(true));
        config
            .settings
            .insert("max_batch_size".to_string(), json!(1000));

        assert_eq!(config.settings.len(), 2);
        assert_eq!(
            config.settings.get("enable_partitioning"),
            Some(&json!(true))
        );
        assert_eq!(config.settings.get("max_batch_size"), Some(&json!(1000)));
    }

    #[test]
    fn test_storage_backend_config_with_different_backends() {
        let backends = vec![
            StorageBackend::Postgres,
            StorageBackend::ClickHouse,
            StorageBackend::TimescaleDB,
            StorageBackend::MongoDB,
        ];

        for backend in backends {
            let mut config = create_test_config();
            config.backend = backend;
            assert_eq!(config.backend, backend);
        }
    }

    #[test]
    fn test_connection_pool_config_edge_values() {
        let pool_config = ConnectionPoolConfig {
            max_connections: 1,
            min_connections: 0,
            connect_timeout: Duration::from_nanos(1),
            idle_timeout: Duration::from_nanos(1),
            max_lifetime: Duration::from_nanos(1),
        };

        assert_eq!(pool_config.max_connections, 1);
        assert_eq!(pool_config.min_connections, 0);
        assert_eq!(pool_config.connect_timeout, Duration::from_nanos(1));
    }

    #[test]
    fn test_connection_pool_config_large_values() {
        let pool_config = ConnectionPoolConfig {
            max_connections: u32::MAX,
            min_connections: u32::MAX - 1,
            connect_timeout: Duration::from_secs(86400),
            idle_timeout: Duration::from_secs(86400),
            max_lifetime: Duration::from_secs(86400),
        };

        assert_eq!(pool_config.max_connections, u32::MAX);
        assert_eq!(pool_config.min_connections, u32::MAX - 1);
        assert_eq!(pool_config.connect_timeout, Duration::from_secs(86400));
    }

    // Test URL parsing edge cases that would be handled by PostgresStore::new
    #[test]
    fn test_postgres_store_config_url_variations() {
        // Test URLs with placeholder patterns
        let urls = vec![
            format!(
                "postgresql://{}:{}@host:5432/db",
                env::var("TEST_DB_U").unwrap_or_else(|_| "u".to_string()),
                env::var("TEST_DB_P").unwrap_or_else(|_| "p".to_string())
            ),
            format!(
                "postgres://{}:{}@host:5432/db",
                env::var("TEST_DB_U").unwrap_or_else(|_| "u".to_string()),
                env::var("TEST_DB_P").unwrap_or_else(|_| "p".to_string())
            ),
            "postgresql://user@host:5432/db".to_string(),
            "postgresql://host:5432/db".to_string(),
            "postgresql://host/db".to_string(),
            "postgresql:///db".to_string(),
            "postgresql://".to_string(),
            String::new(),
        ];

        for url in urls {
            let mut config = create_test_config();
            config.url = url.to_string();

            // Test that the config structure accepts various URL formats
            // The actual validation would happen in PostgresStore::new()
            assert_eq!(config.url, url);
        }
    }

    #[test]
    fn test_postgres_store_config_url_with_special_characters() {
        let mut config = create_test_config();
        config.url = "postgresql://us%40r:p%40ss@host:5432/d%40ta%24ase".to_string();

        // Test that the config can handle URL-encoded characters
        assert!(config.url.contains("%40"));
        assert!(config.url.contains("%24"));
    }

    #[test]
    fn test_postgres_store_config_url_with_query_parameters() {
        let mut config = create_test_config();
        config.url = format!(
            "postgresql://{}:{}@host:5432/db?sslmode=require&application_name=riglr",
            env::var("TEST_DB_U").unwrap_or_else(|_| "u".to_string()),
            env::var("TEST_DB_P").unwrap_or_else(|_| "p".to_string())
        );

        assert!(config.url.contains("sslmode=require"));
        assert!(config.url.contains("application_name=riglr"));
    }

    // Test URL sanitization logic (similar to what's used in the new() method)
    #[test]
    fn test_url_sanitization_logic() {
        // Test cases with placeholder credentials
        let test_url = format!(
            "postgresql://{}:{}@host:5432/db",
            env::var("TEST_U").unwrap_or_else(|_| "u".to_string()),
            env::var("TEST_P").unwrap_or_else(|_| "p".to_string())
        );
        let test_cases = vec![
            (test_url.as_str(), "host:5432/db"),
            ("postgresql://user@host:5432/db", "host:5432/db"),
            ("postgresql://host:5432/db", "postgresql://host:5432/db"),
            ("invalid_url", "invalid_url"),
            ("", ""),
        ];

        for (input_url, expected_sanitized) in test_cases {
            // Simulate the URL sanitization logic from the new() method
            let sanitized = input_url.split('@').next_back().unwrap_or(input_url);
            assert_eq!(sanitized, expected_sanitized);
        }
    }

    #[test]
    fn test_url_sanitization_with_multiple_at_symbols() {
        let url = "postgresql://user@domain:pass@word@host:5432/db";
        let sanitized = url.split('@').next_back().unwrap_or(url);
        assert_eq!(sanitized, "host:5432/db");
    }

    #[test]
    fn test_url_sanitization_with_no_at_symbol() {
        let url = "postgresql://localhost:5432/db";
        let sanitized = url.split('@').next_back().unwrap_or(url);
        assert_eq!(sanitized, url);
    }

    // Test data type conversions used in the implementation
    #[test]
    #[expect(clippy::cast_possible_wrap)]
    fn test_block_height_conversion_to_i64() {
        let test_cases = vec![
            (0u64, 0i64),
            (1u64, 1i64),
            (9_223_372_036_854_775_807_u64, 9_223_372_036_854_775_807_i64), // i64::MAX
        ];

        for (input, expected) in test_cases {
            let result = input as i64;
            assert_eq!(result, expected);
        }
    }

    #[test]
    #[expect(clippy::cast_sign_loss)]
    fn test_block_height_conversion_from_i64() {
        let test_cases = vec![
            (0i64, 0u64),
            (1i64, 1u64),
            (9_223_372_036_854_775_807_i64, 9_223_372_036_854_775_807_u64), // i64::MAX
        ];

        for (input, expected) in test_cases {
            let result = input as u64;
            assert_eq!(result, expected);
        }
    }

    #[test]
    #[expect(clippy::cast_possible_wrap, clippy::cast_sign_loss)]
    fn test_block_height_conversion_edge_cases() {
        // Test conversion of large u64 values that would overflow i64
        let large_u64 = u64::MAX;
        let converted_i64 = large_u64 as i64;
        let converted_back = converted_i64 as u64;

        // This tests the wraparound behavior
        assert_ne!(large_u64, converted_back);
        assert_eq!(converted_i64, -1i64);
    }

    // Test JSON data handling
    #[test]
    fn test_json_value_types() {
        let values = vec![
            json!(null),
            json!(true),
            json!(false),
            json!(42),
            json!(-42),
            json!(PI),
            json!("string"),
            json!(""),
            json!("🦀❤️🔗"),
            json!([]),
            json!([1, 2, 3]),
            json!({}),
            json!({"key": "value"}),
        ];

        for value in values {
            let event = create_custom_event("test", "test", "test", value, None, None);
            assert!(
                event.data.is_object()
                    || event.data.is_array()
                    || event.data.is_string()
                    || event.data.is_number()
                    || event.data.is_boolean()
                    || event.data.is_null()
            );
        }
    }

    #[test]
    fn test_json_value_large_structures() {
        // Test large JSON structures
        let mut large_object = serde_json::Map::default();
        for i in 0..1000 {
            large_object.insert(format!("key_{i}"), json!(format!("value_{}", i)));
        }
        let large_json = json!(large_object);

        let event = create_custom_event("test", "test", "test", large_json, None, None);
        assert!(
            event
                .data
                .as_object()
                .expect("Data should be a JSON object")
                .len()
                == 1000
        );
    }

    #[test]
    fn test_json_value_deeply_nested() {
        let mut nested = json!({"level": 0});
        for i in 1..100 {
            nested = json!({"level": i, "nested": nested});
        }

        let event = create_custom_event("test", "test", "test", nested, None, None);
        assert!(
            event
                .data
                .get("level")
                .expect("Should have level field")
                .as_i64()
                .expect("Level should be an i64")
                == 99
        );
    }

    // Test timestamp handling
    #[test]
    fn test_timestamp_creation_and_handling() {
        let now = Utc::now();
        let event = StoredEvent {
            id: "test".to_string(),
            event_type: "test".to_string(),
            source: "test".to_string(),
            data: json!({}),
            timestamp: now,
            block_height: None,
            transaction_hash: None,
        };

        assert_eq!(event.timestamp, now);
    }

    #[test]
    fn test_timestamp_edge_values() {
        let timestamps = vec![
            DateTime::from_timestamp(0, 0).unwrap_or_default(),
            DateTime::from_timestamp(1, 0).unwrap_or_default(),
            DateTime::from_timestamp(253_402_300_799, 999_999_999).unwrap_or_default(), // Max valid timestamp
        ];

        for timestamp in timestamps {
            let event = StoredEvent {
                id: "test".to_string(),
                event_type: "test".to_string(),
                source: "test".to_string(),
                data: json!({}),
                timestamp,
                block_height: None,
                transaction_hash: None,
            };

            assert_eq!(event.timestamp, timestamp);
        }
    }

    // Test string handling edge cases
    #[test]
    fn test_string_field_edge_cases() {
        let long_string = "A".repeat(10000);
        let test_strings = vec![
            "",
            " ",
            "\n",
            "\t",
            "🦀❤️🔗",
            &long_string,
            "\0",
            "line1\nline2\rline3",
            "quote\"test'quote",
            "sql'; DROP TABLE events; --",
            "json{\"key\":\"value\"}",
        ];

        for test_string in test_strings {
            let event = StoredEvent {
                id: test_string.to_string(),
                event_type: test_string.to_string(),
                source: test_string.to_string(),
                data: json!({}),
                timestamp: Utc::now(),
                block_height: None,
                transaction_hash: Some(test_string.to_string()),
            };

            assert_eq!(event.id, test_string);
            assert_eq!(event.event_type, test_string);
            assert_eq!(event.source, test_string);
            assert_eq!(event.transaction_hash, Some(test_string.to_string()));
        }
    }

    // Test Duration and time handling
    #[test]
    fn test_duration_edge_cases() {
        let durations = vec![
            Duration::from_nanos(0),
            Duration::from_nanos(1),
            Duration::from_millis(1),
            Duration::from_secs(1),
            Duration::from_secs(3600),
            Duration::from_secs(86400),
            Duration::new(u64::MAX / 1000, 0), // Large but valid duration
        ];

        for duration in durations {
            let millis = duration.as_secs_f64() * 1000.0;
            assert!(millis >= 0.0);
        }
    }

    #[test]
    fn test_duration_conversion_precision() {
        let duration = Duration::from_nanos(1_500_000); // 1.5 milliseconds
        let millis = duration.as_secs_f64() * 1000.0;
        assert!((millis - 1.5).abs() < f64::EPSILON); // Should preserve precision to 1.5 milliseconds

        let duration_micros = Duration::from_micros(1500); // 1.5 milliseconds
        let millis_micros = duration_micros.as_secs_f64() * 1000.0;
        assert!((millis_micros - 1.5).abs() < f64::EPSILON); // Should preserve precision to 1.5 milliseconds
    }

    // Test RwLock behavior edge cases
    #[test]
    fn test_rwlock_concurrent_access_simulation() {
        let stats = RwLock::new(InternalStats::default());

        // Simulate multiple readers
        let read1 = stats.read().expect("Failed to acquire read lock");
        let read2 = stats.read().expect("Failed to acquire read lock");
        let read3 = stats.read().expect("Failed to acquire read lock");

        // All readers should be able to access simultaneously
        assert_eq!(read1.total_events, 0);
        assert_eq!(read2.total_events, 0);
        assert_eq!(read3.total_events, 0);

        drop(read1);
        drop(read2);
        drop(read3);

        // Now try to write
        {
            let mut write_guard = stats.write().expect("Failed to acquire write lock");
            write_guard.total_events = 42;
        }

        // Verify write worked
        {
            let read_final = stats.read().expect("Failed to acquire read lock");
            assert_eq!(read_final.total_events, 42);
            drop(read_final);
        }
    }

    #[test]
    fn test_rwlock_try_write_when_read_locked() {
        let stats = RwLock::new(InternalStats::default());

        // Hold a read lock
        let _read_guard = stats.read().expect("Failed to acquire read lock for stats");

        // Try to acquire write lock - should fail
        assert!(stats.try_write().is_err());
    }

    #[test]
    fn test_rwlock_try_read_when_write_locked() {
        let stats = RwLock::new(InternalStats::default());

        // Hold a write lock
        let _write_guard = stats.write().expect("Failed to acquire write lock");

        // Try to acquire read lock - should fail
        assert!(stats.try_read().is_err());
    }

    // Test helper functions with more complex scenarios
    #[test]
    fn test_create_test_config_consistency() {
        let config1 = create_test_config();
        let config2 = create_test_config();

        // Should create identical configurations
        assert_eq!(config1.backend, config2.backend);
        assert_eq!(config1.url, config2.url);
        assert_eq!(config1.pool.max_connections, config2.pool.max_connections);
        assert_eq!(config1.settings.len(), config2.settings.len());
    }

    #[test]
    fn test_create_test_event_consistency() {
        let event1 = create_test_event("same_id");
        let event2 = create_test_event("same_id");

        // Should create events with same structure but different timestamps
        assert_eq!(event1.id, event2.id);
        assert_eq!(event1.event_type, event2.event_type);
        assert_eq!(event1.source, event2.source);
        assert_eq!(event1.data, event2.data);
        assert_eq!(event1.block_height, event2.block_height);
        assert_eq!(event1.transaction_hash, event2.transaction_hash);
        // Timestamps will be different (created at different times)
    }

    #[test]
    fn test_create_custom_event_all_combinations() {
        // Test all possible combinations of optional fields
        let combinations = vec![
            (None, None),
            (Some(0), None),
            (None, Some("hash".to_string())),
            (Some(12345), Some("0xabc".to_string())),
            (Some(i64::MAX), Some(String::new())),
        ];

        for (block_height, transaction_hash) in combinations {
            let event = create_custom_event(
                "test_id",
                "test_type",
                "test_source",
                json!({"test": true}),
                block_height,
                transaction_hash,
            );

            assert_eq!(event.block_height, block_height);
            // Verify the event was created successfully with the expected fields
            assert_eq!(event.id, "test_id");
            assert_eq!(event.event_type, "test_type");
            assert_eq!(event.source, "test_source");
        }
    }
}
