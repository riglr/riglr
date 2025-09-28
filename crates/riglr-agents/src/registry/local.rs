/// Local in-memory agent registry implementation.
extern crate alloc;

use super::{Config, Registry};
use crate::types::AgentState;
use crate::{Agent, AgentError, AgentId, AgentStatus, CapabilityType, Result};
use alloc::sync::Arc;
use async_trait::async_trait;
use core::str::FromStr as _;
use std::collections::HashMap;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// In-memory agent registry for single-node deployments.
///
/// This registry stores all agent information in memory and provides
/// fast access to agent data. It's suitable for development, testing,
/// and single-node production deployments.
#[derive(Debug)]
pub struct Local {
    /// Registered agents
    agents: RwLock<HashMap<AgentId, Arc<dyn Agent>>>,
    /// Registry configuration
    config: Config,
    /// Agent status information
    statuses: RwLock<HashMap<AgentId, AgentStatus>>,
}

impl Local {
    /// Get the current configuration.
    #[inline]
    pub const fn config(&self) -> &Config {
        &self.config
    }

    /// Create a new local agent registry with default configuration.
    #[must_use]
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Get statistics about the registry.
    #[inline]
    pub async fn stats(&self) -> Stats {
        let agents = self.agents.read().await;
        let statuses = self.statuses.read().await;

        Stats {
            active_agents: statuses
                .values()
                .filter(|status| matches!(status.status, AgentState::Active))
                .count(),
            busy_agents: statuses
                .values()
                .filter(|status| matches!(status.status, AgentState::Busy))
                .count(),
            idle_agents: statuses
                .values()
                .filter(|status| matches!(status.status, AgentState::Idle))
                .count(),
            offline_agents: statuses
                .values()
                .filter(|status| matches!(status.status, AgentState::Offline))
                .count(),
            total_agents: agents.len(),
        }
    }

    /// Create a new local agent registry with custom configuration.
    #[inline]
    pub fn with_config(config: Config) -> Self {
        info!("Creating local agent registry with config: {:?}", config);
        Self {
            agents: RwLock::default(),
            config,
            statuses: RwLock::default(),
        }
    }
}

impl Default for Local {
    #[inline]
    fn default() -> Self {
        let config = Config::default();
        info!("Creating local agent registry with config: {:?}", config);
        Self {
            agents: RwLock::default(),
            config,
            statuses: RwLock::default(),
        }
    }
}

#[async_trait]
impl Registry for Local {
    #[inline]
    async fn agent_count(&self) -> Result<usize> {
        let agents = self.agents.read().await;
        Ok(agents.len())
    }

    #[inline]
    async fn find_agent_statuses_by_capability(
        &self,
        capability: &str,
    ) -> Result<Vec<AgentStatus>> {
        let matching_agent_ids: Vec<AgentId> = {
            let agents = self.agents.read().await;
            agents
                .iter()
                .filter_map(|(agent_id, agent)| {
                    let cap_type = CapabilityType::from_str(capability).ok()?;
                    agent
                        .capabilities()
                        .contains(&cap_type)
                        .then(|| agent_id.clone())
                })
                .collect()
        };

        let statuses = self.statuses.read().await;
        let matching_statuses = matching_agent_ids
            .into_iter()
            .filter_map(|agent_id| statuses.get(&agent_id).cloned())
            .collect();

        Ok(matching_statuses)
    }

    #[inline]
    async fn find_agents_by_capability(&self, capability: &str) -> Result<Vec<Arc<dyn Agent>>> {
        debug!("Finding agents with capability: {}", capability);

        let matching_agents: Vec<Arc<dyn Agent>> = {
            let agents = self.agents.read().await;
            agents
                .values()
                .filter(|agent| {
                    let cap_type = CapabilityType::from_str(capability)
                        .unwrap_or_else(|_| CapabilityType::Custom(capability.to_string()));
                    agent.capabilities().contains(&cap_type)
                })
                .cloned()
                .collect()
        };

        debug!(
            "Found {} agents with capability '{}': {:?}",
            matching_agents.len(),
            capability,
            matching_agents
                .iter()
                .map(|agent| agent.id().as_str())
                .collect::<Vec<_>>()
        );

        Ok(matching_agents)
    }

    #[inline]
    async fn get_agent(&self, agent_id: &AgentId) -> Result<Option<Arc<dyn Agent>>> {
        let agents = self.agents.read().await;
        Ok(agents.get(agent_id).cloned())
    }

    #[inline]
    async fn get_agent_status(&self, agent_id: &AgentId) -> Result<Option<AgentStatus>> {
        let statuses = self.statuses.read().await;
        Ok(statuses.get(agent_id).cloned())
    }

    #[inline]
    async fn health_check(&self) -> Result<bool> {
        // For local registry, we just check if we can access the data structures
        let _agents = self.agents.read().await;
        let _statuses = self.statuses.read().await;
        Ok(true)
    }

    #[inline]
    async fn is_agent_registered(&self, agent_id: &AgentId) -> Result<bool> {
        let agents = self.agents.read().await;
        Ok(agents.contains_key(agent_id))
    }

    #[inline]
    async fn list_agent_statuses(&self) -> Result<Vec<AgentStatus>> {
        let statuses = self.statuses.read().await;
        Ok(statuses.values().cloned().collect())
    }

    #[inline]
    async fn list_agents(&self) -> Result<Vec<Arc<dyn Agent>>> {
        let agents = self.agents.read().await;
        Ok(agents.values().cloned().collect())
    }

    #[inline]
    async fn register_agent(&self, agent: Arc<dyn Agent>) -> Result<()> {
        let agent_id = agent.id().clone();

        debug!("Registering agent: {}", agent_id);

        // Check capacity limits
        if let Some(max_agents) = self.config.max_agents {
            let current_count = self.agents.read().await.len();
            if current_count >= max_agents {
                warn!(
                    "Cannot register agent {}: registry at capacity ({}/{})",
                    agent_id, current_count, max_agents
                );
                Err(AgentError::registry(format!(
                    "Registry at capacity ({current_count}/{max_agents})"
                )))?;
            }
        }

        // Check if agent already exists and register in one operation
        {
            let mut agents = self.agents.write().await;
            if agents.contains_key(&agent_id) {
                warn!("Agent {} is already registered", agent_id);
                Err(AgentError::registry(format!(
                    "Agent {agent_id} is already registered"
                )))?;
            }
            agents.insert(agent_id.clone(), Arc::<dyn Agent>::clone(&agent));
        };

        // Initialize agent status
        let status = agent.status();
        self.statuses.write().await.insert(agent_id.clone(), status);

        info!("Successfully registered agent: {}", agent_id);
        debug!(
            "Agent {} capabilities: {:?}",
            agent_id,
            agent.capabilities()
        );

        Ok(())
    }

    #[inline]
    async fn unregister_agent(&self, agent_id: &AgentId) -> Result<()> {
        debug!("Unregistering agent: {}", agent_id);

        let removed = self.agents.write().await.remove(agent_id);
        self.statuses.write().await.remove(agent_id);

        if removed.is_some() {
            info!("Successfully unregistered agent: {}", agent_id);
            Ok(())
        } else {
            warn!("Attempted to unregister non-existent agent: {}", agent_id);
            Err(AgentError::agent_not_found(agent_id.as_str()))
        }
    }

    #[inline]
    async fn update_agent_status(&self, status: AgentStatus) -> Result<()> {
        debug!("Updating status for agent: {}", status.agent_id);

        // Verify the agent exists
        if !self.agents.read().await.contains_key(&status.agent_id) {
            warn!(
                "Attempted to update status for non-existent agent: {}",
                status.agent_id
            );
            Err(AgentError::agent_not_found(status.agent_id.as_str()))?;
        }

        self.statuses
            .write()
            .await
            .insert(status.agent_id.clone(), status);
        Ok(())
    }
}

/// Statistics about the registry state.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Stats {
    /// Number of active agents
    pub active_agents: usize,
    /// Number of busy agents
    pub busy_agents: usize,
    /// Number of idle agents
    pub idle_agents: usize,
    /// Number of offline agents
    pub offline_agents: usize,
    /// Total number of registered agents
    pub total_agents: usize,
}

/// Type alias for agent storage.
pub type Storage = Local;

/// Type alias for statistics.
pub type Metrics = Stats;

#[cfg(test)]
#[expect(clippy::expect_used)]
mod tests {
    use super::*;
    use crate::types::*;
    use core::time::Duration;

    #[derive(Clone, Debug)]
    struct TestAgent {
        capabilities: Vec<CapabilityType>,
        id: AgentId,
    }

    #[async_trait]
    impl Agent for TestAgent {
        async fn execute_task(&self, _task: crate::Task) -> Result<crate::TaskResult> {
            Ok(TaskResult::success(
                serde_json::json!({"test": "result"}),
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
    }

    #[tokio::test]
    async fn test_local_registry_registration() {
        let registry = Local::default();
        let agent = Arc::new(TestAgent {
            id: AgentId::new("test-agent"),
            capabilities: vec![CapabilityType::Custom("trading".to_string())],
        });

        // Test successful registration
        registry
            .register_agent(agent.clone())
            .await
            .expect("Failed to register test agent");
        assert!(registry
            .is_agent_registered(&AgentId::new("test-agent"))
            .await
            .expect("Failed to check agent registration"));
        assert_eq!(
            registry
                .agent_count()
                .await
                .expect("Failed to get agent count"),
            1
        );

        // Test duplicate registration fails
        let result = registry.register_agent(agent).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_local_registry_unregistration() {
        let registry = Local::default();
        let agent = Arc::new(TestAgent {
            id: AgentId::new("test-agent"),
            capabilities: vec![CapabilityType::Custom("trading".to_string())],
        });

        // Register then unregister
        registry
            .register_agent(agent)
            .await
            .expect("Failed to register test agent in unregistration test");
        registry
            .unregister_agent(&AgentId::new("test-agent"))
            .await
            .expect("Failed to unregister test agent");
        assert!(!registry
            .is_agent_registered(&AgentId::new("test-agent"))
            .await
            .expect("Failed to check agent registration after unregistration"));

        // Test unregistering non-existent agent fails
        let result = registry
            .unregister_agent(&AgentId::new("non-existent"))
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_local_registry_capability_search() {
        let registry = Local::default();

        let trading_agent = Arc::new(TestAgent {
            id: AgentId::new("trading-agent"),
            capabilities: vec![CapabilityType::Trading, CapabilityType::RiskAnalysis],
        });

        let research_agent = Arc::new(TestAgent {
            id: AgentId::new("research-agent"),
            capabilities: vec![CapabilityType::Research, CapabilityType::Monitoring],
        });

        registry
            .register_agent(trading_agent)
            .await
            .expect("Failed to register trading agent");
        registry
            .register_agent(research_agent)
            .await
            .expect("Failed to register research agent");

        // Test capability searches
        let trading_agents = registry
            .find_agents_by_capability("trading")
            .await
            .expect("Failed to find trading agents");
        assert_eq!(trading_agents.len(), 1);
        assert_eq!(
            trading_agents
                .first()
                .expect("Trading agent should exist")
                .id()
                .as_str(),
            "trading-agent"
        );

        let research_agents = registry
            .find_agents_by_capability("research")
            .await
            .expect("Failed to find research agents");
        assert_eq!(research_agents.len(), 1);
        assert_eq!(
            research_agents
                .first()
                .expect("Research agent should exist")
                .id()
                .as_str(),
            "research-agent"
        );

        let risk_agents = registry
            .find_agents_by_capability("risk_analysis")
            .await
            .expect("Failed to find risk analysis agents");
        assert_eq!(risk_agents.len(), 1);

        let non_existent = registry
            .find_agents_by_capability("non_existent")
            .await
            .expect("Failed to find non-existent capability agents");
        assert_eq!(non_existent.len(), 0);
    }

    #[tokio::test]
    async fn test_local_registry_status_management() {
        let registry = Local::default();
        let agent = Arc::new(TestAgent {
            id: AgentId::new("test-agent"),
            capabilities: vec![CapabilityType::Custom("trading".to_string())],
        });

        registry
            .register_agent(agent)
            .await
            .expect("Failed to register test agent");

        // Test initial status
        let status = registry
            .get_agent_status(&AgentId::new("test-agent"))
            .await
            .expect("Failed to get agent status");
        assert!(status.is_some());

        // Test status update
        let mut new_status = status.expect("Agent status should exist");
        new_status.status = AgentState::Busy;
        new_status.active_tasks = 5;
        new_status.load = 0.8;

        registry
            .update_agent_status(new_status.clone())
            .await
            .expect("Failed to update agent status");

        let updated_status = registry
            .get_agent_status(&AgentId::new("test-agent"))
            .await
            .expect("Failed to get updated agent status");
        assert!(updated_status.is_some());
        let updated_status = updated_status.expect("Updated agent status should exist");
        assert!(matches!(updated_status.status, AgentState::Busy));
        assert_eq!(updated_status.active_tasks, 5);
        #[expect(clippy::float_cmp)]
        {
            assert_eq!(updated_status.load, 0.8);
        }
    }

    #[tokio::test]
    async fn test_local_registry_capacity_limits() {
        let config = Config {
            max_agents: Some(2),
            ..Config::default()
        };
        let registry = Local::with_config(config);

        // Register up to capacity
        let agent1 = Arc::new(TestAgent {
            id: AgentId::new("agent1"),
            capabilities: vec![CapabilityType::Custom("trading".to_string())],
        });
        let agent2 = Arc::new(TestAgent {
            id: AgentId::new("agent2"),
            capabilities: vec![CapabilityType::Custom("research".to_string())],
        });
        let agent3 = Arc::new(TestAgent {
            id: AgentId::new("agent3"),
            capabilities: vec![CapabilityType::Custom("monitoring".to_string())],
        });

        registry
            .register_agent(agent1)
            .await
            .expect("Failed to register agent1");
        registry
            .register_agent(agent2)
            .await
            .expect("Failed to register agent2");

        // Third registration should fail
        let result = registry.register_agent(agent3).await;
        assert!(result.is_err());
        assert_eq!(
            registry
                .agent_count()
                .await
                .expect("Failed to get agent count"),
            2
        );
    }

    #[tokio::test]
    async fn test_local_registry_stats() {
        let registry = Local::default();

        let agent1 = Arc::new(TestAgent {
            id: AgentId::new("agent1"),
            capabilities: vec![CapabilityType::Custom("trading".to_string())],
        });
        let agent2 = Arc::new(TestAgent {
            id: AgentId::new("agent2"),
            capabilities: vec![CapabilityType::Custom("research".to_string())],
        });

        registry
            .register_agent(agent1)
            .await
            .expect("Failed to register agent1");
        registry
            .register_agent(agent2)
            .await
            .expect("Failed to register agent2");

        // Update one agent status to busy
        let agent1_status =
            AgentStatus::new(AgentId::new("agent1"), AgentState::Busy, 2, 0.5, vec![]);
        registry
            .update_agent_status(agent1_status)
            .await
            .expect("Failed to update agent status");

        let registry_stats = registry.stats().await;
        assert_eq!(registry_stats.total_agents, 2);
        assert_eq!(registry_stats.busy_agents, 1);
        assert_eq!(registry_stats.idle_agents, 1);
    }

    #[tokio::test]
    async fn test_local_registry_health_check() {
        let registry = Local::default();
        assert!(registry
            .health_check()
            .await
            .expect("Failed to perform health check"));
    }

    #[tokio::test]
    async fn test_local_registry_new() {
        let registry = Local::default();
        assert_eq!(
            registry
                .agent_count()
                .await
                .expect("Failed to get agent count"),
            0
        );
        assert!(registry
            .health_check()
            .await
            .expect("Failed to perform health check"));
    }

    #[tokio::test]
    async fn test_local_registry_config() {
        let config = Config {
            max_agents: Some(10),
            ..Config::default()
        };
        let registry = Local::with_config(config);

        let retrieved_config = registry.config();
        assert_eq!(retrieved_config.max_agents, Some(10));
    }

    #[tokio::test]
    async fn test_stats_with_all_agent_states() {
        let registry = Local::default();

        // Create agents with different states
        let agent1 = Arc::new(TestAgent {
            id: AgentId::new("active-agent"),
            capabilities: vec![CapabilityType::Custom("trading".to_string())],
        });
        let agent2 = Arc::new(TestAgent {
            id: AgentId::new("busy-agent"),
            capabilities: vec![CapabilityType::Custom("research".to_string())],
        });
        let agent3 = Arc::new(TestAgent {
            id: AgentId::new("idle-agent"),
            capabilities: vec![CapabilityType::Custom("monitoring".to_string())],
        });
        let agent4 = Arc::new(TestAgent {
            id: AgentId::new("offline-agent"),
            capabilities: vec![CapabilityType::Custom("analysis".to_string())],
        });

        registry
            .register_agent(agent1)
            .await
            .expect("Failed to register agent1");
        registry
            .register_agent(agent2)
            .await
            .expect("Failed to register agent2");
        registry
            .register_agent(agent3)
            .await
            .expect("Failed to register agent3");
        registry
            .register_agent(agent4)
            .await
            .expect("Failed to register agent4");

        // Update statuses to different states
        let active_status = AgentStatus::new(
            AgentId::new("active-agent"),
            AgentState::Active,
            1,
            0.3,
            vec![],
        );

        let busy_status =
            AgentStatus::new(AgentId::new("busy-agent"), AgentState::Busy, 5, 0.9, vec![]);

        let idle_status =
            AgentStatus::new(AgentId::new("idle-agent"), AgentState::Idle, 0, 0.0, vec![]);

        let offline_status = AgentStatus::new(
            AgentId::new("offline-agent"),
            AgentState::Offline,
            0,
            0.0,
            vec![],
        );

        registry
            .update_agent_status(active_status)
            .await
            .expect("Failed to update active agent status");
        registry
            .update_agent_status(busy_status)
            .await
            .expect("Failed to update busy agent status");
        registry
            .update_agent_status(idle_status)
            .await
            .expect("Failed to update idle agent status");
        registry
            .update_agent_status(offline_status)
            .await
            .expect("Failed to update offline agent status");

        let stats = registry.stats().await;
        assert_eq!(stats.total_agents, 4);
        assert_eq!(stats.active_agents, 1);
        assert_eq!(stats.busy_agents, 1);
        assert_eq!(stats.idle_agents, 1);
        assert_eq!(stats.offline_agents, 1);
    }

    #[tokio::test]
    async fn test_stats_with_empty_registry() {
        let registry = Local::default();
        let stats = registry.stats().await;

        assert_eq!(stats.total_agents, 0);
        assert_eq!(stats.active_agents, 0);
        assert_eq!(stats.busy_agents, 0);
        assert_eq!(stats.idle_agents, 0);
        assert_eq!(stats.offline_agents, 0);
    }

    #[tokio::test]
    async fn test_get_agent_existing() {
        let registry = Local::default();
        let agent = Arc::new(TestAgent {
            id: AgentId::new("test-agent"),
            capabilities: vec![CapabilityType::Custom("trading".to_string())],
        });

        registry
            .register_agent(agent.clone())
            .await
            .expect("Failed to register test agent");

        let retrieved_agent = registry
            .get_agent(&AgentId::new("test-agent"))
            .await
            .expect("Failed to get test agent");
        assert!(retrieved_agent.is_some());
        assert_eq!(
            retrieved_agent.expect("Agent should exist").id().as_str(),
            "test-agent"
        );
    }

    #[tokio::test]
    async fn test_get_agent_non_existing() {
        let registry = Local::default();

        let retrieved_agent = registry
            .get_agent(&AgentId::new("non-existent"))
            .await
            .expect("Failed to get non-existent agent");
        assert!(retrieved_agent.is_none());
    }

    #[tokio::test]
    async fn test_list_agents_empty() {
        let registry = Local::default();

        let agents = registry
            .list_agents()
            .await
            .expect("Failed to list empty agents");
        assert_eq!(agents.len(), 0);
    }

    #[tokio::test]
    async fn test_list_agents_populated() {
        let registry = Local::default();

        let agent1 = Arc::new(TestAgent {
            id: AgentId::new("agent1"),
            capabilities: vec![CapabilityType::Custom("trading".to_string())],
        });
        let agent2 = Arc::new(TestAgent {
            id: AgentId::new("agent2"),
            capabilities: vec![CapabilityType::Custom("research".to_string())],
        });

        registry
            .register_agent(agent1)
            .await
            .expect("Failed to register agent1");
        registry
            .register_agent(agent2)
            .await
            .expect("Failed to register agent2");

        let agent_list = registry
            .list_agents()
            .await
            .expect("Failed to list populated agents");
        assert_eq!(agent_list.len(), 2);

        let agent_ids: Vec<String> = agent_list.iter().map(|a| a.id().to_string()).collect();
        assert!(agent_ids.contains(&"agent1".to_string()));
        assert!(agent_ids.contains(&"agent2".to_string()));
    }

    #[tokio::test]
    async fn test_list_agent_statuses_empty() {
        let registry = Local::default();

        let statuses = registry
            .list_agent_statuses()
            .await
            .expect("Failed to list agent statuses");
        assert_eq!(statuses.len(), 0);
    }

    #[tokio::test]
    async fn test_list_agent_statuses_populated() {
        let registry = Local::default();

        let agent1 = Arc::new(TestAgent {
            id: AgentId::new("agent1"),
            capabilities: vec![CapabilityType::Custom("trading".to_string())],
        });
        let agent2 = Arc::new(TestAgent {
            id: AgentId::new("agent2"),
            capabilities: vec![CapabilityType::Custom("research".to_string())],
        });

        registry
            .register_agent(agent1)
            .await
            .expect("Failed to register agent1");
        registry
            .register_agent(agent2)
            .await
            .expect("Failed to register agent2");

        let statuses = registry
            .list_agent_statuses()
            .await
            .expect("Failed to list agent statuses");
        assert_eq!(statuses.len(), 2);

        let status_agent_ids: Vec<String> =
            statuses.iter().map(|s| s.agent_id.to_string()).collect();
        assert!(status_agent_ids.contains(&"agent1".to_string()));
        assert!(status_agent_ids.contains(&"agent2".to_string()));
    }

    #[tokio::test]
    async fn test_update_agent_status_non_existent_agent() {
        let registry = Local::default();

        let status = AgentStatus::new(
            AgentId::new("non-existent"),
            AgentState::Active,
            1,
            0.5,
            vec![],
        );

        let result = registry.update_agent_status(status).await;
        assert!(result.is_err());

        if let Err(AgentError::AgentNotFound { agent_id }) = result {
            assert_eq!(agent_id, "non-existent");
        } else {
            unreachable!("Expected AgentNotFound error");
        }
    }

    #[tokio::test]
    async fn test_get_agent_status_non_existent() {
        let registry = Local::default();

        let status = registry
            .get_agent_status(&AgentId::new("non-existent"))
            .await
            .expect("Failed to get non-existent agent status");
        assert!(status.is_none());
    }

    #[tokio::test]
    async fn test_find_agents_by_capability_empty_capability() {
        let registry = Local::default();

        let agent = Arc::new(TestAgent {
            id: AgentId::new("test-agent"),
            capabilities: vec![CapabilityType::Custom("trading".to_string())],
        });
        registry
            .register_agent(agent)
            .await
            .expect("Failed to register test agent");

        let agents = registry
            .find_agents_by_capability("")
            .await
            .expect("Failed to find agents by empty capability");
        assert_eq!(agents.len(), 0);
    }

    #[tokio::test]
    async fn test_find_agents_by_capability_no_matching_agents() {
        let registry = Local::default();

        let agent = Arc::new(TestAgent {
            id: AgentId::new("test-agent"),
            capabilities: vec![CapabilityType::Custom("trading".to_string())],
        });
        registry
            .register_agent(agent)
            .await
            .expect("Failed to register test agent");

        let agents = registry
            .find_agents_by_capability("non_existent_capability")
            .await
            .expect("Failed to find agents by non-existent capability");
        assert_eq!(agents.len(), 0);
    }

    #[tokio::test]
    async fn test_registry_with_no_max_agents_limit() {
        let config = Config {
            max_agents: None,
            ..Config::default()
        };
        let registry = Local::with_config(config);

        // Should be able to register many agents without limit
        for i in 0..100 {
            let agent = Arc::new(TestAgent {
                id: AgentId::new(format!("agent{i}")),
                capabilities: vec![CapabilityType::Custom("trading".to_string())],
            });
            registry
                .register_agent(agent)
                .await
                .expect("Failed to register agent in capacity test");
        }

        assert_eq!(
            registry
                .agent_count()
                .await
                .expect("Failed to get agent count"),
            100
        );
    }

    #[tokio::test]
    async fn test_capacity_limit_edge_case_exact_limit() {
        let config = Config {
            max_agents: Some(1),
            ..Config::default()
        };
        let registry = Local::with_config(config);

        let agent1 = Arc::new(TestAgent {
            id: AgentId::new("agent1"),
            capabilities: vec![CapabilityType::Custom("trading".to_string())],
        });
        let agent2 = Arc::new(TestAgent {
            id: AgentId::new("agent2"),
            capabilities: vec![CapabilityType::Custom("research".to_string())],
        });

        // First registration should succeed
        registry
            .register_agent(agent1)
            .await
            .expect("Failed to register agent1");
        assert_eq!(
            registry
                .agent_count()
                .await
                .expect("Failed to get agent count"),
            1
        );

        // Second registration should fail
        let result = registry.register_agent(agent2).await;
        assert!(result.is_err());
        assert_eq!(
            registry
                .agent_count()
                .await
                .expect("Failed to get agent count"),
            1
        );
    }

    #[tokio::test]
    async fn test_is_agent_registered_false() {
        let registry = Local::default();

        let is_registered = registry
            .is_agent_registered(&AgentId::new("non-existent"))
            .await
            .expect("Failed to check if agent is registered");
        assert!(!is_registered);
    }

    #[tokio::test]
    async fn test_agent_count_after_operations() {
        let registry = Local::default();

        // Initially empty
        assert_eq!(
            registry
                .agent_count()
                .await
                .expect("Failed to get agent count"),
            0
        );

        // Add one agent
        let agent1 = Arc::new(TestAgent {
            id: AgentId::new("agent1"),
            capabilities: vec![CapabilityType::Custom("trading".to_string())],
        });
        registry
            .register_agent(agent1)
            .await
            .expect("Failed to register agent1");
        assert_eq!(
            registry
                .agent_count()
                .await
                .expect("Failed to get agent count"),
            1
        );

        // Add another agent
        let agent2 = Arc::new(TestAgent {
            id: AgentId::new("agent2"),
            capabilities: vec![CapabilityType::Custom("research".to_string())],
        });
        registry
            .register_agent(agent2)
            .await
            .expect("Failed to register agent2");
        assert_eq!(
            registry
                .agent_count()
                .await
                .expect("Failed to get agent count"),
            2
        );

        // Remove one agent
        registry
            .unregister_agent(&AgentId::new("agent1"))
            .await
            .expect("Failed to unregister agent1");
        assert_eq!(
            registry
                .agent_count()
                .await
                .expect("Failed to get agent count"),
            1
        );

        // Remove last agent
        registry
            .unregister_agent(&AgentId::new("agent2"))
            .await
            .expect("Failed to unregister agent2");
        assert_eq!(
            registry
                .agent_count()
                .await
                .expect("Failed to get agent count"),
            0
        );
    }
}
