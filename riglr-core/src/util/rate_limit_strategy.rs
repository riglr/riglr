//! Rate limiting strategies for flexible request rate control
//!
//! This module provides trait-based abstractions for different rate limiting algorithms,
//! allowing users to choose the most appropriate strategy for their use case.

use crate::ToolError;
use std::time::{Duration, Instant};

/// Trait defining the interface for rate limiting strategies
///
/// Different strategies can implement this trait to provide various
/// rate limiting algorithms such as token bucket, fixed window, sliding window, etc.
pub trait RateLimitStrategy: Send + Sync {
    /// Check if a request should be allowed for the given client
    ///
    /// # Arguments
    /// * `client_id` - Unique identifier for the client
    ///
    /// # Returns
    /// * `Ok(())` if the request is allowed
    /// * `Err(ToolError::RateLimited)` if the rate limit is exceeded
    fn check_rate_limit(&self, client_id: &str) -> Result<(), ToolError>;

    /// Reset rate limit state for a specific client
    fn reset_client(&self, client_id: &str);

    /// Clear all rate limit data
    fn clear_all(&self);

    /// Get current request count for a client
    fn get_request_count(&self, client_id: &str) -> usize;

    /// Get strategy name for debugging/logging
    fn strategy_name(&self) -> &str;
}

/// Information about a client's rate limit status
#[derive(Debug, Clone)]
pub struct ClientRateInfo {
    /// Timestamps of recent requests
    pub request_times: Vec<Instant>,
    /// Number of tokens available for burst (stored as f64 for fractional replenishment)
    pub burst_tokens: f64,
    /// Last time tokens were refilled
    pub last_refill: Instant,
}

impl ClientRateInfo {
    /// Create new client rate info with initial values
    pub fn new(initial_tokens: f64) -> Self {
        Self {
            request_times: Vec::new(),
            burst_tokens: initial_tokens,
            last_refill: Instant::now(),
        }
    }
}

/// Fixed window rate limiting strategy
///
/// This strategy divides time into fixed windows and allows a fixed number
/// of requests per window. When a window expires, the count resets.
#[derive(Debug)]
pub struct FixedWindowStrategy {
    /// Maximum requests per window
    pub max_requests: usize,
    /// Duration of each window
    pub window_duration: Duration,
    /// Client tracking
    pub clients: dashmap::DashMap<String, FixedWindowClientInfo>,
}

/// Information about a client's fixed window rate limit state
#[derive(Debug, Clone)]
pub struct FixedWindowClientInfo {
    /// Start of current window
    pub window_start: Instant,
    /// Number of requests in current window
    pub request_count: usize,
}

impl RateLimitStrategy for FixedWindowStrategy {
    fn check_rate_limit(&self, client_id: &str) -> Result<(), ToolError> {
        let now = Instant::now();
        let mut entry = self
            .clients
            .entry(client_id.to_string())
            .or_insert_with(|| FixedWindowClientInfo {
                window_start: now,
                request_count: 0,
            });

        // Check if we're in a new window
        if now.duration_since(entry.window_start) >= self.window_duration {
            // Reset for new window
            entry.window_start = now;
            entry.request_count = 0;
        }

        // Check if limit exceeded
        if entry.request_count >= self.max_requests {
            let time_until_reset = self
                .window_duration
                .saturating_sub(now.duration_since(entry.window_start));

            return Err(ToolError::RateLimited {
                source: None,
                source_message: format!(
                    "Fixed window rate limit: {} requests per {:?}",
                    self.max_requests, self.window_duration
                ),
                context: format!("Exceeded {} requests in current window", self.max_requests),
                retry_after: Some(time_until_reset),
            });
        }

        entry.request_count += 1;
        Ok(())
    }

    fn reset_client(&self, client_id: &str) {
        self.clients.remove(client_id);
    }

    fn clear_all(&self) {
        self.clients.clear();
    }

    fn get_request_count(&self, client_id: &str) -> usize {
        self.clients
            .get(client_id)
            .map(|entry| entry.request_count)
            .unwrap_or(0)
    }

    fn strategy_name(&self) -> &str {
        "FixedWindow"
    }
}

impl FixedWindowStrategy {
    /// Create a new fixed window rate limiter
    pub fn new(max_requests: usize, window_duration: Duration) -> Self {
        Self {
            max_requests,
            window_duration,
            clients: dashmap::DashMap::new(),
        }
    }
}
