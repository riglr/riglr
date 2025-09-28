/// Redis-based implementation of distributed task queue.
///
/// This module provides a Redis-specific implementation of the
/// `DistributedTaskQueue` trait for distributed task execution.
extern crate alloc;
use super::queue_trait::DistributedTaskQueue;
use crate::{AgentId, Result, Task, TaskResult};
use async_trait::async_trait;
use redis::aio::MultiplexedConnection;

use alloc::sync::Arc;
use core::future::Future;
use core::time::Duration;
use redis::AsyncCommands as _;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Message sent to a remote agent via queue.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct TaskMessage {
    /// Unique ID for this message
    pub message_id: String,
    /// Queue where the response should be sent
    pub response_queue: String,
    /// The task to execute
    pub task: Task,
    /// Timestamp when the message was created
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl TaskMessage {
    /// Create a new `TaskMessage`.
    #[must_use]
    #[inline]
    pub const fn new(
        message_id: String,
        response_queue: String,
        task: Task,
        timestamp: chrono::DateTime<chrono::Utc>,
    ) -> Self {
        Self {
            message_id,
            response_queue,
            task,
            timestamp,
        }
    }
}

/// Response from a remote agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct TaskResponse {
    /// ID of the agent that executed the task
    pub agent_id: AgentId,
    /// ID of the original message
    pub message_id: String,
    /// Result of task execution
    pub result: TaskResult,
    /// Timestamp when the response was created
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl TaskResponse {
    /// Create a new `TaskResponse`.
    #[must_use]
    #[inline]
    pub const fn new(
        agent_id: AgentId,
        message_id: String,
        result: TaskResult,
        timestamp: chrono::DateTime<chrono::Utc>,
    ) -> Self {
        Self {
            agent_id,
            message_id,
            result,
            timestamp,
        }
    }
}

/// Redis-based implementation of distributed task storage.
#[derive(Debug)]
pub struct Redis {
    /// Redis connection pool for task operations
    connection: Arc<Mutex<MultiplexedConnection>>,
}

impl Redis {
    /// Internal method to dispatch a task via Redis.
    #[expect(clippy::cognitive_complexity)]
    async fn dispatch_task_internal(
        &self,
        agent_id: &AgentId,
        task: Task,
        task_timeout: Duration,
    ) -> Result<TaskResult> {
        let message_id = Uuid::new_v4().to_string();
        let response_queue = format!("response:{message_id}");

        // Create the task message
        let message = TaskMessage::new(
            message_id.clone(),
            response_queue.clone(),
            task.clone(),
            chrono::Utc::now(),
        );

        // Serialize the message
        let message_json = serde_json::to_string(&message).map_err(|error| {
            crate::AgentError::task_execution(format!("Failed to serialize task: {error}"))
        })?;

        // Push to the agent's queue
        let agent_queue = format!("queue:{}", agent_id.as_str());
        debug!(
            "Enqueuing task {} to queue {} for agent {}",
            task.id, agent_queue, agent_id
        );

        self.connection
            .lock()
            .await
            .lpush::<_, _, ()>(&agent_queue, &message_json)
            .await
            .map_err(|error| {
                crate::AgentError::communication(format!("Failed to enqueue task: {error}"))
            })?;

        // Wait for response (timeout is handled by brpop in wait_for_response)
        self.wait_for_response(&response_queue, &message_id, task_timeout)
            .await;

        // Since wait_for_response no longer returns a Result, we need to check for the response manually
        let response: Option<(String, String)> = self
            .connection
            .lock()
            .await
            .brpop(&response_queue, 0.0) // 0.0 means no additional timeout
            .await
            .map_err(|error| {
                crate::AgentError::communication(format!("Failed to read response: {error}"))
            })?;

        if let Some((_queue, response_json)) = response {
            let response: TaskResponse = serde_json::from_str(&response_json).map_err(|error| {
                crate::AgentError::task_execution(format!("Failed to parse response: {error}"))
            })?;

            info!(
                "Received response for task {} from agent {}",
                task.id, response.agent_id
            );
            Ok(response.result)
        } else {
            warn!("Task {} timed out after {:?}", task.id, task_timeout);
            Ok(TaskResult::Timeout {
                duration: task_timeout,
            })
        }
    }

    /// Create a new Redis-based task queue.
    #[must_use]
    #[inline]
    pub fn new(connection: MultiplexedConnection) -> Self {
        Self {
            connection: Arc::new(Mutex::new(connection)),
        }
    }

    /// Wait for a response on the specified queue.
    #[expect(clippy::cognitive_complexity)]
    async fn wait_for_response(
        &self,
        response_queue: &str,
        message_id: &str,
        timeout_duration: Duration,
    ) {
        // Check if timeout is less than Redis's minimum and warn the user
        if timeout_duration.as_secs() < 1 && timeout_duration.as_millis() > 0 {
            warn!(
                "Requested task timeout of {:?} is less than Redis's 1-second minimum for BRPOP. The timeout will be rounded up to 1 second.",
                timeout_duration
            );
        }

        // Redis BRPOP requires a timeout of at least 1 second. If the requested
        // timeout is shorter, it will be rounded up to the minimum.
        let timeout_secs = timeout_duration.as_secs().max(1); // BRPOP requires at least 1 second timeout
        #[expect(clippy::cast_precision_loss)]
        let timeout_f64 = timeout_secs as f64;

        // Use blocking pop to wait for response
        let response: Option<(String, String)> = match self
            .connection
            .lock()
            .await
            .brpop(response_queue, timeout_f64)
            .await
        {
            Ok(result) => result,
            Err(error) => {
                error!("Failed to read response: {}", error);
                return;
            }
        };

        match response {
            Some((_queue, response_json)) => {
                // Parse the response
                let response: TaskResponse = match serde_json::from_str(&response_json) {
                    Ok(resp) => resp,
                    Err(error) => {
                        error!("Failed to parse response: {}", error);
                        return;
                    }
                };

                // Verify it's for our message
                if response.message_id == message_id {
                    debug!("Received response for message {}", message_id);
                } else {
                    warn!(
                        "Received response for wrong message: expected {}, got {}",
                        message_id, response.message_id
                    );
                }
            }
            None => {
                // If response is None, it means BRPOP timed out
                warn!("Timeout waiting for response to message {}", message_id);
            }
        }
    }
}

#[async_trait]
impl DistributedTaskQueue for Redis {
    #[inline]
    async fn dispatch_remote_task(
        &self,
        agent_id: &AgentId,
        task: Task,
        task_timeout: Duration,
    ) -> Result<TaskResult> {
        self.dispatch_task_internal(agent_id, task, task_timeout)
            .await
    }
}

/// Worker that processes tasks from a queue for a local agent.
#[derive(Debug)]
pub struct Worker {
    /// The agent ID this worker represents
    agent_id: AgentId,
    /// Redis connection for queue operations
    connection: MultiplexedConnection,
}

impl Worker {
    /// Create a new queue worker.
    #[must_use]
    #[inline]
    pub const fn new(agent_id: AgentId, connection: MultiplexedConnection) -> Self {
        Self {
            agent_id,
            connection,
        }
    }

    /// Start processing tasks from the queue.
    ///
    /// This method runs indefinitely, processing tasks as they arrive.
    ///
    /// # Errors
    ///
    /// Returns an error if Redis communication fails or task processing encounters unrecoverable errors.
    #[expect(clippy::cognitive_complexity)]
    #[inline]
    pub async fn run<F, Fut>(&mut self, executor: F) -> Result<()>
    where
        F: Fn(Task) -> Fut + Send + Sync,
        Fut: Future<Output = Result<TaskResult>> + Send,
    {
        let queue_name = format!("queue:{}", self.agent_id.as_str());
        info!(
            "Queue worker started for agent {} on queue {}",
            self.agent_id, queue_name
        );

        loop {
            // Block waiting for a task (with timeout to allow for graceful shutdown checks)
            let brpop_result = self.connection.brpop(&queue_name, 5.0).await;
            let task_json: Option<String> = brpop_result.map_err(|error| {
                crate::AgentError::communication(format!("Failed to read from queue: {error}"))
            })?;

            if let Some(json) = task_json {
                // Parse the task message
                match serde_json::from_str::<TaskMessage>(&json) {
                    Ok(message) => {
                        debug!("Processing task {} from queue", message.task.id);

                        // Execute the task
                        let result = executor(message.task.clone()).await;

                        // Create response
                        let response = TaskResponse::new(
                            self.agent_id.clone(),
                            message.message_id,
                            result.unwrap_or_else(|error| {
                                TaskResult::failure(
                                    format!("Task execution failed: {error}"),
                                    false,
                                    Duration::from_millis(0),
                                )
                            }),
                            chrono::Utc::now(),
                        );

                        // Send response
                        if let Err(error) =
                            self.send_response(&message.response_queue, response).await
                        {
                            error!("Failed to send response: {}", error);
                        }
                    }
                    Err(error) => {
                        error!("Failed to parse task message: {}", error);
                    }
                }
            }
        }
    }

    /// Send a response to the specified queue.
    async fn send_response(&mut self, response_queue: &str, response: TaskResponse) -> Result<()> {
        let response_json = serde_json::to_string(&response).map_err(|error| {
            crate::AgentError::task_execution(format!("Failed to serialize response: {error}"))
        })?;

        self.connection
            .lpush::<_, _, ()>(response_queue, response_json)
            .await
            .map_err(|error| {
                crate::AgentError::communication(format!("Failed to send response: {error}"))
            })?;

        debug!(
            "Sent response for message {} to queue {}",
            response.message_id, response_queue
        );
        Ok(())
    }
}

/// Type alias for Redis-based task storage.
pub type RedisTaskStorage = Redis;

/// Type alias for task worker.
pub type TaskWorker = Worker;

#[cfg(test)]
#[expect(clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn test_task_message_serialization() {
        let message = TaskMessage::new(
            "test-123".to_string(),
            "response:test-123".to_string(),
            Task::new(crate::TaskType::Trading, serde_json::json!({"test": true})),
            chrono::Utc::now(),
        );

        let json =
            serde_json::to_string(&message).expect("Failed to serialize TaskMessage in test");
        let deserialized: TaskMessage =
            serde_json::from_str(&json).expect("Failed to deserialize TaskMessage in test");

        assert_eq!(message.message_id, deserialized.message_id);
        assert_eq!(message.response_queue, deserialized.response_queue);
    }

    #[test]
    fn test_task_response_serialization() {
        let response = TaskResponse::new(
            AgentId::new("agent-1"),
            "test-123".to_string(),
            TaskResult::success(
                serde_json::json!({"status": "completed"}),
                Some("0x123".to_string()),
                Duration::from_secs(1),
            ),
            chrono::Utc::now(),
        );

        let json =
            serde_json::to_string(&response).expect("Failed to serialize TaskResponse in test");
        let deserialized: TaskResponse =
            serde_json::from_str(&json).expect("Failed to deserialize TaskResponse in test");

        assert_eq!(response.message_id, deserialized.message_id);
        assert_eq!(response.agent_id, deserialized.agent_id);
    }
}
