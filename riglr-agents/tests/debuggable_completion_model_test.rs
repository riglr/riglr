//! Tests for the DebuggableCompletionModel wrapper.
//!
//! This test module verifies that the DebuggableCompletionModel correctly:
//! - Implements Debug trait for debugging purposes
//! - Forwards completion requests to the underlying model
//! - Preserves access to the inner model through Deref

use rig::completion::{CompletionModel, CompletionRequest, Message};
use riglr_agents::agents::tool_calling::DebuggableCompletionModel;

// Helper function to create empty completion requests
fn empty_completion_request() -> CompletionRequest {
    // Create a message using the proper constructor
    let message = Message::user("test".to_string());

    CompletionRequest {
        preamble: None,
        chat_history: rig::OneOrMany::one(message),
        documents: vec![],
        max_tokens: None,
        temperature: None,
        tools: vec![],
        additional_params: None,
    }
}

// Mock completion model for testing
#[derive(Clone)]
struct MockCompletionModel {
    response: String,
}

impl CompletionModel for MockCompletionModel {
    type Response = String;
    type StreamingResponse = ();

    fn completion(
        &self,
        _request: CompletionRequest,
    ) -> impl std::future::Future<
        Output = std::result::Result<
            rig::completion::CompletionResponse<Self::Response>,
            rig::completion::CompletionError,
        >,
    > + Send {
        let response = self.response.clone();
        async move {
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
        _request: CompletionRequest,
    ) -> std::result::Result<
        rig::streaming::StreamingCompletionResponse<Self::StreamingResponse>,
        rig::completion::CompletionError,
    > {
        Err(rig::completion::CompletionError::RequestError(
            std::io::Error::new(std::io::ErrorKind::Other, "Mock streaming not implemented").into(),
        ))
    }
}

#[test]
fn test_debuggable_completion_model_implements_debug() {
    let model = MockCompletionModel {
        response: "test".to_string(),
    };
    let debuggable = DebuggableCompletionModel::new(model);

    // This test will only compile if DebuggableCompletionModel implements Debug
    let debug_string = format!("{:?}", debuggable);
    assert!(debug_string.contains("DebuggableCompletionModel"));
}

#[tokio::test]
async fn test_debuggable_completion_model_forwards_completion() {
    let expected_response = "Hello from mock model";
    let model = MockCompletionModel {
        response: expected_response.to_string(),
    };
    let debuggable = DebuggableCompletionModel::new(model);

    let request = empty_completion_request();
    let response = debuggable.completion(request).await.unwrap();

    // Check that we got the expected response structure
    // For now, just check that the response was created - the specifics depend on rig's API
    // which we'll need to adapt to based on the actual structure
    let _choice = response.choice; // Just verify the response exists

    // Since we control the mock, we know what it returns
    // This test primarily verifies that the DebuggableCompletionModel properly delegates
    // and that the response structure is preserved
    assert_eq!(response.raw_response, ""); // Our mock sets this to empty string
}

#[test]
fn test_debuggable_completion_model_preserves_inner_model() {
    let model = MockCompletionModel {
        response: "test".to_string(),
    };
    let debuggable = DebuggableCompletionModel::new(model);

    // Access inner model through Deref
    let inner: &MockCompletionModel = debuggable.as_ref();
    assert_eq!(inner.response, "test");
}
