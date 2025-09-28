/// Agent proxy types for representing local and remote agents.
///
/// This module provides the `AgentProxy` enum which allows the dispatcher
/// to work with both local agents (direct function calls) and remote
/// agents (message-based communication).
extern crate alloc;

use crate::types::AgentState;
use crate::util::task_type_to_capability;
use crate::{Agent, AgentId, AgentStatus, CapabilityType, Task};
use alloc::sync::Arc;
use core::str::FromStr as _;

/// Represents either a local or remote agent for the dispatcher.
///
/// The dispatcher uses this enum to abstract over whether an agent
/// is running in the same process (Local) or in a different process
/// (Remote). This enables the dispatcher to route tasks appropriately.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum Proxy {
    /// A local agent that can be called directly
    Local(Arc<dyn Agent>),
    /// A remote agent represented by its status
    Remote(AgentStatus),
}

impl Proxy {
    /// Get the local agent if this is a local proxy.
    #[must_use]
    #[inline]
    pub fn as_local(&self) -> Option<&Arc<dyn Agent>> {
        match *self {
            Self::Local(ref agent) => Some(agent),
            Self::Remote(_) => None,
        }
    }

    /// Get the remote status if this is a remote proxy.
    #[must_use]
    #[inline]
    pub const fn as_remote(&self) -> Option<&AgentStatus> {
        match *self {
            Self::Local(_) => None,
            Self::Remote(ref status) => Some(status),
        }
    }

    /// Check if the agent can handle a specific task.
    #[must_use]
    #[inline]
    pub fn can_handle(&self, task: &Task) -> bool {
        match *self {
            Self::Local(ref agent) => agent.can_handle(task),
            Self::Remote(ref status) => {
                // For remote agents, we check if they have the required capability
                let required_capability = task_type_to_capability(&task.task_type);
                status
                    .capabilities
                    .iter()
                    .any(|capability| capability.name == required_capability.to_string())
            }
        }
    }

    /// Get the agent's capabilities.
    ///
    /// # Panics
    ///
    /// Panics if a remote agent's capability name cannot be parsed as a valid `CapabilityType`.
    #[must_use]
    #[inline]
    pub fn capabilities(&self) -> Vec<CapabilityType> {
        match *self {
            Self::Local(ref agent) => agent.capabilities(),
            Self::Remote(ref status) => {
                // Convert Capability strings to CapabilityType using FromStr
                status
                    .capabilities
                    .iter()
                    .filter_map(|capability| CapabilityType::from_str(&capability.name).ok())
                    .collect()
            }
        }
    }

    /// Get the agent's ID.
    #[must_use]
    #[inline]
    pub fn id(&self) -> &AgentId {
        match *self {
            Self::Local(ref agent) => agent.id(),
            Self::Remote(ref status) => &status.agent_id,
        }
    }

    /// Check if the agent is available to accept tasks.
    #[must_use]
    #[inline]
    pub fn is_available(&self) -> bool {
        match *self {
            Self::Local(ref agent) => agent.is_available(),
            Self::Remote(ref status) => {
                matches!(status.status, AgentState::Active | AgentState::Idle)
            }
        }
    }

    /// Check if this is a local agent.
    #[must_use]
    #[inline]
    pub const fn is_local(&self) -> bool {
        matches!(*self, Self::Local(_))
    }

    /// Check if this is a remote agent.
    #[must_use]
    #[inline]
    pub const fn is_remote(&self) -> bool {
        matches!(*self, Self::Remote(_))
    }

    /// Get the agent's current load (0.0 to 1.0).
    #[must_use]
    #[inline]
    pub fn load(&self) -> f64 {
        match *self {
            Self::Local(ref agent) => agent.load(),
            Self::Remote(ref status) => status.load,
        }
    }
}

/// Type alias for agent proxy wrapper.
pub type AgentWrapper = Proxy;
