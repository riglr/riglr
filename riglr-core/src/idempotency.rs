//! Idempotency store for preventing duplicate execution of jobs.

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;

use crate::jobs::JobResult;

/// Trait for idempotency store implementations
#[async_trait]
pub trait IdempotencyStore: Send + Sync {
    /// Check if a result exists for the given idempotency key
    async fn get(&self, key: &str) -> anyhow::Result<Option<JobResult>>;

    /// Store a result with the given idempotency key and TTL
    async fn set(&self, key: &str, result: &JobResult, ttl: Duration) -> anyhow::Result<()>;

    /// Remove an entry by key
    async fn remove(&self, key: &str) -> anyhow::Result<()>;
}

/// Entry in the idempotency store
#[derive(Clone)]
struct IdempotencyEntry {
    result: JobResult,
    expires_at: SystemTime,
}

/// In-memory idempotency store for testing and development
pub struct InMemoryIdempotencyStore {
    store: Arc<RwLock<HashMap<String, IdempotencyEntry>>>,
}

impl InMemoryIdempotencyStore {
    /// Create a new in-memory idempotency store
    #[must_use]
    pub fn new() -> Self {
        Self {
            store: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Clean up expired entries
    async fn cleanup_expired(&self) {
        let now = SystemTime::now();
        let mut store = self.store.write().await;
        store.retain(|_, entry| entry.expires_at > now);
    }
}

impl Default for InMemoryIdempotencyStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl IdempotencyStore for InMemoryIdempotencyStore {
    async fn get(&self, key: &str) -> anyhow::Result<Option<JobResult>> {
        // Clean up expired entries periodically
        self.cleanup_expired().await;

        let store = self.store.read().await;
        store.get(key).map_or_else(
            || Ok(None),
            |entry| {
                if entry.expires_at > SystemTime::now() {
                    Ok(Some(entry.result.clone()))
                } else {
                    Ok(None)
                }
            },
        )
    }

    async fn set(&self, key: &str, result: &JobResult, ttl: Duration) -> anyhow::Result<()> {
        let expires_at = SystemTime::now() + ttl;
        self.store.write().await.insert(
            key.to_string(),
            IdempotencyEntry {
                result: result.clone(),
                expires_at,
            },
        );
        Ok(())
    }

    async fn remove(&self, key: &str) -> anyhow::Result<()> {
        self.store.write().await.remove(key);
        Ok(())
    }
}

/// Redis-based idempotency store for production use
#[cfg(feature = "redis")]
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
            key_prefix: key_prefix.unwrap_or("riglr:idempotency:").to_string(),
        })
    }

    fn make_key(&self, key: &str) -> String {
        format!("{}{}", self.key_prefix, key)
    }
}

#[cfg(feature = "redis")]
#[async_trait]
impl IdempotencyStore for RedisIdempotencyStore {
    async fn get(&self, key: &str) -> anyhow::Result<Option<JobResult>> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let redis_key = self.make_key(key);

        let result: Option<String> = redis::cmd("GET")
            .arg(&redis_key)
            .query_async(&mut conn)
            .await?;

        match result {
            Some(json_str) => {
                let result: JobResult = serde_json::from_str(&json_str)?;
                Ok(Some(result))
            }
            None => Ok(None),
        }
    }

    async fn set(&self, key: &str, result: &JobResult, ttl: Duration) -> anyhow::Result<()> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let redis_key = self.make_key(key);
        let json_str = serde_json::to_string(result)?;
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

    #[tokio::test]
    async fn test_in_memory_idempotency_store() {
        let store = InMemoryIdempotencyStore::new();

        let result = JobResult::success(&"test_value").unwrap();
        let key = "test_key";

        // Initially, key should not exist
        assert!(store.get(key).await.unwrap().is_none());

        // Store a result
        store
            .set(key, &result, Duration::from_secs(60))
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

    #[tokio::test]
    async fn test_idempotency_expiry() {
        let store = InMemoryIdempotencyStore::new();

        let result = JobResult::success(&"test_value").unwrap();
        let key = "test_key";

        // Store with short TTL (very generous for instrumented runs)
        store
            .set(key, &result, Duration::from_millis(200))
            .await
            .unwrap();

        // Should exist initially
        assert!(store.get(key).await.unwrap().is_some());

        // Wait for expiry (very generous timeout for instrumented runs)
        tokio::time::sleep(Duration::from_millis(500)).await;

        // Should be expired now
        assert!(store.get(key).await.unwrap().is_none());
    }
}
