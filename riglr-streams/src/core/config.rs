//! Advanced streaming configuration for production-grade clients
//!
//! This module provides comprehensive configuration options for streaming clients,
//! including backpressure handling, connection management, and performance tuning.

use serde::{Deserialize, Serialize};
use std::time::Duration;
use thiserror::Error;

/// Configuration errors
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Invalid configuration: {0}")]
    Invalid(String),
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
    Retry { max_attempts: usize, base_wait_ms: u64 },
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
    pub fn batch_timeout(&self) -> Duration {
        Duration::from_millis(self.batch_timeout_ms)
    }

    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.batch_size == 0 {
            return Err(ConfigError::Invalid("batch_size must be greater than 0".into()));
        }
        if self.batch_timeout_ms == 0 {
            return Err(ConfigError::Invalid("batch_timeout_ms must be greater than 0".into()));
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
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.channel_size == 0 {
            return Err(ConfigError::Invalid("channel_size must be greater than 0".into()));
        }
        if self.high_watermark_pct <= self.low_watermark_pct {
            return Err(ConfigError::Invalid("high_watermark_pct must be greater than low_watermark_pct".into()));
        }
        if self.high_watermark_pct > 100 || self.low_watermark_pct > 100 {
            return Err(ConfigError::Invalid("watermark percentages must be <= 100".into()));
        }
        Ok(())
    }

    pub fn high_watermark(&self) -> usize {
        (self.channel_size * self.high_watermark_pct as usize) / 100
    }

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
    pub fn connect_timeout(&self) -> Duration {
        Duration::from_secs(self.connect_timeout_secs)
    }

    pub fn request_timeout(&self) -> Duration {
        Duration::from_secs(self.request_timeout_secs)
    }

    pub fn keepalive_interval(&self) -> Duration {
        Duration::from_secs(self.keepalive_interval_secs)
    }

    pub fn retry_base_delay(&self) -> Duration {
        Duration::from_millis(self.retry_base_delay_ms)
    }

    pub fn retry_max_delay(&self) -> Duration {
        Duration::from_millis(self.retry_max_delay_ms)
    }

    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.connect_timeout_secs == 0 {
            return Err(ConfigError::Invalid("connect_timeout_secs must be greater than 0".into()));
        }
        if self.request_timeout_secs == 0 {
            return Err(ConfigError::Invalid("request_timeout_secs must be greater than 0".into()));
        }
        if self.max_message_size == 0 {
            return Err(ConfigError::Invalid("max_message_size must be greater than 0".into()));
        }
        if self.retry_base_delay_ms >= self.retry_max_delay_ms {
            return Err(ConfigError::Invalid("retry_base_delay_ms must be less than retry_max_delay_ms".into()));
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
    pub fn window_duration(&self) -> Duration {
        Duration::from_secs(self.window_secs)
    }

    pub fn report_interval(&self) -> Duration {
        Duration::from_secs(self.report_interval_secs)
    }

    pub fn slow_threshold(&self) -> Duration {
        Duration::from_millis(self.slow_threshold_ms)
    }
}

/// Comprehensive streaming client configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
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
        if let Ok(timeout) = std::env::var("RIGLR_CONNECT_TIMEOUT_SECS") {
            config.connection.connect_timeout_secs = timeout.parse()
                .map_err(|e| ConfigError::Environment(format!("Invalid RIGLR_CONNECT_TIMEOUT_SECS: {}", e)))?;
        }
        
        if let Ok(size) = std::env::var("RIGLR_CHANNEL_SIZE") {
            config.backpressure.channel_size = size.parse()
                .map_err(|e| ConfigError::Environment(format!("Invalid RIGLR_CHANNEL_SIZE: {}", e)))?;
        }
        
        if let Ok(strategy) = std::env::var("RIGLR_BACKPRESSURE_STRATEGY") {
            config.backpressure.strategy = match strategy.to_lowercase().as_str() {
                "block" => BackpressureStrategy::Block,
                "drop" => BackpressureStrategy::Drop,
                "adaptive" => BackpressureStrategy::Adaptive,
                _ => return Err(ConfigError::Environment(format!("Invalid RIGLR_BACKPRESSURE_STRATEGY: {}", strategy))),
            };
        }
        
        if let Ok(enabled) = std::env::var("RIGLR_METRICS_ENABLED") {
            config.metrics.enabled = enabled.parse()
                .map_err(|e| ConfigError::Environment(format!("Invalid RIGLR_METRICS_ENABLED: {}", e)))?;
        }
        
        config.validate()?;
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}