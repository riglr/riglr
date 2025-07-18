-- Initial schema for RIGLR Indexer
-- Migration: 001_initial_schema

-- Events table for storing all blockchain events
CREATE TABLE IF NOT EXISTS events (
    id TEXT PRIMARY KEY,
    event_type TEXT NOT NULL,
    source TEXT NOT NULL,
    data JSONB NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL,
    block_height BIGINT,
    transaction_hash TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes for performance
CREATE INDEX IF NOT EXISTS idx_events_timestamp ON events (timestamp);
CREATE INDEX IF NOT EXISTS idx_events_event_type ON events (event_type);
CREATE INDEX IF NOT EXISTS idx_events_source ON events (source);
CREATE INDEX IF NOT EXISTS idx_events_block_height ON events (block_height) WHERE block_height IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_events_transaction_hash ON events (transaction_hash) WHERE transaction_hash IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_events_data_gin ON events USING GIN (data);
CREATE INDEX IF NOT EXISTS idx_events_created_at ON events (created_at);

-- Composite indexes for common query patterns
CREATE INDEX IF NOT EXISTS idx_events_type_timestamp ON events (event_type, timestamp);
CREATE INDEX IF NOT EXISTS idx_events_source_timestamp ON events (source, timestamp);

-- Event statistics table for quick aggregations
CREATE TABLE IF NOT EXISTS event_stats (
    id SERIAL PRIMARY KEY,
    event_type TEXT NOT NULL,
    source TEXT NOT NULL,
    count BIGINT NOT NULL DEFAULT 0,
    last_updated TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(event_type, source)
);

-- Trigger to update the updated_at column
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_events_updated_at 
    BEFORE UPDATE ON events 
    FOR EACH ROW 
    EXECUTE FUNCTION update_updated_at_column();

-- Function to increment event stats
CREATE OR REPLACE FUNCTION increment_event_stats()
RETURNS TRIGGER AS $$
BEGIN
    INSERT INTO event_stats (event_type, source, count, last_updated)
    VALUES (NEW.event_type, NEW.source, 1, NOW())
    ON CONFLICT (event_type, source)
    DO UPDATE SET
        count = event_stats.count + 1,
        last_updated = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER increment_stats_on_insert
    AFTER INSERT ON events
    FOR EACH ROW
    EXECUTE FUNCTION increment_event_stats();

-- Table for tracking schema migrations
CREATE TABLE IF NOT EXISTS schema_migrations (
    version TEXT PRIMARY KEY,
    applied_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    description TEXT
);

-- Insert this migration
INSERT INTO schema_migrations (version, description) 
VALUES ('001', 'Initial schema with events table and indexes')
ON CONFLICT (version) DO NOTHING;