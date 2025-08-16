//! Agent registry for managing agent discovery and lifecycle.
//!
//! The registry system provides a way to register, discover, and manage agents
//! in the riglr-agents system. It supports both local in-memory registries
//! and distributed registries for scaled deployments.

use crate::{Agent, AgentId, AgentStatus, Result};
use async_trait::async_trait;
use std::sync::Arc;

pub mod distributed;
pub mod local;

pub use distributed::DistributedAgentRegistry;
pub use local::LocalAgentRegistry;

/// Trait for agent registry implementations.
///
/// Registries manage the lifecycle and discovery of agents in the system.
/// They provide methods to register new agents, discover existing agents,
/// and query agent status and capabilities.
#[async_trait]
pub trait AgentRegistry: Send + Sync {
    /// Register a new agent in the registry.
    ///
    /// # Arguments
    ///
    /// * `agent` - The agent to register
    ///
    /// # Returns
    ///
    /// Ok(()) if registration was successful, Err otherwise.
    async fn register_agent(&self, agent: Arc<dyn Agent>) -> Result<()>;

    /// Unregister an agent from the registry.
    ///
    /// # Arguments
    ///
    /// * `agent_id` - The ID of the agent to unregister
    ///
    /// # Returns
    ///
    /// Ok(()) if unregistration was successful, Err otherwise.
    async fn unregister_agent(&self, agent_id: &AgentId) -> Result<()>;

    /// Get an agent by its ID.
    ///
    /// # Arguments
    ///
    /// * `agent_id` - The ID of the agent to retrieve
    ///
    /// # Returns
    ///
    /// The agent if found, None otherwise.
    async fn get_agent(&self, agent_id: &AgentId) -> Result<Option<Arc<dyn Agent>>>;

    /// List all registered agents.
    ///
    /// # Returns
    ///
    /// A vector of all registered agents.
    async fn list_agents(&self) -> Result<Vec<Arc<dyn Agent>>>;

    /// Find agents that can handle a specific capability.
    ///
    /// # Arguments
    ///
    /// * `capability` - The capability to search for
    ///
    /// # Returns
    ///
    /// A vector of agents that support the given capability.
    async fn find_agents_by_capability(&self, capability: &str) -> Result<Vec<Arc<dyn Agent>>>;

    /// Get the status of an agent.
    ///
    /// # Arguments
    ///
    /// * `agent_id` - The ID of the agent
    ///
    /// # Returns
    ///
    /// The agent's status if found, None otherwise.
    async fn get_agent_status(&self, agent_id: &AgentId) -> Result<Option<AgentStatus>>;

    /// Update the status of an agent.
    ///
    /// # Arguments
    ///
    /// * `status` - The new status for the agent
    ///
    /// # Returns
    ///
    /// Ok(()) if the update was successful, Err otherwise.
    async fn update_agent_status(&self, status: AgentStatus) -> Result<()>;

    /// Get all agent statuses.
    ///
    /// # Returns
    ///
    /// A vector of all agent statuses.
    async fn list_agent_statuses(&self) -> Result<Vec<AgentStatus>>;

    /// Check if an agent is registered.
    ///
    /// # Arguments
    ///
    /// * `agent_id` - The ID of the agent to check
    ///
    /// # Returns
    ///
    /// true if the agent is registered, false otherwise.
    async fn is_agent_registered(&self, agent_id: &AgentId) -> Result<bool> {
        Ok(self.get_agent(agent_id).await?.is_some())
    }

    /// Get the number of registered agents.
    ///
    /// # Returns
    ///
    /// The count of registered agents.
    async fn agent_count(&self) -> Result<usize> {
        Ok(self.list_agents().await?.len())
    }

    /// Health check for the registry.
    ///
    /// # Returns
    ///
    /// true if the registry is healthy, false otherwise.
    async fn health_check(&self) -> Result<bool> {
        // Default implementation just checks if we can list agents
        self.list_agents().await.map(|_| true)
    }
}

/// Configuration for agent registry implementations.
#[derive(Debug, Clone)]
pub struct RegistryConfig {
    /// Maximum number of agents that can be registered
    pub max_agents: Option<usize>,
    /// Timeout for registry operations
    pub operation_timeout: std::time::Duration,
    /// Whether to enable health checks
    pub enable_health_checks: bool,
    /// Interval for background maintenance tasks
    pub maintenance_interval: std::time::Duration,
}

impl Default for RegistryConfig {
    fn default() -> Self {
        Self {
            max_agents: None,
            operation_timeout: std::time::Duration::from_secs(30),
            enable_health_checks: true,
            maintenance_interval: std::time::Duration::from_secs(60),
        }
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
    }

    #[async_trait::async_trait]
    impl Agent for MockAgent {
        async fn execute_task(&self, _task: crate::Task) -> Result<crate::TaskResult> {
            Ok(TaskResult::success(
                serde_json::json!({"result": "mock"}),
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

    async fn test_registry_basic_operations<R: AgentRegistry>(registry: R) {
        let agent = Arc::new(MockAgent {
            id: AgentId::new("test-agent"),
            capabilities: vec!["trading".to_string()],
        });

        // Test registration
        registry.register_agent(agent.clone()).await.unwrap();

        // Test retrieval
        let retrieved = registry
            .get_agent(&AgentId::new("test-agent"))
            .await
            .unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id(), &AgentId::new("test-agent"));

        // Test listing
        let agents = registry.list_agents().await.unwrap();
        assert_eq!(agents.len(), 1);

        // Test capability search
        let trading_agents = registry.find_agents_by_capability("trading").await.unwrap();
        assert_eq!(trading_agents.len(), 1);

        let research_agents = registry
            .find_agents_by_capability("research")
            .await
            .unwrap();
        assert_eq!(research_agents.len(), 0);

        // Test count
        assert_eq!(registry.agent_count().await.unwrap(), 1);

        // Test unregistration
        registry
            .unregister_agent(&AgentId::new("test-agent"))
            .await
            .unwrap();
        let retrieved = registry
            .get_agent(&AgentId::new("test-agent"))
            .await
            .unwrap();
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_local_registry() {
        let registry = LocalAgentRegistry::new();
        test_registry_basic_operations(registry).await;
    }
}
