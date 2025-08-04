//! Utility functions for riglr-core

use std::env;

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
/// use riglr_core::util::get_required_env;
///
/// # std::env::set_var("MY_API_KEY", "secret123");
/// let api_key = get_required_env("MY_API_KEY").expect("MY_API_KEY must be set");
/// assert_eq!(api_key, "secret123");
/// # std::env::remove_var("MY_API_KEY");
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
/// use riglr_core::util::get_env_or_default;
///
/// # std::env::remove_var("OPTIONAL_SETTING");
/// let setting = get_env_or_default("OPTIONAL_SETTING", "default_value");
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
/// use riglr_core::util::validate_required_env;
///
/// # std::env::set_var("API_KEY", "value1");
/// # std::env::set_var("DATABASE_URL", "value2");
/// let required = vec!["API_KEY", "DATABASE_URL"];
/// validate_required_env(&required).expect("Missing required environment variables");
/// # std::env::remove_var("API_KEY");
/// # std::env::remove_var("DATABASE_URL");
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
/// use riglr_core::util::get_env_vars;
/// use std::collections::HashMap;
///
/// # std::env::set_var("VAR1", "value1");
/// # std::env::set_var("VAR2", "value2");
/// let vars = get_env_vars(&["VAR1", "VAR2", "VAR3"]);
/// assert_eq!(vars.get("VAR1"), Some(&"value1".to_string()));
/// assert_eq!(vars.get("VAR3"), None);
/// # std::env::remove_var("VAR1");
/// # std::env::remove_var("VAR2");
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
    fn test_get_required_env_with_existing_var() {
        env::set_var("TEST_REQUIRED_VAR", "required_value");
        let result = get_required_env("TEST_REQUIRED_VAR").unwrap();
        assert_eq!(result, "required_value");
        env::remove_var("TEST_REQUIRED_VAR");
    }

    #[test]
    fn test_get_required_env_with_missing_var() {
        env::remove_var("TEST_MISSING_REQUIRED");
        let result = get_required_env("TEST_MISSING_REQUIRED");
        assert!(result.is_err());
        match result {
            Err(EnvError::MissingRequired(key)) => {
                assert_eq!(key, "TEST_MISSING_REQUIRED");
            }
            _ => panic!("Expected MissingRequired error"),
        }
    }

    #[test]
    fn test_validate_required_env_all_present() {
        env::set_var("TEST_VAR1", "value1");
        env::set_var("TEST_VAR2", "value2");

        let result = validate_required_env(&["TEST_VAR1", "TEST_VAR2"]);
        assert!(result.is_ok());

        env::remove_var("TEST_VAR1");
        env::remove_var("TEST_VAR2");
    }

    #[test]
    fn test_validate_required_env_missing_one() {
        env::set_var("TEST_VAR1", "value1");
        env::remove_var("TEST_VAR2");

        let result = validate_required_env(&["TEST_VAR1", "TEST_VAR2"]);
        assert!(result.is_err());

        env::remove_var("TEST_VAR1");
    }

    #[test]
    fn test_get_env_vars() {
        env::set_var("TEST_MULTI_1", "value1");
        env::set_var("TEST_MULTI_2", "value2");
        env::remove_var("TEST_MULTI_3");

        let vars = get_env_vars(&["TEST_MULTI_1", "TEST_MULTI_2", "TEST_MULTI_3"]);

        assert_eq!(vars.get("TEST_MULTI_1"), Some(&"value1".to_string()));
        assert_eq!(vars.get("TEST_MULTI_2"), Some(&"value2".to_string()));
        assert_eq!(vars.get("TEST_MULTI_3"), None);

        env::remove_var("TEST_MULTI_1");
        env::remove_var("TEST_MULTI_2");
    }
}
