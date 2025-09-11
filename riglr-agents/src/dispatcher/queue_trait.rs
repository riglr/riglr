//! Trait abstraction for distributed task queue implementations.
//!
//! This module defines the `DistributedTaskQueue` trait which provides
//! a generic interface for distributed task execution backends.

use crate::{AgentId, Result, Task, TaskResult};
use async_trait::async_trait;
use std::time::Duration;

/// Trait for distributed task queue implementations.
///
/// This trait abstracts the underlying message queue mechanism,
/// allowing for different implementations (Redis, RabbitMQ, NATS, etc.)
/// while maintaining a consistent interface for the dispatcher.
#[async_trait]
pub trait DistributedTaskQueue: Send + Sync + std::fmt::Debug {
    /// Dispatch a task to a remote agent and wait for the response.
    ///
    /// # Arguments
    ///
    /// * `agent_id` - The ID of the target agent
    /// * `task` - The task to execute
    /// * `task_timeout` - Maximum time to wait for task completion
    ///
    /// # Returns
    ///
    /// The result of the task execution from the remote agent.
    ///
    /// # Note on Timeouts
    ///
    /// Some backend implementations may have minimum timeout resolutions that differ
    /// from the requested timeout. For example:
    /// - Redis `BRPOP` has a minimum timeout resolution of 1 second and will round up
    ///   any timeout value less than 1 second to exactly 1 second
    /// - Other backends may have different timeout behaviors
    ///
    /// Implementations should document their specific timeout behavior and may log
    /// warnings when the actual timeout differs from the requested timeout.
    async fn dispatch_remote_task(
        &self,
        agent_id: &AgentId,
        task: Task,
        task_timeout: Duration,
    ) -> Result<TaskResult>;
}
