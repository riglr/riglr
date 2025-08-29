//! Agent proxy types for representing local and remote agents.
//!
//! This module provides the AgentProxy enum which allows the dispatcher
//! to work with both local agents (direct function calls) and remote
//! agents (message-based communication).

use crate::{Agent, AgentId, AgentStatus, CapabilityType, Task};
use std::str::FromStr;
use std::sync::Arc;

/// Represents either a local or remote agent for the dispatcher.
///
/// The dispatcher uses this enum to abstract over whether an agent
/// is running in the same process (Local) or in a different process
/// (Remote). This enables the dispatcher to route tasks appropriately.
#[derive(Debug, Clone)]
pub enum AgentProxy {
    /// A local agent that can be called directly
    Local(Arc<dyn Agent>),
    /// A remote agent represented by its status
    Remote(AgentStatus),
}

impl AgentProxy {
    /// Get the agent's ID.
    pub fn id(&self) -> &AgentId {
        match self {
            AgentProxy::Local(agent) => agent.id(),
            AgentProxy::Remote(status) => &status.agent_id,
        }
    }

    /// Get the agent's capabilities.
    pub fn capabilities(&self) -> Vec<CapabilityType> {
        match self {
            AgentProxy::Local(agent) => agent.capabilities(),
            AgentProxy::Remote(status) => {
                // Convert Capability strings to CapabilityType using FromStr
                status
                    .capabilities
                    .iter()
                    .map(|c| CapabilityType::from_str(&c.name).unwrap())
                    .collect()
            }
        }
    }

    /// Get the agent's current load (0.0 to 1.0).
    pub fn load(&self) -> f64 {
        match self {
            AgentProxy::Local(agent) => agent.load(),
            AgentProxy::Remote(status) => status.load,
        }
    }

    /// Check if the agent is available to accept tasks.
    pub fn is_available(&self) -> bool {
        match self {
            AgentProxy::Local(agent) => agent.is_available(),
            AgentProxy::Remote(status) => {
                matches!(
                    status.status,
                    crate::types::AgentState::Active | crate::types::AgentState::Idle
                )
            }
        }
    }

    /// Check if the agent can handle a specific task.
    pub fn can_handle(&self, task: &Task) -> bool {
        match self {
            AgentProxy::Local(agent) => agent.can_handle(task),
            AgentProxy::Remote(status) => {
                // For remote agents, we check if they have the required capability
                let required_capability = crate::util::task_type_to_capability(&task.task_type);
                status
                    .capabilities
                    .iter()
                    .any(|c| c.name == required_capability.to_string())
            }
        }
    }

    /// Check if this is a local agent.
    pub fn is_local(&self) -> bool {
        matches!(self, AgentProxy::Local(_))
    }

    /// Check if this is a remote agent.
    pub fn is_remote(&self) -> bool {
        matches!(self, AgentProxy::Remote(_))
    }

    /// Get the local agent if this is a local proxy.
    pub fn as_local(&self) -> Option<&Arc<dyn Agent>> {
        match self {
            AgentProxy::Local(agent) => Some(agent),
            AgentProxy::Remote(_) => None,
        }
    }

    /// Get the remote status if this is a remote proxy.
    pub fn as_remote(&self) -> Option<&AgentStatus> {
        match self {
            AgentProxy::Local(_) => None,
            AgentProxy::Remote(status) => Some(status),
        }
    }
}
