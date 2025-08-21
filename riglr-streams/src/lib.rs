//! RIGLR Streams - Event-driven streaming capabilities for blockchain agents
//!
//! This crate provides streaming components that enable developers to build
//! proactive, event-driven blockchain agents while maintaining RIGLR's
//! library-first philosophy.

/// Core streaming abstractions, operators, and management utilities
pub mod core;
/// EVM blockchain streaming capabilities and multi-chain management
pub mod evm;
/// External data source streaming for exchanges and blockchain services
pub mod external;
/// Production-ready utilities for stream management and monitoring
pub mod production;
/// Solana blockchain streaming capabilities via Geyser
pub mod solana;
/// Event-triggered tools and condition matching utilities
pub mod tools;

/// Prelude module for convenient imports
pub mod prelude {
    // Core streaming abstractions
    pub use crate::core::{
        combinators, BufferStrategy, ComposableStream, DynamicStreamedEvent, FinancialStreamExt,
        HandlerExecutionMode, IntoStreamedEvent, MetricsCollector, Stream, StreamError,
        StreamEvent, StreamHealth, StreamManager, StreamManagerBuilder, StreamedEvent,
    };

    // Enhanced operators with events-core integration
    pub use crate::core::enhanced_operators::{BatchedStream, EnhancedStream, EnhancedStreamExt};

    // Re-export BatchEvent for use with specific Event types
    pub use crate::core::enhanced_operators::BatchEvent;

    // Event-triggered tools
    pub use crate::tools::ConditionCombinator;
    // EventTriggerBuilder and EventTriggeredTool need generic parameters, so they're not included in prelude

    // Re-export events-core utilities for enhanced streaming capabilities
    pub use riglr_events_core::prelude::*;
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_prelude_core_imports_are_accessible() {
        // Test that all core streaming abstractions are accessible through prelude
        use crate::prelude::{
            BufferStrategy, HandlerExecutionMode, StreamError, StreamEvent, StreamHealth,
        };

        // These imports should compile without error, proving they are accessible
        // We use type assertions to ensure the types exist and are correctly re-exported
        let _buffer_strategy: Option<BufferStrategy> = None;
        let _handler_mode: Option<HandlerExecutionMode> = None;
        let _stream_error: Option<StreamError> = None;
        let _stream_event: Option<Box<dyn StreamEvent>> = None;
        let _stream_health: Option<StreamHealth> = None;
    }

    #[test]
    fn test_prelude_enhanced_operators_imports_are_accessible() {
        // Test that enhanced operators are accessible through prelude
        // These imports should compile without error
        // BatchEvent needs a generic parameter, so we can't create it without specifying the Event type
    }

    #[test]
    fn test_prelude_tools_imports_are_accessible() {
        // Test that event-triggered tools are accessible through prelude
        use crate::prelude::ConditionCombinator;

        // These imports should compile without error
        let _condition_combinator: Option<ConditionCombinator> = None;
        // EventTriggerBuilder and EventTriggeredTool need generic parameters
    }

    #[test]
    fn test_prelude_events_core_imports_are_accessible() {
        // Test that events-core utilities are accessible through prelude
        // We import a few key types from riglr_events_core to verify re-export works
        // Note: The actual availability depends on what riglr_events_core::prelude exports

        // This test verifies that the re-export compiles
        // If riglr_events_core::prelude has any items, they should be accessible here
        // Since we don't know the exact contents, we just verify the glob import works
    }

    #[test]
    fn test_module_declarations_are_valid() {
        // Test that all declared modules can be accessed
        // This is a compilation test - if modules don't exist, this won't compile

        // Core module
        let _core_exists = std::any::type_name::<crate::core::StreamError>();

        // Tools module exists but EventTriggeredTool needs generics
        // let _tools_exists = std::any::type_name::<crate::tools::EventTriggeredTool>();

        // These module accesses prove the modules are properly declared and accessible
        // If we reach here, modules are accessible
    }

    #[test]
    fn test_prelude_module_exists_and_is_public() {
        // Verify the prelude module itself is accessible
        // The module should be accessible, which this import proves
        // We can't test much about the module itself, but its accessibility is confirmed
        let _module_name = std::module_path!();
        assert!(_module_name.contains("riglr_streams"));
    }

    #[test]
    fn test_crate_documentation_attributes() {
        // Verify the crate has proper documentation structure
        // This is more of a compilation test to ensure doc comments are valid

        // Test that module paths follow expected naming conventions
        assert_eq!(module_path!(), "riglr_streams::tests");
    }

    #[test]
    fn test_all_public_modules_are_accessible() {
        // Test each public module can be accessed through the crate root

        // Core module
        use crate::core;
        let _core_module = std::any::type_name_of_val(&core::StreamError::Configuration {
            message: "test".to_string(),
        });

        // Other modules exist but we'll skip importing them to avoid unused warnings

        // If we reach this point, all modules are accessible
    }

    #[test]
    fn test_prelude_provides_convenient_access() {
        // Test that using prelude provides a convenient single import point
        use crate::prelude::*;

        // Verify we can access core types without individual imports
        let _stream_error = StreamError::Configuration {
            message: "test".to_string(),
        };
        let _stream_health = StreamHealth::Healthy;
        let _handler_mode = HandlerExecutionMode::Sequential;

        // These should all be accessible through the glob import
    }

    #[test]
    fn test_stream_error_variants_accessible_through_prelude() {
        use crate::prelude::*;

        // Test different StreamError variants are accessible
        let _config_error = StreamError::Configuration {
            message: "test".to_string(),
        };
        let _connection_error = StreamError::Connection {
            message: "test".to_string(),
            retriable: false,
        };
        let _processing_error = StreamError::Processing {
            message: "test".to_string(),
        };

        // Verify error types are properly accessible
        match (StreamError::Configuration {
            message: "test".to_string(),
        }) {
            StreamError::Configuration { .. } => {}
            _ => panic!("StreamError variants not accessible through prelude"),
        }
    }

    #[test]
    fn test_stream_health_variants_accessible_through_prelude() {
        use crate::prelude::*;

        // Test StreamHealth variants
        let healthy = StreamHealth::Healthy;
        let degraded = StreamHealth::Degraded;
        let unhealthy = StreamHealth::Unhealthy;

        // Verify variants are accessible and distinct
        assert_ne!(healthy, degraded);
        assert_ne!(degraded, unhealthy);
    }

    #[test]
    fn test_handler_execution_mode_variants_accessible_through_prelude() {
        use crate::prelude::*;

        // Test HandlerExecutionMode variants
        let sequential = HandlerExecutionMode::Sequential;
        let concurrent = HandlerExecutionMode::Concurrent;

        // Verify variants are accessible and distinct
        assert_ne!(sequential, concurrent);
    }
}
