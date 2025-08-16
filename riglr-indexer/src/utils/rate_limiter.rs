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

        let mut state = self.state.try_lock()
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
            tracing::debug!("Rate limiter adjustment: success_rate={:.2}, factor={:.2}", 
                          success_rate, adjustment_factor);
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
        assert!(timeout(Duration::from_millis(200), limiter.wait_for_tokens(1)).await.is_ok());
        let elapsed = start.elapsed();

        // Should have waited at least 100ms (1 token at 10 tokens/sec)
        assert!(elapsed >= Duration::from_millis(90)); // Some tolerance
    }
}