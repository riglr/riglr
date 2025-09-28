/// Core type definitions for the riglr-agents system.
///
/// This module provides the fundamental types used throughout the multi-agent
/// coordination system, including agent identifiers, task definitions, routing
/// rules, and messaging structures.
use core::convert::Infallible;
use core::fmt::{Display, Formatter, Result as FmtResult};
use core::str::FromStr;
use core::time::Duration;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Agent capability types with strong typing.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum CapabilityType {
    /// Custom capability type
    Custom(String),
    /// Market monitoring capabilities
    Monitoring,
    /// Portfolio management capabilities
    Portfolio,
    /// Research and analysis capabilities
    Research,
    /// Risk assessment and management capabilities
    RiskAnalysis,
    /// Tool calling capabilities for interacting with external systems
    ToolCalling,
    /// Trading-related capabilities
    Trading,
}

impl Display for CapabilityType {
    #[inline]
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        match *self {
            Self::Trading => write!(formatter, "trading"),
            Self::Research => write!(formatter, "research"),
            Self::RiskAnalysis => write!(formatter, "risk_analysis"),
            Self::Portfolio => write!(formatter, "portfolio"),
            Self::Monitoring => write!(formatter, "monitoring"),
            Self::ToolCalling => write!(formatter, "tool_calling"),
            Self::Custom(ref name) => write!(formatter, "{name}"),
        }
    }
}

impl FromStr for CapabilityType {
    type Err = Infallible;

    #[inline]
    fn from_str(input_string: &str) -> Result<Self, Self::Err> {
        Ok(match input_string {
            "trading" => Self::Trading,
            "research" => Self::Research,
            "risk_analysis" => Self::RiskAnalysis,
            "portfolio" => Self::Portfolio,
            "monitoring" => Self::Monitoring,
            "tool_calling" => Self::ToolCalling,
            custom => Self::Custom(custom.to_owned()),
        })
    }
}

impl PartialEq<&str> for CapabilityType {
    #[inline]
    fn eq(&self, other: &&str) -> bool {
        match *self {
            Self::Trading => *other == "trading",
            Self::Research => *other == "research",
            Self::RiskAnalysis => *other == "risk_analysis",
            Self::Portfolio => *other == "portfolio",
            Self::Monitoring => *other == "monitoring",
            Self::ToolCalling => *other == "tool_calling",
            Self::Custom(ref name) => name == *other,
        }
    }
}

impl PartialEq<str> for CapabilityType {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        match *self {
            Self::Trading => other == "trading",
            Self::Research => other == "research",
            Self::RiskAnalysis => other == "risk_analysis",
            Self::Portfolio => other == "portfolio",
            Self::Monitoring => other == "monitoring",
            Self::ToolCalling => other == "tool_calling",
            Self::Custom(ref name) => name == other,
        }
    }
}

/// Unique identifier for an agent in the system.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub struct AgentId(pub String);

impl AgentId {
    /// Get the inner string value.
    #[must_use]
    #[inline]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Generate a random agent ID.
    #[must_use]
    #[inline]
    pub fn generate() -> Self {
        Self(Uuid::new_v4().to_string())
    }

    /// Create a new agent ID from a string.
    #[must_use]
    #[inline]
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

impl From<String> for AgentId {
    #[inline]
    fn from(id: String) -> Self {
        Self(id)
    }
}

impl From<&str> for AgentId {
    #[inline]
    fn from(id: &str) -> Self {
        Self(id.to_owned())
    }
}

impl Display for AgentId {
    #[inline]
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        write!(formatter, "{}", self.0)
    }
}

/// Types of tasks that can be executed by agents.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum TaskType {
    /// Custom task type
    Custom(String),
    /// Market monitoring
    Monitoring,
    /// Portfolio management
    Portfolio,
    /// Research and analysis tasks
    Research,
    /// Risk assessment and management
    RiskAnalysis,
    /// Trading-related operations
    Trading,
}

impl TaskType {
    /// Check if this task type matches another (including wildcard matching).
    #[must_use]
    #[inline]
    pub fn matches(&self, other: &Self) -> bool {
        self == other
    }
}

impl Display for TaskType {
    #[inline]
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        match *self {
            Self::Trading => write!(formatter, "trading"),
            Self::Research => write!(formatter, "research"),
            Self::RiskAnalysis => write!(formatter, "risk_analysis"),
            Self::Portfolio => write!(formatter, "portfolio"),
            Self::Monitoring => write!(formatter, "monitoring"),
            Self::Custom(ref name) => write!(formatter, "custom:{name}"),
        }
    }
}

/// Priority levels for task execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
#[non_exhaustive]
pub enum Priority {
    /// Critical priority tasks (emergency)
    Critical = 4,
    /// High priority tasks
    High = 3,
    /// Low priority tasks
    Low = 1,
    /// Normal priority tasks
    #[default]
    Normal = 2,
}

/// A task to be executed by an agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Task {
    /// Timestamp when task was created
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Optional deadline for task completion
    pub deadline: Option<chrono::DateTime<chrono::Utc>>,
    /// Unique task identifier
    pub id: String,
    /// Number of retry attempts allowed
    pub max_retries: u32,
    /// Metadata for the task
    pub metadata: HashMap<String, serde_json::Value>,
    /// Task parameters
    pub parameters: serde_json::Value,
    /// Task priority
    pub priority: Priority,
    /// Current retry count
    pub retry_count: u32,
    /// Type of task
    pub task_type: TaskType,
    /// Maximum execution timeout
    pub timeout: Option<Duration>,
}

impl Task {
    /// Check if the task can be retried.
    #[must_use]
    #[inline]
    pub const fn can_retry(&self) -> bool {
        self.retry_count < self.max_retries
    }

    /// Increment the retry count.
    #[inline]
    pub const fn increment_retry(&mut self) {
        self.retry_count = self.retry_count.saturating_add(1);
    }

    /// Check if the task has exceeded its deadline.
    #[must_use]
    #[inline]
    pub fn is_past_deadline(&self) -> bool {
        self.deadline
            .is_some_and(|deadline| chrono::Utc::now() > deadline)
    }

    /// Create a new task with the given type and parameters.
    #[must_use]
    #[inline]
    pub fn new(task_type: TaskType, parameters: serde_json::Value) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            task_type,
            parameters,
            priority: Priority::default(),
            timeout: None,
            max_retries: 3,
            retry_count: 0,
            created_at: chrono::Utc::now(),
            deadline: None,
            metadata: HashMap::new(),
        }
    }

    /// Set the task deadline.
    #[must_use]
    #[inline]
    pub const fn with_deadline(mut self, deadline: chrono::DateTime<chrono::Utc>) -> Self {
        self.deadline = Some(deadline);
        self
    }

    /// Set the maximum retry count.
    #[must_use]
    #[inline]
    pub const fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }

    /// Add metadata to the task.
    #[must_use]
    #[inline]
    pub fn with_metadata(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }

    /// Set the task priority.
    #[must_use]
    #[inline]
    pub const fn with_priority(mut self, priority: Priority) -> Self {
        self.priority = priority;
        self
    }

    /// Set the task timeout.
    #[must_use]
    #[inline]
    pub const fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }
}

/// Result of task execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub enum TaskResult {
    /// Task was cancelled
    Cancelled {
        /// Cancellation reason
        reason: String,
    },
    /// Task failed with error
    Failure {
        /// Error message
        error: String,
        /// Whether the failure is retriable
        retriable: bool,
        /// Execution duration before failure
        duration: Duration,
    },
    /// Task completed successfully
    Success {
        /// Result data
        data: serde_json::Value,
        /// Optional transaction hash
        tx_hash: Option<String>,
        /// Execution duration
        duration: Duration,
    },
    /// Task timed out
    Timeout {
        /// Timeout duration
        duration: Duration,
    },
}

impl TaskResult {
    /// Create a cancelled task result.
    #[must_use]
    #[inline]
    pub const fn cancelled(reason: String) -> Self {
        Self::Cancelled { reason }
    }

    /// Get the data from a successful result.
    #[must_use]
    #[inline]
    pub const fn data(&self) -> Option<&serde_json::Value> {
        match *self {
            Self::Success { ref data, .. } => Some(data),
            Self::Failure { .. } | Self::Cancelled { .. } | Self::Timeout { .. } => None,
        }
    }

    /// Get the error message from a failed result.
    #[must_use]
    #[inline]
    pub fn error(&self) -> Option<&str> {
        match *self {
            Self::Failure { ref error, .. } => Some(error),
            Self::Success { .. } | Self::Cancelled { .. } | Self::Timeout { .. } => None,
        }
    }

    /// Create a failed task result.
    #[must_use]
    #[inline]
    pub const fn failure(error: String, retriable: bool, duration: Duration) -> Self {
        Self::Failure {
            error,
            retriable,
            duration,
        }
    }

    /// Check if the result represents a retriable failure.
    #[must_use]
    #[inline]
    pub const fn is_retriable(&self) -> bool {
        matches!(
            *self,
            Self::Failure {
                retriable: true,
                ..
            }
        )
    }

    /// Check if the result represents success.
    #[must_use]
    #[inline]
    pub const fn is_success(&self) -> bool {
        matches!(*self, Self::Success { .. })
    }

    /// Create a successful task result.
    #[must_use]
    #[inline]
    pub const fn success(
        data: serde_json::Value,
        tx_hash: Option<String>,
        duration: Duration,
    ) -> Self {
        Self::Success {
            data,
            tx_hash,
            duration,
        }
    }

    /// Create a timeout task result.
    #[must_use]
    #[inline]
    pub const fn timeout(duration: Duration) -> Self {
        Self::Timeout { duration }
    }
}

/// Message passed between agents.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct AgentMessage {
    /// Optional expiration time
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Source agent ID
    pub from: AgentId,
    /// Message ID
    pub id: String,
    /// Message type/topic
    pub message_type: String,
    /// Message metadata
    pub metadata: HashMap<String, serde_json::Value>,
    /// Message payload
    pub payload: serde_json::Value,
    /// Message priority
    pub priority: Priority,
    /// Timestamp when message was created
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Target agent ID (None for broadcast)
    pub to: Option<AgentId>,
}

impl AgentMessage {
    /// Create a broadcast message (no specific recipient).
    #[must_use]
    #[inline]
    pub fn broadcast(from: AgentId, message_type: String, payload: serde_json::Value) -> Self {
        Self::new(from, None, message_type, payload)
    }

    /// Check if the message has expired.
    #[must_use]
    #[inline]
    pub fn is_expired(&self) -> bool {
        self.expires_at
            .is_some_and(|expiry| chrono::Utc::now() > expiry)
    }

    /// Create a new message.
    #[must_use]
    #[inline]
    pub fn new(
        from: AgentId,
        to: Option<AgentId>,
        message_type: String,
        payload: serde_json::Value,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            from,
            to,
            message_type,
            payload,
            priority: Priority::default(),
            timestamp: chrono::Utc::now(),
            expires_at: None,
            metadata: HashMap::new(),
        }
    }

    /// Set message expiration.
    #[must_use]
    #[inline]
    pub const fn with_expiration(mut self, expires_at: chrono::DateTime<chrono::Utc>) -> Self {
        self.expires_at = Some(expires_at);
        self
    }

    /// Set message priority.
    #[must_use]
    #[inline]
    pub const fn with_priority(mut self, priority: Priority) -> Self {
        self.priority = priority;
        self
    }
}

/// Rules for routing tasks to agents.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub enum RoutingRule {
    /// Route to specific agent
    Agent(AgentId),
    /// Combination of rules (ALL must match)
    All(Vec<RoutingRule>),
    /// Alternative rules (ANY can match)
    Any(Vec<RoutingRule>),
    /// Route based on agent capability
    Capability(String),
    /// Custom routing logic
    Custom(String),
    /// Route to least loaded agent
    LeastLoaded,
    /// Route based on priority
    Priority(Priority),
    /// Round-robin routing among matching agents
    RoundRobin,
    /// Route based on task type
    TaskType(TaskType),
}

impl RoutingRule {
    /// Check if this rule matches the given task and agent context.
    #[must_use]
    #[inline]
    pub fn matches(&self, task: &Task, agent_id: &AgentId, capabilities: &[String]) -> bool {
        match *self {
            Self::TaskType(ref task_type) => task_type.matches(&task.task_type),
            Self::Capability(ref capability) => capabilities.contains(capability),
            Self::Agent(ref target_agent) => target_agent == agent_id,
            Self::Priority(priority) => task.priority >= priority,
            Self::All(ref rules) => rules
                .iter()
                .all(|rule| rule.matches(task, agent_id, capabilities)),
            Self::Any(ref rules) => rules
                .iter()
                .any(|rule| rule.matches(task, agent_id, capabilities)),
            // These routing strategies require external context
            Self::RoundRobin | Self::LeastLoaded | Self::Custom(_) => false,
        }
    }
}

/// Agent capability definition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Capability {
    /// Capability name
    pub name: String,
    /// Optional capability parameters
    pub parameters: HashMap<String, serde_json::Value>,
    /// Optional JSON schema for the capability
    #[serde(default)]
    pub schema: Option<serde_json::Value>,
    /// Capability version
    pub version: String,
}

impl Capability {
    /// Create a new capability.
    #[must_use]
    #[inline]
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            parameters: HashMap::new(),
            schema: None,
        }
    }

    /// Add a parameter to the capability.
    #[must_use]
    #[inline]
    pub fn with_parameter(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.parameters.insert(key.into(), value);
        self
    }

    /// Set the JSON schema for the capability.
    #[must_use]
    #[inline]
    pub fn with_schema(mut self, schema: serde_json::Value) -> Self {
        self.schema = Some(schema);
        self
    }
}

/// Agent status information.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct AgentStatus {
    /// Number of active tasks
    pub active_tasks: u32,
    /// Agent ID
    pub agent_id: AgentId,
    /// Agent capabilities
    pub capabilities: Vec<Capability>,
    /// Last heartbeat timestamp
    pub last_heartbeat: chrono::DateTime<chrono::Utc>,
    /// Agent load (0.0 to 1.0)
    pub load: f64,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
    /// Current status
    pub status: AgentState,
}

impl AgentStatus {
    /// Create a new agent status.
    #[must_use]
    #[inline]
    pub fn new(
        agent_id: AgentId,
        status: AgentState,
        active_tasks: u32,
        load: f64,
        capabilities: Vec<Capability>,
    ) -> Self {
        Self {
            agent_id,
            status,
            active_tasks,
            load,
            last_heartbeat: chrono::Utc::now(),
            capabilities,
            metadata: HashMap::new(),
        }
    }

    /// Create a new agent status with metadata.
    #[must_use]
    #[inline]
    pub fn with_metadata(mut self, metadata: HashMap<String, serde_json::Value>) -> Self {
        self.metadata = metadata;
        self
    }
}

/// Agent state enumeration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[non_exhaustive]
pub enum AgentState {
    /// Agent is active and ready to accept tasks
    Active,
    /// Agent is busy but can accept more tasks
    Busy,
    /// Agent is at capacity
    Full,
    /// Agent is idle
    #[default]
    Idle,
    /// Agent is in maintenance mode
    Maintenance,
    /// Agent is offline/unavailable
    Offline,
}

#[cfg(test)]
#[expect(clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_id_creation() {
        let id1 = AgentId::new("test-agent");
        let id2 = AgentId::from("test-agent");
        let id3: AgentId = "test-agent".into();

        assert_eq!(id1, id2);
        assert_eq!(id2, id3);
        assert_eq!(id1.as_str(), "test-agent");
    }

    #[test]
    fn test_agent_id_generation() {
        let id1 = AgentId::generate();
        let id2 = AgentId::generate();

        assert_ne!(id1, id2);
        assert!(!id1.as_str().is_empty());
    }

    #[test]
    fn test_task_creation() {
        let task = Task::new(
            TaskType::Trading,
            serde_json::json!({"symbol": "BTC/USD", "action": "buy"}),
        );

        assert_eq!(task.task_type, TaskType::Trading);
        assert_eq!(task.priority, Priority::Normal);
        assert_eq!(task.retry_count, 0);
        assert!(task.can_retry());
    }

    #[test]
    fn test_task_builder_pattern() {
        let deadline = chrono::Utc::now() + chrono::Duration::minutes(10);
        let task = Task::new(TaskType::Research, serde_json::json!({}))
            .with_priority(Priority::High)
            .with_timeout(Duration::from_secs(30))
            .with_max_retries(5)
            .with_deadline(deadline)
            .with_metadata("source", serde_json::json!("external"));

        assert_eq!(task.priority, Priority::High);
        assert_eq!(task.timeout, Some(Duration::from_secs(30)));
        assert_eq!(task.max_retries, 5);
        assert_eq!(task.deadline, Some(deadline));
        assert_eq!(
            task.metadata.get("source"),
            Some(&serde_json::json!("external"))
        );
    }

    #[test]
    fn test_task_retry_logic() {
        let mut task = Task::new(TaskType::Trading, serde_json::json!({})).with_max_retries(2);

        assert!(task.can_retry());
        task.increment_retry();
        assert!(task.can_retry());
        task.increment_retry();
        assert!(!task.can_retry());
    }

    #[test]
    fn test_task_result_creation() {
        let success = TaskResult::success(
            serde_json::json!({"result": "completed"}),
            Some("0x123".to_string()),
            Duration::from_millis(500),
        );
        assert!(success.is_success());

        let failure = TaskResult::failure(
            "Network error".to_string(),
            true,
            Duration::from_millis(100),
        );
        assert!(!failure.is_success());
        assert!(failure.is_retriable());

        let permanent_failure = TaskResult::failure(
            "Invalid parameters".to_string(),
            false,
            Duration::from_millis(50),
        );
        assert!(!permanent_failure.is_retriable());
    }

    #[test]
    fn test_agent_message_creation() {
        let from = AgentId::new("agent1");
        let to = AgentId::new("agent2");

        let message = AgentMessage::new(
            from.clone(),
            Some(to.clone()),
            "task_update".to_string(),
            serde_json::json!({"status": "completed"}),
        );

        assert_eq!(message.from, from);
        assert_eq!(message.to, Some(to));
        assert_eq!(message.message_type, "task_update");
        assert!(!message.is_expired());
    }

    #[test]
    fn test_broadcast_message() {
        let from = AgentId::new("broadcaster");
        let message = AgentMessage::broadcast(
            from.clone(),
            "system_alert".to_string(),
            serde_json::json!({"alert": "high_volatility"}),
        );

        assert_eq!(message.from, from);
        assert_eq!(message.to, None);
        assert_eq!(message.message_type, "system_alert");
    }

    #[test]
    fn test_routing_rule_matching() {
        let task =
            Task::new(TaskType::Trading, serde_json::json!({})).with_priority(Priority::High);
        let agent_id = AgentId::new("trading-agent");
        let capabilities = vec!["trading".to_string(), "risk_management".to_string()];

        let task_type_rule = RoutingRule::TaskType(TaskType::Trading);
        assert!(task_type_rule.matches(&task, &agent_id, &capabilities));

        let capability_rule = RoutingRule::Capability("trading".to_string());
        assert!(capability_rule.matches(&task, &agent_id, &capabilities));

        let priority_rule = RoutingRule::Priority(Priority::Normal);
        assert!(priority_rule.matches(&task, &agent_id, &capabilities));

        let agent_rule = RoutingRule::Agent(agent_id.clone());
        assert!(agent_rule.matches(&task, &agent_id, &capabilities));

        let all_rule = RoutingRule::All(vec![task_type_rule, capability_rule]);
        assert!(all_rule.matches(&task, &agent_id, &capabilities));
    }

    #[test]
    fn test_capability_creation() {
        let capability = Capability::new("trading", "1.0").with_parameter(
            "supported_exchanges",
            serde_json::json!(["binance", "coinbase"]),
        );

        assert_eq!(capability.name, "trading");
        assert_eq!(capability.version, "1.0");
        assert!(capability.parameters.contains_key("supported_exchanges"));
    }

    #[test]
    fn test_priority_ordering() {
        assert!(Priority::Critical > Priority::High);
        assert!(Priority::High > Priority::Normal);
        assert!(Priority::Normal > Priority::Low);
    }

    #[test]
    fn test_task_type_display() {
        assert_eq!(TaskType::Trading.to_string(), "trading");
        assert_eq!(
            TaskType::Custom("arbitrage".to_string()).to_string(),
            "custom:arbitrage"
        );
    }

    // Additional tests for 100% coverage

    #[test]
    fn test_agent_id_display() {
        let id = AgentId::new("test-display");
        assert_eq!(format!("{id}"), "test-display");
    }

    #[test]
    fn test_agent_id_from_string() {
        let string_id = String::from("string-agent");
        let agent_id = AgentId::from(string_id);
        assert_eq!(agent_id.as_str(), "string-agent");
    }

    #[test]
    fn test_task_type_matches() {
        let trading = TaskType::Trading;
        let research = TaskType::Research;
        let custom1 = TaskType::Custom("test".to_string());
        let custom2 = TaskType::Custom("test".to_string());
        let custom3 = TaskType::Custom("different".to_string());

        assert!(trading.matches(&TaskType::Trading));
        assert!(!trading.matches(&research));
        assert!(custom1.matches(&custom2));
        assert!(!custom1.matches(&custom3));
    }

    #[test]
    fn test_task_type_display_all_variants() {
        assert_eq!(TaskType::Research.to_string(), "research");
        assert_eq!(TaskType::RiskAnalysis.to_string(), "risk_analysis");
        assert_eq!(TaskType::Portfolio.to_string(), "portfolio");
        assert_eq!(TaskType::Monitoring.to_string(), "monitoring");
    }

    #[test]
    fn test_priority_default() {
        assert_eq!(Priority::default(), Priority::Normal);
    }

    #[test]
    fn test_task_is_past_deadline_no_deadline() {
        let task = Task::new(TaskType::Trading, serde_json::json!({}));
        assert!(!task.is_past_deadline());
    }

    #[test]
    fn test_task_is_past_deadline_future() {
        let future_deadline = chrono::Utc::now() + chrono::Duration::minutes(10);
        let task =
            Task::new(TaskType::Trading, serde_json::json!({})).with_deadline(future_deadline);
        assert!(!task.is_past_deadline());
    }

    #[test]
    fn test_task_is_past_deadline_past() {
        let past_deadline = chrono::Utc::now() - chrono::Duration::minutes(10);
        let task = Task::new(TaskType::Trading, serde_json::json!({})).with_deadline(past_deadline);
        assert!(task.is_past_deadline());
    }

    #[test]
    fn test_task_result_cancelled() {
        let cancelled = TaskResult::cancelled("User requested".to_string());
        assert!(!cancelled.is_success());
        assert!(!cancelled.is_retriable());
        assert!(cancelled.data().is_none());
        assert!(cancelled.error().is_none());
    }

    #[test]
    fn test_task_result_timeout() {
        let timeout = TaskResult::timeout(Duration::from_secs(30));
        assert!(!timeout.is_success());
        assert!(!timeout.is_retriable());
        assert!(timeout.data().is_none());
        assert!(timeout.error().is_none());
    }

    #[test]
    fn test_task_result_data_extraction() {
        let data = serde_json::json!({"key": "value"});
        let success = TaskResult::success(data.clone(), None, Duration::from_millis(100));
        assert_eq!(success.data(), Some(&data));

        let failure = TaskResult::failure("error".to_string(), false, Duration::from_millis(50));
        assert!(failure.data().is_none());
    }

    #[test]
    fn test_task_result_error_extraction() {
        let error_msg = "Something went wrong";
        let failure = TaskResult::failure(error_msg.to_string(), true, Duration::from_millis(100));
        assert_eq!(failure.error(), Some(error_msg));

        let success = TaskResult::success(serde_json::json!({}), None, Duration::from_millis(100));
        assert!(success.error().is_none());
    }

    #[test]
    fn test_agent_message_with_priority() {
        let from = AgentId::new("sender");
        let message = AgentMessage::new(from, None, "test".to_string(), serde_json::json!({}))
            .with_priority(Priority::Critical);

        assert_eq!(message.priority, Priority::Critical);
    }

    #[test]
    fn test_agent_message_with_expiration() {
        let from = AgentId::new("sender");
        let expiry = chrono::Utc::now() + chrono::Duration::minutes(5);
        let message = AgentMessage::new(from, None, "test".to_string(), serde_json::json!({}))
            .with_expiration(expiry);

        assert_eq!(message.expires_at, Some(expiry));
        assert!(!message.is_expired());
    }

    #[test]
    fn test_agent_message_is_expired_no_expiry() {
        let from = AgentId::new("sender");
        let message = AgentMessage::new(from, None, "test".to_string(), serde_json::json!({}));

        assert!(!message.is_expired());
    }

    #[test]
    fn test_agent_message_is_expired_past() {
        let from = AgentId::new("sender");
        let past_expiry = chrono::Utc::now() - chrono::Duration::minutes(5);
        let message = AgentMessage::new(from, None, "test".to_string(), serde_json::json!({}))
            .with_expiration(past_expiry);

        assert!(message.is_expired());
    }

    #[test]
    fn test_routing_rule_task_type_no_match() {
        let task = Task::new(TaskType::Trading, serde_json::json!({}));
        let agent_id = AgentId::new("agent");
        let capabilities = vec![];

        let rule = RoutingRule::TaskType(TaskType::Research);
        assert!(!rule.matches(&task, &agent_id, &capabilities));
    }

    #[test]
    fn test_routing_rule_capability_no_match() {
        let task = Task::new(TaskType::Trading, serde_json::json!({}));
        let agent_id = AgentId::new("agent");
        let capabilities = vec!["research".to_string()];

        let rule = RoutingRule::Capability("trading".to_string());
        assert!(!rule.matches(&task, &agent_id, &capabilities));
    }

    #[test]
    fn test_routing_rule_agent_no_match() {
        let task = Task::new(TaskType::Trading, serde_json::json!({}));
        let agent_id = AgentId::new("agent1");
        let capabilities = vec![];

        let rule = RoutingRule::Agent(AgentId::new("agent2"));
        assert!(!rule.matches(&task, &agent_id, &capabilities));
    }

    #[test]
    fn test_routing_rule_priority_no_match() {
        let task = Task::new(TaskType::Trading, serde_json::json!({})).with_priority(Priority::Low);
        let agent_id = AgentId::new("agent");
        let capabilities = vec![];

        let rule = RoutingRule::Priority(Priority::High);
        assert!(!rule.matches(&task, &agent_id, &capabilities));
    }

    #[test]
    fn test_routing_rule_all_partial_match() {
        let task = Task::new(TaskType::Trading, serde_json::json!({}));
        let agent_id = AgentId::new("agent");
        let capabilities = vec!["trading".to_string()];

        let rule = RoutingRule::All(vec![
            RoutingRule::TaskType(TaskType::Trading),
            RoutingRule::Capability("research".to_string()), // This will fail
        ]);
        assert!(!rule.matches(&task, &agent_id, &capabilities));
    }

    #[test]
    fn test_routing_rule_any_partial_match() {
        let task = Task::new(TaskType::Trading, serde_json::json!({}));
        let agent_id = AgentId::new("agent");
        let capabilities = vec!["trading".to_string()];

        let rule = RoutingRule::Any(vec![
            RoutingRule::TaskType(TaskType::Research), // This will fail
            RoutingRule::Capability("trading".to_string()), // This will succeed
        ]);
        assert!(rule.matches(&task, &agent_id, &capabilities));
    }

    #[test]
    fn test_routing_rule_any_no_match() {
        let task = Task::new(TaskType::Trading, serde_json::json!({}));
        let agent_id = AgentId::new("agent");
        let capabilities = vec!["trading".to_string()];

        let rule = RoutingRule::Any(vec![
            RoutingRule::TaskType(TaskType::Research),
            RoutingRule::Capability("research".to_string()),
        ]);
        assert!(!rule.matches(&task, &agent_id, &capabilities));
    }

    #[test]
    fn test_routing_rule_round_robin() {
        let task = Task::new(TaskType::Trading, serde_json::json!({}));
        let agent_id = AgentId::new("agent");
        let capabilities = vec![];

        let rule = RoutingRule::RoundRobin;
        assert!(!rule.matches(&task, &agent_id, &capabilities));
    }

    #[test]
    fn test_routing_rule_least_loaded() {
        let task = Task::new(TaskType::Trading, serde_json::json!({}));
        let agent_id = AgentId::new("agent");
        let capabilities = vec![];

        let rule = RoutingRule::LeastLoaded;
        assert!(!rule.matches(&task, &agent_id, &capabilities));
    }

    #[test]
    fn test_routing_rule_custom() {
        let task = Task::new(TaskType::Trading, serde_json::json!({}));
        let agent_id = AgentId::new("agent");
        let capabilities = vec![];

        let rule = RoutingRule::Custom("custom_logic".to_string());
        assert!(!rule.matches(&task, &agent_id, &capabilities));
    }

    #[test]
    fn test_capability_with_parameter() {
        let capability = Capability::new("test", "1.0")
            .with_parameter("param1", serde_json::json!("value1"))
            .with_parameter("param2", serde_json::json!(42));

        assert_eq!(capability.name, "test");
        assert_eq!(capability.version, "1.0");
        assert_eq!(capability.parameters.len(), 2);
        assert_eq!(
            capability.parameters.get("param1"),
            Some(&serde_json::json!("value1"))
        );
        assert_eq!(
            capability.parameters.get("param2"),
            Some(&serde_json::json!(42))
        );
        assert!(capability.schema.is_none());
    }

    #[test]
    fn test_capability_with_schema() {
        let schema = serde_json::json!({
            "type": "object",
            "properties": {
                "symbol": {"type": "string"},
                "amount": {"type": "number"}
            },
            "required": ["symbol", "amount"]
        });

        let capability = Capability::new("trading", "2.0").with_schema(schema.clone());

        assert_eq!(capability.name, "trading");
        assert_eq!(capability.version, "2.0");
        assert_eq!(capability.schema, Some(schema));
    }

    #[test]
    fn test_agent_state_default() {
        assert_eq!(AgentState::default(), AgentState::Idle);
    }

    #[test]
    fn test_agent_state_all_variants() {
        // Test all variants can be created and are distinct
        let states = [
            AgentState::Active,
            AgentState::Busy,
            AgentState::Full,
            AgentState::Idle,
            AgentState::Offline,
            AgentState::Maintenance,
        ];

        // Ensure all variants are distinct
        for (i, state1) in states.iter().enumerate() {
            for (j, state2) in states.iter().enumerate() {
                if i == j {
                    assert_eq!(state1, state2);
                } else {
                    assert_ne!(state1, state2);
                }
            }
        }
    }

    #[test]
    fn test_task_retry_at_limit() {
        let mut task = Task::new(TaskType::Trading, serde_json::json!({})).with_max_retries(0);

        assert!(!task.can_retry()); // Already at limit
        task.increment_retry();
        assert_eq!(task.retry_count, 1);
        assert!(!task.can_retry());
    }

    #[test]
    fn test_task_with_metadata_multiple() {
        let task = Task::new(TaskType::Trading, serde_json::json!({}))
            .with_metadata("key1", serde_json::json!("value1"))
            .with_metadata("key2", serde_json::json!(42))
            .with_metadata("key3", serde_json::json!({"nested": "object"}));

        assert_eq!(task.metadata.len(), 3);
        assert_eq!(
            task.metadata.get("key1"),
            Some(&serde_json::json!("value1"))
        );
        assert_eq!(task.metadata.get("key2"), Some(&serde_json::json!(42)));
        assert_eq!(
            task.metadata.get("key3"),
            Some(&serde_json::json!({"nested": "object"}))
        );
    }

    #[test]
    fn test_task_result_success_with_tx_hash() {
        let data = serde_json::json!({"amount": 100});
        let tx_hash = Some("0xabcdef".to_string());
        let duration = Duration::from_millis(200);

        let result = TaskResult::success(data.clone(), tx_hash.clone(), duration);

        match result {
            TaskResult::Success {
                data: ref result_data,
                tx_hash: ref result_tx_hash,
                duration: ref result_duration,
            } => {
                assert_eq!(result_data, &data);
                assert_eq!(result_tx_hash, &tx_hash);
                assert_eq!(result_duration, &duration);
            }
            _ => panic!("Expected Success variant"),
        }

        assert!(result.is_success());
        assert!(!result.is_retriable());
    }

    #[test]
    fn test_task_result_success_without_tx_hash() {
        let data = serde_json::json!({"amount": 100});
        let duration = Duration::from_millis(200);

        let result = TaskResult::success(data.clone(), None, duration);

        match result {
            TaskResult::Success {
                data: ref result_data,
                tx_hash: ref result_tx_hash,
                duration: ref result_duration,
            } => {
                assert_eq!(result_data, &data);
                assert_eq!(result_tx_hash, &None);
                assert_eq!(result_duration, &duration);
            }
            _ => panic!("Expected Success variant"),
        }
    }

    #[test]
    fn test_task_result_failure_retriable() {
        let error = "Network timeout".to_string();
        let duration = Duration::from_millis(150);

        let result = TaskResult::failure(error.clone(), true, duration);

        match result {
            TaskResult::Failure {
                error: ref result_error,
                retriable: result_retriable,
                duration: ref result_duration,
            } => {
                assert_eq!(result_error, &error);
                assert!(result_retriable);
                assert_eq!(result_duration, &duration);
            }
            _ => panic!("Expected Failure variant"),
        }

        assert!(!result.is_success());
        assert!(result.is_retriable());
    }

    #[test]
    fn test_task_result_failure_not_retriable() {
        let error = "Invalid input".to_string();
        let duration = Duration::from_millis(50);

        let result = TaskResult::failure(error.clone(), false, duration);

        match result {
            TaskResult::Failure {
                error: ref result_error,
                retriable: result_retriable,
                duration: ref result_duration,
            } => {
                assert_eq!(result_error, &error);
                assert!(!result_retriable);
                assert_eq!(result_duration, &duration);
            }
            _ => panic!("Expected Failure variant"),
        }

        assert!(!result.is_success());
        assert!(!result.is_retriable());
    }

    #[test]
    fn test_task_result_cancelled_variant() {
        let reason = "User cancelled".to_string();

        let result = TaskResult::cancelled(reason.clone());

        match result {
            TaskResult::Cancelled {
                reason: ref result_reason,
            } => {
                assert_eq!(result_reason, &reason);
            }
            _ => panic!("Expected Cancelled variant"),
        }

        assert!(!result.is_success());
        assert!(!result.is_retriable());
    }

    #[test]
    fn test_task_result_timeout_variant() {
        let duration = Duration::from_secs(30);

        let result = TaskResult::timeout(duration);

        match result {
            TaskResult::Timeout {
                duration: result_duration,
            } => {
                assert_eq!(result_duration, duration);
            }
            _ => panic!("Expected Timeout variant"),
        }

        assert!(!result.is_success());
        assert!(!result.is_retriable());
    }

    #[test]
    fn test_routing_rule_priority_equal_match() {
        let task =
            Task::new(TaskType::Trading, serde_json::json!({})).with_priority(Priority::High);
        let agent_id = AgentId::new("agent");
        let capabilities = vec![];

        let rule = RoutingRule::Priority(Priority::High);
        assert!(rule.matches(&task, &agent_id, &capabilities));
    }

    #[test]
    fn test_routing_rule_all_empty() {
        let task = Task::new(TaskType::Trading, serde_json::json!({}));
        let agent_id = AgentId::new("agent");
        let capabilities = vec![];

        let rule = RoutingRule::All(vec![]);
        assert!(rule.matches(&task, &agent_id, &capabilities)); // Empty All should match
    }

    #[test]
    fn test_routing_rule_any_empty() {
        let task = Task::new(TaskType::Trading, serde_json::json!({}));
        let agent_id = AgentId::new("agent");
        let capabilities = vec![];

        let rule = RoutingRule::Any(vec![]);
        assert!(!rule.matches(&task, &agent_id, &capabilities)); // Empty Any should not match
    }

    #[test]
    fn test_capability_type_partial_eq_str() {
        // Test PartialEq<&str> implementation
        assert_eq!(CapabilityType::Trading, "trading");
        assert_eq!(CapabilityType::Research, "research");
        assert_eq!(CapabilityType::RiskAnalysis, "risk_analysis");
        assert_eq!(CapabilityType::Portfolio, "portfolio");
        assert_eq!(CapabilityType::Monitoring, "monitoring");
        assert_eq!(CapabilityType::ToolCalling, "tool_calling");
        assert_eq!(
            CapabilityType::Custom("custom_name".to_string()),
            "custom_name"
        );

        // Test inequality
        assert_ne!(CapabilityType::Trading, "research");
        assert_ne!(CapabilityType::Custom("foo".to_string()), "bar");
    }

    #[test]
    fn test_capability_type_vec_comparison() {
        // Test Vec<CapabilityType> comparison with Vec<&str>
        let capabilities = vec![
            CapabilityType::Trading,
            CapabilityType::Research,
            CapabilityType::Custom("custom".to_string()),
        ];

        let expected = vec!["trading", "research", "custom"];

        assert_eq!(capabilities, expected);
    }
}
