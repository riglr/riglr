//! # riglr-events-core
//!
//! Core event processing abstractions and traits for riglr blockchain agents.
//!
//! This crate provides the foundational types and traits for building event-driven
//! blockchain applications. It is designed to be blockchain-agnostic while providing
//! specific implementations for major blockchains.
//!
//! ## Core Concepts
//!
//! - **Events**: Structured data representing blockchain or external system events
//! - **Parsers**: Components that extract events from raw data
//! - **Streams**: Async streams of events from various sources
//! - **Filters**: Components that route and filter events based on criteria
//! - **Handlers**: Components that process events and take actions
//!
//! ## Design Principles
//!
//! - **Zero-copy parsing**: Minimize allocations for high-performance event processing
//! - **Async-first**: All operations are async with proper Send/Sync bounds
//! - **Extensible**: Easy to add support for new blockchains and event types
//! - **Type-safe**: Leverage Rust's type system to prevent runtime errors
//! - **Error-rich**: Comprehensive error handling with context preservation
//!
//! ## Usage
//!
//! ```rust
//! use riglr_events_core::prelude::*;
//! use tokio_stream::StreamExt;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), EventError> {
//!     // Create a simple event
//!     let event = GenericEvent::new(
//!         "test-event".into(),
//!         EventKind::Transaction,
//!         serde_json::json!({"value": 42}),
//!     );
//!
//!     // Process the event
//!     println!("Event: {}", event.id());
//!     Ok(())
//! }
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]
#![allow(clippy::module_inception)]

pub mod error;
pub mod parser;
pub mod traits;
pub mod types;
pub mod utils;

/// Prelude module with commonly used types and traits
pub mod prelude {
    pub use crate::error::*;
    pub use crate::parser::*;
    pub use crate::traits::*;
    pub use crate::types::*;
    pub use crate::utils::{
        EventBatchStream, EventBatcher, EventDeduplicator, EventIdGenerator,
        EventPerformanceMetrics, EventStream, MetricsSummary, RateLimiter, StreamUtils,
    };
}

// Re-export key types at crate root for convenience
pub use error::{EventError, EventResult};
pub use traits::{Event, EventFilter, EventHandler, EventParser};
pub use types::{EventKind, EventMetadata, GenericEvent, StreamInfo};
pub use utils::{EventBatchStream, EventStream};
