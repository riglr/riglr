//! Error types for the riglr-agents system.
//!
//! This module provides comprehensive error handling that integrates with
//! riglr-core's error system while adding agent-specific error types.

use riglr_core::{error::WorkerError, ToolError};
use thiserror::Error;

/// Main error type for riglr-agents operations.
#[derive(Error, Debug)]
pub enum AgentError {
    /// Agent not found in registry
    #[error("Agent '{agent_id}' not found in registry")]
    AgentNotFound {
        /// The identifier of the agent that was not found
        agent_id: String,
    },

    /// No suitable agent found for task
    #[error("No agent found capable of handling task type '{task_type}'")]
    NoSuitableAgent {
        /// The type of task that no agent could handle
        task_type: String,
    },

    /// Agent is not available
    #[error("Agent '{agent_id}' is not available (status: {status})")]
    AgentUnavailable {
        /// The identifier of the unavailable agent
        agent_id: String,
        /// The current status of the agent
        status: String,
    },

    /// Task execution failed
    #[error("Task execution failed: {message}")]
    TaskExecution {
        /// The error message describing the failure
        message: String,
        /// The underlying cause of the execution failure
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Task timeout
    #[error("Task '{task_id}' timed out after {duration:?}")]
    TaskTimeout {
        /// The identifier of the task that timed out
        task_id: String,
        /// The duration after which the task timed out
        duration: std::time::Duration,
    },

    /// Task cancelled
    #[error("Task '{task_id}' was cancelled: {reason}")]
    TaskCancelled {
        /// The identifier of the cancelled task
        task_id: String,
        /// The reason for the cancellation
        reason: String,
    },

    /// Invalid routing rule
    #[error("Invalid routing rule: {rule}")]
    InvalidRoutingRule {
        /// The invalid routing rule that was provided
        rule: String,
    },

    /// Communication error
    #[error("Communication error: {message}")]
    Communication {
        /// The error message describing the communication failure
        message: String,
        /// The underlying cause of the communication error
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Message delivery failed
    #[error("Failed to deliver message '{message_id}' to agent '{agent_id}'")]
    MessageDeliveryFailed {
        /// The identifier of the message that failed to deliver
        message_id: String,
        /// The identifier of the target agent
        agent_id: String,
    },

    /// Registry operation failed
    #[error("Registry operation failed: {operation}")]
    Registry {
        /// The registry operation that failed
        operation: String,
        /// The underlying cause of the registry operation failure
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Dispatcher error
    #[error("Dispatcher error: {message}")]
    Dispatcher {
        /// The error message describing the dispatcher failure
        message: String,
        /// The underlying cause of the dispatcher error
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Configuration error
    #[error("Configuration error: {message}")]
    Configuration {
        /// The error message describing the configuration issue
        message: String,
    },

    /// Serialization error
    #[error("Serialization error")]
    Serialization {
        /// The underlying JSON serialization error
        #[from]
        source: serde_json::Error,
    },

    /// Tool error (from riglr-core)
    #[error("Tool error")]
    Tool {
        /// The underlying tool error from riglr-core
        #[from]
        source: ToolError,
    },

    /// Generic error
    #[error("Agent system error: {message}")]
    Generic {
        /// The error message describing the generic failure
        message: String,
        /// The underlying cause of the generic error
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
}

impl AgentError {
    /// Create an agent not found error.
    pub fn agent_not_found(agent_id: impl Into<String>) -> Self {
        Self::AgentNotFound {
            agent_id: agent_id.into(),
        }
    }

    /// Create a no suitable agent error.
    pub fn no_suitable_agent(task_type: impl Into<String>) -> Self {
        Self::NoSuitableAgent {
            task_type: task_type.into(),
        }
    }

    /// Create an agent unavailable error.
    pub fn agent_unavailable(agent_id: impl Into<String>, status: impl Into<String>) -> Self {
        Self::AgentUnavailable {
            agent_id: agent_id.into(),
            status: status.into(),
        }
    }

    /// Create a task execution error.
    pub fn task_execution(message: impl Into<String>) -> Self {
        Self::TaskExecution {
            message: message.into(),
            source: None,
        }
    }

    /// Create a task execution error with source.
    pub fn task_execution_with_source<E>(message: impl Into<String>, source: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        Self::TaskExecution {
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }

    /// Create a task timeout error.
    pub fn task_timeout(task_id: impl Into<String>, duration: std::time::Duration) -> Self {
        Self::TaskTimeout {
            task_id: task_id.into(),
            duration,
        }
    }

    /// Create a task cancelled error.
    pub fn task_cancelled(task_id: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::TaskCancelled {
            task_id: task_id.into(),
            reason: reason.into(),
        }
    }

    /// Create an invalid routing rule error.
    pub fn invalid_routing_rule(rule: impl Into<String>) -> Self {
        Self::InvalidRoutingRule { rule: rule.into() }
    }

    /// Create a communication error.
    pub fn communication(message: impl Into<String>) -> Self {
        Self::Communication {
            message: message.into(),
            source: None,
        }
    }

    /// Create a communication error with source.
    pub fn communication_with_source<E>(message: impl Into<String>, source: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        Self::Communication {
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }

    /// Create a message delivery failed error.
    pub fn message_delivery_failed(
        message_id: impl Into<String>,
        agent_id: impl Into<String>,
    ) -> Self {
        Self::MessageDeliveryFailed {
            message_id: message_id.into(),
            agent_id: agent_id.into(),
        }
    }

    /// Create a registry error.
    pub fn registry(operation: impl Into<String>) -> Self {
        Self::Registry {
            operation: operation.into(),
            source: None,
        }
    }

    /// Create a registry error with source.
    pub fn registry_with_source<E>(operation: impl Into<String>, source: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        Self::Registry {
            operation: operation.into(),
            source: Some(Box::new(source)),
        }
    }

    /// Create a dispatcher error.
    pub fn dispatcher(message: impl Into<String>) -> Self {
        Self::Dispatcher {
            message: message.into(),
            source: None,
        }
    }

    /// Create a dispatcher error with source.
    pub fn dispatcher_with_source<E>(message: impl Into<String>, source: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        Self::Dispatcher {
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }

    /// Create a configuration error.
    pub fn configuration(message: impl Into<String>) -> Self {
        Self::Configuration {
            message: message.into(),
        }
    }

    /// Create a generic error.
    pub fn generic(message: impl Into<String>) -> Self {
        Self::Generic {
            message: message.into(),
            source: None,
        }
    }

    /// Create a generic error with source.
    pub fn generic_with_source<E>(message: impl Into<String>, source: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        Self::Generic {
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }

    /// Check if this error is retriable.
    ///
    /// Some agent errors represent temporary conditions that may succeed on retry.
    pub fn is_retriable(&self) -> bool {
        match self {
            // Network/communication issues are typically retriable
            AgentError::Communication { .. } => true,
            AgentError::MessageDeliveryFailed { .. } => true,

            // Agent availability might change
            AgentError::AgentUnavailable { .. } => true,

            // Task timeouts might succeed with more time
            AgentError::TaskTimeout { .. } => true,

            // Registry operations might succeed on retry
            AgentError::Registry { .. } => true,

            // Some dispatcher errors might be retriable
            AgentError::Dispatcher { .. } => false, // Generally configuration issues

            // Tool errors delegate to ToolError's retriable logic
            AgentError::Tool { source } => source.is_retriable(),

            // These are permanent failures
            AgentError::AgentNotFound { .. } => false,
            AgentError::NoSuitableAgent { .. } => false,
            AgentError::TaskCancelled { .. } => false,
            AgentError::InvalidRoutingRule { .. } => false,
            AgentError::Configuration { .. } => false,
            AgentError::Serialization { .. } => false,

            // Task execution and generic errors depend on context
            AgentError::TaskExecution { .. } => false, // Generally permanent
            AgentError::Generic { .. } => false,       // Conservative default
        }
    }

    /// Get the retry delay for retriable errors.
    pub fn retry_delay(&self) -> Option<std::time::Duration> {
        match self {
            AgentError::Tool { source } => source.retry_after(),
            AgentError::Communication { .. } => Some(std::time::Duration::from_secs(1)),
            AgentError::MessageDeliveryFailed { .. } => Some(std::time::Duration::from_millis(500)),
            AgentError::Registry { .. } => Some(std::time::Duration::from_millis(100)),
            _ => None,
        }
    }
}

/// Convert AgentError to ToolError for integration with riglr-core systems.
impl From<AgentError> for ToolError {
    fn from(err: AgentError) -> Self {
        match err {
            AgentError::Tool { source } => source,
            err if err.is_retriable() => {
                if let Some(delay) = err.retry_delay() {
                    ToolError::rate_limited_with_source(err, "Agent error", Some(delay))
                } else {
                    ToolError::retriable_with_source(err, "Agent error")
                }
            }
            err => ToolError::permanent_with_source(err, "Agent error"),
        }
    }
}

// Common error conversions
impl From<String> for AgentError {
    fn from(msg: String) -> Self {
        AgentError::generic(msg)
    }
}

impl From<&str> for AgentError {
    fn from(msg: &str) -> Self {
        AgentError::generic(msg.to_string())
    }
}

impl From<tokio::time::error::Elapsed> for AgentError {
    fn from(err: tokio::time::error::Elapsed) -> Self {
        AgentError::generic_with_source("Operation timed out", err)
    }
}

impl From<WorkerError> for AgentError {
    fn from(err: WorkerError) -> Self {
        AgentError::task_execution_with_source("Worker error", err)
    }
}

impl From<rig::completion::PromptError> for AgentError {
    fn from(err: rig::completion::PromptError) -> Self {
        AgentError::task_execution_with_source("LLM prompt error", err)
    }
}

/// Result type alias for riglr-agents operations.
pub type Result<T> = std::result::Result<T, AgentError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let agent_error = AgentError::agent_not_found("test-agent");
        assert!(matches!(agent_error, AgentError::AgentNotFound { .. }));
        assert!(!agent_error.is_retriable());

        let comm_error = AgentError::communication("Network timeout");
        assert!(matches!(comm_error, AgentError::Communication { .. }));
        assert!(comm_error.is_retriable());
    }

    #[test]
    fn test_error_retriability() {
        // Retriable errors
        assert!(AgentError::communication("timeout").is_retriable());
        assert!(AgentError::agent_unavailable("agent", "busy").is_retriable());
        assert!(
            AgentError::task_timeout("task", std::time::Duration::from_secs(30)).is_retriable()
        );

        // Non-retriable errors
        assert!(!AgentError::agent_not_found("agent").is_retriable());
        assert!(!AgentError::no_suitable_agent("trading").is_retriable());
        assert!(!AgentError::configuration("invalid config").is_retriable());
    }

    #[test]
    fn test_error_conversion_to_tool_error() {
        let agent_error = AgentError::communication("Network error");
        let tool_error: ToolError = agent_error.into();
        assert!(tool_error.is_retriable());

        let permanent_error = AgentError::agent_not_found("missing-agent");
        let tool_error: ToolError = permanent_error.into();
        assert!(!tool_error.is_retriable());
    }

    #[test]
    fn test_error_with_source() {
        use std::error::Error;
        #[derive(Debug, thiserror::Error)]
        #[error("Source error")]
        struct SourceError;

        let error = AgentError::task_execution_with_source("Task failed", SourceError);
        assert!(error.source().is_some());
        assert!(error
            .source()
            .unwrap()
            .downcast_ref::<SourceError>()
            .is_some());
    }

    #[test]
    fn test_retry_delay() {
        let comm_error = AgentError::communication("timeout");
        assert!(comm_error.retry_delay().is_some());

        let not_found_error = AgentError::agent_not_found("agent");
        assert!(not_found_error.retry_delay().is_none());
    }

    #[test]
    fn test_string_conversions() {
        let error: AgentError = "test error".into();
        assert!(matches!(error, AgentError::Generic { .. }));

        let error: AgentError = "test error".to_string().into();
        assert!(matches!(error, AgentError::Generic { .. }));
    }

    // === Additional comprehensive tests for 100% coverage ===

    #[test]
    fn test_agent_not_found_constructor() {
        let error = AgentError::agent_not_found("test-agent-123");
        match &error {
            AgentError::AgentNotFound { agent_id } => {
                assert_eq!(agent_id, "test-agent-123");
            }
            _ => panic!("Expected AgentNotFound variant"),
        }
        assert_eq!(
            format!("{}", error),
            "Agent 'test-agent-123' not found in registry"
        );
    }

    #[test]
    fn test_no_suitable_agent_constructor() {
        let error = AgentError::no_suitable_agent("data_processing");
        match &error {
            AgentError::NoSuitableAgent { task_type } => {
                assert_eq!(task_type, "data_processing");
            }
            _ => panic!("Expected NoSuitableAgent variant"),
        }
        assert_eq!(
            format!("{}", error),
            "No agent found capable of handling task type 'data_processing'"
        );
    }

    #[test]
    fn test_agent_unavailable_constructor() {
        let error = AgentError::agent_unavailable("agent-456", "offline");
        match &error {
            AgentError::AgentUnavailable { agent_id, status } => {
                assert_eq!(agent_id, "agent-456");
                assert_eq!(status, "offline");
            }
            _ => panic!("Expected AgentUnavailable variant"),
        }
        assert_eq!(
            format!("{}", error),
            "Agent 'agent-456' is not available (status: offline)"
        );
    }

    #[test]
    fn test_task_execution_constructor() {
        let error = AgentError::task_execution("Processing failed");
        match &error {
            AgentError::TaskExecution { message, source } => {
                assert_eq!(message, "Processing failed");
                assert!(source.is_none());
            }
            _ => panic!("Expected TaskExecution variant"),
        }
        assert_eq!(
            format!("{}", error),
            "Task execution failed: Processing failed"
        );
    }

    #[test]
    fn test_task_execution_with_source_constructor() {
        #[derive(Debug, thiserror::Error)]
        #[error("Custom source error")]
        struct CustomError;

        let error = AgentError::task_execution_with_source("Task failed", CustomError);
        match &error {
            AgentError::TaskExecution { message, source } => {
                assert_eq!(message, "Task failed");
                assert!(source.is_some());
            }
            _ => panic!("Expected TaskExecution variant"),
        }
    }

    #[test]
    fn test_task_timeout_constructor() {
        let duration = std::time::Duration::from_secs(30);
        let error = AgentError::task_timeout("task-789", duration);
        match &error {
            AgentError::TaskTimeout {
                task_id,
                duration: timeout_duration,
            } => {
                assert_eq!(task_id, "task-789");
                assert_eq!(timeout_duration, &duration);
            }
            _ => panic!("Expected TaskTimeout variant"),
        }
        assert_eq!(format!("{}", error), "Task 'task-789' timed out after 30s");
    }

    #[test]
    fn test_task_cancelled_constructor() {
        let error = AgentError::task_cancelled("task-101", "User requested cancellation");
        match &error {
            AgentError::TaskCancelled { task_id, reason } => {
                assert_eq!(task_id, "task-101");
                assert_eq!(reason, "User requested cancellation");
            }
            _ => panic!("Expected TaskCancelled variant"),
        }
        assert_eq!(
            format!("{}", error),
            "Task 'task-101' was cancelled: User requested cancellation"
        );
    }

    #[test]
    fn test_invalid_routing_rule_constructor() {
        let error = AgentError::invalid_routing_rule("invalid_pattern_*");
        match &error {
            AgentError::InvalidRoutingRule { rule } => {
                assert_eq!(rule, "invalid_pattern_*");
            }
            _ => panic!("Expected InvalidRoutingRule variant"),
        }
        assert_eq!(
            format!("{}", error),
            "Invalid routing rule: invalid_pattern_*"
        );
    }

    #[test]
    fn test_communication_constructor() {
        let error = AgentError::communication("Connection refused");
        match &error {
            AgentError::Communication { message, source } => {
                assert_eq!(message, "Connection refused");
                assert!(source.is_none());
            }
            _ => panic!("Expected Communication variant"),
        }
        assert_eq!(
            format!("{}", error),
            "Communication error: Connection refused"
        );
    }

    #[test]
    fn test_communication_with_source_constructor() {
        #[derive(Debug, thiserror::Error)]
        #[error("Network timeout")]
        struct NetworkError;

        let error = AgentError::communication_with_source("Failed to connect", NetworkError);
        match &error {
            AgentError::Communication { message, source } => {
                assert_eq!(message, "Failed to connect");
                assert!(source.is_some());
            }
            _ => panic!("Expected Communication variant"),
        }
    }

    #[test]
    fn test_message_delivery_failed_constructor() {
        let error = AgentError::message_delivery_failed("msg-123", "agent-456");
        match &error {
            AgentError::MessageDeliveryFailed {
                message_id,
                agent_id,
            } => {
                assert_eq!(message_id, "msg-123");
                assert_eq!(agent_id, "agent-456");
            }
            _ => panic!("Expected MessageDeliveryFailed variant"),
        }
        assert_eq!(
            format!("{}", error),
            "Failed to deliver message 'msg-123' to agent 'agent-456'"
        );
    }

    #[test]
    fn test_registry_constructor() {
        let error = AgentError::registry("register_agent");
        match &error {
            AgentError::Registry { operation, source } => {
                assert_eq!(operation, "register_agent");
                assert!(source.is_none());
            }
            _ => panic!("Expected Registry variant"),
        }
        assert_eq!(
            format!("{}", error),
            "Registry operation failed: register_agent"
        );
    }

    #[test]
    fn test_registry_with_source_constructor() {
        #[derive(Debug, thiserror::Error)]
        #[error("Database error")]
        struct DbError;

        let error = AgentError::registry_with_source("save_agent", DbError);
        match &error {
            AgentError::Registry { operation, source } => {
                assert_eq!(operation, "save_agent");
                assert!(source.is_some());
            }
            _ => panic!("Expected Registry variant"),
        }
    }

    #[test]
    fn test_dispatcher_constructor() {
        let error = AgentError::dispatcher("Route not found");
        match &error {
            AgentError::Dispatcher { message, source } => {
                assert_eq!(message, "Route not found");
                assert!(source.is_none());
            }
            _ => panic!("Expected Dispatcher variant"),
        }
        assert_eq!(format!("{}", error), "Dispatcher error: Route not found");
    }

    #[test]
    fn test_dispatcher_with_source_constructor() {
        #[derive(Debug, thiserror::Error)]
        #[error("Config parse error")]
        struct ParseError;

        let error = AgentError::dispatcher_with_source("Invalid configuration", ParseError);
        match &error {
            AgentError::Dispatcher { message, source } => {
                assert_eq!(message, "Invalid configuration");
                assert!(source.is_some());
            }
            _ => panic!("Expected Dispatcher variant"),
        }
    }

    #[test]
    fn test_configuration_constructor() {
        let error = AgentError::configuration("Missing required field 'agent_type'");
        match &error {
            AgentError::Configuration { message } => {
                assert_eq!(message, "Missing required field 'agent_type'");
            }
            _ => panic!("Expected Configuration variant"),
        }
        assert_eq!(
            format!("{}", error),
            "Configuration error: Missing required field 'agent_type'"
        );
    }

    #[test]
    fn test_generic_constructor() {
        let error = AgentError::generic("Something went wrong");
        match &error {
            AgentError::Generic { message, source } => {
                assert_eq!(message, "Something went wrong");
                assert!(source.is_none());
            }
            _ => panic!("Expected Generic variant"),
        }
        assert_eq!(
            format!("{}", error),
            "Agent system error: Something went wrong"
        );
    }

    #[test]
    fn test_generic_with_source_constructor() {
        #[derive(Debug, thiserror::Error)]
        #[error("Unknown error")]
        struct UnknownError;

        let error = AgentError::generic_with_source("Unexpected failure", UnknownError);
        match &error {
            AgentError::Generic { message, source } => {
                assert_eq!(message, "Unexpected failure");
                assert!(source.is_some());
            }
            _ => panic!("Expected Generic variant"),
        }
    }

    #[test]
    fn test_serialization_error_from_conversion() {
        let json_error = serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err();
        let agent_error: AgentError = json_error.into();
        match agent_error {
            AgentError::Serialization { .. } => {
                // Success, the conversion worked
            }
            _ => panic!("Expected Serialization variant"),
        }
        assert_eq!(format!("{}", agent_error), "Serialization error");
    }

    #[test]
    fn test_tool_error_from_conversion() {
        let tool_error = ToolError::permanent_string("Test tool error");
        let agent_error: AgentError = tool_error.into();
        match agent_error {
            AgentError::Tool { .. } => {
                // Success, the conversion worked
            }
            _ => panic!("Expected Tool variant"),
        }
        assert_eq!(format!("{}", agent_error), "Tool error");
    }

    #[tokio::test]
    async fn test_tokio_elapsed_error_conversion() {
        // Create a real timeout error by timing out a slow operation
        let result = tokio::time::timeout(
            std::time::Duration::from_millis(1),
            tokio::time::sleep(std::time::Duration::from_millis(10)),
        )
        .await;

        if let Err(elapsed_error) = result {
            let agent_error: AgentError = elapsed_error.into();
            match agent_error {
                AgentError::Generic { message, source } => {
                    assert_eq!(message, "Operation timed out");
                    assert!(source.is_some());
                }
                _ => panic!("Expected Generic variant with timeout message"),
            }
        } else {
            panic!("Expected timeout error");
        }
    }

    #[test]
    fn test_is_retriable_all_variants() {
        // Test all retriable variants
        assert!(AgentError::communication("error").is_retriable());
        assert!(AgentError::message_delivery_failed("msg", "agent").is_retriable());
        assert!(AgentError::agent_unavailable("agent", "busy").is_retriable());
        assert!(AgentError::task_timeout("task", std::time::Duration::from_secs(1)).is_retriable());
        assert!(AgentError::registry("operation").is_retriable());

        // Test all non-retriable variants
        assert!(!AgentError::dispatcher("error").is_retriable());
        assert!(!AgentError::agent_not_found("agent").is_retriable());
        assert!(!AgentError::no_suitable_agent("task").is_retriable());
        assert!(!AgentError::task_cancelled("task", "reason").is_retriable());
        assert!(!AgentError::invalid_routing_rule("rule").is_retriable());
        assert!(!AgentError::configuration("error").is_retriable());
        assert!(!AgentError::task_execution("error").is_retriable());
        assert!(!AgentError::generic("error").is_retriable());

        // Test serialization error (should not be retriable)
        let json_error = serde_json::from_str::<serde_json::Value>("invalid").unwrap_err();
        let serialization_error: AgentError = json_error.into();
        assert!(!serialization_error.is_retriable());
    }

    #[test]
    fn test_tool_error_retriability_delegation() {
        // Test retriable tool error
        let retriable_tool_error = ToolError::retriable_string("Temporary tool failure");
        let agent_error = AgentError::Tool {
            source: retriable_tool_error,
        };
        assert!(agent_error.is_retriable());

        // Test permanent tool error
        let permanent_tool_error = ToolError::permanent_string("Permanent tool failure");
        let agent_error = AgentError::Tool {
            source: permanent_tool_error,
        };
        assert!(!agent_error.is_retriable());
    }

    #[test]
    fn test_retry_delay_all_variants() {
        // Test variants with specific retry delays
        let comm_error = AgentError::communication("timeout");
        assert_eq!(
            comm_error.retry_delay(),
            Some(std::time::Duration::from_secs(1))
        );

        let delivery_error = AgentError::message_delivery_failed("msg", "agent");
        assert_eq!(
            delivery_error.retry_delay(),
            Some(std::time::Duration::from_millis(500))
        );

        let registry_error = AgentError::registry("operation");
        assert_eq!(
            registry_error.retry_delay(),
            Some(std::time::Duration::from_millis(100))
        );

        // Test variants with no retry delay
        let agent_not_found = AgentError::agent_not_found("agent");
        assert_eq!(agent_not_found.retry_delay(), None);

        let no_suitable = AgentError::no_suitable_agent("task");
        assert_eq!(no_suitable.retry_delay(), None);

        let unavailable = AgentError::agent_unavailable("agent", "busy");
        assert_eq!(unavailable.retry_delay(), None);

        let timeout = AgentError::task_timeout("task", std::time::Duration::from_secs(1));
        assert_eq!(timeout.retry_delay(), None);

        let cancelled = AgentError::task_cancelled("task", "reason");
        assert_eq!(cancelled.retry_delay(), None);

        let invalid_rule = AgentError::invalid_routing_rule("rule");
        assert_eq!(invalid_rule.retry_delay(), None);

        let dispatcher_err = AgentError::dispatcher("error");
        assert_eq!(dispatcher_err.retry_delay(), None);

        let config_err = AgentError::configuration("error");
        assert_eq!(config_err.retry_delay(), None);

        let task_exec = AgentError::task_execution("error");
        assert_eq!(task_exec.retry_delay(), None);

        let generic_err = AgentError::generic("error");
        assert_eq!(generic_err.retry_delay(), None);
    }

    #[test]
    fn test_tool_error_retry_delay_delegation() {
        // Define a simple test error type
        #[derive(Debug)]
        struct TestError;
        impl std::fmt::Display for TestError {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "test error")
            }
        }
        impl std::error::Error for TestError {}

        // Test tool error with retry_after set explicitly
        let tool_error_with_delay = ToolError::rate_limited_with_source(
            TestError,
            "Tool rate limited",
            Some(std::time::Duration::from_secs(5)),
        );
        let agent_error = AgentError::Tool {
            source: tool_error_with_delay,
        };
        assert_eq!(
            agent_error.retry_delay(),
            Some(std::time::Duration::from_secs(5))
        );

        // Test tool error without retry_after (rate_limited_string creates None)
        let tool_error_rate_limited = ToolError::rate_limited_string("Tool rate limited");
        let agent_error = AgentError::Tool {
            source: tool_error_rate_limited,
        };
        assert_eq!(agent_error.retry_delay(), None);

        // Test retriable tool error without retry_after
        let tool_error_no_delay = ToolError::retriable_string("Tool error without delay");
        let agent_error = AgentError::Tool {
            source: tool_error_no_delay,
        };
        assert_eq!(agent_error.retry_delay(), None);
    }

    #[test]
    fn test_from_agent_error_to_tool_error_all_cases() {
        // Test Tool variant - should return the inner ToolError
        let original_tool_error = ToolError::permanent_string("Original tool error");
        let expected_format = format!("{}", original_tool_error);
        let agent_error = AgentError::Tool {
            source: original_tool_error,
        };
        let converted: ToolError = agent_error.into();
        assert_eq!(format!("{}", converted), expected_format);

        // Test retriable error with delay
        let comm_error = AgentError::communication("Network error");
        let converted: ToolError = comm_error.into();
        assert!(converted.is_retriable());
        assert_eq!(
            converted.retry_after(),
            Some(std::time::Duration::from_secs(1))
        );

        // Test retriable error without delay (should use retriable_with_source)
        let unavailable_error = AgentError::agent_unavailable("agent", "busy");
        let converted: ToolError = unavailable_error.into();
        assert!(converted.is_retriable());
        assert_eq!(converted.retry_after(), None);

        // Test non-retriable error
        let not_found_error = AgentError::agent_not_found("missing-agent");
        let converted: ToolError = not_found_error.into();
        assert!(!converted.is_retriable());
    }

    #[test]
    fn test_result_type_alias() {
        // Test that our Result type alias works correctly
        fn test_function() -> Result<String> {
            Ok("success".to_string())
        }

        fn test_function_error() -> Result<String> {
            Err(AgentError::agent_not_found("test"))
        }

        assert!(test_function().is_ok());
        assert!(test_function_error().is_err());
    }

    #[test]
    fn test_error_display_messages() {
        // Test all error variant display messages
        let agent_not_found = AgentError::agent_not_found("test-agent");
        assert_eq!(
            format!("{}", agent_not_found),
            "Agent 'test-agent' not found in registry"
        );

        let no_suitable = AgentError::no_suitable_agent("trading");
        assert_eq!(
            format!("{}", no_suitable),
            "No agent found capable of handling task type 'trading'"
        );

        let unavailable = AgentError::agent_unavailable("agent-1", "offline");
        assert_eq!(
            format!("{}", unavailable),
            "Agent 'agent-1' is not available (status: offline)"
        );

        let task_exec = AgentError::task_execution("Failed to process");
        assert_eq!(
            format!("{}", task_exec),
            "Task execution failed: Failed to process"
        );

        let timeout = AgentError::task_timeout("task-1", std::time::Duration::from_secs(30));
        assert_eq!(format!("{}", timeout), "Task 'task-1' timed out after 30s");

        let cancelled = AgentError::task_cancelled("task-2", "User cancelled");
        assert_eq!(
            format!("{}", cancelled),
            "Task 'task-2' was cancelled: User cancelled"
        );

        let invalid_rule = AgentError::invalid_routing_rule("bad-rule");
        assert_eq!(
            format!("{}", invalid_rule),
            "Invalid routing rule: bad-rule"
        );

        let communication = AgentError::communication("Connection failed");
        assert_eq!(
            format!("{}", communication),
            "Communication error: Connection failed"
        );

        let delivery_failed = AgentError::message_delivery_failed("msg-1", "agent-2");
        assert_eq!(
            format!("{}", delivery_failed),
            "Failed to deliver message 'msg-1' to agent 'agent-2'"
        );

        let registry = AgentError::registry("save_operation");
        assert_eq!(
            format!("{}", registry),
            "Registry operation failed: save_operation"
        );

        let dispatcher = AgentError::dispatcher("Route not found");
        assert_eq!(
            format!("{}", dispatcher),
            "Dispatcher error: Route not found"
        );

        let configuration = AgentError::configuration("Invalid config");
        assert_eq!(
            format!("{}", configuration),
            "Configuration error: Invalid config"
        );

        let generic = AgentError::generic("Something failed");
        assert_eq!(
            format!("{}", generic),
            "Agent system error: Something failed"
        );
    }

    #[test]
    fn test_error_source_chain() {
        use std::error::Error;

        #[derive(Debug, thiserror::Error)]
        #[error("Root cause")]
        struct RootError;

        let error = AgentError::task_execution_with_source("Task failed", RootError);

        // Test that source() returns the correct error
        assert!(error.source().is_some());
        let source = error.source().unwrap();
        assert_eq!(format!("{}", source), "Root cause");

        // Test downcast
        assert!(source.downcast_ref::<RootError>().is_some());
    }
}
