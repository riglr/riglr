//! Job queue abstractions and implementations.
//!
//! This module provides the queue infrastructure for distributed job processing,
//! supporting both in-memory and Redis-backed implementations for scalability.

use crate::jobs::Job;
use anyhow::Result;
use async_trait::async_trait;
use std::time::Duration;

/// Trait for job queue implementations.
///
/// Provides a common interface for different queue backends, enabling
/// both local development with in-memory queues and production deployment
/// with distributed Redis queues.
#[async_trait]
pub trait JobQueue: Send + Sync {
    /// Add a job to the queue
    async fn enqueue(&self, job: Job) -> Result<()>;

    /// Get the next job from the queue, blocks until a job is available or timeout
    async fn dequeue(&self) -> Result<Option<Job>>;

    /// Get the next job from the queue with timeout
    async fn dequeue_with_timeout(&self, timeout: Duration) -> Result<Option<Job>>;

    /// Get queue length
    async fn len(&self) -> Result<usize>;

    /// Check if queue is empty
    async fn is_empty(&self) -> Result<bool> {
        Ok(self.len().await? == 0)
    }
}

/// In-memory job queue implementation for testing and development
#[derive(Debug)]
pub struct InMemoryJobQueue {
    queue: tokio::sync::Mutex<std::collections::VecDeque<Job>>,
    notify: tokio::sync::Notify,
}

impl InMemoryJobQueue {
    /// Create a new in-memory job queue
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for InMemoryJobQueue {
    fn default() -> Self {
        Self {
            queue: tokio::sync::Mutex::new(std::collections::VecDeque::default()),
            notify: tokio::sync::Notify::default(),
        }
    }
}

#[async_trait]
impl JobQueue for InMemoryJobQueue {
    async fn enqueue(&self, job: Job) -> Result<()> {
        let mut queue = self.queue.lock().await;
        queue.push_back(job);
        self.notify.notify_one();
        Ok(())
    }

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
            _ = tokio::time::sleep(timeout) => Ok(None),
            _ = self.notify.notified() => {
                let mut queue = self.queue.lock().await;
                Ok(queue.pop_front())
            }
        }
    }

    async fn len(&self) -> Result<usize> {
        let queue = self.queue.lock().await;
        Ok(queue.len())
    }
}

/// Redis-based job queue implementation for production use
#[cfg(feature = "redis")]
#[derive(Debug)]
pub struct RedisJobQueue {
    client: redis::Client,
    queue_key: String,
    timeout_seconds: u64,
}

#[cfg(feature = "redis")]
impl RedisJobQueue {
    /// Create a new Redis job queue
    ///
    /// # Arguments
    /// * `redis_url` - Redis connection URL (e.g., "redis://127.0.0.1:6379")
    /// * `queue_name` - Name of the queue (will be prefixed with "riglr:queue:")
    pub fn new(redis_url: &str, queue_name: &str) -> Result<Self> {
        let client = redis::Client::open(redis_url)?;
        Ok(Self {
            client,
            queue_key: format!("riglr:queue:{}", queue_name),
            timeout_seconds: 5,
        })
    }

    /// Set the blocking timeout for dequeue operations
    pub fn with_timeout(mut self, timeout_seconds: u64) -> Self {
        self.timeout_seconds = timeout_seconds;
        self
    }
}

#[cfg(feature = "redis")]
#[async_trait]
impl JobQueue for RedisJobQueue {
    async fn enqueue(&self, job: Job) -> Result<()> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let serialized = serde_json::to_string(&job)?;
        let _: () = redis::cmd("LPUSH")
            .arg(&self.queue_key)
            .arg(serialized)
            .query_async(&mut conn)
            .await?;
        Ok(())
    }

    async fn dequeue(&self) -> Result<Option<Job>> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;

        // BRPOP blocks until an item is available or timeout
        let result: Option<(String, String)> = redis::cmd("BRPOP")
            .arg(&self.queue_key)
            .arg(self.timeout_seconds)
            .query_async(&mut conn)
            .await?;

        match result {
            Some((_, job_str)) => {
                let job: Job = serde_json::from_str(&job_str)?;
                Ok(Some(job))
            }
            None => Ok(None),
        }
    }

    async fn dequeue_with_timeout(&self, timeout: Duration) -> Result<Option<Job>> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let timeout_seconds = timeout.as_secs().max(1);

        let result: Option<(String, String)> = redis::cmd("BRPOP")
            .arg(&self.queue_key)
            .arg(timeout_seconds)
            .query_async(&mut conn)
            .await?;

        match result {
            Some((_, job_str)) => {
                let job: Job = serde_json::from_str(&job_str)?;
                Ok(Some(job))
            }
            None => Ok(None),
        }
    }

    async fn len(&self) -> Result<usize> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let len: usize = redis::cmd("LLEN")
            .arg(&self.queue_key)
            .query_async(&mut conn)
            .await?;
        Ok(len)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tokio::time::{timeout, Duration};

    // Test InMemoryJobQueue::default()
    #[tokio::test]
    async fn test_in_memory_queue_default() {
        let queue = InMemoryJobQueue::default();
        assert_eq!(queue.len().await.unwrap(), 0);
        assert!(queue.is_empty().await.unwrap());
    }

    // Test basic enqueue and dequeue operations
    #[tokio::test]
    async fn test_in_memory_queue_enqueue_dequeue() {
        let queue = InMemoryJobQueue::default();

        // Test enqueue and dequeue
        let job = Job::new("test_tool", &serde_json::json!({}), 3).unwrap();
        let job_id = job.job_id;

        queue.enqueue(job).await.unwrap();
        assert_eq!(queue.len().await.unwrap(), 1);
        assert!(!queue.is_empty().await.unwrap());

        // Use timeout to avoid blocking forever in tests
        let dequeued = queue
            .dequeue_with_timeout(Duration::from_secs(1))
            .await
            .unwrap();
        assert!(dequeued.is_some());
        assert_eq!(dequeued.unwrap().job_id, job_id);

        assert_eq!(queue.len().await.unwrap(), 0);
        assert!(queue.is_empty().await.unwrap());
    }

    // Test dequeue with timeout when queue is empty
    #[tokio::test]
    async fn test_in_memory_queue_dequeue_timeout_empty() {
        let queue = InMemoryJobQueue::default();

        // Dequeue with timeout should return None when queue is empty
        let result = queue
            .dequeue_with_timeout(Duration::from_millis(100))
            .await
            .unwrap();
        assert!(result.is_none());
    }

    // Test dequeue with timeout when job is immediately available
    #[tokio::test]
    async fn test_in_memory_queue_dequeue_timeout_immediate() {
        let queue = InMemoryJobQueue::default();
        let job = Job::new("test_tool", &serde_json::json!({}), 3).unwrap();
        let job_id = job.job_id;

        // Enqueue first
        queue.enqueue(job).await.unwrap();

        // Dequeue with timeout should return immediately
        let result = queue
            .dequeue_with_timeout(Duration::from_secs(1))
            .await
            .unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().job_id, job_id);
    }

    // Test dequeue with timeout when job becomes available after waiting
    #[tokio::test]
    async fn test_in_memory_queue_dequeue_timeout_wait_for_job() {
        let queue = Arc::new(InMemoryJobQueue::default());
        let queue_clone = Arc::clone(&queue);

        // Spawn a task to enqueue a job after a short delay
        let job = Job::new("delayed_tool", &serde_json::json!({}), 3).unwrap();
        let job_id = job.job_id;

        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(50)).await;
            queue_clone.enqueue(job).await.unwrap();
        });

        // Dequeue should wait and then receive the job
        let result = queue
            .dequeue_with_timeout(Duration::from_secs(1))
            .await
            .unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().job_id, job_id);
    }

    // Test blocking dequeue with concurrent enqueue
    #[tokio::test]
    async fn test_in_memory_queue_dequeue_blocking() {
        let queue = Arc::new(InMemoryJobQueue::default());
        let queue_clone = Arc::clone(&queue);

        let job = Job::new("blocking_test", &serde_json::json!({}), 3).unwrap();
        let job_id = job.job_id;

        // Spawn a task to enqueue after a delay
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(50)).await;
            queue_clone.enqueue(job).await.unwrap();
        });

        // Use timeout wrapper to prevent infinite blocking in tests
        let result = timeout(Duration::from_secs(1), queue.dequeue()).await;
        assert!(result.is_ok());
        let dequeued = result.unwrap().unwrap();
        assert!(dequeued.is_some());
        assert_eq!(dequeued.unwrap().job_id, job_id);
    }

    // Test multiple enqueue and dequeue operations (FIFO order)
    #[tokio::test]
    async fn test_in_memory_queue_fifo_order() {
        let queue = InMemoryJobQueue::default();

        // Enqueue multiple jobs
        let job1 = Job::new("tool1", &serde_json::json!({}), 3).unwrap();
        let job2 = Job::new("tool2", &serde_json::json!({}), 3).unwrap();
        let job3 = Job::new("tool3", &serde_json::json!({}), 3).unwrap();

        let job1_id = job1.job_id;
        let job2_id = job2.job_id;
        let job3_id = job3.job_id;

        queue.enqueue(job1).await.unwrap();
        queue.enqueue(job2).await.unwrap();
        queue.enqueue(job3).await.unwrap();

        assert_eq!(queue.len().await.unwrap(), 3);

        // Dequeue should return jobs in FIFO order
        let dequeued1 = queue
            .dequeue_with_timeout(Duration::from_secs(1))
            .await
            .unwrap();
        assert_eq!(dequeued1.unwrap().job_id, job1_id);

        let dequeued2 = queue
            .dequeue_with_timeout(Duration::from_secs(1))
            .await
            .unwrap();
        assert_eq!(dequeued2.unwrap().job_id, job2_id);

        let dequeued3 = queue
            .dequeue_with_timeout(Duration::from_secs(1))
            .await
            .unwrap();
        assert_eq!(dequeued3.unwrap().job_id, job3_id);

        assert_eq!(queue.len().await.unwrap(), 0);
        assert!(queue.is_empty().await.unwrap());
    }

    // Test is_empty default implementation when queue has items
    #[tokio::test]
    async fn test_in_memory_queue_is_empty_with_items() {
        let queue = InMemoryJobQueue::default();
        let job = Job::new("test_tool", &serde_json::json!({}), 3).unwrap();

        queue.enqueue(job).await.unwrap();
        assert!(!queue.is_empty().await.unwrap());
        assert_eq!(queue.len().await.unwrap(), 1);
    }

    // Test len() with multiple items
    #[tokio::test]
    async fn test_in_memory_queue_len_multiple_items() {
        let queue = InMemoryJobQueue::default();

        assert_eq!(queue.len().await.unwrap(), 0);

        for i in 0..5 {
            let job = Job::new(&format!("tool_{}", i), &serde_json::json!({}), 3).unwrap();
            queue.enqueue(job).await.unwrap();
            assert_eq!(queue.len().await.unwrap(), i + 1);
        }
    }

    // Test concurrent enqueue operations
    #[tokio::test]
    async fn test_in_memory_queue_concurrent_enqueue() {
        let queue = Arc::new(InMemoryJobQueue::default());
        let mut handles = vec![];

        // Spawn multiple tasks to enqueue concurrently
        for i in 0..10 {
            let queue_clone = Arc::clone(&queue);
            let handle = tokio::spawn(async move {
                let job =
                    Job::new(&format!("concurrent_tool_{}", i), &serde_json::json!({}), 3).unwrap();
                queue_clone.enqueue(job).await.unwrap();
            });
            handles.push(handle);
        }

        // Wait for all enqueue operations to complete
        for handle in handles {
            handle.await.unwrap();
        }

        assert_eq!(queue.len().await.unwrap(), 10);
    }

    // Test edge case: very short timeout
    #[tokio::test]
    async fn test_in_memory_queue_very_short_timeout() {
        let queue = InMemoryJobQueue::default();

        let result = queue
            .dequeue_with_timeout(Duration::from_nanos(1))
            .await
            .unwrap();
        assert!(result.is_none());
    }

    // Test zero timeout
    #[tokio::test]
    async fn test_in_memory_queue_zero_timeout() {
        let queue = InMemoryJobQueue::default();

        let result = queue
            .dequeue_with_timeout(Duration::from_secs(0))
            .await
            .unwrap();
        assert!(result.is_none());
    }

    #[cfg(feature = "redis")]
    mod redis_tests {
        use super::*;

        // Test RedisJobQueue::new with valid URL
        #[test]
        fn test_redis_queue_new_valid_url() {
            let result = RedisJobQueue::new("redis://127.0.0.1:6379", "test_queue");
            assert!(result.is_ok());
            let queue = result.unwrap();
            assert_eq!(queue.queue_key, "riglr:queue:test_queue");
            assert_eq!(queue.timeout_seconds, 5);
        }

        // Test RedisJobQueue::new with invalid URL
        #[test]
        fn test_redis_queue_new_invalid_url() {
            let result = RedisJobQueue::new("invalid_url", "test_queue");
            assert!(result.is_err());
        }

        // Test RedisJobQueue::with_timeout
        #[test]
        fn test_redis_queue_with_timeout() {
            let queue = RedisJobQueue::new("redis://127.0.0.1:6379", "test_queue")
                .unwrap()
                .with_timeout(10);
            assert_eq!(queue.timeout_seconds, 10);
        }

        // Test queue key formatting with different queue names
        #[test]
        fn test_redis_queue_key_formatting() {
            let queue1 = RedisJobQueue::new("redis://127.0.0.1:6379", "simple").unwrap();
            assert_eq!(queue1.queue_key, "riglr:queue:simple");

            let queue2 = RedisJobQueue::new("redis://127.0.0.1:6379", "complex_name_123").unwrap();
            assert_eq!(queue2.queue_key, "riglr:queue:complex_name_123");

            let queue3 = RedisJobQueue::new("redis://127.0.0.1:6379", "").unwrap();
            assert_eq!(queue3.queue_key, "riglr:queue:");
        }

        // Test with_timeout chaining
        #[test]
        fn test_redis_queue_timeout_chaining() {
            let queue = RedisJobQueue::new("redis://127.0.0.1:6379", "test")
                .unwrap()
                .with_timeout(15)
                .with_timeout(20);
            assert_eq!(queue.timeout_seconds, 20);
        }

        // Integration tests would require a running Redis instance
        // These are commented out but show the structure for full integration testing
        /*
        #[tokio::test]
        async fn test_redis_queue_integration() {
            // This test would require a running Redis instance
            let queue = RedisJobQueue::new("redis://127.0.0.1:6379", "test_integration").unwrap();

            let job = Job::new("redis_test", &serde_json::json!({}), 3).unwrap();
            let job_id = job.job_id;

            // Test enqueue
            queue.enqueue(job).await.unwrap();
            assert_eq!(queue.len().await.unwrap(), 1);
            assert!(!queue.is_empty().await.unwrap());

            // Test dequeue
            let dequeued = queue.dequeue_with_timeout(Duration::from_secs(1)).await.unwrap();
            assert!(dequeued.is_some());
            assert_eq!(dequeued.unwrap().job_id, job_id);

            assert_eq!(queue.len().await.unwrap(), 0);
            assert!(queue.is_empty().await.unwrap());
        }

        #[tokio::test]
        async fn test_redis_queue_timeout_scenarios() {
            let queue = RedisJobQueue::new("redis://127.0.0.1:6379", "test_timeout").unwrap();

            // Test timeout when queue is empty
            let result = queue.dequeue_with_timeout(Duration::from_millis(100)).await.unwrap();
            assert!(result.is_none());

            // Test very long timeout handling
            let result = queue.dequeue_with_timeout(Duration::from_secs(3600)).await.unwrap();
            assert!(result.is_none());
        }
        */
    }
}
