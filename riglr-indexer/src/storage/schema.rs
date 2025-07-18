//! Database schema definitions and migrations

/// SQL schema for PostgreSQL
pub const POSTGRES_SCHEMA: &str = r#"
-- Main events table
CREATE TABLE IF NOT EXISTS events (
    id TEXT PRIMARY KEY,
    event_type TEXT NOT NULL,
    source TEXT NOT NULL,
    data JSONB NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL,
    block_height BIGINT,
    transaction_hash TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_events_timestamp ON events (timestamp);
CREATE INDEX IF NOT EXISTS idx_events_event_type ON events (event_type);
CREATE INDEX IF NOT EXISTS idx_events_source ON events (source);
CREATE INDEX IF NOT EXISTS idx_events_block_height ON events (block_height) WHERE block_height IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_events_transaction_hash ON events (transaction_hash) WHERE transaction_hash IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_events_data_gin ON events USING GIN (data);
CREATE INDEX IF NOT EXISTS idx_events_created_at ON events (created_at);

-- Event statistics table
CREATE TABLE IF NOT EXISTS event_stats (
    id SERIAL PRIMARY KEY,
    event_type TEXT NOT NULL,
    source TEXT NOT NULL,
    count BIGINT NOT NULL DEFAULT 0,
    last_updated TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(event_type, source)
);
"#;

/// Migration version tracking
pub const VERSION_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS schema_migrations (
    version TEXT PRIMARY KEY,
    applied_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
"#;