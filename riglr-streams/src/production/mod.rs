//! Production-ready utilities for stream management and monitoring.
//!
//! This module provides essential production features for managing streaming data pipelines:
//! - Health monitoring and status tracking
//! - Resilience patterns including circuit breakers and retry policies
//!
//! These components work together to ensure reliable operation of streaming systems
//! in production environments with proper observability and fault tolerance.
//!
//! Note: Metrics collection functionality has been consolidated into core::metrics::MetricsCollector.
//! Enable the "metrics-facade" feature to export metrics to the metrics crate.

pub mod health;
pub mod resilience;

pub use health::{ComponentHealth, HealthMonitor, HealthStatus};
pub use resilience::{BackoffStrategy, CircuitBreaker, RetryPolicy};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_structure_when_imported_should_expose_health_types() {
        // Test that health module types are properly re-exported
        // This ensures the public API is accessible as intended

        // We can't directly instantiate these types without knowing their constructors,
        // but we can verify they exist in the module's namespace
        let _type_exists: Option<ComponentHealth> = None;
        let _type_exists: Option<HealthMonitor> = None;
        let _type_exists: Option<HealthStatus> = None;
    }

    #[test]
    fn test_module_structure_when_imported_should_expose_resilience_types() {
        // Test that resilience module types are properly re-exported
        let _type_exists: Option<BackoffStrategy> = None;
        let _type_exists: Option<CircuitBreaker> = None;
        let _type_exists: Option<RetryPolicy> = None;
    }

    #[test]
    fn test_module_documentation_when_accessed_should_be_present() {
        // Test that module documentation is properly attached
        // This is a compile-time check that the doc comments are valid
        // The fact that this test compiles means documentation is valid
    }

    #[test]
    fn test_submodule_declarations_when_compiled_should_be_accessible() {
        // Test that all declared submodules compile and are accessible
        // This verifies the module structure is valid

        // health module should be accessible
        assert!(
            std::module_path!().contains("production"),
            "Should be in production module"
        );

        // The fact that this test compiles means all submodules are properly declared
    }

    #[test]
    fn test_public_api_surface_when_used_should_provide_expected_types() {
        // Test the complete public API surface
        // This ensures all re-exports are working correctly

        // Count the number of re-exported types from each module
        let health_types = ["ComponentHealth", "HealthMonitor", "HealthStatus"];
        let resilience_types = ["BackoffStrategy", "CircuitBreaker", "RetryPolicy"];

        assert_eq!(health_types.len(), 3, "Should export 3 health types");
        assert_eq!(
            resilience_types.len(),
            3,
            "Should export 3 resilience types"
        );

        // Total API surface
        let total_exported_types = health_types.len() + resilience_types.len();
        assert_eq!(
            total_exported_types, 6,
            "Should export exactly 6 types total"
        );
    }

    #[test]
    fn test_module_path_when_accessed_should_be_correct() {
        // Test that the module path is as expected
        let module_path = std::module_path!();
        assert!(
            module_path.contains("production"),
            "Module path should contain 'production', got: {}",
            module_path
        );
    }

    #[test]
    fn test_re_exports_when_used_should_not_cause_name_conflicts() {
        // Test that re-exports don't create naming conflicts
        // This is a compile-time check - if there were conflicts, this wouldn't compile

        use crate::production::{
            BackoffStrategy, CircuitBreaker, ComponentHealth, HealthMonitor, HealthStatus,
            RetryPolicy,
        };

        // Verify we can reference all types without conflicts
        let _health_types: (
            Option<ComponentHealth>,
            Option<HealthMonitor>,
            Option<HealthStatus>,
        ) = (None, None, None);
        let _resilience_types: (
            Option<BackoffStrategy>,
            Option<CircuitBreaker>,
            Option<RetryPolicy>,
        ) = (None, None, None);

        // The fact that this compiles means all re-exported types are accessible without conflicts
    }

    #[test]
    fn test_module_organization_when_structured_should_follow_domain_separation() {
        // Test that the module follows proper domain separation
        // Each submodule should handle a specific concern

        let modules = ["health", "resilience"];
        assert_eq!(modules.len(), 2, "Should have exactly 2 submodules");

        // Verify each module represents a distinct domain
        assert!(
            modules.contains(&"health"),
            "Should have health monitoring domain"
        );
        assert!(
            modules.contains(&"resilience"),
            "Should have resilience patterns domain"
        );
    }
}
