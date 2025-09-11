//! Configuration error types

use thiserror::Error;

/// Configuration result type
pub type ConfigResult<T> = Result<T, ConfigError>;

/// Configuration errors
#[derive(Debug, Error)]
pub enum ConfigError {
    /// Environment variable not found
    #[error("Missing environment variable: {0}")]
    MissingEnvVar(String),

    /// Failed to parse environment variables
    #[error("Environment parsing error: {0}")]
    EnvParse(String),

    /// Invalid configuration value
    #[error("Invalid configuration: {0}")]
    ValidationError(String),

    /// Failed to parse configuration
    #[error("Failed to parse configuration: {0}")]
    ParseError(String),

    /// IO error
    #[error("IO error: {0}")]
    IoError(String),

    /// Chain not supported
    #[error("Chain {0} is not supported")]
    ChainNotSupported(u64),

    /// Provider not configured
    #[error("Provider {0} is not configured")]
    ProviderNotConfigured(String),

    /// Configuration already locked
    #[error("Configuration locked: {0}")]
    ConfigLocked(String),

    /// Generic error
    #[error("{0}")]
    Generic(String),
}

impl ConfigError {
    /// Create a validation error
    pub fn validation<S: Into<String>>(msg: S) -> Self {
        Self::ValidationError(msg.into())
    }

    /// Create a parse error
    pub fn parse<S: Into<String>>(msg: S) -> Self {
        Self::ParseError(msg.into())
    }

    /// Create an IO error
    pub fn io<S: Into<String>>(msg: S) -> Self {
        Self::IoError(msg.into())
    }

    /// Create a generic error
    pub fn generic<S: Into<String>>(msg: S) -> Self {
        Self::Generic(msg.into())
    }
}

impl From<std::io::Error> for ConfigError {
    fn from(err: std::io::Error) -> Self {
        Self::IoError(err.to_string())
    }
}

impl From<toml::de::Error> for ConfigError {
    fn from(err: toml::de::Error) -> Self {
        Self::ParseError(err.to_string())
    }
}

impl From<envy::Error> for ConfigError {
    fn from(err: envy::Error) -> Self {
        Self::ParseError(err.to_string())
    }
}

#[cfg(test)]
#[allow(unsafe_code)] // Test functions use unsafe std::env operations for Rust 2024 compatibility with proper safety documentation
mod tests {
    use super::*;
    use std::io;

    #[test]
    fn test_config_error_missing_env_var_display() {
        let error = ConfigError::MissingEnvVar("DATABASE_URL".to_string());
        assert_eq!(
            error.to_string(),
            "Missing environment variable: DATABASE_URL"
        );
    }

    #[test]
    fn test_config_error_validation_error_display() {
        let error = ConfigError::ValidationError("Invalid port number".to_string());
        assert_eq!(
            error.to_string(),
            "Invalid configuration: Invalid port number"
        );
    }

    #[test]
    fn test_config_error_parse_error_display() {
        let error = ConfigError::ParseError("Invalid TOML format".to_string());
        assert_eq!(
            error.to_string(),
            "Failed to parse configuration: Invalid TOML format"
        );
    }

    #[test]
    fn test_config_error_io_error_display() {
        let error = ConfigError::IoError("File not found".to_string());
        assert_eq!(error.to_string(), "IO error: File not found");
    }

    #[test]
    fn test_config_error_chain_not_supported_display() {
        let error = ConfigError::ChainNotSupported(1337);
        assert_eq!(error.to_string(), "Chain 1337 is not supported");
    }

    #[test]
    fn test_config_error_provider_not_configured_display() {
        let error = ConfigError::ProviderNotConfigured("RPC".to_string());
        assert_eq!(error.to_string(), "Provider RPC is not configured");
    }

    #[test]
    fn test_config_error_generic_display() {
        let error = ConfigError::Generic("Something went wrong".to_string());
        assert_eq!(error.to_string(), "Something went wrong");
    }

    #[test]
    fn test_validation_constructor_with_string() {
        let error = ConfigError::validation("Invalid value".to_string());
        match error {
            ConfigError::ValidationError(msg) => assert_eq!(msg, "Invalid value"),
            _ => panic!("Expected ValidationError"),
        }
    }

    #[test]
    fn test_validation_constructor_with_str() {
        let error = ConfigError::validation("Invalid value");
        match error {
            ConfigError::ValidationError(msg) => assert_eq!(msg, "Invalid value"),
            _ => panic!("Expected ValidationError"),
        }
    }

    #[test]
    fn test_parse_constructor_with_string() {
        let error = ConfigError::parse("Parse failed".to_string());
        match error {
            ConfigError::ParseError(msg) => assert_eq!(msg, "Parse failed"),
            _ => panic!("Expected ParseError"),
        }
    }

    #[test]
    fn test_parse_constructor_with_str() {
        let error = ConfigError::parse("Parse failed");
        match error {
            ConfigError::ParseError(msg) => assert_eq!(msg, "Parse failed"),
            _ => panic!("Expected ParseError"),
        }
    }

    #[test]
    fn test_io_constructor_with_string() {
        let error = ConfigError::io("IO failed".to_string());
        match error {
            ConfigError::IoError(msg) => assert_eq!(msg, "IO failed"),
            _ => panic!("Expected IoError"),
        }
    }

    #[test]
    fn test_io_constructor_with_str() {
        let error = ConfigError::io("IO failed");
        match error {
            ConfigError::IoError(msg) => assert_eq!(msg, "IO failed"),
            _ => panic!("Expected IoError"),
        }
    }

    #[test]
    fn test_generic_constructor_with_string() {
        let error = ConfigError::generic("Generic error".to_string());
        match error {
            ConfigError::Generic(msg) => assert_eq!(msg, "Generic error"),
            _ => panic!("Expected Generic"),
        }
    }

    #[test]
    fn test_generic_constructor_with_str() {
        let error = ConfigError::generic("Generic error");
        match error {
            ConfigError::Generic(msg) => assert_eq!(msg, "Generic error"),
            _ => panic!("Expected Generic"),
        }
    }

    #[test]
    fn test_from_std_io_error() {
        let io_error = io::Error::new(io::ErrorKind::NotFound, "File not found");
        let config_error: ConfigError = io_error.into();

        match config_error {
            ConfigError::IoError(msg) => assert!(msg.contains("File not found")),
            _ => panic!("Expected IoError"),
        }
    }

    #[test]
    fn test_from_toml_de_error() {
        let toml_content = "invalid = toml = content";
        let toml_error = toml::from_str::<toml::Value>(toml_content).unwrap_err();
        let config_error: ConfigError = toml_error.into();

        match config_error {
            ConfigError::ParseError(_) => {}
            _ => panic!("Expected ParseError"),
        }
    }

    #[test]
    fn test_from_envy_error() {
        // Create an envy error by trying to deserialize an empty environment
        const TEST_REQUIRED_VAR: &str = "TEST_REQUIRED_VAR";
        #[allow(clippy::disallowed_methods)]
        // SAFETY: This is a test function and we're only removing a test environment variable
        // temporarily for the duration of this test. This ensures the test runs in a clean
        // environment without the required variable, allowing us to test error handling.
        unsafe {
            std::env::remove_var(TEST_REQUIRED_VAR);
        }

        #[derive(Debug, serde::Deserialize)]
        #[allow(dead_code)]
        struct TestConfig {
            test_required_var: String,
        }

        let envy_error = envy::from_env::<TestConfig>().unwrap_err();
        let config_error: ConfigError = envy_error.into();

        match config_error {
            ConfigError::ParseError(_) => {}
            _ => panic!("Expected ParseError"),
        }
    }

    #[test]
    fn test_config_error_debug() {
        let error = ConfigError::MissingEnvVar("TEST_VAR".to_string());
        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("MissingEnvVar"));
        assert!(debug_str.contains("TEST_VAR"));
    }

    #[test]
    fn test_config_result_type_alias() {
        let success: ConfigResult<i32> = Ok(42);
        assert_eq!(success.unwrap(), 42);

        let failure: ConfigResult<i32> = Err(ConfigError::Generic("test".to_string()));
        assert!(failure.is_err());
    }

    #[test]
    fn test_all_error_variants_can_be_constructed() {
        let _missing_env = ConfigError::MissingEnvVar("VAR".to_string());
        let _validation = ConfigError::ValidationError("msg".to_string());
        let _parse = ConfigError::ParseError("msg".to_string());
        let _io = ConfigError::IoError("msg".to_string());
        let _chain = ConfigError::ChainNotSupported(1);
        let _provider = ConfigError::ProviderNotConfigured("provider".to_string());
        let _generic = ConfigError::Generic("msg".to_string());
    }

    #[test]
    fn test_error_equality_and_matching() {
        let error1 = ConfigError::ChainNotSupported(42);
        let error2 = ConfigError::ChainNotSupported(42);

        match (&error1, &error2) {
            (ConfigError::ChainNotSupported(id1), ConfigError::ChainNotSupported(id2)) => {
                assert_eq!(id1, id2);
            }
            _ => panic!("Pattern matching failed"),
        }
    }

    #[test]
    fn test_error_with_empty_strings() {
        let error = ConfigError::MissingEnvVar("".to_string());
        assert_eq!(error.to_string(), "Missing environment variable: ");

        let generic_error = ConfigError::generic("");
        assert_eq!(generic_error.to_string(), "");
    }

    #[test]
    fn test_error_with_special_characters() {
        let error = ConfigError::validation("Invalid: üñíçødé");
        match error {
            ConfigError::ValidationError(msg) => assert_eq!(msg, "Invalid: üñíçødé"),
            _ => panic!("Expected ValidationError"),
        }
    }

    #[test]
    fn test_chain_id_zero() {
        let error = ConfigError::ChainNotSupported(0);
        assert_eq!(error.to_string(), "Chain 0 is not supported");
    }

    #[test]
    fn test_chain_id_max_value() {
        let error = ConfigError::ChainNotSupported(u64::MAX);
        assert_eq!(
            error.to_string(),
            format!("Chain {} is not supported", u64::MAX)
        );
    }
}
