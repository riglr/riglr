//! PostgreSQL storage implementation

use sqlx::{PgPool, Row};
use std::time::{Duration, Instant};
use tracing::{debug, error, info, warn};

use crate::config::StorageBackendConfig;
use crate::error::{IndexerError, IndexerResult, StorageError};
use crate::storage::{DataStore, EventFilter, EventQuery, StorageStats, StoredEvent};

/// PostgreSQL storage implementation
pub struct PostgresStore {
    /// Database connection pool
    pool: PgPool,
    /// Configuration
    config: StorageBackendConfig,
    /// Statistics tracking
    stats: std::sync::RwLock<InternalStats>,
}

#[derive(Default)]
struct InternalStats {
    total_events: u64,
    total_writes: u64,
    total_reads: u64,
    write_latency_sum: f64,
    read_latency_sum: f64,
}

impl PostgresStore {
    /// Create a new PostgreSQL store
    pub async fn new(config: &StorageBackendConfig) -> IndexerResult<Self> {
        info!(
            "Connecting to PostgreSQL at {}",
            config.url.split('@').next_back().unwrap_or(&config.url)
        );

        let pool = sqlx::PgPool::connect(&config.url).await.map_err(|e| {
            IndexerError::Storage(StorageError::ConnectionFailed {
                message: e.to_string(),
            })
        })?;

        // Test connection
        sqlx::query("SELECT 1").execute(&pool).await.map_err(|e| {
            IndexerError::Storage(StorageError::ConnectionFailed {
                message: format!("Connection test failed: {}", e),
            })
        })?;

        info!("PostgreSQL connection established successfully");

        Ok(Self {
            pool,
            config: config.clone(),
            stats: std::sync::RwLock::new(InternalStats::default()),
        })
    }

    /// Build WHERE clause from filter
    fn build_where_clause(
        filter: &EventFilter,
    ) -> (
        String,
        Vec<Box<dyn sqlx::Encode<'static, sqlx::Postgres> + Send + Sync + 'static>>,
    ) {
        let mut conditions = Vec::new();
        let mut params: Vec<
            Box<dyn sqlx::Encode<'static, sqlx::Postgres> + Send + Sync + 'static>,
        > = Vec::new();
        let mut param_count = 1;

        if let Some(event_types) = &filter.event_types {
            if !event_types.is_empty() {
                conditions.push(format!("event_type = ANY(${})", param_count));
                params.push(Box::new(event_types.clone()));
                param_count += 1;
            }
        }

        if let Some(sources) = &filter.sources {
            if !sources.is_empty() {
                conditions.push(format!("source = ANY(${})", param_count));
                params.push(Box::new(sources.clone()));
                param_count += 1;
            }
        }

        if let Some((start, end)) = &filter.time_range {
            conditions.push(format!(
                "timestamp >= ${} AND timestamp <= ${}",
                param_count,
                param_count + 1
            ));
            params.push(Box::new(*start));
            params.push(Box::new(*end));
            param_count += 2;
        }

        if let Some((min_height, max_height)) = &filter.block_height_range {
            conditions.push(format!(
                "block_height >= ${} AND block_height <= ${}",
                param_count,
                param_count + 1
            ));
            params.push(Box::new(*min_height as i64));
            params.push(Box::new(*max_height as i64));
            param_count += 2;
        }

        if let Some(tx_hash) = &filter.transaction_hash {
            conditions.push(format!("transaction_hash = ${}", param_count));
            params.push(Box::new(tx_hash.clone()));
            // param_count += 1; // This would be the last parameter, so increment is not needed
        }

        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", conditions.join(" AND "))
        };

        (where_clause, params)
    }

    /// Record write latency
    fn record_write_latency(&self, duration: Duration) {
        if let Ok(mut stats) = self.stats.write() {
            stats.total_writes += 1;
            stats.write_latency_sum += duration.as_millis() as f64;
        }
    }

    /// Record read latency
    fn record_read_latency(&self, duration: Duration) {
        if let Ok(mut stats) = self.stats.write() {
            stats.total_reads += 1;
            stats.read_latency_sum += duration.as_millis() as f64;
        }
    }
}

#[async_trait::async_trait]
impl DataStore for PostgresStore {
    async fn initialize(&self) -> IndexerResult<()> {
        info!("Initializing PostgreSQL schema");

        // Create main events table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS events (
                id TEXT PRIMARY KEY,
                event_type TEXT NOT NULL,
                source TEXT NOT NULL,
                data JSONB NOT NULL,
                timestamp TIMESTAMPTZ NOT NULL,
                block_height BIGINT,
                transaction_hash TEXT,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|_e| {
            IndexerError::Storage(StorageError::QueryFailed {
                query: "CREATE TABLE events".to_string(),
            })
        })?;

        // Create indexes for better query performance
        let indexes = [
            "CREATE INDEX IF NOT EXISTS idx_events_timestamp ON events (timestamp)",
            "CREATE INDEX IF NOT EXISTS idx_events_event_type ON events (event_type)",
            "CREATE INDEX IF NOT EXISTS idx_events_source ON events (source)",
            "CREATE INDEX IF NOT EXISTS idx_events_block_height ON events (block_height) WHERE block_height IS NOT NULL",
            "CREATE INDEX IF NOT EXISTS idx_events_transaction_hash ON events (transaction_hash) WHERE transaction_hash IS NOT NULL",
            "CREATE INDEX IF NOT EXISTS idx_events_data_gin ON events USING GIN (data)",
            "CREATE INDEX IF NOT EXISTS idx_events_created_at ON events (created_at)",
        ];

        for index_sql in indexes {
            if let Err(e) = sqlx::query(index_sql).execute(&self.pool).await {
                warn!("Failed to create index: {} - {}", index_sql, e);
                // Don't fail initialization for index creation failures
            }
        }

        // Create partitioning for large datasets (optional)
        if self
            .config
            .settings
            .get("enable_partitioning")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
        {
            info!("Setting up table partitioning");

            // This would require more complex setup for production use
            // For now, just log that partitioning is requested
            info!("Partitioning setup would be implemented here for production");
        }

        info!("PostgreSQL schema initialized successfully");
        Ok(())
    }

    async fn insert_event(&self, event: &StoredEvent) -> IndexerResult<()> {
        let start_time = Instant::now();

        sqlx::query(
            r#"
            INSERT INTO events (id, event_type, source, data, timestamp, block_height, transaction_hash)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (id) DO UPDATE SET
                event_type = EXCLUDED.event_type,
                source = EXCLUDED.source,
                data = EXCLUDED.data,
                timestamp = EXCLUDED.timestamp,
                block_height = EXCLUDED.block_height,
                transaction_hash = EXCLUDED.transaction_hash
            "#,
        )
        .bind(&event.id)
        .bind(&event.event_type)
        .bind(&event.source)
        .bind(&event.data)
        .bind(event.timestamp)
        .bind(event.block_height.map(|h| h as i64))
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
            stats.total_events += 1;
        }

        debug!("Event {} inserted successfully", event.id);
        Ok(())
    }

    async fn insert_events(&self, events: &[StoredEvent]) -> IndexerResult<()> {
        if events.is_empty() {
            return Ok(());
        }

        let start_time = Instant::now();
        info!("Batch inserting {} events", events.len());

        // Use a transaction for batch insert
        let mut tx = self.pool.begin().await.map_err(|_e| {
            IndexerError::Storage(StorageError::TransactionFailed {
                operation: "begin batch insert".to_string(),
            })
        })?;

        for event in events {
            sqlx::query(
                r#"
                INSERT INTO events (id, event_type, source, data, timestamp, block_height, transaction_hash)
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                ON CONFLICT (id) DO UPDATE SET
                    event_type = EXCLUDED.event_type,
                    source = EXCLUDED.source,
                    data = EXCLUDED.data,
                    timestamp = EXCLUDED.timestamp,
                    block_height = EXCLUDED.block_height,
                    transaction_hash = EXCLUDED.transaction_hash
                "#,
            )
            .bind(&event.id)
            .bind(&event.event_type)
            .bind(&event.source)
            .bind(&event.data)
            .bind(event.timestamp)
            .bind(event.block_height.map(|h| h as i64))
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

        tx.commit().await.map_err(|e| {
            error!("Failed to commit batch insert: {}", e);
            IndexerError::Storage(StorageError::TransactionFailed {
                operation: "commit batch insert".to_string(),
            })
        })?;

        self.record_write_latency(start_time.elapsed());

        if let Ok(mut stats) = self.stats.write() {
            stats.total_events += events.len() as u64;
        }

        info!(
            "Batch insert of {} events completed in {:?}",
            events.len(),
            start_time.elapsed()
        );
        Ok(())
    }

    async fn query_events(&self, query: &EventQuery) -> IndexerResult<Vec<StoredEvent>> {
        let start_time = Instant::now();

        let (where_clause, _params) = Self::build_where_clause(&query.filter);

        // Build the complete query
        let mut sql = format!(
            r#"
            SELECT id, event_type, source, data, timestamp, block_height, transaction_hash
            FROM events
            {}
            "#,
            where_clause
        );

        // Add sorting
        if let Some((sort_field, ascending)) = &query.sort {
            let direction = if *ascending { "ASC" } else { "DESC" };
            sql.push_str(&format!(" ORDER BY {} {}", sort_field, direction));
        }

        // Add pagination
        if let Some(limit) = query.limit {
            sql.push_str(&format!(" LIMIT {}", limit));
        }
        if let Some(offset) = query.offset {
            sql.push_str(&format!(" OFFSET {}", offset));
        }

        debug!("Executing query: {}", sql);

        // For now, we'll use a simpler approach without dynamic parameters
        // In production, you'd want to properly bind the parameters
        let rows = sqlx::query(&sql).fetch_all(&self.pool).await.map_err(|e| {
            error!("Query failed: {}", e);
            IndexerError::Storage(StorageError::QueryFailed { query: sql })
        })?;

        let mut events = Vec::new();
        for row in rows {
            let event = StoredEvent {
                id: row.get("id"),
                event_type: row.get("event_type"),
                source: row.get("source"),
                data: row.get("data"),
                timestamp: row.get("timestamp"),
                block_height: row.get::<Option<i64>, _>("block_height").map(|h| h as u64),
                transaction_hash: row.get("transaction_hash"),
            };
            events.push(event);
        }

        self.record_read_latency(start_time.elapsed());

        debug!(
            "Query returned {} events in {:?}",
            events.len(),
            start_time.elapsed()
        );
        Ok(events)
    }

    async fn get_event(&self, id: &str) -> IndexerResult<Option<StoredEvent>> {
        let start_time = Instant::now();

        let row = sqlx::query(
            r#"
            SELECT id, event_type, source, data, timestamp, block_height, transaction_hash
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
                query: format!("GET event {}", id),
            })
        })?;

        self.record_read_latency(start_time.elapsed());

        if let Some(row) = row {
            Ok(Some(StoredEvent {
                id: row.get("id"),
                event_type: row.get("event_type"),
                source: row.get("source"),
                data: row.get("data"),
                timestamp: row.get("timestamp"),
                block_height: row.get::<Option<i64>, _>("block_height").map(|h| h as u64),
                transaction_hash: row.get("transaction_hash"),
            }))
        } else {
            Ok(None)
        }
    }

    async fn delete_events(&self, filter: &EventFilter) -> IndexerResult<u64> {
        let (where_clause, _params) = Self::build_where_clause(filter);

        if where_clause.is_empty() {
            return Err(IndexerError::validation(
                "Cannot delete all events without filter",
            ));
        }

        let sql = format!("DELETE FROM events {}", where_clause);

        let result = sqlx::query(&sql).execute(&self.pool).await.map_err(|e| {
            error!("Delete failed: {}", e);
            IndexerError::Storage(StorageError::QueryFailed { query: sql })
        })?;

        Ok(result.rows_affected())
    }

    async fn count_events(&self, filter: &EventFilter) -> IndexerResult<u64> {
        let (where_clause, _params) = Self::build_where_clause(filter);

        let sql = format!("SELECT COUNT(*) as count FROM events {}", where_clause);

        let row = sqlx::query(&sql).fetch_one(&self.pool).await.map_err(|e| {
            error!("Count query failed: {}", e);
            IndexerError::Storage(StorageError::QueryFailed { query: sql })
        })?;

        let count: i64 = row.get("count");
        Ok(count as u64)
    }

    async fn get_stats(&self) -> IndexerResult<StorageStats> {
        // Get database stats
        let size_row = sqlx::query(
            r#"
            SELECT pg_total_relation_size('events') as size_bytes,
                   COUNT(*) as total_events
            FROM events
            "#,
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

        // Get connection stats
        let pool_state = self.pool.options().get_max_connections();

        let stats = self
            .stats
            .read()
            .map_err(|_| IndexerError::internal("Failed to read stats"))?;

        let avg_write_latency_ms = if stats.total_writes > 0 {
            stats.write_latency_sum / stats.total_writes as f64
        } else {
            0.0
        };

        let avg_read_latency_ms = if stats.total_reads > 0 {
            stats.read_latency_sum / stats.total_reads as f64
        } else {
            0.0
        };

        Ok(StorageStats {
            total_events: total_events as u64,
            storage_size_bytes: storage_size_bytes as u64,
            avg_write_latency_ms,
            avg_read_latency_ms,
            active_connections: pool_state,
            cache_hit_rate: 0.0, // Not applicable for direct PostgreSQL access
        })
    }

    async fn health_check(&self) -> IndexerResult<()> {
        sqlx::query("SELECT 1")
            .execute(&self.pool)
            .await
            .map_err(|e| {
                IndexerError::Storage(StorageError::ConnectionFailed {
                    message: format!("Health check failed: {}", e),
                })
            })?;

        Ok(())
    }
}
