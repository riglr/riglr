/// Routing engine for selecting agents based on different strategies.
use super::{proxy::Proxy, RoutingStrategy};
use crate::{AgentError, Result, Task};
use core::{
    sync::atomic::{AtomicUsize, Ordering},
    time::Duration,
};
use tracing::debug;

/// Router for selecting agents based on routing strategies.
#[derive(Debug, Default)]
pub struct Router {
    /// Round-robin counter for cycling through agents
    round_robin_counter: AtomicUsize,
    /// Routing strategy for selecting agents
    strategy: RoutingStrategy,
}

impl Router {
    /// Select an agent from the available agents using the configured strategy.
    ///
    /// # Arguments
    ///
    /// * `agents` - Available agent proxies to choose from
    /// * `task` - The task to be executed (used for routing decisions)
    ///
    /// # Returns
    ///
    /// The selected agent proxy.
    ///
    /// # Errors
    ///
    /// Returns an error if no suitable agent is found for the task.
    #[inline]
    pub fn select_agent(&self, agents: &[Proxy], task: &Task) -> Result<Proxy> {
        if agents.is_empty() {
            return Err(AgentError::no_suitable_agent(task.task_type.to_string()));
        }

        let selected = match self.strategy {
            RoutingStrategy::Capability => Self::select_by_capability(agents, task)?,
            RoutingStrategy::RoundRobin => self.select_round_robin(agents),
            RoutingStrategy::LeastLoaded => Self::select_least_loaded(agents),
            RoutingStrategy::Random => Self::select_random(agents),
            RoutingStrategy::Direct => Self::select_direct(agents, task)?,
        };

        debug!(
            "Router selected agent {} using strategy {:?} ({})",
            selected.id(),
            self.strategy,
            if selected.is_local() {
                "local"
            } else {
                "remote"
            }
        );

        Ok(selected)
    }

    /// Select agent based on capabilities (first capable agent).
    fn select_by_capability(agents: &[Proxy], task: &Task) -> Result<Proxy> {
        for agent in agents {
            if agent.can_handle(task) {
                return Ok(agent.clone());
            }
        }

        Err(AgentError::no_suitable_agent(task.task_type.to_string()))
    }

    /// Select agent directly (for directed task routing).
    fn select_direct(agents: &[Proxy], task: &Task) -> Result<Proxy> {
        // Look for agent ID in task metadata
        if let Some(target_agent_id) = task.metadata.get("target_agent_id") {
            if let Some(agent_id_str) = target_agent_id.as_str() {
                for agent in agents {
                    if agent.id().as_str() == agent_id_str {
                        return Ok(agent.clone());
                    }
                }
                return Err(AgentError::agent_not_found(agent_id_str));
            }
        }

        // Fall back to capability-based selection if no direct target
        Self::select_by_capability(agents, task)
    }

    /// Select the least loaded agent.
    fn select_least_loaded(agents: &[Proxy]) -> Proxy {
        use core::cmp::Ordering;

        // Since select_agent already checks for empty agents, we can safely access the first element
        agents
            .iter()
            .min_by(|first_agent, second_agent| {
                first_agent
                    .load()
                    .partial_cmp(&second_agent.load())
                    .unwrap_or(Ordering::Equal)
            })
            .map_or_else(
                || {
                    agents
                        .first()
                        .map_or_else(|| unreachable!("agents cannot be empty"), Clone::clone)
                },
                Clone::clone,
            )
    }

    /// Select a random agent.
    fn select_random(agents: &[Proxy]) -> Proxy {
        use core::hash::{Hash as _, Hasher as _};
        use std::collections::hash_map::DefaultHasher;
        use std::time::{SystemTime, UNIX_EPOCH};

        if agents.is_empty() {
            // This should never happen since select_agent checks for empty agents
            // but provide a safe fallback by creating a dummy proxy
            unreachable!("agents slice cannot be empty")
        }

        // Use current time as seed for pseudo-randomness
        let mut hasher = DefaultHasher::default();
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|_| Duration::from_nanos(0))
            .as_nanos()
            .hash(&mut hasher);

        #[expect(clippy::cast_possible_truncation)]
        #[expect(clippy::arithmetic_side_effects)]
        let index = (hasher.finish() as usize) % agents.len();
        agents.get(index).map_or_else(
            || {
                agents
                    .first()
                    .map_or_else(|| unreachable!("agents cannot be empty"), Clone::clone)
            },
            Clone::clone,
        )
    }

    /// Select agent using round-robin strategy.
    fn select_round_robin(&self, agents: &[Proxy]) -> Proxy {
        if agents.is_empty() {
            // This should never happen since select_agent checks for empty agents
            unreachable!("agents slice cannot be empty")
        }

        #[expect(clippy::arithmetic_side_effects)]
        let index = self.round_robin_counter.fetch_add(1, Ordering::Relaxed) % agents.len();
        agents.get(index).map_or_else(
            || {
                agents
                    .first()
                    .map_or_else(|| unreachable!("agents cannot be empty"), Clone::clone)
            },
            Clone::clone,
        )
    }

    /// Change the routing strategy.
    #[inline]
    pub fn set_strategy(&mut self, strategy: RoutingStrategy) {
        self.strategy = strategy;
        debug!("Router strategy changed to {:?}", strategy);
    }

    /// Get the current routing strategy.
    #[inline]
    pub const fn strategy(&self) -> RoutingStrategy {
        self.strategy
    }

    /// Create a new router with the specified strategy.
    #[inline]
    #[must_use]
    pub fn with_strategy(strategy: RoutingStrategy) -> Self {
        Self {
            strategy,
            round_robin_counter: AtomicUsize::default(),
        }
    }
}

#[cfg(test)]
#[expect(clippy::expect_used)]
mod tests {
    use super::*;
    use crate::types::*;
    use crate::Agent;
    use std::sync::Arc;

    #[derive(Clone, Debug)]
    struct MockAgent {
        capabilities: Vec<CapabilityType>,
        id: AgentId,
        load: f64,
    }

    #[async_trait::async_trait]
    impl Agent for MockAgent {
        async fn execute_task(&self, _task: Task) -> Result<TaskResult> {
            Ok(TaskResult::success(
                serde_json::json!({}),
                None,
                Duration::from_millis(10),
            ))
        }

        fn capabilities(&self) -> Vec<CapabilityType> {
            self.capabilities.clone()
        }

        fn id(&self) -> &AgentId {
            &self.id
        }

        fn load(&self) -> f64 {
            self.load
        }
    }

    #[tokio::test]
    async fn test_router_capability_strategy() {
        let router = Router::with_strategy(RoutingStrategy::Capability);

        let agents = vec![
            Proxy::Local(Arc::new(MockAgent {
                id: AgentId::new("agent1"),
                capabilities: vec![CapabilityType::Research],
                load: 0.5,
            })),
            Proxy::Local(Arc::new(MockAgent {
                id: AgentId::new("agent2"),
                capabilities: vec![CapabilityType::Trading],
                load: 0.3,
            })),
        ];

        let trading_task = Task::new(TaskType::Trading, serde_json::json!({}));
        let selected = router
            .select_agent(&agents, &trading_task)
            .expect("Router should select agent for trading task");
        assert_eq!(selected.id().as_str(), "agent2");

        let research_task = Task::new(TaskType::Research, serde_json::json!({}));
        let selected = router
            .select_agent(&agents, &research_task)
            .expect("Router should select agent for research task");
        assert_eq!(selected.id().as_str(), "agent1");
    }

    #[tokio::test]
    async fn test_router_round_robin_strategy() {
        let router = Router::with_strategy(RoutingStrategy::RoundRobin);

        let agents = vec![
            Proxy::Local(Arc::new(MockAgent {
                id: AgentId::new("agent1"),
                capabilities: vec![CapabilityType::Trading],
                load: 0.8,
            })),
            Proxy::Local(Arc::new(MockAgent {
                id: AgentId::new("agent2"),
                capabilities: vec![CapabilityType::Trading],
                load: 0.2,
            })),
        ];

        let task = Task::new(TaskType::Trading, serde_json::json!({}));

        // Should alternate between agents
        let selected1 = router
            .select_agent(&agents, &task)
            .expect("Router should select first agent in round-robin");
        let selected2 = router
            .select_agent(&agents, &task)
            .expect("Router should select second agent in round-robin");

        assert_ne!(selected1.id(), selected2.id());
    }

    #[tokio::test]
    async fn test_router_least_loaded_strategy() {
        let router = Router::with_strategy(RoutingStrategy::LeastLoaded);

        let agents = vec![
            Proxy::Local(Arc::new(MockAgent {
                id: AgentId::new("high-load"),
                capabilities: vec![CapabilityType::Trading],
                load: 0.8,
            })),
            Proxy::Local(Arc::new(MockAgent {
                id: AgentId::new("low-load"),
                capabilities: vec![CapabilityType::Trading],
                load: 0.2,
            })),
        ];

        let task = Task::new(TaskType::Trading, serde_json::json!({}));
        let selected = router
            .select_agent(&agents, &task)
            .expect("Router should select least loaded agent");

        // Should select the least loaded agent
        assert_eq!(selected.id().as_str(), "low-load");
    }

    #[tokio::test]
    async fn test_router_direct_strategy() {
        let router = Router::with_strategy(RoutingStrategy::Direct);

        let agents = vec![
            Proxy::Local(Arc::new(MockAgent {
                id: AgentId::new("agent1"),
                capabilities: vec![CapabilityType::Trading],
                load: 0.5,
            })),
            Proxy::Local(Arc::new(MockAgent {
                id: AgentId::new("agent2"),
                capabilities: vec![CapabilityType::Trading],
                load: 0.3,
            })),
        ];

        // Task with direct agent targeting
        let task = Task::new(TaskType::Trading, serde_json::json!({}))
            .with_metadata("target_agent_id", serde_json::json!("agent2"));

        let selected = router
            .select_agent(&agents, &task)
            .expect("Router should select agent with direct targeting");
        assert_eq!(selected.id().as_str(), "agent2");

        // Task without direct targeting should fall back to capability-based
        let task_no_target = Task::new(TaskType::Trading, serde_json::json!({}));
        let selected = router
            .select_agent(&agents, &task_no_target)
            .expect("Router should select agent via capability fallback");
        assert!(selected.can_handle(&task_no_target));
    }

    #[tokio::test]
    async fn test_router_random_strategy() {
        let router = Router::with_strategy(RoutingStrategy::Random);

        let agents = vec![
            Proxy::Local(Arc::new(MockAgent {
                id: AgentId::new("agent1"),
                capabilities: vec![CapabilityType::Trading],
                load: 0.5,
            })),
            Proxy::Local(Arc::new(MockAgent {
                id: AgentId::new("agent2"),
                capabilities: vec![CapabilityType::Trading],
                load: 0.3,
            })),
        ];

        let task = Task::new(TaskType::Trading, serde_json::json!({}));

        // Should always select a valid agent
        let selected = router
            .select_agent(&agents, &task)
            .expect("Router should select random agent");
        assert!(agents.iter().any(|a| a.id() == selected.id()));
    }

    #[tokio::test]
    async fn test_router_empty_agents() {
        let router = Router::with_strategy(RoutingStrategy::Capability);
        let agents: Vec<Proxy> = vec![];
        let task = Task::new(TaskType::Trading, serde_json::json!({}));

        let result = router.select_agent(&agents, &task);
        assert!(result.is_err());
    }

    #[test]
    fn test_router_strategy_change() {
        let mut router = Router::with_strategy(RoutingStrategy::Capability);
        assert_eq!(router.strategy(), RoutingStrategy::Capability);

        router.set_strategy(RoutingStrategy::LeastLoaded);
        assert_eq!(router.strategy(), RoutingStrategy::LeastLoaded);
    }

    #[test]
    fn test_router_default() {
        let router = Router::default();
        assert_eq!(router.strategy(), RoutingStrategy::Capability); // Default strategy
    }

    #[tokio::test]
    async fn test_capability_strategy_no_suitable_agent() {
        let router = Router::with_strategy(RoutingStrategy::Capability);

        let agents = vec![Proxy::Local(Arc::new(MockAgent {
            id: AgentId::new("agent1"),
            capabilities: vec![CapabilityType::Research],
            load: 0.5,
        }))];

        // Task that no agent can handle
        let unsupported_task = Task::new(TaskType::Trading, serde_json::json!({}));
        let result = router.select_agent(&agents, &unsupported_task);
        assert!(result.is_err());

        if let Err(AgentError::NoSuitableAgent { task_type }) = result {
            assert_eq!(task_type, "trading");
        } else {
            unreachable!("Expected NoSuitableAgent error");
        }
    }

    #[tokio::test]
    async fn test_direct_strategy_target_agent_not_string() {
        let router = Router::with_strategy(RoutingStrategy::Direct);

        let agents = vec![Proxy::Local(Arc::new(MockAgent {
            id: AgentId::new("agent1"),
            capabilities: vec![CapabilityType::Trading],
            load: 0.5,
        }))];

        // Task with non-string target_agent_id
        let task = Task::new(TaskType::Trading, serde_json::json!({}))
            .with_metadata("target_agent_id", serde_json::json!(123));

        let selected = router
            .select_agent(&agents, &task)
            .expect("Router should select agent with non-string target fallback");
        // Should fall back to capability-based selection
        assert_eq!(selected.id().as_str(), "agent1");
    }

    #[tokio::test]
    async fn test_direct_strategy_target_agent_not_found() {
        let router = Router::with_strategy(RoutingStrategy::Direct);

        let agents = vec![Proxy::Local(Arc::new(MockAgent {
            id: AgentId::new("agent1"),
            capabilities: vec![CapabilityType::Trading],
            load: 0.5,
        }))];

        // Task with target agent that doesn't exist
        let task = Task::new(TaskType::Trading, serde_json::json!({}))
            .with_metadata("target_agent_id", serde_json::json!("nonexistent"));

        let result = router.select_agent(&agents, &task);
        assert!(result.is_err());

        if let Err(AgentError::AgentNotFound { agent_id }) = result {
            assert_eq!(agent_id, "nonexistent");
        } else {
            unreachable!("Expected AgentNotFound error");
        }
    }

    #[tokio::test]
    async fn test_least_loaded_strategy_equal_loads() {
        let router = Router::with_strategy(RoutingStrategy::LeastLoaded);

        let agents = vec![
            Proxy::Local(Arc::new(MockAgent {
                id: AgentId::new("agent1"),
                capabilities: vec![CapabilityType::Trading],
                load: 0.5,
            })),
            Proxy::Local(Arc::new(MockAgent {
                id: AgentId::new("agent2"),
                capabilities: vec![CapabilityType::Trading],
                load: 0.5,
            })),
        ];

        let task = Task::new(TaskType::Trading, serde_json::json!({}));
        let selected = router
            .select_agent(&agents, &task)
            .expect("Router should select agent with equal loads");

        // Should select the first agent when loads are equal
        assert_eq!(selected.id().as_str(), "agent1");
    }

    #[tokio::test]
    async fn test_round_robin_with_single_agent() {
        let router = Router::with_strategy(RoutingStrategy::RoundRobin);

        let agents = vec![Proxy::Local(Arc::new(MockAgent {
            id: AgentId::new("only-agent"),
            capabilities: vec![CapabilityType::Trading],
            load: 0.5,
        }))];

        let task = Task::new(TaskType::Trading, serde_json::json!({}));

        // Should always select the only agent
        let selected1 = router
            .select_agent(&agents, &task)
            .expect("Router should select single agent first time");
        let selected2 = router
            .select_agent(&agents, &task)
            .expect("Router should select single agent second time");

        assert_eq!(selected1.id().as_str(), "only-agent");
        assert_eq!(selected2.id().as_str(), "only-agent");
    }

    #[tokio::test]
    async fn test_random_strategy_with_single_agent() {
        let router = Router::with_strategy(RoutingStrategy::Random);

        let agents = vec![Proxy::Local(Arc::new(MockAgent {
            id: AgentId::new("only-agent"),
            capabilities: vec![CapabilityType::Trading],
            load: 0.5,
        }))];

        let task = Task::new(TaskType::Trading, serde_json::json!({}));
        let selected = router
            .select_agent(&agents, &task)
            .expect("Router should select single agent in random strategy");

        assert_eq!(selected.id().as_str(), "only-agent");
    }

    #[tokio::test]
    async fn test_round_robin_counter_increment() {
        let router = Router::with_strategy(RoutingStrategy::RoundRobin);

        let agents = vec![
            Proxy::Local(Arc::new(MockAgent {
                id: AgentId::new("agent1"),
                capabilities: vec![CapabilityType::Trading],
                load: 0.5,
            })),
            Proxy::Local(Arc::new(MockAgent {
                id: AgentId::new("agent2"),
                capabilities: vec![CapabilityType::Trading],
                load: 0.5,
            })),
            Proxy::Local(Arc::new(MockAgent {
                id: AgentId::new("agent3"),
                capabilities: vec![CapabilityType::Trading],
                load: 0.5,
            })),
        ];

        let task = Task::new(TaskType::Trading, serde_json::json!({}));

        // Test that round-robin cycles through all agents
        let selected1 = router
            .select_agent(&agents, &task)
            .expect("Router should select first agent in cycle");
        let selected2 = router
            .select_agent(&agents, &task)
            .expect("Router should select second agent in cycle");
        let selected3 = router
            .select_agent(&agents, &task)
            .expect("Router should select third agent in cycle");
        let selected4 = router
            .select_agent(&agents, &task)
            .expect("Router should cycle back to first agent"); // Should cycle back

        assert_eq!(selected1.id().as_str(), "agent1");
        assert_eq!(selected2.id().as_str(), "agent2");
        assert_eq!(selected3.id().as_str(), "agent3");
        assert_eq!(selected4.id().as_str(), "agent1"); // Cycles back
    }

    #[tokio::test]
    async fn test_direct_strategy_fallback_no_suitable_agent() {
        let router = Router::with_strategy(RoutingStrategy::Direct);

        let agents = vec![Proxy::Local(Arc::new(MockAgent {
            id: AgentId::new("agent1"),
            capabilities: vec![CapabilityType::Research], // Cannot handle trading
            load: 0.5,
        }))];

        // Task without target_agent_id, should fall back to capability-based
        let task = Task::new(TaskType::Trading, serde_json::json!({}));
        let result = router.select_agent(&agents, &task);

        assert!(result.is_err());
        if let Err(AgentError::NoSuitableAgent { task_type }) = result {
            assert_eq!(task_type, "trading");
        } else {
            unreachable!("Expected NoSuitableAgent error");
        }
    }
}
