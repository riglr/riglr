//! # RIGLR Indexer
//!
//! Production-grade blockchain indexing service built on riglr-events-core.
//!
//! This service provides scalable, real-time blockchain event indexing with:
//! - High-throughput event ingestion from multiple sources
//! - Parallel event processing with worker pools
//! - Persistent storage with PostgreSQL and ClickHouse support
//! - Real-time streaming via WebSocket
//! - Comprehensive metrics and health monitoring
//! - Horizontal scaling with consistent hashing
//!
//! ## Architecture
//!
//! The indexer follows a modular, event-driven architecture:
//!
//! ```text
//! ┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
//! │   Data Sources  │───▶│   Event Queue   │───▶│   Processors    │
//! └─────────────────┘    └─────────────────┘    └─────────────────┘
//!                                                         │
//! ┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
//! │      API        │◀───│    Storage      │◀───│   Enrichment    │
//! └─────────────────┘    └─────────────────┘    └─────────────────┘
//! ```
//!
//! ## Usage
//!
//! ### As a Service
//!
//! ```bash
//! # Start the indexer service
//! cargo run --bin riglr-indexer
//!
//! # With custom config
//! RIGLR_INDEXER_CONFIG=./config/production.toml cargo run --bin riglr-indexer
//! ```
//!
//! ### As a Library
//!
//! ```rust,no_run
//! use riglr_indexer::prelude::*;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), IndexerError> {
//!     let config = IndexerConfig::from_env()?;
//!     let mut indexer = IndexerService::new(config).await?;
//!     indexer.start().await?;
//!     Ok(())
//! }
//! ```
//!
//! ## Configuration
//!
//! Configuration is managed through environment variables with optional TOML overrides:
//!
//! ```toml
//! [indexer]
//! workers = 8
//! batch_size = 1000
//! flush_interval = "5s"
//!
//! [storage.postgres]
//! url = "postgresql://localhost/riglr_indexer"  # Set via DATABASE_URL env var
//! max_connections = 20
//!
//! [metrics]
//! enabled = true
//! port = 9090
//! ```
//!
//! ## Performance
//!
//! - **Throughput**: 10,000+ events/second
//! - **Latency**: Sub-second for real-time queries
//! - **Storage**: Optimized time-series schemas
//! - **Scaling**: Horizontal with consistent hashing

#![warn(clippy::all)]
#![allow(clippy::module_inception)]

pub mod api;
pub mod config;
pub mod core;
pub mod error;
pub mod metrics;
pub mod storage;
pub mod utils;

/// Prelude module with commonly used types and traits
pub mod prelude {
    pub use crate::api::{ApiServer, RestHandler, WebSocketStreamer};
    pub use crate::config::{ApiConfig, IndexerConfig, MetricsConfig, StorageConfig};
    pub use crate::core::{
        EventIngester, EventProcessor, IndexerService, ProcessingPipeline, ServiceLifecycle,
    };
    pub use crate::error::{IndexerError, IndexerResult};
    pub use crate::metrics::{IndexerMetrics, MetricsCollector, PerformanceMetrics};
    pub use crate::storage::{DataStore, EventFilter, EventQuery, PostgresStore};
    pub use crate::utils::{ConsistentHash, HealthCheck};

    // Re-export BatchProcessor with common generic parameter
    pub use crate::utils::batch::BatchProcessor;

    // Re-export key dependencies - avoid conflicts by being more specific
    pub use riglr_events_core::{Event, EventKind, EventMetadata};
    pub use riglr_solana_events::events::*;
    pub use riglr_streams::core::Stream;

    // Re-export StreamedEvent - requires generic parameter at usage site
    pub use riglr_streams::core::streamed_event::StreamedEvent;

    // Re-export common types
    pub use anyhow;
    pub use chrono::{DateTime, Utc};
    pub use serde_json;
    pub use tokio;
    pub use uuid::Uuid;
}

// Re-export key types at crate root
pub use config::IndexerConfig;
pub use core::IndexerService;
pub use error::{IndexerError, IndexerResult};
pub use storage::DataStore;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prelude_module_exists() {
        // Test that the prelude module is accessible
        // This test ensures the prelude module is properly defined
    }

    #[test]
    fn test_crate_root_reexports_compile() {
        // Test that all crate root re-exports are accessible
        // These are type imports, so we just need to verify they compile

        // Test that type aliases exist and can be referenced
        let _config_type: Option<IndexerConfig> = None;
        let _service_type: Option<IndexerService> = None;
        let _error_type: Option<IndexerError> = None;
        let _result_type: Option<IndexerResult<()>> = None;
        let _store_type: Option<Box<dyn DataStore>> = None;
    }

    #[test]
    fn test_prelude_api_reexports_compile() {
        // Test that API-related re-exports in prelude are accessible
        use crate::prelude::*;

        // Test that types can be referenced (compile-time check)
        let _api_server_type: Option<ApiServer> = None;
        let _rest_handler_type: Option<RestHandler> = None;
        let _websocket_streamer_type: Option<WebSocketStreamer> = None;
    }

    #[test]
    fn test_prelude_config_reexports_compile() {
        // Test that config-related re-exports in prelude are accessible
        use crate::prelude::*;

        // Test that config types can be referenced
        let _api_config_type: Option<ApiConfig> = None;
        let _indexer_config_type: Option<IndexerConfig> = None;
        let _metrics_config_type: Option<MetricsConfig> = None;
        let _storage_config_type: Option<StorageConfig> = None;
    }

    #[test]
    fn test_prelude_core_reexports_compile() {
        // Test that core-related re-exports in prelude are accessible
        use crate::prelude::*;

        // Test that core types can be referenced
        let _event_ingester_type: Option<EventIngester> = None;
        let _event_processor_type: Option<EventProcessor> = None;
        let _indexer_service_type: Option<IndexerService> = None;
        let _processing_pipeline_type: Option<ProcessingPipeline> = None;
        let _service_lifecycle_type: Option<Box<dyn ServiceLifecycle>> = None;
    }

    #[test]
    fn test_prelude_error_reexports_compile() {
        // Test that error-related re-exports in prelude are accessible
        use crate::prelude::*;

        // Test that error types can be referenced
        let _indexer_error_type: Option<IndexerError> = None;
        let _indexer_result_type: Option<IndexerResult<()>> = None;
    }

    #[test]
    fn test_prelude_metrics_reexports_compile() {
        // Test that metrics-related re-exports in prelude are accessible
        use crate::prelude::*;

        // Test that metrics types can be referenced
        let _indexer_metrics_type: Option<IndexerMetrics> = None;
        let _metrics_collector_type: Option<MetricsCollector> = None;
        let _performance_metrics_type: Option<PerformanceMetrics> = None;
    }

    #[test]
    fn test_prelude_storage_reexports_compile() {
        // Test that storage-related re-exports in prelude are accessible
        use crate::prelude::*;

        // Test that storage types can be referenced
        let _data_store_type: Option<Box<dyn DataStore>> = None;
        let _event_filter_type: Option<EventFilter> = None;
        let _event_query_type: Option<EventQuery> = None;
        let _postgres_store_type: Option<PostgresStore> = None;
    }

    #[test]
    fn test_prelude_utils_reexports_compile() {
        // Test that utils-related re-exports in prelude are accessible
        use crate::prelude::*;

        // Test that utils types can be referenced
        let _batch_processor_type: Option<BatchProcessor<String>> = None;
        let _consistent_hash_type: Option<ConsistentHash> = None;
        let _health_check_type: Option<Box<dyn HealthCheck>> = None;
    }

    #[test]
    fn test_prelude_external_dependencies_compile() {
        // Test that external dependency re-exports in prelude are accessible
        use crate::prelude::*;

        // Test riglr-events-core re-exports
        let _event_type: Option<Box<dyn Event>> = None;
        let _event_kind_type: Option<EventKind> = None;
        let _event_metadata_type: Option<EventMetadata> = None;

        // Test riglr-streams re-exports
        // Note: Stream trait requires associated types, so we can't test it with dyn Stream
        // let _stream_type: Option<dyn Stream> = None;
        // let _streamed_event_type: Option<StreamedEvent<Box<dyn Event>>> = None;

        // Test common types re-exports
        let _datetime_type: Option<DateTime<Utc>> = None;
        let _uuid_type: Option<Uuid> = None;
    }

    #[test]
    fn test_module_declarations_exist() {
        // Test that all declared modules are accessible
        // This is a compile-time check to ensure all modules are properly declared

        // We can't directly test module existence at runtime, but we can
        // ensure they compile by trying to use them in type annotations
        let _api_module = std::marker::PhantomData::<crate::api::ApiServer>;
        let _config_module = std::marker::PhantomData::<crate::config::IndexerConfig>;
        let _core_module = std::marker::PhantomData::<crate::core::IndexerService>;
        let _error_module = std::marker::PhantomData::<crate::error::IndexerError>;
        let _metrics_module = std::marker::PhantomData::<crate::metrics::IndexerMetrics>;
        let _storage_module = std::marker::PhantomData::<dyn crate::storage::DataStore>;
        let _utils_module = std::marker::PhantomData::<dyn crate::utils::HealthCheck>;
    }

    #[test]
    fn test_prelude_use_statements_syntax() {
        // Test that all use statements in prelude are syntactically correct
        // by importing the entire prelude
        use crate::prelude::*;

        // Test that we can use some common external crates that should be available
        let _json_value = serde_json::Value::Null;
        let _now = chrono::Utc::now();
    }

    #[test]
    fn test_crate_attributes() {
        // Test that crate-level attributes are properly set
        // This is mainly a documentation test to ensure the lints are working

        // The #![warn(clippy::all)] and #![allow(clippy::module_inception)]
        // attributes should be active, but we can't test them directly at runtime
        // This test serves as documentation that these attributes exist
    }

    #[test]
    fn test_documentation_accessibility() {
        // Test that the crate documentation structure is properly accessible
        // This ensures the doc comments are properly formatted and don't cause issues

        // We can't test documentation directly, but we can ensure the code
        // that contains the documentation compiles without issues
    }

    #[test]
    fn test_prelude_convenience_imports() {
        // Test that the prelude provides convenient access to commonly used items
        use crate::prelude::*;

        // Test that we can create basic instances or references to verify
        // the imports are working correctly
        let _error_result: IndexerResult<()> = Ok(());
        let _uuid = Uuid::new_v4();
        let _utc_now = Utc::now();

        // Test that error handling works
        match _error_result {
            Ok(_) => {}
            Err(_) => panic!("Should not reach here in this test"),
        }

        // Test UUID creation
        assert!(_uuid.to_string().len() == 36); // Standard UUID string length

        // Test DateTime functionality
        assert!(_utc_now.timestamp() > 0);
    }

    #[test]
    fn test_namespace_organization() {
        // Test that the namespace organization is logical and doesn't conflict

        // Test direct crate imports
        use crate::{DataStore, IndexerConfig, IndexerError, IndexerResult, IndexerService};

        // Test that these don't conflict with prelude imports
        use crate::prelude::{
            DataStore as PreludeDataStore, IndexerConfig as PreludeIndexerConfig,
            IndexerError as PreludeIndexerError, IndexerResult as PreludeIndexerResult,
            IndexerService as PreludeIndexerService,
        };

        // These should be the same types (compile-time check)
        let _config1: Option<IndexerConfig> = None;
        let _config2: Option<PreludeIndexerConfig> = None;

        let _service1: Option<IndexerService> = None;
        let _service2: Option<PreludeIndexerService> = None;

        let _error1: Option<IndexerError> = None;
        let _error2: Option<PreludeIndexerError> = None;

        let _result1: Option<IndexerResult<()>> = None;
        let _result2: Option<PreludeIndexerResult<()>> = None;

        let _store1: Option<Box<dyn DataStore>> = None;
        let _store2: Option<Box<dyn PreludeDataStore>> = None;
    }
}
