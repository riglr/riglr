//! Generic retry utilities for robust async operations
//!
//! This module provides a centralized retry mechanism with exponential backoff,
//! jitter, and error classification for any async operation.

use backoff::{backoff::Backoff, ExponentialBackoff, ExponentialBackoffBuilder};
use std::future::Future;
use std::time::Duration;
use tracing::{debug, warn};

/// Configuration for retry behavior
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_retries: u32,
    /// Initial retry delay in milliseconds
    pub base_delay_ms: u64,
    /// Maximum retry delay in milliseconds
    pub max_delay_ms: u64,
    /// Multiplier for exponential backoff
    pub backoff_multiplier: f64,
    /// Whether to use jitter to avoid thundering herd
    pub use_jitter: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_delay_ms: 1000,     // Start with 1 second
            max_delay_ms: 30_000,    // Cap at 30 seconds
            backoff_multiplier: 2.0, // Double each time
            use_jitter: true,
        }
    }
}

impl RetryConfig {
    /// Create a config for fast retries (e.g., RPC calls)
    pub fn fast() -> Self {
        Self {
            max_retries: 5,
            base_delay_ms: 100,      // Start with 100ms
            max_delay_ms: 5_000,     // Cap at 5 seconds
            backoff_multiplier: 1.5, // Gentler increase
            use_jitter: true,
        }
    }

    /// Create a config for slow retries (e.g., rate-limited APIs)
    pub fn slow() -> Self {
        Self {
            max_retries: 3,
            base_delay_ms: 5000,     // Start with 5 seconds
            max_delay_ms: 60_000,    // Cap at 1 minute
            backoff_multiplier: 2.0, // Double each time
            use_jitter: true,
        }
    }
}

/// Error classification for retry logic
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorClass {
    /// Error is permanent and should not be retried
    Permanent,
    /// Error is temporary and should be retried
    Retryable,
    /// Error indicates rate limiting, use longer backoff
    RateLimited,
}

/// Create a backoff instance from our config
fn create_backoff(config: &RetryConfig) -> ExponentialBackoff {
    let mut builder = ExponentialBackoffBuilder::new();
    builder.with_initial_interval(Duration::from_millis(config.base_delay_ms));
    builder.with_max_interval(Duration::from_millis(config.max_delay_ms));
    builder.with_multiplier(config.backoff_multiplier);
    builder.with_max_elapsed_time(None); // We handle max retries manually

    if !config.use_jitter {
        builder.with_randomization_factor(0.0);
    } else {
        // Default randomization factor is 0.5 (±50%)
        builder.with_randomization_factor(0.25);
    }

    builder.build()
}

/// Execute an async operation with retry logic
///
/// This function provides a generic retry mechanism for any async operation.
/// It automatically applies exponential backoff with optional jitter.
///
/// # Arguments
///
/// * `operation` - Async closure that performs the operation
/// * `classifier` - Function to classify errors as permanent or retryable
/// * `config` - Retry configuration
/// * `operation_name` - Human-readable name for logging
///
/// # Type Parameters
///
/// * `F` - The async operation closure type
/// * `Fut` - The future type returned by the operation
/// * `T` - The success type
/// * `E` - The error type
/// * `C` - The error classifier function type
///
/// # Returns
///
/// Returns the successful result or the last error after all retries are exhausted
///
/// # Examples
///
/// ```rust,ignore
/// use riglr_core::retry::{retry_async, RetryConfig, ErrorClass};
///
/// async fn example() -> Result<String, MyError> {
///     retry_async(
///         || async {
///             // Your async operation here
///             fetch_data().await
///         },
///         |error| {
///             // Classify error
///             match error {
///                 MyError::NetworkTimeout => ErrorClass::Retryable,
///                 MyError::InvalidInput => ErrorClass::Permanent,
///                 MyError::RateLimited => ErrorClass::RateLimited,
///             }
///         },
///         &RetryConfig::default(),
///         "fetch_data"
///     ).await
/// }
/// ```
pub async fn retry_async<F, Fut, T, E, C>(
    mut operation: F,
    classifier: C,
    config: &RetryConfig,
    operation_name: &str,
) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    E: std::fmt::Display + Clone,
    C: Fn(&E) -> ErrorClass,
{
    debug!(
        "Starting operation '{}' with retry config: max_retries={}, base_delay={}ms",
        operation_name, config.max_retries, config.base_delay_ms
    );

    let mut backoff = create_backoff(config);
    let mut attempts = 0u32;

    loop {
        attempts += 1;
        debug!("Attempt {} for '{}'", attempts, operation_name);

        let result = operation().await;
        match result {
            Ok(result) => {
                if attempts > 1 {
                    debug!(
                        "Operation '{}' succeeded after {} attempts",
                        operation_name, attempts
                    );
                }
                return Ok(result);
            }
            Err(error) => {
                let error_class = classifier(&error);

                warn!(
                    "Operation '{}' failed (attempt {}): {} (class: {:?})",
                    operation_name, attempts, error, error_class
                );

                // Check if we should retry
                match error_class {
                    ErrorClass::Permanent => {
                        debug!("Error is permanent, not retrying");
                        return Err(error);
                    }
                    ErrorClass::Retryable | ErrorClass::RateLimited => {
                        if attempts > config.max_retries {
                            warn!(
                                "Operation '{}' failed after {} attempts",
                                operation_name, attempts
                            );
                            return Err(error);
                        }

                        // Get next backoff duration
                        let delay = if let Some(duration) = backoff.next_backoff() {
                            // For rate-limited errors, double the delay
                            if error_class == ErrorClass::RateLimited {
                                duration * 2
                            } else {
                                duration
                            }
                        } else {
                            // Backoff exhausted (shouldn't happen with our config)
                            warn!("Backoff exhausted for '{}'", operation_name);
                            return Err(error);
                        };

                        debug!("Retrying '{}' after {:?}", operation_name, delay);
                        tokio::time::sleep(delay).await;
                    }
                }
            }
        }
    }
}

/// Simplified retry for operations that return std::result::Result
pub async fn retry_with_backoff<F, Fut, T>(
    operation: F,
    config: &RetryConfig,
    operation_name: &str,
) -> Result<T, String>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, String>>,
{
    retry_async(
        operation,
        |_| ErrorClass::Retryable, // Treat all errors as retryable by default
        config,
        operation_name,
    )
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    #[tokio::test]
    async fn test_retry_succeeds_first_attempt() {
        let config = RetryConfig::fast();
        let result = retry_async(
            || async { Ok::<_, String>("success") },
            |_| ErrorClass::Retryable,
            &config,
            "test_op",
        )
        .await;

        assert_eq!(result.unwrap(), "success");
    }

    #[tokio::test]
    async fn test_retry_succeeds_after_failures() {
        let attempts = Arc::new(AtomicU32::new(0));
        let attempts_clone = attempts.clone();

        let config = RetryConfig::fast();
        let result = retry_async(
            || {
                let attempts = attempts_clone.clone();
                async move {
                    let count = attempts.fetch_add(1, Ordering::SeqCst);
                    if count < 2 {
                        Err("temporary failure".to_string())
                    } else {
                        Ok("success")
                    }
                }
            },
            |_| ErrorClass::Retryable,
            &config,
            "test_op",
        )
        .await;

        assert_eq!(result.unwrap(), "success");
        assert_eq!(attempts.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_retry_permanent_error_no_retry() {
        let attempts = Arc::new(AtomicU32::new(0));
        let attempts_clone = attempts.clone();

        let config = RetryConfig::fast();
        let result = retry_async(
            || {
                let attempts = attempts_clone.clone();
                async move {
                    attempts.fetch_add(1, Ordering::SeqCst);
                    Err::<String, _>("permanent error".to_string())
                }
            },
            |_| ErrorClass::Permanent,
            &config,
            "test_op",
        )
        .await;

        assert!(result.is_err());
        assert_eq!(attempts.load(Ordering::SeqCst), 1); // Only one attempt
    }

    #[tokio::test]
    async fn test_retry_exhausts_all_attempts() {
        let attempts = Arc::new(AtomicU32::new(0));
        let attempts_clone = attempts.clone();

        let config = RetryConfig {
            max_retries: 2,
            base_delay_ms: 10,
            max_delay_ms: 100,
            backoff_multiplier: 2.0,
            use_jitter: false,
        };

        let result = retry_async(
            || {
                let attempts = attempts_clone.clone();
                async move {
                    attempts.fetch_add(1, Ordering::SeqCst);
                    Err::<String, _>("always fails".to_string())
                }
            },
            |_| ErrorClass::Retryable,
            &config,
            "test_op",
        )
        .await;

        assert!(result.is_err());
        assert_eq!(attempts.load(Ordering::SeqCst), 3); // Initial + 2 retries
    }

    #[test]
    fn test_create_backoff_config() {
        // Test that create_backoff produces correct ExponentialBackoff from our config
        let config = RetryConfig {
            max_retries: 5,
            base_delay_ms: 100,
            max_delay_ms: 10_000,
            backoff_multiplier: 2.0,
            use_jitter: false,
        };

        let backoff = create_backoff(&config);
        assert_eq!(backoff.initial_interval, Duration::from_millis(100));
        assert_eq!(backoff.max_interval, Duration::from_millis(10_000));
        assert_eq!(backoff.multiplier, 2.0);
        assert_eq!(backoff.randomization_factor, 0.0); // No jitter
    }

    #[test]
    fn test_create_backoff_with_jitter() {
        let config = RetryConfig {
            max_retries: 5,
            base_delay_ms: 100,
            max_delay_ms: 10_000,
            backoff_multiplier: 2.0,
            use_jitter: true,
        };

        let backoff = create_backoff(&config);
        assert_eq!(backoff.randomization_factor, 0.25); // 25% jitter
    }

    #[test]
    fn test_retry_config_presets() {
        let fast = RetryConfig::fast();
        assert_eq!(fast.base_delay_ms, 100);
        assert_eq!(fast.max_retries, 5);

        let slow = RetryConfig::slow();
        assert_eq!(slow.base_delay_ms, 5000);
        assert_eq!(slow.max_retries, 3);
    }
}
