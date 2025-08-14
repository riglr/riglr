//! Caching layer for the indexer storage

use std::time::Duration;
use async_trait::async_trait;

use crate::error::IndexerResult;
use crate::storage::StoredEvent;

/// Cache trait for storing and retrieving events
#[async_trait]
pub trait Cache: Send + Sync {
    /// Store an event in cache
    async fn set(&self, key: &str, event: &StoredEvent, ttl: Option<Duration>) -> IndexerResult<()>;

    /// Retrieve an event from cache
    async fn get(&self, key: &str) -> IndexerResult<Option<StoredEvent>>;

    /// Delete an event from cache
    async fn delete(&self, key: &str) -> IndexerResult<()>;

    /// Check if cache is healthy
    async fn health_check(&self) -> IndexerResult<()>;

    /// Clear all cached data
    async fn clear(&self) -> IndexerResult<()>;
}

/// Redis-based cache implementation
pub struct RedisCache {
    // Implementation would go here
}

/// In-memory cache implementation
pub struct MemoryCache {
    // Implementation would go here
}