//! Job queue abstractions and implementations.
//!
//! This module provides the queue infrastructure for distributed job processing,
//! supporting both in-memory and Redis-backed implementations for scalability.

extern crate alloc;

use crate::jobs::Job;

use alloc::collections::VecDeque;
use anyhow::Result;
use async_trait::async_trait;
use core::time::Duration;
use tokio::sync::{Mutex, Notify};
use tokio::time::sleep;

/// Trait for queue implementations.
///
/// Provides a common interface for different queue backends, enabling
/// both local development with in-memory queues and production deployment
/// with distributed Redis queues.
#[async_trait]
pub trait Queue: Send + Sync {
    /// Get the next job from the queue, blocks until a job is available or timeout
    async fn dequeue(&self) -> Result<Option<Job>>;

    /// Get the next job from the queue with timeout
    async fn dequeue_with_timeout(&self, timeout: Duration) -> Result<Option<Job>>;

    /// Add a job to the queue
    async fn enqueue(&self, job: Job) -> Result<()>;

    /// Check if queue is empty
    #[inline]
    async fn is_empty(&self) -> Result<bool> {
        Ok(self.len().await? == 0)
    }

    /// Get queue length
    async fn len(&self) -> Result<usize>;
}

/// In-memory queue implementation for testing and development
#[derive(Debug)]
pub struct InMemory {
    /// Notification primitive for waiting consumers
    notify: Notify,
    /// Thread-safe job storage queue
    queue: Mutex<VecDeque<Job>>,
}

impl InMemory {
    /// Create a new in-memory queue
    #[must_use]
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for InMemory {
    #[inline]
    fn default() -> Self {
        Self {
            notify: Notify::default(),
            queue: Mutex::new(VecDeque::default()),
        }
    }
}

#[async_trait]
impl Queue for InMemory {
    #[inline]
    async fn dequeue(&self) -> Result<Option<Job>> {
        loop {
            {
                let mut queue = self.queue.lock().await;
                if let Some(job) = queue.pop_front() {
                    return Ok(Some(job));
                }
            }
            self.notify.notified().await;
        }
    }

    #[inline]
    #[expect(clippy::ignored_unit_patterns)]
    async fn dequeue_with_timeout(&self, timeout: Duration) -> Result<Option<Job>> {
        // First check if there are any items immediately available
        {
            let mut queue = self.queue.lock().await;
            if let Some(job) = queue.pop_front() {
                return Ok(Some(job));
            }
        }

        // If no items available, wait for notification or timeout
        tokio::select! {
            _ = sleep(timeout) => return Ok(None),
            _ = self.notify.notified() => {
                let mut queue = self.queue.lock().await;
                Ok(queue.pop_front())
            }
        }
    }

    #[inline]
    async fn enqueue(&self, job: Job) -> Result<()> {
        self.queue.lock().await.push_back(job);
        self.notify.notify_one();
        Ok(())
    }

    #[inline]
    async fn is_empty(&self) -> Result<bool> {
        Ok(self.len().await? == 0)
    }

    #[inline]
    async fn len(&self) -> Result<usize> {
        let queue = self.queue.lock().await;
        Ok(queue.len())
    }
}

/// Redis-based queue implementation for production use
#[cfg(feature = "redis")]
#[derive(Debug)]
pub struct Redis {
    /// Redis client connection
    client: redis::Client,
    /// Redis key for the queue
    queue_key: String,
    /// Default timeout in seconds for blocking operations
    timeout_seconds: u64,
}

#[cfg(feature = "redis")]
impl Redis {
    /// Create a new Redis queue
    ///
    /// # Arguments
    /// * `redis_url` - Redis connection URL (e.g., `redis://127.0.0.1:6379`)
    /// * `queue_name` - Name of the queue (will be prefixed with "riglr:queue:")
    ///
    /// # Errors
    /// Returns error if Redis client cannot be created with the provided URL
    #[inline]
    pub fn new(redis_url: &str, queue_name: &str) -> Result<Self> {
        match redis::Client::open(redis_url) {
            Ok(client) => Ok(Self {
                client,
                queue_key: format!("riglr:queue:{queue_name}"),
                timeout_seconds: 5,
            }),
            Err(redis_error) => Err(anyhow::Error::from(redis_error)),
        }
    }

    /// Set the blocking timeout for dequeue operations
    #[must_use]
    #[inline]
    pub const fn with_timeout(mut self, timeout_seconds: u64) -> Self {
        self.timeout_seconds = timeout_seconds;
        self
    }
}

#[cfg(feature = "redis")]
#[async_trait]
impl Queue for Redis {
    #[inline]
    async fn dequeue(&self) -> Result<Option<Job>> {
        match self.client.get_multiplexed_async_connection().await {
            Ok(mut conn) => {
                // BRPOP blocks until an item is available or timeout
                match redis::cmd("BRPOP")
                    .arg(&self.queue_key)
                    .arg(self.timeout_seconds)
                    .query_async::<Option<(String, String)>>(&mut conn)
                    .await
                {
                    Ok(Some((_, job_str))) => match serde_json::from_str::<Job>(&job_str) {
                        Ok(job) => Ok(Some(job)),
                        Err(serde_error) => Err(anyhow::Error::from(serde_error)),
                    },
                    Ok(None) => Ok(None),
                    Err(redis_error) => Err(anyhow::Error::from(redis_error)),
                }
            }
            Err(connection_error) => Err(anyhow::Error::from(connection_error)),
        }
    }

    #[inline]
    async fn dequeue_with_timeout(&self, timeout: Duration) -> Result<Option<Job>> {
        match self.client.get_multiplexed_async_connection().await {
            Ok(mut conn) => {
                let timeout_seconds = timeout.as_secs().max(1);
                match redis::cmd("BRPOP")
                    .arg(&self.queue_key)
                    .arg(timeout_seconds)
                    .query_async::<Option<(String, String)>>(&mut conn)
                    .await
                {
                    Ok(Some((_, job_str))) => match serde_json::from_str::<Job>(&job_str) {
                        Ok(job) => Ok(Some(job)),
                        Err(serde_error) => Err(anyhow::Error::from(serde_error)),
                    },
                    Ok(None) => Ok(None),
                    Err(redis_error) => Err(anyhow::Error::from(redis_error)),
                }
            }
            Err(connection_error) => Err(anyhow::Error::from(connection_error)),
        }
    }

    #[inline]
    async fn enqueue(&self, job: Job) -> Result<()> {
        match self.client.get_multiplexed_async_connection().await {
            Ok(mut conn) => match serde_json::to_string(&job) {
                Ok(serialized) => {
                    match redis::cmd("LPUSH")
                        .arg(&self.queue_key)
                        .arg(serialized)
                        .query_async::<()>(&mut conn)
                        .await
                    {
                        Ok(()) => Ok(()),
                        Err(redis_error) => Err(anyhow::Error::from(redis_error)),
                    }
                }
                Err(serde_error) => Err(anyhow::Error::from(serde_error)),
            },
            Err(connection_error) => Err(anyhow::Error::from(connection_error)),
        }
    }

    #[inline]
    async fn is_empty(&self) -> Result<bool> {
        Ok(self.len().await? == 0)
    }

    #[inline]
    async fn len(&self) -> Result<usize> {
        match self.client.get_multiplexed_async_connection().await {
            Ok(mut conn) => {
                match redis::cmd("LLEN")
                    .arg(&self.queue_key)
                    .query_async::<usize>(&mut conn)
                    .await
                {
                    Ok(len) => Ok(len),
                    Err(redis_error) => Err(anyhow::Error::from(redis_error)),
                }
            }
            Err(connection_error) => Err(anyhow::Error::from(connection_error)),
        }
    }
}

#[cfg(test)]
#[expect(clippy::expect_used)]
mod tests {
    use super::*;

    // Allow expect() usage in test code following established project patterns
    mod redis_tests {
        use super::*;

        #[test]
        fn redis_queue_new_valid_url() {
            let result = Redis::new("redis://127.0.0.1:6379", "test_queue");
            assert!(result.is_ok());
            let queue = result.expect("Failed to create Redis queue with valid URL");
            assert_eq!(queue.queue_key, "riglr:queue:test_queue");
            assert_eq!(queue.timeout_seconds, 5);
        }

        #[test]
        fn redis_queue_new_invalid_url() {
            let result = Redis::new("invalid_url", "test_queue");
            assert!(result.is_err());
        }

        #[test]
        fn redis_queue_with_timeout() {
            let queue = Redis::new("redis://127.0.0.1:6379", "test_queue")
                .expect("Failed to create Redis queue for timeout test")
                .with_timeout(10);
            assert_eq!(queue.timeout_seconds, 10);
        }

        #[test]
        fn redis_queue_key_formatting() {
            let queue1 = Redis::new("redis://127.0.0.1:6379", "simple")
                .expect("Failed to create simple queue");
            assert_eq!(queue1.queue_key, "riglr:queue:simple");

            let queue2 = Redis::new("redis://127.0.0.1:6379", "complex_name_123")
                .expect("Failed to create complex named queue");
            assert_eq!(queue2.queue_key, "riglr:queue:complex_name_123");

            let queue3 = Redis::new("redis://127.0.0.1:6379", "")
                .expect("Failed to create empty named queue");
            assert_eq!(queue3.queue_key, "riglr:queue:");
        }

        #[test]
        fn redis_queue_timeout_chaining() {
            let queue = Redis::new("redis://127.0.0.1:6379", "test")
                .expect("Failed to create Redis queue for chaining test")
                .with_timeout(15)
                .with_timeout(20);
            assert_eq!(queue.timeout_seconds, 20);
        }
    }

    #[tokio::test]
    async fn in_memory_queue_zero_timeout() {
        let queue = InMemory::default();

        let result = queue
            .dequeue_with_timeout(Duration::from_secs(0))
            .await
            .expect("Failed to dequeue with zero timeout");
        assert!(result.is_none());
    }
}
