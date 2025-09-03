//! Rate limiting utilities for riglr-core
//!
//! This module provides flexible, strategy-based rate limiting that supports
//! multiple algorithms including token bucket, fixed window, and custom strategies.

use std::sync::Arc;
use std::time::Duration;

use super::rate_limit_strategy::{FixedWindowStrategy, RateLimitStrategy};
use super::token_bucket::TokenBucketStrategy;
use crate::ToolError;

/// Rate limiting strategy type
#[derive(Debug, Clone, Copy)]
pub enum RateLimitStrategyType {
    /// Token bucket algorithm (default)
    TokenBucket,
    /// Fixed window algorithm
    FixedWindow,
}

/// A configurable rate limiter for controlling request rates.
///
/// This rate limiter supports multiple strategies:
/// - Token bucket: Continuous token replenishment with burst capacity
/// - Fixed window: Fixed request count per time window
/// - Custom strategies via the RateLimitStrategy trait
///
/// # Example
///
/// ```rust
/// use riglr_core::util::{RateLimiter, RateLimitStrategyType};
/// use std::time::Duration;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Default token bucket strategy
/// let rate_limiter = RateLimiter::new(10, Duration::from_secs(60));
///
/// // Check rate limit for a user
/// rate_limiter.check_rate_limit("user123")?;
///
/// // Use fixed window strategy
/// let fixed_limiter = RateLimiter::builder()
///     .strategy(RateLimitStrategyType::FixedWindow)
///     .max_requests(100)
///     .time_window(Duration::from_secs(60))
///     .build();
/// # Ok(())
/// # }
/// ```
#[derive(Clone)]
pub struct RateLimiter {
    /// The underlying rate limiting strategy
    strategy: Arc<dyn RateLimitStrategy>,
}

impl std::fmt::Debug for RateLimiter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RateLimiter")
            .field("strategy", &self.strategy.strategy_name())
            .finish()
    }
}

impl RateLimiter {
    /// Create a new rate limiter with the default token bucket strategy
    pub fn new(max_requests: usize, time_window: Duration) -> Self {
        Self {
            strategy: Arc::new(TokenBucketStrategy::new(max_requests, time_window)),
        }
    }

    /// Create a rate limiter with a custom strategy
    pub fn with_strategy<S: RateLimitStrategy + 'static>(strategy: S) -> Self {
        Self {
            strategy: Arc::new(strategy),
        }
    }

    /// Create a new rate limiter builder for advanced configuration
    pub fn builder() -> RateLimiterBuilder {
        RateLimiterBuilder::default()
    }

    /// Check if a client has exceeded the rate limit
    ///
    /// # Arguments
    /// * `client_id` - Unique identifier for the client (e.g., IP address, user ID)
    ///
    /// # Returns
    /// * `Ok(())` if the request is allowed
    /// * `Err(ToolError::RateLimited)` if the rate limit is exceeded
    pub fn check_rate_limit(&self, client_id: &str) -> Result<(), ToolError> {
        self.strategy.check_rate_limit(client_id)
    }

    /// Reset rate limit for a specific client
    pub fn reset_client(&self, client_id: &str) {
        self.strategy.reset_client(client_id)
    }

    /// Clear all rate limit data
    pub fn clear_all(&self) {
        self.strategy.clear_all()
    }

    /// Get current request count for a client
    pub fn get_request_count(&self, client_id: &str) -> usize {
        self.strategy.get_request_count(client_id)
    }

    /// Get the name of the current strategy
    pub fn strategy_name(&self) -> &str {
        self.strategy.strategy_name()
    }
}

/// Builder for creating customized RateLimiter instances
#[derive(Debug, Default)]
pub struct RateLimiterBuilder {
    strategy_type: Option<RateLimitStrategyType>,
    max_requests: Option<usize>,
    time_window: Option<Duration>,
    burst_size: Option<usize>,
}

impl RateLimiterBuilder {
    /// Set the rate limiting strategy type
    pub fn strategy(mut self, strategy: RateLimitStrategyType) -> Self {
        self.strategy_type = Some(strategy);
        self
    }

    /// Set the maximum number of requests allowed in the time window
    pub fn max_requests(mut self, max: usize) -> Self {
        self.max_requests = Some(max);
        self
    }

    /// Set the time window for rate limiting
    pub fn time_window(mut self, window: Duration) -> Self {
        self.time_window = Some(window);
        self
    }

    /// Set the burst size for temporary spikes
    pub fn burst_size(mut self, size: usize) -> Self {
        self.burst_size = Some(size);
        self
    }

    /// Build the RateLimiter
    pub fn build(self) -> RateLimiter {
        let max_requests = self.max_requests.unwrap_or(10);
        let time_window = self.time_window.unwrap_or_else(|| Duration::from_secs(60));
        let strategy_type = self
            .strategy_type
            .unwrap_or(RateLimitStrategyType::TokenBucket);

        let strategy: Arc<dyn RateLimitStrategy> = match strategy_type {
            RateLimitStrategyType::TokenBucket => {
                if let Some(burst_size) = self.burst_size {
                    Arc::new(TokenBucketStrategy::with_burst(
                        max_requests,
                        time_window,
                        burst_size,
                    ))
                } else {
                    Arc::new(TokenBucketStrategy::new(max_requests, time_window))
                }
            }
            RateLimitStrategyType::FixedWindow => {
                Arc::new(FixedWindowStrategy::new(max_requests, time_window))
            }
        };

        RateLimiter { strategy }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_rate_limiter_allows_requests_within_limit() {
        let limiter = RateLimiter::new(3, Duration::from_secs(1));

        assert!(limiter.check_rate_limit("user1").is_ok());
        assert!(limiter.check_rate_limit("user1").is_ok());
        assert!(limiter.check_rate_limit("user1").is_ok());
    }

    #[test]
    fn test_rate_limiter_blocks_requests_over_limit() {
        let limiter = RateLimiter::new(2, Duration::from_secs(1));

        assert!(limiter.check_rate_limit("user1").is_ok());
        assert!(limiter.check_rate_limit("user1").is_ok());
        assert!(limiter.check_rate_limit("user1").is_err());
    }

    #[test]
    fn test_rate_limiter_with_different_clients() {
        let limiter = RateLimiter::new(1, Duration::from_secs(1));

        assert!(limiter.check_rate_limit("user1").is_ok());
        assert!(limiter.check_rate_limit("user2").is_ok());
        assert!(limiter.check_rate_limit("user1").is_err());
        assert!(limiter.check_rate_limit("user2").is_err());
    }

    #[test]
    fn test_rate_limiter_builder() {
        let limiter = RateLimiter::builder()
            .max_requests(5)
            .time_window(Duration::from_secs(10))
            .burst_size(2)
            .build();

        assert!(limiter.check_rate_limit("user1").is_ok());
    }

    #[test]
    fn test_reset_client() {
        let limiter = RateLimiter::new(1, Duration::from_secs(1));

        assert!(limiter.check_rate_limit("user1").is_ok());
        assert!(limiter.check_rate_limit("user1").is_err());

        limiter.reset_client("user1");
        assert!(limiter.check_rate_limit("user1").is_ok());
    }

    #[test]
    fn test_time_based_token_replenishment() {
        // Create a rate limiter with 10 requests per second (for faster testing)
        let limiter = RateLimiter::new(10, Duration::from_millis(1000));

        // Exhaust initial tokens
        for _ in 0..10 {
            assert!(limiter.check_rate_limit("user1").is_ok());
        }

        // Should be blocked now
        assert!(limiter.check_rate_limit("user1").is_err());

        // Wait for tokens to replenish (100ms should give us ~1 token)
        thread::sleep(Duration::from_millis(150));

        // Should be allowed now due to token replenishment
        assert!(limiter.check_rate_limit("user1").is_ok());

        // Should be blocked again
        assert!(limiter.check_rate_limit("user1").is_err());
    }

    #[test]
    fn test_burst_size_cap() {
        // Create a rate limiter with burst size
        let limiter = RateLimiter::builder()
            .max_requests(5)
            .time_window(Duration::from_secs(1))
            .burst_size(3) // Burst size smaller than max_requests
            .build();

        // Should be able to make 3 burst requests (initial bucket capacity)
        assert!(limiter.check_rate_limit("user1").is_ok());
        assert!(limiter.check_rate_limit("user1").is_ok());
        assert!(limiter.check_rate_limit("user1").is_ok());

        // Now should be blocked (burst tokens exhausted)
        assert!(limiter.check_rate_limit("user1").is_err());

        // Wait for tokens to replenish (200ms should give us 1 token at 5/sec rate)
        thread::sleep(Duration::from_millis(250));

        // Should be allowed now due to replenishment
        assert!(limiter.check_rate_limit("user1").is_ok());
    }

    #[test]
    fn test_token_accumulation_capped() {
        // Create a rate limiter with burst size
        let limiter = RateLimiter::builder()
            .max_requests(10)
            .time_window(Duration::from_millis(100))
            .burst_size(5)
            .build();

        // Wait long enough that tokens would accumulate beyond burst size if uncapped
        // (200ms would generate 20 tokens at 100 tokens/sec rate, but capped at 5)
        thread::sleep(Duration::from_millis(200));

        // Should only be able to make burst_size (5) requests rapidly
        for _ in 0..5 {
            assert!(limiter.check_rate_limit("user1").is_ok());
        }

        // Should be blocked now (all 5 tokens consumed)
        assert!(limiter.check_rate_limit("user1").is_err());

        // Wait for one more token to replenish (10ms for 1 token at 100/sec)
        thread::sleep(Duration::from_millis(15));

        // Should be allowed one more request
        assert!(limiter.check_rate_limit("user1").is_ok());

        // Should be blocked again
        assert!(limiter.check_rate_limit("user1").is_err());
    }

    #[test]
    fn test_fractional_token_replenishment() {
        // Create a rate limiter with 1 request per second
        let limiter = RateLimiter::new(1, Duration::from_secs(1));

        // Use the token
        assert!(limiter.check_rate_limit("user1").is_ok());
        assert!(limiter.check_rate_limit("user1").is_err());

        // Wait for half a second (should get 0.5 tokens)
        thread::sleep(Duration::from_millis(500));

        // Should still be blocked (need 1.0 token)
        assert!(limiter.check_rate_limit("user1").is_err());

        // Wait another 600ms (total 1.1 seconds, should have > 1 token)
        thread::sleep(Duration::from_millis(600));

        // Should be allowed now
        assert!(limiter.check_rate_limit("user1").is_ok());
    }
}
