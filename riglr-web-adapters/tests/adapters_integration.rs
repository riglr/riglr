use riglr_web_adapters::{
    core::{handle_agent_stream, handle_agent_completion, PromptRequest, CompletionResponse},
};
use riglr_core::{
    signer::{TransactionSigner, SignerError, EvmClient},
};
use std::sync::Arc;

/// Test error type
#[derive(Debug)]
struct TestError(String);

impl std::fmt::Display for TestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for TestError {}
use solana_sdk::transaction::Transaction;
use actix_web::{
    test,
    web::{self, Data},
    http::header::{HeaderMap, HeaderName, HeaderValue},
    HttpRequest,
    App,
};
use futures_util::StreamExt;

/// Mock agent for testing web adapter functionality
#[derive(Clone)]
struct MockAgent {
    responses: Vec<String>,
}

impl MockAgent {
    fn new(responses: Vec<String>) -> Self {
        Self { responses }
    }
}

#[async_trait::async_trait]
impl riglr_web_adapters::core::Agent for MockAgent {
    type Error = TestError;
    
    async fn prompt(&self, _prompt: &str) -> Result<String, Self::Error> {
        Ok(self.responses.first().unwrap_or(&"Mock response".to_string()).clone())
    }
    
    async fn prompt_stream(&self, _prompt: &str) -> Result<futures_util::stream::BoxStream<'_, Result<String, Self::Error>>, Self::Error> {
        let stream = futures_util::stream::iter(self.responses.clone().into_iter().map(Ok::<_, TestError>));
        Ok(Box::pin(stream))
    }
}


/// Mock signer for testing adapter authentication
struct MockAdapterSigner {
    user_id: String,
    wallet_address: Option<String>,
}

impl MockAdapterSigner {
    fn new(user_id: String, wallet_address: Option<String>) -> Self {
        Self { user_id, wallet_address }
    }
}

#[async_trait::async_trait]
impl TransactionSigner for MockAdapterSigner {
    fn pubkey(&self) -> Option<String> {
        None
    }

    fn address(&self) -> Option<String> {
        self.wallet_address.clone()
    }

    async fn sign_and_send_solana_transaction(&self, _tx: &mut Transaction) -> Result<String, SignerError> {
        Ok(format!("solana_tx_{}", self.user_id))
    }

    async fn sign_and_send_evm_transaction(&self, _tx: alloy::rpc::types::TransactionRequest) -> Result<String, SignerError> {
        Ok(format!("evm_tx_{}", self.user_id))
    }

    fn solana_client(&self) -> Option<Arc<solana_client::rpc_client::RpcClient>> {
        Some(Arc::new(solana_client::rpc_client::RpcClient::new("https://api.devnet.solana.com".to_string())))
    }

    fn evm_client(&self) -> Result<Arc<dyn EvmClient>, SignerError> {
        Err(SignerError::UnsupportedOperation("Mock signer does not provide EVM client".to_string()))
    }
}

impl std::fmt::Debug for MockAdapterSigner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MockAdapterSigner")
            .field("user_id", &self.user_id)
            .field("wallet_address", &self.wallet_address)
            .finish()
    }
}

#[tokio::test]
async fn test_core_handler_logic_completion() {
    let agent = MockAgent::new(vec!["Test response".to_string()]);
    let signer = Arc::new(MockAdapterSigner::new("user123".to_string(), Some("0x742d35Cc2F5f8a89A0D2EAd5a53c97c49444E34F".to_string())));
    
    let prompt = PromptRequest {
        text: "Test prompt".to_string(),
        conversation_id: None,
        request_id: None,
    };
    
    let result = handle_agent_completion(
        agent,
        signer,
        prompt,
    ).await;
    
    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.response, "Test response");
}

#[tokio::test]
async fn test_core_handler_logic_streaming() {
    let agent = MockAgent::new(vec![
        "First chunk".to_string(),
        "Second chunk".to_string(),
        "Third chunk".to_string(),
    ]);
    let signer = Arc::new(MockAdapterSigner::new("user123".to_string(), Some("0x742d35Cc2F5f8a89A0D2EAd5a53c97c49444E34F".to_string())));
    
    let prompt = PromptRequest {
        text: "Test prompt".to_string(),
        conversation_id: Some("test-conv".to_string()),
        request_id: Some("test-req".to_string()),
    };
    
    let result = handle_agent_stream(
        agent,
        signer,
        prompt,
    ).await;
    
    assert!(result.is_ok());
    let mut stream = result.unwrap();
    
    let mut events = Vec::new();
    while let Some(chunk_result) = stream.next().await {
        assert!(chunk_result.is_ok());
        let json_str = chunk_result.unwrap();
        let event: riglr_web_adapters::core::AgentEvent = serde_json::from_str(&json_str).unwrap();
        events.push(event);
    }
    
    // Should have: Start + 3 Content chunks + Complete = 5 events
    assert_eq!(events.len(), 5);
    
    // First event should be Start
    assert!(matches!(events[0], riglr_web_adapters::core::AgentEvent::Start { .. }));
    
    // Middle events should be Content
    assert!(matches!(events[1], riglr_web_adapters::core::AgentEvent::Content { ref content, .. } if content == "First chunk"));
    assert!(matches!(events[2], riglr_web_adapters::core::AgentEvent::Content { ref content, .. } if content == "Second chunk"));
    assert!(matches!(events[3], riglr_web_adapters::core::AgentEvent::Content { ref content, .. } if content == "Third chunk"));
    
    // Last event should be Complete
    assert!(matches!(events[4], riglr_web_adapters::core::AgentEvent::Complete { .. }));
}

#[tokio::test]
async fn test_signer_context_in_handlers() {
    let agent = MockAgent::new(vec!["Response".to_string()]);
    let signer = MockAdapterSigner::new("user456".to_string(), Some("0x1234567890123456789012345678901234567890".to_string()));
    
    // Test that signer context is properly set within handler
    let prompt = PromptRequest {
        text: "Test prompt".to_string(),
        conversation_id: None,
        request_id: None,
    };
    let result = handle_agent_completion(
        agent,
        Arc::new(signer),
        prompt,
    ).await;
    
    assert!(result.is_ok());
    
    // The actual verification of signer context would happen within the agent's tools
    // For this test, we just verify the handler completes successfully
}

#[tokio::test]
async fn test_concurrent_handler_requests() {
    let agents = (0..5).map(|i| {
        MockAgent::new(vec![format!("Response {}", i)])
    }).collect::<Vec<_>>();
    
    let signers = (0..5).map(|i| {
        MockAdapterSigner::new(
            format!("user{}", i),
            Some(format!("0x{:040x}", i))
        )
    }).collect::<Vec<_>>();
    
    let handles = agents.into_iter().zip(signers.into_iter()).enumerate().map(|(i, (agent, signer))| {
        tokio::spawn(async move {
            let prompt = PromptRequest {
                text: format!("Prompt {}", i),
                conversation_id: None,
                request_id: None,
            };
            handle_agent_completion(
                agent,
                Arc::new(signer),
                prompt,
            ).await
        })
    }).collect::<Vec<_>>();
    
    let results = futures::future::join_all(handles).await;
    
    for (i, result) in results.into_iter().enumerate() {
        let response = result.unwrap().unwrap();
        assert_eq!(response.response, format!("Response {}", i));
    }
}

#[actix_web::test]
async fn test_actix_completion_handler() {
    let app = test::init_service(
        App::new()
            .app_data(Data::new(MockAgent::new(vec!["Test response".to_string()])))
            .route("/completion", web::post().to(mock_completion_handler))
    ).await;
    
    let prompt = PromptRequest {
        text: "Test prompt".to_string(),
        conversation_id: None,
        request_id: None,
    };
    
    let req = test::TestRequest::post()
        .uri("/completion")
        .insert_header(("Authorization", "Bearer mock_token"))
        .set_json(&prompt)
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    
    let response_body: CompletionResponse = test::read_body_json(resp).await;
    assert_eq!(response_body.response, "Test response");
    assert_eq!(response_body.model, "claude-3-5-sonnet");
}

#[tokio::test]
async fn test_authorization_header_extraction() {
    // Test valid authorization header
    let mut headers = HeaderMap::new();
    headers.insert(
        HeaderName::from_static("authorization"),
        HeaderValue::from_static("Bearer valid_token_123")
    );
    
    let result = extract_mock_auth_token(&headers);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "valid_token_123");
    
    // Test missing authorization header
    let empty_headers = HeaderMap::new();
    let result = extract_mock_auth_token(&empty_headers);
    assert!(result.is_err());
    
    // Test invalid authorization format
    let mut invalid_headers = HeaderMap::new();
    invalid_headers.insert(
        HeaderName::from_static("authorization"),
        HeaderValue::from_static("InvalidFormat token")
    );
    
    let result = extract_mock_auth_token(&invalid_headers);
    assert!(result.is_err());
}

#[tokio::test]
async fn test_event_streaming_format() {
    let agent = MockAgent::new(vec![
        "First".to_string(),
        "Second".to_string(),
        "Third".to_string(),
    ]);
    let signer = MockAdapterSigner::new("user123".to_string(), None);
    
    let prompt = PromptRequest {
        text: "Test".to_string(),
        conversation_id: Some("test-conv".to_string()),
        request_id: Some("test-req".to_string()),
    };
    let result = handle_agent_stream(agent, Arc::new(signer), prompt).await;
    assert!(result.is_ok());
    
    let mut stream = result.unwrap();
    let mut events = Vec::new();
    
    while let Some(chunk_result) = stream.next().await {
        assert!(chunk_result.is_ok());
        let json_str = chunk_result.unwrap();
        
        // Verify it's valid JSON and can be parsed as AgentEvent
        assert!(!json_str.is_empty());
        let event: riglr_web_adapters::core::AgentEvent = serde_json::from_str(&json_str).unwrap();
        events.push(event);
    }
    
    // Should have: Start + 3 Content chunks + Complete = 5 events
    assert_eq!(events.len(), 5);
    
    // Verify event sequence
    assert!(matches!(events[0], riglr_web_adapters::core::AgentEvent::Start { .. }));
    assert!(matches!(events[1], riglr_web_adapters::core::AgentEvent::Content { .. }));
    assert!(matches!(events[2], riglr_web_adapters::core::AgentEvent::Content { .. }));
    assert!(matches!(events[3], riglr_web_adapters::core::AgentEvent::Content { .. }));
    assert!(matches!(events[4], riglr_web_adapters::core::AgentEvent::Complete { .. }));
}

#[tokio::test]
async fn test_error_handling_in_adapters() {
    // Test agent error
    let error_agent = MockErrorAgent::new("Agent processing error".to_string());
    let signer = MockAdapterSigner::new("user123".to_string(), None);
    
    let prompt = PromptRequest {
        text: "Test prompt".to_string(),
        conversation_id: None,
        request_id: None,
    };
    
    let result = handle_agent_completion(
        error_agent,
        Arc::new(signer),
        prompt,
    ).await;
    
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.to_string().contains("Agent processing error"));
}

#[tokio::test]
async fn test_adapter_with_different_signer_types() {
    let agent = MockAgent::new(vec!["Response".to_string()]);
    
    // Test with Solana signer (no address)
    let solana_signer = MockAdapterSigner::new("solana_user".to_string(), None);
    let prompt = PromptRequest {
        text: "Test".to_string(),
        conversation_id: None,
        request_id: None,
    };
    let result = handle_agent_completion(agent.clone(), Arc::new(solana_signer), prompt).await;
    assert!(result.is_ok());
    
    // Test with EVM signer (with address)
    let evm_signer = MockAdapterSigner::new(
        "evm_user".to_string(),
        Some("0x742d35Cc2F5f8a89A0D2EAd5a53c97c49444E34F".to_string())
    );
    let prompt = PromptRequest {
        text: "Test".to_string(),
        conversation_id: None,
        request_id: None,
    };
    let result = handle_agent_completion(agent, Arc::new(evm_signer), prompt).await;
    assert!(result.is_ok());
}

// Helper functions for testing

async fn mock_completion_handler(
    req: HttpRequest,
    agent: Data<MockAgent>,
    prompt: web::Json<PromptRequest>,
) -> Result<web::Json<CompletionResponse>, actix_web::Error> {
    // Mock signer extraction
    let _token = extract_mock_auth_token(req.headers())
        .map_err(|_| actix_web::error::ErrorUnauthorized("Invalid auth"))?;
    
    let signer = MockAdapterSigner::new("test_user".to_string(), None);
    
    match handle_agent_completion(
        agent.get_ref().clone(),
        Arc::new(signer),
        prompt.into_inner(),
    ).await {
        Ok(response) => Ok(web::Json(response)),
        Err(e) => Err(actix_web::error::ErrorInternalServerError(e.to_string())),
    }
}

fn extract_mock_auth_token(headers: &HeaderMap) -> Result<String, &'static str> {
    let auth_header = headers
        .get("authorization")
        .ok_or("Missing Authorization header")?
        .to_str()
        .map_err(|_| "Invalid Authorization header format")?;
    
    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or("Invalid Authorization format")?;
    
    Ok(token.to_string())
}

/// Mock agent that returns errors for testing error handling
#[derive(Clone)]
struct MockErrorAgent {
    error_message: String,
}

impl MockErrorAgent {
    fn new(error_message: String) -> Self {
        Self { error_message }
    }
}

#[async_trait::async_trait]
impl riglr_web_adapters::core::Agent for MockErrorAgent {
    type Error = TestError;
    
    async fn prompt(&self, _prompt: &str) -> Result<String, Self::Error> {
        Err(TestError(self.error_message.clone()))
    }
    
    async fn prompt_stream(&self, _prompt: &str) -> Result<futures_util::stream::BoxStream<'_, Result<String, Self::Error>>, Self::Error> {
        Err(TestError(self.error_message.clone()))
    }
}

