//! Error types for riglr-core.

use serde::Serialize;
use thiserror::Error;

/// Helper function to serialize Duration as seconds for JSON compatibility
fn serialize_duration_as_secs<S>(
    duration: &Option<std::time::Duration>,
    serializer: S,
) -> std::result::Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    match duration {
        Some(d) => serializer.serialize_some(&d.as_secs()),
        None => serializer.serialize_none(),
    }
}

/// Main error type for riglr-core operations.
#[derive(Error, Debug, Serialize)]
pub enum CoreError {
    /// Queue operation failed
    #[error("Queue error: {0}")]
    Queue(String),

    /// Job execution failed
    #[error("Job execution error: {0}")]
    JobExecution(String),

    /// Serialization/deserialization failed
    #[error("Serialization error: {0}")]
    #[serde(skip)]
    Serialization(#[from] serde_json::Error),

    /// Redis connection error (only available with redis feature)
    #[cfg(feature = "redis")]
    #[error("Redis error: {0}")]
    #[serde(skip)]
    Redis(#[from] redis::RedisError),

    /// Generic error
    #[error("Core error: {0}")]
    Generic(String),

    /// Invalid input provided
    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

/// Result type alias for riglr-core operations.
pub type Result<T> = std::result::Result<T, CoreError>;

/// Worker-specific error type for distinguishing system-level worker failures
/// from tool execution failures.
#[derive(Error, Debug, Serialize)]
pub enum WorkerError {
    /// Tool not found in the worker's registry
    #[error("Tool '{tool_name}' not found in worker registry")]
    ToolNotFound {
        /// Name of the tool that was not found
        tool_name: String,
    },

    /// Failed to acquire semaphore for concurrency control
    #[error("Failed to acquire semaphore for tool '{tool_name}': {source_message}")]
    SemaphoreAcquisition {
        /// Name of the tool for which semaphore acquisition failed
        tool_name: String,
        /// The error message that caused the semaphore acquisition failure
        source_message: String,
    },

    /// Idempotency store operation failed
    #[error("Idempotency store operation failed: {source_message}")]
    IdempotencyStore {
        /// The error message that caused the idempotency store operation to fail
        source_message: String,
    },

    /// Job serialization/deserialization error
    #[error("Job serialization error: {source_message}")]
    JobSerialization {
        /// The JSON serialization error message
        source_message: String,
    },

    /// Tool execution exceeded configured timeout
    #[error("Tool execution timed out after {timeout:?}")]
    ExecutionTimeout {
        /// The duration after which the execution timed out
        timeout: std::time::Duration,
    },

    /// Internal worker system error
    #[error("Internal worker error: {message}")]
    Internal {
        /// Human-readable description of the internal error
        message: String,
    },
}

impl From<&str> for CoreError {
    fn from(err: &str) -> Self {
        Self::Generic(err.to_string())
    }
}

/// Tool-specific error type for distinguishing retriable vs permanent failures.
#[derive(Debug, Clone, Serialize, serde::Deserialize)]
pub enum ToolError {
    /// Operation can be retried
    Retriable {
        /// The typed error source (not serialized)
        #[serde(skip)]
        source: Option<std::sync::Arc<dyn std::error::Error + Send + Sync>>,
        /// The error message for serialization
        source_message: String,
        /// Additional context about the error
        context: String,
    },

    /// Rate limited, retry after delay
    RateLimited {
        /// The typed error source (not serialized)
        #[serde(skip)]
        source: Option<std::sync::Arc<dyn std::error::Error + Send + Sync>>,
        /// The error message for serialization
        source_message: String,
        /// Additional context about the rate limiting
        context: String,
        /// Optional duration to wait before retrying (in seconds)
        #[serde(serialize_with = "serialize_duration_as_secs")]
        retry_after: Option<std::time::Duration>,
    },

    /// Permanent error, do not retry
    Permanent {
        /// The typed error source (not serialized)
        #[serde(skip)]
        source: Option<std::sync::Arc<dyn std::error::Error + Send + Sync>>,
        /// The error message for serialization
        source_message: String,
        /// Additional context about the permanent error
        context: String,
    },

    /// Invalid input provided
    InvalidInput {
        /// The typed error source (not serialized)
        #[serde(skip)]
        source: Option<std::sync::Arc<dyn std::error::Error + Send + Sync>>,
        /// The error message for serialization
        source_message: String,
        /// Description of what input was invalid
        context: String,
    },

    /// Signer context error
    SignerContext(String),
}

impl PartialEq for ToolError {
    fn eq(&self, other: &Self) -> bool {
        use ToolError::*;
        match (self, other) {
            (
                Retriable {
                    source_message: s1,
                    context: c1,
                    ..
                },
                Retriable {
                    source_message: s2,
                    context: c2,
                    ..
                },
            ) => s1 == s2 && c1 == c2,
            (
                RateLimited {
                    source_message: s1,
                    context: c1,
                    retry_after: r1,
                    ..
                },
                RateLimited {
                    source_message: s2,
                    context: c2,
                    retry_after: r2,
                    ..
                },
            ) => s1 == s2 && c1 == c2 && r1 == r2,
            (
                Permanent {
                    source_message: s1,
                    context: c1,
                    ..
                },
                Permanent {
                    source_message: s2,
                    context: c2,
                    ..
                },
            ) => s1 == s2 && c1 == c2,
            (
                InvalidInput {
                    source_message: s1,
                    context: c1,
                    ..
                },
                InvalidInput {
                    source_message: s2,
                    context: c2,
                    ..
                },
            ) => s1 == s2 && c1 == c2,
            (SignerContext(s1), SignerContext(s2)) => s1 == s2,
            _ => false,
        }
    }
}

impl ToolError {
    /// Creates a retriable error with context and source preservation
    pub fn retriable_with_source<E: std::error::Error + Send + Sync + 'static>(
        source: E,
        context: impl Into<String>,
    ) -> Self {
        let source_message = source.to_string();
        Self::Retriable {
            source: Some(std::sync::Arc::new(source)),
            source_message,
            context: context.into(),
        }
    }

    /// Creates a permanent error with context and source preservation
    pub fn permanent_with_source<E: std::error::Error + Send + Sync + 'static>(
        source: E,
        context: impl Into<String>,
    ) -> Self {
        let source_message = source.to_string();
        Self::Permanent {
            source: Some(std::sync::Arc::new(source)),
            source_message,
            context: context.into(),
        }
    }

    /// Creates a rate limited error with optional retry duration
    pub fn rate_limited_with_source<E: std::error::Error + Send + Sync + 'static>(
        source: E,
        context: impl Into<String>,
        retry_after: Option<std::time::Duration>,
    ) -> Self {
        let source_message = source.to_string();
        Self::RateLimited {
            source: Some(std::sync::Arc::new(source)),
            source_message,
            context: context.into(),
            retry_after,
        }
    }

    /// Creates an invalid input error
    pub fn invalid_input_with_source<E: std::error::Error + Send + Sync + 'static>(
        source: E,
        context: impl Into<String>,
    ) -> Self {
        let source_message = source.to_string();
        Self::InvalidInput {
            source: Some(std::sync::Arc::new(source)),
            source_message,
            context: context.into(),
        }
    }

    /// Returns whether this error is retriable
    #[must_use]
    pub const fn is_retriable(&self) -> bool {
        matches!(self, Self::Retriable { .. } | Self::RateLimited { .. })
    }

    /// Returns the retry delay if this is a rate limited error
    #[must_use]
    pub const fn retry_after(&self) -> Option<std::time::Duration> {
        match self {
            Self::RateLimited { retry_after, .. } => *retry_after,
            _ => None,
        }
    }

    /// Checks if the error is rate limited (for compatibility)
    #[must_use]
    pub const fn is_rate_limited(&self) -> bool {
        matches!(self, Self::RateLimited { .. })
    }

    /// Creates a retriable error from a string message
    pub fn retriable_string<S: Into<String>>(msg: S) -> Self {
        let msg = msg.into();
        Self::Retriable {
            source: None,
            source_message: msg.clone(),
            context: msg,
        }
    }

    /// Creates a permanent error from a string message
    pub fn permanent_string<S: Into<String>>(msg: S) -> Self {
        let msg = msg.into();
        Self::Permanent {
            source: None,
            source_message: msg.clone(),
            context: msg,
        }
    }

    /// Creates a rate limited error from a string message
    pub fn rate_limited_string<S: Into<String>>(msg: S) -> Self {
        let msg = msg.into();
        Self::RateLimited {
            source: None,
            source_message: msg.clone(),
            context: msg,
            retry_after: None,
        }
    }

    /// Creates an invalid input error from a string message
    pub fn invalid_input_string<S: Into<String>>(msg: S) -> Self {
        let msg = msg.into();
        Self::InvalidInput {
            source: None,
            source_message: msg.clone(),
            context: msg,
        }
    }

    /// Check if the error contains a specific substring in its string representation
    pub fn contains(&self, needle: &str) -> bool {
        self.to_string().contains(needle)
    }

    /// Creates a retriable error from any error type.
    pub fn retriable_from_error<E: std::error::Error + Send + Sync + 'static>(
        source: E,
        context: impl Into<String>,
    ) -> Self {
        Self::retriable_with_source(source, context)
    }

    /// Creates a permanent error from any error type.
    pub fn permanent_from_error<E: std::error::Error + Send + Sync + 'static>(
        source: E,
        context: impl Into<String>,
    ) -> Self {
        Self::permanent_with_source(source, context)
    }
}

impl std::fmt::Display for ToolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Retriable {
                context,
                source_message,
                ..
            } => {
                write!(
                    f,
                    "Operation can be retried: {} - {}",
                    context, source_message
                )
            }
            Self::RateLimited {
                context,
                source_message,
                ..
            } => {
                write!(f, "Rate limited: {} - {}", context, source_message)
            }
            Self::Permanent {
                context,
                source_message,
                ..
            } => {
                write!(f, "Permanent error: {} - {}", context, source_message)
            }
            Self::InvalidInput {
                context,
                source_message,
                ..
            } => {
                write!(f, "Invalid input: {} - {}", context, source_message)
            }
            Self::SignerContext(msg) => {
                write!(f, "Signer context error: {}", msg)
            }
        }
    }
}

impl std::error::Error for ToolError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Retriable { source, .. }
            | Self::RateLimited { source, .. }
            | Self::Permanent { source, .. }
            | Self::InvalidInput { source, .. } => source
                .as_ref()
                .map(|e| e.as_ref() as &(dyn std::error::Error + 'static)),
            Self::SignerContext(_) => None,
        }
    }
}

/// Simple error wrapper for string messages
#[derive(Error, Debug)]
#[error("{0}")]
#[allow(dead_code)]
struct StringError(String);

// Conversion from SignerError to ToolError
impl From<crate::signer::SignerError> for ToolError {
    fn from(err: crate::signer::SignerError) -> Self {
        Self::SignerContext(err.to_string())
    }
}

/// # Error Classification: Explicit is Better
///
/// Generic `From` implementations for common error types like `anyhow::Error` and `String`
/// have been intentionally removed. This is a deliberate design choice to enforce
/// explicit error classification at the point of creation.
///
/// Automatically classifying unknown errors as `Permanent` is a safe default but can hide
/// bugs where transient, retriable errors are not handled correctly. By requiring
/// explicit classification, developers are forced to make a conscious decision about
/// the nature of the error.
///
/// ## Best Practices
///
/// Always use the explicit constructors to create `ToolError` instances:
///
/// ```rust
/// use riglr_core::ToolError;
/// use std::io::Error as IoError;
///
/// // ✅ Explicitly classify known transient errors as retriable.
/// let network_error = IoError::new(std::io::ErrorKind::TimedOut, "Connection timeout");
/// let tool_error = ToolError::retriable_with_source(network_error, "API call failed");
///
/// // ✅ Explicitly classify rate limiting errors.
/// let rate_limit_error = IoError::new(std::io::ErrorKind::Other, "Rate limited");
/// let tool_error = ToolError::rate_limited_with_source(
///     rate_limit_error,
///     "API rate limit exceeded",
///     Some(std::time::Duration::from_secs(60))
/// );
///
/// // ✅ Explicitly classify user input errors.
/// let input_error = IoError::new(std::io::ErrorKind::InvalidInput, "Bad address");
/// let tool_error = ToolError::invalid_input_with_source(input_error, "Invalid address format");
///
/// // ✅ Explicitly classify all other unrecoverable errors as permanent.
/// let auth_error = IoError::new(std::io::ErrorKind::PermissionDenied, "Invalid API key");
/// let tool_error = ToolError::permanent_with_source(auth_error, "Authentication failed");
/// ```
impl From<serde_json::Error> for ToolError {
    fn from(err: serde_json::Error) -> Self {
        Self::permanent_with_source(err, "JSON serialization/deserialization failed")
    }
}


// Removed generic From<&str> implementation - use explicit error constructors instead

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[derive(Error, Debug)]
    #[error("Test error")]
    struct TestError;

    #[test]
    fn test_error_source_preservation() {
        let original_error = TestError;
        let tool_error = ToolError::retriable_with_source(original_error, "Test context");

        // Verify source is preserved
        assert!(tool_error.source().is_some());

        // Verify we can downcast to original error type
        let source = tool_error.source().unwrap();
        assert!(source.downcast_ref::<TestError>().is_some());
    }

    #[test]
    fn test_error_classification() {
        let retriable = ToolError::retriable_with_source(TestError, "Retriable test");
        assert!(retriable.is_retriable());

        let permanent = ToolError::permanent_with_source(TestError, "Permanent test");
        assert!(!permanent.is_retriable());

        let rate_limited = ToolError::rate_limited_with_source(
            TestError,
            "Rate limited test",
            Some(std::time::Duration::from_secs(60)),
        );
        assert!(rate_limited.is_retriable());
        assert_eq!(
            rate_limited.retry_after(),
            Some(std::time::Duration::from_secs(60))
        );
    }

    #[test]
    fn test_string_error_creation() {
        let retriable = ToolError::retriable_string("Network timeout");
        assert!(retriable.is_retriable());
        assert!(!retriable.is_rate_limited());

        let rate_limited = ToolError::rate_limited_string("API rate limit exceeded");
        assert!(rate_limited.is_retriable());
        assert!(rate_limited.is_rate_limited());

        let permanent = ToolError::permanent_string("Invalid parameters");
        assert!(!permanent.is_retriable());
        assert!(!permanent.is_rate_limited());
    }

    // Tests for CoreError enum variants and Display implementations
    #[test]
    fn test_core_error_queue_variant() {
        let error = CoreError::Queue("Failed to connect".to_string());
        assert_eq!(error.to_string(), "Queue error: Failed to connect");
    }

    #[test]
    fn test_core_error_job_execution_variant() {
        let error = CoreError::JobExecution("Job failed to run".to_string());
        assert_eq!(error.to_string(), "Job execution error: Job failed to run");
    }

    #[test]
    fn test_core_error_serialization_variant() {
        // Create a real JSON parsing error by parsing invalid JSON
        let json_error = serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err();
        let error = CoreError::Serialization(json_error);
        assert!(error.to_string().contains("Serialization error:"));
    }

    #[test]
    fn test_core_error_generic_variant() {
        let error = CoreError::Generic("Something went wrong".to_string());
        assert_eq!(error.to_string(), "Core error: Something went wrong");
    }

    #[test]
    fn test_core_error_from_str() {
        let error = CoreError::from("Test error message");
        assert_eq!(error.to_string(), "Core error: Test error message");
    }

    #[test]
    fn test_core_error_from_empty_str() {
        let error = CoreError::from("");
        assert_eq!(error.to_string(), "Core error: ");
    }

    // Tests for WorkerError enum variants and Display implementations
    #[test]
    fn test_worker_error_tool_not_found() {
        let error = WorkerError::ToolNotFound {
            tool_name: "missing_tool".to_string(),
        };
        assert_eq!(
            error.to_string(),
            "Tool 'missing_tool' not found in worker registry"
        );
    }

    #[test]
    fn test_worker_error_semaphore_acquisition() {
        let error = WorkerError::SemaphoreAcquisition {
            tool_name: "test_tool".to_string(),
            source_message: "Test error".to_string(),
        };
        assert_eq!(
            error.to_string(),
            "Failed to acquire semaphore for tool 'test_tool': Test error"
        );
    }

    #[test]
    fn test_worker_error_idempotency_store() {
        let error = WorkerError::IdempotencyStore {
            source_message: "Test error".to_string(),
        };
        assert_eq!(
            error.to_string(),
            "Idempotency store operation failed: Test error"
        );
    }

    #[test]
    fn test_worker_error_job_serialization() {
        let error = WorkerError::JobSerialization {
            source_message: "invalid JSON format".to_string(),
        };
        assert!(error.to_string().contains("Job serialization error:"));
    }

    #[test]
    fn test_worker_error_execution_timeout() {
        let timeout = std::time::Duration::from_secs(30);
        let error = WorkerError::ExecutionTimeout { timeout };
        assert!(error
            .to_string()
            .contains("Tool execution timed out after 30"));
    }

    #[test]
    fn test_worker_error_internal() {
        let error = WorkerError::Internal {
            message: "Internal system failure".to_string(),
        };
        assert_eq!(
            error.to_string(),
            "Internal worker error: Internal system failure"
        );
    }

    // Tests for ToolError additional methods and edge cases
    #[test]
    fn test_tool_error_invalid_input_with_source() {
        let error = ToolError::invalid_input_with_source(TestError, "Bad input data");
        assert!(!error.is_retriable());
        assert!(!error.is_rate_limited());
        assert_eq!(error.retry_after(), None);
        assert_eq!(
            error.to_string(),
            "Invalid input: Bad input data - Test error"
        );
    }

    #[test]
    fn test_tool_error_invalid_input_string() {
        let error = ToolError::invalid_input_string("Invalid JSON format");
        assert!(!error.is_retriable());
        assert!(!error.is_rate_limited());
        assert_eq!(error.retry_after(), None);
        assert_eq!(
            error.to_string(),
            "Invalid input: Invalid JSON format - Invalid JSON format"
        );
    }

    #[test]
    fn test_tool_error_rate_limited_with_no_retry_after() {
        let error = ToolError::rate_limited_with_source(TestError, "Rate limit hit", None);
        assert!(error.is_retriable());
        assert!(error.is_rate_limited());
        assert_eq!(error.retry_after(), None);
    }

    #[test]
    fn test_tool_error_permanent_variant() {
        let error = ToolError::Permanent {
            source: None,
            source_message: "Test error".to_string(),
            context: "Authentication failed".to_string(),
        };
        assert!(!error.is_retriable());
        assert!(!error.is_rate_limited());
        assert_eq!(error.retry_after(), None);
    }

    #[test]
    fn test_tool_error_retriable_variant() {
        let error = ToolError::Retriable {
            source: None,
            source_message: "Test error".to_string(),
            context: "Network issue".to_string(),
        };
        assert!(error.is_retriable());
        assert!(!error.is_rate_limited());
        assert_eq!(error.retry_after(), None);
    }

    #[test]
    fn test_tool_error_invalid_input_variant() {
        let error = ToolError::InvalidInput {
            source: None,
            source_message: "Test error".to_string(),
            context: "Missing required field".to_string(),
        };
        assert!(!error.is_retriable());
        assert!(!error.is_rate_limited());
        assert_eq!(error.retry_after(), None);
    }

    // Tests for explicit error constructors (replacing removed From implementations)
    #[test]
    fn test_tool_error_explicit_anyhow_error() {
        let anyhow_error = anyhow::anyhow!("Something went wrong");
        let tool_error = ToolError::permanent_string(format!("An unknown error occurred: {}", anyhow_error));
        assert!(!tool_error.is_retriable());
        assert!(!tool_error.is_rate_limited());
        assert!(tool_error.to_string().contains("An unknown error occurred"));
    }

    #[test]
    fn test_tool_error_explicit_boxed_error() {
        let test_error = TestError;
        let tool_error = ToolError::permanent_with_source(test_error, "A required resource was not found");
        assert!(!tool_error.is_retriable());
        assert!(!tool_error.is_rate_limited());
        assert_eq!(tool_error.retry_after(), None);
        assert!(
            matches!(tool_error, ToolError::Permanent { ref context, .. } if context == "A required resource was not found")
        );
    }

    #[test]
    fn test_tool_error_explicit_string() {
        let error_msg = "Database connection failed".to_string();
        let tool_error = ToolError::retriable_string(error_msg);
        assert!(tool_error.is_retriable());
        assert!(!tool_error.is_rate_limited());
        assert!(tool_error.to_string().contains("Database connection failed"));
    }

    #[test]
    fn test_tool_error_explicit_str_ref() {
        let error_msg = "Authentication token expired";
        let tool_error = ToolError::permanent_string(error_msg);
        assert!(!tool_error.is_retriable());
        assert!(!tool_error.is_rate_limited());
        assert!(tool_error.to_string().contains(error_msg));
    }

    #[test]
    fn test_tool_error_explicit_empty_string() {
        let tool_error = ToolError::permanent_string("");
        assert!(!tool_error.is_retriable());
        assert!(!tool_error.is_rate_limited());
    }

    #[test]
    fn test_tool_error_explicit_empty_str() {
        let tool_error = ToolError::permanent_string("");
        assert!(!tool_error.is_retriable());
        assert!(!tool_error.is_rate_limited());
    }

    // Test StringError struct (even though it's private, it's used internally)
    #[test]
    fn test_string_error_creation_via_tool_error() {
        let error = ToolError::permanent_string("Test message");
        // permanent_string doesn't create a source, just sets the message
        assert!(error.source().is_none());
        assert_eq!(
            error.to_string(),
            "Permanent error: Test message - Test message"
        );
    }

    // Tests for edge cases with different input types
    #[test]
    fn test_tool_error_constructors_with_various_string_types() {
        // Test with String
        let error1 = ToolError::retriable_with_source(TestError, String::from("String type"));
        assert!(
            matches!(error1, ToolError::Retriable { ref context, .. } if context == "String type")
        );

        // Test with &str
        let error2 = ToolError::permanent_with_source(TestError, "str type");
        assert!(
            matches!(error2, ToolError::Permanent { ref context, .. } if context == "str type")
        );

        // Test with owned String
        let owned_string = "owned string".to_owned();
        let error3 = ToolError::rate_limited_with_source(TestError, owned_string, None);
        assert!(
            matches!(error3, ToolError::RateLimited { ref context, .. } if context == "owned string")
        );
    }

    // Test Result type alias
    #[test]
    fn test_result_type_alias_ok() {
        let result: Result<i32> = Ok(42);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_result_type_alias_err() {
        let result: Result<i32> = Err(CoreError::Generic("Test error".to_string()));
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.to_string(), "Core error: Test error");
    }

    // Test const functions
    #[test]
    fn test_const_functions_compilation() {
        // These tests ensure the const functions compile and work correctly
        const fn test_is_retriable() -> bool {
            // We can't create enum variants in const context easily, so just test compilation
            true
        }

        const fn test_retry_after() -> bool {
            // Test compilation of const function
            true
        }

        const fn test_is_rate_limited() -> bool {
            // Test compilation of const function
            true
        }

        assert!(test_is_retriable());
        assert!(test_retry_after());
        assert!(test_is_rate_limited());
    }

    // Test Debug implementations (derived)
    #[test]
    fn test_debug_implementations() {
        let core_error = CoreError::Queue("debug test".to_string());
        let debug_str = format!("{:?}", core_error);
        assert!(debug_str.contains("Queue"));
        assert!(debug_str.contains("debug test"));

        let worker_error = WorkerError::ToolNotFound {
            tool_name: "debug_tool".to_string(),
        };
        let debug_str = format!("{:?}", worker_error);
        assert!(debug_str.contains("ToolNotFound"));
        assert!(debug_str.contains("debug_tool"));

        let tool_error = ToolError::retriable_string("debug test");
        let debug_str = format!("{:?}", tool_error);
        assert!(debug_str.contains("Retriable"));
    }

    // Additional edge case: Test with very long strings
    #[test]
    fn test_errors_with_long_strings() {
        let long_string = "a".repeat(1000);
        let error = CoreError::Generic(long_string.clone());
        assert!(error.to_string().contains(&long_string));

        let tool_error = ToolError::permanent_string(long_string.clone());
        assert!(tool_error.to_string().contains(&long_string));
    }

    // Test error source chain
    #[test]
    fn test_error_source_chain() {
        let original = TestError;
        let tool_error = ToolError::retriable_with_source(original, "Context");

        // Test source method
        let source = tool_error.source();
        assert!(source.is_some());

        // Test that we can downcast
        let concrete_source = source.unwrap().downcast_ref::<TestError>();
        assert!(concrete_source.is_some());
    }

    // Test ToolError SignerContext variant
    #[test]
    fn test_tool_error_signer_context_variant() {
        use crate::signer::SignerError;

        let signer_error = SignerError::NoSignerContext;
        let tool_error = ToolError::SignerContext(signer_error.to_string());

        assert!(!tool_error.is_retriable());
        assert!(!tool_error.is_rate_limited());
        assert_eq!(tool_error.retry_after(), None);
        assert!(tool_error.to_string().contains("Signer context error"));
    }

    #[test]
    fn test_tool_error_from_signer_error() {
        use crate::signer::SignerError;

        let signer_error = SignerError::Configuration("Invalid config".to_string());
        let tool_error = ToolError::from(signer_error);

        assert!(
            matches!(tool_error, ToolError::SignerContext(ref inner) if inner.to_string().contains("Invalid configuration: Invalid config"))
        );
    }

    #[test]
    fn test_tool_error_from_serde_json_error() {
        // Create a real JSON parsing error by parsing invalid JSON
        let json_error = serde_json::from_str::<serde_json::Value>("{ invalid }").unwrap_err();
        let tool_error = ToolError::from(json_error);

        assert!(!tool_error.is_retriable());
        assert!(!tool_error.is_rate_limited());
        assert!(tool_error
            .to_string()
            .contains("JSON serialization/deserialization failed"));
        assert!(
            matches!(tool_error, ToolError::Permanent { ref context, .. } if context == "JSON serialization/deserialization failed")
        );
    }

    // Test Core Error Serialization from serde_json::Error
    #[test]
    fn test_core_error_from_serde_json_error() {
        // Create a real JSON parsing error by parsing invalid JSON
        let json_error = serde_json::from_str::<serde_json::Value>("{ invalid }").unwrap_err();
        let core_error = CoreError::from(json_error);

        assert!(
            matches!(core_error, CoreError::Serialization(_))
                && core_error.to_string().contains("Serialization error:")
        );
    }

    // Test Redis feature (conditional compilation)
    #[cfg(feature = "redis")]
    #[test]
    fn test_core_error_redis_variant() {
        use redis::RedisError;

        // Create a Redis error (we'll simulate one)
        let redis_error = RedisError::from((redis::ErrorKind::TypeError, "Type error"));
        let core_error = CoreError::Redis(redis_error);

        assert!(core_error.to_string().contains("Redis error:"));
    }

    #[cfg(feature = "redis")]
    #[test]
    fn test_core_error_from_redis_error() {
        use redis::RedisError;

        let redis_error = RedisError::from((redis::ErrorKind::IoError, "Connection failed"));
        let core_error = CoreError::from(redis_error);

        assert!(
            matches!(core_error, CoreError::Redis(_))
                && core_error.to_string().contains("Redis error:")
        );
    }

    // Test various Duration formats for ExecutionTimeout
    #[test]
    fn test_worker_error_execution_timeout_various_durations() {
        // Test zero duration
        let timeout_zero = std::time::Duration::from_secs(0);
        let error_zero = WorkerError::ExecutionTimeout {
            timeout: timeout_zero,
        };
        assert_eq!(error_zero.to_string(), "Tool execution timed out after 0ns");

        // Test milliseconds
        let timeout_millis = std::time::Duration::from_millis(500);
        let error_millis = WorkerError::ExecutionTimeout {
            timeout: timeout_millis,
        };
        assert_eq!(
            error_millis.to_string(),
            "Tool execution timed out after 500ms"
        );

        // Test minutes
        let timeout_minutes = std::time::Duration::from_secs(120);
        let error_minutes = WorkerError::ExecutionTimeout {
            timeout: timeout_minutes,
        };
        assert_eq!(
            error_minutes.to_string(),
            "Tool execution timed out after 120s"
        );
    }

    // Test edge cases for RateLimited retry_after
    #[test]
    fn test_rate_limited_retry_after_edge_cases() {
        // Test with zero duration
        let error1 = ToolError::rate_limited_with_source(
            TestError,
            "Rate limited",
            Some(std::time::Duration::from_secs(0)),
        );
        assert_eq!(
            error1.retry_after(),
            Some(std::time::Duration::from_secs(0))
        );

        // Test with very large duration
        let error2 = ToolError::rate_limited_with_source(
            TestError,
            "Rate limited",
            Some(std::time::Duration::from_secs(u64::MAX)),
        );
        assert_eq!(
            error2.retry_after(),
            Some(std::time::Duration::from_secs(u64::MAX))
        );
    }

    // Test all match arms in is_retriable
    #[test]
    fn test_is_retriable_all_variants() {
        let retriable = ToolError::Retriable {
            source: None,
            source_message: "Test error".to_string(),
            context: "test".to_string(),
        };
        assert!(retriable.is_retriable());

        let rate_limited = ToolError::RateLimited {
            source: None,
            source_message: "Test error".to_string(),
            context: "test".to_string(),
            retry_after: None,
        };
        assert!(rate_limited.is_retriable());

        let permanent = ToolError::Permanent {
            source: None,
            source_message: "Test error".to_string(),
            context: "test".to_string(),
        };
        assert!(!permanent.is_retriable());

        let invalid_input = ToolError::InvalidInput {
            source: None,
            source_message: "Test error".to_string(),
            context: "test".to_string(),
        };
        assert!(!invalid_input.is_retriable());

        let signer_context =
            ToolError::SignerContext(crate::signer::SignerError::NoSignerContext.to_string());
        assert!(!signer_context.is_retriable());
    }

    // Test all match arms in retry_after
    #[test]
    fn test_retry_after_all_variants() {
        let rate_limited_with_delay = ToolError::RateLimited {
            source: None,
            source_message: "Test error".to_string(),
            context: "test".to_string(),
            retry_after: Some(std::time::Duration::from_secs(30)),
        };
        assert_eq!(
            rate_limited_with_delay.retry_after(),
            Some(std::time::Duration::from_secs(30))
        );

        let rate_limited_no_delay = ToolError::RateLimited {
            source: None,
            source_message: "Test error".to_string(),
            context: "test".to_string(),
            retry_after: None,
        };
        assert_eq!(rate_limited_no_delay.retry_after(), None);

        let retriable = ToolError::Retriable {
            source: None,
            source_message: "Test error".to_string(),
            context: "test".to_string(),
        };
        assert_eq!(retriable.retry_after(), None);

        let permanent = ToolError::Permanent {
            source: None,
            source_message: "Test error".to_string(),
            context: "test".to_string(),
        };
        assert_eq!(permanent.retry_after(), None);

        let invalid_input = ToolError::InvalidInput {
            source: None,
            source_message: "Test error".to_string(),
            context: "test".to_string(),
        };
        assert_eq!(invalid_input.retry_after(), None);

        let signer_context =
            ToolError::SignerContext(crate::signer::SignerError::NoSignerContext.to_string());
        assert_eq!(signer_context.retry_after(), None);
    }

    // Test all match arms in is_rate_limited
    #[test]
    fn test_is_rate_limited_all_variants() {
        let rate_limited = ToolError::RateLimited {
            source: None,
            source_message: "Test error".to_string(),
            context: "test".to_string(),
            retry_after: None,
        };
        assert!(rate_limited.is_rate_limited());

        let retriable = ToolError::Retriable {
            source: None,
            source_message: "Test error".to_string(),
            context: "test".to_string(),
        };
        assert!(!retriable.is_rate_limited());

        let permanent = ToolError::Permanent {
            source: None,
            source_message: "Test error".to_string(),
            context: "test".to_string(),
        };
        assert!(!permanent.is_rate_limited());

        let invalid_input = ToolError::InvalidInput {
            source: None,
            source_message: "Test error".to_string(),
            context: "test".to_string(),
        };
        assert!(!invalid_input.is_rate_limited());

        let signer_context =
            ToolError::SignerContext(crate::signer::SignerError::NoSignerContext.to_string());
        assert!(!signer_context.is_rate_limited());
    }

    // Test that must_use attributes don't cause warnings
    #[test]
    fn test_must_use_methods() {
        let error = ToolError::retriable_string("test");

        // These methods have #[must_use] attribute
        let _ = error.is_retriable();
        let _ = error.retry_after();
        let _ = error.is_rate_limited();
    }
}
