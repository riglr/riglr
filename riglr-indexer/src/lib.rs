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
//! url = "postgresql://user:pass@localhost/riglr_indexer"
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

#![warn(missing_docs)]
#![warn(clippy::all)]
#![allow(clippy::module_inception)]

pub mod config;
pub mod core;
pub mod storage;
pub mod api;
pub mod metrics;
pub mod error;
pub mod utils;

/// Prelude module with commonly used types and traits
pub mod prelude {
    pub use crate::config::{IndexerConfig, StorageConfig, ApiConfig, MetricsConfig};
    pub use crate::core::{IndexerService, EventIngester, EventProcessor, ProcessingPipeline};
    pub use crate::storage::{DataStore, PostgresStore, EventQuery, EventFilter};
    pub use crate::api::{ApiServer, WebSocketStreamer, RestHandler};
    pub use crate::metrics::{MetricsCollector, IndexerMetrics, PerformanceMetrics};
    pub use crate::error::{IndexerError, IndexerResult};
    pub use crate::utils::{ConsistentHash, BatchProcessor, HealthCheck};
    
    // Re-export key dependencies
    pub use riglr_events_core::prelude::*;
    pub use riglr_solana_events::prelude::*;
    pub use riglr_streams::prelude::*;
    
    // Re-export common types
    pub use tokio;
    pub use anyhow;
    pub use serde_json;
    pub use uuid::Uuid;
    pub use chrono::{DateTime, Utc};
}

// Re-export key types at crate root
pub use config::IndexerConfig;
pub use core::IndexerService;
pub use error::{IndexerError, IndexerResult};
pub use storage::DataStore;