//! Environment-specific configuration support

use std::env;

/// Custom environment variable resolver function type
pub type EnvResolver = Box<dyn Fn(&str) -> Option<String> + Send + Sync>;

/// Source of environment variables (for testing and custom providers)
pub enum EnvironmentSource {
    /// Use system environment variables
    System,
    /// Use custom environment provider (for testing)
    #[allow(dead_code)]
    Custom(EnvResolver),
}

impl std::fmt::Debug for EnvironmentSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::System => write!(f, "System"),
            Self::Custom(_) => write!(f, "Custom(...)"),
        }
    }
}

impl Clone for EnvironmentSource {
    fn clone(&self) -> Self {
        match self {
            Self::System => Self::System,
            // For Custom variant, we can't clone the function, so we just return System
            // This is only used in tests anyway
            Self::Custom(_) => Self::System,
        }
    }
}

impl Default for EnvironmentSource {
    fn default() -> Self {
        Self::System
    }
}

impl EnvironmentSource {
    /// Get environment variable value
    pub fn get(&self, key: &str) -> Option<String> {
        match self {
            Self::System => env::var(key).ok(),
            Self::Custom(provider) => provider(key),
        }
    }

    /// Check if environment variable exists
    #[allow(dead_code)]
    pub fn exists(&self, key: &str) -> bool {
        self.get(key).is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    // Constants for environment variable names used in tests
    const TEST_VAR_EXISTS: &str = "TEST_VAR_EXISTS";
    const TEST_VAR_NOT_EXISTS: &str = "TEST_VAR_NOT_EXISTS";
    const CUSTOM_VAR: &str = "CUSTOM_VAR";
    const NON_EXISTENT_VAR: &str = "NON_EXISTENT_VAR";
    const EXISTS_VAR: &str = "EXISTS_VAR";
    const NOT_EXISTS_VAR: &str = "NOT_EXISTS_VAR";

    // Helper to create a custom environment source for testing
    fn create_test_env(vars: HashMap<String, String>) -> EnvironmentSource {
        EnvironmentSource::Custom(Box::new(move |key: &str| vars.get(key).cloned()))
    }

    #[test]
    fn test_environment_source_system_get_when_var_exists_should_return_some() {
        // Set a test environment variable
        env::set_var(TEST_VAR_EXISTS, "test_value");

        let source = EnvironmentSource::System;
        let result = source.get(TEST_VAR_EXISTS);

        assert_eq!(result, Some("test_value".to_string()));

        // Clean up
        env::remove_var(TEST_VAR_EXISTS);
    }

    #[test]
    fn test_environment_source_system_get_when_var_not_exists_should_return_none() {
        let source = EnvironmentSource::System;
        let result = source.get(TEST_VAR_NOT_EXISTS);

        assert_eq!(result, None);
    }

    #[test]
    fn test_environment_source_custom_get_when_var_exists_should_return_some() {
        let mut vars = HashMap::new();
        vars.insert(CUSTOM_VAR.to_string(), "custom_value".to_string());

        let source = create_test_env(vars);
        let result = source.get(CUSTOM_VAR);

        assert_eq!(result, Some("custom_value".to_string()));
    }

    #[test]
    fn test_environment_source_custom_get_when_var_not_exists_should_return_none() {
        let vars = HashMap::new();

        let source = create_test_env(vars);
        let result = source.get(NON_EXISTENT_VAR);

        assert_eq!(result, None);
    }

    #[test]
    fn test_environment_source_default_should_be_system() {
        let source = EnvironmentSource::default();

        match source {
            EnvironmentSource::System => {
                // Expected
            }
            _ => panic!("Default should be System"),
        }
    }

    #[test]
    fn test_environment_source_exists_when_var_exists_should_return_true() {
        let mut vars = HashMap::new();
        vars.insert(EXISTS_VAR.to_string(), "value".to_string());

        let source = create_test_env(vars);
        let result = source.exists(EXISTS_VAR);

        assert!(result);
    }

    #[test]
    fn test_environment_source_exists_when_var_not_exists_should_return_false() {
        let vars = HashMap::new();

        let source = create_test_env(vars);
        let result = source.exists(NOT_EXISTS_VAR);

        assert!(!result);
    }
}
