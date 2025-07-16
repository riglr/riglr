//! # riglr-agents
//!
//! Multi-agent coordination system for riglr blockchain automation.
//!
//! This crate provides a framework for building sophisticated multi-agent systems
//! that can coordinate blockchain operations while preserving riglr's security
//! guarantees through the SignerContext pattern.
//!
//! ## Core Concepts
//!
//! ### Agents
//!
//! Agents are autonomous units that can execute tasks and communicate with other
//! agents. They implement the [`Agent`] trait and can be specialized for different
//! types of operations:
//!
//! ```rust
//! use riglr_agents::{Agent, Task, TaskResult, AgentId};
//! use async_trait::async_trait;
//! use std::sync::Arc;
//!
//! struct TradingAgent {
//!     id: AgentId,
//! }
//!
//! #[async_trait]
//! impl Agent for TradingAgent {
//!     async fn execute_task(&self, task: Task) -> riglr_agents::Result<TaskResult> {
//!         // Execute trading logic using SignerContext::current()
//!         todo!("Implement trading logic")
//!     }
//!
//!     fn id(&self) -> &AgentId {
//!         &self.id
//!     }
//!
//!     fn capabilities(&self) -> Vec<String> {
//!         vec!["trading".to_string(), "risk_management".to_string()]
//!     }
//! }
//! ```
//!
//! ### Agent Registry
//!
//! The [`AgentRegistry`] manages agent discovery and lifecycle:
//!
//! ```rust
//! use riglr_agents::{LocalAgentRegistry, AgentRegistry};
//! use std::sync::Arc;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
//! let registry = LocalAgentRegistry::new();
//! 
//! // Register agents
//! // let agent = Arc::new(TradingAgent { id: AgentId::new("trader-1") });
//! // registry.register_agent(agent).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ### Task Dispatching
//!
//! The [`AgentDispatcher`] routes tasks to appropriate agents based on capabilities
//! and routing rules:
//!
//! ```rust
//! use riglr_agents::{AgentDispatcher, Task, TaskType, Priority};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
//! // let dispatcher = AgentDispatcher::new(registry);
//! 
//! let task = Task::new(
//!     TaskType::Trading,
//!     serde_json::json!({"symbol": "BTC/USD", "action": "buy"})
//! ).with_priority(Priority::High);
//!
//! // let result = dispatcher.dispatch_task(task).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ### Inter-Agent Communication
//!
//! Agents can communicate through the messaging system:
//!
//! ```rust
//! use riglr_agents::{AgentMessage, AgentId};
//!
//! let message = AgentMessage::new(
//!     AgentId::new("sender"),
//!     Some(AgentId::new("receiver")),
//!     "market_update".to_string(),
//!     serde_json::json!({"price": "50000", "trend": "up"})
//! );
//! ```
//!
//! ## Security and Isolation
//!
//! The riglr-agents system preserves riglr's security model:
//!
//! - **SignerContext Isolation**: Each agent operation maintains its own signer context
//! - **No Cross-Tenant Access**: Agents cannot access signers from other contexts
//! - **Secure Task Execution**: All blockchain operations use the established SignerContext pattern
//!
//! ## Features
//!
//! - **Multi-Agent Coordination**: Orchestrate complex workflows across multiple specialized agents
//! - **Flexible Routing**: Route tasks based on capabilities, load, priority, and custom rules
//! - **Inter-Agent Communication**: Message passing system for agent coordination
//! - **Scalable Architecture**: Support for both local and distributed agent registries
//! - **Integration Ready**: Seamless integration with existing riglr tools and patterns

use async_trait::async_trait;
use std::sync::Arc;

// Public exports
pub mod types;
pub mod registry;
pub mod dispatcher;
pub mod communication;
pub mod error;
pub mod builder;
pub mod integration;

// Re-export commonly used types
pub use types::{
    AgentId, Task, TaskResult, TaskType, Priority, AgentMessage, RoutingRule,
    Capability, AgentStatus, AgentState,
};
pub use registry::{AgentRegistry, LocalAgentRegistry};
pub use dispatcher::{AgentDispatcher, DispatchConfig, RoutingStrategy};
pub use communication::{AgentCommunication, ChannelCommunication};
pub use error::{AgentError, Result};
pub use builder::AgentBuilder;
pub use integration::SignerContextIntegration;

/// Core trait that all agents must implement.
///
/// Agents are autonomous units that can execute tasks and participate in
/// multi-agent workflows. They maintain their own state and capabilities
/// while respecting the SignerContext security model.
#[async_trait]
pub trait Agent: Send + Sync {
    /// Execute a task assigned to this agent.
    ///
    /// This method should use `SignerContext::current()` to access blockchain
    /// signers and perform operations. The agent should handle errors gracefully
    /// and return appropriate `TaskResult` variants.
    ///
    /// # Arguments
    ///
    /// * `task` - The task to execute
    ///
    /// # Returns
    ///
    /// A `TaskResult` indicating success, failure, or other outcomes.
    async fn execute_task(&self, task: Task) -> Result<TaskResult>;

    /// Get the unique identifier for this agent.
    fn id(&self) -> &AgentId;

    /// Get the list of capabilities this agent supports.
    ///
    /// Capabilities are used by the routing system to determine which
    /// agents can handle specific tasks.
    fn capabilities(&self) -> Vec<String>;

    /// Get the current status of this agent.
    ///
    /// The default implementation returns an idle status with basic information.
    fn status(&self) -> AgentStatus {
        AgentStatus {
            agent_id: self.id().clone(),
            status: AgentState::Idle,
            active_tasks: 0,
            load: 0.0,
            last_heartbeat: chrono::Utc::now(),
            capabilities: self.capabilities().into_iter()
                .map(|cap| Capability::new(cap, "1.0"))
                .collect(),
            metadata: std::collections::HashMap::new(),
        }
    }

    /// Optional hook called before task execution.
    ///
    /// This can be used for setup, validation, or logging.
    async fn before_task(&self, _task: &Task) -> Result<()> {
        Ok(())
    }

    /// Optional hook called after task execution.
    ///
    /// This can be used for cleanup, logging, or metrics collection.
    async fn after_task(&self, _task: &Task, _result: &TaskResult) -> Result<()> {
        Ok(())
    }

    /// Handle incoming messages from other agents.
    ///
    /// The default implementation ignores all messages. Agents that need
    /// to participate in inter-agent communication should override this method.
    async fn handle_message(&self, _message: AgentMessage) -> Result<()> {
        Ok(())
    }

    /// Check if this agent can handle the given task.
    ///
    /// The default implementation checks if any of the agent's capabilities
    /// match the task type. Agents can override this for custom logic.
    fn can_handle(&self, task: &Task) -> bool {
        let capabilities = self.capabilities();
        match &task.task_type {
            TaskType::Trading => capabilities.contains(&"trading".to_string()),
            TaskType::Research => capabilities.contains(&"research".to_string()),
            TaskType::RiskAnalysis => capabilities.contains(&"risk_analysis".to_string()),
            TaskType::Portfolio => capabilities.contains(&"portfolio".to_string()),
            TaskType::Monitoring => capabilities.contains(&"monitoring".to_string()),
            TaskType::Custom(name) => capabilities.contains(name),
        }
    }

    /// Get the agent's current load factor (0.0 to 1.0).
    ///
    /// This is used by load-balancing routing strategies.
    fn load(&self) -> f64 {
        0.0
    }

    /// Check if the agent is available to accept new tasks.
    fn is_available(&self) -> bool {
        match self.status().status {
            AgentState::Active | AgentState::Idle | AgentState::Busy => true,
            AgentState::Full | AgentState::Offline | AgentState::Maintenance => false,
        }
    }
}

/// Trait for objects that can be converted into an Agent.
///
/// This allows for flexible agent creation and registration patterns.
pub trait IntoAgent {
    /// Convert this object into an Agent.
    fn into_agent(self) -> Arc<dyn Agent>;
}

impl<T> IntoAgent for T
where
    T: Agent + 'static,
{
    fn into_agent(self) -> Arc<dyn Agent> {
        Arc::new(self)
    }
}

impl IntoAgent for Arc<dyn Agent> {
    fn into_agent(self) -> Arc<dyn Agent> {
        self
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
        should_fail: bool,
    }

    #[async_trait]
    impl Agent for MockAgent {
        async fn execute_task(&self, task: Task) -> Result<TaskResult> {
            if self.should_fail {
                return Ok(TaskResult::failure(
                    "Mock agent failure".to_string(),
                    true,
                    std::time::Duration::from_millis(10),
                ));
            }

            Ok(TaskResult::success(
                serde_json::json!({
                    "agent_id": self.id.as_str(),
                    "task_id": task.id,
                    "task_type": task.task_type.to_string()
                }),
                None,
                std::time::Duration::from_millis(100),
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
    async fn test_agent_basic_functionality() {
        let agent = MockAgent {
            id: AgentId::new("test-agent"),
            capabilities: vec!["trading".to_string()],
            should_fail: false,
        };

        let task = Task::new(
            TaskType::Trading,
            serde_json::json!({"symbol": "BTC/USD"}),
        );

        assert!(agent.can_handle(&task));
        assert!(agent.is_available());

        let result = agent.execute_task(task).await.unwrap();
        assert!(result.is_success());
    }

    #[tokio::test]
    async fn test_agent_capability_matching() {
        let trading_agent = MockAgent {
            id: AgentId::new("trading-agent"),
            capabilities: vec!["trading".to_string(), "risk_analysis".to_string()],
            should_fail: false,
        };

        let research_agent = MockAgent {
            id: AgentId::new("research-agent"),
            capabilities: vec!["research".to_string(), "monitoring".to_string()],
            should_fail: false,
        };

        let trading_task = Task::new(TaskType::Trading, serde_json::json!({}));
        let research_task = Task::new(TaskType::Research, serde_json::json!({}));

        assert!(trading_agent.can_handle(&trading_task));
        assert!(!trading_agent.can_handle(&research_task));

        assert!(!research_agent.can_handle(&trading_task));
        assert!(research_agent.can_handle(&research_task));
    }

    #[tokio::test]
    async fn test_agent_error_handling() {
        let agent = MockAgent {
            id: AgentId::new("failing-agent"),
            capabilities: vec!["trading".to_string()],
            should_fail: true,
        };

        let task = Task::new(TaskType::Trading, serde_json::json!({}));
        let result = agent.execute_task(task).await.unwrap();

        assert!(!result.is_success());
        assert!(result.is_retriable());
    }

    #[test]
    fn test_into_agent_trait() {
        let agent = MockAgent {
            id: AgentId::new("test"),
            capabilities: vec![],
            should_fail: false,
        };

        let arc_agent: Arc<dyn Agent> = agent.into_agent();
        assert_eq!(arc_agent.id().as_str(), "test");

        // Test with already-Arc wrapped agent
        let already_arc = Arc::new(MockAgent {
            id: AgentId::new("test2"),
            capabilities: vec![],
            should_fail: false,
        }) as Arc<dyn Agent>;

        let arc_agent2: Arc<dyn Agent> = already_arc.into_agent();
        assert_eq!(arc_agent2.id().as_str(), "test2");
    }

    #[test]
    fn test_agent_status() {
        let agent = MockAgent {
            id: AgentId::new("status-test"),
            capabilities: vec!["trading".to_string(), "research".to_string()],
            should_fail: false,
        };

        let status = agent.status();
        assert_eq!(status.agent_id, AgentId::new("status-test"));
        assert_eq!(status.status, AgentState::Idle);
        assert_eq!(status.capabilities.len(), 2);
        assert!(status.capabilities.iter().any(|c| c.name == "trading"));
        assert!(status.capabilities.iter().any(|c| c.name == "research"));
    }
}