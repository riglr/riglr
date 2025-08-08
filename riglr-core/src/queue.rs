//! Job queue abstractions and implementations.

use crate::jobs::Job;
use anyhow::Result;
use async_trait::async_trait;
use std::time::Duration;

/// Trait for job queue implementations
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
pub struct InMemoryJobQueue {
    queue: tokio::sync::Mutex<std::collections::VecDeque<Job>>,
    notify: tokio::sync::Notify,
}

impl InMemoryJobQueue {
    /// Create a new in-memory job queue
    pub fn new() -> Self {
        Self {
            queue: tokio::sync::Mutex::new(std::collections::VecDeque::new()),
            notify: tokio::sync::Notify::new(),
        }
    }
}

impl Default for InMemoryJobQueue {
    fn default() -> Self {
        Self::new()
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
        let mut conn = self.client.get_async_connection().await?;
        let serialized = serde_json::to_string(&job)?;
        redis::cmd("LPUSH")
            .arg(&self.queue_key)
            .arg(serialized)
            .query_async(&mut conn)
            .await?;
        Ok(())
    }
    
    async fn dequeue(&self) -> Result<Option<Job>> {
        let mut conn = self.client.get_async_connection().await?;
        
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
        let mut conn = self.client.get_async_connection().await?;
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
        let mut conn = self.client.get_async_connection().await?;
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
    
    #[tokio::test]
    async fn test_in_memory_queue() {
        let queue = InMemoryJobQueue::new();
        
        // Test enqueue and dequeue
        let job = Job::new("test_tool", &serde_json::json!({}), 3).unwrap();
        let job_id = job.job_id;
        
        queue.enqueue(job).await.unwrap();
        assert_eq!(queue.len().await.unwrap(), 1);
        assert!(!queue.is_empty().await.unwrap());
        
        // Use timeout to avoid blocking forever in tests
        let dequeued = queue.dequeue_with_timeout(Duration::from_secs(1)).await.unwrap();
        assert!(dequeued.is_some());
        assert_eq!(dequeued.unwrap().job_id, job_id);
        
        assert_eq!(queue.len().await.unwrap(), 0);
        assert!(queue.is_empty().await.unwrap());
    }
    
    #[tokio::test]
    async fn test_queue_timeout() {
        let queue = InMemoryJobQueue::new();
        
        // Dequeue with timeout should return None when queue is empty
        let result = queue.dequeue_with_timeout(Duration::from_millis(100)).await.unwrap();
        assert!(result.is_none());
    }
}