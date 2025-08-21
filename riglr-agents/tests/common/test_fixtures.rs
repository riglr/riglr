use riglr_agents::*;
use serde_json::json;
use std::time::Duration;

pub struct TestTaskBuilder {
    task_type: TaskType,
    parameters: serde_json::Value,
    priority: Priority,
    timeout: Option<Duration>,
    max_retries: u32,
    deadline: Option<chrono::DateTime<chrono::Utc>>,
    metadata: std::collections::HashMap<String, serde_json::Value>,
}

impl TestTaskBuilder {
    pub fn new(task_type: TaskType) -> Self {
        Self {
            task_type,
            parameters: json!({}),
            priority: Priority::Normal,
            timeout: None,
            max_retries: 3,
            deadline: None,
            metadata: std::collections::HashMap::new(),
        }
    }

    pub fn with_parameters(mut self, parameters: serde_json::Value) -> Self {
        self.parameters = parameters;
        self
    }

    pub fn with_priority(mut self, priority: Priority) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }

    pub fn with_deadline(mut self, deadline: chrono::DateTime<chrono::Utc>) -> Self {
        self.deadline = Some(deadline);
        self
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }

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

    pub fn trading() -> Self {
        Self::new(TaskType::Trading)
    }

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

    pub fn high_priority(mut self) -> Self {
        self.priority = Priority::High;
        self
    }

    pub fn with_deadline_in(mut self, duration: Duration) -> Self {
        self.deadline = Some(chrono::Utc::now() + chrono::Duration::from_std(duration).unwrap());
        self
    }
}

// Test scenarios based on trading workflows
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

pub fn create_research_task() -> Task {
    TestTaskBuilder::new(TaskType::Research)
        .with_parameters(json!({"query": "BONK market conditions"}))
        .with_priority(Priority::Normal)
        .build()
}

pub fn create_trading_task() -> Task {
    TestTaskBuilder::new(TaskType::Trading)
        .with_parameters(json!({"symbol": "BTC/USD", "action": "buy"}))
        .with_priority(Priority::High)
        .build()
}

pub fn create_risk_analysis_task() -> Task {
    TestTaskBuilder::new(TaskType::RiskAnalysis)
        .with_parameters(json!({"portfolio": "main", "risk_level": "moderate"}))
        .with_priority(Priority::Normal)
        .build()
}

pub fn create_portfolio_task() -> Task {
    TestTaskBuilder::new(TaskType::Portfolio)
        .with_parameters(
            json!({"action": "rebalance", "target_allocation": {"BTC": 50, "ETH": 30, "SOL": 20}}),
        )
        .with_priority(Priority::Low)
        .build()
}

pub fn create_monitoring_task() -> Task {
    TestTaskBuilder::new(TaskType::Monitoring)
        .with_parameters(json!({"targets": ["BTC/USD", "ETH/USD"], "interval": "1m"}))
        .with_priority(Priority::Low)
        .build()
}

pub fn create_custom_task(capability: &str) -> Task {
    TestTaskBuilder::new(TaskType::Custom(capability.to_string()))
        .with_parameters(json!({"custom_param": "test_value"}))
        .with_priority(Priority::Normal)
        .build()
}

// Message builders for communication testing
pub fn create_test_message(
    from: &str,
    to: Option<&str>,
    message_type: &str,
    payload: serde_json::Value,
) -> AgentMessage {
    AgentMessage::new(
        AgentId::new(from),
        to.map(|t| AgentId::new(t)),
        message_type.to_string(),
        payload,
    )
}

pub fn create_broadcast_message(
    from: &str,
    message_type: &str,
    payload: serde_json::Value,
) -> AgentMessage {
    AgentMessage::broadcast(AgentId::new(from), message_type.to_string(), payload)
}

pub fn create_market_update_message(from: &str, to: Option<&str>) -> AgentMessage {
    create_test_message(
        from,
        to,
        "market_update",
        json!({
            "symbol": "BTC/USD",
            "price": 50000,
            "volume": 1000000,
            "timestamp": chrono::Utc::now().to_rfc3339()
        }),
    )
}

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

// Test configuration builders
pub fn create_test_dispatch_config() -> DispatchConfig {
    DispatchConfig {
        default_task_timeout: Duration::from_secs(10),
        max_retries: 2,
        retry_delay: Duration::from_millis(100),
        max_concurrent_tasks_per_agent: 5,
        enable_load_balancing: true,
        routing_strategy: RoutingStrategy::Capability,
    }
}

pub fn create_fast_dispatch_config() -> DispatchConfig {
    DispatchConfig {
        default_task_timeout: Duration::from_millis(500),
        max_retries: 1,
        retry_delay: Duration::from_millis(10),
        max_concurrent_tasks_per_agent: 10,
        enable_load_balancing: false,
        routing_strategy: RoutingStrategy::RoundRobin,
    }
}

// Test data validation helpers
pub fn validate_task_result_success(result: &TaskResult) -> bool {
    match result {
        TaskResult::Success { .. } => true,
        _ => false,
    }
}

pub fn validate_task_result_failure(result: &TaskResult) -> bool {
    match result {
        TaskResult::Failure { .. } => true,
        _ => false,
    }
}

pub fn extract_task_result_data(result: &TaskResult) -> Option<&serde_json::Value> {
    match result {
        TaskResult::Success { data, .. } => Some(data),
        _ => None,
    }
}

pub fn extract_task_result_tx_hash(result: &TaskResult) -> Option<&str> {
    match result {
        TaskResult::Success {
            tx_hash: Some(hash),
            ..
        } => Some(hash),
        _ => None,
    }
}

pub fn extract_task_result_error(result: &TaskResult) -> Option<&str> {
    match result {
        TaskResult::Failure { error, .. } => Some(error),
        _ => None,
    }
}

// Performance testing helpers
pub fn create_load_test_tasks(count: usize, task_type: TaskType) -> Vec<Task> {
    (0..count)
        .map(|i| {
            TestTaskBuilder::new(task_type.clone())
                .with_parameters(json!({"index": i, "timestamp": chrono::Utc::now().to_rfc3339()}))
                .with_priority(Priority::Normal)
                .build()
        })
        .collect()
}

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
            let priority = priorities[i % priorities.len()];
            let task_type = task_types[i % task_types.len()].clone();

            TestTaskBuilder::new(task_type)
                .with_parameters(json!({"batch_index": i}))
                .with_priority(priority)
                .build()
        })
        .collect()
}
