//! Redis-based implementation of distributed task queue.
//!
//! This module provides a Redis-specific implementation of the
//! `DistributedTaskQueue` trait for distributed task execution.

use super::queue_trait::DistributedTaskQueue;
use crate::{AgentId, Result, Task, TaskResult};
use async_trait::async_trait;
use redis::aio::MultiplexedConnection;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Message sent to a remote agent via queue.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskMessage {
    /// Unique ID for this message
    pub message_id: String,
    /// The task to execute
    pub task: Task,
    /// Queue where the response should be sent
    pub response_queue: String,
    /// Timestamp when the message was created
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Response from a remote agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResponse {
    /// ID of the original message
    pub message_id: String,
    /// Result of task execution
    pub result: TaskResult,
    /// ID of the agent that executed the task
    pub agent_id: AgentId,
    /// Timestamp when the response was created
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Redis-based implementation of distributed task queue.
#[derive(Debug)]
pub struct RedisTaskQueue {
    connection: Arc<Mutex<MultiplexedConnection>>,
}

impl RedisTaskQueue {
    /// Create a new Redis-based task queue.
    pub fn new(connection: MultiplexedConnection) -> Self {
        Self {
            connection: Arc::new(Mutex::new(connection)),
        }
    }

    /// Internal method to dispatch a task via Redis.
    async fn dispatch_task_internal(
        &self,
        agent_id: &AgentId,
        task: Task,
        task_timeout: Duration,
    ) -> Result<TaskResult> {
        let message_id = Uuid::new_v4().to_string();
        let response_queue = format!("response:{}", message_id);

        // Create the task message
        let message = TaskMessage {
            message_id: message_id.clone(),
            task: task.clone(),
            response_queue: response_queue.clone(),
            timestamp: chrono::Utc::now(),
        };

        // Serialize the message
        let message_json = serde_json::to_string(&message).map_err(|e| {
            crate::AgentError::task_execution(format!("Failed to serialize task: {}", e))
        })?;

        // Push to the agent's queue
        let agent_queue = format!("queue:{}", agent_id.as_str());
        debug!(
            "Enqueuing task {} to queue {} for agent {}",
            task.id, agent_queue, agent_id
        );

        {
            let mut conn = self.connection.lock().await;
            conn.lpush::<_, _, ()>(&agent_queue, &message_json)
                .await
                .map_err(|e| {
                    crate::AgentError::communication(format!("Failed to enqueue task: {}", e))
                })?;
        }

        // Wait for response (timeout is handled by brpop in wait_for_response)
        match self
            .wait_for_response(&response_queue, &message_id, task_timeout)
            .await
        {
            Ok(response) => {
                info!(
                    "Received response for task {} from agent {}",
                    task.id, response.agent_id
                );
                Ok(response.result)
            }
            Err(e) => {
                // Check if it's a timeout error
                if e.to_string().contains("timeout") {
                    warn!("Task {} timed out after {:?}", task.id, task_timeout);
                    Ok(TaskResult::Timeout {
                        duration: task_timeout,
                    })
                } else {
                    error!("Failed to get response for task {}: {}", task.id, e);
                    Err(e)
                }
            }
        }
    }

    /// Wait for a response on the specified queue.
    async fn wait_for_response(
        &self,
        response_queue: &str,
        message_id: &str,
        timeout_duration: Duration,
    ) -> Result<TaskResponse> {
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

        // Use blocking pop to wait for response
        let response: Option<(String, String)> = {
            let mut conn = self.connection.lock().await;
            let brpop_result = conn.brpop(response_queue, timeout_secs as f64).await;
            brpop_result.map_err(|e| {
                crate::AgentError::communication(format!("Failed to read response: {}", e))
            })?
        };

        if let Some((_queue, response_json)) = response {
            // Parse the response
            let response: TaskResponse = serde_json::from_str(&response_json).map_err(|e| {
                crate::AgentError::task_execution(format!("Failed to parse response: {}", e))
            })?;

            // Verify it's for our message
            if response.message_id == message_id {
                return Ok(response);
            } else {
                warn!(
                    "Received response for wrong message: expected {}, got {}",
                    message_id, response.message_id
                );
                // Return an error for mismatched message ID
                return Err(crate::AgentError::task_execution(format!(
                    "Response message ID mismatch: expected {}, got {}",
                    message_id, response.message_id
                )));
            }
        }

        // If response is None, it means BRPOP timed out
        Err(crate::AgentError::task_timeout(
            format!("message_{}", message_id),
            timeout_duration,
        ))
    }
}

#[async_trait]
impl DistributedTaskQueue for RedisTaskQueue {
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
pub struct QueueWorker {
    agent_id: AgentId,
    connection: MultiplexedConnection,
}

impl QueueWorker {
    /// Create a new queue worker.
    pub fn new(agent_id: AgentId, connection: MultiplexedConnection) -> Self {
        Self {
            agent_id,
            connection,
        }
    }

    /// Start processing tasks from the queue.
    ///
    /// This method runs indefinitely, processing tasks as they arrive.
    pub async fn run<F, Fut>(&mut self, executor: F) -> Result<()>
    where
        F: Fn(Task) -> Fut + Send + Sync,
        Fut: std::future::Future<Output = Result<TaskResult>> + Send,
    {
        let queue_name = format!("queue:{}", self.agent_id.as_str());
        info!(
            "Queue worker started for agent {} on queue {}",
            self.agent_id, queue_name
        );

        loop {
            // Block waiting for a task (with timeout to allow for graceful shutdown checks)
            let brpop_result = self.connection.brpop(&queue_name, 5.0).await;
            let task_json: Option<String> = brpop_result.map_err(|e| {
                crate::AgentError::communication(format!("Failed to read from queue: {}", e))
            })?;

            if let Some(json) = task_json {
                // Parse the task message
                match serde_json::from_str::<TaskMessage>(&json) {
                    Ok(message) => {
                        debug!("Processing task {} from queue", message.task.id);

                        // Execute the task
                        let result = executor(message.task.clone()).await;

                        // Create response
                        let response = TaskResponse {
                            message_id: message.message_id,
                            result: result.unwrap_or_else(|e| {
                                TaskResult::failure(
                                    format!("Task execution failed: {}", e),
                                    false,
                                    Duration::from_millis(0),
                                )
                            }),
                            agent_id: self.agent_id.clone(),
                            timestamp: chrono::Utc::now(),
                        };

                        // Send response
                        if let Err(e) = self.send_response(&message.response_queue, response).await
                        {
                            error!("Failed to send response: {}", e);
                        }
                    }
                    Err(e) => {
                        error!("Failed to parse task message: {}", e);
                    }
                }
            }
        }
    }

    /// Send a response to the specified queue.
    async fn send_response(&mut self, response_queue: &str, response: TaskResponse) -> Result<()> {
        let response_json = serde_json::to_string(&response).map_err(|e| {
            crate::AgentError::task_execution(format!("Failed to serialize response: {}", e))
        })?;

        self.connection
            .lpush::<_, _, ()>(response_queue, response_json)
            .await
            .map_err(|e| {
                crate::AgentError::communication(format!("Failed to send response: {}", e))
            })?;

        debug!(
            "Sent response for message {} to queue {}",
            response.message_id, response_queue
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_message_serialization() {
        let message = TaskMessage {
            message_id: "test-123".to_string(),
            task: Task::new(crate::TaskType::Trading, serde_json::json!({"test": true})),
            response_queue: "response:test-123".to_string(),
            timestamp: chrono::Utc::now(),
        };

        let json = serde_json::to_string(&message).unwrap();
        let deserialized: TaskMessage = serde_json::from_str(&json).unwrap();

        assert_eq!(message.message_id, deserialized.message_id);
        assert_eq!(message.response_queue, deserialized.response_queue);
    }

    #[test]
    fn test_task_response_serialization() {
        let response = TaskResponse {
            message_id: "test-123".to_string(),
            result: TaskResult::success(
                serde_json::json!({"status": "completed"}),
                Some("0x123".to_string()),
                Duration::from_secs(1),
            ),
            agent_id: AgentId::new("agent-1"),
            timestamp: chrono::Utc::now(),
        };

        let json = serde_json::to_string(&response).unwrap();
        let deserialized: TaskResponse = serde_json::from_str(&json).unwrap();

        assert_eq!(response.message_id, deserialized.message_id);
        assert_eq!(response.agent_id, deserialized.agent_id);
    }
}
