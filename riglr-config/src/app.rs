//! Application-level configuration

use crate::{ConfigError, ConfigResult};
use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError};

/// Application configuration
#[derive(Debug, Clone, Deserialize, Serialize, Validate)]
pub struct AppConfig {
    /// Server port
    #[serde(default = "default_port")]
    #[validate(range(min = 1, max = 65535))]
    pub port: u16,

    /// Application environment
    #[serde(default)]
    pub environment: Environment,

    /// Log level (trace, debug, info, warn, error)
    #[serde(default = "default_log_level")]
    pub log_level: String,

    /// Whether to use testnet chains
    #[serde(default)]
    pub use_testnet: bool,

    /// Transaction settings
    #[serde(flatten)]
    #[validate(nested)]
    pub transaction: TransactionConfig,

    /// Retry configuration
    #[serde(flatten)]
    #[validate(nested)]
    pub retry: RetryConfig,
}

/// Application environment
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Environment {
    /// Development environment for local testing and debugging
    Development,
    /// Staging environment for pre-production testing
    Staging,
    /// Production environment for live deployment
    Production,
}

impl Default for Environment {
    fn default() -> Self {
        Self::Development
    }
}

/// Transaction configuration
#[derive(Debug, Clone, Deserialize, Serialize, Validate)]
pub struct TransactionConfig {
    /// Maximum gas price in gwei
    #[serde(default = "default_max_gas_price")]
    #[validate(range(min = 1))]
    pub max_gas_price_gwei: u64,

    /// Priority fee in gwei
    #[serde(default = "default_priority_fee")]
    pub priority_fee_gwei: u64,

    /// Slippage tolerance as percentage (e.g., 0.5 = 0.5%)
    #[serde(default = "default_slippage")]
    #[validate(range(min = 0.0, max = 100.0))]
    pub slippage_tolerance_percent: f64,

    /// Transaction deadline in seconds
    #[serde(default = "default_deadline")]
    pub deadline_seconds: u64,
}

/// Retry configuration
#[derive(Debug, Clone, Deserialize, Serialize, Validate)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    #[serde(default = "default_max_retries")]
    #[validate(range(min = 1, max = 100))]
    pub max_retry_attempts: u32,

    /// Initial retry delay in milliseconds
    #[serde(default = "default_retry_delay")]
    pub retry_delay_ms: u64,

    /// Backoff multiplier for exponential backoff
    #[serde(default = "default_backoff_multiplier")]
    #[validate(custom(function = "validate_backoff_multiplier"))]
    pub retry_backoff_multiplier: f64,

    /// Maximum retry delay in milliseconds
    #[serde(default = "default_max_retry_delay")]
    pub max_retry_delay_ms: u64,
}

/// Custom validator for backoff multiplier
fn validate_backoff_multiplier(value: f64) -> Result<(), ValidationError> {
    if value <= 1.0 {
        return Err(ValidationError::new(
            "retry_backoff_multiplier must be greater than 1.0",
        ));
    }
    if value > 10.0 {
        return Err(ValidationError::new(
            "retry_backoff_multiplier must be at most 10.0",
        ));
    }
    Ok(())
}

impl AppConfig {
    /// Validates the application configuration for correctness
    pub fn validate(&self) -> ConfigResult<()> {
        // Use validator crate for field validation
        Validate::validate(self)
            .map_err(|e| ConfigError::validation(format!("Validation failed: {}", e)))?;

        // Validate log level (custom validation not covered by validator attributes)
        match self.log_level.as_str() {
            "trace" | "debug" | "info" | "warn" | "error" => {}
            _ => {
                return Err(ConfigError::validation(format!(
                    "Invalid log level: {}",
                    self.log_level
                )))
            }
        }

        // Additional custom validation for retry config
        if self.retry.max_retry_delay_ms < self.retry.retry_delay_ms {
            return Err(ConfigError::validation(
                "max_retry_delay_ms must be >= retry_delay_ms",
            ));
        }

        Ok(())
    }

    /// Validates the application configuration
    pub fn validate_config(&self) -> ConfigResult<()> {
        Validate::validate(self)
            .map_err(|e| ConfigError::validation(format!("Validation failed: {}", e)))
    }
}

impl RetryConfig {
    /// Validates the retry configuration
    pub fn validate_config(&self) -> ConfigResult<()> {
        Validate::validate(self)
            .map_err(|e| ConfigError::validation(format!("Validation failed: {}", e)))
    }
}

// RetryConfig validation is now handled by the Validate trait from validator crate

// Default value functions
fn default_port() -> u16 {
    8080
}
fn default_log_level() -> String {
    "info".to_string()
}
fn default_max_gas_price() -> u64 {
    100
}
fn default_priority_fee() -> u64 {
    2
}
fn default_slippage() -> f64 {
    0.5
}
fn default_deadline() -> u64 {
    300
} // 5 minutes
fn default_max_retries() -> u32 {
    3
}
fn default_retry_delay() -> u64 {
    1000
}
fn default_backoff_multiplier() -> f64 {
    2.0
}
fn default_max_retry_delay() -> u64 {
    30000
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            port: default_port(),
            environment: Environment::default(),
            log_level: default_log_level(),
            use_testnet: false,
            transaction: TransactionConfig::default(),
            retry: RetryConfig::default(),
        }
    }
}

impl Default for TransactionConfig {
    fn default() -> Self {
        Self {
            max_gas_price_gwei: default_max_gas_price(),
            priority_fee_gwei: default_priority_fee(),
            slippage_tolerance_percent: default_slippage(),
            deadline_seconds: default_deadline(),
        }
    }
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retry_attempts: default_max_retries(),
            retry_delay_ms: default_retry_delay(),
            retry_backoff_multiplier: default_backoff_multiplier(),
            max_retry_delay_ms: default_max_retry_delay(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_environment_default_should_return_development() {
        assert_eq!(Environment::default(), Environment::Development);
    }

    #[test]
    fn test_environment_serialization() {
        // Test serde serialization/deserialization
        let env = Environment::Production;
        let json = serde_json::to_string(&env).unwrap();
        assert_eq!(json, "\"production\"");

        let deserialized: Environment = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, Environment::Production);
    }

    #[test]
    fn test_default_functions() {
        assert_eq!(default_port(), 8080);
        assert_eq!(default_log_level(), "info");
        assert_eq!(default_max_gas_price(), 100);
        assert_eq!(default_priority_fee(), 2);
        assert_eq!(default_slippage(), 0.5);
        assert_eq!(default_deadline(), 300);
        assert_eq!(default_max_retries(), 3);
        assert_eq!(default_retry_delay(), 1000);
        assert_eq!(default_backoff_multiplier(), 2.0);
        assert_eq!(default_max_retry_delay(), 30000);
    }

    #[test]
    fn test_app_config_default() {
        let config = AppConfig::default();
        assert_eq!(config.port, 8080);
        assert_eq!(config.environment, Environment::Development);
        assert_eq!(config.log_level, "info");
        assert!(!config.use_testnet);
        assert_eq!(config.transaction.max_gas_price_gwei, 100);
        assert_eq!(config.retry.max_retry_attempts, 3);
    }

    #[test]
    fn test_transaction_config_default() {
        let config = TransactionConfig::default();
        assert_eq!(config.max_gas_price_gwei, 100);
        assert_eq!(config.priority_fee_gwei, 2);
        assert_eq!(config.slippage_tolerance_percent, 0.5);
        assert_eq!(config.deadline_seconds, 300);
    }

    #[test]
    fn test_retry_config_default() {
        let config = RetryConfig::default();
        assert_eq!(config.max_retry_attempts, 3);
        assert_eq!(config.retry_delay_ms, 1000);
        assert_eq!(config.retry_backoff_multiplier, 2.0);
        assert_eq!(config.max_retry_delay_ms, 30000);
    }

    #[test]
    fn test_app_config_validate_when_valid_should_return_ok() {
        let config = AppConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_app_config_validate_when_invalid_log_level_should_return_err() {
        let mut config = AppConfig::default();
        config.log_level = "invalid".to_string();

        let result = config.validate();
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("Invalid log level: invalid"));
    }

    #[test]
    fn test_app_config_validate_when_valid_log_levels_should_return_ok() {
        let valid_levels = ["trace", "debug", "info", "warn", "error"];

        for level in valid_levels {
            let mut config = AppConfig::default();
            config.log_level = level.to_string();
            assert!(
                config.validate().is_ok(),
                "Log level {} should be valid",
                level
            );
        }
    }

    #[test]
    fn test_app_config_validate_when_zero_gas_price_should_return_err() {
        let mut config = AppConfig::default();
        config.transaction.max_gas_price_gwei = 0;

        let result = config.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_app_config_validate_when_negative_slippage_should_return_err() {
        let mut config = AppConfig::default();
        config.transaction.slippage_tolerance_percent = -1.0;

        let result = config.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_app_config_validate_when_excessive_slippage_should_return_err() {
        let mut config = AppConfig::default();
        config.transaction.slippage_tolerance_percent = 101.0;

        let result = config.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_app_config_validate_when_boundary_slippage_should_return_ok() {
        // Test boundary values for slippage
        let mut config = AppConfig::default();

        // Test 0.0
        config.transaction.slippage_tolerance_percent = 0.0;
        assert!(config.validate().is_ok());

        // Test 100.0
        config.transaction.slippage_tolerance_percent = 100.0;
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_app_config_validate_when_retry_validation_fails_should_return_err() {
        let mut config = AppConfig::default();
        config.retry.max_retry_attempts = 0;

        let result = config.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_retry_config_validate_when_valid_should_return_ok() {
        let config = RetryConfig::default();
        assert!(Validate::validate(&config).is_ok());
    }

    #[test]
    fn test_retry_config_validate_when_zero_attempts_should_return_err() {
        let mut config = RetryConfig::default();
        config.max_retry_attempts = 0;

        let result = Validate::validate(&config);
        assert!(result.is_err());
    }

    #[test]
    fn test_retry_config_validate_when_backoff_multiplier_too_low_should_return_err() {
        let mut config = RetryConfig::default();
        config.retry_backoff_multiplier = 1.0;

        let result = Validate::validate(&config);
        assert!(result.is_err());
    }

    #[test]
    fn test_retry_config_validate_when_backoff_multiplier_negative_should_return_err() {
        let mut config = RetryConfig::default();
        config.retry_backoff_multiplier = 0.5;

        let result = Validate::validate(&config);
        assert!(result.is_err());
    }

    #[test]
    fn test_retry_config_validate_when_max_delay_less_than_initial_should_return_err() {
        let mut config = AppConfig::default();
        config.retry.retry_delay_ms = 5000;
        config.retry.max_retry_delay_ms = 3000;

        let result = config.validate();
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error
            .to_string()
            .contains("max_retry_delay_ms must be >= retry_delay_ms"));
    }

    #[test]
    fn test_retry_config_validate_when_max_delay_equals_initial_should_return_ok() {
        let mut config = AppConfig::default();
        config.retry.retry_delay_ms = 5000;
        config.retry.max_retry_delay_ms = 5000;

        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_app_config_serialization() {
        let config = AppConfig::default();

        // Test serialization
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("\"port\":8080"));
        assert!(json.contains("\"environment\":\"development\""));
        assert!(json.contains("\"log_level\":\"info\""));

        // Test deserialization
        let deserialized: AppConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.port, config.port);
        assert_eq!(deserialized.environment, config.environment);
        assert_eq!(deserialized.log_level, config.log_level);
    }

    #[test]
    fn test_transaction_config_serialization() {
        let config = TransactionConfig::default();

        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("\"max_gas_price_gwei\":100"));
        assert!(json.contains("\"priority_fee_gwei\":2"));
        assert!(json.contains("\"slippage_tolerance_percent\":0.5"));
        assert!(json.contains("\"deadline_seconds\":300"));

        let deserialized: TransactionConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.max_gas_price_gwei, config.max_gas_price_gwei);
        assert_eq!(deserialized.priority_fee_gwei, config.priority_fee_gwei);
        assert_eq!(
            deserialized.slippage_tolerance_percent,
            config.slippage_tolerance_percent
        );
        assert_eq!(deserialized.deadline_seconds, config.deadline_seconds);
    }

    #[test]
    fn test_retry_config_serialization() {
        let config = RetryConfig::default();

        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("\"max_retry_attempts\":3"));
        assert!(json.contains("\"retry_delay_ms\":1000"));
        assert!(json.contains("\"retry_backoff_multiplier\":2.0"));
        assert!(json.contains("\"max_retry_delay_ms\":30000"));

        let deserialized: RetryConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.max_retry_attempts, config.max_retry_attempts);
        assert_eq!(deserialized.retry_delay_ms, config.retry_delay_ms);
        assert_eq!(
            deserialized.retry_backoff_multiplier,
            config.retry_backoff_multiplier
        );
        assert_eq!(deserialized.max_retry_delay_ms, config.max_retry_delay_ms);
    }

    #[test]
    fn test_environment_variants() {
        // Test all environment variants
        assert_eq!(format!("{:?}", Environment::Development), "Development");
        assert_eq!(format!("{:?}", Environment::Staging), "Staging");
        assert_eq!(format!("{:?}", Environment::Production), "Production");

        // Test equality
        assert_eq!(Environment::Development, Environment::Development);
        assert_ne!(Environment::Development, Environment::Production);
    }

    #[test]
    fn test_config_debug_implementation() {
        let config = AppConfig::default();
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("AppConfig"));
        assert!(debug_str.contains("port: 8080"));
        assert!(debug_str.contains("environment: Development"));
    }

    #[test]
    fn test_config_clone_implementation() {
        let config = AppConfig::default();
        let cloned = config.clone();
        assert_eq!(config.port, cloned.port);
        assert_eq!(config.environment, cloned.environment);
        assert_eq!(config.log_level, cloned.log_level);
        assert_eq!(config.use_testnet, cloned.use_testnet);
    }

    #[test]
    fn test_app_config_with_custom_values() {
        let mut config = AppConfig::default();
        config.port = 3000;
        config.environment = Environment::Production;
        config.log_level = "debug".to_string();
        config.use_testnet = true;
        config.transaction.max_gas_price_gwei = 200;
        config.retry.max_retry_attempts = 5;

        assert!(config.validate().is_ok());
        assert_eq!(config.port, 3000);
        assert_eq!(config.environment, Environment::Production);
        assert_eq!(config.log_level, "debug");
        assert!(config.use_testnet);
        assert_eq!(config.transaction.max_gas_price_gwei, 200);
        assert_eq!(config.retry.max_retry_attempts, 5);
    }
}
