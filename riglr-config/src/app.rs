//! Application-level configuration

use crate::{ConfigError, ConfigResult};
use serde::{Deserialize, Serialize};

/// Application configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AppConfig {
    /// Server port
    #[serde(default = "default_port")]
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
    pub transaction: TransactionConfig,

    /// Retry configuration
    #[serde(flatten)]
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
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TransactionConfig {
    /// Maximum gas price in gwei
    #[serde(default = "default_max_gas_price")]
    pub max_gas_price_gwei: u64,

    /// Priority fee in gwei
    #[serde(default = "default_priority_fee")]
    pub priority_fee_gwei: u64,

    /// Slippage tolerance as percentage (e.g., 0.5 = 0.5%)
    #[serde(default = "default_slippage")]
    pub slippage_tolerance_percent: f64,

    /// Transaction deadline in seconds
    #[serde(default = "default_deadline")]
    pub deadline_seconds: u64,
}

/// Retry configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    #[serde(default = "default_max_retries")]
    pub max_retry_attempts: u32,

    /// Initial retry delay in milliseconds
    #[serde(default = "default_retry_delay")]
    pub retry_delay_ms: u64,

    /// Backoff multiplier for exponential backoff
    #[serde(default = "default_backoff_multiplier")]
    pub retry_backoff_multiplier: f64,

    /// Maximum retry delay in milliseconds
    #[serde(default = "default_max_retry_delay")]
    pub max_retry_delay_ms: u64,
}

impl AppConfig {
    /// Validates the application configuration for correctness
    pub fn validate(&self) -> ConfigResult<()> {
        // Validate log level
        match self.log_level.as_str() {
            "trace" | "debug" | "info" | "warn" | "error" => {}
            _ => {
                return Err(ConfigError::validation(format!(
                    "Invalid log level: {}",
                    self.log_level
                )))
            }
        }

        // Validate transaction settings
        if self.transaction.max_gas_price_gwei == 0 {
            return Err(ConfigError::validation(
                "max_gas_price_gwei must be greater than 0",
            ));
        }

        if self.transaction.slippage_tolerance_percent < 0.0
            || self.transaction.slippage_tolerance_percent > 100.0
        {
            return Err(ConfigError::validation(
                "slippage_tolerance_percent must be between 0 and 100",
            ));
        }

        // Validate retry settings
        self.retry.validate()?;

        Ok(())
    }
}

impl RetryConfig {
    /// Validates the retry configuration for correctness
    pub fn validate(&self) -> ConfigResult<()> {
        if self.max_retry_attempts == 0 {
            return Err(ConfigError::validation(
                "max_retry_attempts must be at least 1",
            ));
        }

        if self.retry_backoff_multiplier <= 1.0 {
            return Err(ConfigError::validation(
                "retry_backoff_multiplier must be greater than 1.0",
            ));
        }

        if self.max_retry_delay_ms < self.retry_delay_ms {
            return Err(ConfigError::validation(
                "max_retry_delay_ms must be >= retry_delay_ms",
            ));
        }

        Ok(())
    }
}

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
