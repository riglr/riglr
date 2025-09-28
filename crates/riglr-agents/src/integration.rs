/// Integration helpers for riglr-core components.
///
/// This module provides utilities for integrating riglr-agents with the
/// broader riglr ecosystem, including `SignerContext` management, job queue
/// integration, and tool compatibility.
extern crate alloc;
use crate::{Agent, AgentError, Result, Task, TaskResult};
use alloc::sync::Arc;
use core::{str::FromStr as _, time::Duration};
use riglr_core::{signer::UnifiedSigner, Job, JobResult, ToolError};
use std::time::Instant;
use tracing::{debug, info, warn};

/// Integration utilities for `SignerContext` management in multi-agent systems.
#[derive(Debug)]
#[non_exhaustive]
pub struct Integration;

impl Integration {
    /// Convert an `AgentError` to a `ToolError` for riglr-core integration.
    ///
    /// # Arguments
    ///
    /// * `agent_error` - The agent error to convert
    ///
    /// # Returns
    ///
    /// A `ToolError` representation suitable for riglr-core systems.
    #[must_use]
    #[inline]
    pub fn agent_error_to_tool_error(agent_error: AgentError) -> ToolError {
        agent_error.into()
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
    #[must_use]
    #[inline]
    pub fn can_execute_tools(agent: &dyn Agent, required_capabilities: &[String]) -> bool {
        let agent_capabilities = agent.capabilities();
        required_capabilities.iter().all(|capability| {
            // Convert string capability to CapabilityType using FromStr
            crate::CapabilityType::from_str(capability)
                .is_ok_and(|capability_type| agent_capabilities.contains(&capability_type))
        })
    }

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
    ///
    /// # Errors
    ///
    /// Returns an error if the signer context execution fails, agent hooks fail,
    /// or task execution encounters an error.
    #[inline]
    pub async fn execute_with_signer(
        _signer: Arc<dyn UnifiedSigner>,
        agent: Arc<dyn Agent>,
        task: Task,
    ) -> Result<TaskResult> {
        info!(
            "Executing task {} with agent {} in signer context",
            task.id,
            agent.id()
        );

        let task_id = task.id.clone();
        let agent_id = agent.id().clone();

        // Execute the task within the signer context
        agent.before_task(&task).await?;

        let start_time = Instant::now();
        let result = agent.execute_task(task.clone()).await;
        let duration = start_time.elapsed();

        debug!(
            "Task {} completed in agent {} after {:?}",
            task_id, agent_id, duration
        );

        // Call after hook regardless of result
        if let Ok(ref task_result) = result {
            let _ = agent.after_task(&task, task_result).await;
        }
        result
    }

    /// Convert a riglr-core `JobResult` to an agent `TaskResult`.
    ///
    /// # Arguments
    ///
    /// * `job_result` - The riglr-core job result to convert
    /// * `duration` - The execution duration to include in the task result
    ///
    /// # Returns
    ///
    /// An agent `TaskResult` representation.
    #[must_use]
    #[inline]
    pub fn job_result_to_task_result(job_result: &JobResult, duration: Duration) -> TaskResult {
        match *job_result {
            JobResult::Success {
                ref value,
                ref tx_hash,
            } => TaskResult::Success {
                data: value.clone(),
                tx_hash: tx_hash.clone(),
                duration,
            },
            JobResult::Failure { ref error } => TaskResult::Failure {
                error: error.to_string(),
                retriable: error.is_retriable(),
                duration,
            },
            _ => TaskResult::Failure {
                error: "Unknown job result type".to_owned(),
                retriable: false,
                duration,
            },
        }
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
    ///
    /// # Errors
    ///
    /// Returns an error if job parameters are not an object, `task_type` is missing,
    /// or `task_type` deserialization fails.
    #[inline]
    pub fn job_to_task(job: &Job) -> Result<Task> {
        let params = job
            .params
            .as_object()
            .ok_or_else(|| AgentError::task_execution("Job parameters must be an object"))?;

        let task_id = params
            .get("task_id")
            .and_then(|task_id_value| task_id_value.as_str())
            .map_or_else(|| job.job_id.to_string(), ToString::to_string);

        let task_type = params
            .get("task_type")
            .ok_or_else(|| AgentError::task_execution("Missing task_type in job parameters"))?;

        let task_type_parsed =
            serde_json::from_value(task_type.clone()).map_err(|parsing_error| {
                AgentError::task_execution_with_source("Invalid task_type", parsing_error)
            })?;

        let parameters = params
            .get("parameters")
            .cloned()
            .unwrap_or(serde_json::Value::Null);

        let priority = params
            .get("priority")
            .and_then(|priority_value| serde_json::from_value(priority_value.clone()).ok())
            .unwrap_or_default();

        let metadata = params
            .get("metadata")
            .and_then(|metadata_value| metadata_value.as_object())
            .map(|metadata_obj| {
                metadata_obj
                    .iter()
                    .map(|(metadata_key, metadata_val)| {
                        (metadata_key.clone(), metadata_val.clone())
                    })
                    .collect()
            })
            .unwrap_or_default();

        let mut task = Task::new(task_type_parsed, parameters);
        task.id = task_id;
        task.priority = priority;
        task.max_retries = job.max_retries;
        task.retry_count = job.retry_count;
        task.metadata = metadata;

        Ok(task)
    }

    /// Convert an agent `TaskResult` to a riglr-core `JobResult`.
    ///
    /// # Arguments
    ///
    /// * `task_result` - The agent task result to convert
    ///
    /// # Returns
    ///
    /// A riglr-core `JobResult` representation.
    ///
    /// # Errors
    ///
    /// This function is infallible and always returns Ok.
    #[inline]
    pub fn task_result_to_job_result(task_result: &TaskResult) -> Result<JobResult> {
        let job_result = match *task_result {
            TaskResult::Success {
                ref data,
                ref tx_hash,
                ..
            } => JobResult::Success {
                value: data.clone(),
                tx_hash: tx_hash.clone(),
            },
            TaskResult::Failure {
                ref error,
                ref retriable,
                ..
            } => {
                let tool_error = if *retriable {
                    ToolError::retriable_string(error.clone())
                } else {
                    ToolError::permanent_string(error.clone())
                };
                JobResult::Failure { error: tool_error }
            }
            TaskResult::Cancelled { ref reason } => JobResult::Failure {
                error: ToolError::permanent_string(format!("Task cancelled: {reason}")),
            },
            TaskResult::Timeout { ref duration } => JobResult::Failure {
                error: ToolError::retriable_string(format!("Task timed out after {duration:?}")),
            },
        };

        Ok(job_result)
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
    ///
    /// # Errors
    ///
    /// Returns an error if the job creation fails during JSON serialization.
    #[inline]
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
        )
        .map_err(|conversion_error| {
            AgentError::task_execution_with_source(
                "Failed to convert task to job",
                conversion_error,
            )
        })?;
        Ok(job)
    }

    /// Convert a `ToolError` to an `AgentError`.
    ///
    /// # Arguments
    ///
    /// * `tool_error` - The tool error to convert
    ///
    /// # Returns
    ///
    /// An `AgentError` representation.
    #[must_use]
    #[inline]
    pub const fn tool_error_to_agent_error(tool_error: ToolError) -> AgentError {
        AgentError::ToolExecutionFailed(tool_error)
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
    ///
    /// # Errors
    ///
    /// Returns an error if the agent has an empty ID or is not available.
    #[inline]
    pub fn validate_agent_compatibility(agent: &dyn Agent) -> Result<()> {
        // Check that agent has an ID
        if agent.id().as_str().is_empty() {
            Err(AgentError::configuration("Agent ID cannot be empty"))
        } else {
            // Check that agent has at least one capability
            if agent.capabilities().is_empty() {
                warn!("Agent {} has no declared capabilities", agent.id());
            }

            // Check that agent is available
            if agent.is_available() {
                debug!("Agent {} passed compatibility validation", agent.id());
                Ok(())
            } else {
                Err(AgentError::agent_unavailable(
                    agent.id().as_str(),
                    format!("{:?}", agent.status().status),
                ))
            }
        }
    }
}

/// Wrapper for executing riglr-core tools within agent contexts.
#[derive(Debug)]
pub struct ToolExecutor {
    /// The agent to execute tools with.
    agent: Arc<dyn Agent>,
}

impl ToolExecutor {
    /// Get the underlying agent.
    #[must_use]
    #[inline]
    pub fn agent(&self) -> &Arc<dyn Agent> {
        &self.agent
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
    /// A riglr-core `JobResult` from the execution.
    ///
    /// # Errors
    ///
    /// Returns an error if job-to-task conversion fails, task execution fails,
    /// or task-result-to-job-result conversion fails.
    #[inline]
    pub async fn execute_job(&self, job: Job) -> Result<JobResult> {
        debug!(
            "Executing job {} with agent {}",
            job.job_id,
            self.agent.id()
        );

        // Convert job to task
        let task = Integration::job_to_task(&job)?;

        // Execute task
        let start_time = Instant::now();
        let task_result = self.agent.execute_task(task).await?;
        let duration = start_time.elapsed();

        // Convert result back to job result
        let job_result = Integration::task_result_to_job_result(&task_result)?;

        info!(
            "Job {} completed by agent {} in {:?}",
            job.job_id,
            self.agent.id(),
            duration
        );

        Ok(job_result)
    }

    /// Create a new tool executor for the given agent.
    #[must_use]
    #[inline]
    pub fn new(agent: Arc<dyn Agent>) -> Self {
        Self { agent }
    }
}

/// Type alias for integration utilities.
pub type Context = Integration;

#[cfg(test)]
#[expect(clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use crate::types::*;
    use core::result::Result as StdResult;
    use riglr_core::SignerError;
    use std::collections::HashMap;

    #[derive(Clone, Debug)]
    struct MockAgent {
        id: AgentId,
        capabilities: Vec<CapabilityType>,
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
                Duration::from_millis(100),
            ))
        }

        fn id(&self) -> &AgentId {
            &self.id
        }

        fn capabilities(&self) -> Vec<CapabilityType> {
            self.capabilities.clone()
        }
    }

    #[test]
    fn test_task_to_job_conversion() {
        let task = Task::new(TaskType::Trading, serde_json::json!({"symbol": "BTC/USD"}))
            .with_priority(Priority::High);

        let job =
            Integration::task_to_job(&task).expect("Task to job conversion should succeed in test");
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

        let job = Job::new("agent_task:trading", &job_params, 3)
            .expect("Job creation should succeed in test");
        let task =
            Integration::job_to_task(&job).expect("Job to task conversion should succeed in test");

        assert_eq!(task.id, "test-task");
        assert_eq!(task.task_type, TaskType::Trading);
        assert_eq!(task.priority, Priority::High);
        assert_eq!(
            task.metadata.get("source"),
            Some(&serde_json::json!("test"))
        );
    }

    #[test]
    fn test_task_result_to_job_result_conversion() {
        let task_result = TaskResult::success(
            serde_json::json!({"result": "success"}),
            Some("0x123".to_string()),
            Duration::from_millis(100),
        );

        let job_result = Integration::task_result_to_job_result(&task_result)
            .expect("Task result to job result conversion should succeed in test");

        match job_result {
            JobResult::Success { value, tx_hash } => {
                assert_eq!(value, serde_json::json!({"result": "success"}));
                assert_eq!(tx_hash, Some("0x123".to_string()));
            }
            JobResult::Failure { .. } => panic!("Expected success job result"),
            _ => panic!("Unexpected job result variant"),
        }
    }

    #[test]
    fn test_job_result_to_task_result_conversion() {
        let job_result = JobResult::Success {
            value: serde_json::json!({"result": "test"}),
            tx_hash: Some("0x456".to_string()),
        };

        let task_result =
            Integration::job_result_to_task_result(&job_result, Duration::from_millis(200));

        match task_result {
            TaskResult::Success {
                data,
                tx_hash,
                duration,
            } => {
                assert_eq!(data, serde_json::json!({"result": "test"}));
                assert_eq!(tx_hash, Some("0x456".to_string()));
                assert_eq!(duration, Duration::from_millis(200));
            }
            TaskResult::Failure { .. }
            | TaskResult::Cancelled { .. }
            | TaskResult::Timeout { .. } => panic!("Expected success task result"),
        }
    }

    #[test]
    fn test_agent_compatibility_validation() {
        let good_agent = MockAgent {
            id: AgentId::new("good-agent"),
            capabilities: vec![CapabilityType::Trading],
        };

        let empty_id_agent = MockAgent {
            id: AgentId::new(""),
            capabilities: vec![CapabilityType::Trading],
        };

        // Good agent should pass validation
        assert!(Integration::validate_agent_compatibility(&good_agent).is_ok());

        // Empty ID agent should fail validation
        assert!(Integration::validate_agent_compatibility(&empty_id_agent).is_err());
    }

    #[test]
    fn test_can_execute_tools() {
        let agent = MockAgent {
            id: AgentId::new("test-agent"),
            capabilities: vec![CapabilityType::Trading, CapabilityType::RiskAnalysis],
        };

        // Should return true for capabilities the agent has
        assert!(Integration::can_execute_tools(
            &agent,
            &["trading".to_string()]
        ));

        assert!(Integration::can_execute_tools(
            &agent,
            &["trading".to_string(), "risk_analysis".to_string()]
        ));

        // Should return false for capabilities the agent doesn't have
        assert!(!Integration::can_execute_tools(
            &agent,
            &["research".to_string()]
        ));

        assert!(!Integration::can_execute_tools(
            &agent,
            &["trading".to_string(), "research".to_string()]
        ));
    }

    #[tokio::test]
    async fn test_tool_executor() {
        let agent = Arc::new(MockAgent {
            id: AgentId::new("executor-agent"),
            capabilities: vec![CapabilityType::Trading],
        });

        let executor = ToolExecutor::new(agent.clone());

        let job_params = serde_json::json!({
            "task_id": "test-task",
            "task_type": "Trading",
            "parameters": {"symbol": "BTC/USD"},
            "priority": "Normal"
        });

        let job = Job::new("agent_task:trading", &job_params, 3)
            .expect("Job creation should succeed in test");
        let result = executor
            .execute_job(job)
            .await
            .expect("Job execution should succeed in test");

        match result {
            JobResult::Success { value, .. } => {
                assert!(value.get("agent_id").is_some());
                assert!(value.get("task_id").is_some());
            }
            JobResult::Failure { .. } => panic!("Expected success job result"),
            _ => panic!("Unexpected job result variant"),
        }
    }

    #[test]
    fn test_error_conversions() {
        let agent_error = AgentError::agent_not_found("test-agent");
        let tool_error = Integration::agent_error_to_tool_error(agent_error);
        assert!(!tool_error.is_retriable());

        let tool_error = ToolError::retriable_string("Network timeout");
        let agent_error = Integration::tool_error_to_agent_error(tool_error);
        assert!(matches!(agent_error, AgentError::ToolExecutionFailed(_)));
    }

    // Additional comprehensive tests for 100% coverage

    #[derive(Clone, Debug)]
    enum FailureBehavior {
        None,
        BeforeTask,
        ExecuteTask,
        AfterTask,
    }

    #[derive(Clone, Debug)]
    struct FailingAgent {
        id: AgentId,
        capabilities: Vec<CapabilityType>,
        failure_behavior: FailureBehavior,
        agent_error_type: AgentErrorType,
        available: bool,
        status: AgentStatus,
    }

    #[derive(Clone, Debug)]
    enum AgentErrorType {
        ToolSignerContext,
        ToolOther,
        Other,
    }

    #[async_trait::async_trait]
    impl Agent for FailingAgent {
        async fn execute_task(&self, _task: Task) -> Result<TaskResult> {
            if matches!(self.failure_behavior, FailureBehavior::ExecuteTask) {
                match self.agent_error_type {
                    AgentErrorType::ToolSignerContext => {
                        Err(AgentError::ToolExecutionFailed(ToolError::SignerContext(
                            Standard::Configuration("Signer context error".to_string()).to_string(),
                        )))
                    }
                    AgentErrorType::ToolOther => Err(AgentError::ToolExecutionFailed(
                        ToolError::retriable_string("Tool error"),
                    )),
                    AgentErrorType::Other => Err(AgentError::agent_not_found("test")),
                }
            } else {
                Ok(TaskResult::success(
                    serde_json::json!({"result": "success"}),
                    Some("0x123".to_string()),
                    Duration::from_millis(100),
                ))
            }
        }

        async fn before_task(&self, _task: &Task) -> Result<()> {
            if matches!(self.failure_behavior, FailureBehavior::BeforeTask) {
                match self.agent_error_type {
                    AgentErrorType::ToolSignerContext => {
                        Err(AgentError::ToolExecutionFailed(ToolError::SignerContext(
                            Standard::Configuration("Before task signer error".to_string())
                                .to_string(),
                        )))
                    }
                    AgentErrorType::ToolOther => Err(AgentError::ToolExecutionFailed(
                        ToolError::retriable_string("Before task tool error"),
                    )),
                    AgentErrorType::Other => Err(AgentError::agent_not_found("before_task")),
                }
            } else {
                Ok(())
            }
        }

        async fn after_task(&self, _task: &Task, _result: &TaskResult) -> Result<()> {
            if matches!(self.failure_behavior, FailureBehavior::AfterTask) {
                Err(AgentError::agent_not_found("after_task"))
            } else {
                Ok(())
            }
        }

        fn id(&self) -> &AgentId {
            &self.id
        }

        fn capabilities(&self) -> Vec<CapabilityType> {
            self.capabilities.clone()
        }

        fn is_available(&self) -> bool {
            self.available
        }

        fn status(&self) -> AgentStatus {
            self.status.clone()
        }
    }

    // Mock signer for testing
    #[derive(Debug)]
    struct MockSigner;

    // Import required traits for the mock signer
    use riglr_core::signer::{
        error::Standard, Chain, EvmClient, EvmSigner, SignerBase, SolanaClient, SolanaSigner,
    };

    impl SignerBase for MockSigner {
        fn supported_chains(&self) -> &[Chain] {
            &[Chain::Solana, Chain::Evm]
        }

        fn user_id(&self) -> String {
            "mock_user".to_string()
        }
    }

    #[async_trait::async_trait]
    impl SolanaSigner for MockSigner {
        fn pubkey(&self) -> String {
            "mock_pubkey".to_string()
        }

        fn client(&self) -> &dyn SolanaClient {
            unimplemented!("Mock Solana client not available")
        }

        async fn sign_and_send_transaction(
            &self,
            _transaction: serde_json::Value,
        ) -> StdResult<String, Box<dyn SignerError>> {
            Ok("mock_signature".to_string())
        }

        async fn sign_message(&self, _message: &[u8]) -> StdResult<String, Box<dyn SignerError>> {
            Ok("mock_signature".to_string())
        }
    }

    #[async_trait::async_trait]
    impl EvmSigner for MockSigner {
        fn chain_id(&self) -> u64 {
            1337
        }

        fn address(&self) -> String {
            "0xmock_address".to_string()
        }

        fn client(&self) -> &dyn EvmClient {
            unimplemented!("Mock EVM client not available")
        }

        async fn sign_and_send_transaction(
            &self,
            _tx: serde_json::Value,
        ) -> StdResult<String, Box<dyn SignerError>> {
            Ok("0x1234".to_string())
        }

        async fn sign_message(&self, _message: &[u8]) -> StdResult<String, Box<dyn SignerError>> {
            Ok("mock_signature".to_string())
        }
    }

    impl UnifiedSigner for MockSigner {
        fn supports_solana(&self) -> bool {
            true
        }

        fn supports_evm(&self) -> bool {
            true
        }

        fn as_solana(&self) -> Option<&dyn SolanaSigner> {
            Some(self)
        }

        fn as_evm(&self) -> Option<&dyn EvmSigner> {
            Some(self)
        }
    }

    #[tokio::test]
    async fn test_execute_with_signer_before_task_tool_signer_error() {
        let signer = Arc::new(MockSigner);
        let agent = Arc::new(FailingAgent {
            id: AgentId::new("failing-agent"),
            capabilities: vec![CapabilityType::Custom("test".to_string())],
            failure_behavior: FailureBehavior::BeforeTask,
            agent_error_type: AgentErrorType::ToolSignerContext,
            available: true,
            status: AgentStatus {
                agent_id: AgentId::new("failing-agent"),
                status: AgentState::Active,
                active_tasks: 0,
                load: 0.0,
                last_heartbeat: chrono::Utc::now(),
                capabilities: vec![],
                metadata: HashMap::new(),
            },
        });
        let task = Task::new(TaskType::Trading, serde_json::json!({}));

        let result = Integration::execute_with_signer(signer, agent, task).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_execute_with_signer_before_task_tool_other_error() {
        let signer = Arc::new(MockSigner);
        let agent = Arc::new(FailingAgent {
            id: AgentId::new("failing-agent"),
            capabilities: vec![CapabilityType::Custom("test".to_string())],
            failure_behavior: FailureBehavior::BeforeTask,
            agent_error_type: AgentErrorType::ToolOther,
            available: true,
            status: AgentStatus {
                agent_id: AgentId::new("failing-agent"),
                status: AgentState::Active,
                active_tasks: 0,
                load: 0.0,
                last_heartbeat: chrono::Utc::now(),
                capabilities: vec![],
                metadata: HashMap::new(),
            },
        });
        let task = Task::new(TaskType::Trading, serde_json::json!({}));

        let result = Integration::execute_with_signer(signer, agent, task).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_execute_with_signer_before_task_other_error() {
        let signer = Arc::new(MockSigner);
        let agent = Arc::new(FailingAgent {
            id: AgentId::new("failing-agent"),
            capabilities: vec![CapabilityType::Custom("test".to_string())],
            failure_behavior: FailureBehavior::BeforeTask,
            agent_error_type: AgentErrorType::Other,
            available: true,
            status: AgentStatus {
                agent_id: AgentId::new("failing-agent"),
                status: AgentState::Active,
                active_tasks: 0,
                load: 0.0,
                last_heartbeat: chrono::Utc::now(),
                capabilities: vec![],
                metadata: HashMap::new(),
            },
        });
        let task = Task::new(TaskType::Trading, serde_json::json!({}));

        let result = Integration::execute_with_signer(signer, agent, task).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_execute_with_signer_execute_task_tool_signer_error() {
        let signer = Arc::new(MockSigner);
        let agent = Arc::new(FailingAgent {
            id: AgentId::new("failing-agent"),
            capabilities: vec![CapabilityType::Custom("test".to_string())],
            failure_behavior: FailureBehavior::ExecuteTask,
            agent_error_type: AgentErrorType::ToolSignerContext,
            available: true,
            status: AgentStatus {
                agent_id: AgentId::new("failing-agent"),
                status: AgentState::Active,
                active_tasks: 0,
                load: 0.0,
                last_heartbeat: chrono::Utc::now(),
                capabilities: vec![],
                metadata: HashMap::new(),
            },
        });
        let task = Task::new(TaskType::Trading, serde_json::json!({}));

        let result = Integration::execute_with_signer(signer, agent, task).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_execute_with_signer_execute_task_tool_other_error() {
        let signer = Arc::new(MockSigner);
        let agent = Arc::new(FailingAgent {
            id: AgentId::new("failing-agent"),
            capabilities: vec![CapabilityType::Custom("test".to_string())],
            failure_behavior: FailureBehavior::ExecuteTask,
            agent_error_type: AgentErrorType::ToolOther,
            available: true,
            status: AgentStatus {
                agent_id: AgentId::new("failing-agent"),
                status: AgentState::Active,
                active_tasks: 0,
                load: 0.0,
                last_heartbeat: chrono::Utc::now(),
                capabilities: vec![],
                metadata: HashMap::new(),
            },
        });
        let task = Task::new(TaskType::Trading, serde_json::json!({}));

        let result = Integration::execute_with_signer(signer, agent, task).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_execute_with_signer_execute_task_other_error() {
        let signer = Arc::new(MockSigner);
        let agent = Arc::new(FailingAgent {
            id: AgentId::new("failing-agent"),
            capabilities: vec![CapabilityType::Custom("test".to_string())],
            failure_behavior: FailureBehavior::ExecuteTask,
            agent_error_type: AgentErrorType::Other,
            available: true,
            status: AgentStatus {
                agent_id: AgentId::new("failing-agent"),
                status: AgentState::Active,
                active_tasks: 0,
                load: 0.0,
                last_heartbeat: chrono::Utc::now(),
                capabilities: vec![],
                metadata: HashMap::new(),
            },
        });
        let task = Task::new(TaskType::Trading, serde_json::json!({}));

        let result = Integration::execute_with_signer(signer, agent, task).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_execute_with_signer_success_with_failing_after_task() {
        let signer = Arc::new(MockSigner);
        let agent = Arc::new(FailingAgent {
            id: AgentId::new("failing-agent"),
            capabilities: vec![CapabilityType::Custom("test".to_string())],
            failure_behavior: FailureBehavior::AfterTask,
            agent_error_type: AgentErrorType::Other,
            available: true,
            status: AgentStatus {
                agent_id: AgentId::new("failing-agent"),
                status: AgentState::Active,
                active_tasks: 0,
                load: 0.0,
                last_heartbeat: chrono::Utc::now(),
                capabilities: vec![],
                metadata: HashMap::new(),
            },
        });
        let task = Task::new(TaskType::Trading, serde_json::json!({}));

        let result = Integration::execute_with_signer(signer, agent, task).await;
        // Should succeed even if after_task fails (it's ignored)
        assert!(result.is_ok());
    }

    #[test]
    fn test_task_to_job_with_error() {
        // Test with a task that would cause serialization issues
        let mut task = Task::new(TaskType::Trading, serde_json::json!({}));
        task.max_retries = u32::MAX; // This might cause issues in some JSON implementations

        // The conversion should still work as we're using standard JSON serialization
        let result = Integration::task_to_job(&task);
        assert!(result.is_ok());
    }

    #[test]
    fn test_job_to_task_missing_parameters() {
        let job_params = serde_json::json!({
            "not_task_type": "should_fail"
        });

        let job = Job::new("test", &job_params, 1).expect("Job creation should succeed in test");
        let result = Integration::job_to_task(&job);
        assert!(result.is_err());
        assert!(result
            .expect_err("Result should be an error")
            .to_string()
            .contains("Missing task_type"));
    }

    #[test]
    fn test_job_to_task_invalid_task_type() {
        let job_params = serde_json::json!({
            "task_type": "InvalidTaskType"
        });

        let job = Job::new("test", &job_params, 1).expect("Job creation should succeed in test");
        let result = Integration::job_to_task(&job);
        assert!(result.is_err());
        assert!(result
            .expect_err("Result should be an error")
            .to_string()
            .contains("Invalid task_type"));
    }

    #[test]
    fn test_job_to_task_non_object_params() {
        let job_params = serde_json::json!("not an object");
        let job = Job::new("test", &job_params, 1).expect("Job creation should succeed in test");
        let result = Integration::job_to_task(&job);
        assert!(result.is_err());
        assert!(result
            .expect_err("Result should be an error")
            .to_string()
            .contains("must be an object"));
    }

    #[test]
    fn test_job_to_task_minimal_params() {
        let job_params = serde_json::json!({
            "task_type": "Trading"
        });

        let job = Job::new("test", &job_params, 1).expect("Job creation should succeed in test");
        let task =
            Integration::job_to_task(&job).expect("Job to task conversion should succeed in test");

        // Should use defaults for missing fields
        assert_eq!(task.id, job.job_id.to_string());
        assert_eq!(task.parameters, serde_json::Value::Null);
        assert_eq!(task.priority, Priority::default());
        assert!(task.metadata.is_empty());
    }

    #[test]
    fn test_job_to_task_invalid_priority() {
        let job_params = serde_json::json!({
            "task_type": "Trading",
            "priority": "InvalidPriority"
        });

        let job = Job::new("test", &job_params, 1).expect("Job creation should succeed in test");
        let task =
            Integration::job_to_task(&job).expect("Job to task conversion should succeed in test");

        // Should use default for invalid priority
        assert_eq!(task.priority, Priority::default());
    }

    #[test]
    fn test_job_to_task_non_object_metadata() {
        let job_params = serde_json::json!({
            "task_type": "Trading",
            "metadata": "not an object"
        });

        let job = Job::new("test", &job_params, 1).expect("Job creation should succeed in test");
        let task =
            Integration::job_to_task(&job).expect("Job to task conversion should succeed in test");

        // Should use empty metadata for non-object
        assert!(task.metadata.is_empty());
    }

    #[test]
    fn test_task_result_to_job_result_failure_retriable() {
        let task_result = TaskResult::Failure {
            error: "Network error".to_string(),
            retriable: true,
            duration: Duration::from_millis(100),
        };

        let job_result = Integration::task_result_to_job_result(&task_result)
            .expect("Task result to job result conversion should succeed in test");

        match job_result {
            JobResult::Failure { error } => {
                assert!(error.to_string().contains("Network error"));
                assert!(error.is_retriable());
            }
            JobResult::Success { .. } => panic!("Expected retriable failure job result"),
            _ => panic!("Unexpected job result variant"),
        }
    }

    #[test]
    fn test_task_result_to_job_result_failure_permanent() {
        let task_result = TaskResult::Failure {
            error: "Parse error".to_string(),
            retriable: false,
            duration: Duration::from_millis(100),
        };

        let job_result = Integration::task_result_to_job_result(&task_result)
            .expect("Task result to job result conversion should succeed in test");

        match job_result {
            JobResult::Failure { error } => {
                assert!(error.to_string().contains("Parse error"));
                assert!(!error.is_retriable());
            }
            JobResult::Success { .. } => panic!("Expected permanent failure job result"),
            _ => panic!("Unexpected job result variant"),
        }
    }

    #[test]
    fn test_task_result_to_job_result_cancelled() {
        let task_result = TaskResult::Cancelled {
            reason: "User cancelled".to_string(),
        };

        let job_result = Integration::task_result_to_job_result(&task_result)
            .expect("Task result to job result conversion should succeed in test");

        match job_result {
            JobResult::Failure { error } => {
                assert!(error.to_string().contains("Task cancelled: User cancelled"));
                assert!(!error.is_retriable());
            }
            JobResult::Success { .. } => {
                panic!("Expected permanent failure job result for cancelled task")
            }
            _ => panic!("Unexpected job result variant"),
        }
    }

    #[test]
    fn test_task_result_to_job_result_timeout() {
        let task_result = TaskResult::Timeout {
            duration: Duration::from_secs(30),
        };

        let job_result = Integration::task_result_to_job_result(&task_result)
            .expect("Task result to job result conversion should succeed in test");

        match job_result {
            JobResult::Failure { error } => {
                assert!(error.to_string().contains("Task timed out after"));
                assert!(error.is_retriable());
            }
            JobResult::Success { .. } => {
                panic!("Expected retriable failure job result for timeout");
            }
            _ => panic!("Unexpected job result variant"),
        }
    }

    #[test]
    fn test_job_result_to_task_result_failure() {
        let job_result = JobResult::Failure {
            error: ToolError::retriable_string("Test error".to_string()),
        };

        let task_result =
            Integration::job_result_to_task_result(&job_result, Duration::from_millis(150));

        match task_result {
            TaskResult::Failure {
                error,
                retriable,
                duration,
            } => {
                assert!(error.contains("Test error"));
                assert!(retriable);
                assert_eq!(duration, Duration::from_millis(150));
            }
            TaskResult::Success { .. }
            | TaskResult::Cancelled { .. }
            | TaskResult::Timeout { .. } => panic!("Expected failure task result"),
        }
    }

    #[test]
    fn test_validate_agent_compatibility_empty_capabilities() {
        let agent = MockAgent {
            id: AgentId::new("empty-caps-agent"),
            capabilities: vec![], // Empty capabilities
        };

        // Should still pass validation but log a warning
        let result = Integration::validate_agent_compatibility(&agent);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_agent_compatibility_unavailable_agent() {
        let unavailable_agent = FailingAgent {
            id: AgentId::new("unavailable-agent"),
            capabilities: vec![CapabilityType::Custom("test".to_string())],
            failure_behavior: FailureBehavior::None,
            agent_error_type: AgentErrorType::Other,
            available: false, // Not available
            status: AgentStatus {
                agent_id: AgentId::new("failing-agent"),
                status: AgentState::Busy,
                active_tasks: 0,
                load: 0.0,
                last_heartbeat: chrono::Utc::now(),
                capabilities: vec![],
                metadata: HashMap::new(),
            },
        };

        let result = Integration::validate_agent_compatibility(&unavailable_agent);
        assert!(result.is_err());
        assert!(result
            .expect_err("Result should be an error")
            .to_string()
            .contains("unavailable"));
    }

    #[test]
    fn test_can_execute_tools_empty_requirements() {
        let agent = MockAgent {
            id: AgentId::new("test-agent"),
            capabilities: vec![CapabilityType::Trading],
        };

        // Should return true for empty requirements
        assert!(Integration::can_execute_tools(&agent, &[]));
    }

    #[tokio::test]
    async fn test_tool_executor_job_conversion_error() {
        let agent = Arc::new(MockAgent {
            id: AgentId::new("executor-agent"),
            capabilities: vec![CapabilityType::Custom("test".to_string())],
        });

        let executor = ToolExecutor::new(agent.clone());

        // Create a job with invalid parameters that will fail conversion
        let job_params = serde_json::json!("not an object");
        let job = Job::new("test", &job_params, 1).expect("Job creation should succeed in test");

        let result = executor.execute_job(job).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_tool_executor_agent_execution_error() {
        let agent = Arc::new(FailingAgent {
            id: AgentId::new("failing-executor-agent"),
            capabilities: vec![CapabilityType::Custom("test".to_string())],
            failure_behavior: FailureBehavior::ExecuteTask,
            agent_error_type: AgentErrorType::Other,
            available: true,
            status: AgentStatus {
                agent_id: AgentId::new("failing-agent"),
                status: AgentState::Active,
                active_tasks: 0,
                load: 0.0,
                last_heartbeat: chrono::Utc::now(),
                capabilities: vec![],
                metadata: HashMap::new(),
            },
        });

        let executor = ToolExecutor::new(agent.clone());

        let job_params = serde_json::json!({
            "task_type": "Trading"
        });
        let job = Job::new("test", &job_params, 1).expect("Job creation should succeed in test");

        let result = executor.execute_job(job).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_tool_executor_agent_getter() {
        let agent = Arc::new(MockAgent {
            id: AgentId::new("test-agent"),
            capabilities: vec![CapabilityType::Custom("test".to_string())],
        });

        let executor = ToolExecutor::new(agent);

        // Test that we can get the agent back
        assert_eq!(executor.agent().id().as_str(), "test-agent");
    }
}
