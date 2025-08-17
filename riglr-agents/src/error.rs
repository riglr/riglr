//! Error types for the riglr-agents system.
//!
//! This module provides comprehensive error handling that integrates with
//! riglr-core's error system while adding agent-specific error types.

use riglr_core::ToolError;
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
}
