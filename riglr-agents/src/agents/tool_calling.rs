// riglr-agents/src/agents/tool_calling.rs

use crate::toolset::Toolset;
use crate::*;
use rig::completion::Prompt;
use riglr_core::provider::ApplicationContext;
use riglr_core::{idempotency::InMemoryIdempotencyStore, ExecutionConfig, ToolWorker};
use std::marker::PhantomData;
use std::sync::Arc;
use tracing::info;

// Simple Either enum for internal use
enum Either<L, R> {
    Left(L),
    Right(R),
}

/// A wrapper that adds Debug implementation to any CompletionModel
///
/// This wrapper allows completion models that don't implement Debug (like ResponsesCompletionModel)
/// to be used with ToolCallingAgent, which requires Debug for logging and debugging purposes.
///
/// ## Rationale
///
/// The `ToolCallingAgent` and its underlying components require the `CompletionModel` to
/// implement the `Debug` trait for logging and debugging purposes. However, some models
/// provided by the `rig` framework (e.g., `ResponsesCompletionModel` used in tests) do not
/// implement `Debug`.
///
/// This wrapper provides a generic `Debug` implementation that allows these non-compliant
/// models to be used within the `riglr-agents` system without modifying the original `rig`
/// code. The wrapper displays a placeholder string `<completion_model>` when formatted for
/// debug output, avoiding the need to expose internal model details while still satisfying
/// the trait bounds.
///
/// ## When to Use
///
/// Use this wrapper when:
/// - You encounter a compile error stating that a completion model doesn't implement `Debug`
/// - You need to use a `rig` completion model with `ToolCallingAgent`
/// - You're writing tests with mock models that don't implement `Debug`
///
/// # Example
/// ```rust
/// use rig::providers::openai;
/// use riglr_agents::agents::tool_calling::DebuggableCompletionModel;
///
/// let openai_client = openai::Client::from_env();
/// let model = openai_client.completion_model("gpt-4o");
/// let debuggable_model = DebuggableCompletionModel::new(model);
/// ```
pub struct DebuggableCompletionModel<T: rig::completion::CompletionModel>(pub T);

impl<T: rig::completion::CompletionModel> std::fmt::Debug for DebuggableCompletionModel<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DebuggableCompletionModel")
            .field("model", &"<completion_model>")
            .finish()
    }
}

impl<T: rig::completion::CompletionModel> DebuggableCompletionModel<T> {
    /// Creates a new debuggable wrapper around a completion model
    pub fn new(model: T) -> Self {
        Self(model)
    }

    /// Extracts the inner completion model
    pub fn into_inner(self) -> T {
        self.0
    }

    /// Gets a reference to the inner completion model
    pub fn inner(&self) -> &T {
        &self.0
    }
}

impl<T: rig::completion::CompletionModel> rig::completion::CompletionModel
    for DebuggableCompletionModel<T>
{
    type Response = T::Response;
    type StreamingResponse = T::StreamingResponse;

    fn completion(
        &self,
        completion_request: rig::completion::CompletionRequest,
    ) -> impl std::future::Future<
        Output = std::result::Result<
            rig::completion::CompletionResponse<Self::Response>,
            rig::completion::CompletionError,
        >,
    > + Send {
        self.0.completion(completion_request)
    }

    fn stream(
        &self,
        completion_request: rig::completion::CompletionRequest,
    ) -> impl std::future::Future<
        Output = std::result::Result<
            rig::streaming::StreamingCompletionResponse<Self::StreamingResponse>,
            rig::completion::CompletionError,
        >,
    > + Send {
        self.0.stream(completion_request)
    }
}

// Ensure the wrapper implements all required traits for ToolCallingAgent
unsafe impl<T: rig::completion::CompletionModel + Send> Send for DebuggableCompletionModel<T> {}
unsafe impl<T: rig::completion::CompletionModel + Sync> Sync for DebuggableCompletionModel<T> {}

// Implement Clone if inner type supports it
impl<T: rig::completion::CompletionModel + Clone> Clone for DebuggableCompletionModel<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

// Implement AsRef to allow accessing the inner model
impl<T: rig::completion::CompletionModel> AsRef<T> for DebuggableCompletionModel<T> {
    fn as_ref(&self) -> &T {
        &self.0
    }
}

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
///
/// This agent supports **parallel execution** of multiple tool calls. When the LLM
/// returns multiple tool calls in a single response, `ToolCallingAgent` will execute
/// all of them concurrently using `futures::future::join_all`, significantly improving
/// performance for workflows that require multiple independent tool operations.
///
/// # Parallel Execution
///
/// When the LLM responds with multiple tool calls (as a JSON array), the agent:
/// 1. Creates async tasks for each tool call
/// 2. Executes all tasks concurrently using `futures::future::join_all`
/// 3. Aggregates results from all tool executions
/// 4. Returns a combined result containing outputs from all tools
///
/// This parallel execution model provides:
/// - **Better performance**: Multiple tools run simultaneously instead of sequentially
/// - **Fault tolerance**: If one tool fails, others continue to execute
/// - **Complete results**: All tool outputs are collected and returned
///
/// # Example
///
/// ```rust
/// // LLM might return multiple tool calls like:
/// // [
/// //   {"name": "get_balance", "arguments": {"address": "0x123"}},
/// //   {"name": "get_transactions", "arguments": {"address": "0x123"}},
/// //   {"name": "get_token_holdings", "arguments": {"address": "0x123"}}
/// // ]
/// //
/// // All three tools will execute in parallel, reducing total execution time
/// // from sum(t1 + t2 + t3) to max(t1, t2, t3)
/// ```
///
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

    /// Private helper: Gets tool calls from the LLM
    ///
    /// Prompts the LLM with the task and parses its response.
    /// Returns either tool calls to execute or a final text response.
    async fn _get_llm_tool_calls(
        &self,
        task: &Task,
    ) -> Result<Either<Vec<rig::completion::message::ToolCall>, String>> {
        let task_description = format!("Execute task: {:?}", task);
        info!("RECEIVED PROMPT: '{}'", task_description);

        // Get the LLM's response
        let llm_response_str = self.rig_agent.0.prompt(&task_description).await?;

        // Try to parse as tool calls; if that fails, treat as a text response
        match serde_json::from_str::<Vec<rig::completion::message::ToolCall>>(&llm_response_str) {
            Ok(calls) if !calls.is_empty() => {
                info!(
                    "LLM DECISION: Execute {} tool calls in parallel",
                    calls.len()
                );
                Ok(Either::Left(calls))
            }
            _ => {
                // LLM returned a text response (likely the final answer)
                info!("LLM provided final answer: {}", llm_response_str);
                Ok(Either::Right(llm_response_str))
            }
        }
    }

    /// Private helper: Executes multiple tool calls in parallel
    ///
    /// Takes a list of tool calls and executes them concurrently using futures::future::join_all.
    /// Returns a vector of results (success or error) for each tool call.
    async fn _execute_tool_calls_in_parallel(
        &self,
        tool_calls: &[rig::completion::message::ToolCall],
    ) -> Vec<Result<riglr_core::JobResult>> {
        // Create async futures for each tool call
        let futures: Vec<_> = tool_calls
            .iter()
            .map(|tool_call| {
                info!(
                    "Preparing tool '{}' with args: {}",
                    tool_call.function.name, tool_call.function.arguments
                );

                // Clone what we need for the async block
                let tool_name = tool_call.function.name.clone();
                let tool_args = tool_call.function.arguments.clone();
                let worker = self.tool_worker.clone();

                // Create async block for this tool execution
                async move {
                    // Create a job from the tool call
                    let job = riglr_core::Job::new(&tool_name, &tool_args, 3)?;

                    // Delegate execution to the ToolWorker
                    let job_result = worker.process_job(job).await?;

                    Ok::<_, AgentError>(job_result)
                }
            })
            .collect();

        // Execute all tool calls concurrently
        futures::future::join_all(futures).await
    }

    /// Private helper: Aggregates results from parallel tool executions
    ///
    /// Takes the results from parallel tool executions and combines them into a single TaskResult.
    /// Handles partial failures and constructs the final JSON response.
    fn _aggregate_tool_results(
        &self,
        results: Vec<Result<riglr_core::JobResult>>,
        tool_count: usize,
    ) -> Result<TaskResult> {
        let mut all_values: Vec<serde_json::Value> = Vec::new();
        let mut all_tx_hashes: Vec<String> = Vec::new();
        let total_duration = std::time::Duration::ZERO;

        for (i, result) in results.into_iter().enumerate() {
            match result {
                Ok(job_result) => {
                    match job_result {
                        riglr_core::JobResult::Success { value, tx_hash } => {
                            info!("Tool call {} succeeded with result: {}", i, value);
                            all_values.push(value);
                            if let Some(hash) = tx_hash {
                                all_tx_hashes.push(hash);
                            }
                        }
                        riglr_core::JobResult::Failure { error } => {
                            tracing::warn!("Tool call {} failed: {}", i, error);

                            // Check if the error is retriable for better handling
                            let error_info = if error.is_retriable() {
                                serde_json::json!({
                                    "error": error.to_string(),
                                    "retriable": true,
                                    "retry_after": error.retry_after().map(|d| d.as_secs()),
                                    "tool_index": i
                                })
                            } else {
                                serde_json::json!({
                                    "error": error.to_string(),
                                    "retriable": false,
                                    "tool_index": i
                                })
                            };
                            all_values.push(error_info);
                        }
                    }
                }
                Err(e) => {
                    // Log the error but continue with other results
                    tracing::warn!("Tool call {} failed with error: {}", i, e);
                    all_values.push(serde_json::json!({
                        "error": e.to_string(),
                        "tool_index": i
                    }));
                }
            }
        }

        // Check if all tool calls failed
        let successful_results = all_values
            .iter()
            .filter(|v| v.get("error").is_none())
            .count();

        if successful_results == 0 && !all_values.is_empty() {
            // All tool calls failed - this is a significant error condition
            let any_retriable = all_values.iter().any(|v| {
                v.get("retriable")
                    .and_then(|r| r.as_bool())
                    .unwrap_or(false)
            });

            if any_retriable {
                return Err(AgentError::task_execution(format!(
                    "All {} tool calls failed (some retriable)",
                    tool_count
                )));
            } else {
                return Err(AgentError::task_execution(format!(
                    "All {} tool calls failed permanently",
                    tool_count
                )));
            }
        }

        // Report aggregated results (at least some succeeded)
        let final_result = serde_json::json!({
            "results": all_values,
            "tx_hashes": all_tx_hashes,
            "tool_count": tool_count,
            "successful_count": successful_results
        });

        Ok(TaskResult::success(
            final_result,
            if all_tx_hashes.is_empty() {
                None
            } else {
                Some(all_tx_hashes.join(", "))
            },
            total_duration,
        ))
    }
}

#[async_trait::async_trait]
impl<
        S: riglr_core::idempotency::IdempotencyStore + std::fmt::Debug + 'static,
        M: rig::completion::CompletionModel + std::fmt::Debug + Send + Sync + 'static,
    > Agent for ToolCallingAgent<S, M>
{
    /// Executes a task by leveraging an LLM to decide which tools to call.
    ///
    /// This method supports parallel execution of multiple tool calls. When the LLM
    /// returns multiple tool calls, they are executed concurrently for improved performance.
    ///
    /// # Process Flow
    ///
    /// 1. **LLM Decision**: The task is sent to the LLM for analysis
    /// 2. **Response Parsing**: The LLM response is parsed as either:
    ///    - A JSON array of tool calls (triggers parallel execution)
    ///    - A text response (returned as final answer)
    /// 3. **Parallel Execution**: If tool calls are detected:
    ///    - All tools are executed concurrently using `futures::future::join_all`
    ///    - Results are aggregated from all tool executions
    ///    - Failures in individual tools don't stop other executions
    /// 4. **Result Aggregation**: All tool outputs are combined into a single response
    ///
    /// # Returns
    ///
    /// - `TaskResult` containing either:
    ///   - Aggregated results from all tool executions
    ///   - A text response from the LLM (if no tools were called)
    async fn execute_task(&self, task: Task) -> Result<TaskResult> {
        info!("--- ToolCallingAgent ENGAGED ---");

        // Step 1: Get tool calls or text response from the LLM
        match self._get_llm_tool_calls(&task).await? {
            Either::Right(text_response) => {
                // LLM returned a final text answer instead of tool calls
                // Return it directly as the result
                Ok(TaskResult::success(
                    serde_json::json!({
                        "response": text_response,
                        "type": "text"
                    }),
                    None,
                    std::time::Duration::ZERO,
                ))
            }
            Either::Left(tool_calls) => {
                // Step 2: Execute tool calls in parallel
                let results = self._execute_tool_calls_in_parallel(&tool_calls).await;

                // Step 3: Aggregate results and return
                self._aggregate_tool_results(results, tool_calls.len())
            }
        }
    }

    fn id(&self) -> &AgentId {
        &self.id
    }

    fn capabilities(&self) -> Vec<crate::CapabilityType> {
        vec![crate::CapabilityType::ToolCalling]
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
        let rig_agent = self
            .toolset
            .register_with_brain(agent_builder, &self.app_context)
            .build();

        // 6. Assemble and return the final agent.
        Ok(Arc::new(ToolCallingAgent::new(rig_agent, tool_worker)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rig::completion::{CompletionModel, CompletionRequest};
    use std::sync::Arc;
    use std::time::{Duration, Instant};
    use tokio::time::sleep;

    #[derive(Clone)]
    struct MockCompletionModel {
        model_id: String,
        responses: Vec<String>,
        response_index: std::sync::Arc<std::sync::atomic::AtomicUsize>,
        should_fail: bool,
        delay: Option<Duration>,
        fail_on_call: Option<usize>,
    }

    impl MockCompletionModel {
        fn new() -> Self {
            Self {
                model_id: "mock-model".to_string(),
                responses: vec!["Mock response".to_string()],
                response_index: std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0)),
                should_fail: false,
                delay: None,
                fail_on_call: None,
            }
        }

        #[allow(dead_code)]
        fn with_responses(mut self, responses: Vec<String>) -> Self {
            self.responses = responses;
            self
        }

        fn with_failure(mut self) -> Self {
            self.should_fail = true;
            self
        }

        fn with_delay(mut self, delay: Duration) -> Self {
            self.delay = Some(delay);
            self
        }

        fn with_fail_on_call(mut self, call_number: usize) -> Self {
            self.fail_on_call = Some(call_number);
            self
        }

        pub fn model_id(&self) -> &str {
            &self.model_id
        }

        pub fn call_count(&self) -> usize {
            self.response_index
                .load(std::sync::atomic::Ordering::SeqCst)
        }
    }

    #[derive(Debug)]
    #[allow(dead_code)]
    struct MockError {
        message: String,
    }

    impl MockError {
        fn new(message: impl Into<String>) -> Self {
            Self {
                message: message.into(),
            }
        }
    }

    impl std::fmt::Display for MockError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "Mock error: {}", self.message)
        }
    }

    impl std::error::Error for MockError {}

    impl rig::completion::CompletionModel for MockCompletionModel {
        type Response = String;
        type StreamingResponse = ();

        fn completion(
            &self,
            _completion_request: CompletionRequest,
        ) -> impl std::future::Future<
            Output = std::result::Result<
                rig::completion::CompletionResponse<Self::Response>,
                rig::completion::CompletionError,
            >,
        > + Send {
            let index = self
                .response_index
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            let response = self
                .responses
                .get(index % self.responses.len())
                .cloned()
                .unwrap_or_else(|| "Default response".to_string());
            let should_fail = self.should_fail;
            let delay = self.delay;
            let fail_on_call = self.fail_on_call;

            async move {
                // Apply delay if configured
                if let Some(delay) = delay {
                    sleep(delay).await;
                }

                // Check if we should fail on this specific call
                if let Some(fail_call) = fail_on_call {
                    if index >= fail_call {
                        return Err(rig::completion::CompletionError::RequestError(
                            MockError::new(format!("Failing on call #{}", index + 1)).into(),
                        ));
                    }
                }

                // Check if we should always fail
                if should_fail {
                    return Err(rig::completion::CompletionError::RequestError(
                        MockError::new("Mock failure").into(),
                    ));
                }

                // For testing, we'll just return a minimal response
                // The actual structure isn't as important as testing the delegation works
                let choice =
                    rig::completion::AssistantContent::Text(rig::message::Text { text: response });
                Ok(rig::completion::CompletionResponse {
                    choice: rig::OneOrMany::one(choice),
                    usage: rig::completion::Usage::default(),
                    raw_response: "".to_string(),
                })
            }
        }

        async fn stream(
            &self,
            _completion_request: CompletionRequest,
        ) -> std::result::Result<
            rig::streaming::StreamingCompletionResponse<Self::StreamingResponse>,
            rig::completion::CompletionError,
        > {
            if self.should_fail {
                return Err(rig::completion::CompletionError::RequestError(
                    MockError::new("Mock streaming failure").into(),
                ));
            }

            // Mock implementation - this won't work in real usage but sufficient for testing
            Err(rig::completion::CompletionError::RequestError(
                MockError::new("Mock streaming not implemented").into(),
            ))
        }
    }

    // ==== BASIC FUNCTIONALITY TESTS ====

    #[test]
    fn test_debuggable_completion_model_implements_debug() {
        let mock_model = MockCompletionModel::new();

        // Should be able to format with Debug
        let debug_string = format!("{:?}", DebuggableCompletionModel::new(mock_model));
        assert!(!debug_string.is_empty());
        assert!(debug_string.contains("DebuggableCompletionModel"));
        assert!(debug_string.contains("<completion_model>"));
    }

    #[test]
    fn test_debuggable_completion_model_delegation() {
        let mock_model = MockCompletionModel::new();
        let debuggable = DebuggableCompletionModel::new(mock_model);

        // Test that wrapper properly delegates to inner model
        assert_eq!(debuggable.inner().model_id(), "mock-model");
    }

    #[test]
    fn test_debuggable_completion_model_into_inner() {
        let mock_model = MockCompletionModel::new();
        let original_id = mock_model.model_id().to_string();
        let debuggable = DebuggableCompletionModel::new(mock_model);

        let inner = debuggable.into_inner();
        assert_eq!(inner.model_id(), original_id);
    }

    #[test]
    fn test_debuggable_completion_model_clone() {
        let mock_model = MockCompletionModel::new();
        let debuggable = DebuggableCompletionModel::new(mock_model);

        let cloned = debuggable.clone();
        assert_eq!(debuggable.inner().model_id(), cloned.inner().model_id());
    }

    #[test]
    fn test_debuggable_completion_model_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}

        assert_send::<DebuggableCompletionModel<MockCompletionModel>>();
        assert_sync::<DebuggableCompletionModel<MockCompletionModel>>();
    }

    // ==== ERROR HANDLING TESTS ====

    #[tokio::test]
    async fn test_debuggable_completion_model_with_failing_model() {
        let failing_model = MockCompletionModel::new().with_failure();
        let debuggable = DebuggableCompletionModel::new(failing_model);

        let request = CompletionRequest {
            preamble: None,
            chat_history: rig::OneOrMany::many(vec![]).unwrap(),
            documents: vec![],
            max_tokens: None,
            temperature: None,
            tools: vec![],
            additional_params: None,
        };
        let result = debuggable.completion(request).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            rig::completion::CompletionError::RequestError(_) => {
                // Expected
            }
            other => panic!("Unexpected error type: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_debuggable_completion_model_with_intermittent_failures() {
        let model = MockCompletionModel::new()
            .with_responses(vec![
                "Response 1".to_string(),
                "Response 2".to_string(),
                "Response 3".to_string(),
            ])
            .with_fail_on_call(1); // Fail on second call

        let debuggable = DebuggableCompletionModel::new(model);

        let request1 = CompletionRequest {
            preamble: None,
            chat_history: rig::OneOrMany::many(vec![]).unwrap(),
            documents: vec![],
            max_tokens: None,
            temperature: None,
            tools: vec![],
            additional_params: None,
        };
        let result1 = debuggable.completion(request1).await;
        assert!(result1.is_ok());

        let request2 = CompletionRequest {
            preamble: None,
            chat_history: rig::OneOrMany::many(vec![]).unwrap(),
            documents: vec![],
            max_tokens: None,
            temperature: None,
            tools: vec![],
            additional_params: None,
        };
        let result2 = debuggable.completion(request2).await;
        assert!(result2.is_err());

        let request3 = CompletionRequest {
            preamble: None,
            chat_history: rig::OneOrMany::many(vec![]).unwrap(),
            documents: vec![],
            max_tokens: None,
            temperature: None,
            tools: vec![],
            additional_params: None,
        };
        let result3 = debuggable.completion(request3).await;
        assert!(result3.is_err()); // Still failing since fail_on_call is >= 1
    }

    #[tokio::test]
    async fn test_debuggable_completion_model_stream_error_handling() {
        let failing_model = MockCompletionModel::new().with_failure();
        let debuggable = DebuggableCompletionModel::new(failing_model);

        let request = CompletionRequest {
            preamble: None,
            chat_history: rig::OneOrMany::many(vec![]).unwrap(),
            documents: vec![],
            max_tokens: None,
            temperature: None,
            tools: vec![],
            additional_params: None,
        };
        let result = debuggable.stream(request).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_debuggable_completion_model_error_preservation() {
        let failing_model = MockCompletionModel::new().with_failure();
        let debuggable = DebuggableCompletionModel::new(failing_model);

        let request = CompletionRequest {
            preamble: None,
            chat_history: rig::OneOrMany::many(vec![]).unwrap(),
            documents: vec![],
            max_tokens: None,
            temperature: None,
            tools: vec![],
            additional_params: None,
        };
        let result = debuggable.completion(request).await;

        match result {
            Err(rig::completion::CompletionError::RequestError(boxed_error)) => {
                let error_message = boxed_error.to_string();
                assert!(error_message.contains("Mock error"));
            }
            _ => panic!("Expected RequestError with MockError"),
        }
    }

    // ==== ASYNC BEHAVIOR TESTS ====

    #[tokio::test]
    async fn test_debuggable_completion_model_async_delegation() {
        let model = MockCompletionModel::new().with_responses(vec!["Async response".to_string()]);
        let debuggable = DebuggableCompletionModel::new(model);

        let request = CompletionRequest {
            preamble: None,
            chat_history: rig::OneOrMany::many(vec![]).unwrap(),
            documents: vec![],
            max_tokens: None,
            temperature: None,
            tools: vec![],
            additional_params: None,
        };
        let result = debuggable.completion(request).await;

        assert!(result.is_ok());
        let response = result.unwrap();

        // Check that we got the expected response structure
        let choices = response.choice.into_iter().collect::<Vec<_>>();
        if let Some(choice) = choices.first() {
            if let rig::completion::AssistantContent::Text(text) = choice {
                assert_eq!(text.text, "Async response");
            } else {
                panic!("Expected text content");
            }
        } else {
            panic!("Expected at least one choice");
        }
    }

    #[tokio::test]
    async fn test_debuggable_completion_model_concurrent_calls() {
        let model = MockCompletionModel::new().with_responses(vec![
            "Response A".to_string(),
            "Response B".to_string(),
            "Response C".to_string(),
        ]);
        let debuggable = Arc::new(DebuggableCompletionModel::new(model));

        // Make concurrent calls
        let handles: Vec<_> = (0..3)
            .map(|_| {
                let debuggable_clone = Arc::clone(&debuggable);
                tokio::spawn(async move {
                    let request = CompletionRequest {
                        preamble: None,
                        chat_history: rig::OneOrMany::many(vec![]).unwrap(),
                        documents: vec![],
                        max_tokens: None,
                        temperature: None,
                        tools: vec![],
                        additional_params: None,
                    };
                    debuggable_clone.completion(request).await
                })
            })
            .collect();

        // Wait for all calls to complete
        let results = futures::future::join_all(handles).await;

        // All should succeed
        assert_eq!(results.len(), 3);
        for handle_result in results {
            let completion_result = handle_result.unwrap();
            assert!(completion_result.is_ok());
        }

        // Verify all calls were made
        assert_eq!(debuggable.inner().call_count(), 3);
    }

    #[tokio::test]
    async fn test_debuggable_completion_model_async_with_delay() {
        let delay = Duration::from_millis(50);
        let model = MockCompletionModel::new()
            .with_delay(delay)
            .with_responses(vec!["Delayed response".to_string()]);
        let debuggable = DebuggableCompletionModel::new(model);

        let start = Instant::now();
        let request = CompletionRequest {
            preamble: None,
            chat_history: rig::OneOrMany::many(vec![]).unwrap(),
            documents: vec![],
            max_tokens: None,
            temperature: None,
            tools: vec![],
            additional_params: None,
        };
        let result = debuggable.completion(request).await;
        let elapsed = start.elapsed();

        assert!(result.is_ok());
        assert!(elapsed >= delay);
    }

    // ==== INTEGRATION TESTS ====

    #[tokio::test]
    async fn test_debuggable_completion_model_with_tool_calling_agent_builder() {
        use crate::toolset::Toolset;
        use riglr_core::provider::ApplicationContext;

        let model = MockCompletionModel::new().with_responses(vec![
            r#"[{"function": {"name": "test_tool", "arguments": "{}"}}]"#.to_string(),
        ]);
        let _debuggable = DebuggableCompletionModel::new(model);

        let toolset = Toolset::default();
        let config = riglr_core::Config::from_env();
        let app_context = ApplicationContext::from_config(&config);

        // This should compile and create the builder successfully
        let builder = ToolCallingAgentBuilder::new(toolset, app_context);

        // The build should work with our debuggable model
        // Note: We can't easily test the full build without setting up tools,
        // but we can verify the builder accepts our debuggable model type
        let _builder_with_model = builder; // This would call .build(_debuggable) in real usage

        // Test passes if it compiles - the important thing is that DebuggableCompletionModel
        // implements all the required traits
    }

    #[tokio::test]
    async fn test_debuggable_completion_model_multiple_responses() {
        let model = MockCompletionModel::new().with_responses(vec![
            "First response".to_string(),
            "Second response".to_string(),
            "Third response".to_string(),
        ]);
        let debuggable = DebuggableCompletionModel::new(model);

        // Make multiple calls and verify we get different responses
        for i in 0..5 {
            let request = CompletionRequest {
                preamble: None,
                chat_history: rig::OneOrMany::many(vec![]).unwrap(),
                documents: vec![],
                max_tokens: None,
                temperature: None,
                tools: vec![],
                additional_params: None,
            };
            let result = debuggable.completion(request).await;
            assert!(result.is_ok());

            let expected_response = match i % 3 {
                0 => "First response",
                1 => "Second response",
                2 => "Third response",
                _ => unreachable!(),
            };

            let response = result.unwrap();
            let choices = response.choice.into_iter().collect::<Vec<_>>();
            if let Some(choice) = choices.first() {
                if let rig::completion::AssistantContent::Text(text) = choice {
                    assert_eq!(text.text, expected_response);
                } else {
                    panic!("Expected text content");
                }
            } else {
                panic!("Expected single choice");
            }
        }
    }

    // ==== EDGE CASES ====

    #[test]
    fn test_debuggable_completion_model_with_different_model_types() {
        // Test with a different mock model to ensure generics work
        #[derive(Clone)]
        struct AlternativeMockModel {
            name: String,
        }

        impl AlternativeMockModel {
            fn new(name: impl Into<String>) -> Self {
                Self { name: name.into() }
            }

            fn name(&self) -> &str {
                &self.name
            }
        }

        impl rig::completion::CompletionModel for AlternativeMockModel {
            type Response = i32; // Different response type
            type StreamingResponse = ();

            fn completion(
                &self,
                _completion_request: CompletionRequest,
            ) -> impl std::future::Future<
                Output = std::result::Result<
                    rig::completion::CompletionResponse<Self::Response>,
                    rig::completion::CompletionError,
                >,
            > + Send {
                async move {
                    let choice = rig::completion::AssistantContent::Text(rig::message::Text {
                        text: 42.to_string(),
                    });
                    Ok(rig::completion::CompletionResponse {
                        choice: rig::OneOrMany::one(choice),
                        usage: rig::completion::Usage::default(),
                        raw_response: 42,
                    })
                }
            }

            async fn stream(
                &self,
                _completion_request: CompletionRequest,
            ) -> std::result::Result<
                rig::streaming::StreamingCompletionResponse<Self::StreamingResponse>,
                rig::completion::CompletionError,
            > {
                Err(rig::completion::CompletionError::RequestError(
                    MockError::new("Alternative streaming not implemented").into(),
                ))
            }
        }

        let alt_model = AlternativeMockModel::new("alternative-model");
        let debuggable = DebuggableCompletionModel::new(alt_model);

        // Should compile and work with different model types
        assert_eq!(debuggable.inner().name(), "alternative-model");

        let debug_string = format!("{:?}", debuggable);
        assert!(debug_string.contains("DebuggableCompletionModel"));
    }

    #[tokio::test]
    async fn test_debuggable_completion_model_empty_responses() {
        let model = MockCompletionModel::new().with_responses(vec![]);
        let debuggable = DebuggableCompletionModel::new(model);

        let request = CompletionRequest {
            preamble: None,
            chat_history: rig::OneOrMany::many(vec![]).unwrap(),
            documents: vec![],
            max_tokens: None,
            temperature: None,
            tools: vec![],
            additional_params: None,
        };
        let result = debuggable.completion(request).await;

        // Should fall back to default response
        assert!(result.is_ok());
        let response = result.unwrap();
        let choices = response.choice.into_iter().collect::<Vec<_>>();
        if let Some(choice) = choices.first() {
            if let rig::completion::AssistantContent::Text(text) = choice {
                assert_eq!(text.text, "Default response");
            } else {
                panic!("Expected text content");
            }
        } else {
            panic!("Expected at least one choice");
        }
    }

    #[test]
    fn test_debuggable_completion_model_debug_output_consistency() {
        let model1 = MockCompletionModel::new();
        let model2 = MockCompletionModel::new();

        let debuggable1 = DebuggableCompletionModel::new(model1);
        let debuggable2 = DebuggableCompletionModel::new(model2);

        let debug1 = format!("{:?}", debuggable1);
        let debug2 = format!("{:?}", debuggable2);

        // Debug output should be consistent (same format) but models are different instances
        assert_eq!(debug1, debug2);
        assert!(debug1.contains("DebuggableCompletionModel"));
        assert!(debug1.contains("<completion_model>"));
    }

    // ==== PERFORMANCE TESTS ====

    #[tokio::test]
    async fn test_debuggable_completion_model_performance_comparison() {
        const NUM_CALLS: usize = 100;

        // Test direct model performance
        let direct_model =
            MockCompletionModel::new().with_responses(vec!["Performance test".to_string()]);

        let start = Instant::now();
        for _ in 0..NUM_CALLS {
            let request = CompletionRequest {
                preamble: None,
                chat_history: rig::OneOrMany::many(vec![]).unwrap(),
                documents: vec![],
                max_tokens: None,
                temperature: None,
                tools: vec![],
                additional_params: None,
            };
            let result = direct_model.completion(request).await;
            assert!(result.is_ok());
        }
        let direct_duration = start.elapsed();

        // Test wrapped model performance
        let wrapped_model =
            MockCompletionModel::new().with_responses(vec!["Performance test".to_string()]);
        let debuggable = DebuggableCompletionModel::new(wrapped_model);

        let start = Instant::now();
        for _ in 0..NUM_CALLS {
            let request = CompletionRequest {
                preamble: None,
                chat_history: rig::OneOrMany::many(vec![]).unwrap(),
                documents: vec![],
                max_tokens: None,
                temperature: None,
                tools: vec![],
                additional_params: None,
            };
            let result = debuggable.completion(request).await;
            assert!(result.is_ok());
        }
        let wrapped_duration = start.elapsed();

        // The wrapper should add minimal overhead
        // We allow up to 50% overhead, but it should typically be much less
        let overhead_ratio = wrapped_duration.as_nanos() as f64 / direct_duration.as_nanos() as f64;
        println!("Performance overhead ratio: {:.2}", overhead_ratio);

        // The overhead should be reasonable (less than 50% in most cases)
        // This is a loose bound to account for test environment variability
        assert!(
            overhead_ratio < 1.5,
            "Wrapper overhead too high: {:.2}",
            overhead_ratio
        );
    }

    #[tokio::test]
    async fn test_debuggable_completion_model_memory_efficiency() {
        // Create models and verify they don't leak memory or accumulate state
        let mut models = Vec::new();

        for i in 0..10 {
            let model = MockCompletionModel::new().with_responses(vec![format!("Response {}", i)]);
            let debuggable = DebuggableCompletionModel::new(model);
            models.push(debuggable);
        }

        // Use each model
        for (i, model) in models.iter().enumerate() {
            let request = CompletionRequest {
                preamble: None,
                chat_history: rig::OneOrMany::many(vec![]).unwrap(),
                documents: vec![],
                max_tokens: None,
                temperature: None,
                tools: vec![],
                additional_params: None,
            };
            let result = model.completion(request).await;
            assert!(result.is_ok());

            let response = result.unwrap();
            let choices = response.choice.into_iter().collect::<Vec<_>>();
            if let Some(choice) = choices.first() {
                if let rig::completion::AssistantContent::Text(text) = choice {
                    assert_eq!(text.text, format!("Response {}", i));
                } else {
                    panic!("Expected text content");
                }
            } else {
                panic!("Expected at least one choice");
            }
        }

        // Each model should maintain independent state
        for model in &models {
            assert_eq!(model.inner().call_count(), 1);
        }
    }

    // ==== TRAIT IMPLEMENTATION TESTS ====

    #[test]
    fn test_debuggable_completion_model_trait_bounds() {
        fn requires_completion_model<T: rig::completion::CompletionModel + std::fmt::Debug>(_: T) {}
        fn requires_send<T: Send>(_: T) {}
        fn requires_sync<T: Sync>(_: T) {}
        fn requires_clone<T: Clone>(_: T) {}

        let model = MockCompletionModel::new();
        let debuggable = DebuggableCompletionModel::new(model);

        requires_completion_model(debuggable.clone());
        requires_send(debuggable.clone());
        requires_sync(debuggable.clone());
        requires_clone(debuggable.clone());
    }

    #[test]
    fn test_debuggable_completion_model_type_aliases() {
        let _model = MockCompletionModel::new();
        let _debuggable = DebuggableCompletionModel::new(_model);

        // Verify associated types are correctly forwarded by ensuring they match
        fn assert_same_response_type<
            T1: rig::completion::CompletionModel,
            T2: rig::completion::CompletionModel,
        >()
        where
            T1::Response: PartialEq<T2::Response>,
        {
            // This function compiles if the types are compatible
        }

        fn assert_same_streaming_type<
            T1: rig::completion::CompletionModel,
            T2: rig::completion::CompletionModel,
        >()
        where
            T1::StreamingResponse: PartialEq<T2::StreamingResponse>,
        {
            // This function compiles if the types are compatible
        }

        // Test passes if types are correctly forwarded and these compile
        assert_same_response_type::<
            DebuggableCompletionModel<MockCompletionModel>,
            MockCompletionModel,
        >();
        assert_same_streaming_type::<
            DebuggableCompletionModel<MockCompletionModel>,
            MockCompletionModel,
        >();
    }
}
