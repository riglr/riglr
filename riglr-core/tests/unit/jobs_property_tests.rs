//! Property-based tests for jobs module using proptest
//!
//! These tests use property-based testing to verify that job creation,
//! serialization, and behavior work correctly with a wide range of inputs.

use riglr_core::jobs::{Job, JobResult};
use proptest::prelude::*;
use serde_json::{json, Value};
use std::collections::HashMap;

// Strategies for generating test data

/// Strategy for generating valid tool names
fn tool_name_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        // Simple names
        "[a-zA-Z][a-zA-Z0-9_]{0,30}",
        // Names with prefixes (for resource pool testing)
        "solana_[a-zA-Z][a-zA-Z0-9_]{0,20}",
        "evm_[a-zA-Z][a-zA-Z0-9_]{0,20}",
        "web_[a-zA-Z][a-zA-Z0-9_]{0,20}",
        // Special characters
        "[a-zA-Z][a-zA-Z0-9_.-]{0,30}",
        // Unicode names
        Just("测试工具".to_string()),
        Just("инструмент".to_string()),
        Just("outil_français".to_string()),
    ]
}

/// Strategy for generating JSON parameters
fn params_strategy() -> impl Strategy<Value = Value> {
    let leaf = prop_oneof![
        Just(Value::Null),
        any::<bool>().prop_map(Value::Bool),
        any::<i64>().prop_map(|i| Value::Number(i.into())),
        any::<f64>().prop_filter("must be finite", |f| f.is_finite())
            .prop_map(|f| json!(f)),
        "[a-zA-Z0-9_ ]{0,100}".prop_map(Value::String),
    ];

    leaf.prop_recursive(
        2, // max depth
        64, // max size
        8, // items per collection
        |inner| {
            prop_oneof![
                prop::collection::vec(inner.clone(), 0..8).prop_map(Value::Array),
                prop::collection::hash_map("[a-zA-Z][a-zA-Z0-9_]{0,20}", inner, 0..8)
                    .prop_map(|map| {
                        Value::Object(map.into_iter().collect())
                    }),
            ]
        },
    )
}

/// Strategy for generating retry counts
fn retry_count_strategy() -> impl Strategy<Value = u32> {
    prop_oneof![
        Just(0u32),
        Just(1u32),
        1u32..10u32,
        10u32..100u32,
        // Test some edge cases
        Just(u32::MAX - 1),
        Just(u32::MAX),
    ]
}

/// Strategy for generating idempotency keys
fn idempotency_key_strategy() -> impl Strategy<Value = Option<String>> {
    prop_oneof![
        Just(None),
        "[a-zA-Z0-9_-]{1,100}".prop_map(Some),
        Just(Some("".to_string())), // Empty key
        Just(Some("a".repeat(1000))), // Very long key
        // UUID-like keys
        "[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}".prop_map(Some),
    ]
}

/// Strategy for generating error messages
fn error_message_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("".to_string()),
        "[a-zA-Z0-9 ]{1,50}",
        Just("Network connection failed".to_string()),
        Just("Invalid parameters provided".to_string()),
        Just("Rate limit exceeded".to_string()),
        Just("Internal server error".to_string()),
        // Very long error message
        Just("x".repeat(10000)),
        // Unicode error messages
        Just("错误信息".to_string()),
        Just("Erreur système".to_string()),
    ]
}

proptest! {
    #[test]
    fn test_job_creation_with_arbitrary_inputs(
        tool_name in tool_name_strategy(),
        params in params_strategy(),
        max_retries in retry_count_strategy(),
    ) {
        // Job creation should always succeed with valid inputs
        let result = Job::new(&tool_name, &params, max_retries);
        prop_assert!(result.is_ok());

        if let Ok(job) = result {
            prop_assert_eq!(job.tool_name, tool_name);
            prop_assert_eq!(job.params, params);
            prop_assert_eq!(job.max_retries, max_retries);
            prop_assert_eq!(job.retry_count, 0);
            prop_assert!(job.idempotency_key.is_none());
            prop_assert!(job.can_retry() == (max_retries > 0));
        }
    }

    #[test]
    fn test_idempotent_job_creation_with_arbitrary_inputs(
        tool_name in tool_name_strategy(),
        params in params_strategy(),
        max_retries in retry_count_strategy(),
        idempotency_key in "[a-zA-Z0-9_-]{1,100}",
    ) {
        let result = Job::new_idempotent(&tool_name, &params, max_retries, &idempotency_key);
        prop_assert!(result.is_ok());

        if let Ok(job) = result {
            prop_assert_eq!(job.tool_name, tool_name);
            prop_assert_eq!(job.params, params);
            prop_assert_eq!(job.max_retries, max_retries);
            prop_assert_eq!(job.retry_count, 0);
            prop_assert_eq!(job.idempotency_key, Some(idempotency_key));
            prop_assert!(job.can_retry() == (max_retries > 0));
        }
    }

    #[test]
    fn test_job_retry_logic_properties(
        max_retries in retry_count_strategy(),
        increment_count in 0u32..200u32,
    ) {
        let job_result = Job::new("test_tool", &json!({}), max_retries);
        prop_assert!(job_result.is_ok());

        if let Ok(mut job) = job_result {
            // Initially should be able to retry if max_retries > 0
            prop_assert_eq!(job.can_retry(), max_retries > 0);
            prop_assert_eq!(job.retry_count, 0);

            // Increment retry count and check can_retry logic
            for i in 0..increment_count {
                let before_retry = job.can_retry();
                job.increment_retry();

                prop_assert_eq!(job.retry_count, i + 1);

                // can_retry should be true if retry_count < max_retries
                let expected_can_retry = job.retry_count < max_retries;
                prop_assert_eq!(job.can_retry(), expected_can_retry);

                // If we couldn't retry before, we still can't after increment
                if !before_retry {
                    prop_assert!(!job.can_retry());
                }
            }
        }
    }

    #[test]
    fn test_job_serialization_roundtrip(
        tool_name in tool_name_strategy(),
        params in params_strategy(),
        max_retries in retry_count_strategy(),
        idempotency_key in idempotency_key_strategy(),
        retry_increments in 0u32..50u32,
    ) {
        let job_result = if let Some(ref key) = idempotency_key {
            Job::new_idempotent(&tool_name, &params, max_retries, key)
        } else {
            Job::new(&tool_name, &params, max_retries)
        };

        prop_assert!(job_result.is_ok());

        if let Ok(mut original_job) = job_result {
            // Increment retry count to test serialization of modified state
            for _ in 0..retry_increments.min(max_retries + 10) {
                original_job.increment_retry();
            }

            // Serialize to JSON
            let serialized = serde_json::to_string(&original_job);
            prop_assert!(serialized.is_ok());

            if let Ok(json_string) = serialized {
                // Deserialize back
                let deserialized: Result<Job, _> = serde_json::from_str(&json_string);
                prop_assert!(deserialized.is_ok());

                if let Ok(deserialized_job) = deserialized {
                    // All fields should match
                    prop_assert_eq!(deserialized_job.job_id, original_job.job_id);
                    prop_assert_eq!(deserialized_job.tool_name, original_job.tool_name);
                    prop_assert_eq!(deserialized_job.params, original_job.params);
                    prop_assert_eq!(deserialized_job.idempotency_key, original_job.idempotency_key);
                    prop_assert_eq!(deserialized_job.max_retries, original_job.max_retries);
                    prop_assert_eq!(deserialized_job.retry_count, original_job.retry_count);
                    prop_assert_eq!(deserialized_job.can_retry(), original_job.can_retry());
                }
            }
        }
    }

    #[test]
    fn test_job_result_success_creation(
        value in params_strategy(),
        tx_hash in prop::option::of("[0-9a-fA-F]{64}"),
    ) {
        if let Some(hash) = tx_hash {
            let result = JobResult::success_with_tx(&value, &hash);
            prop_assert!(result.is_ok());

            if let Ok(job_result) = result {
                prop_assert!(job_result.is_success());
                prop_assert!(!job_result.is_retriable());

                match job_result {
                    JobResult::Success { value: result_value, tx_hash: result_hash } => {
                        prop_assert_eq!(result_value, value);
                        prop_assert_eq!(result_hash, Some(hash));
                    }
                    _ => prop_assert!(false, "Expected Success variant"),
                }
            }
        } else {
            let result = JobResult::success(&value);
            prop_assert!(result.is_ok());

            if let Ok(job_result) = result {
                prop_assert!(job_result.is_success());
                prop_assert!(!job_result.is_retriable());

                match job_result {
                    JobResult::Success { value: result_value, tx_hash: result_hash } => {
                        prop_assert_eq!(result_value, value);
                        prop_assert!(result_hash.is_none());
                    }
                    _ => prop_assert!(false, "Expected Success variant"),
                }
            }
        }
    }

    #[test]
    fn test_job_result_failure_creation(
        error_message in error_message_strategy(),
        retriable in any::<bool>(),
    ) {
        let result = if retriable {
            JobResult::retriable_failure(&error_message)
        } else {
            JobResult::permanent_failure(&error_message)
        };

        prop_assert!(!result.is_success());
        prop_assert_eq!(result.is_retriable(), retriable);

        match result {
            JobResult::Failure { error, retriable: is_retriable } => {
                prop_assert_eq!(error, error_message);
                prop_assert_eq!(is_retriable, retriable);
            }
            _ => prop_assert!(false, "Expected Failure variant"),
        }
    }

    #[test]
    fn test_job_result_serialization_roundtrip(
        value in params_strategy(),
        tx_hash in prop::option::of("[0-9a-fA-F]{64}"),
        error_message in error_message_strategy(),
        is_success in any::<bool>(),
        retriable in any::<bool>(),
    ) {
        let original_result = if is_success {
            if let Some(ref hash) = tx_hash {
                JobResult::success_with_tx(&value, hash).unwrap()
            } else {
                JobResult::success(&value).unwrap()
            }
        } else {
            if retriable {
                JobResult::retriable_failure(&error_message)
            } else {
                JobResult::permanent_failure(&error_message)
            }
        };

        // Serialize
        let serialized = serde_json::to_string(&original_result);
        prop_assert!(serialized.is_ok());

        if let Ok(json_string) = serialized {
            // Deserialize
            let deserialized: Result<JobResult, _> = serde_json::from_str(&json_string);
            prop_assert!(deserialized.is_ok());

            if let Ok(deserialized_result) = deserialized {
                // Properties should match
                prop_assert_eq!(deserialized_result.is_success(), original_result.is_success());
                prop_assert_eq!(deserialized_result.is_retriable(), original_result.is_retriable());

                match (&original_result, &deserialized_result) {
                    (
                        JobResult::Success { value: v1, tx_hash: h1 },
                        JobResult::Success { value: v2, tx_hash: h2 }
                    ) => {
                        prop_assert_eq!(v1, v2);
                        prop_assert_eq!(h1, h2);
                    }
                    (
                        JobResult::Failure { error: e1, retriable: r1 },
                        JobResult::Failure { error: e2, retriable: r2 }
                    ) => {
                        prop_assert_eq!(e1, e2);
                        prop_assert_eq!(r1, r2);
                    }
                    _ => prop_assert!(false, "Result variants should match"),
                }
            }
        }
    }

    #[test]
    fn test_job_ids_are_unique(
        tool_names in prop::collection::vec(tool_name_strategy(), 1..100),
    ) {
        let mut job_ids = std::collections::HashSet::new();
        let mut duplicate_found = false;

        for tool_name in tool_names {
            if let Ok(job) = Job::new(&tool_name, &json!({}), 0) {
                if !job_ids.insert(job.job_id) {
                    duplicate_found = true;
                    break;
                }
            }
        }

        // Job IDs should be unique (UUID v4 has negligible collision probability)
        prop_assert!(!duplicate_found, "Found duplicate job IDs");
    }

    #[test]
    fn test_job_clone_properties(
        tool_name in tool_name_strategy(),
        params in params_strategy(),
        max_retries in retry_count_strategy(),
        idempotency_key in idempotency_key_strategy(),
        retry_increments in 0u32..20u32,
    ) {
        let job_result = if let Some(ref key) = idempotency_key {
            Job::new_idempotent(&tool_name, &params, max_retries, key)
        } else {
            Job::new(&tool_name, &params, max_retries)
        };

        prop_assert!(job_result.is_ok());

        if let Ok(mut original_job) = job_result {
            // Modify the job state
            for _ in 0..retry_increments.min(max_retries + 5) {
                original_job.increment_retry();
            }

            // Clone the job
            let cloned_job = original_job.clone();

            // All fields should be identical
            prop_assert_eq!(cloned_job.job_id, original_job.job_id);
            prop_assert_eq!(cloned_job.tool_name, original_job.tool_name);
            prop_assert_eq!(cloned_job.params, original_job.params);
            prop_assert_eq!(cloned_job.idempotency_key, original_job.idempotency_key);
            prop_assert_eq!(cloned_job.max_retries, original_job.max_retries);
            prop_assert_eq!(cloned_job.retry_count, original_job.retry_count);
            prop_assert_eq!(cloned_job.can_retry(), original_job.can_retry());

            // Modifying clone should not affect original
            let original_retry_count = original_job.retry_count;
            let mut cloned_job_mut = cloned_job;
            cloned_job_mut.increment_retry();

            prop_assert_eq!(original_job.retry_count, original_retry_count);
            prop_assert_eq!(cloned_job_mut.retry_count, original_retry_count + 1);
        }
    }

    #[test]
    fn test_job_result_clone_properties(
        value in params_strategy(),
        error_message in error_message_strategy(),
        is_success in any::<bool>(),
        retriable in any::<bool>(),
    ) {
        let original_result = if is_success {
            JobResult::success(&value).unwrap()
        } else {
            if retriable {
                JobResult::retriable_failure(&error_message)
            } else {
                JobResult::permanent_failure(&error_message)
            }
        };

        let cloned_result = original_result.clone();

        // Properties should match
        prop_assert_eq!(cloned_result.is_success(), original_result.is_success());
        prop_assert_eq!(cloned_result.is_retriable(), original_result.is_retriable());

        // Content should match
        match (&original_result, &cloned_result) {
            (
                JobResult::Success { value: v1, tx_hash: h1 },
                JobResult::Success { value: v2, tx_hash: h2 }
            ) => {
                prop_assert_eq!(v1, v2);
                prop_assert_eq!(h1, h2);
            }
            (
                JobResult::Failure { error: e1, retriable: r1 },
                JobResult::Failure { error: e2, retriable: r2 }
            ) => {
                prop_assert_eq!(e1, e2);
                prop_assert_eq!(r1, r2);
            }
            _ => prop_assert!(false, "Result variants should match"),
        }
    }

    #[test]
    fn test_edge_case_retry_counts(
        max_retries in prop::option::of(0u32..1000u32),
    ) {
        let max_retries = max_retries.unwrap_or(0);
        let job_result = Job::new("edge_test", &json!({}), max_retries);
        prop_assert!(job_result.is_ok());

        if let Ok(mut job) = job_result {
            // Test initial state
            prop_assert_eq!(job.retry_count, 0);
            prop_assert_eq!(job.can_retry(), max_retries > 0);

            // Increment up to and beyond max_retries
            for i in 1..=(max_retries + 10) {
                job.increment_retry();
                prop_assert_eq!(job.retry_count, i);

                // can_retry should be true only if retry_count < max_retries
                let expected_can_retry = i < max_retries;
                prop_assert_eq!(job.can_retry(), expected_can_retry);
            }
        }
    }
}

// Additional hand-written property tests for complex scenarios

#[test]
fn test_job_consistency_invariants() {
    // Test that certain invariants always hold
    proptest!(|(
        tool_name in tool_name_strategy(),
        params in params_strategy(),
        max_retries in retry_count_strategy(),
    )| {
        if let Ok(mut job) = Job::new(&tool_name, &params, max_retries) {
            // Invariant 1: retry_count should never exceed u32::MAX
            for _ in 0..10 {
                let before = job.retry_count;
                job.increment_retry();
                prop_assert!(job.retry_count == before.saturating_add(1));
            }

            // Invariant 2: can_retry() should always equal (retry_count < max_retries)
            prop_assert_eq!(job.can_retry(), job.retry_count < max_retries);

            // Invariant 3: job_id should never be nil UUID
            prop_assert_ne!(job.job_id.to_string(), "00000000-0000-0000-0000-000000000000");

            // Invariant 4: tool_name should never be empty (our strategy ensures this)
            prop_assert!(!job.tool_name.is_empty());
        }
    });
}

#[test]
fn test_job_result_consistency_invariants() {
    proptest!(|(
        value in params_strategy(),
        error_msg in error_message_strategy(),
    )| {
        // Success results
        if let Ok(success) = JobResult::success(&value) {
            // Invariant 1: Success results are never retriable
            prop_assert!(success.is_success());
            prop_assert!(!success.is_retriable());
        }

        // Retriable failure results
        let retriable_failure = JobResult::retriable_failure(&error_msg);
        prop_assert!(!retriable_failure.is_success());
        prop_assert!(retriable_failure.is_retriable());

        // Permanent failure results
        let permanent_failure = JobResult::permanent_failure(&error_msg);
        prop_assert!(!permanent_failure.is_success());
        prop_assert!(!permanent_failure.is_retriable());
    });
}

#[test]
fn test_large_data_handling() {
    // Test handling of very large data structures
    proptest!(|(
        array_size in 0usize..1000,
        string_length in 0usize..1000,
    )| {
        // Large array
        let large_array = (0..array_size).map(|i| json!(i)).collect::<Vec<_>>();
        let large_params = json!({"large_array": large_array});

        if let Ok(job) = Job::new("large_data_test", &large_params, 3) {
            prop_assert_eq!(job.params["large_array"].as_array().unwrap().len(), array_size);
        }

        // Large string
        let large_string = "x".repeat(string_length);
        let string_params = json!({"large_string": large_string});

        if let Ok(job) = Job::new("large_string_test", &string_params, 3) {
            prop_assert_eq!(
                job.params["large_string"].as_str().unwrap().len(),
                string_length
            );
        }

        // Test serialization with large data
        if string_length > 0 && string_length < 100 { // Avoid timeout with very large strings
            let large_result = JobResult::success(&json!({"data": "x".repeat(string_length)}));
            prop_assert!(large_result.is_ok());

            if let Ok(result) = large_result {
                let serialized = serde_json::to_string(&result);
                prop_assert!(serialized.is_ok());
            }
        }
    });
}

#[test]
fn test_job_with_serialization_failing_params() {
    // Test that Jobs handle serialization errors properly
    // Create a type that will fail serialization
    struct UnserializableType {
        _data: std::rc::Rc<()>, // Rc cannot be serialized
    }

    impl serde::Serialize for UnserializableType {
        fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            Err(serde::ser::Error::custom("Cannot serialize this type"))
        }
    }

    let unserializable = UnserializableType {
        _data: std::rc::Rc::new(()),
    };

    // Job::new should return an error for unserializable params
    let result = Job::new("test_tool", &unserializable, 3);
    assert!(result.is_err());

    // Job::new_idempotent should also return an error
    let result_idempotent = Job::new_idempotent(
        "test_tool",
        &unserializable,
        3,
        "test_key"
    );
    assert!(result_idempotent.is_err());
}

#[test]
fn test_job_retry_count_saturation() {
    // Test that retry_count saturates at u32::MAX and doesn't overflow
    let mut job = Job::new("test_tool", &json!({}), u32::MAX).unwrap();

    // Set retry_count to MAX - 1
    job.retry_count = u32::MAX - 1;
    assert!(job.can_retry()); // MAX-1 < MAX, should be able to retry

    // Increment to MAX
    job.increment_retry();
    assert_eq!(job.retry_count, u32::MAX);
    assert!(!job.can_retry()); // MAX >= MAX, cannot retry

    // Try to increment beyond MAX - should saturate
    job.increment_retry();
    assert_eq!(job.retry_count, u32::MAX.saturating_add(1)); // Should still be MAX
    assert!(!job.can_retry());
}

#[test]
fn test_job_result_success_with_serialization_failing_value() {
    // Test that JobResult handles serialization errors properly
    struct UnserializableType {
        _data: std::rc::Rc<()>,
    }

    impl serde::Serialize for UnserializableType {
        fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            Err(serde::ser::Error::custom("Cannot serialize this type"))
        }
    }

    let unserializable = UnserializableType {
        _data: std::rc::Rc::new(()),
    };

    // JobResult::success should return an error
    let result = JobResult::success(&unserializable);
    assert!(result.is_err());

    // JobResult::success_with_tx should also return an error
    let result_with_tx = JobResult::success_with_tx(&unserializable, "0xabc123");
    assert!(result_with_tx.is_err());
}

#[test]
fn test_job_with_empty_and_special_strings() {
    // Test edge cases with empty and special strings
    let empty_tool = Job::new("", &json!({}), 0);
    assert!(empty_tool.is_ok()); // Empty tool name is technically allowed

    let whitespace_tool = Job::new("   ", &json!({}), 0);
    assert!(whitespace_tool.is_ok());

    let special_chars = Job::new("!@#$%^&*()", &json!({}), 0);
    assert!(special_chars.is_ok());

    // Test with empty idempotency key
    let empty_key = Job::new_idempotent("tool", &json!({}), 0, "");
    assert!(empty_key.is_ok());
    if let Ok(job) = empty_key {
        assert_eq!(job.idempotency_key, Some("".to_string()));
    }

    // Test JobResult with empty error message
    let empty_error = JobResult::retriable_failure("");
    assert!(empty_error.is_retriable());

    let empty_permanent = JobResult::permanent_failure("");
    assert!(!empty_permanent.is_retriable());
}

#[test]
fn test_job_result_pattern_exhaustiveness() {
    // Test that all JobResult patterns are handled correctly
    let success = JobResult::success(&json!({"data": "test"})).unwrap();
    let retriable = JobResult::retriable_failure("error");
    let permanent = JobResult::permanent_failure("error");

    fn check_result(result: &JobResult) -> String {
        match result {
            JobResult::Success { value, tx_hash } => {
                format!("Success: value={:?}, tx={:?}", value, tx_hash)
            }
            JobResult::Failure { error, retriable } => {
                format!("Failure: error={}, retriable={}", error, retriable)
            }
        }
    }

    // Ensure all variants are covered
    assert!(check_result(&success).starts_with("Success"));
    assert!(check_result(&retriable).starts_with("Failure"));
    assert!(check_result(&permanent).starts_with("Failure"));
}