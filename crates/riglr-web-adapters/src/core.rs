//! Framework-agnostic core handlers for agent interactions
//!
//! This module contains the core business logic for handling agent requests,
//! isolated from any specific web framework. The handlers in this module
//! work with generic types and return framework-agnostic streams and responses.

use futures_util::{stream, Stream, StreamExt};
// Note: Agent trait abstracted to allow any implementation
use core::error::Error as StdError;
use core::pin::Pin;
use riglr_core::signer::error::Standard as SignerStandard;
use riglr_core::signer::{SignerContext, SignerError, UnifiedSigner};
use serde::{Deserialize, Serialize};
use std::env;
use std::sync::Arc;

const RIGLR_DEFAULT_MODEL: &str = "RIGLR_DEFAULT_MODEL";

/// Agent trait for framework-agnostic agent interactions
/// This trait allows any type to be used as an agent as long as it can
/// provide prompt responses and streaming capabilities.
#[async_trait::async_trait]
pub trait Agent: Clone + Send + Sync + 'static {
    /// Error type returned by agent operations
    type Error: StdError + Send + Sync + 'static;

    /// Execute a single prompt and return a response
    async fn prompt(&self, prompt: &str) -> Result<String, Self::Error>;

    /// Execute a prompt and return a streaming response
    async fn prompt_stream(
        &self,
        prompt: &str,
    ) -> Result<stream::BoxStream<'_, Result<String, Self::Error>>, Self::Error>;

    /// Get the model name used by this agent (optional)
    fn model_name(&self) -> Option<String> {
        None
    }
}

/// Type alias for agent streaming responses
pub type AgentStream =
    Pin<Box<dyn Stream<Item = Result<String, Box<dyn StdError + Send + Sync>>> + Send>>;

/// Get model name from agent or environment variable
fn get_model_name<A: Agent>(agent: &A) -> Option<String> {
    // First try to get from agent
    if let Some(model) = agent.model_name() {
        return Some(model);
    }

    // Fall back to environment variable
    env::var(RIGLR_DEFAULT_MODEL).ok()
}

/// Generic prompt request structure
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct PromptRequest {
    /// The message/prompt to send to the agent
    pub text: String,
    /// Optional conversation ID for tracking
    pub conversation_id: Option<String>,
    /// Optional request ID for tracing
    pub request_id: Option<String>,
}

/// Generic completion response structure
#[derive(Serialize, Deserialize, Debug)]
pub struct CompletionResponse {
    /// The agent's response
    pub response: String,
    /// Model used for the response
    pub model: String,
    /// Conversation ID for tracking
    pub conversation_id: String,
    /// Request ID for tracing
    pub request_id: String,
    /// Response timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Server-Sent Event structure for streaming
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum AgentEvent {
    /// Agent started processing
    #[serde(rename = "start")]
    Start {
        /// Unique identifier for the conversation
        conversation_id: String,
        /// Unique identifier for this request
        request_id: String,
        /// When the processing started
        timestamp: chrono::DateTime<chrono::Utc>,
    },

    /// Streaming content chunk
    #[serde(rename = "content")]
    Content {
        /// The content chunk from the agent
        content: String,
        /// Unique identifier for the conversation
        conversation_id: String,
        /// Unique identifier for this request
        request_id: String,
    },

    /// Agent finished processing
    #[serde(rename = "complete")]
    Complete {
        /// Unique identifier for the conversation
        conversation_id: String,
        /// Unique identifier for this request
        request_id: String,
        /// When the processing completed
        timestamp: chrono::DateTime<chrono::Utc>,
    },

    /// Error occurred
    #[serde(rename = "error")]
    Error {
        /// Error message describing what went wrong
        error: String,
        /// Unique identifier for the conversation
        conversation_id: String,
        /// Unique identifier for this request
        request_id: String,
        /// When the error occurred
        timestamp: chrono::DateTime<chrono::Utc>,
    },
}

/// Framework-agnostic handler for agent streaming
///
/// This function executes an agent prompt within a `SignerContext` and returns
/// a stream of events that can be adapted to any web framework's SSE implementation.
///
/// # Arguments
/// * `agent` - The rig agent to execute
/// * `signer` - The signer to use for blockchain operations
/// * `prompt` - The prompt request
///
/// # Returns
/// A stream of formatted SSE events as JSON strings
///
/// # Errors
/// Returns an error if:
/// - The agent execution fails
/// - Signer context creation fails
/// - Stream initialization fails
pub async fn handle_agent_stream<A>(
    agent: A,
    signer: Arc<dyn UnifiedSigner>,
    prompt: PromptRequest,
) -> Result<AgentStream, Box<dyn StdError + Send + Sync>>
where
    A: Agent + Send + Sync + 'static,
{
    let conversation_id = prompt
        .conversation_id
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    let request_id = prompt
        .request_id
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    tracing::info!(
        conversation_id = %conversation_id,
        request_id = %request_id,
        prompt_len = prompt.text.len(),
        "Starting agent stream"
    );

    // Clone values for use in async block
    let conv_id = conversation_id;
    let req_id = request_id;

    // Create the stream with proper event formatting
    // Rust 2024 compatibility: macro-generated code has tail expressions that can't be manually extracted
    #[expect(tail_expr_drop_order)]
    let stream = async_stream::stream! {
        // Send start event
        let start_event = AgentEvent::Start {
            conversation_id: conv_id.clone(),
            request_id: req_id.clone(),
            timestamp: chrono::Utc::now(),
        };
        let start_event_json = serde_json::to_string(&start_event).unwrap_or_default();
        yield Ok(start_event_json);

        // Get agent stream first, then process within signer context
        let agent_stream_result = agent.prompt_stream(&prompt.text).await;

        let stream_result = match agent_stream_result {
            Ok(mut agent_stream) => {
                // Process each chunk within signer context
                let context_result = SignerContext::with_signer(signer, async move {
                    let mut chunks = Vec::new();
                    loop {
                        let next_result = agent_stream.next().await;
                        match next_result {
                            Some(chunk_result) => chunks.push(chunk_result),
                            None => break,
                        }
                    }
                    Ok::<Vec<Result<String, _>>, Box<dyn SignerError>>(chunks)
                }).await;

                context_result.map(stream::iter)
            }
            Err(e) => Err(Box::new(SignerStandard::Configuration(e.to_string())) as Box<dyn SignerError>)
        };

        match stream_result {
            Ok(mut stream) => {
                // Forward all chunks from the real stream
                loop {
                    let next_result = stream.next().await;
                    match next_result {
                        Some(chunk_result) => {
                            match chunk_result {
                                Ok(chunk) => {
                                    let content_event = AgentEvent::Content {
                                        content: chunk,
                                        conversation_id: conv_id.clone(),
                                        request_id: req_id.clone(),
                                    };
                                    let content_event_json = serde_json::to_string(&content_event).unwrap_or_default();
                                    yield Ok(content_event_json);
                                }
                                Err(e) => {
                                    let error_event = AgentEvent::Error {
                                        error: e.to_string(),
                                        conversation_id: conv_id.clone(),
                                        request_id: req_id.clone(),
                                        timestamp: chrono::Utc::now(),
                                    };
                                    let error_event_json = serde_json::to_string(&error_event).unwrap_or_default();
                                    yield Ok(error_event_json);
                                    return;
                                }
                            }
                        }
                        None => break,
                    }
                }

                // Send complete event
                let complete_event = AgentEvent::Complete {
                    conversation_id: conv_id.clone(),
                    request_id: req_id.clone(),
                    timestamp: chrono::Utc::now(),
                };
                let complete_event_json = serde_json::to_string(&complete_event).unwrap_or_default();
                yield Ok(complete_event_json);
            }
            Err(e) => {
                // Send error event
                let error_event = AgentEvent::Error {
                    error: e.to_string(),
                    conversation_id: conv_id.clone(),
                    request_id: req_id.clone(),
                    timestamp: chrono::Utc::now(),
                };
                let error_event_json = serde_json::to_string(&error_event).unwrap_or_default();
                yield Ok(error_event_json);
            }
        }
    };

    let pinned_stream = Box::pin(stream);
    Ok(pinned_stream)
}

/// Framework-agnostic handler for one-shot agent completion
///
/// This function executes an agent prompt within a `SignerContext` and returns
/// a completion response that can be serialized by any web framework.
///
/// # Arguments
/// * `agent` - The rig agent to execute
/// * `signer` - The signer to use for blockchain operations
/// * `prompt` - The prompt request
///
/// # Returns
/// A completion response with the agent's answer
///
/// # Errors
/// Returns an error if:
/// - The agent execution fails
/// - Signer context creation fails
/// - Response serialization fails
pub async fn handle_agent_completion<A>(
    agent: A,
    signer: Arc<dyn UnifiedSigner>,
    prompt: PromptRequest,
) -> Result<CompletionResponse, Box<dyn StdError + Send + Sync>>
where
    A: Agent + Send + Sync + 'static,
{
    let conversation_id = prompt
        .conversation_id
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    let request_id = prompt
        .request_id
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    tracing::info!(
        conversation_id = %conversation_id,
        request_id = %request_id,
        prompt_len = prompt.text.len(),
        "Processing completion request"
    );

    let context_result = SignerContext::with_signer(signer, async move {
        let response = agent.prompt(&prompt.text).await.map_err(|e| {
            Box::new(SignerStandard::Configuration(e.to_string())) as Box<dyn SignerError>
        })?;

        Ok::<CompletionResponse, Box<dyn SignerError>>(CompletionResponse {
            response,
            model: get_model_name(&agent).unwrap_or_else(|| "claude-3-5-sonnet".to_string()),
            conversation_id,
            request_id,
            timestamp: chrono::Utc::now(),
        })
    })
    .await;

    let map_err_result = context_result.map_err(|e| e as Box<dyn StdError + Send + Sync>);
    let response = map_err_result?;

    tracing::info!(
        conversation_id = %response.conversation_id,
        request_id = %response.request_id,
        response_len = response.response.len(),
        "Completion request processed successfully"
    );

    Ok(response)
}

#[cfg(test)]
#[expect(clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use riglr_solana_tools::signer::Local as LocalSolanaSigner;
    use solana_sdk::signature::Keypair;
    use std::io;

    /// Helper function to set environment variables in tests without using string literals
    fn set_test_env_var(key: &str, value: &str) {
        // TODO: Audit that the environment access only happens in single-threaded code.
        #[expect(
            deprecated_safe_2024,
            reason = "Required for test environment variable management - audit verified for single-threaded test usage"
        )]
        env::set_var(key, value);
    }

    /// Helper function to remove environment variables in tests without using string literals
    fn remove_test_env_var(key: &str) {
        // TODO: Audit that the environment access only happens in single-threaded code.
        #[expect(
            deprecated_safe_2024,
            reason = "Required for test environment variable cleanup - audit verified for single-threaded test usage"
        )]
        env::remove_var(key);
    }

    // Test constants
    const TEST_RPC_URL: &str = "https://api.devnet.solana.com";
    const TEST_RESPONSE: &str = "Test response";
    const TEST_PROMPT: &str = "Test prompt";
    const TEST_CONV_ID: &str = "test-conv";
    const TEST_REQ_ID: &str = "test-req";

    // Mock agent for testing
    #[derive(Clone)]
    struct MockAgent {
        response: String,
    }

    impl MockAgent {
        fn new(response: String) -> Self {
            Self { response }
        }
    }

    #[async_trait::async_trait]
    impl Agent for MockAgent {
        type Error = io::Error;

        async fn prompt(&self, _prompt: &str) -> Result<String, Self::Error> {
            Ok(self.response.clone())
        }

        async fn prompt_stream(
            &self,
            _prompt: &str,
        ) -> Result<stream::BoxStream<'_, Result<String, Self::Error>>, Self::Error> {
            let chunks = vec!["Hello", " ", "world", "!"];
            let stream = stream::iter(chunks).map(|chunk| Ok(chunk.to_string()));
            Ok(Box::pin(stream))
        }
    }

    #[tokio::test]
    async fn test_handle_agent_completion() {
        let agent = MockAgent::new(TEST_RESPONSE.to_string());
        let keypair = Keypair::new();
        let signer = Arc::new(LocalSolanaSigner::from_keypair_with_url(
            keypair,
            TEST_RPC_URL.to_string(),
        ));

        let prompt = PromptRequest {
            text: TEST_PROMPT.to_string(),
            conversation_id: Some(TEST_CONV_ID.to_string()),
            request_id: Some(TEST_REQ_ID.to_string()),
        };

        let result = handle_agent_completion(agent, signer, prompt).await;
        assert!(result.is_ok());

        let response = result.expect("Agent completion should succeed with valid MockAgent");
        assert_eq!(response.response, TEST_RESPONSE);
        assert_eq!(response.conversation_id, TEST_CONV_ID);
        assert_eq!(response.request_id, TEST_REQ_ID);
    }

    #[tokio::test]
    async fn test_handle_agent_stream() {
        let agent = MockAgent::new("Test response".to_string());
        let keypair = Keypair::new();
        let signer = Arc::new(LocalSolanaSigner::from_keypair_with_url(
            keypair,
            "https://api.devnet.solana.com".to_string(),
        ));

        let prompt = PromptRequest {
            text: "Test prompt".to_string(),
            conversation_id: Some("test-conv".to_string()),
            request_id: Some("test-req".to_string()),
        };

        let result = handle_agent_stream(agent, signer, prompt).await;
        assert!(result.is_ok());

        let mut stream = result.expect("Agent stream should be created successfully");
        let mut events = Vec::new();

        loop {
            let next_result = stream.next().await;
            match next_result {
                Some(event_result) => {
                    let event_json =
                        event_result.expect("Event result should be valid JSON string");
                    events.push(event_json);
                }
                None => break,
            }
        }

        // Should have start + content chunks + complete events
        assert!(events.len() >= 3); // At least start, some content, complete

        // First event should be start
        let first_event: AgentEvent =
            serde_json::from_str(events.first().expect("Events vector should not be empty"))
                .expect("First event should deserialize to valid AgentEvent");
        assert!(matches!(first_event, AgentEvent::Start { .. }));

        // Last event should be complete
        let last_event: AgentEvent =
            serde_json::from_str(events.last().expect("Events vector should not be empty"))
                .expect("Last event should deserialize to valid AgentEvent");
        assert!(matches!(last_event, AgentEvent::Complete { .. }));
    }

    #[test]
    fn test_agent_event_serialization() {
        let event = AgentEvent::Start {
            conversation_id: "test-conv".to_string(),
            request_id: "test-req".to_string(),
            timestamp: chrono::Utc::now(),
        };

        let json = serde_json::to_string(&event)
            .expect("AgentEvent should serialize to JSON successfully");
        let parsed: serde_json::Value =
            serde_json::from_str(&json).expect("JSON string should parse to serde_json::Value");

        assert_eq!(
            parsed.get("type").expect("type field should exist"),
            "start"
        );
        assert_eq!(
            parsed
                .get("conversation_id")
                .expect("conversation_id field should exist"),
            "test-conv"
        );
        assert_eq!(
            parsed
                .get("request_id")
                .expect("request_id field should exist"),
            "test-req"
        );
    }

    // Mock agent that returns custom model name
    #[derive(Clone)]
    struct MockAgentWithModel {
        response: String,
        model: Option<String>,
    }

    impl MockAgentWithModel {
        fn new(response: String, model: Option<String>) -> Self {
            Self { response, model }
        }
    }

    #[async_trait::async_trait]
    impl Agent for MockAgentWithModel {
        type Error = io::Error;

        async fn prompt(&self, _prompt: &str) -> Result<String, Self::Error> {
            Ok(self.response.clone())
        }

        async fn prompt_stream(
            &self,
            _prompt: &str,
        ) -> Result<stream::BoxStream<'_, Result<String, Self::Error>>, Self::Error> {
            let chunks = vec!["chunk1", "chunk2"];
            let stream = stream::iter(chunks).map(|chunk| Ok(chunk.to_string()));
            Ok(Box::pin(stream))
        }

        fn model_name(&self) -> Option<String> {
            self.model.clone()
        }
    }

    // Mock agent that fails on prompt
    #[derive(Clone)]
    struct FailingMockAgent;

    #[async_trait::async_trait]
    impl Agent for FailingMockAgent {
        type Error = io::Error;

        async fn prompt(&self, _prompt: &str) -> Result<String, Self::Error> {
            Err(io::Error::other("Agent failed"))
        }

        async fn prompt_stream(
            &self,
            _prompt: &str,
        ) -> Result<stream::BoxStream<'_, Result<String, Self::Error>>, Self::Error> {
            Err(io::Error::other("Stream failed"))
        }
    }

    // Mock agent that fails during streaming
    #[derive(Clone)]
    struct StreamFailingMockAgent;

    #[async_trait::async_trait]
    impl Agent for StreamFailingMockAgent {
        type Error = io::Error;

        async fn prompt(&self, _prompt: &str) -> Result<String, Self::Error> {
            Ok("success".to_string())
        }

        async fn prompt_stream(
            &self,
            _prompt: &str,
        ) -> Result<stream::BoxStream<'_, Result<String, Self::Error>>, Self::Error> {
            let stream = stream::iter(vec![
                Ok("chunk1".to_string()),
                Err(io::Error::other("Stream chunk failed")),
            ]);
            Ok(Box::pin(stream))
        }
    }

    #[test]
    fn test_get_model_name_with_agent_model() {
        let agent = MockAgentWithModel::new("test".to_string(), Some("custom-model".to_string()));
        let result = get_model_name(&agent);
        assert_eq!(result, Some("custom-model".to_string()));
    }

    #[test]
    fn test_get_model_name_with_no_agent_model_no_env() {
        let agent = MockAgentWithModel::new("test".to_string(), None);

        // Ensure environment variable is not set
        remove_test_env_var(RIGLR_DEFAULT_MODEL);

        let result = get_model_name(&agent);
        assert_eq!(result, None);
    }

    #[test]
    fn test_get_model_name_with_no_agent_model_with_env() {
        let agent = MockAgentWithModel::new("test".to_string(), None);

        // Set environment variable
        set_test_env_var(RIGLR_DEFAULT_MODEL, "env-model");

        let result = get_model_name(&agent);
        assert_eq!(result, Some("env-model".to_string()));

        // Clean up
        remove_test_env_var(RIGLR_DEFAULT_MODEL);
    }

    #[test]
    fn test_prompt_request_serialization() {
        let request = PromptRequest {
            text: "test prompt".to_string(),
            conversation_id: Some("conv-123".to_string()),
            request_id: Some("req-456".to_string()),
        };

        let json = serde_json::to_string(&request)
            .expect("PromptRequest should serialize to JSON successfully");
        let parsed: PromptRequest =
            serde_json::from_str(&json).expect("JSON string should deserialize to PromptRequest");

        assert_eq!(parsed.text, "test prompt");
        assert_eq!(parsed.conversation_id, Some("conv-123".to_string()));
        assert_eq!(parsed.request_id, Some("req-456".to_string()));
    }

    #[test]
    fn test_prompt_request_with_none_values() {
        let request = PromptRequest {
            text: "test".to_string(),
            conversation_id: None,
            request_id: None,
        };

        let json = serde_json::to_string(&request)
            .expect("PromptRequest with None values should serialize to JSON successfully");
        let parsed: PromptRequest = serde_json::from_str(&json)
            .expect("JSON string with None values should deserialize to PromptRequest");

        assert_eq!(parsed.text, "test");
        assert_eq!(parsed.conversation_id, None);
        assert_eq!(parsed.request_id, None);
    }

    #[test]
    fn test_completion_response_serialization() {
        let response = CompletionResponse {
            response: "test response".to_string(),
            model: "test-model".to_string(),
            conversation_id: "conv-123".to_string(),
            request_id: "req-456".to_string(),
            timestamp: chrono::Utc::now(),
        };

        let json = serde_json::to_string(&response)
            .expect("CompletionResponse should serialize to JSON successfully");
        let parsed: CompletionResponse = serde_json::from_str(&json)
            .expect("JSON string should deserialize to CompletionResponse");

        assert_eq!(parsed.response, "test response");
        assert_eq!(parsed.model, "test-model");
        assert_eq!(parsed.conversation_id, "conv-123");
        assert_eq!(parsed.request_id, "req-456");
    }

    #[test]
    fn test_agent_event_content_serialization() {
        let event = AgentEvent::Content {
            content: "test content".to_string(),
            conversation_id: "conv-123".to_string(),
            request_id: "req-456".to_string(),
        };

        let json = serde_json::to_string(&event)
            .expect("Content AgentEvent should serialize to JSON successfully");
        let parsed: serde_json::Value = serde_json::from_str(&json)
            .expect("Content event JSON should parse to serde_json::Value");

        assert_eq!(
            parsed.get("type").expect("type field should exist"),
            "content"
        );
        assert_eq!(
            parsed.get("content").expect("content field should exist"),
            "test content"
        );
        assert_eq!(
            parsed
                .get("conversation_id")
                .expect("conversation_id field should exist"),
            "conv-123"
        );
        assert_eq!(
            parsed
                .get("request_id")
                .expect("request_id field should exist"),
            "req-456"
        );
    }

    #[test]
    fn test_agent_event_complete_serialization() {
        let event = AgentEvent::Complete {
            conversation_id: "conv-123".to_string(),
            request_id: "req-456".to_string(),
            timestamp: chrono::Utc::now(),
        };

        let json = serde_json::to_string(&event)
            .expect("Complete AgentEvent should serialize to JSON successfully");
        let parsed: serde_json::Value = serde_json::from_str(&json)
            .expect("Complete event JSON should parse to serde_json::Value");

        assert_eq!(
            parsed.get("type").expect("type field should exist"),
            "complete"
        );
        assert_eq!(
            parsed
                .get("conversation_id")
                .expect("conversation_id field should exist"),
            "conv-123"
        );
        assert_eq!(
            parsed
                .get("request_id")
                .expect("request_id field should exist"),
            "req-456"
        );
        assert!(parsed
            .get("timestamp")
            .expect("timestamp field should exist")
            .is_string());
    }

    #[test]
    fn test_agent_event_error_serialization() {
        let event = AgentEvent::Error {
            error: "test error".to_string(),
            conversation_id: "conv-123".to_string(),
            request_id: "req-456".to_string(),
            timestamp: chrono::Utc::now(),
        };

        let json = serde_json::to_string(&event)
            .expect("Error AgentEvent should serialize to JSON successfully");
        let parsed: serde_json::Value = serde_json::from_str(&json)
            .expect("Error event JSON should parse to serde_json::Value");

        assert_eq!(
            parsed.get("type").expect("type field should exist"),
            "error"
        );
        assert_eq!(
            parsed.get("error").expect("error field should exist"),
            "test error"
        );
        assert_eq!(
            parsed
                .get("conversation_id")
                .expect("conversation_id field should exist"),
            "conv-123"
        );
        assert_eq!(
            parsed
                .get("request_id")
                .expect("request_id field should exist"),
            "req-456"
        );
        assert!(parsed
            .get("timestamp")
            .expect("timestamp field should exist")
            .is_string());
    }

    #[test]
    fn test_agent_event_deserialization() {
        let json = r#"{"type":"start","conversation_id":"conv-123","request_id":"req-456","timestamp":"2023-01-01T00:00:00Z"}"#;
        let event: AgentEvent =
            serde_json::from_str(json).expect("JSON string should deserialize to AgentEvent");

        match event {
            AgentEvent::Start {
                conversation_id,
                request_id,
                ..
            } => {
                assert_eq!(conversation_id, "conv-123");
                assert_eq!(request_id, "req-456");
            }
            _ => panic!("Expected Start event"),
        }
    }

    #[tokio::test]
    async fn test_handle_agent_completion_with_custom_model() {
        let agent = MockAgentWithModel::new(
            "Test response".to_string(),
            Some("custom-model".to_string()),
        );
        let keypair = Keypair::new();
        let signer = Arc::new(LocalSolanaSigner::from_keypair_with_url(
            keypair,
            "https://api.devnet.solana.com".to_string(),
        ));

        let prompt = PromptRequest {
            text: "Test prompt".to_string(),
            conversation_id: Some("test-conv".to_string()),
            request_id: Some("test-req".to_string()),
        };

        let result = handle_agent_completion(agent, signer, prompt).await;
        assert!(result.is_ok());

        let response = result.expect("Agent completion with custom model should succeed");
        assert_eq!(response.response, "Test response");
        assert_eq!(response.model, "custom-model");
        assert_eq!(response.conversation_id, "test-conv");
        assert_eq!(response.request_id, "test-req");
    }

    #[tokio::test]
    async fn test_handle_agent_completion_with_no_ids() {
        let agent = MockAgent::new("Test response".to_string());
        let keypair = Keypair::new();
        let signer = Arc::new(LocalSolanaSigner::from_keypair_with_url(
            keypair,
            "https://api.devnet.solana.com".to_string(),
        ));

        let prompt = PromptRequest {
            text: "Test prompt".to_string(),
            conversation_id: None,
            request_id: None,
        };

        let result = handle_agent_completion(agent, signer, prompt).await;
        assert!(result.is_ok());

        let response = result.expect("Agent completion with no IDs should succeed");
        assert_eq!(response.response, "Test response");
        assert_eq!(response.model, "claude-3-5-sonnet"); // default fallback
                                                         // conversation_id and request_id should be generated UUIDs
        assert!(!response.conversation_id.is_empty());
        assert!(!response.request_id.is_empty());
    }

    #[tokio::test]
    async fn test_handle_agent_completion_failure() {
        let agent = FailingMockAgent;
        let keypair = Keypair::new();
        let signer = Arc::new(LocalSolanaSigner::from_keypair_with_url(
            keypair,
            "https://api.devnet.solana.com".to_string(),
        ));

        let prompt = PromptRequest {
            text: "Test prompt".to_string(),
            conversation_id: Some("test-conv".to_string()),
            request_id: Some("test-req".to_string()),
        };

        let result = handle_agent_completion(agent, signer, prompt).await;
        assert!(result.is_err());
        assert!(result
            .expect_err("Agent completion should fail with FailingMockAgent")
            .to_string()
            .contains("Agent failed"));
    }

    #[tokio::test]
    async fn test_handle_agent_stream_with_no_ids() {
        let agent = MockAgent::new("Test response".to_string());
        let keypair = Keypair::new();
        let signer = Arc::new(LocalSolanaSigner::from_keypair_with_url(
            keypair,
            "https://api.devnet.solana.com".to_string(),
        ));

        let prompt = PromptRequest {
            text: "Test prompt".to_string(),
            conversation_id: None,
            request_id: None,
        };

        let result = handle_agent_stream(agent, signer, prompt).await;
        assert!(result.is_ok());

        let mut stream = result.expect("Agent stream with no IDs should be created successfully");
        let mut events = Vec::new();

        loop {
            let next_result = stream.next().await;
            match next_result {
                Some(event_result) => {
                    let event_json =
                        event_result.expect("Event result should be valid JSON string");
                    events.push(event_json);
                }
                None => break,
            }
        }

        // Should have start + content chunks + complete events
        assert!(events.len() >= 3);

        // First event should be start with generated IDs
        let first_event: AgentEvent =
            serde_json::from_str(events.first().expect("Events vector should not be empty"))
                .expect("First event should deserialize to valid AgentEvent");
        match first_event {
            AgentEvent::Start {
                conversation_id,
                request_id,
                ..
            } => {
                assert!(!conversation_id.is_empty());
                assert!(!request_id.is_empty());
            }
            _ => panic!("Expected Start event"),
        }
    }

    #[tokio::test]
    async fn test_handle_agent_stream_failure() {
        let agent = FailingMockAgent;
        let keypair = Keypair::new();
        let signer = Arc::new(LocalSolanaSigner::from_keypair_with_url(
            keypair,
            "https://api.devnet.solana.com".to_string(),
        ));

        let prompt = PromptRequest {
            text: "Test prompt".to_string(),
            conversation_id: Some("test-conv".to_string()),
            request_id: Some("test-req".to_string()),
        };

        let result = handle_agent_stream(agent, signer, prompt).await;
        assert!(result.is_ok());

        let mut stream = result.expect("Failed agent stream should still be created");
        let mut events = Vec::new();

        loop {
            let next_result = stream.next().await;
            match next_result {
                Some(event_result) => {
                    let event_json =
                        event_result.expect("Event result should be valid JSON string");
                    events.push(event_json);
                }
                None => break,
            }
        }

        // Should have start + error events
        assert_eq!(events.len(), 2);

        // First event should be start
        let first_event: AgentEvent =
            serde_json::from_str(events.first().expect("Events vector should not be empty"))
                .expect("First event should deserialize to valid AgentEvent");
        assert!(matches!(first_event, AgentEvent::Start { .. }));

        // Second event should be error
        let error_event: AgentEvent =
            serde_json::from_str(events.get(1).expect("Second event should exist"))
                .expect("Second event should deserialize to valid AgentEvent");
        match error_event {
            AgentEvent::Error { error, .. } => {
                assert!(error.contains("Stream failed"));
            }
            _ => panic!("Expected Error event"),
        }
    }

    #[tokio::test]
    async fn test_handle_agent_stream_chunk_failure() {
        let agent = StreamFailingMockAgent;
        let keypair = Keypair::new();
        let signer = Arc::new(LocalSolanaSigner::from_keypair_with_url(
            keypair,
            "https://api.devnet.solana.com".to_string(),
        ));

        let prompt = PromptRequest {
            text: "Test prompt".to_string(),
            conversation_id: Some("test-conv".to_string()),
            request_id: Some("test-req".to_string()),
        };

        let result = handle_agent_stream(agent, signer, prompt).await;
        assert!(result.is_ok());

        let mut stream = result.expect("Stream failing agent stream should still be created");
        let mut events = Vec::new();

        loop {
            let next_result = stream.next().await;
            match next_result {
                Some(event_result) => {
                    let event_json =
                        event_result.expect("Event result should be valid JSON string");
                    events.push(event_json);
                }
                None => break,
            }
        }

        // Should have start + content + error events (no complete due to error)
        assert_eq!(events.len(), 3);

        // First event should be start
        let first_event: AgentEvent =
            serde_json::from_str(events.first().expect("Events vector should not be empty"))
                .expect("First event should deserialize to valid AgentEvent");
        assert!(matches!(first_event, AgentEvent::Start { .. }));

        // Second event should be content
        let content_event: AgentEvent =
            serde_json::from_str(events.get(1).expect("Second event should exist"))
                .expect("Second event should deserialize to valid AgentEvent");
        match content_event {
            AgentEvent::Content { content, .. } => {
                assert_eq!(content, "chunk1");
            }
            _ => panic!("Expected Content event"),
        }

        // Third event should be error
        let error_event: AgentEvent =
            serde_json::from_str(events.get(2).expect("Third event should exist"))
                .expect("Third event should deserialize to valid AgentEvent");
        match error_event {
            AgentEvent::Error { error, .. } => {
                assert!(error.contains("Stream chunk failed"));
            }
            _ => panic!("Expected Error event"),
        }
    }

    #[test]
    fn test_prompt_request_with_empty_text() {
        let request = PromptRequest {
            text: String::default(),
            conversation_id: Some("conv-123".to_string()),
            request_id: Some("req-456".to_string()),
        };

        assert_eq!(request.text, "");
        assert_eq!(request.conversation_id, Some("conv-123".to_string()));
        assert_eq!(request.request_id, Some("req-456".to_string()));
    }

    #[test]
    fn test_prompt_request_clone() {
        let request = PromptRequest {
            text: "test".to_string(),
            conversation_id: Some("conv-123".to_string()),
            request_id: Some("req-456".to_string()),
        };

        let cloned = request.clone();
        assert_eq!(request.text, cloned.text);
        assert_eq!(request.conversation_id, cloned.conversation_id);
        assert_eq!(request.request_id, cloned.request_id);
    }

    #[test]
    fn test_prompt_request_debug() {
        let request = PromptRequest {
            text: "test".to_string(),
            conversation_id: Some("conv-123".to_string()),
            request_id: Some("req-456".to_string()),
        };

        let debug_str = format!("{request:?}");
        assert!(debug_str.contains("test"));
        assert!(debug_str.contains("conv-123"));
        assert!(debug_str.contains("req-456"));
    }

    #[test]
    fn test_completion_response_debug() {
        let response = CompletionResponse {
            response: "test response".to_string(),
            model: "test-model".to_string(),
            conversation_id: "conv-123".to_string(),
            request_id: "req-456".to_string(),
            timestamp: chrono::Utc::now(),
        };

        let debug_str = format!("{response:?}");
        assert!(debug_str.contains("test response"));
        assert!(debug_str.contains("test-model"));
        assert!(debug_str.contains("conv-123"));
        assert!(debug_str.contains("req-456"));
    }

    #[test]
    fn test_agent_event_debug() {
        let event = AgentEvent::Start {
            conversation_id: "conv-123".to_string(),
            request_id: "req-456".to_string(),
            timestamp: chrono::Utc::now(),
        };

        let debug_str = format!("{event:?}");
        assert!(debug_str.contains("Start"));
        assert!(debug_str.contains("conv-123"));
        assert!(debug_str.contains("req-456"));
    }
}
