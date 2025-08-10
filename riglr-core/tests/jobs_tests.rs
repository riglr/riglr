//! Comprehensive tests for jobs module

use riglr_core::jobs::{Job, JobResult};
use serde_json::json;
use uuid::Uuid;

#[test]
fn test_job_creation_with_various_params() {
    // Test with simple params
    let simple_params = json!({"key": "value"});
    let job = Job::new("simple_tool", &simple_params, 3).unwrap();
    assert_eq!(job.tool_name, "simple_tool");
    assert_eq!(job.params, simple_params);
    assert_eq!(job.max_retries, 3);
    assert_eq!(job.retry_count, 0);
    assert!(job.idempotency_key.is_none());

    // Test with complex params
    let complex_params = json!({
        "nested": {
            "array": [1, 2, 3],
            "object": {"inner": "value"}
        },
        "number": 42,
        "bool": true,
        "null": null
    });
    let job = Job::new("complex_tool", &complex_params, 5).unwrap();
    assert_eq!(job.params, complex_params);

    // Test with empty params
    let empty_params = json!({});
    let job = Job::new("empty_tool", &empty_params, 0).unwrap();
    assert_eq!(job.params, empty_params);
    assert_eq!(job.max_retries, 0);
}

#[test]
fn test_job_idempotent_creation() {
    let params = json!({"test": "data"});

    // Test with string idempotency key
    let job = Job::new_idempotent("tool1", &params, 2, "idempotent_key_123").unwrap();
    assert_eq!(job.idempotency_key, Some("idempotent_key_123".to_string()));

    // Test with generated idempotency key
    let uuid = Uuid::new_v4().to_string();
    let job = Job::new_idempotent("tool2", &params, 1, uuid.clone()).unwrap();
    assert_eq!(job.idempotency_key, Some(uuid));

    // Test with empty idempotency key
    let job = Job::new_idempotent("tool3", &params, 0, "").unwrap();
    assert_eq!(job.idempotency_key, Some("".to_string()));
}

#[test]
fn test_job_retry_logic_edge_cases() {
    let params = json!({});

    // Test with zero max retries
    let mut job = Job::new("tool", &params, 0).unwrap();
    assert!(!job.can_retry());
    job.increment_retry();
    assert_eq!(job.retry_count, 1);
    assert!(!job.can_retry());

    // Test with high retry count
    let mut job = Job::new("tool", &params, 100).unwrap();
    for _ in 0..100 {
        assert!(job.can_retry());
        job.increment_retry();
    }
    assert!(!job.can_retry());
    assert_eq!(job.retry_count, 100);

    // Continue incrementing beyond max
    job.increment_retry();
    assert_eq!(job.retry_count, 101);
    assert!(!job.can_retry());
}

#[test]
fn test_job_id_uniqueness() {
    let params = json!({});
    let job1 = Job::new("tool", &params, 0).unwrap();
    let job2 = Job::new("tool", &params, 0).unwrap();

    // Job IDs should be unique
    assert_ne!(job1.job_id, job2.job_id);
}

#[test]
fn test_job_result_success_variations() {
    // Simple success
    let result = JobResult::success(&"simple").unwrap();
    assert!(result.is_success());
    assert!(!result.is_retriable());
    match result {
        JobResult::Success { value, tx_hash } => {
            assert_eq!(value, json!("simple"));
            assert!(tx_hash.is_none());
        }
        _ => panic!("Expected Success"),
    }

    // Success with complex value
    let complex_value = json!({
        "status": "completed",
        "data": [1, 2, 3],
        "metadata": {"timestamp": 123456}
    });
    let result = JobResult::success(&complex_value).unwrap();
    match result {
        JobResult::Success { value, .. } => {
            assert_eq!(value, complex_value);
        }
        _ => panic!("Expected Success"),
    }

    // Success with transaction hash
    let result = JobResult::success_with_tx(&42, "0xabc123def456").unwrap();
    assert!(result.is_success());
    match result {
        JobResult::Success { value, tx_hash } => {
            assert_eq!(value, json!(42));
            assert_eq!(tx_hash, Some("0xabc123def456".to_string()));
        }
        _ => panic!("Expected Success"),
    }

    // Success with empty tx hash
    let result = JobResult::success_with_tx(&"data", "").unwrap();
    match result {
        JobResult::Success { tx_hash, .. } => {
            assert_eq!(tx_hash, Some("".to_string()));
        }
        _ => panic!("Expected Success"),
    }
}

#[test]
fn test_job_result_failure_variations() {
    // Retriable failure
    let result = JobResult::retriable_failure("Network timeout");
    assert!(!result.is_success());
    assert!(result.is_retriable());
    match result {
        JobResult::Failure { error, retriable } => {
            assert_eq!(error, "Network timeout");
            assert!(retriable);
        }
        _ => panic!("Expected Failure"),
    }

    // Permanent failure
    let result = JobResult::permanent_failure("Invalid input data");
    assert!(!result.is_success());
    assert!(!result.is_retriable());
    match result {
        JobResult::Failure { error, retriable } => {
            assert_eq!(error, "Invalid input data");
            assert!(!retriable);
        }
        _ => panic!("Expected Failure"),
    }

    // Failure with empty error message
    let result = JobResult::permanent_failure("");
    match result {
        JobResult::Failure { error, .. } => {
            assert_eq!(error, "");
        }
        _ => panic!("Expected Failure"),
    }

    // Failure with very long error message
    let long_error = "x".repeat(10000);
    let result = JobResult::retriable_failure(long_error.clone());
    match result {
        JobResult::Failure { error, .. } => {
            assert_eq!(error, long_error);
        }
        _ => panic!("Expected Failure"),
    }
}

#[test]
fn test_job_serialization_deserialization() {
    // Create a job with all fields populated
    let mut job =
        Job::new_idempotent("test_tool", &json!({"param": "value"}), 5, "test_key").unwrap();
    job.retry_count = 2;

    // Serialize to JSON
    let serialized = serde_json::to_string(&job).unwrap();

    // Deserialize back
    let deserialized: Job = serde_json::from_str(&serialized).unwrap();

    // Verify all fields
    assert_eq!(deserialized.job_id, job.job_id);
    assert_eq!(deserialized.tool_name, job.tool_name);
    assert_eq!(deserialized.params, job.params);
    assert_eq!(deserialized.idempotency_key, job.idempotency_key);
    assert_eq!(deserialized.max_retries, job.max_retries);
    assert_eq!(deserialized.retry_count, job.retry_count);
}

#[test]
fn test_job_result_serialization_deserialization() {
    // Test Success serialization
    let success = JobResult::success_with_tx(&json!({"data": 123}), "tx_123").unwrap();
    let serialized = serde_json::to_string(&success).unwrap();
    let deserialized: JobResult = serde_json::from_str(&serialized).unwrap();
    assert!(deserialized.is_success());

    // Test Failure serialization
    let failure = JobResult::retriable_failure("error message");
    let serialized = serde_json::to_string(&failure).unwrap();
    let deserialized: JobResult = serde_json::from_str(&serialized).unwrap();
    assert!(deserialized.is_retriable());
}

#[test]
fn test_job_clone() {
    let job = Job::new_idempotent("clone_tool", &json!({"test": true}), 3, "clone_key").unwrap();

    let cloned = job.clone();

    // Verify all fields are cloned correctly
    assert_eq!(cloned.job_id, job.job_id);
    assert_eq!(cloned.tool_name, job.tool_name);
    assert_eq!(cloned.params, job.params);
    assert_eq!(cloned.idempotency_key, job.idempotency_key);
    assert_eq!(cloned.max_retries, job.max_retries);
    assert_eq!(cloned.retry_count, job.retry_count);
}

#[test]
fn test_job_result_clone() {
    let success = JobResult::success_with_tx(&"data", "tx").unwrap();
    let cloned = success.clone();
    assert!(cloned.is_success());

    let failure = JobResult::retriable_failure("error");
    let cloned = failure.clone();
    assert!(cloned.is_retriable());
}

#[test]
fn test_job_debug_format() {
    let job = Job::new("debug_tool", &json!({"key": "value"}), 2).unwrap();
    let debug_str = format!("{:?}", job);

    // Verify debug output contains key fields
    assert!(debug_str.contains("job_id"));
    assert!(debug_str.contains("tool_name"));
    assert!(debug_str.contains("debug_tool"));
    assert!(debug_str.contains("params"));
}

#[test]
fn test_job_result_debug_format() {
    let result = JobResult::success(&"test").unwrap();
    let debug_str = format!("{:?}", result);
    assert!(debug_str.contains("Success"));

    let failure = JobResult::retriable_failure("error");
    let debug_str = format!("{:?}", failure);
    assert!(debug_str.contains("Failure"));
    assert!(debug_str.contains("retriable"));
}

#[test]
fn test_job_with_special_characters_in_tool_name() {
    let special_names = vec![
        "tool-with-dash",
        "tool_with_underscore",
        "tool.with.dot",
        "tool/with/slash",
        "tool:with:colon",
        "tool@with@at",
        "ツール", // Japanese characters
        "🔧",     // Emoji
    ];

    for name in special_names {
        let job = Job::new(name, &json!({}), 0).unwrap();
        assert_eq!(job.tool_name, name);
    }
}

#[test]
fn test_job_result_with_various_value_types() {
    // Test with different JSON value types
    assert!(JobResult::success(&true).unwrap().is_success());
    assert!(JobResult::success(&false).unwrap().is_success());
    assert!(JobResult::success(&123i32).unwrap().is_success());
    assert!(JobResult::success(&123.456f64).unwrap().is_success());
    assert!(JobResult::success(&"string").unwrap().is_success());
    assert!(JobResult::success(&vec![1, 2, 3]).unwrap().is_success());
    assert!(JobResult::success(&Option::<i32>::None)
        .unwrap()
        .is_success());
    assert!(JobResult::success(&Some(42)).unwrap().is_success());
}

#[test]
fn test_job_creation_serialization_errors() {
    use serde::ser::{Error, Serialize, Serializer};

    // Create a type that always fails to serialize
    struct FailingSerialize;

    impl Serialize for FailingSerialize {
        fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            Err(S::Error::custom("Intentional serialization failure"))
        }
    }

    let failing_params = FailingSerialize;

    // These should fail because our custom type fails to serialize
    let result = Job::new("test_tool", &failing_params, 3);
    assert!(result.is_err());

    let result = Job::new_idempotent("test_tool", &failing_params, 3, "key");
    assert!(result.is_err());
}

#[test]
fn test_job_result_serialization_errors() {
    use serde::ser::{Error, Serialize, Serializer};

    // Create a type that always fails to serialize
    struct FailingSerialize;

    impl Serialize for FailingSerialize {
        fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            Err(S::Error::custom("Intentional serialization failure"))
        }
    }

    let failing_value = FailingSerialize;

    // These should fail because our custom type fails to serialize
    let result = JobResult::success(&failing_value);
    assert!(result.is_err());

    let result = JobResult::success_with_tx(&failing_value, "tx_hash");
    assert!(result.is_err());
}
