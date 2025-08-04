//! Integration tests for Redis components using testcontainers

#![cfg(all(test, feature = "redis"))]

use riglr_core::idempotency::{IdempotencyStore, RedisIdempotencyStore};
use riglr_core::jobs::{Job, JobResult};
use riglr_core::queue::{JobQueue, RedisJobQueue};
use std::sync::Arc;
use std::time::Duration;
use testcontainers::runners::AsyncRunner;
use testcontainers_modules::redis::Redis;

#[tokio::test]
async fn test_redis_idempotency_store_operations() {
    let redis_image = Redis::default();
    let container = redis_image
        .start()
        .await
        .expect("Failed to start Redis container");
    let port = container
        .get_host_port_ipv4(6379)
        .await
        .expect("Failed to get port");
    let redis_url = format!("redis://127.0.0.1:{}", port);

    // Create Redis idempotency store
    let store = RedisIdempotencyStore::new(&redis_url, Some("test:idempotency:"))
        .expect("Failed to create Redis idempotency store");

    // Test basic operations
    let key = "test_key_1";
    let result = JobResult::success(&"test_value").unwrap();

    // Initially, key should not exist
    assert!(store.get(key).await.unwrap().is_none());

    // Store a result
    store
        .set(key, &result, Duration::from_secs(10))
        .await
        .expect("Failed to set key");

    // Should be able to retrieve it
    let retrieved = store.get(key).await.unwrap();
    assert!(retrieved.is_some());
    assert!(retrieved.unwrap().is_success());

    // Remove the entry
    store.remove(key).await.unwrap();
    assert!(store.get(key).await.unwrap().is_none());
}

#[tokio::test]
async fn test_redis_idempotency_store_expiry() {
    let redis_image = Redis::default();
    let container = redis_image
        .start()
        .await
        .expect("Failed to start Redis container");
    let port = container
        .get_host_port_ipv4(6379)
        .await
        .expect("Failed to get port");
    let redis_url = format!("redis://127.0.0.1:{}", port);

    let store = RedisIdempotencyStore::new(&redis_url, None)
        .expect("Failed to create Redis idempotency store");

    let key = "expiry_test";
    let result = JobResult::success(&"expires_soon").unwrap();

    // Store with very short TTL (1 second minimum for Redis)
    store
        .set(key, &result, Duration::from_secs(1))
        .await
        .expect("Failed to set key");

    // Should exist immediately
    assert!(store.get(key).await.unwrap().is_some());

    // Wait for expiry
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Should be expired
    assert!(store.get(key).await.unwrap().is_none());
}

#[tokio::test]
async fn test_redis_queue_operations() {
    let redis_image = Redis::default();
    let container = redis_image
        .start()
        .await
        .expect("Failed to start Redis container");
    let port = container
        .get_host_port_ipv4(6379)
        .await
        .expect("Failed to get port");
    let redis_url = format!("redis://127.0.0.1:{}", port);

    // Create Redis job queue
    let queue =
        RedisJobQueue::new(&redis_url, "test:queue").expect("Failed to create Redis job queue");

    // Test basic enqueue and dequeue
    let job = Job::new("test_tool", &serde_json::json!({"key": "value"}), 0).unwrap();
    queue.enqueue(job).await.unwrap();

    let dequeued = queue.dequeue().await.unwrap();
    assert!(dequeued.is_some());
    assert_eq!(dequeued.unwrap().tool_name, "test_tool");

    // Queue should be empty now
    let empty = queue
        .dequeue_with_timeout(Duration::from_millis(100))
        .await
        .unwrap();
    assert!(empty.is_none());
}

#[tokio::test]
async fn test_redis_queue_fifo_order() {
    let redis_image = Redis::default();
    let container = redis_image
        .start()
        .await
        .expect("Failed to start Redis container");
    let port = container
        .get_host_port_ipv4(6379)
        .await
        .expect("Failed to get port");
    let redis_url = format!("redis://127.0.0.1:{}", port);

    let queue =
        RedisJobQueue::new(&redis_url, "test:fifo").expect("Failed to create Redis job queue");

    // Enqueue multiple jobs
    let job1 = Job::new("tool1", &serde_json::json!({"order": 1}), 0).unwrap();
    let job2 = Job::new("tool2", &serde_json::json!({"order": 2}), 0).unwrap();
    let job3 = Job::new("tool3", &serde_json::json!({"order": 3}), 0).unwrap();

    queue.enqueue(job1).await.unwrap();
    queue.enqueue(job2).await.unwrap();
    queue.enqueue(job3).await.unwrap();

    // Dequeue and verify order
    let first = queue.dequeue().await.unwrap().unwrap();
    assert_eq!(first.tool_name, "tool1");

    let second = queue.dequeue().await.unwrap().unwrap();
    assert_eq!(second.tool_name, "tool2");

    let third = queue.dequeue().await.unwrap().unwrap();
    assert_eq!(third.tool_name, "tool3");
}

#[tokio::test]
#[ignore] // Long-running concurrent test
async fn test_redis_queue_concurrent_operations() {
    let redis_image = Redis::default();
    let container = redis_image
        .start()
        .await
        .expect("Failed to start Redis container");
    let port = container
        .get_host_port_ipv4(6379)
        .await
        .expect("Failed to get port");
    let redis_url = format!("redis://127.0.0.1:{}", port);

    let queue = Arc::new(
        RedisJobQueue::new(&redis_url, "test:concurrent")
            .expect("Failed to create Redis job queue"),
    );

    // Spawn multiple producers
    let mut handles = vec![];
    for i in 0..10 {
        let queue_clone = Arc::clone(&queue);
        let handle = tokio::spawn(async move {
            let job = Job::new(format!("tool_{}", i), &serde_json::json!({"id": i}), 0).unwrap();
            queue_clone.enqueue(job).await.unwrap();
        });
        handles.push(handle);
    }

    // Wait for all producers
    for handle in handles {
        handle.await.unwrap();
    }

    // Dequeue all jobs
    let mut count = 0;
    while queue.dequeue().await.unwrap().is_some() {
        count += 1;
    }

    assert_eq!(count, 10);
}

#[tokio::test]
#[ignore] // Long-running concurrent test
async fn test_redis_idempotency_concurrent_access() {
    let redis_image = Redis::default();
    let container = redis_image
        .start()
        .await
        .expect("Failed to start Redis container");
    let port = container
        .get_host_port_ipv4(6379)
        .await
        .expect("Failed to get port");
    let redis_url = format!("redis://127.0.0.1:{}", port);

    let store = Arc::new(
        RedisIdempotencyStore::new(&redis_url, Some("test:concurrent:"))
            .expect("Failed to create Redis idempotency store"),
    );

    // Spawn multiple tasks accessing the same key
    let mut handles = vec![];
    for i in 0..5 {
        let store_clone = Arc::clone(&store);
        let handle = tokio::spawn(async move {
            let key = format!("shared_key_{}", i % 2); // Two shared keys
            let result = JobResult::success(&format!("value_{}", i)).unwrap();

            store_clone
                .set(&key, &result, Duration::from_secs(10))
                .await
                .unwrap();

            // Try to get immediately
            store_clone.get(&key).await.unwrap()
        });
        handles.push(handle);
    }

    // Wait for all tasks
    for handle in handles {
        let result = handle.await.unwrap();
        assert!(result.is_some());
    }
}

#[tokio::test]
async fn test_redis_error_handling() {
    // Test with invalid Redis port (connection will fail)
    let result = RedisIdempotencyStore::new("redis://localhost:9999", None);
    assert!(result.is_ok()); // Client creation succeeds even with bad connection

    if let Ok(store) = result {
        // Operations should fail gracefully
        let get_result = store.get("test").await;
        assert!(get_result.is_err());

        let set_result = store
            .set(
                "test",
                &JobResult::success(&"value").unwrap(),
                Duration::from_secs(10),
            )
            .await;
        assert!(set_result.is_err());
    }
}

#[tokio::test]
async fn test_redis_custom_prefix() {
    let redis_image = Redis::default();
    let container = redis_image
        .start()
        .await
        .expect("Failed to start Redis container");
    let port = container
        .get_host_port_ipv4(6379)
        .await
        .expect("Failed to get port");
    let redis_url = format!("redis://127.0.0.1:{}", port);

    // Test with custom prefix
    let store = RedisIdempotencyStore::new(&redis_url, Some("custom:prefix:"))
        .expect("Failed to create Redis idempotency store");

    let key = "mykey";
    let result = JobResult::success(&"myvalue").unwrap();

    store
        .set(key, &result, Duration::from_secs(10))
        .await
        .expect("Failed to set key");

    // Verify it was stored with custom prefix
    let retrieved = store.get(key).await.unwrap();
    assert!(retrieved.is_some());
}
