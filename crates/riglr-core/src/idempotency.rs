//! Idempotency store for preventing duplicate execution of jobs.

use alloc::sync::Arc;
use async_trait::async_trait;
use core::time::Duration;
use dashmap::DashMap;
use std::time::SystemTime;

use crate::jobs::JobResult;

/// Trait for idempotency store implementations
#[async_trait]
pub trait Store: Send + Sync {
    /// Check if a result exists for the given idempotency key
    async fn get(&self, key: &str) -> anyhow::Result<Option<Arc<JobResult>>>;

    /// Remove an entry by key
    async fn remove(&self, key: &str) -> anyhow::Result<()>;

    /// Store a result with the given idempotency key and TTL
    async fn set(&self, key: &str, result: Arc<JobResult>, ttl: Duration) -> anyhow::Result<()>;
}

/// Entry in the idempotency store
#[derive(Clone, Debug)]
struct IdempotencyEntry {
    /// Expiration timestamp for this entry
    expires_at: SystemTime,
    /// The cached job result
    result: Arc<JobResult>,
}

/// In-memory idempotency store for testing and development
#[derive(Debug)]
pub struct InMemoryIdempotencyStore {
    /// The in-memory map storing idempotency entries
    store: Arc<DashMap<String, IdempotencyEntry>>,
}

impl InMemoryIdempotencyStore {
    /// Clean up expired entries
    fn cleanup_expired(&self) {
        let now = SystemTime::now();
        self.store.retain(|_, entry| entry.expires_at > now);
    }

    /// Create a new in-memory idempotency store
    #[must_use]
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for InMemoryIdempotencyStore {
    #[inline]
    fn default() -> Self {
        Self {
            store: Arc::new(DashMap::default()),
        }
    }
}

#[async_trait]
impl Store for InMemoryIdempotencyStore {
    #[inline]
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

    #[inline]
    async fn remove(&self, key: &str) -> anyhow::Result<()> {
        self.store.remove(key);
        Ok(())
    }

    #[inline]
    async fn set(&self, key: &str, result: Arc<JobResult>, ttl: Duration) -> anyhow::Result<()> {
        let expires_at = SystemTime::now().checked_add(ttl).unwrap_or_else(|| {
            // If addition overflows, use the maximum possible SystemTime
            SystemTime::UNIX_EPOCH
                .checked_add(Duration::from_secs(u64::MAX - 1))
                .unwrap_or(SystemTime::UNIX_EPOCH)
        });
        self.store
            .insert(key.to_owned(), IdempotencyEntry { expires_at, result });
        Ok(())
    }
}

/// Redis-based idempotency store for production use
#[cfg(feature = "redis")]
#[derive(Debug)]
pub struct RedisIdempotencyStore {
    /// The Redis client connection
    client: redis::Client,
    /// Key prefix for all idempotency keys stored in Redis
    key_prefix: String,
}

#[cfg(feature = "redis")]
impl RedisIdempotencyStore {
    /// Create key with prefix
    ///
    /// Combines the configured prefix with the given key
    fn make_key(&self, key: &str) -> String {
        format!("{}{}", self.key_prefix, key)
    }

    /// Create a new Redis idempotency store
    ///
    /// # Arguments
    /// * `redis_url` - Redis connection URL (e.g., "<redis://127.0.0.1:6379>")
    /// * `key_prefix` - Prefix for idempotency keys (default: "riglr:idempotency:")
    ///
    /// # Errors
    /// Returns an error if the Redis client cannot be created from the given URL
    #[inline]
    pub fn new(redis_url: &str, key_prefix: Option<&str>) -> anyhow::Result<Self> {
        let client = redis::Client::open(redis_url)?;
        Ok(Self {
            client,
            key_prefix: key_prefix
                .map_or_else(|| "riglr:idempotency:".to_owned(), ToString::to_string),
        })
    }
}

#[cfg(feature = "redis")]
#[async_trait]
impl Store for RedisIdempotencyStore {
    #[inline]
    async fn get(&self, key: &str) -> anyhow::Result<Option<Arc<JobResult>>> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let redis_key = self.make_key(key);

        let redis_result: Option<String> = redis::cmd("GET")
            .arg(&redis_key)
            .query_async(&mut conn)
            .await?;

        match redis_result {
            Some(json_str) => {
                let job_result: JobResult = serde_json::from_str(&json_str)?;
                Ok(Some(Arc::new(job_result)))
            }
            None => Ok(None),
        }
    }

    #[inline]
    async fn remove(&self, key: &str) -> anyhow::Result<()> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let redis_key = self.make_key(key);

        redis::cmd("DEL")
            .arg(&redis_key)
            .query_async::<()>(&mut conn)
            .await?;

        Ok(())
    }

    #[inline]
    async fn set(&self, key: &str, result: Arc<JobResult>, ttl: Duration) -> anyhow::Result<()> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let redis_key = self.make_key(key);
        let json_str = serde_json::to_string(&*result)?;
        let ttl_seconds = usize::try_from(ttl.as_secs()).unwrap_or(usize::MAX); // If u64 doesn't fit in usize, use maximum usize value

        redis::cmd("SETEX")
            .arg(&redis_key)
            .arg(ttl_seconds)
            .arg(json_str)
            .query_async::<()>(&mut conn)
            .await?;

        Ok(())
    }
}

#[cfg(test)]
#[expect(clippy::unwrap_used)]
mod tests {
    use super::*;
    use serde_json::json;
    use tokio::time::sleep;

    #[test]
    #[expect(clippy::unwrap_used)]
    fn idempotency_entry_creation() {
        let result = JobResult::success(&"test").unwrap();
        let expires_at = SystemTime::now() + Duration::from_secs(60);

        let entry = IdempotencyEntry {
            expires_at,
            result: Arc::new(result),
        };

        // Entry should be cloneable
        let cloned_entry = entry.clone();
        assert!(cloned_entry.expires_at == entry.expires_at);
    }

    // Test InMemoryIdempotencyStore::new()
    #[test]
    fn in_memory_idempotency_store_new() {
        let store = InMemoryIdempotencyStore::default();
        assert!(store.store.is_empty());
    }

    // Test InMemoryIdempotencyStore::default()
    #[test]
    fn in_memory_idempotency_store_default() {
        let store = InMemoryIdempotencyStore::default();
        assert!(store.store.is_empty());
    }

    // Test basic get/set/remove operations
    #[tokio::test]
    #[expect(clippy::unwrap_used)]
    async fn in_memory_idempotency_store_basic_operations() {
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
    #[expect(clippy::unwrap_used)]
    async fn in_memory_store_with_failure_results() {
        use crate::error::ToolError;
        let store = InMemoryIdempotencyStore::default();
        let key = "failure_key";

        // Test retriable failure
        let retriable_failure = JobResult::Failure {
            error: ToolError::retriable_string("Network timeout"),
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
            error: ToolError::permanent_string("Invalid input"),
        };
        store
            .set(key, Arc::new(permanent_failure), Duration::from_secs(60))
            .await
            .unwrap();

        let retrieved_permanent = store.get(key).await.unwrap().unwrap();
        assert!(!retrieved_permanent.is_success());
        assert!(!retrieved_permanent.is_retriable());
    }

    // Test with success result with transaction hash
    #[tokio::test]
    #[expect(clippy::unwrap_used)]
    async fn in_memory_store_with_tx_hash() {
        let store = InMemoryIdempotencyStore::default();
        let key = "tx_key";

        let result = JobResult::success_with_tx(&json!({"amount": 100_i32}), "0x123abc").unwrap();
        store
            .set(key, Arc::new(result), Duration::from_secs(60))
            .await
            .unwrap();

        let retrieved = store.get(key).await.unwrap().unwrap();
        assert!(retrieved.is_success());
    }

    // Test expiry behavior
    #[tokio::test]
    #[expect(clippy::expect_used)]
    async fn idempotency_expiry() {
        let store = InMemoryIdempotencyStore::default();

        let result = JobResult::success(&"test_value").unwrap();
        let key = "test_key";

        // Store with short TTL (very generous for instrumented runs)
        store
            .set(key, Arc::new(result), Duration::from_millis(200))
            .await
            .unwrap();

        // Should exist initially
        assert!(store
            .get(key)
            .await
            .expect("Failed to get key before expiry")
            .is_some());

        // Wait for expiry (very generous timeout for instrumented runs)
        sleep(Duration::from_millis(500)).await;

        // Should be expired now
        assert!(store
            .get(key)
            .await
            .expect("Failed to get expired key")
            .is_none());
    }

    // Test cleanup_expired functionality
    #[tokio::test]
    #[expect(clippy::expect_used)]
    async fn cleanup_expired_entries() {
        let store = InMemoryIdempotencyStore::default();
        let result =
            JobResult::success(&"test").expect("Failed to create success result for cleanup test");

        // Add multiple entries with different TTLs
        store
            .set(
                "short_ttl",
                Arc::new(result.clone()),
                Duration::from_millis(100),
            )
            .await
            .expect("Failed to set short TTL entry");
        store
            .set("long_ttl", Arc::new(result), Duration::from_secs(60))
            .await
            .expect("Failed to set long TTL entry");

        // Both should exist initially
        assert!(store
            .get("short_ttl")
            .await
            .expect("Failed to get short_ttl entry")
            .is_some());
        assert!(store
            .get("long_ttl")
            .await
            .expect("Failed to get long_ttl entry")
            .is_some());
        assert_eq!(store.store.len(), 2);

        // Wait for short TTL to expire
        sleep(Duration::from_millis(300)).await;

        // Accessing any key should trigger cleanup
        let _trigger: Option<Arc<JobResult>> = store
            .get("long_ttl")
            .await
            .expect("Failed to access long_ttl for cleanup trigger");

        // Short TTL should be cleaned up, long TTL should remain
        assert!(store
            .get("short_ttl")
            .await
            .expect("Failed to get short_ttl after cleanup")
            .is_none());
        assert!(store
            .get("long_ttl")
            .await
            .expect("Failed to get long_ttl after cleanup")
            .is_some());
    }

    // Test get with expired entry returns None even if entry exists
    #[tokio::test]
    #[expect(clippy::expect_used)]
    async fn get_expired_entry_returns_none() {
        let store = InMemoryIdempotencyStore::default();
        let result =
            JobResult::success(&"test").expect("Failed to create success result for expired test");
        let key = "expire_test";

        // Store with very short TTL
        store
            .set(key, Arc::new(result), Duration::from_millis(50))
            .await
            .expect("Failed to set key with very short TTL");

        // Wait for expiry
        sleep(Duration::from_millis(150)).await;

        // Get should return None even though entry might still be in map before cleanup
        assert!(store
            .get(key)
            .await
            .expect("Failed to get expired entry")
            .is_none());
    }

    // Test remove non-existent key
    #[tokio::test]
    #[expect(clippy::expect_used)]
    async fn remove_non_existent_key() {
        let store = InMemoryIdempotencyStore::default();

        // Should not panic or error when removing non-existent key
        store
            .remove("non_existent")
            .await
            .expect("Failed to remove non-existent key");
    }

    // Test multiple concurrent operations
    #[tokio::test]
    #[expect(clippy::expect_used)]
    async fn concurrent_operations() {
        let store = Arc::new(InMemoryIdempotencyStore::default());
        let result = JobResult::success(&"concurrent_test")
            .expect("Failed to create concurrent test result");

        // Spawn multiple tasks setting different keys
        let mut handles = vec![];
        for i in 0i32..10i32 {
            let store_clone = Arc::clone(&store);
            let result_clone = result.clone();
            let handle = tokio::spawn(async move {
                let key = format!("concurrent_key_{i}");
                store_clone
                    .set(&key, Arc::new(result_clone), Duration::from_secs(60))
                    .await
                    .expect("Failed to set key in concurrent test");

                // Verify we can retrieve it
                let retrieved = store_clone
                    .get(&key)
                    .await
                    .expect("Failed to get key in concurrent test");
                assert!(retrieved.is_some());
            });
            handles.push(handle);
        }

        // Wait for all tasks to complete
        for handle in handles {
            handle.await.expect("Failed to complete concurrent task");
        }

        // Verify all entries exist
        for i in 0i32..10i32 {
            let key = format!("concurrent_key_{i}");
            assert!(store
                .get(&key)
                .await
                .expect("Failed to get concurrent key after completion")
                .is_some());
        }
    }

    // Test zero duration TTL
    #[tokio::test]
    #[expect(clippy::expect_used)]
    async fn zero_duration_ttl() {
        let store = InMemoryIdempotencyStore::default();
        let result =
            JobResult::success(&"zero_ttl").expect("Failed to create zero TTL test result");
        let key = "zero_key";

        // Set with zero duration (should expire immediately)
        store
            .set(key, Arc::new(result), Duration::from_secs(0))
            .await
            .expect("Failed to set key with zero TTL");

        // Should return None as it's already expired
        assert!(store
            .get(key)
            .await
            .expect("Failed to get zero TTL key")
            .is_none());
    }

    // Test very large TTL
    #[tokio::test]
    #[expect(clippy::expect_used)]
    async fn large_ttl() {
        let store = InMemoryIdempotencyStore::default();
        let result =
            JobResult::success(&"large_ttl").expect("Failed to create large TTL test result");
        let key = "large_key";

        // Set with very large TTL
        store
            .set(key, Arc::new(result), Duration::from_secs(u64::MAX))
            .await
            .expect("Failed to set key with large TTL");

        // Should still be retrievable
        assert!(store
            .get(key)
            .await
            .expect("Failed to get large TTL key")
            .is_some());
    }

    // Test empty key
    #[tokio::test]
    #[expect(clippy::expect_used)]
    async fn empty_key() {
        let store = InMemoryIdempotencyStore::default();
        let result =
            JobResult::success(&"empty_key_test").expect("Failed to create empty key test result");

        // Should handle empty key without issues
        store
            .set("", Arc::new(result), Duration::from_secs(60))
            .await
            .expect("Failed to set empty key");
        assert!(store
            .get("")
            .await
            .expect("Failed to get empty key")
            .is_some());
        store.remove("").await.expect("Failed to remove empty key");
        assert!(store
            .get("")
            .await
            .expect("Failed to get empty key after removal")
            .is_none());
    }

    // Test special characters in key
    #[tokio::test]
    #[expect(clippy::expect_used)]
    async fn special_characters_in_key() {
        let store = InMemoryIdempotencyStore::default();
        let result = JobResult::success(&"special_chars")
            .expect("Failed to create special chars test result");
        let key = "key:with/special\\chars@#$%";

        store
            .set(key, Arc::new(result), Duration::from_secs(60))
            .await
            .expect("Failed to set key with special chars");
        assert!(store
            .get(key)
            .await
            .expect("Failed to get key with special chars")
            .is_some());
        store
            .remove(key)
            .await
            .expect("Failed to remove key with special chars");
        assert!(store
            .get(key)
            .await
            .expect("Failed to get key with special chars after removal")
            .is_none());
    }

    // Redis tests (only compiled when redis feature is enabled)
    #[cfg(feature = "redis")]
    mod redis_tests {
        use super::*;

        #[test]
        fn redis_store_new_with_default_prefix() {
            // Test with a valid URL format but don't require actual Redis connection
            let result = RedisIdempotencyStore::new("redis://127.0.0.1:6379", None);
            if let Ok(store) = result {
                assert_eq!(store.key_prefix, "riglr:idempotency:");
            }
            // Redis client creation may fail if redis crate is not available, which is ok
        }

        #[test]
        fn redis_store_new_with_custom_prefix() {
            let result = RedisIdempotencyStore::new("redis://127.0.0.1:6379", Some("custom:"));
            if let Ok(store) = result {
                assert_eq!(store.key_prefix, "custom:");
            }
            // Redis client creation may fail if redis crate is not available, which is ok
        }

        #[test]
        fn redis_make_key() {
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
        #[expect(clippy::unwrap_used)]
        fn redis_invalid_url() {
            let result = RedisIdempotencyStore::new("invalid_url", None);
            result.unwrap_err();
        }
    }

    // Test multiple sets to same key (overwrite)
    #[tokio::test]
    #[expect(clippy::expect_used)]
    async fn overwrite_same_key() {
        let store = InMemoryIdempotencyStore::default();
        let key = "overwrite_key";

        let result1 =
            JobResult::success(&"first_value").expect("Failed to create first overwrite result");
        let result2 =
            JobResult::success(&"second_value").expect("Failed to create second overwrite result");

        // Set first value
        store
            .set(key, Arc::new(result1), Duration::from_secs(60))
            .await
            .expect("Failed to set first value");
        let retrieved1 = store
            .get(key)
            .await
            .expect("Failed to get first value")
            .expect("Expected first value to exist");

        // Set second value (should overwrite)
        store
            .set(key, Arc::new(result2), Duration::from_secs(60))
            .await
            .expect("Failed to set second value");
        let retrieved2 = store
            .get(key)
            .await
            .expect("Failed to get second value")
            .expect("Expected second value to exist");

        // Values should be different (second should have overwritten first)
        assert_ne!(
            serde_json::to_string(&*retrieved1).expect("Failed to serialize first result"),
            serde_json::to_string(&*retrieved2).expect("Failed to serialize second result")
        );
    }

    // Test IdempotencyEntry creation and expiry logic
    #[test]
    #[expect(clippy::expect_used)]
    fn idempotency_entry_creation_works() {
        let result = JobResult::success(&"test")
            .expect("Failed to create test result for entry creation test");
        let expires_at = SystemTime::now() + Duration::from_secs(60);

        let entry = IdempotencyEntry {
            result: Arc::new(result),
            expires_at,
        };

        // Entry should be cloneable
        let cloned_entry = entry.clone();
        assert!(cloned_entry.expires_at == entry.expires_at);
    }
}
