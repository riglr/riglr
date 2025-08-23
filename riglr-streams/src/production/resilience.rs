//! Resilience patterns for fault-tolerant stream processing.
//!
//! This module implements essential resilience patterns including circuit breakers,
//! retry policies with configurable backoff strategies, and other fault tolerance
//! mechanisms. These patterns help streaming systems gracefully handle failures,
//! prevent cascading errors, and maintain system stability under adverse conditions.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;
use tracing::{error, info, warn};

/// Circuit breaker for stream connections
pub struct CircuitBreaker {
    /// Name
    name: String,
    /// Failure threshold
    failure_threshold: u64,
    /// Success threshold to reset
    success_threshold: u64,
    /// Timeout duration
    timeout: Duration,
    /// Current failure count
    failure_count: Arc<AtomicU64>,
    /// Current success count
    success_count: Arc<AtomicU64>,
    /// Is open
    is_open: Arc<AtomicBool>,
    /// Last failure time
    last_failure_time: Arc<RwLock<Option<SystemTime>>>,
}

impl CircuitBreaker {
    /// Create new circuit breaker
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            failure_threshold: 5,
            success_threshold: 3,
            timeout: Duration::from_secs(60),
            failure_count: Arc::new(AtomicU64::new(0)),
            success_count: Arc::new(AtomicU64::new(0)),
            is_open: Arc::new(AtomicBool::new(false)),
            last_failure_time: Arc::new(RwLock::new(None)),
        }
    }

    /// Set failure threshold
    pub fn with_failure_threshold(mut self, threshold: u64) -> Self {
        self.failure_threshold = threshold;
        self
    }

    /// Set success threshold
    pub fn with_success_threshold(mut self, threshold: u64) -> Self {
        self.success_threshold = threshold;
        self
    }

    /// Set timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Check if circuit is open
    pub async fn is_open(&self) -> bool {
        if !self.is_open.load(Ordering::Relaxed) {
            return false;
        }

        // Check if timeout has passed
        if let Some(last_failure) = *self.last_failure_time.read().await {
            let elapsed = SystemTime::now()
                .duration_since(last_failure)
                .unwrap_or(Duration::ZERO);

            if elapsed > self.timeout {
                info!(
                    "Circuit breaker {} timeout expired, attempting reset",
                    self.name
                );
                self.is_open.store(false, Ordering::Relaxed);
                self.failure_count.store(0, Ordering::Relaxed);
                return false;
            }
        }

        true
    }

    /// Record success
    pub async fn record_success(&self) {
        self.success_count.fetch_add(1, Ordering::Relaxed);

        if self.is_open.load(Ordering::Relaxed) {
            let success_count = self.success_count.load(Ordering::Relaxed);

            if success_count >= self.success_threshold {
                info!(
                    "Circuit breaker {} closed after {} successes",
                    self.name, success_count
                );
                self.is_open.store(false, Ordering::Relaxed);
                self.failure_count.store(0, Ordering::Relaxed);
                self.success_count.store(0, Ordering::Relaxed);
            }
        } else {
            // Reset failure count on success when closed
            self.failure_count.store(0, Ordering::Relaxed);
        }
    }

    /// Record failure
    pub async fn record_failure(&self) {
        let count = self.failure_count.fetch_add(1, Ordering::Relaxed) + 1;
        *self.last_failure_time.write().await = Some(SystemTime::now());

        if count >= self.failure_threshold && !self.is_open.load(Ordering::Relaxed) {
            error!(
                "Circuit breaker {} opened after {} failures",
                self.name, count
            );
            self.is_open.store(true, Ordering::Relaxed);
            self.success_count.store(0, Ordering::Relaxed);
        }
    }
}

/// Retry policy for operations
#[derive(Clone)]
pub struct RetryPolicy {
    /// Maximum number of retries
    pub max_retries: u32,
    /// Initial delay between retry attempts
    pub initial_delay: Duration,
    /// Backoff strategy to use for calculating delays
    pub backoff: BackoffStrategy,
    /// Maximum delay between retry attempts
    pub max_delay: Duration,
    /// Whether to apply random jitter to delays
    pub jitter: bool,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay: Duration::from_secs(1),
            backoff: BackoffStrategy::Exponential { factor: 2.0 },
            max_delay: Duration::from_secs(60),
            jitter: true,
        }
    }
}

/// Backoff strategy for retries
#[derive(Clone)]
pub enum BackoffStrategy {
    /// Fixed delay between all retry attempts
    Fixed,
    /// Linear backoff with configurable increment
    Linear {
        /// Duration to add per retry attempt
        increment: Duration,
    },
    /// Exponential backoff with configurable factor
    Exponential {
        /// Multiplication factor for each retry attempt
        factor: f64,
    },
}

impl RetryPolicy {
    /// Calculate delay for retry attempt
    pub fn calculate_delay(&self, attempt: u32) -> Duration {
        let mut delay = match &self.backoff {
            BackoffStrategy::Fixed => self.initial_delay,
            BackoffStrategy::Linear { increment } => {
                self.initial_delay + increment.mul_f64(attempt as f64)
            }
            BackoffStrategy::Exponential { factor } => {
                self.initial_delay.mul_f64(factor.powi(attempt as i32))
            }
        };

        // Apply max delay
        if delay > self.max_delay {
            delay = self.max_delay;
        }

        // Apply jitter
        if self.jitter {
            use rand::Rng;
            let mut rng = rand::rng();
            let jitter_factor = rng.random_range(0.8..1.2);
            delay = delay.mul_f64(jitter_factor);
        }

        delay
    }

    /// Execute with retry
    pub async fn execute<F, T, E>(&self, mut operation: F) -> Result<T, E>
    where
        F: FnMut() -> futures::future::BoxFuture<'static, Result<T, E>>,
        E: std::fmt::Display,
    {
        let mut attempt = 0;

        loop {
            match operation().await {
                Ok(result) => return Ok(result),
                Err(e) if attempt >= self.max_retries => {
                    error!("Operation failed after {} retries: {}", attempt, e);
                    return Err(e);
                }
                Err(e) => {
                    let delay = self.calculate_delay(attempt);
                    warn!(
                        "Operation failed (attempt {}), retrying in {:?}: {}",
                        attempt + 1,
                        delay,
                        e
                    );
                    tokio::time::sleep(delay).await;
                    attempt += 1;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicU32;
    use std::time::Instant;

    #[tokio::test]
    async fn test_circuit_breaker_new_creates_with_defaults() {
        let cb = CircuitBreaker::new("test");
        assert_eq!(cb.name, "test");
        assert_eq!(cb.failure_threshold, 5);
        assert_eq!(cb.success_threshold, 3);
        assert_eq!(cb.timeout, Duration::from_secs(60));
        assert_eq!(cb.failure_count.load(Ordering::Relaxed), 0);
        assert_eq!(cb.success_count.load(Ordering::Relaxed), 0);
        assert!(!cb.is_open.load(Ordering::Relaxed));
        assert!(cb.last_failure_time.read().await.is_none());
    }

    #[tokio::test]
    async fn test_circuit_breaker_with_failure_threshold() {
        let cb = CircuitBreaker::new("test").with_failure_threshold(10);
        assert_eq!(cb.failure_threshold, 10);
    }

    #[tokio::test]
    async fn test_circuit_breaker_with_success_threshold() {
        let cb = CircuitBreaker::new("test").with_success_threshold(5);
        assert_eq!(cb.success_threshold, 5);
    }

    #[tokio::test]
    async fn test_circuit_breaker_with_timeout() {
        let timeout = Duration::from_secs(120);
        let cb = CircuitBreaker::new("test").with_timeout(timeout);
        assert_eq!(cb.timeout, timeout);
    }

    #[tokio::test]
    async fn test_circuit_breaker_is_open_when_closed_returns_false() {
        let cb = CircuitBreaker::new("test");
        assert!(!cb.is_open().await);
    }

    #[tokio::test]
    async fn test_circuit_breaker_is_open_when_open_and_timeout_not_expired() {
        let cb = CircuitBreaker::new("test").with_timeout(Duration::from_secs(60));

        // Force the circuit breaker to open
        cb.is_open.store(true, Ordering::Relaxed);
        *cb.last_failure_time.write().await = Some(SystemTime::now());

        assert!(cb.is_open().await);
    }

    #[tokio::test]
    async fn test_circuit_breaker_is_open_when_timeout_expired_resets_circuit() {
        let cb = CircuitBreaker::new("test").with_timeout(Duration::from_millis(1));

        // Force the circuit breaker to open with old failure time
        cb.is_open.store(true, Ordering::Relaxed);
        cb.failure_count.store(10, Ordering::Relaxed);
        *cb.last_failure_time.write().await = Some(SystemTime::now() - Duration::from_secs(1));

        // After timeout, should be reset
        assert!(!cb.is_open().await);
        assert!(!cb.is_open.load(Ordering::Relaxed));
        assert_eq!(cb.failure_count.load(Ordering::Relaxed), 0);
    }

    #[tokio::test]
    async fn test_circuit_breaker_is_open_when_no_last_failure_time() {
        let cb = CircuitBreaker::new("test");

        // Force the circuit breaker to open but no last failure time
        cb.is_open.store(true, Ordering::Relaxed);

        // Should remain open since no last failure time is set
        assert!(cb.is_open().await);
    }

    #[tokio::test]
    async fn test_circuit_breaker_record_success_when_closed() {
        let cb = CircuitBreaker::new("test");
        cb.failure_count.store(3, Ordering::Relaxed);

        cb.record_success().await;

        assert_eq!(cb.success_count.load(Ordering::Relaxed), 1);
        assert_eq!(cb.failure_count.load(Ordering::Relaxed), 0);
    }

    #[tokio::test]
    async fn test_circuit_breaker_record_success_when_open_below_threshold() {
        let cb = CircuitBreaker::new("test").with_success_threshold(3);
        cb.is_open.store(true, Ordering::Relaxed);

        // Record 2 successes (below threshold)
        cb.record_success().await;
        cb.record_success().await;

        assert_eq!(cb.success_count.load(Ordering::Relaxed), 2);
        assert!(cb.is_open.load(Ordering::Relaxed)); // Should still be open
    }

    #[tokio::test]
    async fn test_circuit_breaker_record_success_when_open_reaches_threshold() {
        let cb = CircuitBreaker::new("test").with_success_threshold(3);
        cb.is_open.store(true, Ordering::Relaxed);
        cb.failure_count.store(5, Ordering::Relaxed);

        // Record 3 successes (reaches threshold)
        cb.record_success().await;
        cb.record_success().await;
        cb.record_success().await;

        assert!(!cb.is_open.load(Ordering::Relaxed)); // Should be closed
        assert_eq!(cb.failure_count.load(Ordering::Relaxed), 0);
        assert_eq!(cb.success_count.load(Ordering::Relaxed), 0);
    }

    #[tokio::test]
    async fn test_circuit_breaker_record_failure_below_threshold() {
        let cb = CircuitBreaker::new("test").with_failure_threshold(3);

        cb.record_failure().await;
        cb.record_failure().await;

        assert_eq!(cb.failure_count.load(Ordering::Relaxed), 2);
        assert!(!cb.is_open.load(Ordering::Relaxed)); // Should still be closed
        assert!(cb.last_failure_time.read().await.is_some());
    }

    #[tokio::test]
    async fn test_circuit_breaker_record_failure_reaches_threshold() {
        let cb = CircuitBreaker::new("test").with_failure_threshold(2);

        cb.record_failure().await;
        cb.record_failure().await;

        assert_eq!(cb.failure_count.load(Ordering::Relaxed), 2);
        assert!(cb.is_open.load(Ordering::Relaxed)); // Should be open
        assert_eq!(cb.success_count.load(Ordering::Relaxed), 0);
    }

    #[tokio::test]
    async fn test_circuit_breaker_record_failure_when_already_open() {
        let cb = CircuitBreaker::new("test").with_failure_threshold(2);
        cb.is_open.store(true, Ordering::Relaxed);

        cb.record_failure().await;

        assert_eq!(cb.failure_count.load(Ordering::Relaxed), 1);
        assert!(cb.is_open.load(Ordering::Relaxed)); // Should remain open
    }

    #[test]
    fn test_retry_policy_default() {
        let policy = RetryPolicy::default();
        assert_eq!(policy.max_retries, 3);
        assert_eq!(policy.initial_delay, Duration::from_secs(1));
        assert_eq!(policy.max_delay, Duration::from_secs(60));
        assert!(policy.jitter);

        match policy.backoff {
            BackoffStrategy::Exponential { factor } => assert_eq!(factor, 2.0),
            _ => panic!("Expected exponential backoff"),
        }
    }

    #[test]
    fn test_retry_policy_calculate_delay_fixed() {
        let policy = RetryPolicy {
            max_retries: 3,
            initial_delay: Duration::from_secs(1),
            backoff: BackoffStrategy::Fixed,
            max_delay: Duration::from_secs(60),
            jitter: false,
        };

        assert_eq!(policy.calculate_delay(0), Duration::from_secs(1));
        assert_eq!(policy.calculate_delay(1), Duration::from_secs(1));
        assert_eq!(policy.calculate_delay(5), Duration::from_secs(1));
    }

    #[test]
    fn test_retry_policy_calculate_delay_linear() {
        let policy = RetryPolicy {
            max_retries: 3,
            initial_delay: Duration::from_secs(1),
            backoff: BackoffStrategy::Linear {
                increment: Duration::from_secs(2),
            },
            max_delay: Duration::from_secs(60),
            jitter: false,
        };

        assert_eq!(policy.calculate_delay(0), Duration::from_secs(1));
        assert_eq!(policy.calculate_delay(1), Duration::from_secs(3));
        assert_eq!(policy.calculate_delay(2), Duration::from_secs(5));
    }

    #[test]
    fn test_retry_policy_calculate_delay_exponential() {
        let policy = RetryPolicy {
            max_retries: 3,
            initial_delay: Duration::from_secs(1),
            backoff: BackoffStrategy::Exponential { factor: 2.0 },
            max_delay: Duration::from_secs(60),
            jitter: false,
        };

        assert_eq!(policy.calculate_delay(0), Duration::from_secs(1));
        assert_eq!(policy.calculate_delay(1), Duration::from_secs(2));
        assert_eq!(policy.calculate_delay(2), Duration::from_secs(4));
        assert_eq!(policy.calculate_delay(3), Duration::from_secs(8));
    }

    #[test]
    fn test_retry_policy_calculate_delay_respects_max_delay() {
        let policy = RetryPolicy {
            max_retries: 10,
            initial_delay: Duration::from_secs(1),
            backoff: BackoffStrategy::Exponential { factor: 2.0 },
            max_delay: Duration::from_secs(5),
            jitter: false,
        };

        // After several exponential increases, should cap at max_delay
        assert_eq!(policy.calculate_delay(10), Duration::from_secs(5));
    }

    #[test]
    fn test_retry_policy_calculate_delay_with_jitter() {
        let policy = RetryPolicy {
            max_retries: 3,
            initial_delay: Duration::from_secs(10),
            backoff: BackoffStrategy::Fixed,
            max_delay: Duration::from_secs(60),
            jitter: true,
        };

        let delay = policy.calculate_delay(0);
        // With jitter, delay should be between 8 and 12 seconds (0.8 to 1.2 factor)
        assert!(delay >= Duration::from_secs(8));
        assert!(delay <= Duration::from_secs(12));
    }

    #[tokio::test]
    async fn test_retry_policy_execute_success_on_first_attempt() {
        let policy = RetryPolicy::default();
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let result = policy
            .execute(move || {
                counter_clone.fetch_add(1, Ordering::Relaxed);
                Box::pin(async { Ok::<i32, &'static str>(42) })
            })
            .await;

        assert_eq!(result.unwrap(), 42);
        assert_eq!(counter.load(Ordering::Relaxed), 1);
    }

    #[tokio::test]
    async fn test_retry_policy_execute_success_after_retries() {
        let policy = RetryPolicy {
            max_retries: 3,
            initial_delay: Duration::from_millis(1),
            backoff: BackoffStrategy::Fixed,
            max_delay: Duration::from_secs(1),
            jitter: false,
        };

        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let result = policy
            .execute(move || {
                let count = counter_clone.fetch_add(1, Ordering::Relaxed);
                Box::pin(async move {
                    if count < 2 {
                        Err("temporary failure")
                    } else {
                        Ok::<i32, &'static str>(42)
                    }
                })
            })
            .await;

        assert_eq!(result.unwrap(), 42);
        assert_eq!(counter.load(Ordering::Relaxed), 3);
    }

    #[tokio::test]
    async fn test_retry_policy_execute_failure_after_max_retries() {
        let policy = RetryPolicy {
            max_retries: 2,
            initial_delay: Duration::from_millis(1),
            backoff: BackoffStrategy::Fixed,
            max_delay: Duration::from_secs(1),
            jitter: false,
        };

        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let result = policy
            .execute(move || {
                counter_clone.fetch_add(1, Ordering::Relaxed);
                Box::pin(async { Err::<i32, &'static str>("persistent failure") })
            })
            .await;

        assert_eq!(result.unwrap_err(), "persistent failure");
        assert_eq!(counter.load(Ordering::Relaxed), 3); // Initial + 2 retries
    }

    #[tokio::test]
    async fn test_retry_policy_execute_respects_delay() {
        let policy = RetryPolicy {
            max_retries: 1,
            initial_delay: Duration::from_millis(50),
            backoff: BackoffStrategy::Fixed,
            max_delay: Duration::from_secs(1),
            jitter: false,
        };

        let start = Instant::now();
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let _result = policy
            .execute(move || {
                counter_clone.fetch_add(1, Ordering::Relaxed);
                Box::pin(async { Err::<i32, &'static str>("failure") })
            })
            .await;

        let elapsed = start.elapsed();
        // Should take at least the delay time for the retry
        assert!(elapsed >= Duration::from_millis(40)); // Allow some tolerance
        assert_eq!(counter.load(Ordering::Relaxed), 2); // Initial + 1 retry
    }

    #[test]
    fn test_backoff_strategy_clone() {
        let fixed = BackoffStrategy::Fixed;
        let fixed_clone = fixed.clone();
        match fixed_clone {
            BackoffStrategy::Fixed => (),
            _ => panic!("Expected Fixed strategy"),
        }

        let linear = BackoffStrategy::Linear {
            increment: Duration::from_secs(1),
        };
        let linear_clone = linear.clone();
        match linear_clone {
            BackoffStrategy::Linear { increment } => {
                assert_eq!(increment, Duration::from_secs(1));
            }
            _ => panic!("Expected Linear strategy"),
        }

        let exponential = BackoffStrategy::Exponential { factor: 2.5 };
        let exponential_clone = exponential.clone();
        match exponential_clone {
            BackoffStrategy::Exponential { factor } => {
                assert_eq!(factor, 2.5);
            }
            _ => panic!("Expected Exponential strategy"),
        }
    }

    #[test]
    fn test_retry_policy_clone() {
        let policy = RetryPolicy {
            max_retries: 5,
            initial_delay: Duration::from_secs(2),
            backoff: BackoffStrategy::Linear {
                increment: Duration::from_secs(1),
            },
            max_delay: Duration::from_secs(30),
            jitter: false,
        };

        let cloned = policy.clone();
        assert_eq!(cloned.max_retries, 5);
        assert_eq!(cloned.initial_delay, Duration::from_secs(2));
        assert_eq!(cloned.max_delay, Duration::from_secs(30));
        assert!(!cloned.jitter);

        match cloned.backoff {
            BackoffStrategy::Linear { increment } => {
                assert_eq!(increment, Duration::from_secs(1));
            }
            _ => panic!("Expected Linear strategy"),
        }
    }

    #[test]
    fn test_retry_policy_calculate_delay_zero_attempt() {
        let policy = RetryPolicy {
            max_retries: 3,
            initial_delay: Duration::from_secs(5),
            backoff: BackoffStrategy::Exponential { factor: 3.0 },
            max_delay: Duration::from_secs(60),
            jitter: false,
        };

        // Attempt 0 should return initial delay regardless of backoff strategy
        assert_eq!(policy.calculate_delay(0), Duration::from_secs(5));
    }

    #[test]
    fn test_retry_policy_calculate_delay_large_attempt_exponential() {
        let policy = RetryPolicy {
            max_retries: 20,
            initial_delay: Duration::from_secs(1),
            backoff: BackoffStrategy::Exponential { factor: 2.0 },
            max_delay: Duration::from_secs(10),
            jitter: false,
        };

        // Large attempt number should be capped by max_delay
        assert_eq!(policy.calculate_delay(20), Duration::from_secs(10));
    }

    #[test]
    fn test_retry_policy_calculate_delay_large_attempt_linear() {
        let policy = RetryPolicy {
            max_retries: 20,
            initial_delay: Duration::from_secs(1),
            backoff: BackoffStrategy::Linear {
                increment: Duration::from_secs(5),
            },
            max_delay: Duration::from_secs(15),
            jitter: false,
        };

        // Large attempt number should be capped by max_delay
        assert_eq!(policy.calculate_delay(10), Duration::from_secs(15));
    }
}
