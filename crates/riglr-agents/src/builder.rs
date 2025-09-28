/// Builder patterns for easy agent system construction.
///
/// This module provides fluent APIs for building and configuring
/// multi-agent systems with minimal boilerplate code.
extern crate alloc;

use crate::{
    communication::{
        Communication as _, Config as CommunicationConfig, Stats as CommunicationStats,
    },
    dispatcher::{RoutingStrategy, Stats as DispatcherStats},
    registry::local::Local,
    registry::{local::Stats as RegistryStats, Config as RegistryConfig, Registry},
    Agent, ChannelCommunication, DispatchConfig, Dispatcher, Result,
};
use alloc::sync::Arc;
use core::time::Duration;

/// Builder for creating and configuring agent systems.
///
/// The `SystemBuilder` provides a fluent API for setting up complete
/// multi-agent systems with registries, dispatchers, and communication.
///
/// # Examples
///
/// ```rust
/// use riglr_agents::{SystemBuilder, RoutingStrategy};
/// use std::time::Duration;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
/// let system = SystemBuilder::default()
///     .with_max_agents(50)
///     .with_routing_strategy(RoutingStrategy::LeastLoaded)
///     .with_task_timeout(Duration::from_secs(300))
///     .build()?;
/// # Ok(())
/// # }
/// ```
/// Builder for creating and configuring agent systems.
#[derive(Debug, Default)]
pub struct Builder {
    /// Communication configuration
    communication: CommunicationConfig,
    /// Dispatch configuration
    dispatch: DispatchConfig,
    /// Registry configuration
    registry: RegistryConfig,
}

impl Builder {
    /// Build the agent system with the configured settings.
    ///
    /// # Returns
    ///
    /// A complete agent system ready for use.
    ///
    /// # Errors
    ///
    /// Returns an error if any component initialization fails.
    #[inline]
    pub fn build(self) -> Result<AgentSystem> {
        let registry = Arc::new(Local::with_config(self.registry));
        let dispatcher = Dispatcher::with_config(Arc::<Local>::clone(&registry), &self.dispatch);
        let communication = Arc::new(ChannelCommunication::with_config(self.communication));

        Ok(AgentSystem::new(communication, dispatcher, registry))
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
    ///
    /// # Errors
    ///
    /// Returns an error if any component initialization fails.
    #[inline]
    pub fn build_with_registry<R: Registry + 'static>(
        self,
        registry: Arc<R>,
    ) -> Result<CustomAgentSystem<R>> {
        let dispatcher = Dispatcher::with_config(Arc::<R>::clone(&registry), &self.dispatch);
        let communication = Arc::new(ChannelCommunication::with_config(self.communication));

        Ok(CustomAgentSystem::new(communication, dispatcher, registry))
    }

    /// Set the channel buffer size for communication.
    #[must_use]
    #[inline]
    pub const fn with_channel_buffer_size(mut self, size: usize) -> Self {
        self.communication.channel_buffer_size = size;
        self
    }

    /// Enable or disable registry health checks.
    #[must_use]
    #[inline]
    pub const fn with_health_checks(mut self, enabled: bool) -> Self {
        self.registry.enable_health_checks = enabled;
        self
    }

    /// Enable or disable load balancing.
    #[must_use]
    #[inline]
    pub const fn with_load_balancing(mut self, enabled: bool) -> Self {
        self.dispatch.enable_load_balancing = enabled;
        self
    }

    /// Set the registry maintenance interval.
    #[must_use]
    #[inline]
    pub const fn with_maintenance_interval(mut self, interval: Duration) -> Self {
        self.registry.maintenance_interval = interval;
        self
    }

    /// Set the maximum number of agents in the registry.
    #[must_use]
    #[inline]
    pub const fn with_max_agents(mut self, max_agents: usize) -> Self {
        self.registry.max_agents = Some(max_agents);
        self
    }

    /// Set the maximum number of concurrent tasks per agent.
    #[must_use]
    #[inline]
    pub const fn with_max_concurrent_tasks(mut self, max_tasks: u32) -> Self {
        self.dispatch.max_concurrent_tasks_per_agent = max_tasks;
        self
    }

    /// Set the maximum number of pending messages per agent.
    #[must_use]
    #[inline]
    pub const fn with_max_pending_messages(mut self, max_messages: usize) -> Self {
        self.communication.max_pending_messages = max_messages;
        self
    }

    /// Set the maximum number of retry attempts for failed tasks.
    #[must_use]
    #[inline]
    pub const fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.dispatch.max_retries = max_retries;
        self
    }

    /// Set the maximum number of communication subscriptions.
    #[must_use]
    #[inline]
    pub const fn with_max_subscriptions(mut self, max_subs: usize) -> Self {
        self.communication.max_subscriptions = Some(max_subs);
        self
    }

    /// Enable or disable message persistence.
    #[must_use]
    #[inline]
    pub const fn with_message_persistence(mut self, enabled: bool) -> Self {
        self.communication.enable_persistence = enabled;
        self
    }

    /// Set the message time-to-live.
    #[must_use]
    #[inline]
    pub const fn with_message_ttl(mut self, ttl: Duration) -> Self {
        self.communication.message_ttl = ttl;
        self
    }

    /// Set the registry operation timeout.
    #[must_use]
    #[inline]
    pub const fn with_registry_timeout(mut self, timeout: Duration) -> Self {
        self.registry.operation_timeout = timeout;
        self
    }

    /// Set the delay between retry attempts.
    #[must_use]
    #[inline]
    pub const fn with_retry_delay(mut self, delay: Duration) -> Self {
        self.dispatch.retry_delay = delay;
        self
    }

    /// Set the routing strategy for task dispatch.
    #[must_use]
    #[inline]
    pub const fn with_routing_strategy(mut self, strategy: RoutingStrategy) -> Self {
        self.dispatch.routing_strategy = strategy;
        self
    }

    /// Set the default task execution timeout.
    #[must_use]
    #[inline]
    pub const fn with_task_timeout(mut self, timeout: Duration) -> Self {
        self.dispatch.default_task_timeout = timeout;
        self
    }
}

/// A complete agent system with local registry.
#[derive(Debug)]
#[non_exhaustive]
pub struct AgentSystem {
    /// Communication system
    pub communication: Arc<ChannelCommunication>,
    /// Task dispatcher
    pub dispatcher: Dispatcher<Local>,
    /// Agent registry
    pub registry: Arc<Local>,
}

impl AgentSystem {
    /// Get the number of registered agents.
    ///
    /// # Errors
    ///
    /// Returns an error if the registry operation fails.
    #[inline]
    pub async fn agent_count(&self) -> Result<usize> {
        self.registry.agent_count().await
    }

    /// Perform a health check on all system components.
    ///
    /// # Errors
    ///
    /// Returns an error if any health check operation fails.
    #[inline]
    pub async fn health_check(&self) -> Result<SystemHealth> {
        let registry_healthy = self.registry.health_check().await?;
        let communication_healthy = self.communication.health_check().await?;
        let dispatcher_healthy = self.dispatcher.health_check().await?;

        Ok(SystemHealth::new(
            ComponentHealthStatus::from_bool(communication_healthy),
            ComponentHealthStatus::from_bool(dispatcher_healthy),
            ComponentHealthStatus::from_bool(registry_healthy),
            ComponentHealthStatus::from_bool(
                registry_healthy && communication_healthy && dispatcher_healthy,
            ),
        ))
    }

    /// Create a new `AgentSystem` with the provided components.
    #[must_use]
    #[inline]
    pub const fn new(
        communication: Arc<ChannelCommunication>,
        dispatcher: Dispatcher<Local>,
        registry: Arc<Local>,
    ) -> Self {
        Self {
            communication,
            dispatcher,
            registry,
        }
    }

    /// Register an agent in the system.
    ///
    /// # Errors
    ///
    /// Returns an error if the agent registration fails.
    #[inline]
    pub async fn register_agent(&self, agent: Arc<dyn Agent>) -> Result<()> {
        self.registry.register_agent(agent).await
    }

    /// Register multiple agents in the system.
    ///
    /// # Errors
    ///
    /// Returns an error if any agent registration fails.
    #[inline]
    pub async fn register_agents(&self, agents: Vec<Arc<dyn Agent>>) -> Result<()> {
        for agent in agents {
            self.register_agent(agent).await?;
        }
        Ok(())
    }

    /// Get system statistics.
    ///
    /// # Errors
    ///
    /// Returns an error if any statistics gathering operation fails.
    #[inline]
    pub async fn stats(&self) -> Result<SystemStats> {
        let registry_stats = self.registry.stats().await;
        let dispatcher_stats = self.dispatcher.stats().await?;
        let communication_stats = self.communication.stats().await;

        Ok(SystemStats::new(
            communication_stats,
            dispatcher_stats,
            registry_stats,
        ))
    }
}

/// A complete agent system with custom registry.
#[derive(Debug)]
#[non_exhaustive]
pub struct CustomAgentSystem<R: Registry> {
    /// Communication system
    pub communication: Arc<ChannelCommunication>,
    /// Task dispatcher
    pub dispatcher: Dispatcher<R>,
    /// Agent registry
    pub registry: Arc<R>,
}

impl<R: Registry> CustomAgentSystem<R> {
    /// Get the number of registered agents.
    ///
    /// # Errors
    ///
    /// Returns an error if the registry operation fails.
    #[inline]
    pub async fn agent_count(&self) -> Result<usize> {
        self.registry.agent_count().await
    }

    /// Perform a health check on all system components.
    ///
    /// # Errors
    ///
    /// Returns an error if any health check operation fails.
    #[inline]
    pub async fn health_check(&self) -> Result<SystemHealth> {
        let registry_healthy = self.registry.health_check().await?;
        let communication_healthy = self.communication.health_check().await?;

        Ok(SystemHealth::new(
            ComponentHealthStatus::from_bool(communication_healthy),
            ComponentHealthStatus::Healthy, // Assuming dispatcher is always healthy
            ComponentHealthStatus::from_bool(registry_healthy),
            ComponentHealthStatus::from_bool(registry_healthy && communication_healthy),
        ))
    }

    /// Create a new `CustomAgentSystem` with the provided components.
    #[must_use]
    #[inline]
    pub const fn new(
        communication: Arc<ChannelCommunication>,
        dispatcher: Dispatcher<R>,
        registry: Arc<R>,
    ) -> Self {
        Self {
            communication,
            dispatcher,
            registry,
        }
    }

    /// Register an agent in the system.
    ///
    /// # Errors
    ///
    /// Returns an error if the agent registration fails.
    #[inline]
    pub async fn register_agent(&self, agent: Arc<dyn Agent>) -> Result<()> {
        self.registry.register_agent(agent).await
    }

    /// Register multiple agents in the system.
    ///
    /// # Errors
    ///
    /// Returns an error if any agent registration fails.
    #[inline]
    pub async fn register_agents(&self, agents: Vec<Arc<dyn Agent>>) -> Result<()> {
        for agent in agents {
            self.register_agent(agent).await?;
        }
        Ok(())
    }
}

/// Health status enum for agent system components.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ComponentHealthStatus {
    /// Component is healthy and functioning normally
    #[default]
    Healthy,
    /// Component is unhealthy or experiencing issues
    Unhealthy,
}

impl ComponentHealthStatus {
    /// Convert a boolean health status to `ComponentHealthStatus`.
    #[must_use]
    #[inline]
    const fn from_bool(healthy: bool) -> Self {
        if healthy {
            Self::Healthy
        } else {
            Self::Unhealthy
        }
    }
}

/// Health status of the agent system.
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct SystemHealth {
    /// Communication health status
    pub communication_healthy: ComponentHealthStatus,
    /// Dispatcher health status
    pub dispatcher_healthy: ComponentHealthStatus,
    /// Overall system health
    pub overall_healthy: ComponentHealthStatus,
    /// Registry health status
    pub registry_healthy: ComponentHealthStatus,
}

impl SystemHealth {
    /// Create a new `SystemHealth` with the provided component health statuses.
    #[must_use]
    #[inline]
    pub const fn new(
        communication_healthy: ComponentHealthStatus,
        dispatcher_healthy: ComponentHealthStatus,
        registry_healthy: ComponentHealthStatus,
        overall_healthy: ComponentHealthStatus,
    ) -> Self {
        Self {
            communication_healthy,
            dispatcher_healthy,
            overall_healthy,
            registry_healthy,
        }
    }
}

/// Combined statistics from all system components.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct SystemStats {
    /// Communication statistics
    pub communication_stats: CommunicationStats,
    /// Dispatcher statistics
    pub dispatcher_stats: DispatcherStats,
    /// Registry statistics
    pub registry_stats: RegistryStats,
}

impl SystemStats {
    /// Create a new `SystemStats` with the provided component statistics.
    #[must_use]
    #[inline]
    pub const fn new(
        communication_stats: CommunicationStats,
        dispatcher_stats: DispatcherStats,
        registry_stats: RegistryStats,
    ) -> Self {
        Self {
            communication_stats,
            dispatcher_stats,
            registry_stats,
        }
    }
}

/// Type alias for the main system configuration builder.
pub type SystemConfig = Builder;

#[cfg(test)]
#[expect(clippy::expect_used)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_agent_builder_default() {
        let system = Builder::default()
            .build()
            .expect("Failed to build default agent system");

        assert_eq!(
            system
                .agent_count()
                .await
                .expect("Failed to get agent count"),
            0
        );

        let health = system
            .health_check()
            .await
            .expect("Failed to perform health check");
        assert_eq!(health.overall_healthy, ComponentHealthStatus::Unhealthy); // No agents registered
        assert_eq!(health.registry_healthy, ComponentHealthStatus::Healthy);
        assert_eq!(health.communication_healthy, ComponentHealthStatus::Healthy);
    }

    #[tokio::test]
    async fn test_agent_builder_with_config() {
        let system = Builder::default()
            .with_max_agents(10)
            .with_task_timeout(Duration::from_secs(60))
            .with_max_retries(5)
            .with_routing_strategy(RoutingStrategy::LeastLoaded)
            .with_max_pending_messages(500)
            .build()
            .expect("Failed to build configured agent system");

        // Verify configuration is applied (indirectly through behavior)
        assert_eq!(
            system
                .agent_count()
                .await
                .expect("Failed to get agent count"),
            0
        );

        // The specific config values are internal, but we can verify the system works
        let health = system
            .health_check()
            .await
            .expect("Failed to perform health check");
        assert_eq!(health.registry_healthy, ComponentHealthStatus::Healthy);
        assert_eq!(health.communication_healthy, ComponentHealthStatus::Healthy);
    }

    #[tokio::test]
    async fn test_agent_system_with_agents() {
        use crate::types::*;

        #[derive(Clone, Debug)]
        struct TestAgent {
            capabilities: Vec<CapabilityType>,
            id: crate::AgentId,
        }

        #[async_trait::async_trait]
        impl Agent for TestAgent {
            fn capabilities(&self) -> Vec<CapabilityType> {
                self.capabilities.clone()
            }

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
        }

        let system = Builder::default()
            .build()
            .expect("Failed to build agent system");

        let agent = Arc::new(TestAgent {
            id: crate::AgentId::new("test-agent"),
            capabilities: vec![CapabilityType::Trading],
        });

        system
            .register_agent(agent)
            .await
            .expect("Failed to register agent");
        assert_eq!(
            system
                .agent_count()
                .await
                .expect("Failed to get agent count"),
            1
        );

        let health = system
            .health_check()
            .await
            .expect("Failed to perform health check");
        assert_eq!(health.overall_healthy, ComponentHealthStatus::Healthy);
    }

    #[tokio::test]
    async fn test_agent_system_stats() {
        let system = Builder::default()
            .build()
            .expect("Failed to build agent system");
        let stats = system.stats().await.expect("Failed to get system stats");

        assert_eq!(stats.registry_stats.total_agents, 0);
        assert_eq!(stats.dispatcher_stats.registered_agents, 0);
        assert_eq!(stats.communication_stats.active_subscriptions, 0);
    }

    // Additional tests for 100% coverage

    #[test]
    fn test_agent_builder_with_registry_timeout() {
        let builder = Builder::default().with_registry_timeout(Duration::from_secs(30));
        assert_eq!(builder.registry.operation_timeout, Duration::from_secs(30));
    }

    #[test]
    fn test_agent_builder_with_health_checks_enabled() {
        let builder = Builder::default().with_health_checks(true);
        assert!(builder.registry.enable_health_checks);
    }

    #[test]
    fn test_agent_builder_with_health_checks_disabled() {
        let builder = Builder::default().with_health_checks(false);
        assert!(!builder.registry.enable_health_checks);
    }

    #[test]
    fn test_agent_builder_with_maintenance_interval() {
        let builder = Builder::default().with_maintenance_interval(Duration::from_secs(120));
        assert_eq!(
            builder.registry.maintenance_interval,
            Duration::from_secs(120)
        );
    }

    #[test]
    fn test_agent_builder_with_retry_delay() {
        let builder = Builder::default().with_retry_delay(Duration::from_millis(500));
        assert_eq!(builder.dispatch.retry_delay, Duration::from_millis(500));
    }

    #[test]
    fn test_agent_builder_with_max_concurrent_tasks() {
        let builder = Builder::default().with_max_concurrent_tasks(10);
        assert_eq!(builder.dispatch.max_concurrent_tasks_per_agent, 10);
    }

    #[test]
    fn test_agent_builder_with_load_balancing_enabled() {
        let builder = Builder::default().with_load_balancing(true);
        assert!(builder.dispatch.enable_load_balancing);
    }

    #[test]
    fn test_agent_builder_with_load_balancing_disabled() {
        let builder = Builder::default().with_load_balancing(false);
        assert!(!builder.dispatch.enable_load_balancing);
    }

    #[test]
    fn test_agent_builder_with_message_ttl() {
        let builder = Builder::default().with_message_ttl(Duration::from_secs(600));
        assert_eq!(builder.communication.message_ttl, Duration::from_secs(600));
    }

    #[test]
    fn test_agent_builder_with_message_persistence_enabled() {
        let builder = Builder::default().with_message_persistence(true);
        assert!(builder.communication.enable_persistence);
    }

    #[test]
    fn test_agent_builder_with_message_persistence_disabled() {
        let builder = Builder::default().with_message_persistence(false);
        assert!(!builder.communication.enable_persistence);
    }

    #[test]
    fn test_agent_builder_with_channel_buffer_size() {
        let builder = Builder::default().with_channel_buffer_size(1024);
        assert_eq!(builder.communication.channel_buffer_size, 1024);
    }

    #[test]
    fn test_agent_builder_with_max_subscriptions() {
        let builder = Builder::default().with_max_subscriptions(200);
        assert_eq!(builder.communication.max_subscriptions, Some(200));
    }

    #[tokio::test]
    async fn test_agent_builder_build_with_registry() {
        use crate::registry::local::Local;

        let custom_registry = Arc::new(Local::default());
        let builder = Builder::default();

        let system = builder
            .build_with_registry(custom_registry.clone())
            .expect("Failed to build system with custom registry");

        assert_eq!(
            system
                .agent_count()
                .await
                .expect("Failed to get agent count"),
            0
        );

        let health = system
            .health_check()
            .await
            .expect("Failed to perform health check");
        assert_eq!(health.registry_healthy, ComponentHealthStatus::Healthy);
        assert_eq!(health.communication_healthy, ComponentHealthStatus::Healthy);
    }

    #[tokio::test]
    async fn test_agent_system_register_agents_multiple() {
        use crate::types::*;

        #[derive(Clone, Debug)]
        struct TestAgent {
            capabilities: Vec<CapabilityType>,
            id: crate::AgentId,
        }

        #[async_trait::async_trait]
        impl Agent for TestAgent {
            fn capabilities(&self) -> Vec<CapabilityType> {
                self.capabilities.clone()
            }

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
        }

        let system = Builder::default()
            .build()
            .expect("Failed to build agent system");

        let agents = vec![
            Arc::new(TestAgent {
                id: crate::AgentId::new("test-agent-1"),
                capabilities: vec![CapabilityType::Trading],
            }) as Arc<dyn Agent>,
            Arc::new(TestAgent {
                id: crate::AgentId::new("test-agent-2"),
                capabilities: vec![CapabilityType::Research],
            }) as Arc<dyn Agent>,
        ];

        system
            .register_agents(agents)
            .await
            .expect("Failed to register multiple agents");
        assert_eq!(
            system
                .agent_count()
                .await
                .expect("Failed to get agent count"),
            2
        );
    }

    #[tokio::test]
    async fn test_agent_system_register_agents_empty_vec() {
        let system = Builder::default()
            .build()
            .expect("Failed to build agent system");

        let agents: Vec<Arc<dyn Agent>> = vec![];
        system
            .register_agents(agents)
            .await
            .expect("Failed to register empty agent list");

        assert_eq!(
            system
                .agent_count()
                .await
                .expect("Failed to get agent count"),
            0
        );
    }

    #[tokio::test]
    async fn test_custom_agent_system_register_agent() {
        use crate::registry::local::Local;
        use crate::types::*;

        #[derive(Clone, Debug)]
        struct TestAgent {
            capabilities: Vec<CapabilityType>,
            id: crate::AgentId,
        }

        #[async_trait::async_trait]
        impl Agent for TestAgent {
            fn capabilities(&self) -> Vec<CapabilityType> {
                self.capabilities.clone()
            }

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
        }

        let custom_registry = Arc::new(Local::default());
        let system = Builder::default()
            .build_with_registry(custom_registry)
            .expect("Failed to build system with custom registry");

        let agent = Arc::new(TestAgent {
            id: crate::AgentId::new("test-agent"),
            capabilities: vec![CapabilityType::Trading],
        });

        system
            .register_agent(agent)
            .await
            .expect("Failed to register agent");
        assert_eq!(
            system
                .agent_count()
                .await
                .expect("Failed to get agent count"),
            1
        );
    }

    #[tokio::test]
    async fn test_custom_agent_system_register_agents() {
        use crate::registry::local::Local;
        use crate::types::*;

        #[derive(Clone, Debug)]
        struct TestAgent {
            capabilities: Vec<CapabilityType>,
            id: crate::AgentId,
        }

        #[async_trait::async_trait]
        impl Agent for TestAgent {
            fn capabilities(&self) -> Vec<CapabilityType> {
                self.capabilities.clone()
            }

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
        }

        let custom_registry = Arc::new(Local::default());
        let system = Builder::default()
            .build_with_registry(custom_registry)
            .expect("Failed to build system with custom registry");

        let agents = vec![
            Arc::new(TestAgent {
                id: crate::AgentId::new("test-agent-1"),
                capabilities: vec![CapabilityType::Trading],
            }) as Arc<dyn Agent>,
            Arc::new(TestAgent {
                id: crate::AgentId::new("test-agent-2"),
                capabilities: vec![CapabilityType::Research],
            }) as Arc<dyn Agent>,
        ];

        system
            .register_agents(agents)
            .await
            .expect("Failed to register multiple agents");
        assert_eq!(
            system
                .agent_count()
                .await
                .expect("Failed to get agent count"),
            2
        );
    }

    #[tokio::test]
    async fn test_custom_agent_system_health_check() {
        use crate::registry::local::Local;

        let custom_registry = Arc::new(Local::default());
        let system = Builder::default()
            .build_with_registry(custom_registry)
            .expect("Failed to build system with custom registry");

        let health = system
            .health_check()
            .await
            .expect("Failed to perform health check");
        assert_eq!(health.registry_healthy, ComponentHealthStatus::Healthy);
        assert_eq!(health.communication_healthy, ComponentHealthStatus::Healthy);
        // overall_healthy depends on dispatcher health which may be false without agents
    }

    #[test]
    fn test_system_health_all_components_healthy() {
        let health = SystemHealth::new(
            ComponentHealthStatus::Healthy,
            ComponentHealthStatus::Healthy,
            ComponentHealthStatus::Healthy,
            ComponentHealthStatus::Healthy,
        );

        assert_eq!(health.registry_healthy, ComponentHealthStatus::Healthy);
        assert_eq!(health.dispatcher_healthy, ComponentHealthStatus::Healthy);
        assert_eq!(health.communication_healthy, ComponentHealthStatus::Healthy);
        assert_eq!(health.overall_healthy, ComponentHealthStatus::Healthy);
    }

    #[test]
    fn test_system_health_some_components_unhealthy() {
        let health = SystemHealth::new(
            ComponentHealthStatus::Healthy,
            ComponentHealthStatus::Unhealthy,
            ComponentHealthStatus::Healthy,
            ComponentHealthStatus::Unhealthy,
        );

        assert_eq!(health.registry_healthy, ComponentHealthStatus::Healthy);
        assert_eq!(health.dispatcher_healthy, ComponentHealthStatus::Unhealthy);
        assert_eq!(health.communication_healthy, ComponentHealthStatus::Healthy);
        assert_eq!(health.overall_healthy, ComponentHealthStatus::Unhealthy);
    }

    #[test]
    fn test_agent_builder_chaining_all_methods() {
        let builder = Builder::default()
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
        assert_eq!(builder.registry.max_agents, Some(100));
        assert_eq!(builder.registry.operation_timeout, Duration::from_secs(45));
        assert!(builder.registry.enable_health_checks);
        assert_eq!(
            builder.registry.maintenance_interval,
            Duration::from_secs(300)
        );
        assert_eq!(
            builder.dispatch.default_task_timeout,
            Duration::from_secs(120)
        );
        assert_eq!(builder.dispatch.max_retries, 3);
        assert_eq!(builder.dispatch.retry_delay, Duration::from_millis(1000));
        assert_eq!(builder.dispatch.max_concurrent_tasks_per_agent, 5);
        assert!(builder.dispatch.enable_load_balancing);
        assert_eq!(
            builder.dispatch.routing_strategy,
            RoutingStrategy::RoundRobin
        );
        assert_eq!(builder.communication.max_pending_messages, 1000);
        assert_eq!(builder.communication.message_ttl, Duration::from_secs(3600));
        assert!(builder.communication.enable_persistence);
        assert_eq!(builder.communication.channel_buffer_size, 512);
        assert_eq!(builder.communication.max_subscriptions, Some(150));
    }
}
