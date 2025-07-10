//! Integration tests for riglr-server handlers
//! 
//! These tests demonstrate that the SignerContext integration is working properly
//! in both the chat and stream handlers.

use actix_web::{test, web, App};
use riglr_server::handlers::ChatRequest;
use std::sync::Arc;

#[derive(Clone)]
struct MockAgent {
    _name: String,
}

impl MockAgent {
    fn new(name: &str) -> Self {
        Self {
            _name: name.to_string(),
        }
    }
}

#[actix_web::test]
async fn test_chat_handler_with_signer_context() {
    let agent = Arc::new(MockAgent::new("Test Agent"));
    
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(agent))
            .route("/chat", web::post().to(riglr_server::handlers::chat::<MockAgent>)),
    ).await;

    // Test chat request with default signer config
    let chat_request = ChatRequest {
        message: "Hello, can you check my Solana balance?".to_string(),
        signer_config: None, // Will use default
        conversation_id: None,
        request_id: Some("test-123".to_string()),
    };

    let req = test::TestRequest::post()
        .uri("/chat")
        .set_json(&chat_request)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let response_body: serde_json::Value = test::read_body_json(resp).await;
    
    // Verify the response structure
    assert!(response_body["response"].is_string());
    assert!(response_body["tool_calls"].is_array());
    assert_eq!(response_body["request_id"], "test-123");
    assert!(response_body["conversation_id"].is_string());
    assert!(response_body["timestamp"].is_string());
    
    // Check that the response indicates SignerContext is working
    let response_text = response_body["response"].as_str().unwrap();
    assert!(response_text.contains("SignerContext enabled"));
    
    // Should have tool calls demonstrating signer info access
    let tool_calls = response_body["tool_calls"].as_array().unwrap();
    assert!(!tool_calls.is_empty());
    
    let first_call = &tool_calls[0];
    assert_eq!(first_call["name"], "get_signer_info");
    assert!(first_call["result"].is_object());
}

#[actix_web::test]
async fn test_chat_handler_with_custom_signer_config() {
    let agent = Arc::new(MockAgent::new("Test Agent"));
    
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(agent))
            .route("/chat", web::post().to(riglr_server::handlers::chat::<MockAgent>)),
    ).await;

    // Test chat request with custom signer config
    let chat_request = ChatRequest {
        message: "Test message".to_string(),
        signer_config: Some(riglr_server::server::SignerConfig {
            solana_rpc_url: Some("https://api.mainnet-beta.solana.com".to_string()),
            evm_rpc_url: Some("https://eth.llamarpc.com".to_string()),
            user_id: Some("test-user-456".to_string()),
            locale: Some("es".to_string()),
        }),
        conversation_id: Some("conv-789".to_string()),
        request_id: None,
    };

    let req = test::TestRequest::post()
        .uri("/chat")
        .set_json(&chat_request)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let response_body: serde_json::Value = test::read_body_json(resp).await;
    
    // Verify the custom conversation_id was used
    assert_eq!(response_body["conversation_id"], "conv-789");
    
    // Should still have tool calls with signer info
    let tool_calls = response_body["tool_calls"].as_array().unwrap();
    assert!(!tool_calls.is_empty());
}

#[actix_web::test]
async fn test_stream_handler() {
    let agent = Arc::new(MockAgent::new("Test Agent"));
    
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(agent))
            .route("/stream", web::get().to(riglr_server::handlers::stream::<MockAgent>)),
    ).await;

    // Test streaming request
    let req = test::TestRequest::get()
        .uri("/stream?message=Test%20streaming&conversation_id=stream-test")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    
    // Check SSE headers
    let headers = resp.headers();
    assert_eq!(headers.get("content-type").unwrap(), "text/event-stream");
    assert_eq!(headers.get("cache-control").unwrap(), "no-cache");
    assert_eq!(headers.get("connection").unwrap(), "keep-alive");
}

#[actix_web::test]
async fn test_health_endpoint() {
    let agent = Arc::new(MockAgent::new("Test Agent"));
    
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(agent))
            .route("/health", web::get().to(riglr_server::handlers::health)),
    ).await;

    let req = test::TestRequest::get().uri("/health").to_request();
    let resp = test::call_service(&app, req).await;
    
    assert!(resp.status().is_success());
    
    let response_body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(response_body["status"], "healthy");
    assert!(response_body["timestamp"].is_string());
    assert!(response_body["version"].is_string());
}

#[actix_web::test]
async fn test_index_endpoint() {
    let agent = Arc::new(MockAgent::new("Test Agent"));
    
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(agent))
            .route("/", web::get().to(riglr_server::handlers::index)),
    ).await;

    let req = test::TestRequest::get().uri("/").to_request();
    let resp = test::call_service(&app, req).await;
    
    assert!(resp.status().is_success());
    
    let response_body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(response_body["name"], "riglr-server");
    assert!(response_body["endpoints"].is_array());
    
    let endpoints = response_body["endpoints"].as_array().unwrap();
    assert_eq!(endpoints.len(), 4); // /, /chat, /stream, /health
}