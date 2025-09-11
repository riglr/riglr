//! Utility modules and functions for riglr-core

pub mod rate_limit_strategy;
pub mod rate_limiter;
pub mod secure_keys;
pub mod token_bucket;

// Re-export main types for convenience
pub use rate_limit_strategy::{FixedWindowStrategy, RateLimitStrategy};
pub use rate_limiter::{RateLimitStrategyType, RateLimiter, RateLimiterBuilder};
pub use secure_keys::{
    ensure_key_directory, get_default_key_directory, load_private_key_from_file,
    load_private_key_with_fallback,
};
pub use token_bucket::TokenBucketStrategy;

use std::env;

// Constants for doctest environment variables
#[doc(hidden)]
pub const DOCTEST_API_KEY: &str = "MY_API_KEY";
#[doc(hidden)]
pub const DOCTEST_OPTIONAL_SETTING: &str = "OPTIONAL_SETTING";
#[doc(hidden)]
pub const DOCTEST_API_KEY_VALIDATE: &str = "API_KEY";
#[doc(hidden)]
pub const DOCTEST_DATABASE_URL: &str = "DATABASE_URL";
#[doc(hidden)]
pub const DOCTEST_VAR1: &str = "VAR1";
#[doc(hidden)]
pub const DOCTEST_VAR2: &str = "VAR2";
#[doc(hidden)]
pub const DOCTEST_VAR3: &str = "VAR3";
#[doc(hidden)]
pub const DOCTEST_ENV_FILE_VAR: &str = "TEST_ENV_FILE_VAR";

/// Error type for environment variable operations
#[derive(Debug, thiserror::Error)]
pub enum EnvError {
    /// Required environment variable is not set
    #[error("Environment variable '{0}' is required but not set")]
    MissingRequired(String),

    /// Environment variable contains invalid UTF-8
    #[error("Environment variable '{0}' contains invalid UTF-8")]
    InvalidUtf8(String),
}

/// Result type alias for environment operations
pub type EnvResult<T> = Result<T, EnvError>;

/// Gets a required environment variable, returning an error if not set.
///
/// This is the recommended approach for libraries, allowing the application
/// to decide how to handle missing configuration.
///
/// # Examples
///
/// ```rust
/// use riglr_core::util::{get_required_env, DOCTEST_API_KEY};
///
/// # unsafe { std::env::set_var(DOCTEST_API_KEY, "secret123"); }
/// let api_key = get_required_env(DOCTEST_API_KEY).expect("MY_API_KEY must be set");
/// assert_eq!(api_key, "secret123");
/// # unsafe { std::env::remove_var(DOCTEST_API_KEY); }
/// ```
///
/// # Errors
///
/// Returns [`EnvError::MissingRequired`] if the environment variable is not set.
pub fn get_required_env(key: &str) -> EnvResult<String> {
    env::var(key).map_err(|_| EnvError::MissingRequired(key.to_string()))
}

/// Gets an optional environment variable with a default value.
///
/// # Examples
///
/// ```rust
/// use riglr_core::util::{get_env_or_default, DOCTEST_OPTIONAL_SETTING};
///
/// # unsafe { std::env::remove_var(DOCTEST_OPTIONAL_SETTING); }
/// let setting = get_env_or_default(DOCTEST_OPTIONAL_SETTING, "default_value");
/// assert_eq!(setting, "default_value");
/// ```
pub fn get_env_or_default(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_string())
}

/// Validates that all required environment variables are set.
///
/// This is useful during application initialization to fail fast if
/// configuration is incomplete.
///
/// # Examples
///
/// ```rust
/// use riglr_core::util::{validate_required_env, DOCTEST_API_KEY_VALIDATE, DOCTEST_DATABASE_URL};
///
/// # unsafe { std::env::set_var(DOCTEST_API_KEY_VALIDATE, "value1"); }
/// # unsafe { std::env::set_var(DOCTEST_DATABASE_URL, "value2"); }
/// let required = vec![DOCTEST_API_KEY_VALIDATE, DOCTEST_DATABASE_URL];
/// validate_required_env(&required).expect("Missing required environment variables");
/// # unsafe { std::env::remove_var(DOCTEST_API_KEY_VALIDATE); }
/// # unsafe { std::env::remove_var(DOCTEST_DATABASE_URL); }
/// ```
///
/// # Errors
///
/// Returns the first [`EnvError::MissingRequired`] encountered.
pub fn validate_required_env(keys: &[&str]) -> EnvResult<()> {
    for key in keys {
        get_required_env(key)?;
    }
    Ok(())
}

/// Gets multiple environment variables at once, returning a map.
///
/// # Examples
///
/// ```rust
/// use riglr_core::util::{get_env_vars, DOCTEST_VAR1, DOCTEST_VAR2, DOCTEST_VAR3};
/// use std::collections::HashMap;
///
/// # unsafe { std::env::set_var(DOCTEST_VAR1, "value1"); }
/// # unsafe { std::env::set_var(DOCTEST_VAR2, "value2"); }
/// let vars = get_env_vars(&[DOCTEST_VAR1, DOCTEST_VAR2, DOCTEST_VAR3]);
/// assert_eq!(vars.get(DOCTEST_VAR1), Some(&"value1".to_string()));
/// assert_eq!(vars.get(DOCTEST_VAR3), None);
/// # unsafe { std::env::remove_var(DOCTEST_VAR1); }
/// # unsafe { std::env::remove_var(DOCTEST_VAR2); }
/// ```
pub fn get_env_vars(keys: &[&str]) -> std::collections::HashMap<String, String> {
    keys.iter()
        .filter_map(|&key| env::var(key).ok().map(|value| (key.to_string(), value)))
        .collect()
}

/// Application-level helper that initializes environment from a `.env` file if present.
///
/// This is a convenience function for applications (not libraries) that want to
/// support `.env` files for local development.
///
/// # Examples
///
/// ```rust
/// use riglr_core::util::init_env_from_file;
///
/// // Load .env file if it exists (usually at application startup)
/// init_env_from_file(".env").ok(); // Ignore if file doesn't exist
/// ```
pub fn init_env_from_file(path: &str) -> std::io::Result<()> {
    if std::path::Path::new(path).exists() {
        dotenv::from_filename(path)
            .map_err(|e| std::io::Error::other(format!("Failed to load .env file: {}", e)))?;
    }
    Ok(())
}

#[cfg(test)]
#[allow(
    clippy::missing_safety_doc,
    clippy::undocumented_unsafe_blocks,
    unsafe_code
)] // Test functions have proper inline SAFETY documentation for test environment variable operations - unsafe blocks are required for Rust 2024 compatibility with std::env functions in test contexts
mod tests {
    use super::*;
    use std::env;

    // Test environment variable constants
    const TEST_VAR_EXISTS: &str = "TEST_VAR_EXISTS";
    const TEST_VAR_MISSING: &str = "TEST_VAR_MISSING";
    const TEST_REQUIRED_VAR: &str = "TEST_REQUIRED_VAR";
    const TEST_MISSING_REQUIRED: &str = "TEST_MISSING_REQUIRED";
    const TEST_VALIDATE_VAR1: &str = "TEST_VALIDATE_VAR1";
    const TEST_VALIDATE_VAR2: &str = "TEST_VALIDATE_VAR2";
    const TEST_VALIDATE_MISSING_VAR1: &str = "TEST_VALIDATE_MISSING_VAR1";
    const TEST_VALIDATE_MISSING_VAR2: &str = "TEST_VALIDATE_MISSING_VAR2";
    const TEST_MULTI_1: &str = "TEST_MULTI_1";
    const TEST_MULTI_2: &str = "TEST_MULTI_2";
    const TEST_MULTI_3: &str = "TEST_MULTI_3";
    const TEST_NONEXISTENT_VAR: &str = "NONEXISTENT_VAR_EMPTY_DEFAULT";
    const TEST_EMPTY_VALUE: &str = "TEST_EMPTY_VALUE";
    const TEST_EMPTY_REQUIRED: &str = "TEST_EMPTY_REQUIRED";
    const TEST_FIRST_MISSING: &str = "FIRST_MISSING";
    const TEST_SECOND_EXISTS: &str = "SECOND_EXISTS";
    const TEST_SPECIAL_CHARS: &str = "TEST_SPECIAL_CHARS";

    #[test]
    fn test_get_env_or_default_with_existing_var() {
        // SAFETY: Setting test environment variables in test context is safe
        unsafe {
            env::set_var(TEST_VAR_EXISTS, "test_value");
        }
        let result = get_env_or_default(TEST_VAR_EXISTS, "default");
        assert_eq!(result, "test_value");
        // SAFETY: Removing test environment variables in test context is safe
        unsafe {
            env::remove_var(TEST_VAR_EXISTS);
        }
    }

    #[test]
    fn test_get_env_or_default_with_missing_var() {
        // SAFETY: Removing test environment variables in test context is safe
        unsafe {
            env::remove_var(TEST_VAR_MISSING);
        }
        let result = get_env_or_default(TEST_VAR_MISSING, "default_value");
        assert_eq!(result, "default_value");
    }

    #[test]
    fn test_get_required_env_with_existing_var() {
        // SAFETY: Setting test environment variables in test context is safe
        unsafe {
            env::set_var(TEST_REQUIRED_VAR, "required_value");
        }
        let result = get_required_env(TEST_REQUIRED_VAR).unwrap();
        assert_eq!(result, "required_value");
        // SAFETY: Removing test environment variables in test context is safe
        unsafe {
            env::remove_var(TEST_REQUIRED_VAR);
        }
    }

    #[test]
    fn test_get_required_env_with_missing_var() {
        // SAFETY: Removing test environment variables in test context is safe
        unsafe {
            env::remove_var(TEST_MISSING_REQUIRED);
        }
        let result = get_required_env(TEST_MISSING_REQUIRED);
        assert!(result.is_err());
        match result {
            Err(EnvError::MissingRequired(key)) => {
                assert_eq!(key, TEST_MISSING_REQUIRED);
            }
            _ => panic!("Expected MissingRequired error"),
        }
    }

    #[test]
    fn test_validate_required_env_all_present() {
        // SAFETY: Setting test environment variables in test context is safe
        unsafe {
            env::set_var(TEST_VALIDATE_VAR1, "value1");
            env::set_var(TEST_VALIDATE_VAR2, "value2");
        }

        let result = validate_required_env(&[TEST_VALIDATE_VAR1, TEST_VALIDATE_VAR2]);
        assert!(result.is_ok());

        // SAFETY: Removing test environment variables in test context is safe
        unsafe {
            env::remove_var(TEST_VALIDATE_VAR1);
            env::remove_var(TEST_VALIDATE_VAR2);
        }
    }

    #[test]
    fn test_validate_required_env_missing_one() {
        // SAFETY: Setting and removing test environment variables in test context is safe
        unsafe {
            env::set_var(TEST_VALIDATE_MISSING_VAR1, "value1");
            env::remove_var(TEST_VALIDATE_MISSING_VAR2);
        }

        let result =
            validate_required_env(&[TEST_VALIDATE_MISSING_VAR1, TEST_VALIDATE_MISSING_VAR2]);
        assert!(result.is_err());

        // SAFETY: Removing test environment variables in test context is safe
        unsafe {
            env::remove_var(TEST_VALIDATE_MISSING_VAR1);
        }
    }

    #[test]
    fn test_get_env_vars() {
        // SAFETY: Setting and removing test environment variables in test context is safe
        unsafe {
            env::set_var(TEST_MULTI_1, "value1");
            env::set_var(TEST_MULTI_2, "value2");
            env::remove_var(TEST_MULTI_3);
        }

        let vars = get_env_vars(&[TEST_MULTI_1, TEST_MULTI_2, TEST_MULTI_3]);

        assert_eq!(vars.get(TEST_MULTI_1), Some(&"value1".to_string()));
        assert_eq!(vars.get(TEST_MULTI_2), Some(&"value2".to_string()));
        assert_eq!(vars.get(TEST_MULTI_3), None);

        // SAFETY: Removing test environment variables in test context is safe
        unsafe {
            env::remove_var(TEST_MULTI_1);
            env::remove_var(TEST_MULTI_2);
        }
    }

    #[test]
    fn test_get_env_vars_with_empty_array() {
        let vars = get_env_vars(&[]);
        assert!(vars.is_empty());
    }

    #[test]
    fn test_validate_required_env_with_empty_array() {
        let result = validate_required_env(&[]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_env_error_display_missing_required() {
        let error = EnvError::MissingRequired("TEST_KEY".to_string());
        assert_eq!(
            error.to_string(),
            "Environment variable 'TEST_KEY' is required but not set"
        );
    }

    #[test]
    fn test_env_error_display_invalid_utf8() {
        let error = EnvError::InvalidUtf8("TEST_KEY".to_string());
        assert_eq!(
            error.to_string(),
            "Environment variable 'TEST_KEY' contains invalid UTF-8"
        );
    }

    #[test]
    fn test_env_error_debug_format() {
        let error = EnvError::MissingRequired("TEST_KEY".to_string());
        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("MissingRequired"));
        assert!(debug_str.contains("TEST_KEY"));
    }

    #[test]
    fn test_init_env_from_file_with_nonexistent_file() {
        let result = init_env_from_file("nonexistent_file.env");
        assert!(result.is_ok());
    }

    #[test]
    fn test_init_env_from_file_with_existing_file() {
        use std::fs;
        use std::io::Write;

        // Create a temporary .env file
        let temp_file = "test_temp.env";
        let mut file = fs::File::create(temp_file).expect("Failed to create temp file");
        writeln!(file, "{}=test_value", DOCTEST_ENV_FILE_VAR)
            .expect("Failed to write to temp file");
        drop(file);

        // Test loading the file
        let result = init_env_from_file(temp_file);
        assert!(result.is_ok());

        // Verify the environment variable was loaded
        let loaded_value = env::var(DOCTEST_ENV_FILE_VAR).ok();
        assert_eq!(loaded_value, Some("test_value".to_string()));

        // Clean up
        fs::remove_file(temp_file).ok();
        // SAFETY: Removing test environment variables in test context is safe
        unsafe {
            env::remove_var(DOCTEST_ENV_FILE_VAR);
        }
    }

    #[test]
    fn test_init_env_from_file_with_invalid_file() {
        use std::fs;
        use std::io::Write;

        // Create a temporary file with invalid content that will cause dotenv to fail
        let temp_file = "test_invalid.env";
        let mut file = fs::File::create(temp_file).expect("Failed to create temp file");
        // Write invalid UTF-8 bytes
        file.write_all(&[0xFF, 0xFE])
            .expect("Failed to write invalid bytes");
        drop(file);

        // Test loading the invalid file
        let result = init_env_from_file(temp_file);
        assert!(result.is_err());

        // Clean up
        fs::remove_file(temp_file).ok();
    }

    #[test]
    fn test_get_env_or_default_with_empty_string_default() {
        // SAFETY: Removing test environment variables in test context is safe
        unsafe {
            env::remove_var(TEST_NONEXISTENT_VAR);
        }
        let result = get_env_or_default(TEST_NONEXISTENT_VAR, "");
        assert_eq!(result, "");
    }

    #[test]
    fn test_get_env_or_default_with_empty_string_value() {
        // SAFETY: Setting test environment variables in test context is safe
        unsafe {
            env::set_var(TEST_EMPTY_VALUE, "");
        }
        let result = get_env_or_default(TEST_EMPTY_VALUE, "default");
        assert_eq!(result, "");
        // SAFETY: Removing test environment variables in test context is safe
        unsafe {
            env::remove_var(TEST_EMPTY_VALUE);
        }
    }

    #[test]
    fn test_get_required_env_with_empty_string_value() {
        // SAFETY: Setting test environment variables in test context is safe
        unsafe {
            env::set_var(TEST_EMPTY_REQUIRED, "");
        }
        let result = get_required_env(TEST_EMPTY_REQUIRED).unwrap();
        assert_eq!(result, "");
        // SAFETY: Removing test environment variables in test context is safe
        unsafe {
            env::remove_var(TEST_EMPTY_REQUIRED);
        }
    }

    #[test]
    fn test_validate_required_env_fails_on_first_missing() {
        // SAFETY: Setting and removing test environment variables in test context is safe
        unsafe {
            env::remove_var(TEST_FIRST_MISSING);
            env::set_var(TEST_SECOND_EXISTS, "value");
        }

        let result = validate_required_env(&[TEST_FIRST_MISSING, TEST_SECOND_EXISTS]);
        assert!(result.is_err());

        match result {
            Err(EnvError::MissingRequired(key)) => {
                assert_eq!(key, TEST_FIRST_MISSING);
            }
            _ => panic!("Expected MissingRequired error for first missing variable"),
        }

        // SAFETY: Removing test environment variables in test context is safe
        unsafe {
            env::remove_var(TEST_SECOND_EXISTS);
        }
    }

    #[test]
    fn test_get_env_vars_with_special_characters() {
        // SAFETY: Setting test environment variables in test context is safe
        unsafe {
            env::set_var(TEST_SPECIAL_CHARS, "value with spaces & symbols!");
        }

        let vars = get_env_vars(&[TEST_SPECIAL_CHARS]);
        assert_eq!(
            vars.get(TEST_SPECIAL_CHARS),
            Some(&"value with spaces & symbols!".to_string())
        );

        // SAFETY: Removing test environment variables in test context is safe
        unsafe {
            env::remove_var(TEST_SPECIAL_CHARS);
        }
    }
}
