//! Local in-memory agent registry implementation.

use super::{AgentRegistry, RegistryConfig};
use crate::{Agent, AgentError, AgentId, AgentStatus, Result};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// In-memory agent registry for single-node deployments.
///
/// This registry stores all agent information in memory and provides
/// fast access to agent data. It's suitable for development, testing,
/// and single-node production deployments.
pub struct LocalAgentRegistry {
    /// Registered agents
    agents: RwLock<HashMap<AgentId, Arc<dyn Agent>>>,
    /// Agent status information
    statuses: RwLock<HashMap<AgentId, AgentStatus>>,
    /// Registry configuration
    config: RegistryConfig,
}

impl LocalAgentRegistry {
    /// Create a new local agent registry with default configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new local agent registry with custom configuration.
    pub fn with_config(config: RegistryConfig) -> Self {
        info!("Creating local agent registry with config: {:?}", config);
        Self {
            agents: Default::default(),
            statuses: Default::default(),
            config,
        }
    }

    /// Get the current configuration.
    pub fn config(&self) -> &RegistryConfig {
        &self.config
    }

    /// Get statistics about the registry.
    pub async fn stats(&self) -> RegistryStats {
        let agents = self.agents.read().await;
        let statuses = self.statuses.read().await;

        RegistryStats {
            total_agents: agents.len(),
            active_agents: statuses
                .values()
                .filter(|s| matches!(s.status, crate::types::AgentState::Active))
                .count(),
            busy_agents: statuses
                .values()
                .filter(|s| matches!(s.status, crate::types::AgentState::Busy))
                .count(),
            idle_agents: statuses
                .values()
                .filter(|s| matches!(s.status, crate::types::AgentState::Idle))
                .count(),
            offline_agents: statuses
                .values()
                .filter(|s| matches!(s.status, crate::types::AgentState::Offline))
                .count(),
        }
    }
}

impl Default for LocalAgentRegistry {
    fn default() -> Self {
        let config = RegistryConfig::default();
        info!("Creating local agent registry with config: {:?}", config);
        Self {
            agents: Default::default(),
            statuses: Default::default(),
            config,
        }
    }
}

#[async_trait]
impl AgentRegistry for LocalAgentRegistry {
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
                return Err(AgentError::registry(format!(
                    "Registry at capacity ({}/{})",
                    current_count, max_agents
                )));
            }
        }

        let mut agents = self.agents.write().await;
        let mut statuses = self.statuses.write().await;

        // Check if agent already exists
        if agents.contains_key(&agent_id) {
            warn!("Agent {} is already registered", agent_id);
            return Err(AgentError::registry(format!(
                "Agent {} is already registered",
                agent_id
            )));
        }

        // Register the agent
        agents.insert(agent_id.clone(), agent.clone());

        // Initialize agent status
        let status = agent.status();
        statuses.insert(agent_id.clone(), status);

        info!("Successfully registered agent: {}", agent_id);
        debug!(
            "Agent {} capabilities: {:?}",
            agent_id,
            agent.capabilities()
        );

        Ok(())
    }

    async fn unregister_agent(&self, agent_id: &AgentId) -> Result<()> {
        debug!("Unregistering agent: {}", agent_id);

        let mut agents = self.agents.write().await;
        let mut statuses = self.statuses.write().await;

        let removed = agents.remove(agent_id);
        statuses.remove(agent_id);

        if removed.is_some() {
            info!("Successfully unregistered agent: {}", agent_id);
            Ok(())
        } else {
            warn!("Attempted to unregister non-existent agent: {}", agent_id);
            Err(AgentError::agent_not_found(agent_id.as_str()))
        }
    }

    async fn get_agent(&self, agent_id: &AgentId) -> Result<Option<Arc<dyn Agent>>> {
        let agents = self.agents.read().await;
        Ok(agents.get(agent_id).cloned())
    }

    async fn list_agents(&self) -> Result<Vec<Arc<dyn Agent>>> {
        let agents = self.agents.read().await;
        Ok(agents.values().cloned().collect())
    }

    async fn find_agents_by_capability(&self, capability: &str) -> Result<Vec<Arc<dyn Agent>>> {
        debug!("Finding agents with capability: {}", capability);

        let agents = self.agents.read().await;
        let matching_agents: Vec<Arc<dyn Agent>> = agents
            .values()
            .filter(|agent| agent.capabilities().contains(&capability.to_string()))
            .cloned()
            .collect();

        debug!(
            "Found {} agents with capability '{}': {:?}",
            matching_agents.len(),
            capability,
            matching_agents
                .iter()
                .map(|a| a.id().as_str())
                .collect::<Vec<_>>()
        );

        Ok(matching_agents)
    }

    async fn get_agent_status(&self, agent_id: &AgentId) -> Result<Option<AgentStatus>> {
        let statuses = self.statuses.read().await;
        Ok(statuses.get(agent_id).cloned())
    }

    async fn update_agent_status(&self, status: AgentStatus) -> Result<()> {
        debug!("Updating status for agent: {}", status.agent_id);

        let mut statuses = self.statuses.write().await;

        // Verify the agent exists
        if !self.agents.read().await.contains_key(&status.agent_id) {
            warn!(
                "Attempted to update status for non-existent agent: {}",
                status.agent_id
            );
            return Err(AgentError::agent_not_found(status.agent_id.as_str()));
        }

        statuses.insert(status.agent_id.clone(), status);
        Ok(())
    }

    async fn list_agent_statuses(&self) -> Result<Vec<AgentStatus>> {
        let statuses = self.statuses.read().await;
        Ok(statuses.values().cloned().collect())
    }

    async fn is_agent_registered(&self, agent_id: &AgentId) -> Result<bool> {
        let agents = self.agents.read().await;
        Ok(agents.contains_key(agent_id))
    }

    async fn agent_count(&self) -> Result<usize> {
        let agents = self.agents.read().await;
        Ok(agents.len())
    }

    async fn health_check(&self) -> Result<bool> {
        // For local registry, we just check if we can access the data structures
        let _agents = self.agents.read().await;
        let _statuses = self.statuses.read().await;
        Ok(true)
    }
}

/// Statistics about the registry state.
#[derive(Debug, Clone)]
pub struct RegistryStats {
    /// Total number of registered agents
    pub total_agents: usize,
    /// Number of active agents
    pub active_agents: usize,
    /// Number of busy agents
    pub busy_agents: usize,
    /// Number of idle agents
    pub idle_agents: usize,
    /// Number of offline agents
    pub offline_agents: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;

    #[derive(Clone)]
    struct TestAgent {
        id: AgentId,
        capabilities: Vec<String>,
    }

    #[async_trait]
    impl Agent for TestAgent {
        async fn execute_task(&self, _task: crate::Task) -> Result<crate::TaskResult> {
            Ok(TaskResult::success(
                serde_json::json!({"test": "result"}),
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
    }

    #[tokio::test]
    async fn test_local_registry_registration() {
        let registry = LocalAgentRegistry::new();
        let agent = Arc::new(TestAgent {
            id: AgentId::new("test-agent"),
            capabilities: vec!["trading".to_string()],
        });

        // Test successful registration
        registry.register_agent(agent.clone()).await.unwrap();
        assert!(registry
            .is_agent_registered(&AgentId::new("test-agent"))
            .await
            .unwrap());
        assert_eq!(registry.agent_count().await.unwrap(), 1);

        // Test duplicate registration fails
        let result = registry.register_agent(agent).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_local_registry_unregistration() {
        let registry = LocalAgentRegistry::new();
        let agent = Arc::new(TestAgent {
            id: AgentId::new("test-agent"),
            capabilities: vec!["trading".to_string()],
        });

        // Register then unregister
        registry.register_agent(agent).await.unwrap();
        registry
            .unregister_agent(&AgentId::new("test-agent"))
            .await
            .unwrap();
        assert!(!registry
            .is_agent_registered(&AgentId::new("test-agent"))
            .await
            .unwrap());

        // Test unregistering non-existent agent fails
        let result = registry
            .unregister_agent(&AgentId::new("non-existent"))
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_local_registry_capability_search() {
        let registry = LocalAgentRegistry::new();

        let trading_agent = Arc::new(TestAgent {
            id: AgentId::new("trading-agent"),
            capabilities: vec!["trading".to_string(), "risk_analysis".to_string()],
        });

        let research_agent = Arc::new(TestAgent {
            id: AgentId::new("research-agent"),
            capabilities: vec!["research".to_string(), "monitoring".to_string()],
        });

        registry.register_agent(trading_agent).await.unwrap();
        registry.register_agent(research_agent).await.unwrap();

        // Test capability searches
        let trading_agents = registry.find_agents_by_capability("trading").await.unwrap();
        assert_eq!(trading_agents.len(), 1);
        assert_eq!(trading_agents[0].id().as_str(), "trading-agent");

        let research_agents = registry
            .find_agents_by_capability("research")
            .await
            .unwrap();
        assert_eq!(research_agents.len(), 1);
        assert_eq!(research_agents[0].id().as_str(), "research-agent");

        let risk_agents = registry
            .find_agents_by_capability("risk_analysis")
            .await
            .unwrap();
        assert_eq!(risk_agents.len(), 1);

        let non_existent = registry
            .find_agents_by_capability("non_existent")
            .await
            .unwrap();
        assert_eq!(non_existent.len(), 0);
    }

    #[tokio::test]
    async fn test_local_registry_status_management() {
        let registry = LocalAgentRegistry::new();
        let agent = Arc::new(TestAgent {
            id: AgentId::new("test-agent"),
            capabilities: vec!["trading".to_string()],
        });

        registry.register_agent(agent).await.unwrap();

        // Test initial status
        let status = registry
            .get_agent_status(&AgentId::new("test-agent"))
            .await
            .unwrap();
        assert!(status.is_some());

        // Test status update
        let mut new_status = status.unwrap();
        new_status.status = AgentState::Busy;
        new_status.active_tasks = 5;
        new_status.load = 0.8;

        registry
            .update_agent_status(new_status.clone())
            .await
            .unwrap();

        let updated_status = registry
            .get_agent_status(&AgentId::new("test-agent"))
            .await
            .unwrap();
        assert!(updated_status.is_some());
        let updated_status = updated_status.unwrap();
        assert!(matches!(updated_status.status, AgentState::Busy));
        assert_eq!(updated_status.active_tasks, 5);
        assert_eq!(updated_status.load, 0.8);
    }

    #[tokio::test]
    async fn test_local_registry_capacity_limits() {
        let config = RegistryConfig {
            max_agents: Some(2),
            ..Default::default()
        };
        let registry = LocalAgentRegistry::with_config(config);

        // Register up to capacity
        let agent1 = Arc::new(TestAgent {
            id: AgentId::new("agent1"),
            capabilities: vec!["trading".to_string()],
        });
        let agent2 = Arc::new(TestAgent {
            id: AgentId::new("agent2"),
            capabilities: vec!["research".to_string()],
        });
        let agent3 = Arc::new(TestAgent {
            id: AgentId::new("agent3"),
            capabilities: vec!["monitoring".to_string()],
        });

        registry.register_agent(agent1).await.unwrap();
        registry.register_agent(agent2).await.unwrap();

        // Third registration should fail
        let result = registry.register_agent(agent3).await;
        assert!(result.is_err());
        assert_eq!(registry.agent_count().await.unwrap(), 2);
    }

    #[tokio::test]
    async fn test_local_registry_stats() {
        let registry = LocalAgentRegistry::new();

        let agent1 = Arc::new(TestAgent {
            id: AgentId::new("agent1"),
            capabilities: vec!["trading".to_string()],
        });
        let agent2 = Arc::new(TestAgent {
            id: AgentId::new("agent2"),
            capabilities: vec!["research".to_string()],
        });

        registry.register_agent(agent1).await.unwrap();
        registry.register_agent(agent2).await.unwrap();

        // Update one agent status to busy
        let status = AgentStatus {
            agent_id: AgentId::new("agent1"),
            status: AgentState::Busy,
            active_tasks: 2,
            load: 0.5,
            last_heartbeat: chrono::Utc::now(),
            capabilities: vec![],
            metadata: std::collections::HashMap::default(),
        };
        registry.update_agent_status(status).await.unwrap();

        let stats = registry.stats().await;
        assert_eq!(stats.total_agents, 2);
        assert_eq!(stats.busy_agents, 1);
        assert_eq!(stats.idle_agents, 1);
    }

    #[tokio::test]
    async fn test_local_registry_health_check() {
        let registry = LocalAgentRegistry::new();
        assert!(registry.health_check().await.unwrap());
    }
}
