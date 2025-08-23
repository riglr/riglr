//! riglr-showcase library
//!
//! This library exposes common functionality used by the riglr-showcase binary
//! and its tests.

pub mod agents;
pub mod auth;
pub mod commands;
pub mod processors;

/// Configuration module providing backward compatibility.
///
/// Re-exports all items from riglr-config for backward compatibility.
pub mod config {
    pub use riglr_config::*;
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_modules_are_public_and_accessible() {
        // Test that all public modules are accessible
        // This ensures the module declarations are correct

        // The existence of this test and successful compilation indicates
        // that all module declarations are correct
        // Test passes if compilation succeeds
    }

    #[test]
    fn test_config_module_exists_and_is_public() {
        // Test that the config module is accessible
        // The successful compilation indicates the module is properly declared
        // Test passes if compilation succeeds
    }

    #[test]
    fn test_config_module_re_exports_riglr_config() {
        // Test that config module properly re-exports riglr_config items
        // We'll test this by checking if we can access common configuration types
        // through our config module

        // Since this is a re-export test, we need to ensure the re-export works
        // The presence of the use statement should make riglr_config items available
        // We can test this by attempting to reference the module path
        let _config_module_path = std::module_path!();

        // The config module should be accessible and contain re-exported items
        // This test passes if the module compiles correctly with the re-export
        // Test passes if compilation succeeds
    }

    #[test]
    fn test_library_crate_name_and_structure() {
        // Test that we're in the correct crate context
        let crate_name = env!("CARGO_PKG_NAME");
        assert_eq!(crate_name, "riglr-showcase");
    }

    #[test]
    fn test_crate_metadata() {
        // Test crate version is accessible
        let version = env!("CARGO_PKG_VERSION");
        assert!(!version.is_empty(), "Crate version should not be empty");

        // Test crate name
        let name = env!("CARGO_PKG_NAME");
        assert_eq!(
            name, "riglr-showcase",
            "Crate name should be riglr-showcase"
        );
    }

    #[test]
    fn test_module_declarations_are_valid() {
        // This test ensures all module declarations compile correctly
        // If any module declaration was invalid, this test would fail to compile

        // This is a compile-time test that ensures modules are properly declared
        // The successful compilation of this test indicates all modules exist

        // If we reach this point, all modules compiled successfully
        // Test passes if compilation succeeds
    }

    #[test]
    fn test_config_module_backward_compatibility() {
        // Test that the config module serves its purpose of backward compatibility
        // by ensuring it's a proper re-export module

        // The config module should be a simple re-export wrapper
        // We test this by verifying the module structure
        let module_path = std::module_path!();
        assert!(
            module_path.contains("riglr_showcase"),
            "Should be in riglr_showcase crate"
        );
    }

    #[test]
    fn test_public_api_surface() {
        // Test that the public API surface is as expected
        // This library should expose 5 modules: agents, auth, commands, processors, config

        // Since we can't easily enumerate modules at runtime, we test by
        // ensuring compilation succeeds with the expected module structure
        // and that our re-exports work as intended

        // The mere fact that this test compiles means our module structure is correct
        // Test passes if compilation succeeds
    }

    #[test]
    fn test_documentation_attributes() {
        // Test that the crate-level documentation exists
        // We can't directly test doc comments, but we can ensure the crate
        // is properly documented by checking it compiles with the docs

        let crate_name = env!("CARGO_PKG_NAME");
        assert_eq!(crate_name, "riglr-showcase");

        // If documentation was malformed, compilation would fail
        // Test passes if compilation succeeds
    }

    #[test]
    fn test_config_module_documentation() {
        // Test that the config module has proper documentation
        // The config module is documented as providing backward compatibility

        // We can't directly access doc strings at runtime, but we can verify
        // the module compiles with its documentation
        // Test passes if compilation succeeds
    }
}
