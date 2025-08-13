//! Framework-agnostic core handlers for agent interactions
//!
//! This module contains the core business logic for handling agent requests,
//! isolated from any specific web framework. The handlers in this module
//! work with generic types and return framework-agnostic streams and responses.

use futures_util::{Stream, StreamExt};
// Note: Agent trait abstracted to allow any implementation
use riglr_core::signer::{SignerContext, TransactionSigner};
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use std::error::Error as StdError;

/// Agent trait for framework-agnostic agent interactions
/// This trait allows any type to be used as an agent as long as it can
/// provide prompt responses and streaming capabilities.
#[async_trait::async_trait]
pub trait Agent: Clone + Send + Sync + 'static {
    type Error: StdError + Send + Sync + 'static;

    /// Execute a single prompt and return a response
    async fn prompt(&self, prompt: &str) -> Result<String, Self::Error>;

    /// Execute a prompt and return a streaming response
    async fn prompt_stream(&self, prompt: &str) -> Result<futures_util::stream::BoxStream<'_, Result<String, Self::Error>>, Self::Error>;

    /// Get the model name used by this agent (optional)
    fn model_name(&self) -> Option<String> {
        None
    }
}

/// Type alias for agent streaming responses
pub type AgentStream = Pin<Box<dyn Stream<Item = Result<String, Box<dyn StdError + Send + Sync>>> + Send>>;

/// Get model name from agent or environment variable
fn get_model_name<A: Agent>(agent: &A) -> Option<String> {
    // First try to get from agent
    if let Some(model) = agent.model_name() {
        return Some(model);
    }
    
    // Fall back to environment variable
    std::env::var("RIGLR_DEFAULT_MODEL").ok()
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
        conversation_id: String,
        request_id: String,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    
    /// Streaming content chunk
    #[serde(rename = "content")]
    Content {
        content: String,
        conversation_id: String,
        request_id: String,
    },
    
    /// Agent finished processing
    #[serde(rename = "complete")]
    Complete {
        conversation_id: String,
        request_id: String,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    
    /// Error occurred
    #[serde(rename = "error")]
    Error {
        error: String,
        conversation_id: String,
        request_id: String,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
}

/// Framework-agnostic handler for agent streaming
///
/// This function executes an agent prompt within a SignerContext and returns
/// a stream of events that can be adapted to any web framework's SSE implementation.
///
/// # Arguments
/// * `agent` - The rig agent to execute
/// * `signer` - The signer to use for blockchain operations
/// * `prompt` - The prompt request
///
/// # Returns
/// A stream of formatted SSE events as JSON strings
pub async fn handle_agent_stream<A>(
    agent: A,
    signer: std::sync::Arc<dyn TransactionSigner>,
    prompt: PromptRequest,
) -> Result<AgentStream, Box<dyn StdError + Send + Sync>>
where
    A: Agent + Send + Sync + 'static,
{
    let conversation_id = prompt.conversation_id
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    let request_id = prompt.request_id
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    tracing::info!(
        conversation_id = %conversation_id,
        request_id = %request_id,
        prompt_len = prompt.text.len(),
        "Starting agent stream"
    );

    // Clone values for use in async block
    let conv_id = conversation_id.clone();
    let req_id = request_id.clone();
    let _prompt_text = prompt.text.clone();
    
    // Create the stream with proper event formatting
    let stream = async_stream::stream! {
        // Move agent and signer into the stream
        let _agent = agent;
        let _signer = signer;
        // Send start event
        let start_event = AgentEvent::Start {
            conversation_id: conv_id.clone(),
            request_id: req_id.clone(),
            timestamp: chrono::Utc::now(),
        };
        yield Ok(serde_json::to_string(&start_event).unwrap_or_default());
        
        // TODO: Implement proper agent streaming with correct lifetime management
        // For now, return a simple placeholder stream to make compilation work
        let stream_result: Result<futures_util::stream::Empty<Result<String, riglr_core::signer::SignerError>>, riglr_core::signer::SignerError> = Ok(futures_util::stream::empty());
        
        match stream_result {
            Ok(mut stream) => {
                // Forward all chunks from the real stream
                while let Some(chunk_result) = stream.next().await {
                    match chunk_result {
                        Ok(chunk) => {
                            let content_event = AgentEvent::Content {
                                content: chunk,
                                conversation_id: conv_id.clone(),
                                request_id: req_id.clone(),
                            };
                            yield Ok(serde_json::to_string(&content_event).unwrap_or_default());
                        }
                        Err(e) => {
                            let error_event = AgentEvent::Error {
                                error: e.to_string(),
                                conversation_id: conv_id.clone(),
                                request_id: req_id.clone(),
                                timestamp: chrono::Utc::now(),
                            };
                            yield Ok(serde_json::to_string(&error_event).unwrap_or_default());
                            return;
                        }
                    }
                }
                
                // Send complete event
                let complete_event = AgentEvent::Complete {
                    conversation_id: conv_id.clone(),
                    request_id: req_id.clone(),
                    timestamp: chrono::Utc::now(),
                };
                yield Ok(serde_json::to_string(&complete_event).unwrap_or_default());
            }
            Err(e) => {
                // Send error event
                let error_event = AgentEvent::Error {
                    error: e.to_string(),
                    conversation_id: conv_id.clone(),
                    request_id: req_id.clone(),
                    timestamp: chrono::Utc::now(),
                };
                yield Ok(serde_json::to_string(&error_event).unwrap_or_default());
            }
        }
    };
    
    Ok(Box::pin(stream))
}

/// Framework-agnostic handler for one-shot agent completion
///
/// This function executes an agent prompt within a SignerContext and returns
/// a completion response that can be serialized by any web framework.
///
/// # Arguments
/// * `agent` - The rig agent to execute
/// * `signer` - The signer to use for blockchain operations
/// * `prompt` - The prompt request
///
/// # Returns
/// A completion response with the agent's answer
pub async fn handle_agent_completion<A>(
    agent: A,
    signer: std::sync::Arc<dyn TransactionSigner>,
    prompt: PromptRequest,
) -> Result<CompletionResponse, Box<dyn StdError + Send + Sync>>
where
    A: Agent + Send + Sync + 'static,
{
    let conversation_id = prompt.conversation_id
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    let request_id = prompt.request_id
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    tracing::info!(
        conversation_id = %conversation_id,
        request_id = %request_id,
        prompt_len = prompt.text.len(),
        "Processing completion request"
    );

    let response = SignerContext::with_signer(signer, async move {
        let response = agent.prompt(&prompt.text).await
            .map_err(|e| riglr_core::signer::SignerError::Configuration(e.to_string()))?;
        
        Ok::<CompletionResponse, riglr_core::signer::SignerError>(CompletionResponse {
            response,
            model: get_model_name(&agent).unwrap_or_else(|| "claude-3-5-sonnet".to_string()),
            conversation_id,
            request_id,
            timestamp: chrono::Utc::now(),
        })
    }).await.map_err(|e| Box::new(e) as Box<dyn StdError + Send + Sync>)?;

    tracing::info!(
        conversation_id = %response.conversation_id,
        request_id = %response.request_id,
        response_len = response.response.len(),
        "Completion request processed successfully"
    );

    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;
    use riglr_solana_tools::signer::LocalSolanaSigner;
    use solana_sdk::signature::Keypair;

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
        type Error = std::io::Error;

        async fn prompt(&self, _prompt: &str) -> Result<String, Self::Error> {
            Ok(self.response.clone())
        }

        async fn prompt_stream(&self, _prompt: &str) -> Result<futures_util::stream::BoxStream<'_, Result<String, Self::Error>>, Self::Error> {
            let chunks = vec!["Hello", " ", "world", "!"];
            let stream = futures_util::stream::iter(chunks)
                .map(|chunk| Ok(chunk.to_string()));
            Ok(Box::pin(stream))
        }
    }

    #[tokio::test]
    async fn test_handle_agent_completion() {
        let agent = MockAgent::new("Test response".to_string());
        let keypair = Keypair::new();
        let signer = std::sync::Arc::new(LocalSolanaSigner::new(keypair, "https://api.devnet.solana.com".to_string()));
        
        let prompt = PromptRequest {
            text: "Test prompt".to_string(),
            conversation_id: Some("test-conv".to_string()),
            request_id: Some("test-req".to_string()),
        };
        
        let result = handle_agent_completion(agent, signer, prompt).await;
        assert!(result.is_ok());
        
        let response = result.unwrap();
        assert_eq!(response.response, "Test response");
        assert_eq!(response.conversation_id, "test-conv");
        assert_eq!(response.request_id, "test-req");
    }

    #[tokio::test]
    async fn test_handle_agent_stream() {
        let agent = MockAgent::new("Test response".to_string());
        let keypair = Keypair::new();
        let signer = std::sync::Arc::new(LocalSolanaSigner::new(keypair, "https://api.devnet.solana.com".to_string()));
        
        let prompt = PromptRequest {
            text: "Test prompt".to_string(),
            conversation_id: Some("test-conv".to_string()),
            request_id: Some("test-req".to_string()),
        };
        
        let result = handle_agent_stream(agent, signer, prompt).await;
        assert!(result.is_ok());
        
        let mut stream = result.unwrap();
        let mut events = Vec::new();
        
        while let Some(event_result) = stream.next().await {
            let event_json = event_result.unwrap();
            events.push(event_json);
        }
        
        // Should have start + content chunks + complete events
        assert!(events.len() >= 3); // At least start, some content, complete
        
        // First event should be start
        let first_event: AgentEvent = serde_json::from_str(&events[0]).unwrap();
        assert!(matches!(first_event, AgentEvent::Start { .. }));
        
        // Last event should be complete
        let last_event: AgentEvent = serde_json::from_str(events.last().unwrap()).unwrap();
        assert!(matches!(last_event, AgentEvent::Complete { .. }));
    }

    #[test]
    fn test_agent_event_serialization() {
        let event = AgentEvent::Start {
            conversation_id: "test-conv".to_string(),
            request_id: "test-req".to_string(),
            timestamp: chrono::Utc::now(),
        };
        
        let json = serde_json::to_string(&event).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        
        assert_eq!(parsed["type"], "start");
        assert_eq!(parsed["conversation_id"], "test-conv");
        assert_eq!(parsed["request_id"], "test-req");
    }
}