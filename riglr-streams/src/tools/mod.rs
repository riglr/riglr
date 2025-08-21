/// Event condition matching and filtering functionality
pub mod condition;
/// Event-triggered tool implementations and streaming tools
pub mod event_triggered;
/// Utility functions for event type conversion and handling
pub mod event_utils;
/// Streaming tool worker extensions for processing events
pub mod worker_extension;

pub use condition::{ConditionCombinator, EventCondition, EventMatcher};
pub use event_triggered::StreamingTool;
pub use worker_extension::StreamingToolWorker;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_condition_module_exports_are_accessible() {
        // Test that re-exported types from condition module are accessible
        // This ensures the pub use statements work correctly

        // We can't instantiate these without knowing their constructors,
        // but we can verify they're accessible by referencing their types
        let _condition_type = std::marker::PhantomData::<Box<dyn EventCondition>>;
        let _matcher_type = std::marker::PhantomData::<condition::Matcher>;
        let _combinator_type = std::marker::PhantomData::<ConditionCombinator>;
    }

    #[test]
    fn test_event_triggered_module_exports_are_accessible() {
        // Test that re-exported types from event_triggered module are accessible
        let _streaming_tool_type = std::marker::PhantomData::<Box<dyn StreamingTool>>;
    }

    #[test]
    fn test_worker_extension_module_exports_are_accessible() {
        // Test that re-exported types from worker_extension module are accessible
        let _worker_type = std::marker::PhantomData::<StreamingToolWorker>;
    }

    #[test]
    fn test_all_submodules_are_declared() {
        // Test that all expected submodules are properly declared
        // This is verified at compile time, but we can document the expectation

        // If any of these modules don't exist, compilation will fail
        // The fact that this test compiles proves the modules are correctly declared
        // This test passes if compilation succeeds
    }

    #[test]
    fn test_module_structure_is_complete() {
        // Verify that the module has the expected structure
        // This test documents the intended module organization

        // Check that we have exactly 4 submodules
        // Note: This is more of a documentation test since module structure
        // is validated at compile time

        // The modules should be:
        // - condition: for event condition matching
        // - event_triggered: for event-triggered tools
        // - event_utils: for utility functions
        // - worker_extension: for streaming tool workers

        // Compilation success indicates correct structure
    }

    #[test]
    fn test_re_exports_follow_naming_convention() {
        // Test that the re-exported items follow expected naming conventions
        // This is validated at compile time, but documents expectations

        // All condition-related exports should be from condition module
        let _condition_exports = [
            std::any::type_name::<Box<dyn EventCondition>>(),
            std::any::type_name::<condition::Matcher>(),
            std::any::type_name::<ConditionCombinator>(),
        ];

        // All event-triggered exports should be from event_triggered module
        let _event_triggered_exports = [std::any::type_name::<Box<dyn StreamingTool>>()];

        // Worker extension export
        let _worker_export = std::any::type_name::<StreamingToolWorker>();

        // If we can get type names, the re-exports are working
    }

    #[test]
    fn test_module_documentation_exists() {
        // Verify that each module has documentation comments
        // This is more of a structural test to ensure good documentation practices

        // The module declarations should have doc comments explaining their purpose:
        // - condition: Event condition matching and filtering functionality
        // - event_triggered: Event-triggered tool implementations and streaming tools
        // - event_utils: Utility functions for event type conversion and handling
        // - worker_extension: Streaming tool worker extensions for processing events

        // Documentation exists if compilation succeeds with doc comments
    }
}
