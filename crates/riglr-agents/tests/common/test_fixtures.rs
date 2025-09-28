#![allow(clippy::expect_used)]
use core::time::Duration;
use riglr_agents::*;
use serde_json::json;
use std::collections::HashMap;

/// Builder for creating test tasks with customizable parameters.
/// Provides a fluent interface for constructing tasks with various configurations.
#[derive(Debug)]
pub struct TestTaskBuilder {
    task_type: TaskType,
    parameters: serde_json::Value,
    priority: Priority,
    timeout: Option<Duration>,
    max_retries: u32,
    deadline: Option<chrono::DateTime<chrono::Utc>>,
    metadata: HashMap<String, serde_json::Value>,
}

impl TestTaskBuilder {
    /// Creates a new test task builder with default values.
    #[must_use]
    pub fn new(task_type: TaskType) -> Self {
        Self {
            task_type,
            parameters: json!({}),
            priority: Priority::Normal,
            timeout: None,
            max_retries: 3,
            deadline: None,
            metadata: HashMap::new(),
        }
    }

    /// Sets the task parameters.
    #[must_use]
    pub fn with_parameters(mut self, parameters: serde_json::Value) -> Self {
        self.parameters = parameters;
        self
    }

    /// Sets the task priority.
    #[must_use]
    pub const fn with_priority(mut self, priority: Priority) -> Self {
        self.priority = priority;
        self
    }

    /// Sets the task timeout duration.
    #[must_use]
    pub const fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Sets the maximum number of retry attempts.
    #[must_use]
    pub const fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }

    /// Sets the task deadline.
    #[must_use]
    pub const fn with_deadline(mut self, deadline: chrono::DateTime<chrono::Utc>) -> Self {
        self.deadline = Some(deadline);
        self
    }

    /// Adds a metadata key-value pair to the task.
    #[must_use]
    pub fn with_metadata(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }

    /// Builds and returns the configured task.
    #[must_use]
    pub fn build(self) -> Task {
        let mut task = Task::new(self.task_type, self.parameters)
            .with_priority(self.priority)
            .with_max_retries(self.max_retries);

        if let Some(timeout) = self.timeout {
            task = task.with_timeout(timeout);
        }

        if let Some(deadline) = self.deadline {
            task = task.with_deadline(deadline);
        }

        for (key, value) in self.metadata {
            task = task.with_metadata(key, value);
        }

        task
    }

    /// Creates a builder for a trading task with default configuration.
    #[must_use]
    pub fn trading() -> Self {
        Self::new(TaskType::Trading)
    }

    /// Adds a single parameter to the task's parameters object.
    #[must_use]
    pub fn with_parameter(mut self, key: &str, value: serde_json::Value) -> Self {
        if let serde_json::Value::Object(ref mut obj) = self.parameters {
            obj.insert(key.to_string(), value);
        } else {
            let mut obj = serde_json::Map::new();
            obj.insert(key.to_string(), value);
            self.parameters = serde_json::Value::Object(obj);
        }
        self
    }

    /// Sets the task priority to High.
    #[must_use]
    pub const fn high_priority(mut self) -> Self {
        self.priority = Priority::High;
        self
    }

    /// Sets the task deadline to the current time plus the given duration.
    ///
    /// # Panics
    ///
    /// Panics if the duration cannot be converted to a chrono duration.
    #[must_use]
    pub fn with_deadline_in(mut self, duration: Duration) -> Self {
        #[expect(clippy::arithmetic_side_effects)]
        {
            self.deadline = Some(
                chrono::Utc::now()
                    + chrono::Duration::from_std(duration)
                        .expect("Valid duration for chrono conversion"),
            );
        }
        self
    }
}

/// Creates a scenario with multiple related trading tasks (research, risk analysis, and trading).
#[must_use]
pub fn create_trading_task_scenario() -> Vec<Task> {
    vec![
        TestTaskBuilder::new(TaskType::Research)
            .with_parameters(json!({"symbol": "BONK", "type": "market_analysis"}))
            .with_priority(Priority::High)
            .build(),
        TestTaskBuilder::new(TaskType::RiskAnalysis)
            .with_parameters(json!({"symbol": "BONK", "position_size": 1000}))
            .with_priority(Priority::High)
            .build(),
        TestTaskBuilder::new(TaskType::Trading)
            .with_parameters(json!({"symbol": "BONK", "action": "buy", "amount": 1000}))
            .with_priority(Priority::Critical)
            .build(),
    ]
}

/// Creates a research task for market analysis.
#[must_use]
pub fn create_research_task() -> Task {
    TestTaskBuilder::new(TaskType::Research)
        .with_parameters(json!({"query": "BONK market conditions"}))
        .with_priority(Priority::Normal)
        .build()
}

/// Creates a trading task for buying BTC.
#[must_use]
pub fn create_trading_task() -> Task {
    TestTaskBuilder::new(TaskType::Trading)
        .with_parameters(json!({"symbol": "BTC/USD", "action": "buy"}))
        .with_priority(Priority::High)
        .build()
}

/// Creates a risk analysis task for portfolio evaluation.
#[must_use]
pub fn create_risk_analysis_task() -> Task {
    TestTaskBuilder::new(TaskType::RiskAnalysis)
        .with_parameters(json!({"portfolio": "main", "risk_level": "moderate"}))
        .with_priority(Priority::Normal)
        .build()
}

/// Creates a portfolio rebalancing task with target allocations.
#[must_use]
pub fn create_portfolio_task() -> Task {
    TestTaskBuilder::new(TaskType::Portfolio)
        .with_parameters(
            json!({"action": "rebalance", "target_allocation": {"BTC": 50, "ETH": 30, "SOL": 20}}),
        )
        .with_priority(Priority::Low)
        .build()
}

/// Creates a monitoring task for tracking price movements.
#[must_use]
pub fn create_monitoring_task() -> Task {
    TestTaskBuilder::new(TaskType::Monitoring)
        .with_parameters(json!({"targets": ["BTC/USD", "ETH/USD"], "interval": "1m"}))
        .with_priority(Priority::Low)
        .build()
}

/// Creates a custom task with the specified capability.
#[must_use]
pub fn create_custom_task(capability: &str) -> Task {
    TestTaskBuilder::new(TaskType::Custom(capability.to_string()))
        .with_parameters(json!({"custom_param": "test_value"}))
        .with_priority(Priority::Normal)
        .build()
}

/// Creates a test message for agent communication testing.
pub fn create_test_message(
    from: &str,
    to: Option<&str>,
    message_type: &str,
    payload: serde_json::Value,
) -> AgentMessage {
    AgentMessage::new(
        AgentId::new(from),
        to.map(AgentId::new),
        message_type.to_string(),
        payload,
    )
}

/// Creates a broadcast message sent to all agents.
#[must_use]
pub fn create_broadcast_message(
    from: &str,
    message_type: &str,
    payload: serde_json::Value,
) -> AgentMessage {
    AgentMessage::broadcast(AgentId::new(from), message_type.to_string(), payload)
}

/// Creates a market update message with sample price data.
#[must_use]
pub fn create_market_update_message(from: &str, to: Option<&str>) -> AgentMessage {
    create_test_message(
        from,
        to,
        "market_update",
        json!({
            "symbol": "BTC/USD",
            "price": 50000,
            "volume": 1_000_000,
            "timestamp": chrono::Utc::now().to_rfc3339()
        }),
    )
}

/// Creates a task completion notification message.
#[must_use]
pub fn create_task_completion_message(from: &str, to: &str, task_id: &str) -> AgentMessage {
    create_test_message(
        from,
        Some(to),
        "task_completed",
        json!({
            "task_id": task_id,
            "status": "success",
            "timestamp": chrono::Utc::now().to_rfc3339()
        }),
    )
}

/// Creates a standard dispatch configuration for testing.
#[must_use]
pub const fn create_test_dispatch_config() -> DispatchConfig {
    DispatchConfig {
        default_task_timeout: Duration::from_secs(10),
        max_retries: 2,
        retry_delay: Duration::from_millis(100),
        max_concurrent_tasks_per_agent: 5,
        enable_load_balancing: true,
        routing_strategy: RoutingStrategy::Capability,
        response_wait_timeout: Duration::from_secs(30),
    }
}

/// Creates a fast dispatch configuration with reduced timeouts for performance testing.
#[must_use]
pub const fn create_fast_dispatch_config() -> DispatchConfig {
    DispatchConfig {
        default_task_timeout: Duration::from_millis(500),
        max_retries: 1,
        retry_delay: Duration::from_millis(10),
        max_concurrent_tasks_per_agent: 10,
        enable_load_balancing: false,
        routing_strategy: RoutingStrategy::RoundRobin,
        response_wait_timeout: Duration::from_secs(10),
    }
}

/// Validates that a task result indicates success.
#[must_use]
pub const fn validate_task_result_success(result: &TaskResult) -> bool {
    matches!(result, &TaskResult::Success { .. })
}

/// Validates that a task result indicates failure.
#[must_use]
pub const fn validate_task_result_failure(result: &TaskResult) -> bool {
    matches!(result, &TaskResult::Failure { .. })
}

/// Extracts the data from a successful task result.
#[must_use]
pub const fn extract_task_result_data(result: &TaskResult) -> Option<&serde_json::Value> {
    match *result {
        TaskResult::Success { ref data, .. } => Some(data),
        _ => None,
    }
}

/// Extracts the transaction hash from a successful task result.
#[must_use]
pub fn extract_task_result_tx_hash(result: &TaskResult) -> Option<&str> {
    match result {
        &TaskResult::Success {
            tx_hash: Some(ref hash),
            ..
        } => Some(hash),
        _ => None,
    }
}

/// Extracts the error message from a failed task result.
#[must_use]
pub fn extract_task_result_error(result: &TaskResult) -> Option<&str> {
    match *result {
        TaskResult::Failure { ref error, .. } => Some(error),
        _ => None,
    }
}

/// Creates a batch of identical tasks for load testing.
#[must_use]
pub fn create_load_test_tasks(count: usize, task_type: &TaskType) -> Vec<Task> {
    (0..count)
        .map(|i| {
            TestTaskBuilder::new(task_type.clone())
                .with_parameters(json!({"index": i, "timestamp": chrono::Utc::now().to_rfc3339()}))
                .with_priority(Priority::Normal)
                .build()
        })
        .collect()
}

/// Creates a batch of tasks with varying priorities and types for comprehensive testing.
#[must_use]
pub fn create_mixed_priority_tasks(count: usize) -> Vec<Task> {
    let priorities = [
        Priority::Low,
        Priority::Normal,
        Priority::High,
        Priority::Critical,
    ];
    let task_types = [
        TaskType::Trading,
        TaskType::Research,
        TaskType::RiskAnalysis,
        TaskType::Portfolio,
    ];

    (0..count)
        .map(|i| {
            #[expect(clippy::indexing_slicing, clippy::arithmetic_side_effects)]
            let priority = priorities[i % priorities.len()];
            #[expect(clippy::indexing_slicing, clippy::arithmetic_side_effects)]
            let task_type = task_types[i % task_types.len()].clone();

            TestTaskBuilder::new(task_type)
                .with_parameters(json!({"batch_index": i}))
                .with_priority(priority)
                .build()
        })
        .collect()
}
