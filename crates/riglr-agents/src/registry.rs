//! Agent registry for managing agent discovery and lifecycle.
//!
//! The registry system provides a way to register, discover, and manage agents
//! in the riglr-agents system. It supports both local in-memory registries
//! and distributed registries for scaled deployments.

pub mod distributed;
pub mod local;

use crate::{Agent, AgentId, AgentStatus, Result};
use async_trait::async_trait;
use core::time::Duration;
use std::sync::Arc;

pub use distributed::Redis;
pub use local::Local;

/// Trait for agent registry implementations.
///
/// Registries manage the lifecycle and discovery of agents in the system.
/// They provide methods to register new agents, discover existing agents,
/// and query agent status and capabilities.
#[async_trait]
pub trait Registry: Send + Sync {
    /// Get the number of registered agents.
    ///
    /// # Returns
    ///
    /// The count of registered agents.
    async fn agent_count(&self) -> Result<usize> {
        Ok(self.list_agents().await?.len())
    }

    /// Find agent statuses by capability (local and remote).
    ///
    /// This method returns status information for all agents that support
    /// the given capability, including both local agents in this process
    /// and remote agents in other processes (for distributed registries).
    ///
    /// # Arguments
    ///
    /// * `capability` - The capability to search for
    ///
    /// # Returns
    ///
    /// A vector of agent statuses for agents that support the given capability.
    async fn find_agent_statuses_by_capability(&self, capability: &str)
        -> Result<Vec<AgentStatus>>;

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

    /// Health check for the registry.
    ///
    /// # Returns
    ///
    /// true if the registry is healthy, false otherwise.
    async fn health_check(&self) -> Result<bool> {
        // Default implementation just checks if we can list agents
        self.list_agents().await.map(|_| true)
    }

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

    /// Get all agent statuses.
    ///
    /// # Returns
    ///
    /// A vector of all agent statuses.
    async fn list_agent_statuses(&self) -> Result<Vec<AgentStatus>>;

    /// List all registered agents.
    ///
    /// # Returns
    ///
    /// A vector of all registered agents.
    async fn list_agents(&self) -> Result<Vec<Arc<dyn Agent>>>;

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
}

/// Trait for distributed agent registry implementations.
///
/// This trait extends the base `Registry` trait with additional methods
/// specific to distributed coordination across multiple processes or machines.
///
/// Distributed registries provide capabilities for:
/// - Cross-process agent discovery
/// - Remote agent status monitoring
/// - Distributed capability-based routing
/// - Load balancing across multiple nodes
///
/// # Implementation Note
///
/// Implementations of this trait should handle network partitions gracefully
/// and provide eventual consistency guarantees for distributed operations.
#[async_trait]
pub trait Distributed: Registry {
    /// Find agent statuses by capability from the distributed registry.
    ///
    /// This method queries the distributed backend to find all agents
    /// (both local and remote) that support the specified capability.
    /// This is useful for load balancing and discovering agents across
    /// the entire distributed system.
    ///
    /// # Arguments
    ///
    /// * `capability` - The capability to search for
    ///
    /// # Returns
    ///
    /// A vector of agent statuses for agents that support the given capability
    /// across all processes and machines.
    ///
    /// # Errors
    ///
    /// Returns an error if the distributed backend is unavailable or
    /// if there's a network/serialization issue.
    async fn find_agent_statuses_by_capability_distributed(
        &self,
        capability: &str,
    ) -> Result<Vec<AgentStatus>>;

    /// List all agent statuses from the distributed registry.
    ///
    /// This method queries the distributed backend (e.g., Redis, NATS, database)
    /// to get status information for all registered agents across all processes
    /// and machines in the distributed system.
    ///
    /// # Returns
    ///
    /// A vector of all agent statuses in the distributed system.
    ///
    /// # Errors
    ///
    /// Returns an error if the distributed backend is unavailable or
    /// if there's a network/serialization issue.
    async fn list_agent_statuses_distributed(&self) -> Result<Vec<AgentStatus>>;
}

/// Configuration for agent registry implementations.
#[derive(Debug, Clone)]
pub struct Config {
    /// Whether to enable health checks
    pub enable_health_checks: bool,
    /// TTL for heartbeat entries in distributed registries
    pub heartbeat_ttl: Duration,
    /// Interval for background maintenance tasks
    pub maintenance_interval: Duration,
    /// Maximum number of agents that can be registered
    pub max_agents: Option<usize>,
    /// Timeout for registry operations
    pub operation_timeout: Duration,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            enable_health_checks: true,
            heartbeat_ttl: Duration::from_secs(600), // 10 minutes
            maintenance_interval: Duration::from_secs(60),
            max_agents: None,
            operation_timeout: Duration::from_secs(30),
        }
    }
}

/// Type alias for configuration.
pub type Settings = Config;

/// Type alias for Redis-based agent storage.
pub type RedisStorage = distributed::Redis;

/// Type alias for local agent storage.
pub type LocalStorage = local::Local;

#[cfg(test)]
#[expect(clippy::expect_used)]
mod tests {
    use super::*;
    use crate::types::*;
    use core::str::FromStr;
    use std::collections::HashMap;

    #[derive(Clone, Debug)]
    struct MockAgent {
        capabilities: Vec<CapabilityType>,
        id: AgentId,
    }

    #[async_trait::async_trait]
    impl Agent for MockAgent {
        async fn execute_task(&self, _task: crate::Task) -> Result<crate::TaskResult> {
            return Ok(TaskResult::success(
                serde_json::json!({"result": "mock"}),
                None,
                Duration::from_millis(10),
            ));
        }
        fn capabilities(&self) -> Vec<CapabilityType> {
            self.capabilities.clone()
        }
        fn id(&self) -> &AgentId {
            &self.id
        }
    }

    async fn test_registry_basic_operations<R: Registry>(registry: R) {
        let agent = Arc::new(MockAgent {
            id: AgentId::new("test-agent"),
            capabilities: vec![CapabilityType::Trading],
        });

        // Test registration
        registry
            .register_agent(agent.clone())
            .await
            .expect("Failed to register test agent");

        // Test retrieval
        let retrieved = registry
            .get_agent(&AgentId::new("test-agent"))
            .await
            .expect("Failed to get test agent");
        assert!(retrieved.is_some());
        assert_eq!(
            retrieved.expect("Test agent should be present").id(),
            &AgentId::new("test-agent")
        );

        // Test listing
        let agents = registry.list_agents().await.expect("Failed to list agents");
        assert_eq!(agents.len(), 1);

        // Test capability search
        let trading_agents = registry
            .find_agents_by_capability("trading")
            .await
            .expect("Failed to find trading agents");
        assert_eq!(trading_agents.len(), 1);

        let research_agents = registry
            .find_agents_by_capability("research")
            .await
            .expect("Failed to find research agents");
        assert_eq!(research_agents.len(), 0);

        // Test count
        assert_eq!(
            registry
                .agent_count()
                .await
                .expect("Failed to get agent count"),
            1
        );

        // Test unregistration
        registry
            .unregister_agent(&AgentId::new("test-agent"))
            .await
            .expect("Failed to unregister test agent");
        let retrieved = registry
            .get_agent(&AgentId::new("test-agent"))
            .await
            .expect("Failed to get agent after unregister");
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_local_registry() {
        let registry = local::Local::default();
        test_registry_basic_operations(registry).await;
    }

    #[test]
    fn test_registry_config_default() {
        let config = Config::default();

        assert_eq!(config.max_agents, None);
        assert_eq!(config.operation_timeout, Duration::from_secs(30));
        assert!(config.enable_health_checks);
        assert_eq!(config.maintenance_interval, Duration::from_secs(60));
    }

    #[test]
    fn test_registry_config_custom_values() {
        let config = Config {
            enable_health_checks: false,
            heartbeat_ttl: Duration::from_secs(300),
            maintenance_interval: Duration::from_secs(120),
            max_agents: Some(100),
            operation_timeout: Duration::from_secs(60),
        };

        assert_eq!(config.max_agents, Some(100));
        assert_eq!(config.operation_timeout, Duration::from_secs(60));
        assert!(!config.enable_health_checks);
        assert_eq!(config.maintenance_interval, Duration::from_secs(120));
    }

    #[test]
    fn test_registry_config_edge_cases() {
        let config = Config {
            enable_health_checks: true,
            heartbeat_ttl: Duration::from_millis(1),
            maintenance_interval: Duration::from_millis(1),
            max_agents: Some(0),
            operation_timeout: Duration::from_millis(1),
        };

        assert_eq!(config.max_agents, Some(0));
        assert_eq!(config.operation_timeout, Duration::from_millis(1));
        assert!(config.enable_health_checks);
        assert_eq!(config.maintenance_interval, Duration::from_millis(1));
    }

    #[test]
    fn test_registry_config_clone() {
        let config = Config::default();
        let cloned_config = config.clone();

        assert_eq!(config.max_agents, cloned_config.max_agents);
        assert_eq!(config.operation_timeout, cloned_config.operation_timeout);
        assert_eq!(
            config.enable_health_checks,
            cloned_config.enable_health_checks
        );
        assert_eq!(
            config.maintenance_interval,
            cloned_config.maintenance_interval
        );
    }

    #[test]
    fn test_mock_agent_capabilities() {
        let agent = MockAgent {
            id: AgentId::new("test"),
            capabilities: vec![CapabilityType::Trading, CapabilityType::Research],
        };

        let caps = agent.capabilities();
        assert_eq!(caps.len(), 2);
        assert!(caps.contains(&CapabilityType::Trading));
        assert!(caps.contains(&CapabilityType::Research));
    }

    #[test]
    fn test_mock_agent_id() {
        let agent = MockAgent {
            id: AgentId::new("test-id"),
            capabilities: vec![],
        };

        assert_eq!(agent.id(), &AgentId::new("test-id"));
    }

    #[test]
    fn test_mock_agent_clone() {
        let agent = MockAgent {
            id: AgentId::new("test"),
            capabilities: vec![CapabilityType::Trading],
        };

        let cloned = agent.clone();
        assert_eq!(agent.id(), cloned.id());
        assert_eq!(agent.capabilities(), cloned.capabilities());
    }

    #[tokio::test]
    async fn test_mock_agent_execute_task() {
        let agent = MockAgent {
            id: AgentId::new("test"),
            capabilities: vec![CapabilityType::Trading],
        };

        let task = crate::Task {
            created_at: chrono::Utc::now(),
            deadline: None,
            id: "test-task".to_string(),
            max_retries: 3,
            metadata: HashMap::new(),
            parameters: serde_json::json!({}),
            priority: crate::Priority::Normal,
            retry_count: 0,
            task_type: crate::TaskType::Custom("test".to_string()),
            timeout: Some(Duration::from_secs(30)),
        };

        let result = agent
            .execute_task(task)
            .await
            .expect("Failed to execute mock task");
        assert!(result.is_success());
        // Check if result has a duration field (depends on the enum variant)
        match result {
            crate::TaskResult::Success { duration, .. } => {
                assert!(duration.as_millis() >= 10);
            }
            _ => unreachable!("Expected successful task result"),
        }
    }

    // Mock registry for testing trait default implementations
    struct MockRegistry {
        agents: Vec<Arc<dyn Agent>>,
        should_error: bool,
    }

    #[async_trait]
    impl Registry for MockRegistry {
        async fn find_agents_by_capability(&self, capability: &str) -> Result<Vec<Arc<dyn Agent>>> {
            if self.should_error {
                return Err(crate::AgentError::configuration("Mock error".to_string()));
            }

            Ok(self
                .agents
                .iter()
                .filter(|a| {
                    // CapabilityType::from_str is infallible, but use expect for clarity
                    let cap_type = CapabilityType::from_str(capability)
                        .expect("CapabilityType::from_str is infallible");
                    a.capabilities().contains(&cap_type)
                })
                .cloned()
                .collect())
        }
        async fn find_agent_statuses_by_capability(
            &self,
            _capability: &str,
        ) -> Result<Vec<AgentStatus>> {
            if self.should_error {
                return Err(crate::AgentError::configuration("Mock error".to_string()));
            }
            Ok(vec![])
        }
        async fn get_agent(&self, agent_id: &AgentId) -> Result<Option<Arc<dyn Agent>>> {
            if self.should_error {
                return Err(crate::AgentError::configuration("Mock error".to_string()));
            }

            Ok(self.agents.iter().find(|a| a.id() == agent_id).cloned())
        }
        async fn get_agent_status(&self, _agent_id: &AgentId) -> Result<Option<AgentStatus>> {
            if self.should_error {
                return Err(crate::AgentError::configuration("Mock error".to_string()));
            }
            Ok(None)
        }
        async fn list_agents(&self) -> Result<Vec<Arc<dyn Agent>>> {
            if self.should_error {
                return Err(crate::AgentError::configuration("Mock error".to_string()));
            }
            Ok(self.agents.clone())
        }
        async fn list_agent_statuses(&self) -> Result<Vec<AgentStatus>> {
            if self.should_error {
                return Err(crate::AgentError::configuration("Mock error".to_string()));
            }
            Ok(vec![])
        }
        async fn register_agent(&self, _agent: Arc<dyn Agent>) -> Result<()> {
            if self.should_error {
                return Err(crate::AgentError::configuration("Mock error".to_string()));
            }
            Ok(())
        }
        async fn unregister_agent(&self, _agent_id: &AgentId) -> Result<()> {
            if self.should_error {
                return Err(crate::AgentError::configuration("Mock error".to_string()));
            }
            Ok(())
        }
        async fn update_agent_status(&self, _status: AgentStatus) -> Result<()> {
            if self.should_error {
                return Err(crate::AgentError::configuration("Mock error".to_string()));
            }
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_agent_registry_is_agent_registered_when_agent_exists_should_return_true() {
        let agent = Arc::new(MockAgent {
            id: AgentId::new("test-agent"),
            capabilities: vec![CapabilityType::Trading],
        });

        let registry = MockRegistry {
            agents: vec![agent],
            should_error: false,
        };

        let result = registry
            .is_agent_registered(&AgentId::new("test-agent"))
            .await
            .expect("Failed to check if agent is registered");
        assert!(result);
    }

    #[tokio::test]
    async fn test_agent_registry_is_agent_registered_when_agent_not_exists_should_return_false() {
        let registry = MockRegistry {
            agents: vec![],
            should_error: false,
        };

        let result = registry
            .is_agent_registered(&AgentId::new("non-existent"))
            .await
            .expect("Failed to check if non-existent agent is registered");
        assert!(!result);
    }

    #[tokio::test]
    async fn test_agent_registry_is_agent_registered_when_get_agent_fails_should_return_err() {
        let registry = MockRegistry {
            agents: vec![],
            should_error: true,
        };

        let result = registry.is_agent_registered(&AgentId::new("test")).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_agent_registry_agent_count_when_no_agents_should_return_zero() {
        let registry = MockRegistry {
            agents: vec![],
            should_error: false,
        };

        let count = registry
            .agent_count()
            .await
            .expect("Failed to get agent count");
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn test_agent_registry_agent_count_when_multiple_agents_should_return_correct_count() {
        let agent1 = Arc::new(MockAgent {
            id: AgentId::new("agent1"),
            capabilities: vec![CapabilityType::Trading],
        });
        let agent2 = Arc::new(MockAgent {
            id: AgentId::new("agent2"),
            capabilities: vec![CapabilityType::Research],
        });

        let registry = MockRegistry {
            agents: vec![agent1, agent2],
            should_error: false,
        };

        let count = registry
            .agent_count()
            .await
            .expect("Failed to get agent count");
        assert_eq!(count, 2);
    }

    #[tokio::test]
    async fn test_agent_registry_agent_count_when_list_agents_fails_should_return_err() {
        let registry = MockRegistry {
            agents: vec![],
            should_error: true,
        };

        let result = registry.agent_count().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_agent_registry_health_check_when_list_agents_succeeds_should_return_true() {
        let registry = MockRegistry {
            agents: vec![],
            should_error: false,
        };

        let health = registry
            .health_check()
            .await
            .expect("Failed to perform health check");
        assert!(health);
    }

    #[tokio::test]
    async fn test_agent_registry_health_check_when_list_agents_fails_should_return_err() {
        let registry = MockRegistry {
            agents: vec![],
            should_error: true,
        };

        let result = registry.health_check().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_agent_registry_health_check_when_has_agents_should_return_true() {
        let agent = Arc::new(MockAgent {
            id: AgentId::new("test-agent"),
            capabilities: vec![CapabilityType::Trading],
        });

        let registry = MockRegistry {
            agents: vec![agent],
            should_error: false,
        };

        let health = registry
            .health_check()
            .await
            .expect("Failed to perform health check");
        assert!(health);
    }
}
