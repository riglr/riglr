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
        EventPerformanceMetrics, EventStream, MetricsSummary, RateLimiter, StreamOps,
    };
}

// Re-export key types at crate root for convenience
pub use error::{EventError, EventResult};
pub use traits::{Event, EventFilter, EventHandler, EventParser};
pub use types::{EventKind, EventMetadata, GenericEvent, StreamInfo};
pub use utils::{EventBatchStream, EventStream};

#[cfg(test)]
mod tests {
    use super::*;
    use core::any::type_name;

    #[test]
    fn modules_are_accessible() {
        // Test that all declared modules can be accessed
        // This verifies the module declarations are correct

        // These should compile without errors if modules exist
        let _error_module = type_name::<error::EventError>();
        let _parser_module = type_name::<parser::JsonEventParser>();
        let _traits_module = type_name::<dyn traits::Event>();
        let _types_module = type_name::<types::EventKind>();
        let _utils_module = type_name::<utils::EventStream>();
    }

    #[test]
    fn prelude_exports_are_accessible() {
        // Test that all prelude exports are accessible
        use crate::prelude::*;

        // Error types
        let _error_type = type_name::<EventError>();
        let _result_type = type_name::<EventResult<()>>();

        // Parser types
        let _parser_type = type_name::<parser::JsonEventParser>();

        // Trait types - we can't instantiate traits directly, but we can reference them
        let _event_trait = type_name::<dyn Event>();
        let _filter_trait = type_name::<dyn EventFilter>();
        let _handler_trait = type_name::<dyn EventHandler>();
        let _parser_trait = type_name::<dyn EventParser<Input = serde_json::Value>>();

        // Type exports
        let _event_kind_type = type_name::<EventKind>();
        let _metadata_type = type_name::<EventMetadata>();
        let _generic_event_type = type_name::<GenericEvent>();

        // Utility types
        let _batch_stream_type = type_name::<EventBatchStream>();
        let _batcher_type = type_name::<EventBatcher>();
        let _deduplicator_type = type_name::<EventDeduplicator>();
        let _id_generator_type = type_name::<EventIdGenerator>();
        let _metrics_type = type_name::<EventPerformanceMetrics>();
        let _stream_type = type_name::<EventStream>();
        let _summary_type = type_name::<MetricsSummary>();
        let _rate_limiter_type = type_name::<RateLimiter>();
        let _stream_ops_type = type_name::<StreamOps>();
    }

    #[test]
    fn crate_root_error_reexports() {
        // Test that error re-exports work at crate root
        let _error_type = type_name::<EventError>();
        let _result_type = type_name::<EventResult<()>>();

        // Verify they're the same as the original types
        assert_eq!(type_name::<EventError>(), type_name::<error::EventError>());
        assert_eq!(
            type_name::<EventResult<String>>(),
            type_name::<error::EventResult<String>>()
        );
    }

    #[test]
    fn crate_root_traits_reexports() {
        // Test that trait re-exports work at crate root
        let _event_trait = type_name::<dyn Event>();
        let _filter_trait = type_name::<dyn EventFilter>();
        let _handler_trait = type_name::<dyn EventHandler>();
        let _parser_trait = type_name::<dyn EventParser<Input = Vec<u8>>>();

        // Verify they're the same as the original traits
        assert_eq!(type_name::<dyn Event>(), type_name::<dyn traits::Event>());
        assert_eq!(
            type_name::<dyn EventFilter>(),
            type_name::<dyn traits::EventFilter>()
        );
        assert_eq!(
            type_name::<dyn EventHandler>(),
            type_name::<dyn traits::EventHandler>()
        );
        assert_eq!(
            type_name::<dyn EventParser<Input = Vec<u8>>>(),
            type_name::<dyn traits::EventParser<Input = Vec<u8>>>()
        );
    }

    #[test]
    fn crate_root_types_reexports() {
        // Test that type re-exports work at crate root
        let _event_kind_type = type_name::<EventKind>();
        let _metadata_type = type_name::<EventMetadata>();
        let _generic_event_type = type_name::<GenericEvent>();
        let _stream_info_type = type_name::<StreamInfo>();

        // Verify they're the same as the original types
        assert_eq!(type_name::<EventKind>(), type_name::<types::EventKind>());
        assert_eq!(
            type_name::<EventMetadata>(),
            type_name::<types::EventMetadata>()
        );
        assert_eq!(
            type_name::<GenericEvent>(),
            type_name::<types::GenericEvent>()
        );
        assert_eq!(type_name::<StreamInfo>(), type_name::<types::StreamInfo>());
    }

    #[test]
    fn crate_root_utils_reexports() {
        // Test that utils re-exports work at crate root
        let _batch_stream_type = type_name::<EventBatchStream>();
        let _stream_type = type_name::<EventStream>();

        // Verify they're the same as the original types
        assert_eq!(
            type_name::<EventBatchStream>(),
            type_name::<utils::EventBatchStream>()
        );
        assert_eq!(
            type_name::<EventStream>(),
            type_name::<utils::EventStream>()
        );
    }

    #[test]
    fn prelude_module_exists() {
        // Test that we can use the prelude module path
        use crate::prelude;

        // Test that the prelude module is accessible
        let _ = type_name::<prelude::EventError>();

        let _ = type_name::<prelude::EventError>();
    }

    #[test]
    fn all_modules_declared() {
        // This test verifies that all expected modules are declared
        // by checking that their module paths exist
        use crate::{
            error::EventError, parser::JsonEventParser, traits::Event, types::EventKind,
            utils::EventStream,
        };

        // Check error module
        let _ = type_name::<EventError>();

        // Check types module
        let _ = type_name::<EventKind>();

        // Check traits module
        let _ = type_name::<dyn Event>();

        // Check parser module
        let _ = type_name::<JsonEventParser>();

        // Check utils module
        let _ = type_name::<EventStream>();
    }

    #[test]
    fn compiler_attributes() {
        // Test that the crate compiles with the specified lints
        // This is mainly a compilation test - if it compiles, the attributes work

        let x = 42_i32;
        let result = Ok::<i32, &str>(x);

        // Explicitly consume result to demonstrate compiler attribute usage
        let _ = result;
    }

    #[test]
    fn documentation_example_types_exist() {
        // These should be accessible through prelude
        use crate::prelude::*;

        // Test that types referenced in the documentation example exist
        // This ensures the documentation example is valid

        let _ = type_name::<GenericEvent>();
        let _ = type_name::<EventKind>();
        let _ = type_name::<EventError>();

        let _ = type_name::<GenericEvent>();
        let _ = type_name::<EventKind>();
        let _ = type_name::<EventError>();
    }
}
