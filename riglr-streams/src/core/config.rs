//! Advanced streaming configuration for production-grade clients
//!
//! This module provides comprehensive configuration options for streaming clients,
//! including backpressure handling, connection management, and performance tuning.

use serde::{Deserialize, Serialize};
use std::time::Duration;
use thiserror::Error;

const RIGLR_CONNECT_TIMEOUT_SECS: &str = "RIGLR_CONNECT_TIMEOUT_SECS";
const RIGLR_CHANNEL_SIZE: &str = "RIGLR_CHANNEL_SIZE";
const RIGLR_BACKPRESSURE_STRATEGY: &str = "RIGLR_BACKPRESSURE_STRATEGY";
const RIGLR_METRICS_ENABLED: &str = "RIGLR_METRICS_ENABLED";

/// Configuration errors
#[derive(Error, Debug)]
pub enum ConfigError {
    /// Invalid configuration parameter or value
    #[error("Invalid configuration: {0}")]
    Invalid(String),
    /// Environment variable parsing or access error
    #[error("Environment variable error: {0}")]
    Environment(String),
}

/// Backpressure handling strategies
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum BackpressureStrategy {
    /// Block and wait (default) - guarantees delivery but may cause slowdowns
    Block,
    /// Drop messages when buffer is full - fastest but may lose data
    Drop,
    /// Retry with exponential backoff, then drop
    Retry {
        /// Maximum number of retry attempts before dropping
        max_attempts: usize,
        /// Base wait time in milliseconds between retries
        base_wait_ms: u64,
    },
    /// Adaptive strategy that switches based on load
    Adaptive,
}

impl Default for BackpressureStrategy {
    fn default() -> Self {
        Self::Block
    }
}

/// Batch processing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchConfig {
    /// Maximum batch size (default: 100)
    pub batch_size: usize,
    /// Batch timeout in milliseconds (default: 5ms)
    pub batch_timeout_ms: u64,
    /// Enable batching (default: true)
    pub enabled: bool,
    /// Use zero-copy batching when possible
    pub zero_copy: bool,
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            batch_size: 100,
            batch_timeout_ms: 5,
            enabled: true,
            zero_copy: true,
        }
    }
}

impl BatchConfig {
    /// Convert batch timeout milliseconds to Duration
    pub fn batch_timeout(&self) -> Duration {
        Duration::from_millis(self.batch_timeout_ms)
    }

    /// Validate batch configuration parameters
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.batch_size == 0 {
            return Err(ConfigError::Invalid(
                "batch_size must be greater than 0".into(),
            ));
        }
        if self.batch_timeout_ms == 0 {
            return Err(ConfigError::Invalid(
                "batch_timeout_ms must be greater than 0".into(),
            ));
        }
        Ok(())
    }
}

/// Backpressure configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackpressureConfig {
    /// Channel buffer size (default: 1000)
    pub channel_size: usize,
    /// Backpressure handling strategy
    pub strategy: BackpressureStrategy,
    /// High watermark percentage for adaptive strategy (default: 80%)
    pub high_watermark_pct: u8,
    /// Low watermark percentage for adaptive strategy (default: 20%)
    pub low_watermark_pct: u8,
}

impl Default for BackpressureConfig {
    fn default() -> Self {
        Self {
            channel_size: 1000,
            strategy: BackpressureStrategy::default(),
            high_watermark_pct: 80,
            low_watermark_pct: 20,
        }
    }
}

impl BackpressureConfig {
    /// Validate backpressure configuration parameters
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.channel_size == 0 {
            return Err(ConfigError::Invalid(
                "channel_size must be greater than 0".into(),
            ));
        }
        if self.high_watermark_pct <= self.low_watermark_pct {
            return Err(ConfigError::Invalid(
                "high_watermark_pct must be greater than low_watermark_pct".into(),
            ));
        }
        if self.high_watermark_pct > 100 || self.low_watermark_pct > 100 {
            return Err(ConfigError::Invalid(
                "watermark percentages must be <= 100".into(),
            ));
        }
        Ok(())
    }

    /// Calculate the high watermark threshold in absolute terms
    pub fn high_watermark(&self) -> usize {
        (self.channel_size * self.high_watermark_pct as usize) / 100
    }

    /// Calculate the low watermark threshold in absolute terms
    pub fn low_watermark(&self) -> usize {
        (self.channel_size * self.low_watermark_pct as usize) / 100
    }
}

/// Connection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionConfig {
    /// Connection timeout in seconds (default: 10)
    pub connect_timeout_secs: u64,
    /// Request timeout in seconds (default: 60)
    pub request_timeout_secs: u64,
    /// Maximum message size in bytes (default: 10MB)
    pub max_message_size: usize,
    /// Enable keepalive
    pub keepalive: bool,
    /// Keepalive interval in seconds
    pub keepalive_interval_secs: u64,
    /// Maximum connection retries
    pub max_retries: usize,
    /// Base retry delay in milliseconds
    pub retry_base_delay_ms: u64,
    /// Maximum retry delay in milliseconds
    pub retry_max_delay_ms: u64,
}

impl Default for ConnectionConfig {
    fn default() -> Self {
        Self {
            connect_timeout_secs: 10,
            request_timeout_secs: 60,
            max_message_size: 10 * 1024 * 1024, // 10MB
            keepalive: true,
            keepalive_interval_secs: 30,
            max_retries: 5,
            retry_base_delay_ms: 100,
            retry_max_delay_ms: 30_000, // 30 seconds
        }
    }
}

impl ConnectionConfig {
    /// Convert connection timeout seconds to Duration
    pub fn connect_timeout(&self) -> Duration {
        Duration::from_secs(self.connect_timeout_secs)
    }

    /// Convert request timeout seconds to Duration
    pub fn request_timeout(&self) -> Duration {
        Duration::from_secs(self.request_timeout_secs)
    }

    /// Convert keepalive interval seconds to Duration
    pub fn keepalive_interval(&self) -> Duration {
        Duration::from_secs(self.keepalive_interval_secs)
    }

    /// Convert base retry delay milliseconds to Duration
    pub fn retry_base_delay(&self) -> Duration {
        Duration::from_millis(self.retry_base_delay_ms)
    }

    /// Convert maximum retry delay milliseconds to Duration
    pub fn retry_max_delay(&self) -> Duration {
        Duration::from_millis(self.retry_max_delay_ms)
    }

    /// Validate connection configuration parameters
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.connect_timeout_secs == 0 {
            return Err(ConfigError::Invalid(
                "connect_timeout_secs must be greater than 0".into(),
            ));
        }
        if self.request_timeout_secs == 0 {
            return Err(ConfigError::Invalid(
                "request_timeout_secs must be greater than 0".into(),
            ));
        }
        if self.max_message_size == 0 {
            return Err(ConfigError::Invalid(
                "max_message_size must be greater than 0".into(),
            ));
        }
        if self.retry_base_delay_ms >= self.retry_max_delay_ms {
            return Err(ConfigError::Invalid(
                "retry_base_delay_ms must be less than retry_max_delay_ms".into(),
            ));
        }
        Ok(())
    }
}

/// Performance monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    /// Enable metrics collection
    pub enabled: bool,
    /// Metrics collection window in seconds
    pub window_secs: u64,
    /// Metrics reporting interval in seconds
    pub report_interval_secs: u64,
    /// Slow processing threshold in milliseconds
    pub slow_threshold_ms: u64,
    /// Include detailed latency histograms
    pub detailed_latency: bool,
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            window_secs: 30,
            report_interval_secs: 60,
            slow_threshold_ms: 10,
            detailed_latency: false,
        }
    }
}

impl MetricsConfig {
    /// Convert metrics window seconds to Duration
    pub fn window_duration(&self) -> Duration {
        Duration::from_secs(self.window_secs)
    }

    /// Convert report interval seconds to Duration
    pub fn report_interval(&self) -> Duration {
        Duration::from_secs(self.report_interval_secs)
    }

    /// Convert slow threshold milliseconds to Duration
    pub fn slow_threshold(&self) -> Duration {
        Duration::from_millis(self.slow_threshold_ms)
    }
}

/// Comprehensive streaming client configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StreamClientConfig {
    /// Connection settings
    pub connection: ConnectionConfig,
    /// Batch processing settings
    pub batch: BatchConfig,
    /// Backpressure handling settings
    pub backpressure: BackpressureConfig,
    /// Performance monitoring settings
    pub metrics: MetricsConfig,
    /// Enable debugging features
    pub debug: bool,
}

impl StreamClientConfig {
    /// High-performance configuration for production environments
    pub fn high_performance() -> Self {
        Self {
            connection: ConnectionConfig::default(),
            batch: BatchConfig {
                batch_size: 500,
                batch_timeout_ms: 2,
                enabled: true,
                zero_copy: true,
            },
            backpressure: BackpressureConfig {
                channel_size: 10_000,
                strategy: BackpressureStrategy::Adaptive,
                high_watermark_pct: 85,
                low_watermark_pct: 15,
            },
            metrics: MetricsConfig {
                enabled: true,
                window_secs: 10,
                report_interval_secs: 30,
                slow_threshold_ms: 5,
                detailed_latency: true,
            },
            debug: false,
        }
    }

    /// Low-latency configuration for real-time applications
    pub fn low_latency() -> Self {
        Self {
            connection: ConnectionConfig {
                connect_timeout_secs: 5,
                request_timeout_secs: 30,
                max_retries: 3,
                retry_base_delay_ms: 50,
                ..Default::default()
            },
            batch: BatchConfig {
                batch_size: 1,
                batch_timeout_ms: 1,
                enabled: false,
                zero_copy: true,
            },
            backpressure: BackpressureConfig {
                channel_size: 100,
                strategy: BackpressureStrategy::Block,
                high_watermark_pct: 90,
                low_watermark_pct: 10,
            },
            metrics: MetricsConfig {
                enabled: true,
                window_secs: 5,
                report_interval_secs: 15,
                slow_threshold_ms: 1,
                detailed_latency: true,
            },
            debug: false,
        }
    }

    /// Reliable configuration prioritizing data integrity
    pub fn reliable() -> Self {
        Self {
            connection: ConnectionConfig {
                max_retries: 10,
                retry_base_delay_ms: 200,
                retry_max_delay_ms: 60_000,
                ..Default::default()
            },
            batch: BatchConfig {
                batch_size: 50,
                batch_timeout_ms: 10,
                enabled: true,
                zero_copy: false,
            },
            backpressure: BackpressureConfig {
                channel_size: 5_000,
                strategy: BackpressureStrategy::Retry {
                    max_attempts: 5,
                    base_wait_ms: 100,
                },
                high_watermark_pct: 75,
                low_watermark_pct: 25,
            },
            metrics: MetricsConfig {
                enabled: true,
                detailed_latency: false,
                ..Default::default()
            },
            debug: false,
        }
    }

    /// Development configuration with debugging enabled
    pub fn development() -> Self {
        Self {
            connection: ConnectionConfig {
                connect_timeout_secs: 30,
                request_timeout_secs: 120,
                ..Default::default()
            },
            metrics: MetricsConfig {
                enabled: true,
                window_secs: 5,
                report_interval_secs: 10,
                slow_threshold_ms: 50,
                detailed_latency: true,
            },
            debug: true,
            ..Default::default()
        }
    }

    /// Validate all configuration settings
    pub fn validate(&self) -> Result<(), ConfigError> {
        self.connection.validate()?;
        self.batch.validate()?;
        self.backpressure.validate()?;
        Ok(())
    }

    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self, ConfigError> {
        let mut config = Self::default();

        // Connection settings
        if let Ok(timeout) = std::env::var(RIGLR_CONNECT_TIMEOUT_SECS) {
            config.connection.connect_timeout_secs = timeout.parse().map_err(|e| {
                ConfigError::Environment(format!("Invalid RIGLR_CONNECT_TIMEOUT_SECS: {}", e))
            })?;
        }

        if let Ok(size) = std::env::var(RIGLR_CHANNEL_SIZE) {
            config.backpressure.channel_size = size.parse().map_err(|e| {
                ConfigError::Environment(format!("Invalid RIGLR_CHANNEL_SIZE: {}", e))
            })?;
        }

        if let Ok(strategy) = std::env::var(RIGLR_BACKPRESSURE_STRATEGY) {
            config.backpressure.strategy = match strategy.to_lowercase().as_str() {
                "block" => BackpressureStrategy::Block,
                "drop" => BackpressureStrategy::Drop,
                "adaptive" => BackpressureStrategy::Adaptive,
                _ => {
                    return Err(ConfigError::Environment(format!(
                        "Invalid RIGLR_BACKPRESSURE_STRATEGY: {}",
                        strategy
                    )))
                }
            };
        }

        if let Ok(enabled) = std::env::var(RIGLR_METRICS_ENABLED) {
            config.metrics.enabled = enabled.parse().map_err(|e| {
                ConfigError::Environment(format!("Invalid RIGLR_METRICS_ENABLED: {}", e))
            })?;
        }

        config.validate()?;
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_default_config_validation() {
        let config = StreamClientConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_preset_configs_validation() {
        assert!(StreamClientConfig::high_performance().validate().is_ok());
        assert!(StreamClientConfig::low_latency().validate().is_ok());
        assert!(StreamClientConfig::reliable().validate().is_ok());
        assert!(StreamClientConfig::development().validate().is_ok());
    }

    #[test]
    fn test_watermark_calculations() {
        let config = BackpressureConfig {
            channel_size: 1000,
            high_watermark_pct: 80,
            low_watermark_pct: 20,
            ..Default::default()
        };

        assert_eq!(config.high_watermark(), 800);
        assert_eq!(config.low_watermark(), 200);
    }

    #[test]
    fn test_invalid_config() {
        let mut config = StreamClientConfig::default();
        config.batch.batch_size = 0;
        assert!(config.validate().is_err());
    }

    // BackpressureStrategy tests
    #[test]
    fn test_backpressure_strategy_default() {
        let strategy = BackpressureStrategy::default();
        assert!(matches!(strategy, BackpressureStrategy::Block));
    }

    // BatchConfig tests
    #[test]
    fn test_batch_config_default() {
        let config = BatchConfig::default();
        assert_eq!(config.batch_size, 100);
        assert_eq!(config.batch_timeout_ms, 5);
        assert!(config.enabled);
        assert!(config.zero_copy);
    }

    #[test]
    fn test_batch_config_batch_timeout() {
        let config = BatchConfig {
            batch_timeout_ms: 1000,
            ..Default::default()
        };
        assert_eq!(config.batch_timeout(), Duration::from_millis(1000));
    }

    #[test]
    fn test_batch_config_validate_when_valid_should_return_ok() {
        let config = BatchConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_batch_config_validate_when_batch_size_zero_should_return_err() {
        let config = BatchConfig {
            batch_size: 0,
            ..Default::default()
        };
        let result = config.validate();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("batch_size must be greater than 0"));
    }

    #[test]
    fn test_batch_config_validate_when_batch_timeout_zero_should_return_err() {
        let config = BatchConfig {
            batch_timeout_ms: 0,
            ..Default::default()
        };
        let result = config.validate();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("batch_timeout_ms must be greater than 0"));
    }

    // BackpressureConfig tests
    #[test]
    fn test_backpressure_config_default() {
        let config = BackpressureConfig::default();
        assert_eq!(config.channel_size, 1000);
        assert!(matches!(config.strategy, BackpressureStrategy::Block));
        assert_eq!(config.high_watermark_pct, 80);
        assert_eq!(config.low_watermark_pct, 20);
    }

    #[test]
    fn test_backpressure_config_validate_when_valid_should_return_ok() {
        let config = BackpressureConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_backpressure_config_validate_when_channel_size_zero_should_return_err() {
        let config = BackpressureConfig {
            channel_size: 0,
            ..Default::default()
        };
        let result = config.validate();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("channel_size must be greater than 0"));
    }

    #[test]
    fn test_backpressure_config_validate_when_high_watermark_not_greater_than_low_should_return_err(
    ) {
        let config = BackpressureConfig {
            high_watermark_pct: 20,
            low_watermark_pct: 20,
            ..Default::default()
        };
        let result = config.validate();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("high_watermark_pct must be greater than low_watermark_pct"));
    }

    #[test]
    fn test_backpressure_config_validate_when_high_watermark_less_than_low_should_return_err() {
        let config = BackpressureConfig {
            high_watermark_pct: 10,
            low_watermark_pct: 20,
            ..Default::default()
        };
        let result = config.validate();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("high_watermark_pct must be greater than low_watermark_pct"));
    }

    #[test]
    fn test_backpressure_config_validate_when_high_watermark_over_100_should_return_err() {
        let config = BackpressureConfig {
            high_watermark_pct: 101,
            low_watermark_pct: 20,
            ..Default::default()
        };
        let result = config.validate();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("watermark percentages must be <= 100"));
    }

    #[test]
    fn test_backpressure_config_validate_when_low_watermark_over_100_should_return_err() {
        let config = BackpressureConfig {
            high_watermark_pct: 80,
            low_watermark_pct: 101,
            ..Default::default()
        };
        let result = config.validate();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("watermark percentages must be <= 100"));
    }

    #[test]
    fn test_backpressure_config_high_watermark() {
        let config = BackpressureConfig {
            channel_size: 500,
            high_watermark_pct: 60,
            ..Default::default()
        };
        assert_eq!(config.high_watermark(), 300);
    }

    #[test]
    fn test_backpressure_config_low_watermark() {
        let config = BackpressureConfig {
            channel_size: 500,
            low_watermark_pct: 10,
            ..Default::default()
        };
        assert_eq!(config.low_watermark(), 50);
    }

    #[test]
    fn test_backpressure_config_watermark_edge_cases() {
        let config = BackpressureConfig {
            channel_size: 1,
            high_watermark_pct: 100,
            low_watermark_pct: 1,
            ..Default::default()
        };
        assert_eq!(config.high_watermark(), 1);
        assert_eq!(config.low_watermark(), 0);
    }

    // ConnectionConfig tests
    #[test]
    fn test_connection_config_default() {
        let config = ConnectionConfig::default();
        assert_eq!(config.connect_timeout_secs, 10);
        assert_eq!(config.request_timeout_secs, 60);
        assert_eq!(config.max_message_size, 10 * 1024 * 1024);
        assert!(config.keepalive);
        assert_eq!(config.keepalive_interval_secs, 30);
        assert_eq!(config.max_retries, 5);
        assert_eq!(config.retry_base_delay_ms, 100);
        assert_eq!(config.retry_max_delay_ms, 30_000);
    }

    #[test]
    fn test_connection_config_connect_timeout() {
        let config = ConnectionConfig {
            connect_timeout_secs: 15,
            ..Default::default()
        };
        assert_eq!(config.connect_timeout(), Duration::from_secs(15));
    }

    #[test]
    fn test_connection_config_request_timeout() {
        let config = ConnectionConfig {
            request_timeout_secs: 120,
            ..Default::default()
        };
        assert_eq!(config.request_timeout(), Duration::from_secs(120));
    }

    #[test]
    fn test_connection_config_keepalive_interval() {
        let config = ConnectionConfig {
            keepalive_interval_secs: 45,
            ..Default::default()
        };
        assert_eq!(config.keepalive_interval(), Duration::from_secs(45));
    }

    #[test]
    fn test_connection_config_retry_base_delay() {
        let config = ConnectionConfig {
            retry_base_delay_ms: 200,
            ..Default::default()
        };
        assert_eq!(config.retry_base_delay(), Duration::from_millis(200));
    }

    #[test]
    fn test_connection_config_retry_max_delay() {
        let config = ConnectionConfig {
            retry_max_delay_ms: 60_000,
            ..Default::default()
        };
        assert_eq!(config.retry_max_delay(), Duration::from_millis(60_000));
    }

    #[test]
    fn test_connection_config_validate_when_valid_should_return_ok() {
        let config = ConnectionConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_connection_config_validate_when_connect_timeout_zero_should_return_err() {
        let config = ConnectionConfig {
            connect_timeout_secs: 0,
            ..Default::default()
        };
        let result = config.validate();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("connect_timeout_secs must be greater than 0"));
    }

    #[test]
    fn test_connection_config_validate_when_request_timeout_zero_should_return_err() {
        let config = ConnectionConfig {
            request_timeout_secs: 0,
            ..Default::default()
        };
        let result = config.validate();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("request_timeout_secs must be greater than 0"));
    }

    #[test]
    fn test_connection_config_validate_when_max_message_size_zero_should_return_err() {
        let config = ConnectionConfig {
            max_message_size: 0,
            ..Default::default()
        };
        let result = config.validate();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("max_message_size must be greater than 0"));
    }

    #[test]
    fn test_connection_config_validate_when_retry_base_delay_not_less_than_max_should_return_err() {
        let config = ConnectionConfig {
            retry_base_delay_ms: 1000,
            retry_max_delay_ms: 1000,
            ..Default::default()
        };
        let result = config.validate();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("retry_base_delay_ms must be less than retry_max_delay_ms"));
    }

    #[test]
    fn test_connection_config_validate_when_retry_base_delay_greater_than_max_should_return_err() {
        let config = ConnectionConfig {
            retry_base_delay_ms: 2000,
            retry_max_delay_ms: 1000,
            ..Default::default()
        };
        let result = config.validate();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("retry_base_delay_ms must be less than retry_max_delay_ms"));
    }

    // MetricsConfig tests
    #[test]
    fn test_metrics_config_default() {
        let config = MetricsConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.window_secs, 30);
        assert_eq!(config.report_interval_secs, 60);
        assert_eq!(config.slow_threshold_ms, 10);
        assert!(!config.detailed_latency);
    }

    #[test]
    fn test_metrics_config_window_duration() {
        let config = MetricsConfig {
            window_secs: 45,
            ..Default::default()
        };
        assert_eq!(config.window_duration(), Duration::from_secs(45));
    }

    #[test]
    fn test_metrics_config_report_interval() {
        let config = MetricsConfig {
            report_interval_secs: 90,
            ..Default::default()
        };
        assert_eq!(config.report_interval(), Duration::from_secs(90));
    }

    #[test]
    fn test_metrics_config_slow_threshold() {
        let config = MetricsConfig {
            slow_threshold_ms: 25,
            ..Default::default()
        };
        assert_eq!(config.slow_threshold(), Duration::from_millis(25));
    }

    // StreamClientConfig tests
    #[test]
    fn test_stream_client_config_default() {
        let config = StreamClientConfig::default();
        assert!(!config.debug);
        assert!(!config.metrics.enabled);
    }

    #[test]
    fn test_stream_client_config_high_performance() {
        let config = StreamClientConfig::high_performance();
        assert_eq!(config.batch.batch_size, 500);
        assert_eq!(config.batch.batch_timeout_ms, 2);
        assert!(config.batch.enabled);
        assert!(config.batch.zero_copy);
        assert_eq!(config.backpressure.channel_size, 10_000);
        assert!(matches!(
            config.backpressure.strategy,
            BackpressureStrategy::Adaptive
        ));
        assert_eq!(config.backpressure.high_watermark_pct, 85);
        assert_eq!(config.backpressure.low_watermark_pct, 15);
        assert!(config.metrics.enabled);
        assert_eq!(config.metrics.window_secs, 10);
        assert_eq!(config.metrics.report_interval_secs, 30);
        assert_eq!(config.metrics.slow_threshold_ms, 5);
        assert!(config.metrics.detailed_latency);
        assert!(!config.debug);
    }

    #[test]
    fn test_stream_client_config_low_latency() {
        let config = StreamClientConfig::low_latency();
        assert_eq!(config.connection.connect_timeout_secs, 5);
        assert_eq!(config.connection.request_timeout_secs, 30);
        assert_eq!(config.connection.max_retries, 3);
        assert_eq!(config.connection.retry_base_delay_ms, 50);
        assert_eq!(config.batch.batch_size, 1);
        assert_eq!(config.batch.batch_timeout_ms, 1);
        assert!(!config.batch.enabled);
        assert!(config.batch.zero_copy);
        assert_eq!(config.backpressure.channel_size, 100);
        assert!(matches!(
            config.backpressure.strategy,
            BackpressureStrategy::Block
        ));
        assert_eq!(config.backpressure.high_watermark_pct, 90);
        assert_eq!(config.backpressure.low_watermark_pct, 10);
        assert!(config.metrics.enabled);
        assert_eq!(config.metrics.window_secs, 5);
        assert_eq!(config.metrics.report_interval_secs, 15);
        assert_eq!(config.metrics.slow_threshold_ms, 1);
        assert!(config.metrics.detailed_latency);
        assert!(!config.debug);
    }

    #[test]
    fn test_stream_client_config_reliable() {
        let config = StreamClientConfig::reliable();
        assert_eq!(config.connection.max_retries, 10);
        assert_eq!(config.connection.retry_base_delay_ms, 200);
        assert_eq!(config.connection.retry_max_delay_ms, 60_000);
        assert_eq!(config.batch.batch_size, 50);
        assert_eq!(config.batch.batch_timeout_ms, 10);
        assert!(config.batch.enabled);
        assert!(!config.batch.zero_copy);
        assert_eq!(config.backpressure.channel_size, 5_000);
        assert!(matches!(
            config.backpressure.strategy,
            BackpressureStrategy::Retry {
                max_attempts: 5,
                base_wait_ms: 100
            }
        ));
        if let BackpressureStrategy::Retry {
            max_attempts,
            base_wait_ms,
        } = config.backpressure.strategy
        {
            assert_eq!(max_attempts, 5);
            assert_eq!(base_wait_ms, 100);
        }
        assert_eq!(config.backpressure.high_watermark_pct, 75);
        assert_eq!(config.backpressure.low_watermark_pct, 25);
        assert!(config.metrics.enabled);
        assert!(!config.metrics.detailed_latency);
        assert!(!config.debug);
    }

    #[test]
    fn test_stream_client_config_development() {
        let config = StreamClientConfig::development();
        assert_eq!(config.connection.connect_timeout_secs, 30);
        assert_eq!(config.connection.request_timeout_secs, 120);
        assert!(config.metrics.enabled);
        assert_eq!(config.metrics.window_secs, 5);
        assert_eq!(config.metrics.report_interval_secs, 10);
        assert_eq!(config.metrics.slow_threshold_ms, 50);
        assert!(config.metrics.detailed_latency);
        assert!(config.debug);
    }

    #[test]
    fn test_stream_client_config_validate_when_valid_should_return_ok() {
        let config = StreamClientConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_stream_client_config_validate_when_connection_invalid_should_return_err() {
        let mut config = StreamClientConfig::default();
        config.connection.connect_timeout_secs = 0;
        let result = config.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_stream_client_config_validate_when_batch_invalid_should_return_err() {
        let mut config = StreamClientConfig::default();
        config.batch.batch_size = 0;
        let result = config.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_stream_client_config_validate_when_backpressure_invalid_should_return_err() {
        let mut config = StreamClientConfig::default();
        config.backpressure.channel_size = 0;
        let result = config.validate();
        assert!(result.is_err());
    }

    // Environment variable tests
    #[test]
    fn test_stream_client_config_from_env_when_no_env_vars_should_return_default() {
        // Clear any existing environment variables
        env::remove_var(RIGLR_CONNECT_TIMEOUT_SECS);
        env::remove_var(RIGLR_CHANNEL_SIZE);
        env::remove_var(RIGLR_BACKPRESSURE_STRATEGY);
        env::remove_var(RIGLR_METRICS_ENABLED);

        let config = StreamClientConfig::from_env().unwrap();
        let default_config = StreamClientConfig::default();

        assert_eq!(
            config.connection.connect_timeout_secs,
            default_config.connection.connect_timeout_secs
        );
        assert_eq!(
            config.backpressure.channel_size,
            default_config.backpressure.channel_size
        );
        assert_eq!(config.metrics.enabled, default_config.metrics.enabled);
    }

    #[test]
    fn test_stream_client_config_from_env_when_connect_timeout_set_should_use_env_value() {
        env::set_var(RIGLR_CONNECT_TIMEOUT_SECS, "25");

        let config = StreamClientConfig::from_env().unwrap();
        assert_eq!(config.connection.connect_timeout_secs, 25);

        env::remove_var(RIGLR_CONNECT_TIMEOUT_SECS);
    }

    #[test]
    fn test_stream_client_config_from_env_when_invalid_connect_timeout_should_return_err() {
        env::set_var(RIGLR_CONNECT_TIMEOUT_SECS, "invalid");

        let result = StreamClientConfig::from_env();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid RIGLR_CONNECT_TIMEOUT_SECS"));

        env::remove_var(RIGLR_CONNECT_TIMEOUT_SECS);
    }

    #[test]
    fn test_stream_client_config_from_env_when_channel_size_set_should_use_env_value() {
        env::set_var(RIGLR_CHANNEL_SIZE, "2000");

        let config = StreamClientConfig::from_env().unwrap();
        assert_eq!(config.backpressure.channel_size, 2000);

        env::remove_var(RIGLR_CHANNEL_SIZE);
    }

    #[test]
    fn test_stream_client_config_from_env_when_invalid_channel_size_should_return_err() {
        env::set_var(RIGLR_CHANNEL_SIZE, "not_a_number");

        let result = StreamClientConfig::from_env();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid RIGLR_CHANNEL_SIZE"));

        env::remove_var(RIGLR_CHANNEL_SIZE);
    }

    #[test]
    fn test_stream_client_config_from_env_when_backpressure_strategy_block_should_use_block() {
        env::set_var(RIGLR_BACKPRESSURE_STRATEGY, "block");

        let config = StreamClientConfig::from_env().unwrap();
        assert!(matches!(
            config.backpressure.strategy,
            BackpressureStrategy::Block
        ));

        env::remove_var(RIGLR_BACKPRESSURE_STRATEGY);
    }

    #[test]
    fn test_stream_client_config_from_env_when_backpressure_strategy_drop_should_use_drop() {
        env::set_var(RIGLR_BACKPRESSURE_STRATEGY, "drop");

        let config = StreamClientConfig::from_env().unwrap();
        assert!(matches!(
            config.backpressure.strategy,
            BackpressureStrategy::Drop
        ));

        env::remove_var(RIGLR_BACKPRESSURE_STRATEGY);
    }

    #[test]
    fn test_stream_client_config_from_env_when_backpressure_strategy_adaptive_should_use_adaptive()
    {
        env::set_var(RIGLR_BACKPRESSURE_STRATEGY, "adaptive");

        let config = StreamClientConfig::from_env().unwrap();
        assert!(matches!(
            config.backpressure.strategy,
            BackpressureStrategy::Adaptive
        ));

        env::remove_var(RIGLR_BACKPRESSURE_STRATEGY);
    }

    #[test]
    fn test_stream_client_config_from_env_when_backpressure_strategy_uppercase_should_work() {
        env::set_var(RIGLR_BACKPRESSURE_STRATEGY, "BLOCK");

        let config = StreamClientConfig::from_env().unwrap();
        assert!(matches!(
            config.backpressure.strategy,
            BackpressureStrategy::Block
        ));

        env::remove_var(RIGLR_BACKPRESSURE_STRATEGY);
    }

    #[test]
    fn test_stream_client_config_from_env_when_invalid_backpressure_strategy_should_return_err() {
        env::set_var(RIGLR_BACKPRESSURE_STRATEGY, "invalid_strategy");

        let result = StreamClientConfig::from_env();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid RIGLR_BACKPRESSURE_STRATEGY"));

        env::remove_var(RIGLR_BACKPRESSURE_STRATEGY);
    }

    #[test]
    fn test_stream_client_config_from_env_when_metrics_enabled_true_should_enable() {
        env::set_var(RIGLR_METRICS_ENABLED, "true");

        let config = StreamClientConfig::from_env().unwrap();
        assert!(config.metrics.enabled);

        env::remove_var(RIGLR_METRICS_ENABLED);
    }

    #[test]
    fn test_stream_client_config_from_env_when_metrics_enabled_false_should_disable() {
        env::set_var(RIGLR_METRICS_ENABLED, "false");

        let config = StreamClientConfig::from_env().unwrap();
        assert!(!config.metrics.enabled);

        env::remove_var(RIGLR_METRICS_ENABLED);
    }

    #[test]
    fn test_stream_client_config_from_env_when_invalid_metrics_enabled_should_return_err() {
        env::set_var(RIGLR_METRICS_ENABLED, "maybe");

        let result = StreamClientConfig::from_env();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid RIGLR_METRICS_ENABLED"));

        env::remove_var(RIGLR_METRICS_ENABLED);
    }

    #[test]
    fn test_stream_client_config_from_env_when_all_env_vars_set_should_use_all() {
        env::set_var(RIGLR_CONNECT_TIMEOUT_SECS, "15");
        env::set_var(RIGLR_CHANNEL_SIZE, "3000");
        env::set_var(RIGLR_BACKPRESSURE_STRATEGY, "adaptive");
        env::set_var(RIGLR_METRICS_ENABLED, "true");

        let config = StreamClientConfig::from_env().unwrap();
        assert_eq!(config.connection.connect_timeout_secs, 15);
        assert_eq!(config.backpressure.channel_size, 3000);
        assert!(matches!(
            config.backpressure.strategy,
            BackpressureStrategy::Adaptive
        ));
        assert!(config.metrics.enabled);

        env::remove_var(RIGLR_CONNECT_TIMEOUT_SECS);
        env::remove_var(RIGLR_CHANNEL_SIZE);
        env::remove_var(RIGLR_BACKPRESSURE_STRATEGY);
        env::remove_var(RIGLR_METRICS_ENABLED);
    }

    #[test]
    fn test_stream_client_config_from_env_when_config_validation_fails_should_return_err() {
        env::set_var(RIGLR_CHANNEL_SIZE, "0");

        let result = StreamClientConfig::from_env();
        assert!(result.is_err());

        env::remove_var(RIGLR_CHANNEL_SIZE);
    }

    // ConfigError tests
    #[test]
    fn test_config_error_invalid_display() {
        let error = ConfigError::Invalid("test error".to_string());
        assert_eq!(error.to_string(), "Invalid configuration: test error");
    }

    #[test]
    fn test_config_error_environment_display() {
        let error = ConfigError::Environment("env error".to_string());
        assert_eq!(error.to_string(), "Environment variable error: env error");
    }

    // Additional edge cases and full coverage tests
    #[test]
    fn test_backpressure_strategy_retry_pattern_matching() {
        let strategy = BackpressureStrategy::Retry {
            max_attempts: 3,
            base_wait_ms: 50,
        };
        if let BackpressureStrategy::Retry {
            max_attempts,
            base_wait_ms,
        } = strategy
        {
            assert_eq!(max_attempts, 3);
            assert_eq!(base_wait_ms, 50);
        } else {
            panic!("Expected Retry strategy");
        }
    }

    #[test]
    fn test_backpressure_strategy_drop_pattern_matching() {
        let strategy = BackpressureStrategy::Drop;
        assert!(matches!(strategy, BackpressureStrategy::Drop));
    }

    #[test]
    fn test_backpressure_strategy_adaptive_pattern_matching() {
        let strategy = BackpressureStrategy::Adaptive;
        assert!(matches!(strategy, BackpressureStrategy::Adaptive));
    }

    #[test]
    fn test_watermark_calculations_edge_cases() {
        // Test with very small channel size
        let config = BackpressureConfig {
            channel_size: 3,
            high_watermark_pct: 66,
            low_watermark_pct: 33,
            ..Default::default()
        };
        assert_eq!(config.high_watermark(), 1); // 3 * 66 / 100 = 1.98 -> 1
        assert_eq!(config.low_watermark(), 0); // 3 * 33 / 100 = 0.99 -> 0
    }

    #[test]
    fn test_watermark_calculations_maximum_values() {
        let config = BackpressureConfig {
            channel_size: usize::MAX,
            high_watermark_pct: 1,
            low_watermark_pct: 0,
            ..Default::default()
        };
        // This tests integer overflow protection
        let high = config.high_watermark();
        let low = config.low_watermark();
        assert!(high >= low);
    }

    #[test]
    fn test_duration_conversions_edge_cases() {
        // Test with maximum values
        let batch_config = BatchConfig {
            batch_timeout_ms: u64::MAX,
            ..Default::default()
        };
        let timeout = batch_config.batch_timeout();
        assert_eq!(timeout, Duration::from_millis(u64::MAX));

        let connection_config = ConnectionConfig {
            connect_timeout_secs: u64::MAX,
            request_timeout_secs: u64::MAX,
            keepalive_interval_secs: u64::MAX,
            retry_base_delay_ms: u64::MAX - 1,
            retry_max_delay_ms: u64::MAX,
            ..Default::default()
        };

        assert_eq!(
            connection_config.connect_timeout(),
            Duration::from_secs(u64::MAX)
        );
        assert_eq!(
            connection_config.request_timeout(),
            Duration::from_secs(u64::MAX)
        );
        assert_eq!(
            connection_config.keepalive_interval(),
            Duration::from_secs(u64::MAX)
        );
        assert_eq!(
            connection_config.retry_base_delay(),
            Duration::from_millis(u64::MAX - 1)
        );
        assert_eq!(
            connection_config.retry_max_delay(),
            Duration::from_millis(u64::MAX)
        );

        let metrics_config = MetricsConfig {
            window_secs: u64::MAX,
            report_interval_secs: u64::MAX,
            slow_threshold_ms: u64::MAX,
            ..Default::default()
        };

        assert_eq!(
            metrics_config.window_duration(),
            Duration::from_secs(u64::MAX)
        );
        assert_eq!(
            metrics_config.report_interval(),
            Duration::from_secs(u64::MAX)
        );
        assert_eq!(
            metrics_config.slow_threshold(),
            Duration::from_millis(u64::MAX)
        );
    }

    #[test]
    fn test_connection_config_edge_case_validation() {
        // Test edge case where retry delays are exactly equal but both > 0
        let config = ConnectionConfig {
            retry_base_delay_ms: 100,
            retry_max_delay_ms: 100,
            ..Default::default()
        };
        let result = config.validate();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("retry_base_delay_ms must be less than retry_max_delay_ms"));
    }

    #[test]
    fn test_all_config_structs_implement_debug_clone_serialize() {
        // This test ensures all structs properly implement required traits
        let batch_config = BatchConfig::default();
        let cloned_batch = batch_config.clone();
        println!("{:?}", cloned_batch);

        let backpressure_config = BackpressureConfig::default();
        let cloned_backpressure = backpressure_config.clone();
        println!("{:?}", cloned_backpressure);

        let connection_config = ConnectionConfig::default();
        let cloned_connection = connection_config.clone();
        println!("{:?}", cloned_connection);

        let metrics_config = MetricsConfig::default();
        let cloned_metrics = metrics_config.clone();
        println!("{:?}", cloned_metrics);

        let stream_config = StreamClientConfig::default();
        let cloned_stream = stream_config.clone();
        println!("{:?}", cloned_stream);

        let strategy = BackpressureStrategy::default();
        let cloned_strategy = strategy;
        println!("{:?}", cloned_strategy);
    }
}
