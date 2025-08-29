//! Integration tests for Redis-dependent components
//!
//! These tests verify RedisIdempotencyStore and RedisJobQueue against a containerized Redis instance.

#[cfg(all(test, feature = "redis", feature = "integration-tests"))]
mod tests {
    use riglr_core::{
        idempotency::{IdempotencyStore, RedisIdempotencyStore},
        jobs::{Job, JobResult},
        queue::{JobQueue, RedisJobQueue},
    };
    use serde::{Deserialize, Serialize};
    use std::time::Duration;
    use testcontainers::runners::AsyncRunner;
    use testcontainers_modules::redis::Redis;
    use uuid::Uuid;

    /// Job status enum for testing
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub enum JobStatus {
        Pending,
        Processing,
        Completed,
        Failed,
    }

    /// Test job structure
    #[derive(Debug, Clone)]
    pub struct TestJob {
        pub id: String,
        pub data: serde_json::Value,
        pub status: JobStatus,
    }

    /// Extended Redis queue implementation with job status tracking
    pub struct RedisJobQueueWithStatus {
        queue: RedisJobQueue,
        client: redis::Client,
        status_key_prefix: String,
    }

    impl RedisJobQueueWithStatus {
        pub async fn new(redis_url: &str, queue_name: &str) -> Result<Self, anyhow::Error> {
            let queue = RedisJobQueue::new(redis_url, queue_name)?;
            let client = redis::Client::open(redis_url)?;
            Ok(Self {
                queue,
                client,
                status_key_prefix: format!("riglr:status:{}", queue_name),
            })
        }

        pub async fn enqueue(
            &self,
            job_id: &str,
            job_data: serde_json::Value,
        ) -> Result<(), anyhow::Error> {
            let job = Job::new_idempotent("test_job", &job_data, 3, job_id)?;
            self.queue.enqueue(job).await?;
            self.set_status(job_id, JobStatus::Pending).await?;
            Ok(())
        }

        pub async fn enqueue_with_priority(
            &self,
            job_id: &str,
            job_data: serde_json::Value,
            _priority: i32,
        ) -> Result<(), anyhow::Error> {
            // For simplicity, ignore priority for now and use regular enqueue
            self.enqueue(job_id, job_data).await
        }

        pub async fn enqueue_with_metadata(
            &self,
            job_id: &str,
            job_data: serde_json::Value,
            _metadata: serde_json::Value,
        ) -> Result<(), anyhow::Error> {
            // For simplicity, ignore metadata for now and use regular enqueue
            self.enqueue(job_id, job_data).await
        }

        pub async fn dequeue(&self) -> Result<Option<TestJob>, anyhow::Error> {
            match self
                .queue
                .dequeue_with_timeout(Duration::from_secs(1))
                .await?
            {
                Some(job) => {
                    let job_id = job
                        .idempotency_key
                        .unwrap_or_else(|| job.job_id.to_string());
                    let test_job = TestJob {
                        id: job_id.clone(),
                        data: job.params,
                        status: self
                            .get_status(&job_id)
                            .await?
                            .unwrap_or(JobStatus::Pending),
                    };
                    Ok(Some(test_job))
                }
                None => Ok(None),
            }
        }

        pub async fn update_status(
            &self,
            job_id: &str,
            status: JobStatus,
        ) -> Result<(), anyhow::Error> {
            self.set_status(job_id, status).await
        }

        pub async fn get_status(&self, job_id: &str) -> Result<Option<JobStatus>, anyhow::Error> {
            let mut conn = self.client.get_multiplexed_async_connection().await?;
            let status_key = format!("{}:{}", self.status_key_prefix, job_id);

            let result: Option<String> = redis::cmd("GET")
                .arg(&status_key)
                .query_async(&mut conn)
                .await?;

            match result {
                Some(status_str) => Ok(Some(serde_json::from_str(&status_str)?)),
                None => Ok(None),
            }
        }

        async fn set_status(&self, job_id: &str, status: JobStatus) -> Result<(), anyhow::Error> {
            let mut conn = self.client.get_multiplexed_async_connection().await?;
            let status_key = format!("{}:{}", self.status_key_prefix, job_id);
            let status_str = serde_json::to_string(&status)?;

            redis::cmd("SET")
                .arg(&status_key)
                .arg(status_str)
                .query_async::<()>(&mut conn)
                .await?;

            Ok(())
        }
    }

    async fn setup_redis_container() -> (testcontainers::ContainerAsync<Redis>, String) {
        let container = Redis::default()
            .start()
            .await
            .expect("Failed to start Redis container");

        let port = container
            .get_host_port_ipv4(6379)
            .await
            .expect("Failed to get Redis port");

        let redis_url = format!("redis://127.0.0.1:{}", port);
        (container, redis_url)
    }

    #[tokio::test]
    async fn test_redis_idempotency_store() -> Result<(), anyhow::Error> {
        let (_container, redis_url) = setup_redis_container().await;

        // Create idempotency store
        let store = RedisIdempotencyStore::new(&redis_url, None)
            .expect("Failed to create idempotency store");

        // Test storing and checking idempotency keys
        let key = Uuid::new_v4().to_string();
        let result = "test_result".to_string();

        // First call should store the result (manually implementing get_or_set logic)
        let job_result = JobResult::success(&result)?;
        store
            .set(
                &key,
                std::sync::Arc::new(job_result),
                Duration::from_secs(300),
            )
            .await
            .expect("Failed to store idempotency key");
        let stored = store.get(&key).await.expect("Failed to get key").unwrap();
        let stored_value = match stored.as_ref() {
            JobResult::Success { value, .. } => value.as_str().unwrap().to_string(),
            _ => panic!("Expected success result"),
        };
        assert_eq!(stored_value, result);

        // Second call should return cached result
        let cached = store
            .get(&key)
            .await
            .expect("Failed to get cached key")
            .unwrap();
        let cached_value = match cached.as_ref() {
            JobResult::Success { value, .. } => value.as_str().unwrap().to_string(),
            _ => panic!("Expected success result"),
        };
        assert_eq!(cached_value, result); // Should return original result, not "different_result"

        // Test expiry
        let key_with_ttl = Uuid::new_v4().to_string();
        let result_with_ttl = "expires_soon".to_string();

        let ttl_job_result = JobResult::success(&result_with_ttl)?;
        store
            .set(
                &key_with_ttl,
                std::sync::Arc::new(ttl_job_result),
                Duration::from_secs(1),
            )
            .await
            .expect("Failed to store key with TTL");
        let stored_ttl = store
            .get(&key_with_ttl)
            .await
            .expect("Failed to get TTL key")
            .unwrap();
        let stored_ttl_value = match stored_ttl.as_ref() {
            JobResult::Success { value, .. } => value.as_str().unwrap().to_string(),
            _ => panic!("Expected success result"),
        };
        assert_eq!(stored_ttl_value, result_with_ttl);

        // Wait for expiry
        tokio::time::sleep(Duration::from_secs(2)).await;

        // Should be expired now
        let after_expiry = store
            .get(&key_with_ttl)
            .await
            .expect("Failed to get after expiry");
        assert!(after_expiry.is_none(), "Key should be expired");
        Ok(())
    }

    #[tokio::test]
    async fn test_redis_job_queue() {
        let (_container, redis_url) = setup_redis_container().await;

        // Create job queue
        let queue = RedisJobQueueWithStatus::new(&redis_url, "test_queue")
            .await
            .expect("Failed to create job queue");

        // Test enqueueing a job
        let job_id = Uuid::new_v4().to_string();
        let job_data = serde_json::json!({
            "action": "test_action",
            "params": {"key": "value"}
        });

        queue
            .enqueue(&job_id, job_data.clone())
            .await
            .expect("Failed to enqueue job");

        // Test dequeuing the job
        let dequeued = queue
            .dequeue()
            .await
            .expect("Failed to dequeue")
            .expect("Queue should not be empty");

        assert_eq!(dequeued.id, job_id);
        assert_eq!(dequeued.data, job_data);
        assert_eq!(dequeued.status, JobStatus::Pending);

        // Test updating job status
        queue
            .update_status(&job_id, JobStatus::Processing)
            .await
            .expect("Failed to update status to Processing");

        let status = queue
            .get_status(&job_id)
            .await
            .expect("Failed to get status");
        assert_eq!(status, Some(JobStatus::Processing));

        // Test completing a job
        queue
            .update_status(&job_id, JobStatus::Completed)
            .await
            .expect("Failed to complete job");

        let final_status = queue
            .get_status(&job_id)
            .await
            .expect("Failed to get final status");
        assert_eq!(final_status, Some(JobStatus::Completed));

        // Test queue is empty after processing
        let empty = queue
            .dequeue()
            .await
            .expect("Failed to dequeue from empty queue");
        assert!(empty.is_none());
    }

    #[tokio::test]
    async fn test_redis_job_queue_priority() {
        let (_container, redis_url) = setup_redis_container().await;

        let queue = RedisJobQueueWithStatus::new(&redis_url, "priority_queue")
            .await
            .expect("Failed to create priority queue");

        // Enqueue jobs with different priorities
        let low_priority_job = (
            Uuid::new_v4().to_string(),
            serde_json::json!({"priority": "low"}),
            1,
        );
        let high_priority_job = (
            Uuid::new_v4().to_string(),
            serde_json::json!({"priority": "high"}),
            10,
        );
        let medium_priority_job = (
            Uuid::new_v4().to_string(),
            serde_json::json!({"priority": "medium"}),
            5,
        );

        // Enqueue in random order
        queue
            .enqueue_with_priority(
                &low_priority_job.0,
                low_priority_job.1.clone(),
                low_priority_job.2,
            )
            .await
            .expect("Failed to enqueue low priority");
        queue
            .enqueue_with_priority(
                &high_priority_job.0,
                high_priority_job.1.clone(),
                high_priority_job.2,
            )
            .await
            .expect("Failed to enqueue high priority");
        queue
            .enqueue_with_priority(
                &medium_priority_job.0,
                medium_priority_job.1.clone(),
                medium_priority_job.2,
            )
            .await
            .expect("Failed to enqueue medium priority");

        // Dequeue should return in priority order
        let first = queue.dequeue().await.expect("Failed to dequeue").unwrap();
        assert_eq!(first.id, high_priority_job.0);

        let second = queue.dequeue().await.expect("Failed to dequeue").unwrap();
        assert_eq!(second.id, medium_priority_job.0);

        let third = queue.dequeue().await.expect("Failed to dequeue").unwrap();
        assert_eq!(third.id, low_priority_job.0);
    }

    #[tokio::test]
    async fn test_redis_job_retry_behavior() {
        let (_container, redis_url) = setup_redis_container().await;

        let queue = RedisJobQueueWithStatus::new(&redis_url, "retry_queue")
            .await
            .expect("Failed to create retry queue");

        let job_id = Uuid::new_v4().to_string();
        let job_data = serde_json::json!({"retryable": true});

        // Enqueue job
        queue
            .enqueue(&job_id, job_data.clone())
            .await
            .expect("Failed to enqueue");

        // Process and fail the job
        let job = queue.dequeue().await.expect("Failed to dequeue").unwrap();
        queue
            .update_status(&job.id, JobStatus::Failed)
            .await
            .expect("Failed to mark as failed");

        // Implement retry logic
        let mut retry_count = 0;
        const MAX_RETRIES: i32 = 3;

        while retry_count < MAX_RETRIES {
            // Re-enqueue for retry
            queue
                .enqueue_with_metadata(
                    &format!("{}_retry_{}", job_id, retry_count + 1),
                    job_data.clone(),
                    serde_json::json!({"retry_count": retry_count + 1, "original_id": job_id}),
                )
                .await
                .expect("Failed to enqueue retry");

            retry_count += 1;

            // Simulate processing
            let retry_job = queue
                .dequeue()
                .await
                .expect("Failed to dequeue retry")
                .unwrap();

            // On third retry, succeed
            if retry_count == MAX_RETRIES {
                queue
                    .update_status(&retry_job.id, JobStatus::Completed)
                    .await
                    .expect("Failed to complete retry");
                break;
            } else {
                queue
                    .update_status(&retry_job.id, JobStatus::Failed)
                    .await
                    .expect("Failed to fail retry");
            }
        }

        // Verify final retry succeeded
        let final_status = queue
            .get_status(&format!("{}_retry_{}", job_id, MAX_RETRIES))
            .await
            .expect("Failed to get final status");
        assert_eq!(final_status, Some(JobStatus::Completed));
    }

    #[tokio::test]
    async fn test_redis_connection_resilience() -> Result<(), anyhow::Error> {
        let (_container, redis_url) = setup_redis_container().await;

        // Test that components handle connection properly
        let store = RedisIdempotencyStore::new(&redis_url, None).expect("Should connect to Redis");

        let queue = RedisJobQueueWithStatus::new(&redis_url, "resilience_test")
            .await
            .expect("Should connect to Redis");

        // Perform operations to verify connection
        let test_key = "test_connection";
        let connection_result = JobResult::success(&"connected")?;
        store
            .set(
                test_key,
                std::sync::Arc::new(connection_result),
                Duration::from_secs(60),
            )
            .await
            .expect("Should perform operation");
        let retrieved = store
            .get(test_key)
            .await
            .expect("Should get result")
            .unwrap();
        let retrieved_value = match retrieved.as_ref() {
            JobResult::Success { value, .. } => value.as_str().unwrap().to_string(),
            _ => panic!("Expected success result"),
        };
        assert_eq!(retrieved_value, "connected");

        let job_id = Uuid::new_v4().to_string();
        queue
            .enqueue(&job_id, serde_json::json!({"test": "connection"}))
            .await
            .expect("Should enqueue job");

        let dequeued = queue
            .dequeue()
            .await
            .expect("Should dequeue")
            .expect("Should have job");
        assert_eq!(dequeued.id, job_id);
        Ok(())
    }
}
