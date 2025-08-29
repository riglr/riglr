//! Token bucket rate limiting strategy
//!
//! This module implements a time-based token bucket algorithm where tokens
//! are replenished continuously based on elapsed time.

use super::rate_limit_strategy::{ClientRateInfo, RateLimitStrategy};
use crate::ToolError;
use dashmap::DashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Token bucket rate limiting strategy
///
/// This strategy implements a time-based token bucket algorithm where:
/// - Tokens are replenished continuously based on elapsed time
/// - The replenishment rate is `max_requests` per `time_window`
/// - Burst capacity allows temporary spikes up to `burst_size` tokens
#[derive(Debug, Clone)]
pub struct TokenBucketStrategy {
    /// Maximum number of requests allowed in the time window
    max_requests: usize,
    /// Time window for rate limiting
    time_window: Duration,
    /// Optional burst size for allowing temporary spikes
    burst_size: Option<usize>,
    /// Map of client ID to their request history
    clients: Arc<DashMap<String, ClientRateInfo>>,
}

impl TokenBucketStrategy {
    /// Create a new token bucket strategy
    pub fn new(max_requests: usize, time_window: Duration) -> Self {
        Self {
            max_requests,
            time_window,
            burst_size: None,
            clients: Arc::new(DashMap::new()),
        }
    }

    /// Create with burst capacity
    pub fn with_burst(max_requests: usize, time_window: Duration, burst_size: usize) -> Self {
        Self {
            max_requests,
            time_window,
            burst_size: Some(burst_size),
            clients: Arc::new(DashMap::new()),
        }
    }
}

impl RateLimitStrategy for TokenBucketStrategy {
    fn check_rate_limit(&self, client_id: &str) -> Result<(), ToolError> {
        let now = Instant::now();
        let mut entry = self
            .clients
            .entry(client_id.to_string())
            .or_insert_with(|| {
                ClientRateInfo::new(self.burst_size.unwrap_or(self.max_requests) as f64)
            });

        // Time-based token replenishment
        let elapsed = now.duration_since(entry.last_refill);
        let elapsed_seconds = elapsed.as_secs_f64();

        // Calculate the refill rate: tokens per second
        let refill_rate = self.max_requests as f64 / self.time_window.as_secs_f64();

        // Calculate tokens to add based on elapsed time
        let tokens_to_add = elapsed_seconds * refill_rate;

        // Add tokens up to the burst size limit (or max_requests if no burst size)
        let max_tokens = self.burst_size.unwrap_or(self.max_requests) as f64;
        entry.burst_tokens = (entry.burst_tokens + tokens_to_add).min(max_tokens);
        entry.last_refill = now;

        // Remove old requests outside the time window (for sustained rate tracking)
        entry
            .request_times
            .retain(|&time| now.duration_since(time) < self.time_window);

        // Check if we have tokens available
        let has_burst_token = entry.burst_tokens >= 1.0;

        if !has_burst_token {
            // No tokens available, calculate retry time
            let tokens_needed = 1.0 - entry.burst_tokens;
            let seconds_until_token = tokens_needed / refill_rate;
            let retry_after = Duration::from_secs_f64(seconds_until_token);

            return Err(ToolError::RateLimited {
                source: None,
                source_message: format!(
                    "Token bucket rate limit: {} requests per {:?}",
                    self.max_requests, self.time_window
                ),
                context: format!("User exceeded rate limit of {} requests", self.max_requests),
                retry_after: Some(retry_after),
            });
        }

        // We have a token, consume it and allow the request
        entry.burst_tokens -= 1.0;
        entry.request_times.push(now);
        Ok(())
    }

    fn reset_client(&self, client_id: &str) {
        self.clients.remove(client_id);
    }

    fn clear_all(&self) {
        self.clients.clear();
    }

    fn get_request_count(&self, client_id: &str) -> usize {
        let now = Instant::now();
        self.clients
            .get(client_id)
            .map(|entry| {
                entry
                    .request_times
                    .iter()
                    .filter(|&&time| now.duration_since(time) < self.time_window)
                    .count()
            })
            .unwrap_or(0)
    }

    fn strategy_name(&self) -> &str {
        "TokenBucket"
    }
}
