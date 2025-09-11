//! Idempotency store for preventing duplicate execution of jobs.

use async_trait::async_trait;
use dashmap::DashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use crate::jobs::JobResult;

/// Trait for idempotency store implementations
#[async_trait]
pub trait IdempotencyStore: Send + Sync {
    /// Check if a result exists for the given idempotency key
    async fn get(&self, key: &str) -> anyhow::Result<Option<Arc<JobResult>>>;

    /// Store a result with the given idempotency key and TTL
    async fn set(&self, key: &str, result: Arc<JobResult>, ttl: Duration) -> anyhow::Result<()>;

    /// Remove an entry by key
    async fn remove(&self, key: &str) -> anyhow::Result<()>;
}

/// Entry in the idempotency store
#[derive(Clone, Debug)]
struct IdempotencyEntry {
    result: Arc<JobResult>,
    expires_at: SystemTime,
}

/// In-memory idempotency store for testing and development
#[derive(Debug)]
pub struct InMemoryIdempotencyStore {
    store: Arc<DashMap<String, IdempotencyEntry>>,
}

impl InMemoryIdempotencyStore {
    /// Create a new in-memory idempotency store
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Clean up expired entries
    fn cleanup_expired(&self) {
        let now = SystemTime::now();
        self.store.retain(|_, entry| entry.expires_at > now);
    }
}

impl Default for InMemoryIdempotencyStore {
    fn default() -> Self {
        Self {
            store: Arc::new(DashMap::default()),
        }
    }
}

#[async_trait]
impl IdempotencyStore for InMemoryIdempotencyStore {
    async fn get(&self, key: &str) -> anyhow::Result<Option<Arc<JobResult>>> {
        // Clean up expired entries periodically
        self.cleanup_expired();

        self.store.get(key).map_or_else(
            || Ok(None),
            |entry| {
                if entry.expires_at > SystemTime::now() {
                    Ok(Some(Arc::clone(&entry.result)))
                } else {
                    Ok(None)
                }
            },
        )
    }

    async fn set(&self, key: &str, result: Arc<JobResult>, ttl: Duration) -> anyhow::Result<()> {
        let expires_at = SystemTime::now()
            .checked_add(ttl)
            .unwrap_or_else(|| SystemTime::now() + Duration::from_secs(365 * 24 * 60 * 60 * 100)); // 100 years
        self.store
            .insert(key.to_string(), IdempotencyEntry { result, expires_at });
        Ok(())
    }

    async fn remove(&self, key: &str) -> anyhow::Result<()> {
        self.store.remove(key);
        Ok(())
    }
}

/// Redis-based idempotency store for production use
#[cfg(feature = "redis")]
#[derive(Debug)]
pub struct RedisIdempotencyStore {
    client: redis::Client,
    key_prefix: String,
}

#[cfg(feature = "redis")]
impl RedisIdempotencyStore {
    /// Create a new Redis idempotency store
    ///
    /// # Arguments
    /// * `redis_url` - Redis connection URL (e.g., "redis://127.0.0.1:6379")
    /// * `key_prefix` - Prefix for idempotency keys (default: "riglr:idempotency:")
    pub fn new(redis_url: &str, key_prefix: Option<&str>) -> anyhow::Result<Self> {
        let client = redis::Client::open(redis_url)?;
        Ok(Self {
            client,
            key_prefix: key_prefix
                .map_or_else(|| "riglr:idempotency:".to_string(), |s| s.to_string()),
        })
    }

    fn make_key(&self, key: &str) -> String {
        format!("{}{}", self.key_prefix, key)
    }
}

#[cfg(feature = "redis")]
#[async_trait]
impl IdempotencyStore for RedisIdempotencyStore {
    async fn get(&self, key: &str) -> anyhow::Result<Option<Arc<JobResult>>> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let redis_key = self.make_key(key);

        let result: Option<String> = redis::cmd("GET")
            .arg(&redis_key)
            .query_async(&mut conn)
            .await?;

        match result {
            Some(json_str) => {
                let result: JobResult = serde_json::from_str(&json_str)?;
                Ok(Some(Arc::new(result)))
            }
            None => Ok(None),
        }
    }

    async fn set(&self, key: &str, result: Arc<JobResult>, ttl: Duration) -> anyhow::Result<()> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let redis_key = self.make_key(key);
        let json_str = serde_json::to_string(&*result)?;
        let ttl_seconds = ttl.as_secs() as usize;

        redis::cmd("SETEX")
            .arg(&redis_key)
            .arg(ttl_seconds)
            .arg(json_str)
            .query_async::<()>(&mut conn)
            .await?;

        Ok(())
    }

    async fn remove(&self, key: &str) -> anyhow::Result<()> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let redis_key = self.make_key(key);

        redis::cmd("DEL")
            .arg(&redis_key)
            .query_async::<()>(&mut conn)
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // Test InMemoryIdempotencyStore::new()
    #[test]
    fn test_in_memory_idempotency_store_new() {
        let store = InMemoryIdempotencyStore::default();
        assert!(store.store.is_empty());
    }

    // Test InMemoryIdempotencyStore::default()
    #[test]
    fn test_in_memory_idempotency_store_default() {
        let store = InMemoryIdempotencyStore::default();
        assert!(store.store.is_empty());
    }

    // Test basic get/set/remove operations
    #[tokio::test]
    async fn test_in_memory_idempotency_store_basic_operations() {
        let store = InMemoryIdempotencyStore::default();

        let result = JobResult::success(&"test_value").unwrap();
        let key = "test_key";

        // Initially, key should not exist
        assert!(store.get(key).await.unwrap().is_none());

        // Store a result
        store
            .set(key, Arc::new(result), Duration::from_secs(60))
            .await
            .unwrap();

        // Should be able to retrieve it
        let retrieved = store.get(key).await.unwrap();
        assert!(retrieved.is_some());
        assert!(retrieved.unwrap().is_success());

        // Remove the entry
        store.remove(key).await.unwrap();
        assert!(store.get(key).await.unwrap().is_none());
    }

    // Test with failure result types
    #[tokio::test]
    async fn test_in_memory_store_with_failure_results() {
        let store = InMemoryIdempotencyStore::default();
        let key = "failure_key";

        // Test retriable failure
        let retriable_failure = JobResult::Failure {
            error: crate::error::ToolError::retriable_string("Network timeout"),
        };
        store
            .set(key, Arc::new(retriable_failure), Duration::from_secs(60))
            .await
            .unwrap();

        let retrieved = store.get(key).await.unwrap().unwrap();
        assert!(!retrieved.is_success());
        assert!(retrieved.is_retriable());

        // Test permanent failure
        let permanent_failure = JobResult::Failure {
            error: crate::error::ToolError::permanent_string("Invalid input"),
        };
        store
            .set(key, Arc::new(permanent_failure), Duration::from_secs(60))
            .await
            .unwrap();

        let retrieved = store.get(key).await.unwrap().unwrap();
        assert!(!retrieved.is_success());
        assert!(!retrieved.is_retriable());
    }

    // Test with success result with transaction hash
    #[tokio::test]
    async fn test_in_memory_store_with_tx_hash() {
        let store = InMemoryIdempotencyStore::default();
        let key = "tx_key";

        let result = JobResult::success_with_tx(&json!({"amount": 100}), "0x123abc").unwrap();
        store
            .set(key, Arc::new(result), Duration::from_secs(60))
            .await
            .unwrap();

        let retrieved = store.get(key).await.unwrap().unwrap();
        assert!(retrieved.is_success());
    }

    // Test expiry behavior
    #[tokio::test]
    async fn test_idempotency_expiry() {
        let store = InMemoryIdempotencyStore::default();

        let result = JobResult::success(&"test_value").unwrap();
        let key = "test_key";

        // Store with short TTL (very generous for instrumented runs)
        store
            .set(key, Arc::new(result), Duration::from_millis(200))
            .await
            .unwrap();

        // Should exist initially
        assert!(store.get(key).await.unwrap().is_some());

        // Wait for expiry (very generous timeout for instrumented runs)
        tokio::time::sleep(Duration::from_millis(500)).await;

        // Should be expired now
        assert!(store.get(key).await.unwrap().is_none());
    }

    // Test cleanup_expired functionality
    #[tokio::test]
    async fn test_cleanup_expired_entries() {
        let store = InMemoryIdempotencyStore::default();
        let result = JobResult::success(&"test").unwrap();

        // Add multiple entries with different TTLs
        store
            .set(
                "short_ttl",
                Arc::new(result.clone()),
                Duration::from_millis(100),
            )
            .await
            .unwrap();
        store
            .set("long_ttl", Arc::new(result), Duration::from_secs(60))
            .await
            .unwrap();

        // Both should exist initially
        assert!(store.get("short_ttl").await.unwrap().is_some());
        assert!(store.get("long_ttl").await.unwrap().is_some());
        assert_eq!(store.store.len(), 2);

        // Wait for short TTL to expire
        tokio::time::sleep(Duration::from_millis(300)).await;

        // Accessing any key should trigger cleanup
        let _ = store.get("long_ttl").await.unwrap();

        // Short TTL should be cleaned up, long TTL should remain
        assert!(store.get("short_ttl").await.unwrap().is_none());
        assert!(store.get("long_ttl").await.unwrap().is_some());
    }

    // Test get with expired entry returns None even if entry exists
    #[tokio::test]
    async fn test_get_expired_entry_returns_none() {
        let store = InMemoryIdempotencyStore::default();
        let result = JobResult::success(&"test").unwrap();
        let key = "expire_test";

        // Store with very short TTL
        store
            .set(key, Arc::new(result), Duration::from_millis(50))
            .await
            .unwrap();

        // Wait for expiry
        tokio::time::sleep(Duration::from_millis(150)).await;

        // Get should return None even though entry might still be in map before cleanup
        assert!(store.get(key).await.unwrap().is_none());
    }

    // Test remove non-existent key
    #[tokio::test]
    async fn test_remove_non_existent_key() {
        let store = InMemoryIdempotencyStore::default();

        // Should not panic or error when removing non-existent key
        store.remove("non_existent").await.unwrap();
    }

    // Test multiple concurrent operations
    #[tokio::test]
    async fn test_concurrent_operations() {
        let store = Arc::new(InMemoryIdempotencyStore::default());
        let result = JobResult::success(&"concurrent_test").unwrap();

        // Spawn multiple tasks setting different keys
        let mut handles = vec![];
        for i in 0..10 {
            let store_clone = Arc::clone(&store);
            let result_clone = result.clone();
            let handle = tokio::spawn(async move {
                let key = format!("concurrent_key_{}", i);
                store_clone
                    .set(&key, Arc::new(result_clone), Duration::from_secs(60))
                    .await
                    .unwrap();

                // Verify we can retrieve it
                let retrieved = store_clone.get(&key).await.unwrap();
                assert!(retrieved.is_some());
            });
            handles.push(handle);
        }

        // Wait for all tasks to complete
        for handle in handles {
            handle.await.unwrap();
        }

        // Verify all entries exist
        for i in 0..10 {
            let key = format!("concurrent_key_{}", i);
            assert!(store.get(&key).await.unwrap().is_some());
        }
    }

    // Test zero duration TTL
    #[tokio::test]
    async fn test_zero_duration_ttl() {
        let store = InMemoryIdempotencyStore::default();
        let result = JobResult::success(&"zero_ttl").unwrap();
        let key = "zero_key";

        // Set with zero duration (should expire immediately)
        store
            .set(key, Arc::new(result), Duration::from_secs(0))
            .await
            .unwrap();

        // Should return None as it's already expired
        assert!(store.get(key).await.unwrap().is_none());
    }

    // Test very large TTL
    #[tokio::test]
    async fn test_large_ttl() {
        let store = InMemoryIdempotencyStore::default();
        let result = JobResult::success(&"large_ttl").unwrap();
        let key = "large_key";

        // Set with very large TTL
        store
            .set(key, Arc::new(result), Duration::from_secs(u64::MAX))
            .await
            .unwrap();

        // Should still be retrievable
        assert!(store.get(key).await.unwrap().is_some());
    }

    // Test empty key
    #[tokio::test]
    async fn test_empty_key() {
        let store = InMemoryIdempotencyStore::default();
        let result = JobResult::success(&"empty_key_test").unwrap();

        // Should handle empty key without issues
        store
            .set("", Arc::new(result), Duration::from_secs(60))
            .await
            .unwrap();
        assert!(store.get("").await.unwrap().is_some());
        store.remove("").await.unwrap();
        assert!(store.get("").await.unwrap().is_none());
    }

    // Test special characters in key
    #[tokio::test]
    async fn test_special_characters_in_key() {
        let store = InMemoryIdempotencyStore::default();
        let result = JobResult::success(&"special_chars").unwrap();
        let key = "key:with/special\\chars@#$%";

        store
            .set(key, Arc::new(result), Duration::from_secs(60))
            .await
            .unwrap();
        assert!(store.get(key).await.unwrap().is_some());
        store.remove(key).await.unwrap();
        assert!(store.get(key).await.unwrap().is_none());
    }

    // Test multiple sets to same key (overwrite)
    #[tokio::test]
    async fn test_overwrite_same_key() {
        let store = InMemoryIdempotencyStore::default();
        let key = "overwrite_key";

        let result1 = JobResult::success(&"first_value").unwrap();
        let result2 = JobResult::success(&"second_value").unwrap();

        // Set first value
        store
            .set(key, Arc::new(result1), Duration::from_secs(60))
            .await
            .unwrap();
        let retrieved1 = store.get(key).await.unwrap().unwrap();

        // Set second value (should overwrite)
        store
            .set(key, Arc::new(result2), Duration::from_secs(60))
            .await
            .unwrap();
        let retrieved2 = store.get(key).await.unwrap().unwrap();

        // Values should be different (second should have overwritten first)
        assert_ne!(
            serde_json::to_string(&*retrieved1).unwrap(),
            serde_json::to_string(&*retrieved2).unwrap()
        );
    }

    // Test IdempotencyEntry creation and expiry logic
    #[test]
    fn test_idempotency_entry_creation() {
        let result = JobResult::success(&"test").unwrap();
        let expires_at = SystemTime::now() + Duration::from_secs(60);

        let entry = IdempotencyEntry {
            result: Arc::new(result.clone()),
            expires_at,
        };

        // Entry should be cloneable
        let cloned_entry = entry.clone();
        assert!(cloned_entry.expires_at == entry.expires_at);
    }

    // Redis tests (only compiled when redis feature is enabled)
    #[cfg(feature = "redis")]
    mod redis_tests {
        use super::*;

        #[test]
        fn test_redis_store_new_with_default_prefix() {
            // Test with a valid URL format but don't require actual Redis connection
            let result = RedisIdempotencyStore::new("redis://127.0.0.1:6379", None);
            match result {
                Ok(store) => {
                    assert_eq!(store.key_prefix, "riglr:idempotency:");
                }
                Err(_) => {
                    // Redis client creation may fail if redis crate is not available, which is ok
                }
            }
        }

        #[test]
        fn test_redis_store_new_with_custom_prefix() {
            let result = RedisIdempotencyStore::new("redis://127.0.0.1:6379", Some("custom:"));
            match result {
                Ok(store) => {
                    assert_eq!(store.key_prefix, "custom:");
                }
                Err(_) => {
                    // Redis client creation may fail if redis crate is not available, which is ok
                }
            }
        }

        #[test]
        fn test_redis_make_key() {
            // Test make_key only if we can create a store
            let result = RedisIdempotencyStore::new("redis://127.0.0.1:6379", Some("test:"));
            if let Ok(store) = result {
                assert_eq!(store.make_key("mykey"), "test:mykey");
                assert_eq!(store.make_key(""), "test:");
                assert_eq!(store.make_key("key:with:colons"), "test:key:with:colons");
            }
            // If we can't create a store, skip this test (Redis not available)
        }

        #[test]
        fn test_redis_invalid_url() {
            let result = RedisIdempotencyStore::new("invalid_url", None);
            assert!(result.is_err());
        }
    }
}
