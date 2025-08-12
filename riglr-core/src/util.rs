//! Utility functions for riglr-core

use std::env;

/// Ensures critical environment variables are set at startup
/// Prevents runtime failures by failing fast during initialization
pub fn must_get_env(key: &str) -> String {
    env::var(key).unwrap_or_else(|_| {
        panic!("FATAL: Environment variable '{}' is required but not set", key)
    })
}

/// Helper for optional environment variables with defaults
pub fn get_env_or_default(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_get_env_or_default_with_existing_var() {
        env::set_var("TEST_VAR_EXISTS", "test_value");
        let result = get_env_or_default("TEST_VAR_EXISTS", "default");
        assert_eq!(result, "test_value");
        env::remove_var("TEST_VAR_EXISTS");
    }

    #[test]
    fn test_get_env_or_default_with_missing_var() {
        env::remove_var("TEST_VAR_MISSING");
        let result = get_env_or_default("TEST_VAR_MISSING", "default_value");
        assert_eq!(result, "default_value");
    }

    #[test]
    fn test_must_get_env_with_existing_var() {
        env::set_var("TEST_MUST_VAR", "required_value");
        let result = must_get_env("TEST_MUST_VAR");
        assert_eq!(result, "required_value");
        env::remove_var("TEST_MUST_VAR");
    }

    #[test]
    #[should_panic(expected = "FATAL: Environment variable 'TEST_MISSING_REQUIRED' is required but not set")]
    fn test_must_get_env_with_missing_var() {
        env::remove_var("TEST_MISSING_REQUIRED");
        must_get_env("TEST_MISSING_REQUIRED");
    }
}