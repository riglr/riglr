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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_postgres_schema_when_accessed_should_contain_events_table() {
        // Happy Path: Verify the main events table is defined
        assert!(POSTGRES_SCHEMA.contains("CREATE TABLE IF NOT EXISTS events"));
        assert!(POSTGRES_SCHEMA.contains("id TEXT PRIMARY KEY"));
        assert!(POSTGRES_SCHEMA.contains("event_type TEXT NOT NULL"));
        assert!(POSTGRES_SCHEMA.contains("source TEXT NOT NULL"));
        assert!(POSTGRES_SCHEMA.contains("data JSONB NOT NULL"));
        assert!(POSTGRES_SCHEMA.contains("timestamp TIMESTAMPTZ NOT NULL"));
        assert!(POSTGRES_SCHEMA.contains("block_height BIGINT"));
        assert!(POSTGRES_SCHEMA.contains("transaction_hash TEXT"));
        assert!(POSTGRES_SCHEMA.contains("created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()"));
    }

    #[test]
    fn test_postgres_schema_when_accessed_should_contain_event_stats_table() {
        // Happy Path: Verify the event_stats table is defined
        assert!(POSTGRES_SCHEMA.contains("CREATE TABLE IF NOT EXISTS event_stats"));
        assert!(POSTGRES_SCHEMA.contains("id SERIAL PRIMARY KEY"));
        assert!(POSTGRES_SCHEMA.contains("count BIGINT NOT NULL DEFAULT 0"));
        assert!(POSTGRES_SCHEMA.contains("last_updated TIMESTAMPTZ NOT NULL DEFAULT NOW()"));
        assert!(POSTGRES_SCHEMA.contains("UNIQUE(event_type, source)"));
    }

    #[test]
    fn test_postgres_schema_when_accessed_should_contain_performance_indexes() {
        // Happy Path: Verify all performance indexes are defined
        assert!(POSTGRES_SCHEMA
            .contains("CREATE INDEX IF NOT EXISTS idx_events_timestamp ON events (timestamp)"));
        assert!(POSTGRES_SCHEMA
            .contains("CREATE INDEX IF NOT EXISTS idx_events_event_type ON events (event_type)"));
        assert!(POSTGRES_SCHEMA
            .contains("CREATE INDEX IF NOT EXISTS idx_events_source ON events (source)"));
        assert!(POSTGRES_SCHEMA.contains("CREATE INDEX IF NOT EXISTS idx_events_block_height ON events (block_height) WHERE block_height IS NOT NULL"));
        assert!(POSTGRES_SCHEMA.contains("CREATE INDEX IF NOT EXISTS idx_events_transaction_hash ON events (transaction_hash) WHERE transaction_hash IS NOT NULL"));
        assert!(POSTGRES_SCHEMA
            .contains("CREATE INDEX IF NOT EXISTS idx_events_data_gin ON events USING GIN (data)"));
        assert!(POSTGRES_SCHEMA
            .contains("CREATE INDEX IF NOT EXISTS idx_events_created_at ON events (created_at)"));
    }

    #[test]
    fn test_postgres_schema_when_accessed_should_not_be_empty() {
        // Edge Case: Verify the schema constant is not empty
        assert!(!POSTGRES_SCHEMA.is_empty());
        assert!(POSTGRES_SCHEMA.len() > 100); // Should be a substantial SQL schema
    }

    #[test]
    fn test_postgres_schema_when_accessed_should_contain_valid_sql_syntax() {
        // Happy Path: Verify SQL keywords and syntax are present
        assert!(POSTGRES_SCHEMA.contains("CREATE TABLE"));
        assert!(POSTGRES_SCHEMA.contains("CREATE INDEX"));
        assert!(POSTGRES_SCHEMA.contains("IF NOT EXISTS"));
        assert!(POSTGRES_SCHEMA.contains("PRIMARY KEY"));
        assert!(POSTGRES_SCHEMA.contains("NOT NULL"));
        assert!(POSTGRES_SCHEMA.contains("DEFAULT"));
    }

    #[test]
    fn test_version_table_when_accessed_should_contain_migrations_table() {
        // Happy Path: Verify the migration table is defined
        assert!(VERSION_TABLE.contains("CREATE TABLE IF NOT EXISTS schema_migrations"));
        assert!(VERSION_TABLE.contains("version TEXT PRIMARY KEY"));
        assert!(VERSION_TABLE.contains("applied_at TIMESTAMPTZ NOT NULL DEFAULT NOW()"));
    }

    #[test]
    fn test_version_table_when_accessed_should_not_be_empty() {
        // Edge Case: Verify the version table constant is not empty
        assert!(!VERSION_TABLE.is_empty());
        assert!(VERSION_TABLE.len() > 50); // Should contain meaningful SQL
    }

    #[test]
    fn test_version_table_when_accessed_should_contain_valid_sql_syntax() {
        // Happy Path: Verify SQL keywords are present
        assert!(VERSION_TABLE.contains("CREATE TABLE"));
        assert!(VERSION_TABLE.contains("IF NOT EXISTS"));
        assert!(VERSION_TABLE.contains("PRIMARY KEY"));
        assert!(VERSION_TABLE.contains("NOT NULL"));
        assert!(VERSION_TABLE.contains("DEFAULT"));
        assert!(VERSION_TABLE.contains("NOW()"));
    }

    #[test]
    fn test_postgres_schema_when_accessed_should_have_consistent_formatting() {
        // Edge Case: Verify consistent SQL formatting
        let line_count = POSTGRES_SCHEMA.lines().count();
        assert!(line_count > 20); // Should have multiple lines

        // Verify it ends with proper formatting
        assert!(POSTGRES_SCHEMA.ends_with("\"#"));
    }

    #[test]
    fn test_version_table_when_accessed_should_have_consistent_formatting() {
        // Edge Case: Verify consistent SQL formatting
        let line_count = VERSION_TABLE.lines().count();
        assert!(line_count > 3); // Should have multiple lines

        // Verify it ends with proper formatting
        assert!(VERSION_TABLE.ends_with("\"#"));
    }

    #[test]
    fn test_postgres_schema_when_accessed_should_contain_jsonb_data_type() {
        // Happy Path: Verify JSONB data type is used for flexible data storage
        assert!(POSTGRES_SCHEMA.contains("JSONB"));
        assert!(POSTGRES_SCHEMA.contains("data JSONB NOT NULL"));
    }

    #[test]
    fn test_postgres_schema_when_accessed_should_contain_timestamptz_fields() {
        // Happy Path: Verify timezone-aware timestamps are used
        assert!(POSTGRES_SCHEMA.contains("TIMESTAMPTZ"));
        let timestamptz_count = POSTGRES_SCHEMA.matches("TIMESTAMPTZ").count();
        assert!(timestamptz_count >= 3); // Should have multiple timestamp fields
    }

    #[test]
    fn test_postgres_schema_when_accessed_should_contain_gin_index() {
        // Happy Path: Verify GIN index for JSONB performance
        assert!(POSTGRES_SCHEMA.contains("USING GIN"));
        assert!(POSTGRES_SCHEMA.contains("idx_events_data_gin"));
    }

    #[test]
    fn test_postgres_schema_when_accessed_should_contain_conditional_indexes() {
        // Happy Path: Verify conditional indexes with WHERE clauses
        assert!(POSTGRES_SCHEMA.contains("WHERE block_height IS NOT NULL"));
        assert!(POSTGRES_SCHEMA.contains("WHERE transaction_hash IS NOT NULL"));
    }

    #[test]
    fn test_all_constants_when_accessed_should_be_valid_string_slices() {
        // Edge Case: Verify constants are valid string references
        let _postgres_ref: &str = POSTGRES_SCHEMA;
        let _version_ref: &str = VERSION_TABLE;

        // Test that they can be used in string operations
        assert_eq!(POSTGRES_SCHEMA, POSTGRES_SCHEMA);
        assert_eq!(VERSION_TABLE, VERSION_TABLE);
        assert_ne!(POSTGRES_SCHEMA, VERSION_TABLE);
    }
}
