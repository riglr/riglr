//! Task dispatch and routing system for multi-agent coordination.
//!
//! The dispatcher is responsible for routing tasks to appropriate agents
//! based on capabilities, load, and routing rules. It maintains the
//! SignerContext security model while enabling sophisticated task routing.

pub mod router;

use crate::{Agent, AgentError, AgentRegistry, Result, Task, TaskResult, TaskType};
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
        }
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
            "Selected agent {} for task {}",
            selected_agent.id(),
            task.id
        );

        // Execute the task with timeout
        let task_timeout = task.timeout.unwrap_or(self.config.default_task_timeout);

        match timeout(task_timeout, selected_agent.execute_task(task.clone())).await {
            Ok(Ok(result)) => {
                info!(
                    "Task {} completed by agent {} in {:?}",
                    task.id,
                    selected_agent.id(),
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
                    "Task {} failed in agent {}: {}",
                    task.id,
                    selected_agent.id(),
                    err
                );
                Err(err)
            }
            Err(_) => {
                error!(
                    "Task {} timed out after {:?} in agent {}",
                    task.id,
                    task_timeout,
                    selected_agent.id()
                );
                Err(AgentError::task_timeout(task.id.clone(), task_timeout))
            }
        }
    }

    /// Find agents suitable for executing the given task.
    async fn find_suitable_agents(&self, task: &Task) -> Result<Vec<Arc<dyn Agent>>> {
        debug!("Finding suitable agents for task type {:?}", task.task_type);

        let capability = self.task_type_to_capability(&task.task_type);
        let mut suitable_agents = self.registry.find_agents_by_capability(&capability).await?;

        // Filter by availability and other criteria
        suitable_agents.retain(|agent| agent.is_available() && agent.can_handle(task));

        if self.config.enable_load_balancing {
            // Sort by load for load-aware routing strategies
            suitable_agents.sort_by(|a, b| {
                a.load()
                    .partial_cmp(&b.load())
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
        }

        debug!(
            "Found {} suitable agents for task {}",
            suitable_agents.len(),
            task.id
        );

        Ok(suitable_agents)
    }

    /// Convert task type to capability string.
    fn task_type_to_capability(&self, task_type: &TaskType) -> String {
        match task_type {
            TaskType::Trading => "trading".to_string(),
            TaskType::Research => "research".to_string(),
            TaskType::RiskAnalysis => "risk_analysis".to_string(),
            TaskType::Portfolio => "portfolio".to_string(),
            TaskType::Monitoring => "monitoring".to_string(),
            TaskType::Custom(name) => name.clone(),
        }
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
#[derive(Debug, Clone)]
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

    #[derive(Clone)]
    struct MockAgent {
        id: AgentId,
        capabilities: Vec<String>,
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

        fn capabilities(&self) -> Vec<String> {
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
            capabilities: vec!["trading".to_string()],
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
            capabilities: vec!["trading".to_string()],
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
            capabilities: vec!["trading".to_string()],
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
            capabilities: vec!["trading".to_string()],
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
                capabilities: vec!["trading".to_string()],
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
            capabilities: vec!["trading".to_string()],
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
            capabilities: vec!["trading".to_string()],
            should_fail: false,
            execution_delay: Duration::from_millis(0),
        });
        registry.register_agent(agent).await.unwrap();

        // Should be healthy now
        assert!(dispatcher.health_check().await.unwrap());
    }
}
