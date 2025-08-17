//! Builder patterns for easy agent system construction.
//!
//! This module provides fluent APIs for building and configuring
//! multi-agent systems with minimal boilerplate code.

use crate::{
    communication::{AgentCommunication, CommunicationConfig},
    dispatcher::RoutingStrategy,
    registry::RegistryConfig,
    Agent, AgentDispatcher, AgentError, AgentRegistry, ChannelCommunication, DispatchConfig,
    LocalAgentRegistry, Result,
};
use std::sync::Arc;
use std::time::Duration;

/// Builder for creating and configuring agent systems.
///
/// The AgentBuilder provides a fluent API for setting up complete
/// multi-agent systems with registries, dispatchers, and communication.
///
/// # Examples
///
/// ```rust
/// use riglr_agents::{AgentBuilder, RoutingStrategy};
/// use std::time::Duration;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
/// let system = AgentBuilder::default()
///     .with_max_agents(50)
///     .with_routing_strategy(RoutingStrategy::LeastLoaded)
///     .with_task_timeout(Duration::from_secs(300))
///     .build()
///     .await?;
/// # Ok(())
/// # }
/// ```
#[derive(Default)]
pub struct AgentBuilder {
    registry_config: RegistryConfig,
    dispatch_config: DispatchConfig,
    communication_config: CommunicationConfig,
}

impl AgentBuilder {
    /// Set the maximum number of agents in the registry.
    pub fn with_max_agents(mut self, max_agents: usize) -> Self {
        self.registry_config.max_agents = Some(max_agents);
        self
    }

    /// Set the registry operation timeout.
    pub fn with_registry_timeout(mut self, timeout: Duration) -> Self {
        self.registry_config.operation_timeout = timeout;
        self
    }

    /// Enable or disable registry health checks.
    pub fn with_health_checks(mut self, enabled: bool) -> Self {
        self.registry_config.enable_health_checks = enabled;
        self
    }

    /// Set the registry maintenance interval.
    pub fn with_maintenance_interval(mut self, interval: Duration) -> Self {
        self.registry_config.maintenance_interval = interval;
        self
    }

    /// Set the default task execution timeout.
    pub fn with_task_timeout(mut self, timeout: Duration) -> Self {
        self.dispatch_config.default_task_timeout = timeout;
        self
    }

    /// Set the maximum number of retry attempts for failed tasks.
    pub fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.dispatch_config.max_retries = max_retries;
        self
    }

    /// Set the delay between retry attempts.
    pub fn with_retry_delay(mut self, delay: Duration) -> Self {
        self.dispatch_config.retry_delay = delay;
        self
    }

    /// Set the maximum number of concurrent tasks per agent.
    pub fn with_max_concurrent_tasks(mut self, max_tasks: u32) -> Self {
        self.dispatch_config.max_concurrent_tasks_per_agent = max_tasks;
        self
    }

    /// Enable or disable load balancing.
    pub fn with_load_balancing(mut self, enabled: bool) -> Self {
        self.dispatch_config.enable_load_balancing = enabled;
        self
    }

    /// Set the routing strategy for task dispatch.
    pub fn with_routing_strategy(mut self, strategy: RoutingStrategy) -> Self {
        self.dispatch_config.routing_strategy = strategy;
        self
    }

    /// Set the maximum number of pending messages per agent.
    pub fn with_max_pending_messages(mut self, max_messages: usize) -> Self {
        self.communication_config.max_pending_messages = max_messages;
        self
    }

    /// Set the message time-to-live.
    pub fn with_message_ttl(mut self, ttl: Duration) -> Self {
        self.communication_config.message_ttl = ttl;
        self
    }

    /// Enable or disable message persistence.
    pub fn with_message_persistence(mut self, enabled: bool) -> Self {
        self.communication_config.enable_persistence = enabled;
        self
    }

    /// Set the channel buffer size for communication.
    pub fn with_channel_buffer_size(mut self, size: usize) -> Self {
        self.communication_config.channel_buffer_size = size;
        self
    }

    /// Set the maximum number of communication subscriptions.
    pub fn with_max_subscriptions(mut self, max_subs: usize) -> Self {
        self.communication_config.max_subscriptions = Some(max_subs);
        self
    }

    /// Build the agent system with the configured settings.
    ///
    /// # Returns
    ///
    /// A complete agent system ready for use.
    pub async fn build(self) -> Result<AgentSystem> {
        let registry = Arc::new(LocalAgentRegistry::with_config(self.registry_config));
        let dispatcher = AgentDispatcher::with_config(registry.clone(), self.dispatch_config);
        let communication = Arc::new(ChannelCommunication::with_config(self.communication_config));

        Ok(AgentSystem {
            registry,
            dispatcher,
            communication,
        })
    }

    /// Build with a custom registry implementation.
    ///
    /// # Arguments
    ///
    /// * `registry` - Custom registry implementation to use
    ///
    /// # Returns
    ///
    /// A complete agent system with the custom registry.
    pub async fn build_with_registry<R: AgentRegistry + 'static>(
        self,
        registry: Arc<R>,
    ) -> Result<CustomAgentSystem<R>> {
        let dispatcher = AgentDispatcher::with_config(registry.clone(), self.dispatch_config);
        let communication = Arc::new(ChannelCommunication::with_config(self.communication_config));

        Ok(CustomAgentSystem {
            registry,
            dispatcher,
            communication,
        })
    }
}

/// A complete agent system with local registry.
pub struct AgentSystem {
    /// Agent registry
    pub registry: Arc<LocalAgentRegistry>,
    /// Task dispatcher
    pub dispatcher: AgentDispatcher<LocalAgentRegistry>,
    /// Communication system
    pub communication: Arc<ChannelCommunication>,
}

impl AgentSystem {
    /// Register an agent in the system.
    pub async fn register_agent(&self, agent: Arc<dyn Agent>) -> Result<()> {
        self.registry.register_agent(agent).await
    }

    /// Register multiple agents in the system.
    pub async fn register_agents(&self, agents: Vec<Arc<dyn Agent>>) -> Result<()> {
        for agent in agents {
            self.register_agent(agent).await?;
        }
        Ok(())
    }

    /// Get the number of registered agents.
    pub async fn agent_count(&self) -> Result<usize> {
        self.registry.agent_count().await
    }

    /// Perform a health check on all system components.
    pub async fn health_check(&self) -> Result<SystemHealth> {
        let registry_healthy = self.registry.health_check().await?;
        let dispatcher_healthy = self.dispatcher.health_check().await?;
        let communication_healthy = self.communication.health_check().await?;

        Ok(SystemHealth {
            registry_healthy,
            dispatcher_healthy,
            communication_healthy,
            overall_healthy: registry_healthy && dispatcher_healthy && communication_healthy,
        })
    }

    /// Get system statistics.
    pub async fn stats(&self) -> Result<SystemStats> {
        let registry_stats = self.registry.stats().await;
        let dispatcher_stats = self.dispatcher.stats().await?;
        let communication_stats = self.communication.stats().await;

        Ok(SystemStats {
            registry_stats,
            dispatcher_stats,
            communication_stats,
        })
    }
}

/// A complete agent system with custom registry.
pub struct CustomAgentSystem<R: AgentRegistry> {
    /// Agent registry
    pub registry: Arc<R>,
    /// Task dispatcher
    pub dispatcher: AgentDispatcher<R>,
    /// Communication system
    pub communication: Arc<ChannelCommunication>,
}

impl<R: AgentRegistry> CustomAgentSystem<R> {
    /// Register an agent in the system.
    pub async fn register_agent(&self, agent: Arc<dyn Agent>) -> Result<()> {
        self.registry.register_agent(agent).await
    }

    /// Register multiple agents in the system.
    pub async fn register_agents(&self, agents: Vec<Arc<dyn Agent>>) -> Result<()> {
        for agent in agents {
            self.register_agent(agent).await?;
        }
        Ok(())
    }

    /// Get the number of registered agents.
    pub async fn agent_count(&self) -> Result<usize> {
        self.registry.agent_count().await
    }

    /// Perform a health check on all system components.
    pub async fn health_check(&self) -> Result<SystemHealth> {
        let registry_healthy = self.registry.health_check().await?;
        let dispatcher_healthy = self.dispatcher.health_check().await?;
        let communication_healthy = self.communication.health_check().await?;

        Ok(SystemHealth {
            registry_healthy,
            dispatcher_healthy,
            communication_healthy,
            overall_healthy: registry_healthy && dispatcher_healthy && communication_healthy,
        })
    }
}

/// Health status of the agent system.
#[derive(Debug, Clone)]
pub struct SystemHealth {
    /// Registry health status
    pub registry_healthy: bool,
    /// Dispatcher health status
    pub dispatcher_healthy: bool,
    /// Communication health status
    pub communication_healthy: bool,
    /// Overall system health
    pub overall_healthy: bool,
}

/// Combined statistics from all system components.
#[derive(Debug, Clone)]
pub struct SystemStats {
    /// Registry statistics
    pub registry_stats: crate::registry::local::RegistryStats,
    /// Dispatcher statistics
    pub dispatcher_stats: crate::dispatcher::DispatcherStats,
    /// Communication statistics
    pub communication_stats: crate::communication::CommunicationStats,
}

/// Builder for creating individual agents with common patterns.
#[derive(Default)]
pub struct SingleAgentBuilder {
    agent_id: Option<String>,
    capabilities: Vec<String>,
    metadata: std::collections::HashMap<String, serde_json::Value>,
}

impl SingleAgentBuilder {
    /// Set the agent ID.
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.agent_id = Some(id.into());
        self
    }

    /// Add a capability to the agent.
    pub fn with_capability(mut self, capability: impl Into<String>) -> Self {
        self.capabilities.push(capability.into());
        self
    }

    /// Add multiple capabilities to the agent.
    pub fn with_capabilities(mut self, capabilities: Vec<String>) -> Self {
        self.capabilities.extend(capabilities);
        self
    }

    /// Add metadata to the agent.
    pub fn with_metadata(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }

    /// Validate the builder configuration.
    pub fn validate(&self) -> Result<()> {
        if self.agent_id.is_none() {
            return Err(AgentError::configuration("Agent ID is required"));
        }

        if self.capabilities.is_empty() {
            return Err(AgentError::configuration(
                "At least one capability is required",
            ));
        }

        Ok(())
    }

    /// Get the configured agent ID.
    pub fn agent_id(&self) -> Option<&str> {
        self.agent_id.as_deref()
    }

    /// Get the configured capabilities.
    pub fn capabilities(&self) -> &[String] {
        &self.capabilities
    }

    /// Get the configured metadata.
    pub fn metadata(&self) -> &std::collections::HashMap<String, serde_json::Value> {
        &self.metadata
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_agent_builder_default() {
        let system = AgentBuilder::default().build().await.unwrap();

        assert_eq!(system.agent_count().await.unwrap(), 0);

        let health = system.health_check().await.unwrap();
        assert!(!health.overall_healthy); // No agents registered
        assert!(health.registry_healthy);
        assert!(health.communication_healthy);
    }

    #[tokio::test]
    async fn test_agent_builder_with_config() {
        let system = AgentBuilder::default()
            .with_max_agents(10)
            .with_task_timeout(Duration::from_secs(60))
            .with_max_retries(5)
            .with_routing_strategy(RoutingStrategy::LeastLoaded)
            .with_max_pending_messages(500)
            .build()
            .await
            .unwrap();

        // Verify configuration is applied (indirectly through behavior)
        assert_eq!(system.agent_count().await.unwrap(), 0);

        // The specific config values are internal, but we can verify the system works
        let health = system.health_check().await.unwrap();
        assert!(health.registry_healthy);
        assert!(health.communication_healthy);
    }

    #[tokio::test]
    async fn test_agent_system_with_agents() {
        use crate::types::*;

        #[derive(Clone)]
        struct TestAgent {
            id: crate::AgentId,
            capabilities: Vec<String>,
        }

        #[async_trait::async_trait]
        impl Agent for TestAgent {
            async fn execute_task(&self, _task: crate::Task) -> Result<crate::TaskResult> {
                Ok(TaskResult::success(
                    serde_json::json!({}),
                    None,
                    Duration::from_millis(10),
                ))
            }

            fn id(&self) -> &crate::AgentId {
                &self.id
            }

            fn capabilities(&self) -> Vec<String> {
                self.capabilities.clone()
            }
        }

        let system = AgentBuilder::default().build().await.unwrap();

        let agent = Arc::new(TestAgent {
            id: crate::AgentId::new("test-agent"),
            capabilities: vec!["trading".to_string()],
        });

        system.register_agent(agent).await.unwrap();
        assert_eq!(system.agent_count().await.unwrap(), 1);

        let health = system.health_check().await.unwrap();
        assert!(health.overall_healthy);
    }

    #[test]
    fn test_single_agent_builder() {
        let builder = SingleAgentBuilder::default()
            .with_id("test-agent")
            .with_capability("trading")
            .with_capability("research")
            .with_metadata("version", serde_json::json!("1.0"));

        assert_eq!(builder.agent_id(), Some("test-agent"));
        assert_eq!(builder.capabilities(), ["trading", "research"]);
        assert_eq!(
            builder.metadata().get("version"),
            Some(&serde_json::json!("1.0"))
        );

        assert!(builder.validate().is_ok());
    }

    #[test]
    fn test_single_agent_builder_validation() {
        // Missing ID should fail validation
        let builder = SingleAgentBuilder::default().with_capability("trading");
        assert!(builder.validate().is_err());

        // Missing capabilities should fail validation
        let builder = SingleAgentBuilder::default().with_id("test-agent");
        assert!(builder.validate().is_err());

        // Valid configuration should pass
        let builder = SingleAgentBuilder::default()
            .with_id("test-agent")
            .with_capability("trading");
        assert!(builder.validate().is_ok());
    }

    #[tokio::test]
    async fn test_agent_system_stats() {
        let system = AgentBuilder::default().build().await.unwrap();
        let stats = system.stats().await.unwrap();

        assert_eq!(stats.registry_stats.total_agents, 0);
        assert_eq!(stats.dispatcher_stats.registered_agents, 0);
        assert_eq!(stats.communication_stats.active_subscriptions, 0);
    }
}
