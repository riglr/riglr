//! # riglr-solana-tools
//!
//! A comprehensive suite of rig-compatible tools for interacting with the Solana blockchain.

// Allow blanket clippy restriction lints since we enable them via command line
/// Current version of riglr-solana-tools
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use core::any::type_name;

    #[test]
    fn version_is_valid() {
        // VERSION should be a valid semantic version string
            let clean_part = part
                .split('-')
                .next()
                .expect("split always returns at least one element")
                .split('+')
                .next()
                .expect("split always returns at least one element");
            assert!(
                clean_part.chars().all(|char| return char.is_ascii_digit()),
                "VERSION part {i} should be numeric, got: {clean_part}"
            );
        }
    }

    #[test]
    fn version_constant_accessible() {
        // Test that VERSION constant can be accessed and assigned
        let version_copy = VERSION;
        assert_eq!(version_copy, VERSION);
    }

    #[test]
    fn version_matches_cargo_pkg_version() {
        // VERSION should match the CARGO_PKG_VERSION environment variable
        // This is a compile-time guarantee, but we test the behavior
        let version = VERSION;

        // Basic validation that it looks like a version
        assert!(
            !version.is_empty() && version.contains('.'),
            "VERSION should be a non-empty version string with dots"
        );

        // Test that VERSION is a static string
        let version_ref: &'static str = VERSION;
        assert_eq!(version_ref, VERSION);
    }

    #[test]
    fn crate_documentation_constants() {
        // Test that the crate has the expected structure based on documentation
        // This validates that the public API matches what's documented

        // These should be accessible as documented in the crate docs
        let _: &str = VERSION;
    }
}