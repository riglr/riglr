//! Routing engine for selecting agents based on different strategies.

use crate::{Agent, AgentError, Task, Result};
use super::RoutingStrategy;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use tracing::debug;

/// Router for selecting agents based on routing strategies.
pub struct Router {
    strategy: RoutingStrategy,
    round_robin_counter: AtomicUsize,
}

impl Router {
    /// Create a new router with the specified strategy.
    pub fn new(strategy: RoutingStrategy) -> Self {
        Self {
            strategy,
            round_robin_counter: AtomicUsize::new(0),
        }
    }

    /// Select an agent from the available agents using the configured strategy.
    ///
    /// # Arguments
    ///
    /// * `agents` - Available agents to choose from
    /// * `task` - The task to be executed (used for routing decisions)
    ///
    /// # Returns
    ///
    /// The selected agent.
    pub async fn select_agent(
        &self,
        agents: &[Arc<dyn Agent>],
        task: &Task,
    ) -> Result<Arc<dyn Agent>> {
        if agents.is_empty() {
            return Err(AgentError::no_suitable_agent(task.task_type.to_string()));
        }

        let selected = match self.strategy {
            RoutingStrategy::Capability => self.select_by_capability(agents, task).await?,
            RoutingStrategy::RoundRobin => self.select_round_robin(agents),
            RoutingStrategy::LeastLoaded => self.select_least_loaded(agents),
            RoutingStrategy::Random => self.select_random(agents),
            RoutingStrategy::Direct => self.select_direct(agents, task).await?,
        };

        debug!("Router selected agent {} using strategy {:?}", 
               selected.id(), self.strategy);

        Ok(selected)
    }

    /// Select agent based on capabilities (first capable agent).
    async fn select_by_capability(
        &self,
        agents: &[Arc<dyn Agent>],
        task: &Task,
    ) -> Result<Arc<dyn Agent>> {
        for agent in agents {
            if agent.can_handle(task) {
                return Ok(agent.clone());
            }
        }

        Err(AgentError::no_suitable_agent(task.task_type.to_string()))
    }

    /// Select agent using round-robin strategy.
    fn select_round_robin(&self, agents: &[Arc<dyn Agent>]) -> Arc<dyn Agent> {
        let index = self.round_robin_counter.fetch_add(1, Ordering::Relaxed) % agents.len();
        agents[index].clone()
    }

    /// Select the least loaded agent.
    fn select_least_loaded(&self, agents: &[Arc<dyn Agent>]) -> Arc<dyn Agent> {
        agents
            .iter()
            .min_by(|a, b| {
                a.load().partial_cmp(&b.load()).unwrap_or(std::cmp::Ordering::Equal)
            })
            .unwrap()
            .clone()
    }

    /// Select a random agent.
    fn select_random(&self, agents: &[Arc<dyn Agent>]) -> Arc<dyn Agent> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        // Use current time as seed for pseudo-randomness
        let mut hasher = DefaultHasher::new();
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
            .hash(&mut hasher);
        
        let index = (hasher.finish() as usize) % agents.len();
        agents[index].clone()
    }

    /// Select agent directly (for directed task routing).
    async fn select_direct(
        &self,
        agents: &[Arc<dyn Agent>],
        task: &Task,
    ) -> Result<Arc<dyn Agent>> {
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
        self.select_by_capability(agents, task).await
    }

    /// Get the current routing strategy.
    pub fn strategy(&self) -> RoutingStrategy {
        self.strategy
    }

    /// Change the routing strategy.
    pub fn set_strategy(&mut self, strategy: RoutingStrategy) {
        self.strategy = strategy;
        debug!("Router strategy changed to {:?}", strategy);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;

    #[derive(Clone)]
    struct MockAgent {
        id: AgentId,
        capabilities: Vec<String>,
        load: f64,
    }

    #[async_trait::async_trait]
    impl Agent for MockAgent {
        async fn execute_task(&self, _task: Task) -> Result<TaskResult> {
            Ok(TaskResult::success(
                serde_json::json!({}),
                None,
                std::time::Duration::from_millis(10),
            ))
        }

        fn id(&self) -> &AgentId {
            &self.id
        }

        fn capabilities(&self) -> Vec<String> {
            self.capabilities.clone()
        }

        fn load(&self) -> f64 {
            self.load
        }
    }

    #[tokio::test]
    async fn test_router_capability_strategy() {
        let router = Router::new(RoutingStrategy::Capability);
        
        let agents = vec![
            Arc::new(MockAgent {
                id: AgentId::new("agent1"),
                capabilities: vec!["research".to_string()],
                load: 0.5,
            }) as Arc<dyn Agent>,
            Arc::new(MockAgent {
                id: AgentId::new("agent2"),
                capabilities: vec!["trading".to_string()],
                load: 0.3,
            }) as Arc<dyn Agent>,
        ];

        let trading_task = Task::new(TaskType::Trading, serde_json::json!({}));
        let selected = router.select_agent(&agents, &trading_task).await.unwrap();
        assert_eq!(selected.id().as_str(), "agent2");

        let research_task = Task::new(TaskType::Research, serde_json::json!({}));
        let selected = router.select_agent(&agents, &research_task).await.unwrap();
        assert_eq!(selected.id().as_str(), "agent1");
    }

    #[tokio::test]
    async fn test_router_round_robin_strategy() {
        let router = Router::new(RoutingStrategy::RoundRobin);
        
        let agents = vec![
            Arc::new(MockAgent {
                id: AgentId::new("agent1"),
                capabilities: vec!["trading".to_string()],
                load: 0.8,
            }) as Arc<dyn Agent>,
            Arc::new(MockAgent {
                id: AgentId::new("agent2"),
                capabilities: vec!["trading".to_string()],
                load: 0.2,
            }) as Arc<dyn Agent>,
        ];

        let task = Task::new(TaskType::Trading, serde_json::json!({}));

        // Should alternate between agents
        let selected1 = router.select_agent(&agents, &task).await.unwrap();
        let selected2 = router.select_agent(&agents, &task).await.unwrap();
        
        assert_ne!(selected1.id(), selected2.id());
    }

    #[tokio::test]
    async fn test_router_least_loaded_strategy() {
        let router = Router::new(RoutingStrategy::LeastLoaded);
        
        let agents = vec![
            Arc::new(MockAgent {
                id: AgentId::new("high-load"),
                capabilities: vec!["trading".to_string()],
                load: 0.8,
            }) as Arc<dyn Agent>,
            Arc::new(MockAgent {
                id: AgentId::new("low-load"),
                capabilities: vec!["trading".to_string()],
                load: 0.2,
            }) as Arc<dyn Agent>,
        ];

        let task = Task::new(TaskType::Trading, serde_json::json!({}));
        let selected = router.select_agent(&agents, &task).await.unwrap();
        
        // Should select the least loaded agent
        assert_eq!(selected.id().as_str(), "low-load");
    }

    #[tokio::test]
    async fn test_router_direct_strategy() {
        let router = Router::new(RoutingStrategy::Direct);
        
        let agents = vec![
            Arc::new(MockAgent {
                id: AgentId::new("agent1"),
                capabilities: vec!["trading".to_string()],
                load: 0.5,
            }) as Arc<dyn Agent>,
            Arc::new(MockAgent {
                id: AgentId::new("agent2"),
                capabilities: vec!["trading".to_string()],
                load: 0.3,
            }) as Arc<dyn Agent>,
        ];

        // Task with direct agent targeting
        let task = Task::new(TaskType::Trading, serde_json::json!({}))
            .with_metadata("target_agent_id", serde_json::json!("agent2"));
        
        let selected = router.select_agent(&agents, &task).await.unwrap();
        assert_eq!(selected.id().as_str(), "agent2");

        // Task without direct targeting should fall back to capability-based
        let task_no_target = Task::new(TaskType::Trading, serde_json::json!({}));
        let selected = router.select_agent(&agents, &task_no_target).await.unwrap();
        assert!(selected.can_handle(&task_no_target));
    }

    #[tokio::test]
    async fn test_router_random_strategy() {
        let router = Router::new(RoutingStrategy::Random);
        
        let agents = vec![
            Arc::new(MockAgent {
                id: AgentId::new("agent1"),
                capabilities: vec!["trading".to_string()],
                load: 0.5,
            }) as Arc<dyn Agent>,
            Arc::new(MockAgent {
                id: AgentId::new("agent2"),
                capabilities: vec!["trading".to_string()],
                load: 0.3,
            }) as Arc<dyn Agent>,
        ];

        let task = Task::new(TaskType::Trading, serde_json::json!({}));
        
        // Should always select a valid agent
        let selected = router.select_agent(&agents, &task).await.unwrap();
        assert!(agents.iter().any(|a| a.id() == selected.id()));
    }

    #[tokio::test]
    async fn test_router_empty_agents() {
        let router = Router::new(RoutingStrategy::Capability);
        let agents: Vec<Arc<dyn Agent>> = vec![];
        let task = Task::new(TaskType::Trading, serde_json::json!({}));
        
        let result = router.select_agent(&agents, &task).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_router_strategy_change() {
        let mut router = Router::new(RoutingStrategy::Capability);
        assert_eq!(router.strategy(), RoutingStrategy::Capability);
        
        router.set_strategy(RoutingStrategy::LeastLoaded);
        assert_eq!(router.strategy(), RoutingStrategy::LeastLoaded);
    }
}