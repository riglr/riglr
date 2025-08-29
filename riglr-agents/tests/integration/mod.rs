//! Integration tests for riglr-agents coordination and messaging system.
//!
//! This module contains integration tests that validate the interaction between
//! different components of the agent coordination system including:
//! - Agent coordination and workflow orchestration
//! - Dispatcher, registry, and routing integration
//! - Error handling and recovery mechanisms
//! - Inter-agent communication and message passing

/// Agent coordination and workflow orchestration tests.
pub mod agent_coordination;
/// Dispatcher, registry, and routing integration tests.
pub mod dispatcher_integration;
/// Distributed coordination tests for cross-process agent discovery.
#[cfg(feature = "redis")]
pub mod distributed_coordination;
/// Distributed agent registry tests with Redis backend.
#[cfg(feature = "redis")]
pub mod distributed_registry;
/// Error handling and recovery mechanism tests.
pub mod error_handling;
/// Inter-agent communication and message passing tests.
pub mod message_passing;
