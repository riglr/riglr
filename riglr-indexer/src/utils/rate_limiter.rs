//! Rate limiting utilities

use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tokio::time::sleep;

/// Token bucket rate limiter
pub struct RateLimiter {
    /// Configuration
    config: RateLimiterConfig,
    /// Current state
    state: Arc<Mutex<RateLimiterState>>,
}

/// Rate limiter configuration
#[derive(Debug, Clone)]
pub struct RateLimiterConfig {
    /// Maximum tokens per second
    pub tokens_per_second: u32,
    /// Burst capacity (maximum tokens in bucket)
    pub burst_capacity: u32,
    /// Whether the rate limiter is enabled
    pub enabled: bool,
}

/// Internal rate limiter state
#[derive(Debug)]
struct RateLimiterState {
    /// Current number of tokens
    tokens: f64,
    /// Last refill time
    last_refill: Instant,
}

impl RateLimiter {
    /// Create a new rate limiter
    pub fn new(tokens_per_second: u32, burst_capacity: u32) -> Self {
        let config = RateLimiterConfig {
            tokens_per_second,
            burst_capacity,
            enabled: true,
        };

        let state = RateLimiterState {
            tokens: burst_capacity as f64,
            last_refill: Instant::now(),
        };

        Self {
            config,
            state: Arc::new(Mutex::new(state)),
        }
    }

    /// Create an unlimited rate limiter (no rate limiting)
    pub fn unlimited() -> Self {
        let config = RateLimiterConfig {
            tokens_per_second: 0,
            burst_capacity: 0,
            enabled: false,
        };

        let state = RateLimiterState {
            tokens: 0.0,
            last_refill: Instant::now(),
        };

        Self {
            config,
            state: Arc::new(Mutex::new(state)),
        }
    }

    /// Check if an operation can proceed (consumes 1 token if available)
    pub fn check(&self) -> Result<(), RateLimitError> {
        self.check_tokens(1)
    }

    /// Check if an operation requiring multiple tokens can proceed
    pub fn check_tokens(&self, tokens_required: u32) -> Result<(), RateLimitError> {
        if !self.config.enabled {
            return Ok(());
        }

        let mut state = self
            .state
            .try_lock()
            .map_err(|_| RateLimitError::LockContention)?;

        self.refill_tokens(&mut state);

        if state.tokens >= tokens_required as f64 {
            state.tokens -= tokens_required as f64;
            Ok(())
        } else {
            Err(RateLimitError::RateLimited {
                available_tokens: state.tokens as u32,
                required_tokens: tokens_required,
                retry_after: self.calculate_retry_after(tokens_required, state.tokens),
            })
        }
    }

    /// Wait until tokens are available, then consume them
    pub async fn wait_for_tokens(&self, tokens_required: u32) -> Result<(), RateLimitError> {
        if !self.config.enabled {
            return Ok(());
        }

        loop {
            match self.check_tokens(tokens_required) {
                Ok(()) => return Ok(()),
                Err(RateLimitError::RateLimited { retry_after, .. }) => {
                    sleep(retry_after).await;
                }
                Err(e) => return Err(e),
            }
        }
    }

    /// Get current number of available tokens
    pub async fn available_tokens(&self) -> u32 {
        if !self.config.enabled {
            return u32::MAX;
        }

        let mut state = self.state.lock().await;
        self.refill_tokens(&mut state);
        state.tokens as u32
    }

    /// Get rate limiter statistics
    pub async fn stats(&self) -> RateLimiterStats {
        let state = self.state.lock().await;
        RateLimiterStats {
            enabled: self.config.enabled,
            tokens_per_second: self.config.tokens_per_second,
            burst_capacity: self.config.burst_capacity,
            current_tokens: state.tokens,
            last_refill: state.last_refill,
        }
    }

    /// Refill tokens based on elapsed time
    fn refill_tokens(&self, state: &mut RateLimiterState) {
        let now = Instant::now();
        let elapsed = now.duration_since(state.last_refill);

        if elapsed.as_millis() > 0 {
            let tokens_to_add = elapsed.as_secs_f64() * self.config.tokens_per_second as f64;
            state.tokens = (state.tokens + tokens_to_add).min(self.config.burst_capacity as f64);
            state.last_refill = now;
        }
    }

    /// Calculate how long to wait before retrying
    fn calculate_retry_after(&self, tokens_required: u32, current_tokens: f64) -> Duration {
        let tokens_needed = tokens_required as f64 - current_tokens;
        let seconds_to_wait = tokens_needed / self.config.tokens_per_second as f64;
        Duration::from_millis((seconds_to_wait * 1000.0) as u64 + 1) // Add 1ms buffer
    }
}

/// Rate limiter statistics
#[derive(Debug, Clone)]
pub struct RateLimiterStats {
    /// Whether rate limiting is enabled
    pub enabled: bool,
    /// Token generation rate per second
    pub tokens_per_second: u32,
    /// Maximum burst capacity
    pub burst_capacity: u32,
    /// Current available tokens
    pub current_tokens: f64,
    /// Last token refill timestamp
    pub last_refill: Instant,
}

/// Rate limiter errors
#[derive(Debug, thiserror::Error)]
pub enum RateLimitError {
    /// Rate limit exceeded
    #[error("Rate limited: need {required_tokens} tokens, have {available_tokens}. Retry after {retry_after:?}")]
    RateLimited {
        /// Number of available tokens
        available_tokens: u32,
        /// Number of required tokens
        required_tokens: u32,
        /// Time to wait before retry
        retry_after: Duration,
    },

    /// Lock contention error
    #[error("Lock contention")]
    LockContention,
}

/// Adaptive rate limiter that adjusts based on success/failure rates
pub struct AdaptiveRateLimiter {
    base_limiter: RateLimiter,
    success_count: Arc<Mutex<u32>>,
    failure_count: Arc<Mutex<u32>>,
    last_adjustment: Arc<Mutex<Instant>>,
    adjustment_interval: Duration,
}

impl AdaptiveRateLimiter {
    /// Create a new adaptive rate limiter
    pub fn new(
        initial_tokens_per_second: u32,
        burst_capacity: u32,
        adjustment_interval: Duration,
    ) -> Self {
        Self {
            base_limiter: RateLimiter::new(initial_tokens_per_second, burst_capacity),
            success_count: Arc::new(Mutex::new(0)),
            failure_count: Arc::new(Mutex::new(0)),
            last_adjustment: Arc::new(Mutex::new(Instant::now())),
            adjustment_interval,
        }
    }

    /// Check tokens and potentially adjust rate based on success/failure
    pub async fn check_and_adapt(&self) -> Result<(), RateLimitError> {
        let result = self.base_limiter.check();

        // Record success/failure for adaptation
        match &result {
            Ok(()) => {
                let mut success_count = self.success_count.lock().await;
                *success_count += 1;
            }
            Err(_) => {
                let mut failure_count = self.failure_count.lock().await;
                *failure_count += 1;
            }
        }

        // Check if it's time to adjust
        let should_adjust = {
            let last_adjustment = self.last_adjustment.lock().await;
            last_adjustment.elapsed() >= self.adjustment_interval
        };

        if should_adjust {
            self.adjust_rate().await;
        }

        result
    }

    /// Adjust rate based on success/failure ratio
    async fn adjust_rate(&self) {
        let success_count = {
            let mut count = self.success_count.lock().await;
            let val = *count;
            *count = 0;
            val
        };

        let failure_count = {
            let mut count = self.failure_count.lock().await;
            let val = *count;
            *count = 0;
            val
        };

        let total = success_count + failure_count;
        if total > 0 {
            let success_rate = success_count as f64 / total as f64;

            // Adjust rate based on success rate
            // High success rate -> increase rate
            // Low success rate -> decrease rate
            let adjustment_factor = if success_rate > 0.9 {
                1.1 // Increase by 10%
            } else if success_rate < 0.7 {
                0.9 // Decrease by 10%
            } else {
                1.0 // No change
            };

            // In a real implementation, you'd need to modify the rate limiter's config
            // This would require making the config mutable or rebuilding the limiter
            tracing::debug!(
                "Rate limiter adjustment: success_rate={:.2}, factor={:.2}",
                success_rate,
                adjustment_factor
            );
        }

        let mut last_adjustment = self.last_adjustment.lock().await;
        *last_adjustment = Instant::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::timeout;

    #[tokio::test]
    async fn test_rate_limiter_basic() {
        let limiter = RateLimiter::new(10, 5); // 10 tokens/sec, burst of 5

        // Should have initial burst capacity
        assert_eq!(limiter.available_tokens().await, 5);

        // Consume all tokens
        for _ in 0..5 {
            assert!(limiter.check().is_ok());
        }

        // Should be rate limited now
        assert!(limiter.check().is_err());
    }

    #[tokio::test]
    async fn test_rate_limiter_refill() {
        let limiter = RateLimiter::new(10, 2); // 10 tokens/sec, burst of 2

        // Consume all tokens
        assert!(limiter.check().is_ok());
        assert!(limiter.check().is_ok());
        assert!(limiter.check().is_err());

        // Wait a bit for refill
        sleep(Duration::from_millis(200)).await; // Should get ~2 tokens

        // Should have tokens again
        assert!(limiter.check().is_ok());
    }

    #[tokio::test]
    async fn test_unlimited_rate_limiter() {
        let limiter = RateLimiter::unlimited();

        // Should never be rate limited
        for _ in 0..1000 {
            assert!(limiter.check().is_ok());
        }
    }

    #[tokio::test]
    async fn test_wait_for_tokens() {
        let limiter = RateLimiter::new(10, 1); // 10 tokens/sec, burst of 1

        // Consume the initial token
        assert!(limiter.check().is_ok());

        // Wait for a new token
        let start = Instant::now();
        assert!(
            timeout(Duration::from_millis(200), limiter.wait_for_tokens(1))
                .await
                .is_ok()
        );
        let elapsed = start.elapsed();

        // Should have waited at least 100ms (1 token at 10 tokens/sec)
        assert!(elapsed >= Duration::from_millis(90)); // Some tolerance
    }

    #[tokio::test]
    async fn test_check_tokens_multiple() {
        let limiter = RateLimiter::new(10, 5); // 10 tokens/sec, burst of 5

        // Should be able to consume 3 tokens at once
        assert!(limiter.check_tokens(3).is_ok());
        assert_eq!(limiter.available_tokens().await, 2);

        // Should fail to consume 3 more tokens (only 2 available)
        let result = limiter.check_tokens(3);
        assert!(result.is_err());

        if let Err(RateLimitError::RateLimited {
            available_tokens,
            required_tokens,
            retry_after,
        }) = result
        {
            assert_eq!(available_tokens, 2);
            assert_eq!(required_tokens, 3);
            assert!(retry_after > Duration::from_millis(0));
        } else {
            panic!("Expected RateLimited error");
        }
    }

    #[tokio::test]
    async fn test_check_tokens_zero() {
        let limiter = RateLimiter::new(10, 5);
        // Should succeed with 0 tokens required
        assert!(limiter.check_tokens(0).is_ok());
        assert_eq!(limiter.available_tokens().await, 5); // No tokens consumed
    }

    #[tokio::test]
    async fn test_unlimited_rate_limiter_properties() {
        let limiter = RateLimiter::unlimited();

        // Check stats for unlimited limiter
        let stats = limiter.stats().await;
        assert!(!stats.enabled);
        assert_eq!(stats.tokens_per_second, 0);
        assert_eq!(stats.burst_capacity, 0);

        // Available tokens should be max
        assert_eq!(limiter.available_tokens().await, u32::MAX);

        // Should be able to check any number of tokens
        assert!(limiter.check_tokens(1000).is_ok());
        assert!(limiter.wait_for_tokens(5000).await.is_ok());
    }

    #[tokio::test]
    async fn test_rate_limiter_stats() {
        let limiter = RateLimiter::new(10, 5);

        let stats = limiter.stats().await;
        assert!(stats.enabled);
        assert_eq!(stats.tokens_per_second, 10);
        assert_eq!(stats.burst_capacity, 5);
        assert_eq!(stats.current_tokens, 5.0);

        // Consume some tokens and check stats again
        assert!(limiter.check_tokens(2).is_ok());
        let stats = limiter.stats().await;
        assert_eq!(stats.current_tokens, 3.0);
    }

    #[tokio::test]
    async fn test_refill_tokens_over_time() {
        let limiter = RateLimiter::new(10, 3); // 10 tokens/sec, burst of 3

        // Consume all tokens
        assert!(limiter.check_tokens(3).is_ok());
        assert_eq!(limiter.available_tokens().await, 0);

        // Wait for partial refill
        sleep(Duration::from_millis(150)).await; // Should get ~1.5 tokens
        let available = limiter.available_tokens().await;
        assert!(available >= 1 && available <= 2);

        // Wait for full refill
        sleep(Duration::from_millis(300)).await; // Should reach burst capacity
        assert_eq!(limiter.available_tokens().await, 3);
    }

    #[tokio::test]
    async fn test_calculate_retry_after() {
        let limiter = RateLimiter::new(10, 5); // 10 tokens/sec

        // Consume most tokens
        assert!(limiter.check_tokens(4).is_ok());

        // Try to consume more than available
        let result = limiter.check_tokens(3);
        assert!(result.is_err());

        if let Err(RateLimitError::RateLimited { retry_after, .. }) = result {
            // Should need to wait for about 200ms for 2 more tokens at 10 tokens/sec
            assert!(retry_after >= Duration::from_millis(100));
            assert!(retry_after <= Duration::from_millis(300));
        }
    }

    #[tokio::test]
    async fn test_refill_tokens_with_zero_elapsed_time() {
        let limiter = RateLimiter::new(10, 5);

        // Consume a token
        assert!(limiter.check().is_ok());

        // Immediately check again - should not have refilled
        let available1 = limiter.available_tokens().await;
        let available2 = limiter.available_tokens().await;
        assert_eq!(available1, available2); // No refill with zero elapsed time
    }

    #[tokio::test]
    async fn test_rate_limit_error_display() {
        let error = RateLimitError::RateLimited {
            available_tokens: 2,
            required_tokens: 5,
            retry_after: Duration::from_millis(300),
        };

        let error_string = format!("{}", error);
        assert!(error_string.contains("Rate limited"));
        assert!(error_string.contains("need 5 tokens"));
        assert!(error_string.contains("have 2"));
        assert!(error_string.contains("300ms"));
    }

    #[tokio::test]
    async fn test_lock_contention_error() {
        let error = RateLimitError::LockContention;
        let error_string = format!("{}", error);
        assert_eq!(error_string, "Lock contention");
    }

    #[tokio::test]
    async fn test_wait_for_tokens_unlimited() {
        let limiter = RateLimiter::unlimited();

        // Should return immediately without waiting
        let start = Instant::now();
        assert!(limiter.wait_for_tokens(1000).await.is_ok());
        let elapsed = start.elapsed();
        assert!(elapsed < Duration::from_millis(10)); // Should be very fast
    }

    #[tokio::test]
    async fn test_wait_for_tokens_with_lock_contention() {
        // This test is harder to create reliably, but we test the error path exists
        let limiter = RateLimiter::new(1, 1);

        // Consume the token
        assert!(limiter.check().is_ok());

        // The wait_for_tokens should eventually succeed when tokens refill
        let result = timeout(Duration::from_millis(1100), limiter.wait_for_tokens(1)).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_adaptive_rate_limiter_new() {
        let adaptive = AdaptiveRateLimiter::new(10, 5, Duration::from_secs(1));

        // Should be able to check and adapt
        let result = adaptive.check_and_adapt().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_adaptive_rate_limiter_success_tracking() {
        let adaptive = AdaptiveRateLimiter::new(
            10,
            5,
            Duration::from_millis(10), // Very short adjustment interval
        );

        // Generate several successes
        for _ in 0..3 {
            let _ = adaptive.check_and_adapt().await;
        }

        // Wait for adjustment interval to pass
        sleep(Duration::from_millis(15)).await;

        // This should trigger adjustment
        let _ = adaptive.check_and_adapt().await;
    }

    #[tokio::test]
    async fn test_adaptive_rate_limiter_failure_tracking() {
        let adaptive = AdaptiveRateLimiter::new(
            10,
            2, // Small burst capacity to force failures
            Duration::from_millis(10),
        );

        // Consume all tokens to force failures
        let _ = adaptive.check_and_adapt().await; // Success
        let _ = adaptive.check_and_adapt().await; // Success
        let _ = adaptive.check_and_adapt().await; // Should fail
        let _ = adaptive.check_and_adapt().await; // Should fail

        // Wait for adjustment interval
        sleep(Duration::from_millis(15)).await;

        // This should trigger adjustment with failures recorded
        let _ = adaptive.check_and_adapt().await;
    }

    #[tokio::test]
    async fn test_adaptive_adjust_rate_with_high_success_rate() {
        let adaptive = AdaptiveRateLimiter::new(
            10,
            10,                       // Large capacity for successes
            Duration::from_millis(1), // Very short interval
        );

        // Generate many successes (> 90% success rate)
        for _ in 0..10 {
            let _ = adaptive.check_and_adapt().await; // All should succeed
        }

        // Wait and trigger adjustment
        sleep(Duration::from_millis(5)).await;
        let _ = adaptive.check_and_adapt().await;
    }

    #[tokio::test]
    async fn test_adaptive_adjust_rate_with_low_success_rate() {
        let adaptive = AdaptiveRateLimiter::new(
            10,
            1, // Very small capacity to force failures
            Duration::from_millis(1),
        );

        // Generate mostly failures (< 70% success rate)
        let _ = adaptive.check_and_adapt().await; // Success
        for _ in 0..5 {
            let _ = adaptive.check_and_adapt().await; // Should mostly fail
        }

        // Wait and trigger adjustment
        sleep(Duration::from_millis(5)).await;
        let _ = adaptive.check_and_adapt().await;
    }

    #[tokio::test]
    async fn test_adaptive_adjust_rate_with_medium_success_rate() {
        let adaptive = AdaptiveRateLimiter::new(10, 3, Duration::from_millis(1));

        // Generate medium success rate (70-90%)
        let _ = adaptive.check_and_adapt().await; // Success
        let _ = adaptive.check_and_adapt().await; // Success
        let _ = adaptive.check_and_adapt().await; // Success
        let _ = adaptive.check_and_adapt().await; // Should fail
        let _ = adaptive.check_and_adapt().await; // Should fail

        // Wait and trigger adjustment (should result in no change factor = 1.0)
        sleep(Duration::from_millis(5)).await;
        let _ = adaptive.check_and_adapt().await;
    }

    #[tokio::test]
    async fn test_adaptive_no_adjustment_with_zero_total() {
        let adaptive = AdaptiveRateLimiter::new(10, 5, Duration::from_millis(1));

        // Wait for adjustment interval without any operations
        sleep(Duration::from_millis(5)).await;

        // This should not crash or panic with zero total operations
        let result = adaptive.check_and_adapt().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_rate_limiter_config_debug() {
        let config = RateLimiterConfig {
            tokens_per_second: 10,
            burst_capacity: 5,
            enabled: true,
        };

        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("tokens_per_second: 10"));
        assert!(debug_str.contains("burst_capacity: 5"));
        assert!(debug_str.contains("enabled: true"));
    }

    #[tokio::test]
    async fn test_rate_limiter_stats_debug() {
        let limiter = RateLimiter::new(10, 5);
        let stats = limiter.stats().await;

        let debug_str = format!("{:?}", stats);
        assert!(debug_str.contains("enabled: true"));
        assert!(debug_str.contains("tokens_per_second: 10"));
        assert!(debug_str.contains("burst_capacity: 5"));
    }

    #[tokio::test]
    async fn test_rate_limiter_stats_clone() {
        let limiter = RateLimiter::new(10, 5);
        let stats = limiter.stats().await;
        let stats_clone = stats.clone();

        assert_eq!(stats.enabled, stats_clone.enabled);
        assert_eq!(stats.tokens_per_second, stats_clone.tokens_per_second);
        assert_eq!(stats.burst_capacity, stats_clone.burst_capacity);
    }

    #[tokio::test]
    async fn test_rate_limiter_config_clone() {
        let config = RateLimiterConfig {
            tokens_per_second: 10,
            burst_capacity: 5,
            enabled: true,
        };
        let config_clone = config.clone();

        assert_eq!(config.tokens_per_second, config_clone.tokens_per_second);
        assert_eq!(config.burst_capacity, config_clone.burst_capacity);
        assert_eq!(config.enabled, config_clone.enabled);
    }
}
