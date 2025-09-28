//! Generic retry utilities for robust async operations
//!
//! This module provides a centralized retry mechanism with exponential backoff,
//! jitter, and error classification for any async operation.

use backoff::{backoff::Backoff as _, ExponentialBackoff, ExponentialBackoffBuilder};
use core::{fmt::Display, future::Future, time::Duration};
use tracing::{debug, warn};

// Backward compatibility aliases
#[expect(clippy::module_name_repetitions)]
pub use async_with_classification as retry_async;
#[expect(clippy::module_name_repetitions)]
pub use Config as RetryConfig;

/// Configuration for retry behavior
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Config {
    /// Multiplier for exponential backoff
    pub backoff_multiplier: f64,
    /// Initial retry delay in milliseconds
    pub base_delay_ms: u64,
    /// Maximum retry delay in milliseconds
    pub max_delay_ms: u64,
    /// Maximum number of retry attempts
    pub max_retries: u32,
    /// Whether to use jitter to avoid thundering herd
    pub use_jitter: bool,
}

impl Default for Config {
    #[inline]
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

impl Config {
    /// Create a config for fast retries (e.g., RPC calls)
    #[must_use]
    #[inline]
    pub const fn fast() -> Self {
        Self::new(5, 100, 5_000, 1.5, true)
    }

    /// Create a new retry configuration
    #[must_use]
    #[inline]
    pub const fn new(
        max_retries: u32,
        base_delay_ms: u64,
        max_delay_ms: u64,
        backoff_multiplier: f64,
        use_jitter: bool,
    ) -> Self {
        Self {
            backoff_multiplier,
            base_delay_ms,
            max_delay_ms,
            max_retries,
            use_jitter,
        }
    }

    /// Create a config for slow retries (e.g., rate-limited APIs)
    #[must_use]
    #[inline]
    pub const fn slow() -> Self {
        Self::new(3, 5000, 60_000, 2.0, true)
    }
}

/// Error classification for retry logic
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ErrorClass {
    /// Error is permanent and should not be retried
    Permanent,
    /// Error indicates rate limiting, use longer backoff
    RateLimited,
    /// Error is temporary and should be retried
    Retryable,
}

/// Create a backoff instance from our config
fn create_backoff(config: &Config) -> ExponentialBackoff {
    let mut builder = ExponentialBackoffBuilder::new();
    builder.with_initial_interval(Duration::from_millis(config.base_delay_ms));
    builder.with_max_interval(Duration::from_millis(config.max_delay_ms));
    builder.with_multiplier(config.backoff_multiplier);
    builder.with_max_elapsed_time(None); // We handle max retries manually

    if config.use_jitter {
        // Default randomization factor is 0.5 (±50%)
        builder.with_randomization_factor(0.25);
    } else {
        builder.with_randomization_factor(0.0);
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
/// use riglr_core::retry::{async_with_classification, Config, ErrorClass};
///
/// async fn example() -> Result<String, MyError> {
///     async_with_classification(
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
///         &Config::default(),
///         "fetch_data"
///     ).await
/// }
/// ```
/// Helper function to handle successful operation completion
fn handle_success<T>(result: T, attempts: u32, operation_name: &str) -> T {
    if attempts > 1 {
        debug!(
            "Operation '{}' succeeded after {} attempts",
            operation_name, attempts
        );
    }
    result
}

/// Helper function to determine if we should retry based on error classification
fn should_retry(
    error_class: ErrorClass,
    attempts: u32,
    config: &Config,
    operation_name: &str,
) -> bool {
    match error_class {
        ErrorClass::Permanent => {
            debug!("Error is permanent, not retrying");
            false
        }
        ErrorClass::Retryable | ErrorClass::RateLimited => {
            if attempts > config.max_retries {
                warn!(
                    "Operation '{}' failed after {} attempts",
                    operation_name, attempts
                );
                false
            } else {
                true
            }
        }
    }
}

/// Helper function to calculate retry delay based on error type
///
/// # Errors
/// Returns `Err(())` if the backoff is exhausted
fn calculate_retry_delay(
    backoff: &mut ExponentialBackoff,
    error_class: ErrorClass,
    operation_name: &str,
) -> Result<Duration, ()> {
    let backoff_result = backoff.next_backoff();
    let delay_result = backoff_result.map_or_else(
        || {
            // Backoff exhausted (shouldn't happen with our config)
            warn!("Backoff exhausted for '{}'", operation_name);
            Err(())
        },
        |duration| {
            // For rate-limited errors, double the delay
            let delay = if error_class == ErrorClass::RateLimited {
                #[expect(clippy::arithmetic_side_effects)]
                let doubled = duration * 2;
                doubled
            } else {
                duration
            };
            debug!("Retrying '{}' after {:?}", operation_name, delay);
            Ok(delay)
        },
    );
    delay_result
}

/// Helper function to handle operation errors
///
/// # Errors
/// Returns the original error if retry should not be attempted
async fn handle_operation_error<E, C>(
    error: E,
    classifier: &C,
    attempts: u32,
    config: &Config,
    operation_name: &str,
    backoff: &mut ExponentialBackoff,
) -> Result<(), E>
where
    E: Display + Clone + Send,
    C: Fn(&E) -> ErrorClass + Sync,
{
    use tokio::time::sleep;

    let error_class = classifier(&error);

    warn!(
        "Operation '{}' failed (attempt {}): {} (class: {:?})",
        operation_name, attempts, error, error_class
    );

    if !should_retry(error_class, attempts, config, operation_name) {
        return Err(error);
    }

    match calculate_retry_delay(backoff, error_class, operation_name) {
        Ok(delay) => {
            sleep(delay).await;
            Ok(())
        }
        Err(()) => Err(error),
    }
}

/// Execute an async operation with retry logic
///
/// # Errors
/// Returns the last error encountered if all retries are exhausted
#[inline]
pub async fn async_with_classification<F, Fut, T, E, C>(
    mut operation: F,
    classifier: C,
    config: &Config,
    operation_name: &str,
) -> Result<T, E>
where
    F: FnMut() -> Fut + Send,
    Fut: Future<Output = Result<T, E>> + Send,
    T: Send,
    E: Display + Clone + Send,
    C: Fn(&E) -> ErrorClass + Send + Sync,
{
    debug!(
        "Starting operation '{}' with retry config: max_retries={}, base_delay={}ms",
        operation_name, config.max_retries, config.base_delay_ms
    );

    let mut backoff = create_backoff(config);
    let mut attempts = 0u32;

    loop {
        {
            #[expect(clippy::arithmetic_side_effects)]
            {
                attempts += 1;
            }
        }
        debug!("Attempt {} for '{}'", attempts, operation_name);

        let operation_result = operation().await;
        match operation_result {
            Ok(result) => {
                let success_result = handle_success(result, attempts, operation_name);
                return Ok(success_result);
            }
            Err(error) => {
                let error_handling_result = handle_operation_error(
                    error,
                    &classifier,
                    attempts,
                    config,
                    operation_name,
                    &mut backoff,
                )
                .await;
                match error_handling_result {
                    Ok(()) => {}                 // Retry (continue loop)
                    Err(err) => return Err(err), // Give up
                }
            }
        }
    }
}

/// Simplified retry for operations that return `Result<T, String>`
///
/// # Errors
/// Returns the last error encountered if all retries are exhausted
#[inline]
pub async fn with_backoff<F, Fut, T>(
    operation: F,
    config: &Config,
    operation_name: &str,
) -> Result<T, String>
where
    F: FnMut() -> Fut + Send,
    Fut: Future<Output = Result<T, String>> + Send,
    T: Send,
{
    async_with_classification(
        operation,
        |_error| ErrorClass::Retryable, // Treat all errors as retryable by default
        config,
        operation_name,
    )
    .await
}

#[cfg(test)]
#[expect(clippy::expect_used)]
mod tests {
    use super::*;
    use alloc::sync::Arc;
    use core::sync::atomic::{AtomicU32, Ordering};

    #[tokio::test]
    async fn retry_succeeds_first_attempt() {
        let config = Config::fast();
        let result = async_with_classification(
            || async { Ok::<_, String>("success") },
            |_error| ErrorClass::Retryable,
            &config,
            "test_op",
        )
        .await;

        {
            assert_eq!(result.expect("Retry should succeed"), "success");
        }
    }

    #[tokio::test]
    async fn retry_succeeds_after_failures() {
        let attempts = Arc::new(AtomicU32::new(0));
        let attempts_clone = Arc::<AtomicU32>::clone(&attempts);

        let config = Config::fast();
        let result = async_with_classification(
            || {
                let inner_attempts = Arc::<AtomicU32>::clone(&attempts_clone);
                async move {
                    let count = inner_attempts.fetch_add(1, Ordering::SeqCst);
                    if count < 2 {
                        Err("temporary failure".to_owned())
                    } else {
                        Ok("success")
                    }
                }
            },
            |_error| ErrorClass::Retryable,
            &config,
            "test_op",
        )
        .await;

        {
            assert_eq!(
                result.expect("Retry should succeed after failures"),
                "success"
            );
        }
        assert_eq!(attempts.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn retry_permanent_error_no_retry() {
        let attempts = Arc::new(AtomicU32::new(0));
        let attempts_clone = Arc::<AtomicU32>::clone(&attempts);

        let config = Config::fast();
        let result = async_with_classification(
            || {
                let inner_attempts = Arc::<AtomicU32>::clone(&attempts_clone);
                async move {
                    inner_attempts.fetch_add(1, Ordering::SeqCst);
                    Err::<String, _>("permanent error".to_owned())
                }
            },
            |_error| ErrorClass::Permanent,
            &config,
            "test_op",
        )
        .await;

        {
            result.expect_err("Expected permanent error to fail immediately");
        }
        assert_eq!(attempts.load(Ordering::SeqCst), 1); // Only one attempt
    }

    #[tokio::test]
    async fn retry_exhausts_all_attempts() {
        let attempts = Arc::new(AtomicU32::new(0));
        let attempts_clone = Arc::<AtomicU32>::clone(&attempts);

        let config = Config::new(2, 10, 100, 2.0, false);

        let result = async_with_classification(
            || {
                let inner_attempts = Arc::<AtomicU32>::clone(&attempts_clone);
                async move {
                    inner_attempts.fetch_add(1, Ordering::SeqCst);
                    Err::<String, _>("always fails".to_owned())
                }
            },
            |_error| ErrorClass::Retryable,
            &config,
            "test_op",
        )
        .await;

        {
            result.expect_err("Expected retryable error to eventually fail after max attempts");
        }
        assert_eq!(attempts.load(Ordering::SeqCst), 3); // Initial + 2 retries
    }

    #[test]
    fn create_backoff_with_jitter() {
        let config = Config::new(5, 100, 10_000, 2.0, true);

        let backoff = create_backoff(&config);
        assert!((backoff.randomization_factor - 0.25_f64).abs() < f64::EPSILON);
        // 25% jitter
    }

    #[test]
    fn retry_config_presets() {
        let fast = Config::fast();
        assert_eq!(fast.base_delay_ms, 100);
        assert_eq!(fast.max_retries, 5);

        let slow = Config::slow();
        assert_eq!(slow.base_delay_ms, 5000);
        assert_eq!(slow.max_retries, 3);
    }
}
