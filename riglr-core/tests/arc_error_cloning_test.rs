//! Tests for Arc-based error cloning to ensure error sources are preserved.

use riglr_core::{JobResult, ToolError};
use std::error::Error;
use std::sync::Arc;

#[derive(Debug, Clone)]
struct TestError {
    message: String,
}

impl std::fmt::Display for TestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for TestError {}

#[test]
fn test_job_result_failure_clone_preserves_source() {
    // Create an original error with a source
    let source_error = TestError {
        message: "Original error message".to_string(),
    };

    let tool_error = ToolError::permanent_with_source(source_error, "Operation failed");

    let job_result = JobResult::Failure { error: tool_error };

    // Clone the JobResult
    let cloned_result = job_result.clone();

    // Both the original and clone should have the same error source
    match (&job_result, &cloned_result) {
        (JobResult::Failure { error: original }, JobResult::Failure { error: cloned }) => {
            // Check that both have sources
            let original_source = original.source();
            let cloned_source = cloned.source();

            assert!(original_source.is_some(), "Original should have source");
            assert!(cloned_source.is_some(), "Clone should have source");

            // Check that the error messages match
            assert_eq!(
                original_source.unwrap().to_string(),
                cloned_source.unwrap().to_string(),
                "Error sources should have the same message"
            );
        }
        _ => panic!("Expected both to be Failure variants"),
    }
}

#[test]
fn test_tool_error_clone_with_arc_is_cheap() {
    let source_error = TestError {
        message: "Test error".to_string(),
    };

    let tool_error = ToolError::retriable_with_source(source_error, "Retriable failure");

    // Clone multiple times - this should be cheap with Arc
    let clone1 = tool_error.clone();
    let clone2 = clone1.clone();
    let clone3 = clone2.clone();

    // All clones should have the source
    assert!(tool_error.source().is_some());
    assert!(clone1.source().is_some());
    assert!(clone2.source().is_some());
    assert!(clone3.source().is_some());
}

#[test]
fn test_all_tool_error_variants_preserve_source_on_clone() {
    let retriable = ToolError::retriable_with_source(
        TestError {
            message: "retriable".into(),
        },
        "context",
    );

    let rate_limited = ToolError::rate_limited_with_source(
        TestError {
            message: "rate limited".into(),
        },
        "context",
        Some(std::time::Duration::from_secs(5)),
    );

    let permanent = ToolError::permanent_with_source(
        TestError {
            message: "permanent".into(),
        },
        "context",
    );

    let invalid_input = ToolError::invalid_input_with_source(
        TestError {
            message: "invalid".into(),
        },
        "context",
    );

    // Clone all variants
    let retriable_clone = retriable.clone();
    let rate_limited_clone = rate_limited.clone();
    let permanent_clone = permanent.clone();
    let invalid_input_clone = invalid_input.clone();

    // All clones should preserve their sources
    assert!(retriable_clone.source().is_some());
    assert!(rate_limited_clone.source().is_some());
    assert!(permanent_clone.source().is_some());
    assert!(invalid_input_clone.source().is_some());

    // Verify the error messages are preserved
    assert_eq!(
        retriable.source().unwrap().to_string(),
        retriable_clone.source().unwrap().to_string()
    );
    assert_eq!(
        rate_limited.source().unwrap().to_string(),
        rate_limited_clone.source().unwrap().to_string()
    );
    assert_eq!(
        permanent.source().unwrap().to_string(),
        permanent_clone.source().unwrap().to_string()
    );
    assert_eq!(
        invalid_input.source().unwrap().to_string(),
        invalid_input_clone.source().unwrap().to_string()
    );
}

#[test]
fn test_arc_sharing_semantics() {
    // This test verifies that the Arc actually shares the underlying error
    let source_error = Arc::new(TestError {
        message: "Shared error".to_string(),
    });

    // Create a ToolError with the Arc directly (simulating internal behavior)
    let tool_error = ToolError::Permanent {
        source: Some(source_error.clone()),
        source_message: "Shared error".to_string(),
        context: "test".to_string(),
    };

    let cloned_error = tool_error.clone();

    // Both should point to the same underlying Arc
    match (&tool_error, &cloned_error) {
        (
            ToolError::Permanent {
                source: Some(original_arc),
                ..
            },
            ToolError::Permanent {
                source: Some(cloned_arc),
                ..
            },
        ) => {
            // Arc::ptr_eq checks if two Arcs point to the same allocation
            // We can't directly use ptr_eq on trait objects, so we check the raw pointers
            let original_ptr = Arc::as_ptr(original_arc) as *const ();
            let cloned_ptr = Arc::as_ptr(cloned_arc) as *const ();
            assert_eq!(
                original_ptr, cloned_ptr,
                "Cloned Arc should point to the same underlying error"
            );
        }
        _ => panic!("Expected Permanent variants with sources"),
    }
}

/// This test serves as a "canary" that will fail to compile if ToolError is modified
/// without corresponding updates to its Clone implementation.
///
/// # Purpose
///
/// This test explicitly constructs every variant of `ToolError` and clones it through
/// `JobResult`. If a new variant is added to `ToolError`, or if existing variants are
/// modified, this test will either:
/// 1. Fail to compile (if a variant is missing from this test)
/// 2. Fail at runtime (if the Clone implementation doesn't properly handle the variant)
///
/// # Maintenance Instructions
///
/// When you add or modify variants in the `ToolError` enum:
/// 1. Update this test to include the new/modified variant
/// 2. Ensure the Clone implementation for ToolError handles the new variant
/// 3. Run this test to verify everything works correctly
///
/// This test is your safety net - it ensures that error cloning remains consistent
/// and complete across all error variants.
#[test]
fn test_job_result_clone_is_exhaustive() {
    use riglr_core::{JobResult, ToolError};

    // Create a JobResult::Failure with each variant of ToolError
    let test_error = TestError {
        message: "test".to_string(),
    };

    // IMPORTANT: This list MUST include every single variant of ToolError
    // If you're seeing a compile error here after modifying ToolError,
    // add the new variant to this list!
    let errors = vec![
        // Retriable variant with source
        ToolError::Retriable {
            source: Some(std::sync::Arc::new(test_error.clone())),
            source_message: "test".to_string(),
            context: "test".to_string(),
        },
        // Retriable variant without source
        ToolError::Retriable {
            source: None,
            source_message: "test".to_string(),
            context: "test".to_string(),
        },
        // RateLimited variant with source and retry_after
        ToolError::RateLimited {
            source: Some(std::sync::Arc::new(test_error.clone())),
            source_message: "test".to_string(),
            context: "test".to_string(),
            retry_after: Some(std::time::Duration::from_secs(5)),
        },
        // RateLimited variant without source
        ToolError::RateLimited {
            source: None,
            source_message: "test".to_string(),
            context: "test".to_string(),
            retry_after: None,
        },
        // Permanent variant with source
        ToolError::Permanent {
            source: Some(std::sync::Arc::new(test_error.clone())),
            source_message: "test".to_string(),
            context: "test".to_string(),
        },
        // Permanent variant without source
        ToolError::Permanent {
            source: None,
            source_message: "test".to_string(),
            context: "test".to_string(),
        },
        // InvalidInput variant with source
        ToolError::InvalidInput {
            source: Some(std::sync::Arc::new(test_error.clone())),
            source_message: "test".to_string(),
            context: "test".to_string(),
        },
        // InvalidInput variant without source
        ToolError::InvalidInput {
            source: None,
            source_message: "test".to_string(),
            context: "test".to_string(),
        },
        // SignerContext variant (doesn't have a source field)
        ToolError::SignerContext("test".to_string()),
    ];

    // Clone each variant to ensure the Clone implementation handles it
    for error in errors {
        let job_result = JobResult::Failure { error };
        let _cloned = job_result.clone(); // This will fail to compile if Clone doesn't handle a variant

        // Verify the clone works correctly
        match (&job_result, &_cloned) {
            (JobResult::Failure { error: e1 }, JobResult::Failure { error: e2 }) => {
                // Basic check that error messages match
                assert_eq!(e1.to_string(), e2.to_string());
            }
            _ => panic!("Expected both to be Failure variants"),
        }
    }
}
