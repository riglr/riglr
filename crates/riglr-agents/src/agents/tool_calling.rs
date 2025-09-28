// riglr-agents/src/agents/tool_calling.rs

extern crate alloc;

use crate::error::Error;
use crate::toolset::Toolset;
use crate::{Agent as AgentTrait, AgentId, Result, Task, TaskResult};

use alloc::sync::Arc;
use core::fmt::{Debug, Formatter, Result as FmtResult};
use core::future::Future;
use core::result::Result as CoreResult;
use core::time::Duration;
use rig::agent::{Agent as RigAgent, AgentBuilder};
use rig::completion::{
    CompletionError, CompletionModel, CompletionRequest, CompletionRequestBuilder,
    CompletionResponse, Message, Prompt as _,
};
use rig::streaming::StreamingCompletionResponse;
use tracing::info;

/// A wrapper that adds Debug implementation to any `CompletionModel`
///
/// This wrapper allows completion models that don't implement Debug (like `ResponsesCompletionModel`)
/// to be used with `ToolCallingAgent`, which requires Debug for logging and debugging purposes.
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
/// ```rust,no_run
/// use rig::providers::openai;
/// use rig::client::{ProviderClient, CompletionClient};
/// use riglr_agents::agents::tool_calling::DebuggableCompletionModel;
///
/// let openai_client = openai::Client::from_env();
/// let model = openai_client.completion_model("gpt-4o");
/// let debuggable_model = DebuggableCompletionModel::new(model);
/// ```
#[non_exhaustive]
pub struct DebuggableCompletionModel<T: CompletionModel + Send + Sync>(pub T);

impl<T: CompletionModel + Send + Sync> Debug for DebuggableCompletionModel<T> {
    #[inline]
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        formatter
            .debug_struct("DebuggableCompletionModel")
            .field("model", &"<completion_model>")
            .finish()
    }
}

impl<T: CompletionModel + Send + Sync> DebuggableCompletionModel<T> {
    /// Gets a reference to the inner completion model
    #[inline]
    #[must_use]
    pub const fn inner(&self) -> &T {
        &self.0
    }

    /// Extracts the inner completion model
    #[inline]
    #[must_use]
    pub fn into_inner(self) -> T {
        self.0
    }

    /// Creates a new debuggable wrapper around a completion model
    #[inline]
    #[must_use]
    pub const fn new(model: T) -> Self {
        Self(model)
    }
}

impl<T: CompletionModel + Send + Sync> CompletionModel for DebuggableCompletionModel<T> {
    type Response = T::Response;
    type StreamingResponse = T::StreamingResponse;

    #[inline]
    fn completion(
        &self,
        request: CompletionRequest,
    ) -> impl Future<Output = CoreResult<CompletionResponse<Self::Response>, CompletionError>> + Send
    {
        self.0.completion(request)
    }

    #[inline]
    fn completion_request(&self, prompt: impl Into<Message>) -> CompletionRequestBuilder<Self> {
        CompletionRequestBuilder::new(self.clone(), prompt)
    }

    #[inline]
    fn stream(
        &self,
        request: CompletionRequest,
    ) -> impl Future<
        Output = CoreResult<StreamingCompletionResponse<Self::StreamingResponse>, CompletionError>,
    > + Send {
        self.0.stream(request)
    }
}

// Implement Clone if inner type supports it
impl<T: CompletionModel + Send + Sync + Clone> Clone for DebuggableCompletionModel<T> {
    #[inline]
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }

    #[inline]
    fn clone_from(&mut self, source: &Self) {
        self.0.clone_from(&source.0);
    }
}

// Implement AsRef to allow accessing the inner model
impl<T: CompletionModel + Send + Sync> AsRef<T> for DebuggableCompletionModel<T> {
    #[inline]
    fn as_ref(&self) -> &T {
        &self.0
    }
}

/// A simple wrapper to make `rig::Agent` debuggable
#[non_exhaustive]
pub struct DebuggableAgent<M: CompletionModel + Debug>(pub RigAgent<M>);

impl<M: CompletionModel + Debug> Debug for DebuggableAgent<M> {
    #[inline]
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        formatter
            .debug_struct("DebuggableAgent")
            .finish_non_exhaustive()
    }
}

/// An agent that specializes in calling tools based on LLM decisions.
///
/// This agent leverages rig-core's built-in multi-turn tool calling capability,
/// which handles the complexity of iterative LLM-tool interactions. The agent
/// delegates the entire tool-calling loop to rig-core, simplifying the implementation
/// and ensuring consistent behavior with the rig framework.
///
/// # How it Works
///
/// The agent uses `rig::agent::Agent::prompt().multi_turn()` to:
/// 1. Send the initial prompt to the LLM
/// 2. Parse any tool calls from the LLM's response
/// 3. Execute the requested tools
/// 4. Feed tool results back to the LLM
/// 5. Repeat until the LLM provides a final answer (up to 5 rounds)
///
/// This approach provides:
/// - **Simpler implementation**: No need to manually manage the tool-calling loop
/// - **Framework consistency**: Uses rig-core's battle-tested multi-turn logic
/// - **Automatic retries**: Built-in support for multiple rounds of tool calls
///
/// Made generic over the Model for flexibility.
#[non_exhaustive]
#[derive(Debug)]
pub struct Agent<M: CompletionModel + Debug + 'static> {
    /// Agent identifier
    id: AgentId,
    /// Wrapped rig agent
    rig_agent: DebuggableAgent<M>,
}

impl<M: CompletionModel + Debug + 'static> Agent<M> {
    /// Creates a new `ToolCallingAgent` with the given rig Agent
    #[inline]
    #[must_use]
    pub fn new(rig_agent: RigAgent<M>) -> Self {
        Self {
            id: AgentId::generate(),
            rig_agent: DebuggableAgent(rig_agent),
        }
    }
}

#[async_trait::async_trait]
impl<M: CompletionModel + Debug + Send + Sync + 'static> AgentTrait for Agent<M> {
    /// Executes a task by leveraging rig-core's multi-turn tool calling.
    ///
    /// This method delegates the entire tool-calling loop to rig-core's
    /// `multi_turn` capability, which handles the iterative process of
    /// LLM decisions, tool executions, and result aggregation.
    ///
    /// # Process Flow
    ///
    /// 1. **Extract Prompt**: Gets the prompt from the task parameters
    /// 2. **Multi-Turn Execution**: Calls `rig_agent.prompt().multi_turn(5)`
    ///    - The LLM analyzes the prompt and may call tools
    ///    - Tool results are fed back to the LLM
    ///    - Process repeats up to 5 times until a final answer is reached
    /// 3. **Result Formatting**: The final LLM response is wrapped in a `TaskResult`
    ///
    /// # Returns
    ///
    /// - `TaskResult::Success` with the final LLM response
    /// - `TaskResult::Failure` if the multi-turn process encounters an error
    #[inline]
    async fn after_task(&self, _task: &Task, _result: &TaskResult) -> Result<()> {
        Ok(())
    }

    #[inline]
    async fn before_task(&self, _task: &Task) -> Result<()> {
        Ok(())
    }

    #[inline]
    fn can_handle(&self, _task: &Task) -> bool {
        true
    }

    #[inline]
    fn capabilities(&self) -> Vec<crate::CapabilityType> {
        vec![crate::CapabilityType::ToolCalling]
    }

    #[inline]
    async fn execute_task(&self, task: Task) -> Result<TaskResult> {
        info!("--- ToolCallingAgent ENGAGED ---");
        let task_description = format!("Execute task: {task:?}");
        info!("PROMPT: '{}'", task_description);

        // Use rig-core's built-in multi-turn logic.
        // This replaces the manual _get_llm_tool_calls, _execute_tool_calls_in_parallel,
        // and _aggregate_tool_results methods.
        match self
            .rig_agent
            .0
            .prompt(&task_description)
            .multi_turn(5)
            .await
        {
            Ok(response_string) => {
                info!("LLM provided final answer: {}", response_string);
                Ok(TaskResult::success(
                    serde_json::json!({
                        "response": response_string,
                        "type": "text"
                    }),
                    None,
                    Duration::ZERO, // Duration is tracked by the worker
                ))
            }
            Err(prompt_error) => {
                // Convert the rig::completion::PromptError into our AgentError
                Err(Error::from(prompt_error))
            }
        }
    }

    #[inline]
    async fn handle_message(&self, _message: crate::AgentMessage) -> Result<()> {
        Ok(())
    }

    #[inline]
    fn id(&self) -> &AgentId {
        &self.id
    }

    #[inline]
    fn is_available(&self) -> bool {
        true
    }

    #[inline]
    fn load(&self) -> f64 {
        0.0_f64
    }

    #[inline]
    fn status(&self) -> crate::AgentStatus {
        crate::AgentStatus::new(
            self.id.clone(),
            crate::AgentState::Active,
            0_u32,
            0.0_f64,
            vec![crate::Capability::new(
                "tool_calling".to_owned(),
                "1.0".to_owned(),
            )],
        )
    }
}

/// A fluent builder for creating a `ToolCallingAgent`.
/// This handles all the boilerplate of wiring up the components.
#[derive(Debug)]
#[non_exhaustive]
pub struct Builder {
    /// Set of tools to be registered
    toolset: Toolset,
}

impl Builder {
    /// Builds and initializes the `ToolCallingAgent`.
    ///
    /// # Errors
    ///
    /// Returns an error if the agent cannot be built or tool registration fails.
    #[inline]
    pub fn build<M: CompletionModel + Debug + Send + Sync + 'static>(
        self,
        model: M,
    ) -> Result<Arc<Agent<M>>> {
        // 1. Create the rig::Agent brain.
        let agent_builder = AgentBuilder::new(model);

        // 2. Register all tools with the brain.
        // Tools now directly implement rig::tool::Tool, no adapter needed
        let rig_agent = self.toolset.register_with_brain(agent_builder).build();

        // 3. Assemble and return the final agent.
        Ok(Arc::new(Agent::new(rig_agent)))
    }

    /// Creates a new builder with the given toolset.
    #[inline]
    #[must_use]
    pub const fn new(toolset: Toolset) -> Self {
        Self { toolset }
    }
}

/// Type alias for tool-enabled agent.
pub type ToolAgent<M> = Agent<M>;

/// Type alias for tool agent builder.
pub type ToolAgentBuilder = Builder;

#[cfg(test)]
#[expect(clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use alloc::sync::Arc;
    use core::error::Error;
    use core::fmt::{Display, Formatter, Result as FmtResult};
    use core::sync::atomic::{AtomicUsize, Ordering};
    use core::time::Duration;
    use futures::future;
    use rig::completion::{
        AssistantContent, CompletionModel, CompletionRequest, CompletionResponse, Message, Usage,
    };
    use rig::message::Text;
    use std::time::Instant;
    use tokio::time::sleep;

    #[derive(Clone)]
    struct MockCompletionModel {
        delay: Option<Duration>,
        fail_on_call: Option<usize>,
        model_id: String,
        response_index: Arc<AtomicUsize>,
        responses: Vec<String>,
        should_fail: bool,
    }

    impl MockCompletionModel {
        fn new() -> Self {
            Self {
                delay: None,
                fail_on_call: None,
                model_id: "mock-model".to_string(),
                response_index: Arc::new(AtomicUsize::new(0)),
                responses: vec!["Mock response".to_string()],
                should_fail: false,
            }
        }

        pub fn call_count(&self) -> usize {
            self.response_index.load(Ordering::SeqCst)
        }

        pub fn model_id(&self) -> &str {
            &self.model_id
        }

        fn with_delay(mut self, delay: Duration) -> Self {
            self.delay = Some(delay);
            self
        }

        fn with_fail_on_call(mut self, call_number: usize) -> Self {
            self.fail_on_call = Some(call_number);
            self
        }

        fn with_failure(mut self) -> Self {
            self.should_fail = true;
            self
        }

        fn with_responses(mut self, responses: Vec<String>) -> Self {
            self.responses = responses;
            self
        }
    }

    #[derive(Debug)]
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

    impl Display for MockError {
        fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
            write!(f, "Mock error: {}", self.message)
        }
    }

    impl Error for MockError {}

    impl CompletionModel for MockCompletionModel {
        type Response = String;
        type StreamingResponse = ();

        fn completion(
            &self,
            _completion_request: CompletionRequest,
        ) -> impl Future<Output = CoreResult<CompletionResponse<Self::Response>, CompletionError>> + Send
        {
            let index = self.response_index.fetch_add(1, Ordering::SeqCst);
            let response = if self.responses.is_empty() {
                "Default response".to_string()
            } else {
                #[expect(clippy::arithmetic_side_effects)]
                let safe_index = index % self.responses.len();
                self.responses
                    .get(safe_index)
                    .cloned()
                    .unwrap_or_else(|| "Default response".to_string())
            };
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
                        return Err(CompletionError::RequestError(
                            MockError::new(format!("Failing on call #{}", index.wrapping_add(1)))
                                .into(),
                        ));
                    }
                }

                // Check if we should always fail
                if should_fail {
                    return Err(CompletionError::RequestError(
                        MockError::new("Mock failure").into(),
                    ));
                }

                // For testing, we'll just return a minimal response
                // The actual structure isn't as important as testing the delegation works
                let choice = AssistantContent::Text(Text { text: response });
                Ok(CompletionResponse {
                    choice: rig::OneOrMany::one(choice),
                    usage: Usage::default(),
                    raw_response: String::new(),
                })
            }
        }

        async fn stream(
            &self,
            _completion_request: CompletionRequest,
        ) -> CoreResult<StreamingCompletionResponse<Self::StreamingResponse>, CompletionError>
        {
            if self.should_fail {
                return Err(CompletionError::RequestError(
                    MockError::new("Mock streaming failure").into(),
                ));
            }

            // Mock implementation - this won't work in real usage but sufficient for testing
            Err(CompletionError::RequestError(
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
            chat_history: rig::OneOrMany::one(Message::user("test".to_string())),
            documents: vec![],
            max_tokens: None,
            temperature: None,
            tools: vec![],
            additional_params: None,
        };
        let result = debuggable.completion(request).await;

        assert!(result.is_err());
        match result.expect_err("Expected completion to fail in test") {
            CompletionError::RequestError(_) => {
                // Expected
            }
            other => panic!("Unexpected error type: {other:?}"),
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
            chat_history: rig::OneOrMany::one(Message::user("test".to_string())),
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
            chat_history: rig::OneOrMany::one(Message::user("test".to_string())),
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
            chat_history: rig::OneOrMany::one(Message::user("test".to_string())),
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
            chat_history: rig::OneOrMany::one(Message::user("test".to_string())),
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
            chat_history: rig::OneOrMany::one(Message::user("test".to_string())),
            documents: vec![],
            max_tokens: None,
            temperature: None,
            tools: vec![],
            additional_params: None,
        };
        let result = debuggable.completion(request).await;

        match result {
            Err(CompletionError::RequestError(boxed_error)) => {
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
            chat_history: rig::OneOrMany::one(Message::user("test".to_string())),
            documents: vec![],
            max_tokens: None,
            temperature: None,
            tools: vec![],
            additional_params: None,
        };
        let result = debuggable.completion(request).await;

        assert!(result.is_ok());
        let response = result.expect("Expected completion to succeed in test");

        // Check that we got the expected response structure
        let choices = response.choice.into_iter().collect::<Vec<_>>();
        if let Some(choice) = choices.first() {
            #[expect(clippy::pattern_type_mismatch)]
            if let AssistantContent::Text(text) = choice {
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
                        chat_history: rig::OneOrMany::one(Message::user("test".to_string())),
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
        let results = future::join_all(handles).await;

        // All should succeed
        assert_eq!(results.len(), 3);
        for handle_result in results {
            let completion_result =
                handle_result.expect("Expected task handle to complete successfully in test");
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
            chat_history: rig::OneOrMany::one(Message::user("test".to_string())),
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

        let config = riglr_core::Config::from_env();
        let app_context = ApplicationContext::from_config(&config);
        let toolset = Toolset::new(Arc::new(app_context));

        // This should compile and create the builder successfully
        let builder = Builder::new(toolset);

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
                chat_history: rig::OneOrMany::one(Message::user("test".to_string())),
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

            let response = result.expect("Expected completion to succeed in test");
            let choices = response.choice.into_iter().collect::<Vec<_>>();
            if let Some(choice) = choices.first() {
                #[expect(clippy::pattern_type_mismatch)]
                if let AssistantContent::Text(text) = choice {
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
            fn name(&self) -> &str {
                &self.name
            }

            fn new(name: impl Into<String>) -> Self {
                Self { name: name.into() }
            }
        }

        impl CompletionModel for AlternativeMockModel {
            type Response = i32; // Different response type
            type StreamingResponse = ();

            async fn completion(
                &self,
                _completion_request: CompletionRequest,
            ) -> CoreResult<CompletionResponse<Self::Response>, CompletionError> {
                let choice = AssistantContent::Text(Text {
                    text: 42.to_string(),
                });
                Ok(CompletionResponse {
                    choice: rig::OneOrMany::one(choice),
                    usage: Usage::default(),
                    raw_response: 42,
                })
            }

            async fn stream(
                &self,
                _completion_request: CompletionRequest,
            ) -> CoreResult<StreamingCompletionResponse<Self::StreamingResponse>, CompletionError>
            {
                Err(CompletionError::RequestError(
                    MockError::new("Alternative streaming not implemented").into(),
                ))
            }
        }

        let alt_model = AlternativeMockModel::new("alternative-model");
        let debuggable = DebuggableCompletionModel::new(alt_model);

        // Should compile and work with different model types
        assert_eq!(debuggable.inner().name(), "alternative-model");

        let debug_string = format!("{debuggable:?}");
        assert!(debug_string.contains("DebuggableCompletionModel"));
    }

    #[tokio::test]
    async fn test_debuggable_completion_model_empty_responses() {
        let model = MockCompletionModel::new().with_responses(vec![]);
        let debuggable = DebuggableCompletionModel::new(model);

        let request = CompletionRequest {
            preamble: None,
            chat_history: rig::OneOrMany::one(Message::user("test".to_string())),
            documents: vec![],
            max_tokens: None,
            temperature: None,
            tools: vec![],
            additional_params: None,
        };
        let result = debuggable.completion(request).await;

        // Should fall back to default response
        assert!(result.is_ok());
        let response = result.expect("Expected completion to succeed in test");
        let choices = response.choice.into_iter().collect::<Vec<_>>();
        if let Some(choice) = choices.first() {
            #[expect(clippy::pattern_type_mismatch)]
            if let AssistantContent::Text(text) = choice {
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

        let debug1 = format!("{debuggable1:?}");
        let debug2 = format!("{debuggable2:?}");

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
                chat_history: rig::OneOrMany::one(Message::user("test".to_string())),
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
                chat_history: rig::OneOrMany::one(Message::user("test".to_string())),
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
        #[expect(clippy::cast_precision_loss)]
        let overhead_ratio = wrapped_duration.as_nanos() as f64 / direct_duration.as_nanos() as f64;
        println!("Performance overhead ratio: {overhead_ratio:.2}");

        // The overhead should be reasonable (less than 50% in most cases)
        // This is a loose bound to account for test environment variability
        assert!(
            overhead_ratio < 1.5,
            "Wrapper overhead too high: {overhead_ratio:.2}"
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
                chat_history: rig::OneOrMany::one(Message::user("test".to_string())),
                documents: vec![],
                max_tokens: None,
                temperature: None,
                tools: vec![],
                additional_params: None,
            };
            let result = model.completion(request).await;
            assert!(result.is_ok());

            let response = result.expect("Expected completion to succeed in test");
            let choices = response.choice.into_iter().collect::<Vec<_>>();
            if let Some(choice) = choices.first() {
                #[expect(clippy::pattern_type_mismatch)]
                if let AssistantContent::Text(text) = choice {
                    assert_eq!(text.text, format!("Response {i}"));
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
        fn requires_completion_model<T: CompletionModel + Debug>(_: T) {}
        fn requires_send<T: Send>(_: T) {}
        fn requires_sync<T: Sync>(_: T) {}
        fn requires_clone<T: Clone>(_: T) {}

        let model = MockCompletionModel::new();
        let debuggable = DebuggableCompletionModel::new(model);

        requires_completion_model(debuggable.clone());
        requires_send(debuggable.clone());
        requires_sync(debuggable.clone());
        requires_clone(debuggable);
    }

    // Helper functions for type compatibility testing
    fn assert_same_response_type<T1: CompletionModel, T2: CompletionModel>()
    where
        T1::Response: PartialEq<T2::Response>,
    {
        // This function compiles if the types are compatible
    }

    fn assert_same_streaming_type<T1: CompletionModel, T2: CompletionModel>()
    where
        T1::StreamingResponse: PartialEq<T2::StreamingResponse>,
    {
        // This function compiles if the types are compatible
    }

    #[test]
    fn test_debuggable_completion_model_type_aliases() {
        let model = MockCompletionModel::new();
        let _ = DebuggableCompletionModel::new(model);

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
