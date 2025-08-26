//! Task dispatch and routing system for multi-agent coordination.
//!
//! The dispatcher is responsible for routing tasks to appropriate agents
//! based on capabilities, load, and routing rules. It maintains the
//! SignerContext security model while enabling sophisticated task routing.

pub mod proxy;
pub mod queue;
pub mod queue_trait;
pub mod router;

// Re-export the trait for external use
pub use queue_trait::DistributedTaskQueue;

use crate::{Agent, AgentError, AgentRegistry, Result, Task, TaskResult};
use proxy::AgentProxy;
use router::Router;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;
use tracing::{debug, error, info, warn};

/// Configuration for the agent dispatcher.
#[derive(Debug, Clone)]
pub struct DispatchConfig {
    /// Default timeout for task execution
    pub default_task_timeout: Duration,
    /// Maximum number of retry attempts for failed tasks
    pub max_retries: u32,
    /// Delay between retry attempts
    pub retry_delay: Duration,
    /// Maximum number of concurrent tasks per agent
    pub max_concurrent_tasks_per_agent: u32,
    /// Enable load balancing
    pub enable_load_balancing: bool,
    /// Routing strategy to use
    pub routing_strategy: RoutingStrategy,
    /// Timeout for waiting for queue responses (for distributed execution)
    pub response_wait_timeout: Duration,
}

impl Default for DispatchConfig {
    fn default() -> Self {
        Self {
            default_task_timeout: Duration::from_secs(300), // 5 minutes
            max_retries: 3,
            retry_delay: Duration::from_secs(1),
            max_concurrent_tasks_per_agent: 10,
            enable_load_balancing: true,
            routing_strategy: RoutingStrategy::default(),
            response_wait_timeout: Duration::from_secs(300), // 5 minutes
        }
    }
}

/// Routing strategies for task dispatch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RoutingStrategy {
    /// Route based on agent capabilities
    #[default]
    Capability,
    /// Round-robin among capable agents
    RoundRobin,
    /// Route to least loaded agent
    LeastLoaded,
    /// Route to random capable agent
    Random,
    /// Route to specific agent (useful for directed tasks)
    Direct,
}

/// Agent dispatcher for routing tasks to appropriate agents.
///
/// The dispatcher maintains a registry of available agents and routes
/// incoming tasks based on agent capabilities, load, and routing rules.
/// It preserves the SignerContext security model by ensuring each task
/// execution maintains its own signer context.
pub struct AgentDispatcher<R: AgentRegistry> {
    /// Agent registry
    registry: Arc<R>,
    /// Routing engine
    router: Router,
    /// Dispatcher configuration
    config: DispatchConfig,
    /// Optional distributed task queue for remote execution
    distributed_queue: Option<Arc<dyn DistributedTaskQueue>>,
}

impl<R: AgentRegistry> AgentDispatcher<R> {
    /// Create a new agent dispatcher with the given registry.
    pub fn new(registry: Arc<R>) -> Self {
        Self::with_config(registry, DispatchConfig::default())
    }

    /// Create a new agent dispatcher with custom configuration.
    pub fn with_config(registry: Arc<R>, config: DispatchConfig) -> Self {
        info!("Creating agent dispatcher with config: {:?}", config);
        Self {
            registry,
            router: Router::with_strategy(config.routing_strategy),
            config,
            distributed_queue: None,
        }
    }

    /// Set the distributed task queue for remote task execution.
    pub fn with_distributed_queue<Q>(mut self, queue: Q) -> Self
    where
        Q: DistributedTaskQueue + 'static,
    {
        self.distributed_queue = Some(Arc::new(queue));
        self
    }

    /// Set a Redis connection for distributed task execution.
    /// This is a convenience method that creates a RedisTaskQueue internally.
    pub fn with_redis_connection(mut self, connection: redis::aio::MultiplexedConnection) -> Self {
        let redis_queue = queue::RedisTaskQueue::new(connection);
        self.distributed_queue = Some(Arc::new(redis_queue));
        self
    }

    /// Get the current configuration.
    pub fn config(&self) -> &DispatchConfig {
        &self.config
    }

    /// Dispatch a task to an appropriate agent.
    ///
    /// This method:
    /// 1. Finds suitable agents for the task
    /// 2. Selects the best agent using the routing strategy
    /// 3. Executes the task while preserving SignerContext
    /// 4. Handles retries on retriable failures
    ///
    /// # Arguments
    ///
    /// * `task` - The task to dispatch
    ///
    /// # Returns
    ///
    /// The task result from the executing agent.
    pub async fn dispatch_task(&self, mut task: Task) -> Result<TaskResult> {
        debug!("Dispatching task {} of type {:?}", task.id, task.task_type);

        for attempt in 0..=self.config.max_retries {
            if attempt > 0 {
                debug!("Retry attempt {} for task {}", attempt, task.id);
                tokio::time::sleep(self.config.retry_delay).await;
            }

            match self.try_dispatch_task(&task).await {
                Ok(result) => {
                    debug!("Task {} completed successfully", task.id);
                    return Ok(result);
                }
                Err(err) if err.is_retriable() && attempt < self.config.max_retries => {
                    warn!(
                        "Task {} failed on attempt {}, will retry: {}",
                        task.id,
                        attempt + 1,
                        err
                    );
                    task.increment_retry();
                    continue;
                }
                Err(err) => {
                    error!(
                        "Task {} failed permanently after {} attempts: {}",
                        task.id,
                        attempt + 1,
                        err
                    );
                    return Err(err);
                }
            }
        }

        Err(AgentError::task_execution(format!(
            "Task {} exhausted all retry attempts",
            task.id
        )))
    }

    /// Attempt to dispatch a task once.
    async fn try_dispatch_task(&self, task: &Task) -> Result<TaskResult> {
        // Find suitable agents
        let agents = self.find_suitable_agents(task).await?;
        if agents.is_empty() {
            return Err(AgentError::no_suitable_agent(task.task_type.to_string()));
        }

        // Select the best agent using routing strategy
        let selected_agent = self.router.select_agent(&agents, task).await?;

        debug!(
            "Selected agent {} for task {} ({})",
            selected_agent.id(),
            task.id,
            if selected_agent.is_local() {
                "local execution"
            } else {
                "remote execution"
            }
        );

        // Handle local vs remote execution differently
        match selected_agent {
            AgentProxy::Local(agent) => {
                // Local agent - execute directly
                self.execute_local_task(agent, task).await
            }
            AgentProxy::Remote(status) => {
                // Remote agent - enqueue task for remote execution
                self.execute_remote_task(status, task).await
            }
        }
    }

    /// Execute a task on a local agent.
    async fn execute_local_task(&self, agent: Arc<dyn Agent>, task: &Task) -> Result<TaskResult> {
        let task_timeout = task.timeout.unwrap_or(self.config.default_task_timeout);

        match timeout(task_timeout, agent.execute_task(task.clone())).await {
            Ok(Ok(result)) => {
                info!(
                    "Task {} completed locally by agent {} in {:?}",
                    task.id,
                    agent.id(),
                    if let TaskResult::Success { duration, .. } = &result {
                        Some(*duration)
                    } else {
                        None
                    }
                );
                Ok(result)
            }
            Ok(Err(err)) => {
                warn!(
                    "Task {} failed locally in agent {}: {}",
                    task.id,
                    agent.id(),
                    err
                );
                Err(err)
            }
            Err(_) => {
                error!(
                    "Task {} timed out after {:?} in local agent {}",
                    task.id,
                    task_timeout,
                    agent.id()
                );
                Err(AgentError::task_timeout(task.id.clone(), task_timeout))
            }
        }
    }

    /// Execute a task on a remote agent.
    async fn execute_remote_task(
        &self,
        status: crate::AgentStatus,
        task: &Task,
    ) -> Result<TaskResult> {
        // Check if we have a distributed queue for remote execution
        if let Some(queue) = &self.distributed_queue {
            // Determine task timeout
            let task_timeout = task.timeout.unwrap_or(self.config.default_task_timeout);

            // Dispatch the task to the remote agent via queue
            info!(
                "Dispatching task {} to remote agent {} via distributed queue",
                task.id, status.agent_id
            );

            queue
                .dispatch_remote_task(&status.agent_id, task.clone(), task_timeout)
                .await
        } else {
            // No distributed queue available, return an error
            warn!(
                "Cannot execute task {} on remote agent {} - no distributed queue configured",
                task.id, status.agent_id
            );

            Err(AgentError::configuration(
                "Remote task execution requires a distributed task queue".to_string(),
            ))
        }
    }

    /// Find agents suitable for executing the given task.
    async fn find_suitable_agents(&self, task: &Task) -> Result<Vec<AgentProxy>> {
        debug!("Finding suitable agents for task type {:?}", task.task_type);

        let capability = crate::util::task_type_to_capability(&task.task_type);

        // Get all agent statuses (local and remote) with the required capability
        let agent_statuses = self
            .registry
            .find_agent_statuses_by_capability(&capability.to_string())
            .await?;

        let mut suitable_proxies = Vec::new();

        // For each status, check if we have a local instance
        for status in agent_statuses {
            // Try to get local agent instance
            if let Ok(Some(local_agent)) = self.registry.get_agent(&status.agent_id).await {
                // We have a local instance
                suitable_proxies.push(AgentProxy::Local(local_agent));
            } else {
                // This is a remote agent
                suitable_proxies.push(AgentProxy::Remote(status));
            }
        }

        // Filter by availability and other criteria
        suitable_proxies.retain(|proxy| proxy.is_available() && proxy.can_handle(task));

        if self.config.enable_load_balancing {
            // Sort by load for load-aware routing strategies
            suitable_proxies.sort_by(|a, b| {
                a.load()
                    .partial_cmp(&b.load())
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
        }

        debug!(
            "Found {} suitable agents for task {} ({} local, {} remote)",
            suitable_proxies.len(),
            task.id,
            suitable_proxies.iter().filter(|p| p.is_local()).count(),
            suitable_proxies.iter().filter(|p| p.is_remote()).count()
        );

        Ok(suitable_proxies)
    }

    /// Dispatch multiple tasks concurrently.
    ///
    /// # Arguments
    ///
    /// * `tasks` - Vector of tasks to dispatch
    ///
    /// # Returns
    ///
    /// Vector of task results in the same order as input tasks.
    pub async fn dispatch_tasks(&self, tasks: Vec<Task>) -> Vec<Result<TaskResult>> {
        info!("Dispatching {} tasks concurrently", tasks.len());

        let futures = tasks
            .into_iter()
            .map(|task| Box::pin(self.dispatch_task(task)));

        futures::future::join_all(futures).await
    }

    /// Get statistics about the dispatcher.
    pub async fn stats(&self) -> Result<DispatcherStats> {
        let agent_count = self.registry.agent_count().await?;
        let statuses = self.registry.list_agent_statuses().await?;

        let total_active_tasks = statuses.iter().map(|s| s.active_tasks).sum();
        let average_load = if !statuses.is_empty() {
            statuses.iter().map(|s| s.load).sum::<f64>() / statuses.len() as f64
        } else {
            0.0
        };

        Ok(DispatcherStats {
            registered_agents: agent_count,
            total_active_tasks,
            average_agent_load: average_load,
            routing_strategy: self.config.routing_strategy,
        })
    }

    /// Health check for the dispatcher.
    pub async fn health_check(&self) -> Result<bool> {
        // Check registry health
        self.registry.health_check().await?;

        // Check if we have any agents registered
        let agent_count = self.registry.agent_count().await?;
        if agent_count == 0 {
            warn!("No agents registered in dispatcher");
            return Ok(false);
        }

        Ok(true)
    }
}

/// Statistics about dispatcher state.
#[derive(Debug, Clone, Default)]
pub struct DispatcherStats {
    /// Number of registered agents
    pub registered_agents: usize,
    /// Total active tasks across all agents
    pub total_active_tasks: u32,
    /// Average load across all agents
    pub average_agent_load: f64,
    /// Current routing strategy
    pub routing_strategy: RoutingStrategy,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::LocalAgentRegistry;
    use crate::types::*;

    #[derive(Clone, Debug)]
    struct MockAgent {
        id: AgentId,
        capabilities: Vec<CapabilityType>,
        should_fail: bool,
        execution_delay: Duration,
    }

    #[async_trait::async_trait]
    impl Agent for MockAgent {
        async fn execute_task(&self, task: Task) -> Result<TaskResult> {
            if self.execution_delay > Duration::from_millis(0) {
                tokio::time::sleep(self.execution_delay).await;
            }

            if self.should_fail {
                return Ok(TaskResult::failure(
                    "Mock agent failure".to_string(),
                    true, // retriable
                    Duration::from_millis(10),
                ));
            }

            Ok(TaskResult::success(
                serde_json::json!({
                    "agent_id": self.id.as_str(),
                    "task_id": task.id
                }),
                None,
                Duration::from_millis(100),
            ))
        }

        fn id(&self) -> &AgentId {
            &self.id
        }

        fn capabilities(&self) -> Vec<crate::CapabilityType> {
            self.capabilities.clone()
        }
    }

    #[tokio::test]
    async fn test_dispatcher_basic_dispatch() {
        let registry = Arc::new(LocalAgentRegistry::default());
        let dispatcher = AgentDispatcher::new(registry.clone());

        // Register a trading agent
        let agent = Arc::new(MockAgent {
            id: AgentId::new("trading-agent"),
            capabilities: vec![CapabilityType::Trading],
            should_fail: false,
            execution_delay: Duration::from_millis(10),
        });
        registry.register_agent(agent).await.unwrap();

        // Create and dispatch a trading task
        let task = Task::new(TaskType::Trading, serde_json::json!({"symbol": "BTC/USD"}));

        let result = dispatcher.dispatch_task(task).await.unwrap();
        assert!(result.is_success());
    }

    #[tokio::test]
    async fn test_dispatcher_no_suitable_agent() {
        let registry = Arc::new(LocalAgentRegistry::default());
        let dispatcher = AgentDispatcher::new(registry.clone());

        // Register a trading agent
        let agent = Arc::new(MockAgent {
            id: AgentId::new("trading-agent"),
            capabilities: vec![CapabilityType::Trading],
            should_fail: false,
            execution_delay: Duration::from_millis(0),
        });
        registry.register_agent(agent).await.unwrap();

        // Try to dispatch a research task (no suitable agent)
        let task = Task::new(
            TaskType::Research,
            serde_json::json!({"query": "market analysis"}),
        );

        let result = dispatcher.dispatch_task(task).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            AgentError::NoSuitableAgent { .. }
        ));
    }

    #[tokio::test]
    async fn test_dispatcher_retry_logic() {
        let registry = Arc::new(LocalAgentRegistry::default());
        let config = DispatchConfig {
            max_retries: 2,
            retry_delay: Duration::from_millis(10),
            ..Default::default()
        };
        let dispatcher = AgentDispatcher::with_config(registry.clone(), config);

        // Register an agent that always fails
        let agent = Arc::new(MockAgent {
            id: AgentId::new("failing-agent"),
            capabilities: vec![CapabilityType::Trading],
            should_fail: true,
            execution_delay: Duration::from_millis(0),
        });
        registry.register_agent(agent).await.unwrap();

        let task = Task::new(TaskType::Trading, serde_json::json!({"symbol": "BTC/USD"}));

        // Should retry and eventually return the failure result
        let result = dispatcher.dispatch_task(task).await.unwrap();
        assert!(!result.is_success());
        assert!(result.is_retriable());
    }

    #[tokio::test]
    async fn test_dispatcher_timeout() {
        let registry = Arc::new(LocalAgentRegistry::default());
        let config = DispatchConfig {
            default_task_timeout: Duration::from_millis(50),
            ..Default::default()
        };
        let dispatcher = AgentDispatcher::with_config(registry.clone(), config);

        // Register an agent with long execution delay
        let agent = Arc::new(MockAgent {
            id: AgentId::new("slow-agent"),
            capabilities: vec![CapabilityType::Trading],
            should_fail: false,
            execution_delay: Duration::from_millis(100), // Longer than timeout
        });
        registry.register_agent(agent).await.unwrap();

        let task = Task::new(TaskType::Trading, serde_json::json!({"symbol": "BTC/USD"}));

        let result = dispatcher.dispatch_task(task).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            AgentError::TaskTimeout { .. }
        ));
    }

    #[tokio::test]
    async fn test_dispatcher_concurrent_tasks() {
        let registry = Arc::new(LocalAgentRegistry::default());
        let dispatcher = AgentDispatcher::new(registry.clone());

        // Register multiple agents
        for i in 0..3 {
            let agent = Arc::new(MockAgent {
                id: AgentId::new(format!("agent-{}", i)),
                capabilities: vec![CapabilityType::Trading],
                should_fail: false,
                execution_delay: Duration::from_millis(10),
            });
            registry.register_agent(agent).await.unwrap();
        }

        // Create multiple tasks
        let tasks: Vec<Task> = (0..5)
            .map(|i| {
                Task::new(
                    TaskType::Trading,
                    serde_json::json!({"symbol": format!("COIN{}", i)}),
                )
            })
            .collect();

        let results = dispatcher.dispatch_tasks(tasks).await;
        assert_eq!(results.len(), 5);
        assert!(results.iter().all(|r| r.is_ok()));
    }

    #[tokio::test]
    async fn test_dispatcher_stats() {
        let registry = Arc::new(LocalAgentRegistry::default());
        let dispatcher = AgentDispatcher::new(registry.clone());

        let agent = Arc::new(MockAgent {
            id: AgentId::new("test-agent"),
            capabilities: vec![CapabilityType::Trading],
            should_fail: false,
            execution_delay: Duration::from_millis(0),
        });
        registry.register_agent(agent).await.unwrap();

        let stats = dispatcher.stats().await.unwrap();
        assert_eq!(stats.registered_agents, 1);
        assert_eq!(stats.routing_strategy, RoutingStrategy::Capability);
    }

    #[tokio::test]
    async fn test_dispatcher_health_check() {
        let registry = Arc::new(LocalAgentRegistry::default());
        let dispatcher = AgentDispatcher::new(registry.clone());

        // Should be unhealthy with no agents
        assert!(!dispatcher.health_check().await.unwrap());

        // Register an agent
        let agent = Arc::new(MockAgent {
            id: AgentId::new("test-agent"),
            capabilities: vec![CapabilityType::Trading],
            should_fail: false,
            execution_delay: Duration::from_millis(0),
        });
        registry.register_agent(agent).await.unwrap();

        // Should be healthy now
        assert!(dispatcher.health_check().await.unwrap());
    }

    // Additional tests for 100% coverage

    #[test]
    fn test_dispatch_config_default() {
        let config = DispatchConfig::default();
        assert_eq!(config.default_task_timeout, Duration::from_secs(300));
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.retry_delay, Duration::from_secs(1));
        assert_eq!(config.max_concurrent_tasks_per_agent, 10);
        assert!(config.enable_load_balancing);
        assert_eq!(config.routing_strategy, RoutingStrategy::Capability);
        assert_eq!(config.response_wait_timeout, Duration::from_secs(300));
    }

    #[test]
    fn test_routing_strategy_variants() {
        assert_eq!(RoutingStrategy::default(), RoutingStrategy::Capability);

        // Test all variants exist and can be compared
        let strategies = vec![
            RoutingStrategy::Capability,
            RoutingStrategy::RoundRobin,
            RoutingStrategy::LeastLoaded,
            RoutingStrategy::Random,
            RoutingStrategy::Direct,
        ];

        for strategy in strategies {
            assert_eq!(strategy, strategy); // Test PartialEq
        }
    }

    #[test]
    fn test_dispatcher_config_getter() {
        let registry = Arc::new(LocalAgentRegistry::default());
        let custom_config = DispatchConfig {
            default_task_timeout: Duration::from_secs(120),
            max_retries: 5,
            retry_delay: Duration::from_millis(500),
            max_concurrent_tasks_per_agent: 20,
            enable_load_balancing: false,
            routing_strategy: RoutingStrategy::RoundRobin,
            response_wait_timeout: Duration::from_secs(600),
        };
        let dispatcher = AgentDispatcher::with_config(registry, custom_config.clone());

        let config = dispatcher.config();
        assert_eq!(config.default_task_timeout, Duration::from_secs(120));
        assert_eq!(config.max_retries, 5);
        assert_eq!(config.retry_delay, Duration::from_millis(500));
        assert_eq!(config.max_concurrent_tasks_per_agent, 20);
        assert!(!config.enable_load_balancing);
        assert_eq!(config.routing_strategy, RoutingStrategy::RoundRobin);
        assert_eq!(config.response_wait_timeout, Duration::from_secs(600));
    }

    #[test]
    fn test_task_type_to_capability_all_variants() {
        assert_eq!(
            crate::util::task_type_to_capability(&TaskType::Trading),
            crate::CapabilityType::Trading
        );
        assert_eq!(
            crate::util::task_type_to_capability(&TaskType::Research),
            crate::CapabilityType::Research
        );
        assert_eq!(
            crate::util::task_type_to_capability(&TaskType::RiskAnalysis),
            crate::CapabilityType::RiskAnalysis
        );
        assert_eq!(
            crate::util::task_type_to_capability(&TaskType::Portfolio),
            crate::CapabilityType::Portfolio
        );
        assert_eq!(
            crate::util::task_type_to_capability(&TaskType::Monitoring),
            crate::CapabilityType::Monitoring
        );
        assert_eq!(
            crate::util::task_type_to_capability(&TaskType::Custom(
                "custom_capability".to_string()
            )),
            crate::CapabilityType::Custom("custom_capability".to_string())
        );
    }

    #[derive(Clone, Debug)]
    struct UnavailableMockAgent {
        id: AgentId,
        capabilities: Vec<CapabilityType>,
    }

    #[async_trait::async_trait]
    impl Agent for UnavailableMockAgent {
        async fn execute_task(&self, _task: Task) -> Result<TaskResult> {
            unreachable!("Unavailable agent should not execute tasks")
        }

        fn id(&self) -> &AgentId {
            &self.id
        }

        fn capabilities(&self) -> Vec<crate::CapabilityType> {
            self.capabilities.clone()
        }

        fn is_available(&self) -> bool {
            false // Always unavailable
        }
    }

    #[derive(Clone, Debug)]
    struct CannotHandleMockAgent {
        id: AgentId,
        capabilities: Vec<CapabilityType>,
    }

    #[async_trait::async_trait]
    impl Agent for CannotHandleMockAgent {
        async fn execute_task(&self, _task: Task) -> Result<TaskResult> {
            unreachable!("Agent that cannot handle should not execute tasks")
        }

        fn id(&self) -> &AgentId {
            &self.id
        }

        fn capabilities(&self) -> Vec<crate::CapabilityType> {
            self.capabilities.clone()
        }

        fn can_handle(&self, _task: &Task) -> bool {
            false // Cannot handle any tasks
        }
    }

    #[tokio::test]
    async fn test_find_suitable_agents_unavailable_agents() {
        let registry = Arc::new(LocalAgentRegistry::default());
        let dispatcher = AgentDispatcher::new(registry.clone());

        // Register an unavailable agent
        let agent = Arc::new(UnavailableMockAgent {
            id: AgentId::new("unavailable-agent"),
            capabilities: vec![CapabilityType::Trading],
        });
        registry.register_agent(agent).await.unwrap();

        let task = Task::new(TaskType::Trading, serde_json::json!({"symbol": "BTC/USD"}));

        // Should fail to find suitable agents because the agent is unavailable
        let result = dispatcher.dispatch_task(task).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            AgentError::NoSuitableAgent { .. }
        ));
    }

    #[tokio::test]
    async fn test_find_suitable_agents_cannot_handle() {
        let registry = Arc::new(LocalAgentRegistry::default());
        let dispatcher = AgentDispatcher::new(registry.clone());

        // Register an agent that cannot handle tasks
        let agent = Arc::new(CannotHandleMockAgent {
            id: AgentId::new("cannot-handle-agent"),
            capabilities: vec![CapabilityType::Trading],
        });
        registry.register_agent(agent).await.unwrap();

        let task = Task::new(TaskType::Trading, serde_json::json!({"symbol": "BTC/USD"}));

        // Should fail to find suitable agents because the agent cannot handle the task
        let result = dispatcher.dispatch_task(task).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            AgentError::NoSuitableAgent { .. }
        ));
    }

    #[derive(Clone, Debug)]
    struct NonRetriableMockAgent {
        id: AgentId,
        capabilities: Vec<CapabilityType>,
    }

    #[async_trait::async_trait]
    impl Agent for NonRetriableMockAgent {
        async fn execute_task(&self, _task: Task) -> Result<TaskResult> {
            Err(AgentError::task_execution(
                "Non-retriable error".to_string(),
            ))
        }

        fn id(&self) -> &AgentId {
            &self.id
        }

        fn capabilities(&self) -> Vec<crate::CapabilityType> {
            self.capabilities.clone()
        }
    }

    #[tokio::test]
    async fn test_dispatcher_non_retriable_error() {
        let registry = Arc::new(LocalAgentRegistry::default());
        let config = DispatchConfig {
            max_retries: 2,
            ..Default::default()
        };
        let dispatcher = AgentDispatcher::with_config(registry.clone(), config);

        // Register an agent that returns non-retriable errors
        let agent = Arc::new(NonRetriableMockAgent {
            id: AgentId::new("non-retriable-agent"),
            capabilities: vec![CapabilityType::Trading],
        });
        registry.register_agent(agent).await.unwrap();

        let task = Task::new(TaskType::Trading, serde_json::json!({"symbol": "BTC/USD"}));

        // Should fail immediately without retries for non-retriable errors
        let result = dispatcher.dispatch_task(task).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            AgentError::TaskExecution { .. }
        ));
    }

    #[tokio::test]
    async fn test_dispatcher_custom_task_timeout() {
        let registry = Arc::new(LocalAgentRegistry::default());
        let config = DispatchConfig {
            default_task_timeout: Duration::from_secs(300), // Long default
            ..Default::default()
        };
        let dispatcher = AgentDispatcher::with_config(registry.clone(), config);

        // Register an agent with delay
        let agent = Arc::new(MockAgent {
            id: AgentId::new("slow-agent"),
            capabilities: vec![CapabilityType::Trading],
            should_fail: false,
            execution_delay: Duration::from_millis(100),
        });
        registry.register_agent(agent).await.unwrap();

        // Create task with custom short timeout
        let mut task = Task::new(TaskType::Trading, serde_json::json!({"symbol": "BTC/USD"}));
        task.timeout = Some(Duration::from_millis(50)); // Shorter than execution delay

        let result = dispatcher.dispatch_task(task).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            AgentError::TaskTimeout { .. }
        ));
    }

    #[tokio::test]
    async fn test_dispatcher_load_balancing_disabled() {
        let registry = Arc::new(LocalAgentRegistry::default());
        let config = DispatchConfig {
            enable_load_balancing: false,
            ..Default::default()
        };
        let dispatcher = AgentDispatcher::with_config(registry.clone(), config);

        // Register agents with different loads
        for i in 0..3 {
            let agent = Arc::new(MockAgent {
                id: AgentId::new(format!("agent-{}", i)),
                capabilities: vec![CapabilityType::Trading],
                should_fail: false,
                execution_delay: Duration::from_millis(10),
            });
            registry.register_agent(agent).await.unwrap();
        }

        let task = Task::new(TaskType::Trading, serde_json::json!({"symbol": "BTC/USD"}));

        // Should succeed even with load balancing disabled
        let result = dispatcher.dispatch_task(task).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_dispatcher_stats_with_empty_registry() {
        let registry = Arc::new(LocalAgentRegistry::default());
        let dispatcher = AgentDispatcher::new(registry);

        let stats = dispatcher.stats().await.unwrap();
        assert_eq!(stats.registered_agents, 0);
        assert_eq!(stats.total_active_tasks, 0);
        assert_eq!(stats.average_agent_load, 0.0);
        assert_eq!(stats.routing_strategy, RoutingStrategy::Capability);
    }

    #[test]
    fn test_dispatcher_stats_struct() {
        let stats = DispatcherStats {
            registered_agents: 5,
            total_active_tasks: 10,
            average_agent_load: 2.5,
            routing_strategy: RoutingStrategy::LeastLoaded,
        };

        // Test the struct can be cloned and debugged
        let cloned_stats = stats.clone();
        assert_eq!(cloned_stats.registered_agents, 5);
        assert_eq!(cloned_stats.total_active_tasks, 10);
        assert_eq!(cloned_stats.average_agent_load, 2.5);
        assert_eq!(cloned_stats.routing_strategy, RoutingStrategy::LeastLoaded);

        // Test debug format
        let debug_str = format!("{:?}", stats);
        assert!(debug_str.contains("DispatcherStats"));
    }

    #[tokio::test]
    async fn test_dispatch_tasks_empty_vector() {
        let registry = Arc::new(LocalAgentRegistry::default());
        let dispatcher = AgentDispatcher::new(registry);

        let results = dispatcher.dispatch_tasks(vec![]).await;
        assert_eq!(results.len(), 0);
    }

    #[tokio::test]
    async fn test_dispatch_task_max_retries_exhausted() {
        let registry = Arc::new(LocalAgentRegistry::default());
        let config = DispatchConfig {
            max_retries: 1, // Only 1 retry
            retry_delay: Duration::from_millis(1),
            ..Default::default()
        };
        let dispatcher = AgentDispatcher::with_config(registry.clone(), config);

        // Register an agent that always fails retriably
        let agent = Arc::new(MockAgent {
            id: AgentId::new("always-failing-agent"),
            capabilities: vec![CapabilityType::Trading],
            should_fail: true, // Always return retriable failures
            execution_delay: Duration::from_millis(0),
        });
        registry.register_agent(agent).await.unwrap();

        let task = Task::new(TaskType::Trading, serde_json::json!({"symbol": "BTC/USD"}));

        // Should exhaust retries and get the failure result (not an error)
        let result = dispatcher.dispatch_task(task).await.unwrap();
        assert!(!result.is_success());
        assert!(result.is_retriable());
    }

    #[derive(Clone, Debug)]
    struct LoadedMockAgent {
        id: AgentId,
        capabilities: Vec<CapabilityType>,
        load: f64,
    }

    #[async_trait::async_trait]
    impl Agent for LoadedMockAgent {
        async fn execute_task(&self, task: Task) -> Result<TaskResult> {
            Ok(TaskResult::success(
                serde_json::json!({
                    "agent_id": self.id.as_str(),
                    "task_id": task.id
                }),
                None,
                Duration::from_millis(100),
            ))
        }

        fn id(&self) -> &AgentId {
            &self.id
        }

        fn capabilities(&self) -> Vec<crate::CapabilityType> {
            self.capabilities.clone()
        }

        fn load(&self) -> f64 {
            self.load
        }
    }

    #[tokio::test]
    async fn test_find_suitable_agents_with_load_balancing() {
        let registry = Arc::new(LocalAgentRegistry::default());
        let config = DispatchConfig {
            enable_load_balancing: true,
            ..Default::default()
        };
        let dispatcher = AgentDispatcher::with_config(registry.clone(), config);

        // Register agents with different loads
        let high_load_agent = Arc::new(LoadedMockAgent {
            id: AgentId::new("high-load-agent"),
            capabilities: vec![CapabilityType::Trading],
            load: 0.9,
        });
        let low_load_agent = Arc::new(LoadedMockAgent {
            id: AgentId::new("low-load-agent"),
            capabilities: vec![CapabilityType::Trading],
            load: 0.1,
        });

        registry.register_agent(high_load_agent).await.unwrap();
        registry.register_agent(low_load_agent).await.unwrap();

        let task = Task::new(TaskType::Trading, serde_json::json!({"symbol": "BTC/USD"}));

        // Should succeed and prefer lower load agent
        let result = dispatcher.dispatch_task(task).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_dispatch_task_success_with_duration() {
        let registry = Arc::new(LocalAgentRegistry::default());
        let dispatcher = AgentDispatcher::new(registry.clone());

        let agent = Arc::new(MockAgent {
            id: AgentId::new("duration-agent"),
            capabilities: vec![CapabilityType::Trading],
            should_fail: false,
            execution_delay: Duration::from_millis(10),
        });
        registry.register_agent(agent).await.unwrap();

        let task = Task::new(TaskType::Trading, serde_json::json!({"symbol": "BTC/USD"}));

        let result = dispatcher.dispatch_task(task).await.unwrap();
        assert!(result.is_success());

        // Verify the success result contains duration information
        if let TaskResult::Success { duration, .. } = result {
            assert!(duration > Duration::from_millis(0));
        }
    }
}
