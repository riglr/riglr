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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_modules_are_accessible() {
        // Test that all declared modules can be accessed
        // This verifies the module declarations are correct

        // These should compile without errors if modules exist
        let _error_module = std::any::type_name::<error::EventError>();
        let _parser_module = std::any::type_name::<parser::JsonEventParser>();
        let _traits_module = std::any::type_name::<dyn traits::Event>();
        let _types_module = std::any::type_name::<types::EventKind>();
        let _utils_module = std::any::type_name::<utils::EventStream>();
    }

    #[test]
    fn test_prelude_exports_are_accessible() {
        // Test that all prelude exports are accessible
        use crate::prelude::*;

        // Error types
        let _error_type = std::any::type_name::<EventError>();
        let _result_type = std::any::type_name::<EventResult<()>>();

        // Parser types
        let _parser_type = std::any::type_name::<parser::JsonEventParser>();

        // Trait types - we can't instantiate traits directly, but we can reference them
        let _event_trait = std::any::type_name::<dyn Event>();
        let _filter_trait = std::any::type_name::<dyn EventFilter>();
        let _handler_trait = std::any::type_name::<dyn EventHandler>();
        let _parser_trait = std::any::type_name::<dyn EventParser<Input = serde_json::Value>>();

        // Type exports
        let _event_kind_type = std::any::type_name::<EventKind>();
        let _metadata_type = std::any::type_name::<EventMetadata>();
        let _generic_event_type = std::any::type_name::<GenericEvent>();

        // Utility types
        let _batch_stream_type = std::any::type_name::<EventBatchStream>();
        let _batcher_type = std::any::type_name::<EventBatcher>();
        let _deduplicator_type = std::any::type_name::<EventDeduplicator>();
        let _id_generator_type = std::any::type_name::<EventIdGenerator>();
        let _metrics_type = std::any::type_name::<EventPerformanceMetrics>();
        let _stream_type = std::any::type_name::<EventStream>();
        let _summary_type = std::any::type_name::<MetricsSummary>();
        let _rate_limiter_type = std::any::type_name::<RateLimiter>();
        let _stream_utils_type = std::any::type_name::<StreamUtils>();
    }

    #[test]
    fn test_crate_root_error_reexports() {
        // Test that error re-exports work at crate root
        let _error_type = std::any::type_name::<EventError>();
        let _result_type = std::any::type_name::<EventResult<()>>();

        // Verify they're the same as the original types
        assert_eq!(
            std::any::type_name::<EventError>(),
            std::any::type_name::<error::EventError>()
        );
        assert_eq!(
            std::any::type_name::<EventResult<String>>(),
            std::any::type_name::<error::EventResult<String>>()
        );
    }

    #[test]
    fn test_crate_root_traits_reexports() {
        // Test that trait re-exports work at crate root
        let _event_trait = std::any::type_name::<dyn Event>();
        let _filter_trait = std::any::type_name::<dyn EventFilter>();
        let _handler_trait = std::any::type_name::<dyn EventHandler>();
        let _parser_trait = std::any::type_name::<dyn EventParser<Input = Vec<u8>>>();

        // Verify they're the same as the original traits
        assert_eq!(
            std::any::type_name::<dyn Event>(),
            std::any::type_name::<dyn traits::Event>()
        );
        assert_eq!(
            std::any::type_name::<dyn EventFilter>(),
            std::any::type_name::<dyn traits::EventFilter>()
        );
        assert_eq!(
            std::any::type_name::<dyn EventHandler>(),
            std::any::type_name::<dyn traits::EventHandler>()
        );
        assert_eq!(
            std::any::type_name::<dyn EventParser<Input = Vec<u8>>>(),
            std::any::type_name::<dyn traits::EventParser<Input = Vec<u8>>>()
        );
    }

    #[test]
    fn test_crate_root_types_reexports() {
        // Test that type re-exports work at crate root
        let _event_kind_type = std::any::type_name::<EventKind>();
        let _metadata_type = std::any::type_name::<EventMetadata>();
        let _generic_event_type = std::any::type_name::<GenericEvent>();
        let _stream_info_type = std::any::type_name::<StreamInfo>();

        // Verify they're the same as the original types
        assert_eq!(
            std::any::type_name::<EventKind>(),
            std::any::type_name::<types::EventKind>()
        );
        assert_eq!(
            std::any::type_name::<EventMetadata>(),
            std::any::type_name::<types::EventMetadata>()
        );
        assert_eq!(
            std::any::type_name::<GenericEvent>(),
            std::any::type_name::<types::GenericEvent>()
        );
        assert_eq!(
            std::any::type_name::<StreamInfo>(),
            std::any::type_name::<types::StreamInfo>()
        );
    }

    #[test]
    fn test_crate_root_utils_reexports() {
        // Test that utils re-exports work at crate root
        let _batch_stream_type = std::any::type_name::<EventBatchStream>();
        let _stream_type = std::any::type_name::<EventStream>();

        // Verify they're the same as the original types
        assert_eq!(
            std::any::type_name::<EventBatchStream>(),
            std::any::type_name::<utils::EventBatchStream>()
        );
        assert_eq!(
            std::any::type_name::<EventStream>(),
            std::any::type_name::<utils::EventStream>()
        );
    }

    #[test]
    fn test_prelude_module_exists() {
        // Test that the prelude module is accessible
        let _ = std::any::type_name::<prelude::EventError>();

        // Test that we can use the prelude module path
        use crate::prelude;
        let _ = std::any::type_name::<prelude::EventError>();
    }

    #[test]
    fn test_all_modules_declared() {
        // This test verifies that all expected modules are declared
        // by checking that their module paths exist

        // Check error module
        let _ = std::any::type_name::<crate::error::EventError>();

        // Check types module
        let _ = std::any::type_name::<crate::types::EventKind>();

        // Check traits module
        let _ = std::any::type_name::<dyn crate::traits::Event>();

        // Check parser module
        let _ = std::any::type_name::<crate::parser::JsonEventParser>();

        // Check utils module
        let _ = std::any::type_name::<crate::utils::EventStream>();
    }

    #[test]
    fn test_compiler_attributes() {
        // Test that the crate compiles with the specified lints
        // This is mainly a compilation test - if it compiles, the attributes work

        // These should not trigger clippy warnings due to #![warn(clippy::all)]
        let _x = 42;
        let _result = Ok::<i32, &str>(_x);

        // This would normally trigger clippy::module_inception but should be allowed
        // due to #![allow(clippy::module_inception)]
    }

    #[test]
    fn test_documentation_example_types_exist() {
        // Test that types referenced in the documentation example exist
        // This ensures the documentation example is valid

        let _ = std::any::type_name::<GenericEvent>();
        let _ = std::any::type_name::<EventKind>();
        let _ = std::any::type_name::<EventError>();

        // These should be accessible through prelude
        use crate::prelude::*;
        let _ = std::any::type_name::<GenericEvent>();
        let _ = std::any::type_name::<EventKind>();
        let _ = std::any::type_name::<EventError>();
    }
}
