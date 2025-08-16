//! Integration test modules for riglr-agents.
//!
//! This module organizes and provides common utilities for integration tests.

pub mod multi_agent_workflow;
pub mod signer_isolation;
pub mod error_recovery;
pub mod performance;

/// Common test utilities and helpers.
pub mod test_utils {
    use riglr_agents::*;
    use std::sync::Arc;
    use std::time::Duration;
    use serde_json::json;

    /// Create a simple test agent with configurable behavior.
    pub fn create_test_agent(
        id: impl Into<String>,
        capabilities: Vec<String>,
        should_succeed: bool,
        delay: Duration,
    ) -> Arc<dyn Agent> {
        Arc::new(TestAgent {
            id: AgentId::new(id),
            capabilities,
            should_succeed,
            delay,
        })
    }

    /// Create a test registry with a set of agents.
    pub async fn create_test_registry(
        agent_configs: Vec<(String, Vec<String>, bool, Duration)>,
    ) -> Arc<LocalAgentRegistry> {
        let registry = Arc::new(LocalAgentRegistry::new());

        for (id, caps, success, delay) in agent_configs {
            let agent = create_test_agent(id, caps, success, delay);
            registry.register_agent(agent).await.unwrap();
        }

        registry
    }

    /// Create a test task with specified parameters.
    pub fn create_test_task(
        task_type: TaskType,
        parameters: serde_json::Value,
        priority: Option<Priority>,
    ) -> Task {
        let mut task = Task::new(task_type, parameters);
        if let Some(pri) = priority {
            task = task.with_priority(pri);
        }
        task
    }

    /// Simple test agent implementation.
    #[derive(Clone)]
    pub struct TestAgent {
        pub id: AgentId,
        pub capabilities: Vec<String>,
        pub should_succeed: bool,
        pub delay: Duration,
    }

    #[async_trait::async_trait]
    impl Agent for TestAgent {
        async fn execute_task(&self, task: Task) -> Result<TaskResult> {
            // Simulate processing delay
            if self.delay > Duration::ZERO {
                tokio::time::sleep(self.delay).await;
            }

            if self.should_succeed {
                Ok(TaskResult::success(
                    json!({
                        "agent_id": self.id.as_str(),
                        "task_id": task.id,
                        "task_type": task.task_type.to_string(),
                        "timestamp": chrono::Utc::now().timestamp()
                    }),
                    None,
                    self.delay,
                ))
            } else {
                Ok(TaskResult::failure(
                    "Test agent configured to fail".to_string(),
                    true, // retriable
                    self.delay,
                ))
            }
        }

        fn id(&self) -> &AgentId {
            &self.id
        }

        fn capabilities(&self) -> Vec<String> {
            self.capabilities.clone()
        }
    }

    /// Performance measurement utilities.
    pub struct PerformanceMetrics {
        pub start_time: std::time::Instant,
        pub task_count: usize,
        pub success_count: usize,
        pub failure_count: usize,
        pub total_duration: Duration,
    }

    impl PerformanceMetrics {
        pub fn new(task_count: usize) -> Self {
            Self {
                start_time: std::time::Instant::now(),
                task_count,
                success_count: 0,
                failure_count: 0,
                total_duration: Duration::ZERO,
            }
        }

        pub fn record_result(&mut self, success: bool) {
            if success {
                self.success_count += 1;
            } else {
                self.failure_count += 1;
            }
        }

        pub fn finish(&mut self) {
            self.total_duration = self.start_time.elapsed();
        }

        pub fn throughput(&self) -> f64 {
            self.success_count as f64 / self.total_duration.as_secs_f64()
        }

        pub fn success_rate(&self) -> f64 {
            if self.task_count == 0 {
                0.0
            } else {
                self.success_count as f64 / self.task_count as f64
            }
        }

        pub fn print_summary(&self) {
            println!("Performance Summary:");
            println!("  Tasks: {}", self.task_count);
            println!("  Successes: {}", self.success_count);
            println!("  Failures: {}", self.failure_count);
            println!("  Duration: {:?}", self.total_duration);
            println!("  Throughput: {:.2} tasks/sec", self.throughput());
            println!("  Success Rate: {:.2}%", self.success_rate() * 100.0);
        }
    }

    /// Assertion helpers for test validation.
    pub fn assert_high_success_rate(success_count: usize, total_count: usize, min_rate: f64) {
        let rate = success_count as f64 / total_count as f64;
        assert!(
            rate >= min_rate,
            "Success rate {:.2}% is below minimum {:.2}%",
            rate * 100.0,
            min_rate * 100.0
        );
    }

    pub fn assert_reasonable_throughput(throughput: f64, min_throughput: f64) {
        assert!(
            throughput >= min_throughput,
            "Throughput {:.2} tasks/sec is below minimum {:.2} tasks/sec",
            throughput,
            min_throughput
        );
    }

    pub fn assert_within_timeout(duration: Duration, max_duration: Duration) {
        assert!(
            duration <= max_duration,
            "Duration {:?} exceeds maximum {:?}",
            duration,
            max_duration
        );
    }
}

/// Test configuration constants.
pub mod test_constants {
    use std::time::Duration;

    pub const DEFAULT_TEST_TIMEOUT: Duration = Duration::from_secs(30);
    pub const FAST_TASK_DELAY: Duration = Duration::from_millis(10);
    pub const MEDIUM_TASK_DELAY: Duration = Duration::from_millis(50);
    pub const SLOW_TASK_DELAY: Duration = Duration::from_millis(200);

    pub const MIN_SUCCESS_RATE: f64 = 0.95; // 95%
    pub const MIN_THROUGHPUT: f64 = 10.0; // tasks per second

    pub const SMALL_TASK_BATCH: usize = 10;
    pub const MEDIUM_TASK_BATCH: usize = 50;
    pub const LARGE_TASK_BATCH: usize = 200;
}

/// Shared test fixtures for common scenarios.
pub mod fixtures {
    use super::*;
    use riglr_agents::*;

    /// Create a standard multi-agent setup for testing.
    pub async fn standard_multi_agent_setup() -> (Arc<LocalAgentRegistry>, Arc<AgentDispatcher<LocalAgentRegistry>>) {
        let registry = Arc::new(LocalAgentRegistry::new());
        let dispatcher = Arc::new(AgentDispatcher::new(registry.clone()));

        // Trading agents
        let trading_agent_1 = test_utils::create_test_agent(
            "trader-1",
            vec!["trading".to_string()],
            true,
            test_constants::FAST_TASK_DELAY,
        );

        let trading_agent_2 = test_utils::create_test_agent(
            "trader-2",
            vec!["trading".to_string()],
            true,
            test_constants::MEDIUM_TASK_DELAY,
        );

        // Research agent
        let research_agent = test_utils::create_test_agent(
            "researcher-1",
            vec!["research".to_string()],
            true,
            test_constants::MEDIUM_TASK_DELAY,
        );

        // Risk analysis agent
        let risk_agent = test_utils::create_test_agent(
            "risk-1",
            vec!["risk_analysis".to_string()],
            true,
            test_constants::FAST_TASK_DELAY,
        );

        registry.register_agent(trading_agent_1).await.unwrap();
        registry.register_agent(trading_agent_2).await.unwrap();
        registry.register_agent(research_agent).await.unwrap();
        registry.register_agent(risk_agent).await.unwrap();

        (registry, dispatcher)
    }

    /// Create a high-performance setup for load testing.
    pub async fn high_performance_setup() -> (Arc<LocalAgentRegistry>, Arc<AgentDispatcher<LocalAgentRegistry>>) {
        let registry = Arc::new(LocalAgentRegistry::new());
        let config = DispatchConfig {
            enable_load_balancing: true,
            routing_strategy: RoutingStrategy::LeastLoaded,
            max_concurrent_tasks_per_agent: 20,
            ..Default::default()
        };
        let dispatcher = Arc::new(AgentDispatcher::with_config(registry.clone(), config));

        // Create multiple fast agents
        for i in 0..8 {
            let agent = test_utils::create_test_agent(
                format!("fast-agent-{}", i),
                vec!["benchmark".to_string()],
                true,
                Duration::from_millis(5),
            );
            registry.register_agent(agent).await.unwrap();
        }

        (registry, dispatcher)
    }

    /// Create a fault-tolerant setup for error testing.
    pub async fn fault_tolerant_setup() -> (Arc<LocalAgentRegistry>, Arc<AgentDispatcher<LocalAgentRegistry>>) {
        let registry = Arc::new(LocalAgentRegistry::new());
        let config = DispatchConfig {
            max_retries: 3,
            retry_delay: Duration::from_millis(10),
            default_task_timeout: Duration::from_secs(5),
            ..Default::default()
        };
        let dispatcher = Arc::new(AgentDispatcher::with_config(registry.clone(), config));

        // Mix of reliable and unreliable agents
        let reliable_agent = test_utils::create_test_agent(
            "reliable-agent",
            vec!["test".to_string()],
            true,
            test_constants::FAST_TASK_DELAY,
        );

        let flaky_agent = test_utils::create_test_agent(
            "flaky-agent",
            vec!["test".to_string()],
            false, // Will fail initially
            test_constants::FAST_TASK_DELAY,
        );

        registry.register_agent(reliable_agent).await.unwrap();
        registry.register_agent(flaky_agent).await.unwrap();

        (registry, dispatcher)
    }
}