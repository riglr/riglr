//! Integration helpers for riglr-core components.
//!
//! This module provides utilities for integrating riglr-agents with the
//! broader riglr ecosystem, including SignerContext management, job queue
//! integration, and tool compatibility.

use crate::{Agent, AgentError, Task, TaskResult, Result};
use riglr_core::{SignerContext, TransactionSigner, ToolError, Job, JobResult};
use std::sync::Arc;
use tracing::{debug, info, warn};

/// Integration utilities for SignerContext management in multi-agent systems.
pub struct SignerContextIntegration;

impl SignerContextIntegration {
    /// Execute an agent task within a signer context.
    ///
    /// This method ensures that the agent's task execution maintains the
    /// proper signer context isolation while integrating with riglr-core's
    /// security model.
    ///
    /// # Arguments
    ///
    /// * `signer` - The signer to use for the task execution
    /// * `agent` - The agent to execute the task
    /// * `task` - The task to execute
    ///
    /// # Returns
    ///
    /// The task result from the agent execution.
    pub async fn execute_with_signer(
        signer: Arc<dyn TransactionSigner>,
        agent: Arc<dyn Agent>,
        task: Task,
    ) -> Result<TaskResult> {
        info!("Executing task {} with agent {} in signer context", 
              task.id, agent.id());

        let task_id = task.id.clone();
        let agent_id = agent.id().clone();
        let task_id_for_error = task.id.clone();

        // Execute the task within the signer context
        let result = SignerContext::with_signer(signer, async move {
            debug!("Task {} starting execution in agent {}", task_id, agent_id);
            
            // Call agent hooks
            agent.before_task(&task).await.map_err(|e| match e {
                AgentError::Tool { source } => match source {
                    riglr_core::ToolError::SignerContext(signer_err) => signer_err,
                    _ => riglr_core::SignerError::Configuration(format!("Agent hook failed: {}", source)),
                },
                _ => riglr_core::SignerError::Configuration(format!("Agent hook failed: {}", e)),
            })?;
            
            let start_time = std::time::Instant::now();
            let result = agent.execute_task(task.clone()).await;
            let duration = start_time.elapsed();
            
            debug!("Task {} completed in agent {} after {:?}", 
                   task_id, agent_id, duration);
            
            // Call after hook regardless of result
            if let Ok(ref task_result) = result {
                let _ = agent.after_task(&task, task_result).await;
            }
            
            match result {
                Ok(task_result) => Ok(task_result),
                Err(AgentError::Tool { source }) => match source {
                    riglr_core::ToolError::SignerContext(signer_err) => Err(signer_err),
                    _ => Err(riglr_core::SignerError::Configuration(format!("Agent execution failed: {}", source))),
                },
                Err(e) => Err(riglr_core::SignerError::Configuration(format!("Agent execution failed: {}", e))),
            }
        }).await.map_err(|e| AgentError::task_execution_with_source(
            format!("Failed to execute task {} in signer context", task_id_for_error), e
        ))?;

        Ok(result)
    }

    /// Convert an agent task to a riglr-core Job.
    ///
    /// This enables agents to be integrated with riglr-core's job queue system.
    ///
    /// # Arguments
    ///
    /// * `task` - The agent task to convert
    ///
    /// # Returns
    ///
    /// A riglr-core Job representation of the task.
    pub fn task_to_job(task: &Task) -> Result<Job> {
        let job = Job::new(
            format!("agent_task:{}", task.task_type),
            &serde_json::json!({
                "task_id": task.id,
                "task_type": task.task_type,
                "parameters": task.parameters,
                "priority": task.priority,
                "metadata": task.metadata
            }),
            task.max_retries,
        ).map_err(|e| AgentError::task_execution_with_source(
            "Failed to convert task to job", e
        ))?;

        Ok(job)
    }

    /// Convert a riglr-core Job to an agent Task.
    ///
    /// This enables riglr-core jobs to be executed by agents.
    ///
    /// # Arguments
    ///
    /// * `job` - The riglr-core job to convert
    ///
    /// # Returns
    ///
    /// An agent Task representation of the job.
    pub fn job_to_task(job: &Job) -> Result<Task> {
        let params = job.params.as_object()
            .ok_or_else(|| AgentError::task_execution("Job parameters must be an object"))?;

        let task_id = params.get("task_id")
            .and_then(|v| v.as_str())
            .unwrap_or(&job.job_id.to_string())
            .to_string();

        let task_type = params.get("task_type")
            .ok_or_else(|| AgentError::task_execution("Missing task_type in job parameters"))?;

        let task_type = serde_json::from_value(task_type.clone())
            .map_err(|e| AgentError::task_execution_with_source("Invalid task_type", e))?;

        let parameters = params.get("parameters")
            .cloned()
            .unwrap_or(serde_json::Value::Null);

        let priority = params.get("priority")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default();

        let metadata = params.get("metadata")
            .and_then(|v| v.as_object())
            .map(|obj| {
                obj.iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect()
            })
            .unwrap_or_default();

        let mut task = Task::new(task_type, parameters);
        task.id = task_id;
        task.priority = priority;
        task.max_retries = job.max_retries;
        task.retry_count = job.retry_count;
        task.metadata = metadata;

        Ok(task)
    }

    /// Convert an agent TaskResult to a riglr-core JobResult.
    ///
    /// # Arguments
    ///
    /// * `task_result` - The agent task result to convert
    ///
    /// # Returns
    ///
    /// A riglr-core JobResult representation.
    pub fn task_result_to_job_result(task_result: &TaskResult) -> Result<JobResult> {
        let job_result = match task_result {
            TaskResult::Success { data, tx_hash, .. } => {
                JobResult::Success {
                    value: data.clone(),
                    tx_hash: tx_hash.clone(),
                }
            }
            TaskResult::Failure { error, retriable, .. } => {
                if *retriable {
                    JobResult::retriable_failure(error)
                } else {
                    JobResult::permanent_failure(error)
                }
            }
            TaskResult::Cancelled { reason } => {
                JobResult::permanent_failure(format!("Task cancelled: {}", reason))
            }
            TaskResult::Timeout { duration } => {
                JobResult::retriable_failure(format!("Task timed out after {:?}", duration))
            }
        };

        Ok(job_result)
    }

    /// Convert a riglr-core JobResult to an agent TaskResult.
    ///
    /// # Arguments
    ///
    /// * `job_result` - The riglr-core job result to convert
    /// * `duration` - The execution duration to include in the task result
    ///
    /// # Returns
    ///
    /// An agent TaskResult representation.
    pub fn job_result_to_task_result(
        job_result: &JobResult,
        duration: std::time::Duration,
    ) -> TaskResult {
        match job_result {
            JobResult::Success { value, tx_hash } => {
                TaskResult::Success {
                    data: value.clone(),
                    tx_hash: tx_hash.clone(),
                    duration,
                }
            }
            JobResult::Failure { error, retriable } => {
                TaskResult::Failure {
                    error: error.clone(),
                    retriable: *retriable,
                    duration,
                }
            }
        }
    }

    /// Convert an AgentError to a ToolError for riglr-core integration.
    ///
    /// # Arguments
    ///
    /// * `agent_error` - The agent error to convert
    ///
    /// # Returns
    ///
    /// A ToolError representation suitable for riglr-core systems.
    pub fn agent_error_to_tool_error(agent_error: AgentError) -> ToolError {
        agent_error.into()
    }

    /// Convert a ToolError to an AgentError.
    ///
    /// # Arguments
    ///
    /// * `tool_error` - The tool error to convert
    ///
    /// # Returns
    ///
    /// An AgentError representation.
    pub fn tool_error_to_agent_error(tool_error: ToolError) -> AgentError {
        AgentError::Tool { source: tool_error }
    }

    /// Check if an agent can execute riglr-core tools.
    ///
    /// This validates that the agent has the necessary capabilities
    /// to execute tools from the riglr ecosystem.
    ///
    /// # Arguments
    ///
    /// * `agent` - The agent to check
    /// * `required_capabilities` - List of required capabilities
    ///
    /// # Returns
    ///
    /// true if the agent can execute the tools, false otherwise.
    pub fn can_execute_tools(agent: &dyn Agent, required_capabilities: &[String]) -> bool {
        let agent_capabilities = agent.capabilities();
        required_capabilities.iter().all(|cap| agent_capabilities.contains(cap))
    }

    /// Validate that an agent is compatible with riglr-core patterns.
    ///
    /// This performs various compatibility checks to ensure the agent
    /// can work properly within the riglr ecosystem.
    ///
    /// # Arguments
    ///
    /// * `agent` - The agent to validate
    ///
    /// # Returns
    ///
    /// Ok(()) if the agent is compatible, Err with details otherwise.
    pub fn validate_agent_compatibility(agent: &dyn Agent) -> Result<()> {
        // Check that agent has an ID
        if agent.id().as_str().is_empty() {
            return Err(AgentError::configuration("Agent ID cannot be empty"));
        }

        // Check that agent has at least one capability
        if agent.capabilities().is_empty() {
            warn!("Agent {} has no declared capabilities", agent.id());
        }

        // Check that agent is available
        if !agent.is_available() {
            return Err(AgentError::agent_unavailable(
                agent.id().as_str(),
                format!("{:?}", agent.status().status),
            ));
        }

        debug!("Agent {} passed compatibility validation", agent.id());
        Ok(())
    }
}

/// Wrapper for executing riglr-core tools within agent contexts.
pub struct ToolExecutor {
    agent: Arc<dyn Agent>,
}

impl ToolExecutor {
    /// Create a new tool executor for the given agent.
    pub fn new(agent: Arc<dyn Agent>) -> Self {
        Self { agent }
    }

    /// Execute a riglr-core job using the agent.
    ///
    /// This method bridges riglr-core's Job system with the agent execution model.
    ///
    /// # Arguments
    ///
    /// * `job` - The riglr-core job to execute
    ///
    /// # Returns
    ///
    /// A riglr-core JobResult from the execution.
    pub async fn execute_job(&self, job: Job) -> Result<JobResult> {
        debug!("Executing job {} with agent {}", job.job_id, self.agent.id());

        // Convert job to task
        let task = SignerContextIntegration::job_to_task(&job)?;

        // Execute task
        let start_time = std::time::Instant::now();
        let task_result = self.agent.execute_task(task).await?;
        let duration = start_time.elapsed();

        // Convert result back to job result
        let job_result = SignerContextIntegration::task_result_to_job_result(&task_result)?;

        info!("Job {} completed by agent {} in {:?}", 
              job.job_id, self.agent.id(), duration);

        Ok(job_result)
    }

    /// Get the underlying agent.
    pub fn agent(&self) -> &Arc<dyn Agent> {
        &self.agent
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
    }

    #[async_trait::async_trait]
    impl Agent for MockAgent {
        async fn execute_task(&self, task: Task) -> Result<TaskResult> {
            Ok(TaskResult::success(
                serde_json::json!({
                    "agent_id": self.id.as_str(),
                    "task_id": task.id
                }),
                Some("0x123".to_string()),
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

    #[test]
    fn test_task_to_job_conversion() {
        let task = Task::new(
            TaskType::Trading,
            serde_json::json!({"symbol": "BTC/USD"}),
        ).with_priority(Priority::High);

        let job = SignerContextIntegration::task_to_job(&task).unwrap();
        assert_eq!(job.tool_name, "agent_task:trading");
        assert!(job.params.get("task_id").is_some());
        assert!(job.params.get("parameters").is_some());
    }

    #[test]
    fn test_job_to_task_conversion() {
        let job_params = serde_json::json!({
            "task_id": "test-task",
            "task_type": "Trading",
            "parameters": {"symbol": "BTC/USD"},
            "priority": "High",
            "metadata": {"source": "test"}
        });

        let job = Job::new("agent_task:trading", &job_params, 3).unwrap();
        let task = SignerContextIntegration::job_to_task(&job).unwrap();

        assert_eq!(task.id, "test-task");
        assert_eq!(task.task_type, TaskType::Trading);
        assert_eq!(task.priority, Priority::High);
        assert_eq!(task.metadata.get("source"), Some(&serde_json::json!("test")));
    }

    #[test]
    fn test_task_result_to_job_result_conversion() {
        let task_result = TaskResult::success(
            serde_json::json!({"result": "success"}),
            Some("0x123".to_string()),
            std::time::Duration::from_millis(100),
        );

        let job_result = SignerContextIntegration::task_result_to_job_result(&task_result).unwrap();
        
        match job_result {
            JobResult::Success { value, tx_hash } => {
                assert_eq!(value, serde_json::json!({"result": "success"}));
                assert_eq!(tx_hash, Some("0x123".to_string()));
            }
            _ => panic!("Expected success job result"),
        }
    }

    #[test]
    fn test_job_result_to_task_result_conversion() {
        let job_result = JobResult::Success {
            value: serde_json::json!({"result": "test"}),
            tx_hash: Some("0x456".to_string()),
        };

        let task_result = SignerContextIntegration::job_result_to_task_result(
            &job_result,
            std::time::Duration::from_millis(200),
        );

        match task_result {
            TaskResult::Success { data, tx_hash, duration } => {
                assert_eq!(data, serde_json::json!({"result": "test"}));
                assert_eq!(tx_hash, Some("0x456".to_string()));
                assert_eq!(duration, std::time::Duration::from_millis(200));
            }
            _ => panic!("Expected success task result"),
        }
    }

    #[test]
    fn test_agent_compatibility_validation() {
        let good_agent = MockAgent {
            id: AgentId::new("good-agent"),
            capabilities: vec!["trading".to_string()],
        };

        let empty_id_agent = MockAgent {
            id: AgentId::new(""),
            capabilities: vec!["trading".to_string()],
        };

        // Good agent should pass validation
        assert!(SignerContextIntegration::validate_agent_compatibility(&good_agent).is_ok());

        // Empty ID agent should fail validation
        assert!(SignerContextIntegration::validate_agent_compatibility(&empty_id_agent).is_err());
    }

    #[test]
    fn test_can_execute_tools() {
        let agent = MockAgent {
            id: AgentId::new("test-agent"),
            capabilities: vec!["trading".to_string(), "risk_analysis".to_string()],
        };

        // Should return true for capabilities the agent has
        assert!(SignerContextIntegration::can_execute_tools(
            &agent,
            &["trading".to_string()]
        ));

        assert!(SignerContextIntegration::can_execute_tools(
            &agent,
            &["trading".to_string(), "risk_analysis".to_string()]
        ));

        // Should return false for capabilities the agent doesn't have
        assert!(!SignerContextIntegration::can_execute_tools(
            &agent,
            &["research".to_string()]
        ));

        assert!(!SignerContextIntegration::can_execute_tools(
            &agent,
            &["trading".to_string(), "research".to_string()]
        ));
    }

    #[tokio::test]
    async fn test_tool_executor() {
        let agent = Arc::new(MockAgent {
            id: AgentId::new("executor-agent"),
            capabilities: vec!["trading".to_string()],
        });

        let executor = ToolExecutor::new(agent.clone());

        let job_params = serde_json::json!({
            "task_id": "test-task",
            "task_type": "Trading",
            "parameters": {"symbol": "BTC/USD"},
            "priority": "Normal"
        });

        let job = Job::new("agent_task:trading", &job_params, 3).unwrap();
        let result = executor.execute_job(job).await.unwrap();

        match result {
            JobResult::Success { value, .. } => {
                assert!(value.get("agent_id").is_some());
                assert!(value.get("task_id").is_some());
            }
            _ => panic!("Expected success job result"),
        }
    }

    #[test]
    fn test_error_conversions() {
        let agent_error = AgentError::agent_not_found("test-agent");
        let tool_error = SignerContextIntegration::agent_error_to_tool_error(agent_error);
        assert!(!tool_error.is_retriable());

        let tool_error = ToolError::retriable_string("Network timeout");
        let agent_error = SignerContextIntegration::tool_error_to_agent_error(tool_error);
        assert!(matches!(agent_error, AgentError::Tool { .. }));
    }
}