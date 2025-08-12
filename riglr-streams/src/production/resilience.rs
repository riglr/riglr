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
