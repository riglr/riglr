// riglr-agents/src/agents/tool_calling.rs

use crate::toolset::Toolset;
use crate::*;
use rig::completion::Prompt;
use riglr_core::provider::ApplicationContext;
use riglr_core::{idempotency::InMemoryIdempotencyStore, ExecutionConfig, ToolWorker};
use std::marker::PhantomData;
use std::sync::Arc;
use tracing::info;

/// A simple wrapper to make rig::Agent debuggable
pub struct DebuggableAgent<M: rig::completion::CompletionModel + std::fmt::Debug>(
    pub rig::agent::Agent<M>,
);

impl<M: rig::completion::CompletionModel + std::fmt::Debug> std::fmt::Debug for DebuggableAgent<M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DebuggableAgent").finish_non_exhaustive()
    }
}

/// An agent that specializes in calling tools based on LLM decisions.
/// Made generic over the IdempotencyStore and Model for flexibility.
#[derive(Debug)]
pub struct ToolCallingAgent<
    S: riglr_core::idempotency::IdempotencyStore + std::fmt::Debug + 'static,
    M: rig::completion::CompletionModel + std::fmt::Debug + 'static,
> {
    id: AgentId,
    rig_agent: DebuggableAgent<M>,
    tool_worker: Arc<ToolWorker<S>>,
}

impl<
        S: riglr_core::idempotency::IdempotencyStore + std::fmt::Debug + 'static,
        M: rig::completion::CompletionModel + std::fmt::Debug + 'static,
    > ToolCallingAgent<S, M>
{
    /// Creates a new ToolCallingAgent with the given rig Agent and ToolWorker
    pub fn new(rig_agent: rig::agent::Agent<M>, tool_worker: Arc<ToolWorker<S>>) -> Self {
        Self {
            id: AgentId::generate(),
            rig_agent: DebuggableAgent(rig_agent),
            tool_worker,
        }
    }
}

#[async_trait::async_trait]
impl<
        S: riglr_core::idempotency::IdempotencyStore + std::fmt::Debug + 'static,
        M: rig::completion::CompletionModel + std::fmt::Debug + Send + Sync + 'static,
    > Agent for ToolCallingAgent<S, M>
{
    async fn execute_task(&self, task: Task) -> Result<TaskResult> {
        info!("--- ToolCallingAgent ENGAGED ---");
        let task_description = format!("Execute task: {:?}", task);
        info!("RECEIVED PROMPT: '{}'", task_description);

        // 1. The brain makes a decision and returns a JSON string.
        let llm_response_str = self.rig_agent.0.prompt(&task_description).await?;

        // 2. Parse the JSON string into structured ToolCall objects.
        let tool_calls: Vec<rig::completion::message::ToolCall> =
            serde_json::from_str(&llm_response_str).map_err(|e| {
                AgentError::task_execution_with_source("Failed to parse LLM tool calls", e)
            })?;

        if tool_calls.is_empty() {
            return Ok(TaskResult::failure(
                "LLM did not select a tool.".to_string(),
                false,
                std::time::Duration::ZERO,
            ));
        }

        // For simplicity, we execute the first tool call.
        let tool_call = &tool_calls[0];
        info!(
            "LLM DECISION: Call tool '{}' with args: {}",
            tool_call.function.name, tool_call.function.arguments
        );

        // 3. The agent creates a job from that decision.
        let job = riglr_core::Job::new(&tool_call.function.name, &tool_call.function.arguments, 3)?;

        // 4. The agent delegates execution to the ToolWorker.
        let job_result = self.tool_worker.process_job(job).await?;

        // 5. The agent reports the result.
        Ok(SignerContextIntegration::job_result_to_task_result(
            &job_result,
            std::time::Duration::ZERO,
        ))
    }

    fn id(&self) -> &AgentId {
        &self.id
    }

    fn capabilities(&self) -> Vec<String> {
        vec!["tool_calling".to_string()]
    }
}

/// A fluent builder for creating a `ToolCallingAgent`.
/// This handles all the boilerplate of wiring up the components.
pub struct ToolCallingAgentBuilder<
    S: riglr_core::idempotency::IdempotencyStore + std::fmt::Debug + 'static,
> {
    toolset: Toolset,
    app_context: ApplicationContext,
    execution_config: ExecutionConfig,
    _store: PhantomData<S>,
}

impl ToolCallingAgentBuilder<InMemoryIdempotencyStore> {
    /// Creates a new builder with a default `InMemoryIdempotencyStore`.
    pub fn new(toolset: Toolset, app_context: ApplicationContext) -> Self {
        Self {
            toolset,
            app_context,
            execution_config: ExecutionConfig::default(),
            _store: PhantomData,
        }
    }
}

impl<S: riglr_core::idempotency::IdempotencyStore + std::fmt::Debug + 'static>
    ToolCallingAgentBuilder<S>
{
    /// Specifies a custom `ExecutionConfig` for the `ToolWorker`.
    pub fn with_execution_config(mut self, config: ExecutionConfig) -> Self {
        self.execution_config = config;
        self
    }

    /// Specifies a custom `IdempotencyStore` for the `ToolWorker`.
    pub fn with_idempotency_store<
        NewS: riglr_core::idempotency::IdempotencyStore + std::fmt::Debug + 'static,
    >(
        self,
    ) -> ToolCallingAgentBuilder<NewS> {
        ToolCallingAgentBuilder {
            toolset: self.toolset,
            app_context: self.app_context,
            execution_config: self.execution_config,
            _store: PhantomData,
        }
    }

    /// Builds and initializes the `ToolCallingAgent`.
    ///
    /// This method is `async` to ensure that all components, including the
    /// tool worker, are fully initialized before the agent is returned.
    pub async fn build<
        M: rig::completion::CompletionModel + std::fmt::Debug + Send + Sync + 'static,
    >(
        self,
        model: M,
    ) -> crate::Result<Arc<ToolCallingAgent<S, M>>> {
        // 1. Create the ToolWorker with the specified store and config.
        let tool_worker = Arc::new(ToolWorker::<S>::new(
            self.execution_config,
            self.app_context.clone(),
        ));

        // 2. Register all tools with the worker and AWAIT completion.
        self.toolset.register_with_worker(&tool_worker).await;

        // 3. Create the rig::Agent brain.
        let agent_builder = rig::agent::AgentBuilder::new(model);

        // 5. Register all tools with the brain.
        let rig_agent = self.toolset.register_with_brain(agent_builder).build();

        // 6. Assemble and return the final agent.
        Ok(Arc::new(ToolCallingAgent::new(rig_agent, tool_worker)))
    }
}
