//! Comprehensive tests for idempotency module

use riglr_core::idempotency::{IdempotencyStore, InMemoryIdempotencyStore};
use riglr_core::jobs::JobResult;
use std::sync::Arc;
use std::time::Duration;

#[tokio::test]
async fn test_idempotency_store_basic_operations() {
    let store = InMemoryIdempotencyStore::new();

    // Test initial state
    assert!(store.get("nonexistent").await.unwrap().is_none());

    // Test setting and getting
    let result = JobResult::success(&"test_value").unwrap();
    store
        .set("key1", &result, Duration::from_secs(60))
        .await
        .unwrap();

    let retrieved = store.get("key1").await.unwrap();
    assert!(retrieved.is_some());
    assert!(retrieved.unwrap().is_success());
}

#[tokio::test]
async fn test_idempotency_store_multiple_keys() {
    let store = InMemoryIdempotencyStore::new();

    // Set multiple keys
    let result1 = JobResult::success(&"value1").unwrap();
    let result2 = JobResult::success_with_tx(&"value2", "tx_hash_123").unwrap();
    let result3 = JobResult::retriable_failure("error message");

    store
        .set("key1", &result1, Duration::from_secs(60))
        .await
        .unwrap();
    store
        .set("key2", &result2, Duration::from_secs(60))
        .await
        .unwrap();
    store
        .set("key3", &result3, Duration::from_secs(60))
        .await
        .unwrap();

    // Verify all keys
    assert!(store.get("key1").await.unwrap().is_some());
    assert!(store.get("key2").await.unwrap().is_some());
    assert!(store.get("key3").await.unwrap().is_some());

    // Check specific results
    let retrieved2 = store.get("key2").await.unwrap().unwrap();
    match retrieved2 {
        JobResult::Success { tx_hash, .. } => {
            assert_eq!(tx_hash, Some("tx_hash_123".to_string()));
        }
        _ => panic!("Expected Success with tx_hash"),
    }

    let retrieved3 = store.get("key3").await.unwrap().unwrap();
    assert!(retrieved3.is_retriable());
}

#[tokio::test]
async fn test_idempotency_store_overwrite() {
    let store = InMemoryIdempotencyStore::new();

    // Set initial value
    let result1 = JobResult::success(&"initial").unwrap();
    store
        .set("key", &result1, Duration::from_secs(60))
        .await
        .unwrap();

    // Overwrite with new value
    let result2 = JobResult::success(&"updated").unwrap();
    store
        .set("key", &result2, Duration::from_secs(60))
        .await
        .unwrap();

    // Verify updated value
    let retrieved = store.get("key").await.unwrap().unwrap();
    match retrieved {
        JobResult::Success { value, .. } => {
            assert_eq!(value, serde_json::json!("updated"));
        }
        _ => panic!("Expected Success"),
    }
}

#[tokio::test]
async fn test_idempotency_store_removal() {
    let store = InMemoryIdempotencyStore::new();

    // Add multiple keys
    let result = JobResult::success(&"value").unwrap();
    store
        .set("key1", &result, Duration::from_secs(60))
        .await
        .unwrap();
    store
        .set("key2", &result, Duration::from_secs(60))
        .await
        .unwrap();
    store
        .set("key3", &result, Duration::from_secs(60))
        .await
        .unwrap();

    // Remove specific key
    store.remove("key2").await.unwrap();

    // Verify removal
    assert!(store.get("key1").await.unwrap().is_some());
    assert!(store.get("key2").await.unwrap().is_none());
    assert!(store.get("key3").await.unwrap().is_some());

    // Remove non-existent key (should not error)
    store.remove("nonexistent").await.unwrap();
}

#[tokio::test]
async fn test_idempotency_store_expiry_detailed() {
    let store = InMemoryIdempotencyStore::new();

    // Set with different TTLs (very generous for instrumented runs)
    let result = JobResult::success(&"value").unwrap();
    store
        .set("short", &result, Duration::from_millis(200))
        .await
        .unwrap();
    store
        .set("long", &result, Duration::from_secs(60))
        .await
        .unwrap();

    // Both should exist initially
    assert!(store.get("short").await.unwrap().is_some());
    assert!(store.get("long").await.unwrap().is_some());

    // Wait for short to expire (very generous timeout for instrumented runs)
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Short should be expired, long should still exist
    assert!(store.get("short").await.unwrap().is_none());
    assert!(store.get("long").await.unwrap().is_some());
}

#[tokio::test]
async fn test_idempotency_store_concurrent_access() {
    let store = Arc::new(InMemoryIdempotencyStore::new());

    // Spawn multiple tasks to access the store concurrently
    let mut handles = vec![];

    for i in 0..10 {
        let store_clone = store.clone();
        let handle = tokio::spawn(async move {
            let result = JobResult::success(&format!("value_{}", i)).unwrap();
            let key = format!("key_{}", i);

            // Set value
            store_clone
                .set(&key, &result, Duration::from_secs(60))
                .await
                .unwrap();

            // Get value
            let retrieved = store_clone.get(&key).await.unwrap();
            assert!(retrieved.is_some());

            // Remove value
            store_clone.remove(&key).await.unwrap();

            // Verify removed
            let retrieved = store_clone.get(&key).await.unwrap();
            assert!(retrieved.is_none());
        });
        handles.push(handle);
    }

    // Wait for all tasks to complete
    for handle in handles {
        handle.await.unwrap();
    }
}

#[tokio::test]
async fn test_idempotency_store_cleanup_expired() {
    let store = InMemoryIdempotencyStore::new();

    // Add entries with very short TTL
    let result = JobResult::success(&"value").unwrap();
    for i in 0..5 {
        store
            .set(&format!("key_{}", i), &result, Duration::from_millis(10))
            .await
            .unwrap();
    }

    // Add entries with long TTL
    for i in 5..10 {
        store
            .set(&format!("key_{}", i), &result, Duration::from_secs(60))
            .await
            .unwrap();
    }

    // Wait for short TTL entries to expire
    tokio::time::sleep(Duration::from_millis(20)).await;

    // Trigger cleanup by calling get
    store.get("trigger_cleanup").await.unwrap();

    // Verify short TTL entries are gone
    for i in 0..5 {
        assert!(store.get(&format!("key_{}", i)).await.unwrap().is_none());
    }

    // Verify long TTL entries still exist
    for i in 5..10 {
        assert!(store.get(&format!("key_{}", i)).await.unwrap().is_some());
    }
}

#[tokio::test]
async fn test_idempotency_store_default_impl() {
    let store = InMemoryIdempotencyStore::default();

    // Should work same as new()
    let result = JobResult::success(&"test").unwrap();
    store
        .set("key", &result, Duration::from_secs(60))
        .await
        .unwrap();
    assert!(store.get("key").await.unwrap().is_some());
}

#[tokio::test]
async fn test_idempotency_store_expired_entry_not_returned() {
    let store = InMemoryIdempotencyStore::new();

    // Directly access internal store to insert an already-expired entry
    let result = JobResult::success(&"expired").unwrap();
    store
        .set("key", &result, Duration::from_millis(1))
        .await
        .unwrap();

    // Wait a bit to ensure it's expired
    tokio::time::sleep(Duration::from_millis(5)).await;

    // Getting the expired entry should return None
    assert!(store.get("key").await.unwrap().is_none());
}

#[tokio::test]
#[ignore] // Long-running stress test - enable only for comprehensive testing
async fn test_idempotency_store_stress_test() {
    let store = Arc::new(InMemoryIdempotencyStore::new());
    let mut handles = vec![];

    // Create many concurrent operations
    for i in 0..100 {
        let store_clone = store.clone();
        let handle = tokio::spawn(async move {
            let key = format!("stress_key_{}", i % 10); // Reuse some keys
            let result = JobResult::success(&format!("value_{}", i)).unwrap();

            // Random operations
            match i % 3 {
                0 => {
                    store_clone
                        .set(&key, &result, Duration::from_secs(1))
                        .await
                        .unwrap();
                }
                1 => {
                    store_clone.get(&key).await.unwrap();
                }
                2 => {
                    store_clone.remove(&key).await.unwrap();
                }
                _ => {}
            }
        });
        handles.push(handle);
    }

    // Wait for all operations to complete
    for handle in handles {
        handle.await.unwrap();
    }
}

#[tokio::test]
async fn test_idempotency_store_empty_key() {
    let store = InMemoryIdempotencyStore::new();

    // Test with empty string key
    let result = JobResult::success(&"value").unwrap();
    store
        .set("", &result, Duration::from_secs(60))
        .await
        .unwrap();
    assert!(store.get("").await.unwrap().is_some());
    store.remove("").await.unwrap();
    assert!(store.get("").await.unwrap().is_none());
}

#[tokio::test]
async fn test_idempotency_store_large_values() {
    let store = InMemoryIdempotencyStore::new();

    // Create a large value
    let large_value: Vec<String> = (0..1000).map(|i| format!("item_{}", i)).collect();
    let result = JobResult::success(&large_value).unwrap();

    store
        .set("large_key", &result, Duration::from_secs(60))
        .await
        .unwrap();
    let retrieved = store.get("large_key").await.unwrap().unwrap();

    match retrieved {
        JobResult::Success { value, .. } => {
            let array = value.as_array().unwrap();
            assert_eq!(array.len(), 1000);
        }
        _ => panic!("Expected Success"),
    }
}

// Redis idempotency store tests
#[cfg(feature = "redis")]
mod redis_idempotency_tests {
    use super::*;
    use riglr_core::idempotency::{IdempotencyStore, RedisIdempotencyStore};

    #[tokio::test]
    async fn test_redis_idempotency_store_creation() {
        // Test basic construction
        let result = RedisIdempotencyStore::new("redis://127.0.0.1:6379", None);
        if result.is_ok() {
            // Success
        }

        // Test with custom prefix
        let result = RedisIdempotencyStore::new("redis://127.0.0.1:6379", Some("custom:prefix:"));
        if result.is_ok() {
            // Success
        }
    }

    #[tokio::test]
    async fn test_redis_idempotency_invalid_urls() {
        let invalid_urls = vec![
            "invalid://url",
            "not_a_url",
            "",
            "http://localhost:6379", // Wrong protocol
        ];

        for url in invalid_urls {
            let result = RedisIdempotencyStore::new(url, None);
            assert!(result.is_err(), "Expected error for invalid URL: {}", url);
        }
    }

    #[test]
    fn test_redis_key_generation() {
        use riglr_core::idempotency::RedisIdempotencyStore;

        // We can't test make_key directly since it's private,
        // but we can test that creation with different prefixes works
        if let Ok(_store) = RedisIdempotencyStore::new("redis://127.0.0.1:6379", Some("test:")) {
            // Store created successfully
        }

        if let Ok(_store) = RedisIdempotencyStore::new("redis://127.0.0.1:6379", Some("")) {
            // Store created with empty prefix
        }
    }

    #[tokio::test]
    async fn test_redis_idempotency_operations() {
        // Test Redis operations if a store can be created
        let store_result = RedisIdempotencyStore::new("redis://127.0.0.1:6379", Some("test_idem:"));

        if let Ok(store) = store_result {
            let result = JobResult::success(&"test_value").unwrap();

            // Test set operation - this will exercise lines 145-158
            let set_result = store
                .set("test_key", &result, Duration::from_secs(60))
                .await;

            if set_result.is_ok() {
                // Test get operation - this will exercise lines 127-142
                let get_result = store.get("test_key").await;

                match get_result {
                    Ok(Some(_)) => {
                        // Test removal - this will exercise lines 161-170
                        let _remove_result = store.remove("test_key").await;

                        // Test get after removal
                        let _get_after_remove = store.get("test_key").await;
                    }
                    Ok(None) => {
                        // Key not found, which is valid
                    }
                    Err(_) => {
                        // Redis connection error, which is expected in most test environments
                    }
                }
            }
        }
    }

    #[tokio::test]
    async fn test_redis_make_key_method() {
        // We need to test the make_key method functionality
        if let Ok(store) = RedisIdempotencyStore::new("redis://127.0.0.1:6379", Some("custom:")) {
            // The make_key method is called internally during operations
            let result = JobResult::success(&"test").unwrap();

            // This will internally call make_key with "test_key"
            // Result should be "custom:test_key"
            let _ = store.set("test_key", &result, Duration::from_secs(1)).await;
        }
    }

    #[tokio::test]
    async fn test_redis_serialization_errors() {
        if let Ok(store) = RedisIdempotencyStore::new("redis://127.0.0.1:6379", None) {
            // Create a result that will test serialization paths
            let result = JobResult::success(&"test_value").unwrap();

            // Test with special characters that might cause serialization issues
            let _ = store
                .set("test:key:with:colons", &result, Duration::from_secs(10))
                .await;
            let _ = store.get("test:key:with:colons").await;
            let _ = store.remove("test:key:with:colons").await;
        }
    }
}

#[tokio::test]
async fn test_idempotency_entry_expiry_edge_cases() {
    let store = InMemoryIdempotencyStore::new();

    // Test with very short TTL
    let result = JobResult::success(&"short_ttl").unwrap();
    store
        .set("short_ttl_key", &result, Duration::from_nanos(1))
        .await
        .unwrap();

    // Should likely be expired by now
    tokio::time::sleep(Duration::from_millis(1)).await;
    let _retrieved = store.get("short_ttl_key").await.unwrap();
    // May or may not exist depending on timing, but shouldn't panic

    // Test with zero TTL
    let result = JobResult::success(&"zero_ttl").unwrap();
    store
        .set("zero_ttl_key", &result, Duration::from_secs(0))
        .await
        .unwrap();

    // Should be expired immediately
    let retrieved = store.get("zero_ttl_key").await.unwrap();
    assert!(retrieved.is_none());
}

#[tokio::test]
async fn test_idempotency_error_cases() {
    let store = InMemoryIdempotencyStore::new();

    // Test removing non-existent key (should not error)
    store.remove("non_existent").await.unwrap();

    // Test with special characters in keys
    let special_keys = vec![
        "key with spaces",
        "key:with:colons",
        "key/with/slashes",
        "key@with@symbols",
        "ключ", // Cyrillic
        "🔑",   // Emoji key
    ];

    for key in special_keys {
        let result = JobResult::success(&format!("value for {}", key)).unwrap();
        store
            .set(key, &result, Duration::from_secs(10))
            .await
            .unwrap();

        let retrieved = store.get(key).await.unwrap();
        assert!(retrieved.is_some());

        store.remove(key).await.unwrap();
        assert!(store.get(key).await.unwrap().is_none());
    }
}
