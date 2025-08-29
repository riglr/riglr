//! Builder patterns for easy agent system construction.
//!
//! This module provides fluent APIs for building and configuring
//! multi-agent systems with minimal boilerplate code.

use crate::{
    communication::{AgentCommunication, CommunicationConfig},
    dispatcher::RoutingStrategy,
    registry::RegistryConfig,
    Agent, AgentDispatcher, AgentRegistry, ChannelCommunication, DispatchConfig,
    LocalAgentRegistry, Result,
};
use std::sync::Arc;
use std::time::Duration;

/// Builder for creating and configuring agent systems.
///
/// The AgentSystemBuilder provides a fluent API for setting up complete
/// multi-agent systems with registries, dispatchers, and communication.
///
/// # Examples
///
/// ```rust
/// use riglr_agents::{AgentSystemBuilder, RoutingStrategy};
/// use std::time::Duration;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
/// let system = AgentSystemBuilder::default()
///     .with_max_agents(50)
///     .with_routing_strategy(RoutingStrategy::LeastLoaded)
///     .with_task_timeout(Duration::from_secs(300))
///     .build()
///     .await?;
/// # Ok(())
/// # }
/// ```
#[derive(Default)]
pub struct AgentSystemBuilder {
    registry_config: RegistryConfig,
    dispatch_config: DispatchConfig,
    communication_config: CommunicationConfig,
}

impl AgentSystemBuilder {
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
#[derive(Debug, Clone, Default)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_agent_builder_default() {
        let system = AgentSystemBuilder::default().build().await.unwrap();

        assert_eq!(system.agent_count().await.unwrap(), 0);

        let health = system.health_check().await.unwrap();
        assert!(!health.overall_healthy); // No agents registered
        assert!(health.registry_healthy);
        assert!(health.communication_healthy);
    }

    #[tokio::test]
    async fn test_agent_builder_with_config() {
        let system = AgentSystemBuilder::default()
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

        #[derive(Clone, Debug)]
        struct TestAgent {
            id: crate::AgentId,
            capabilities: Vec<crate::types::CapabilityType>,
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

            fn capabilities(&self) -> Vec<crate::types::CapabilityType> {
                self.capabilities.clone()
            }
        }

        let system = AgentSystemBuilder::default().build().await.unwrap();

        let agent = Arc::new(TestAgent {
            id: crate::AgentId::new("test-agent"),
            capabilities: vec![crate::types::CapabilityType::Trading],
        });

        system.register_agent(agent).await.unwrap();
        assert_eq!(system.agent_count().await.unwrap(), 1);

        let health = system.health_check().await.unwrap();
        assert!(health.overall_healthy);
    }

    #[tokio::test]
    async fn test_agent_system_stats() {
        let system = AgentSystemBuilder::default().build().await.unwrap();
        let stats = system.stats().await.unwrap();

        assert_eq!(stats.registry_stats.total_agents, 0);
        assert_eq!(stats.dispatcher_stats.registered_agents, 0);
        assert_eq!(stats.communication_stats.active_subscriptions, 0);
    }

    // Additional tests for 100% coverage

    #[test]
    fn test_agent_builder_with_registry_timeout() {
        let builder = AgentSystemBuilder::default().with_registry_timeout(Duration::from_secs(30));
        assert_eq!(
            builder.registry_config.operation_timeout,
            Duration::from_secs(30)
        );
    }

    #[test]
    fn test_agent_builder_with_health_checks_enabled() {
        let builder = AgentSystemBuilder::default().with_health_checks(true);
        assert!(builder.registry_config.enable_health_checks);
    }

    #[test]
    fn test_agent_builder_with_health_checks_disabled() {
        let builder = AgentSystemBuilder::default().with_health_checks(false);
        assert!(!builder.registry_config.enable_health_checks);
    }

    #[test]
    fn test_agent_builder_with_maintenance_interval() {
        let builder =
            AgentSystemBuilder::default().with_maintenance_interval(Duration::from_secs(120));
        assert_eq!(
            builder.registry_config.maintenance_interval,
            Duration::from_secs(120)
        );
    }

    #[test]
    fn test_agent_builder_with_retry_delay() {
        let builder = AgentSystemBuilder::default().with_retry_delay(Duration::from_millis(500));
        assert_eq!(
            builder.dispatch_config.retry_delay,
            Duration::from_millis(500)
        );
    }

    #[test]
    fn test_agent_builder_with_max_concurrent_tasks() {
        let builder = AgentSystemBuilder::default().with_max_concurrent_tasks(10);
        assert_eq!(builder.dispatch_config.max_concurrent_tasks_per_agent, 10);
    }

    #[test]
    fn test_agent_builder_with_load_balancing_enabled() {
        let builder = AgentSystemBuilder::default().with_load_balancing(true);
        assert!(builder.dispatch_config.enable_load_balancing);
    }

    #[test]
    fn test_agent_builder_with_load_balancing_disabled() {
        let builder = AgentSystemBuilder::default().with_load_balancing(false);
        assert!(!builder.dispatch_config.enable_load_balancing);
    }

    #[test]
    fn test_agent_builder_with_message_ttl() {
        let builder = AgentSystemBuilder::default().with_message_ttl(Duration::from_secs(600));
        assert_eq!(
            builder.communication_config.message_ttl,
            Duration::from_secs(600)
        );
    }

    #[test]
    fn test_agent_builder_with_message_persistence_enabled() {
        let builder = AgentSystemBuilder::default().with_message_persistence(true);
        assert!(builder.communication_config.enable_persistence);
    }

    #[test]
    fn test_agent_builder_with_message_persistence_disabled() {
        let builder = AgentSystemBuilder::default().with_message_persistence(false);
        assert!(!builder.communication_config.enable_persistence);
    }

    #[test]
    fn test_agent_builder_with_channel_buffer_size() {
        let builder = AgentSystemBuilder::default().with_channel_buffer_size(1024);
        assert_eq!(builder.communication_config.channel_buffer_size, 1024);
    }

    #[test]
    fn test_agent_builder_with_max_subscriptions() {
        let builder = AgentSystemBuilder::default().with_max_subscriptions(200);
        assert_eq!(builder.communication_config.max_subscriptions, Some(200));
    }

    #[tokio::test]
    async fn test_agent_builder_build_with_registry() {
        use crate::registry::LocalAgentRegistry;

        let custom_registry = Arc::new(LocalAgentRegistry::default());
        let builder = AgentSystemBuilder::default();

        let system = builder
            .build_with_registry(custom_registry.clone())
            .await
            .unwrap();

        assert_eq!(system.agent_count().await.unwrap(), 0);

        let health = system.health_check().await.unwrap();
        assert!(health.registry_healthy);
        assert!(health.communication_healthy);
    }

    #[tokio::test]
    async fn test_agent_system_register_agents_multiple() {
        use crate::types::*;

        #[derive(Clone, Debug)]
        struct TestAgent {
            id: crate::AgentId,
            capabilities: Vec<crate::types::CapabilityType>,
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

            fn capabilities(&self) -> Vec<crate::types::CapabilityType> {
                self.capabilities.clone()
            }
        }

        let system = AgentSystemBuilder::default().build().await.unwrap();

        let agents = vec![
            Arc::new(TestAgent {
                id: crate::AgentId::new("test-agent-1"),
                capabilities: vec![crate::types::CapabilityType::Trading],
            }) as Arc<dyn Agent>,
            Arc::new(TestAgent {
                id: crate::AgentId::new("test-agent-2"),
                capabilities: vec![crate::types::CapabilityType::Research],
            }) as Arc<dyn Agent>,
        ];

        system.register_agents(agents).await.unwrap();
        assert_eq!(system.agent_count().await.unwrap(), 2);
    }

    #[tokio::test]
    async fn test_agent_system_register_agents_empty_vec() {
        let system = AgentSystemBuilder::default().build().await.unwrap();

        let agents: Vec<Arc<dyn Agent>> = vec![];
        system.register_agents(agents).await.unwrap();

        assert_eq!(system.agent_count().await.unwrap(), 0);
    }

    #[tokio::test]
    async fn test_custom_agent_system_register_agent() {
        use crate::registry::LocalAgentRegistry;
        use crate::types::*;

        #[derive(Clone, Debug)]
        struct TestAgent {
            id: crate::AgentId,
            capabilities: Vec<crate::types::CapabilityType>,
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

            fn capabilities(&self) -> Vec<crate::types::CapabilityType> {
                self.capabilities.clone()
            }
        }

        let custom_registry = Arc::new(LocalAgentRegistry::default());
        let system = AgentSystemBuilder::default()
            .build_with_registry(custom_registry)
            .await
            .unwrap();

        let agent = Arc::new(TestAgent {
            id: crate::AgentId::new("test-agent"),
            capabilities: vec![crate::types::CapabilityType::Trading],
        });

        system.register_agent(agent).await.unwrap();
        assert_eq!(system.agent_count().await.unwrap(), 1);
    }

    #[tokio::test]
    async fn test_custom_agent_system_register_agents() {
        use crate::registry::LocalAgentRegistry;
        use crate::types::*;

        #[derive(Clone, Debug)]
        struct TestAgent {
            id: crate::AgentId,
            capabilities: Vec<crate::types::CapabilityType>,
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

            fn capabilities(&self) -> Vec<crate::types::CapabilityType> {
                self.capabilities.clone()
            }
        }

        let custom_registry = Arc::new(LocalAgentRegistry::default());
        let system = AgentSystemBuilder::default()
            .build_with_registry(custom_registry)
            .await
            .unwrap();

        let agents = vec![
            Arc::new(TestAgent {
                id: crate::AgentId::new("test-agent-1"),
                capabilities: vec![crate::types::CapabilityType::Trading],
            }) as Arc<dyn Agent>,
            Arc::new(TestAgent {
                id: crate::AgentId::new("test-agent-2"),
                capabilities: vec![crate::types::CapabilityType::Research],
            }) as Arc<dyn Agent>,
        ];

        system.register_agents(agents).await.unwrap();
        assert_eq!(system.agent_count().await.unwrap(), 2);
    }

    #[tokio::test]
    async fn test_custom_agent_system_health_check() {
        use crate::registry::LocalAgentRegistry;

        let custom_registry = Arc::new(LocalAgentRegistry::default());
        let system = AgentSystemBuilder::default()
            .build_with_registry(custom_registry)
            .await
            .unwrap();

        let health = system.health_check().await.unwrap();
        assert!(health.registry_healthy);
        assert!(health.communication_healthy);
        // overall_healthy depends on dispatcher health which may be false without agents
    }

    #[test]
    fn test_system_health_all_components_healthy() {
        let health = SystemHealth {
            registry_healthy: true,
            dispatcher_healthy: true,
            communication_healthy: true,
            overall_healthy: true,
        };

        assert!(health.registry_healthy);
        assert!(health.dispatcher_healthy);
        assert!(health.communication_healthy);
        assert!(health.overall_healthy);
    }

    #[test]
    fn test_system_health_some_components_unhealthy() {
        let health = SystemHealth {
            registry_healthy: true,
            dispatcher_healthy: false,
            communication_healthy: true,
            overall_healthy: false,
        };

        assert!(health.registry_healthy);
        assert!(!health.dispatcher_healthy);
        assert!(health.communication_healthy);
        assert!(!health.overall_healthy);
    }

    #[test]
    fn test_agent_builder_chaining_all_methods() {
        let builder = AgentSystemBuilder::default()
            .with_max_agents(100)
            .with_registry_timeout(Duration::from_secs(45))
            .with_health_checks(true)
            .with_maintenance_interval(Duration::from_secs(300))
            .with_task_timeout(Duration::from_secs(120))
            .with_max_retries(3)
            .with_retry_delay(Duration::from_millis(1000))
            .with_max_concurrent_tasks(5)
            .with_load_balancing(true)
            .with_routing_strategy(RoutingStrategy::RoundRobin)
            .with_max_pending_messages(1000)
            .with_message_ttl(Duration::from_secs(3600))
            .with_message_persistence(true)
            .with_channel_buffer_size(512)
            .with_max_subscriptions(150);

        // Verify all configurations are applied
        assert_eq!(builder.registry_config.max_agents, Some(100));
        assert_eq!(
            builder.registry_config.operation_timeout,
            Duration::from_secs(45)
        );
        assert!(builder.registry_config.enable_health_checks);
        assert_eq!(
            builder.registry_config.maintenance_interval,
            Duration::from_secs(300)
        );
        assert_eq!(
            builder.dispatch_config.default_task_timeout,
            Duration::from_secs(120)
        );
        assert_eq!(builder.dispatch_config.max_retries, 3);
        assert_eq!(
            builder.dispatch_config.retry_delay,
            Duration::from_millis(1000)
        );
        assert_eq!(builder.dispatch_config.max_concurrent_tasks_per_agent, 5);
        assert!(builder.dispatch_config.enable_load_balancing);
        assert_eq!(
            builder.dispatch_config.routing_strategy,
            RoutingStrategy::RoundRobin
        );
        assert_eq!(builder.communication_config.max_pending_messages, 1000);
        assert_eq!(
            builder.communication_config.message_ttl,
            Duration::from_secs(3600)
        );
        assert!(builder.communication_config.enable_persistence);
        assert_eq!(builder.communication_config.channel_buffer_size, 512);
        assert_eq!(builder.communication_config.max_subscriptions, Some(150));
    }
}
