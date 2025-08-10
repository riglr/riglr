//! HTTP request handlers for agent interactions

use actix_web::{web, HttpResponse, HttpRequest};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;
use riglr_core::signer::{SignerContext, LocalSolanaSigner, TransactionSigner, SignerError};
use futures_util::{stream, Stream, StreamExt};
use crate::{server::SignerConfig, sse, Result};

/// Request structure for chat interactions
#[derive(Deserialize, Serialize, Debug)]
pub struct ChatRequest {
    /// The message to send to the agent
    pub message: String,
    /// Optional signer configuration for blockchain operations
    pub signer_config: Option<SignerConfig>,
    /// Optional conversation ID for tracking
    pub conversation_id: Option<String>,
    /// Optional request ID for tracing
    pub request_id: Option<String>,
}

/// Response structure for chat interactions
#[derive(Serialize, Deserialize, Debug)]
pub struct ChatResponse {
    /// The agent's response message
    pub response: String,
    /// List of tool calls made by the agent
    pub tool_calls: Vec<ToolCall>,
    /// Conversation ID for tracking
    pub conversation_id: String,
    /// Request ID for tracing
    pub request_id: String,
    /// Response timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Tool call information
#[derive(Serialize, Deserialize, Debug)]
pub struct ToolCall {
    /// Name of the tool that was called
    pub name: String,
    /// Arguments passed to the tool
    pub arguments: serde_json::Value,
    /// Result of the tool call
    pub result: Option<serde_json::Value>,
    /// Error if tool call failed
    pub error: Option<String>,
}

/// Request structure for streaming interactions
#[derive(Deserialize, Serialize, Debug)]
pub struct StreamRequest {
    /// The message to send to the agent
    pub message: String,
    /// Optional signer configuration for blockchain operations
    pub signer_config: Option<SignerConfig>,
    /// Optional conversation ID for tracking
    pub conversation_id: Option<String>,
}

/// Create a signer from the provided configuration
async fn create_signer_from_config(
    signer_config: &Option<SignerConfig>,
) -> std::result::Result<Arc<dyn TransactionSigner>, SignerError> {
    let config = signer_config.as_ref()
        .cloned()
        .unwrap_or_default();
    
    // For now, use LocalSigner with a generated keypair
    // In production, this would use proper key management
    let keypair = solana_sdk::signer::keypair::Keypair::new();
    let solana_rpc_url = config.solana_rpc_url
        .unwrap_or_else(|| "https://api.devnet.solana.com".to_string());
    
    let signer = LocalSolanaSigner::new(keypair, solana_rpc_url);
    Ok(Arc::new(signer))
}

/// Process an agent request with proper SignerContext (placeholder implementation)
async fn process_agent_request(
    message: &str,
    conversation_id: &str,
    request_id: &str,
) -> std::result::Result<ChatResponse, SignerError> {
    // Check that SignerContext is available
    let signer_available = SignerContext::is_available().await;
    
    // Placeholder AI processing logic
    let response_text = if signer_available {
        format!(
            "I received your message: '{}'. I'm running with SignerContext enabled and can access blockchain tools! \
             This is a demonstration of proper SignerContext integration.",
            message
        )
    } else {
        "Error: SignerContext is not available in this scope.".to_string()
    };
    
    // Simulate tool usage by checking if we can access the current signer
    let mut tool_calls = vec![];
    if signer_available {
        if let Ok(signer) = SignerContext::current().await {
            tool_calls.push(ToolCall {
                name: "get_signer_info".to_string(),
                arguments: serde_json::json!({}),
                result: Some(serde_json::json!({
                    "address": signer.address(),
                    "pubkey": signer.pubkey(),
                    "has_solana_client": true,
                    "has_evm_client": signer.evm_client().is_ok()
                })),
                error: None,
            });
        }
    }
    
    Ok(ChatResponse {
        response: response_text,
        tool_calls,
        conversation_id: conversation_id.to_string(),
        request_id: request_id.to_string(),
        timestamp: chrono::Utc::now(),
    })
}

/// Create a stream that executes within SignerContext
async fn create_signer_context_stream<A>(
    signer: Arc<dyn TransactionSigner>,
    _agent: Arc<A>,
    request: StreamRequest,
    request_id: String,
    conversation_id: String,
) -> impl Stream<Item = std::result::Result<actix_web::web::Bytes, actix_web::Error>>
where
    A: Clone + Send + Sync + 'static,
{
    // Execute within SignerContext to get events
    let events = match SignerContext::with_signer(signer, async {
        process_streaming_agent_request(&request.message, &conversation_id, &request_id).await
    }).await {
        Ok(events) => events,
        Err(e) => vec![sse::AgentEvent::Error {
            error: format!("SignerContext execution failed: {}", e),
            conversation_id,
            request_id,
            timestamp: chrono::Utc::now(),
        }],
    };
    
    // Convert events to SSE stream
    stream::iter(events).map(|event| {
        match serde_json::to_string(&event) {
            Ok(json) => Ok(actix_web::web::Bytes::from(format!("data: {}\n\n", json))),
            Err(e) => Err(actix_web::error::ErrorInternalServerError(format!("JSON error: {}", e))),
        }
    })
}

/// Process a streaming agent request with SignerContext (placeholder implementation)
async fn process_streaming_agent_request(
    message: &str,
    conversation_id: &str,
    request_id: &str,
) -> std::result::Result<Vec<sse::AgentEvent>, SignerError> {
    let signer_available = SignerContext::is_available().await;
    
    let mut events = vec![];
    
    // Start event
    events.push(sse::AgentEvent::Start {
        conversation_id: conversation_id.to_string(),
        request_id: request_id.to_string(),
        timestamp: chrono::Utc::now(),
    });
    
    // Content streaming
    if signer_available {
        events.push(sse::AgentEvent::Content {
            content: format!("I received your message: '{}'", message),
            conversation_id: conversation_id.to_string(),
            request_id: request_id.to_string(),
        });
        
        events.push(sse::AgentEvent::Content {
            content: " I'm running with SignerContext enabled!".to_string(),
            conversation_id: conversation_id.to_string(),
            request_id: request_id.to_string(),
        });
        
        // Simulate tool call
        if let Ok(signer) = SignerContext::current().await {
            events.push(sse::AgentEvent::ToolCall {
                tool_name: "get_signer_info".to_string(),
                arguments: serde_json::json!({}),
                conversation_id: conversation_id.to_string(),
                request_id: request_id.to_string(),
            });
            
            events.push(sse::AgentEvent::ToolResult {
                tool_name: "get_signer_info".to_string(),
                result: Some(serde_json::json!({
                    "address": signer.address(),
                    "pubkey": signer.pubkey(),
                    "has_solana_client": true,
                    "has_evm_client": signer.evm_client().is_ok()
                })),
                error: None,
                conversation_id: conversation_id.to_string(),
                request_id: request_id.to_string(),
            });
        }
        
        events.push(sse::AgentEvent::Content {
            content: " I can access blockchain tools and process transactions securely!".to_string(),
            conversation_id: conversation_id.to_string(),
            request_id: request_id.to_string(),
        });
    } else {
        events.push(sse::AgentEvent::Content {
            content: "Error: SignerContext is not available in this scope.".to_string(),
            conversation_id: conversation_id.to_string(),
            request_id: request_id.to_string(),
        });
    }
    
    // Complete event
    events.push(sse::AgentEvent::Complete {
        conversation_id: conversation_id.to_string(),
        request_id: request_id.to_string(),
        timestamp: chrono::Utc::now(),
        total_tokens: Some(150),
    });
    
    Ok(events)
}

/// Handle POST /chat - Single-turn chat with agent
pub async fn chat<A>(
    _agent: web::Data<Arc<A>>,
    request: web::Json<ChatRequest>,
) -> Result<HttpResponse>
where
    A: Clone + Send + Sync + 'static,
{
    let request_id = request.request_id
        .clone()
        .unwrap_or_else(|| Uuid::new_v4().to_string());
    let conversation_id = request.conversation_id
        .clone()
        .unwrap_or_else(|| Uuid::new_v4().to_string());
    
    tracing::info!(
        request_id = %request_id,
        conversation_id = %conversation_id,
        message_len = request.message.len(),
        "Processing chat request"
    );
    
    // Create signer from request configuration
    let signer_result = create_signer_from_config(&request.signer_config).await;
    
    match signer_result {
        Ok(signer) => {
            // Execute within SignerContext
            let result = SignerContext::with_signer(signer, async {
                // Process AI agent request with available tools
                process_agent_request(&request.message, &conversation_id, &request_id).await
            }).await;
            
            match result {
                Ok(response) => {
                    tracing::info!(
                        request_id = %response.request_id,
                        tool_calls = response.tool_calls.len(),
                        "Chat request completed successfully"
                    );
                    Ok(HttpResponse::Ok().json(response))
                }
                Err(e) => {
                    tracing::error!(
                        request_id = %request_id,
                        error = %e,
                        "SignerContext execution failed"
                    );
                    
                    let error_response = ChatResponse {
                        response: format!("Error processing request: {}", e),
                        tool_calls: vec![],
                        conversation_id,
                        request_id,
                        timestamp: chrono::Utc::now(),
                    };
                    Ok(HttpResponse::InternalServerError().json(error_response))
                }
            }
        }
        Err(e) => {
            tracing::error!(
                request_id = %request_id,
                error = %e,
                "Failed to create signer"
            );
            
            let error_response = ChatResponse {
                response: format!("Configuration error: {}", e),
                tool_calls: vec![],
                conversation_id,
                request_id,
                timestamp: chrono::Utc::now(),
            };
            Ok(HttpResponse::BadRequest().json(error_response))
        }
    }
}

/// Handle GET /stream - Server-Sent Events streaming
pub async fn stream<A>(
    agent: web::Data<Arc<A>>,
    _req: HttpRequest,
    query: web::Query<StreamRequest>,
) -> Result<HttpResponse>
where
    A: Clone + Send + Sync + 'static,
{
    let conversation_id = query.conversation_id
        .clone()
        .unwrap_or_else(|| Uuid::new_v4().to_string());
    let request_id = Uuid::new_v4().to_string();
    
    tracing::info!(
        request_id = %request_id,
        conversation_id = %conversation_id,
        message_len = query.message.len(),
        "Starting SSE stream"
    );
    
    // Create signer from request configuration
    let signer_result = create_signer_from_config(&query.signer_config).await;
    
    match signer_result {
        Ok(signer) => {
            // Create a stream that wraps SignerContext execution
            let stream = create_signer_context_stream(
                signer,
                agent.get_ref().clone(),
                query.into_inner(),
                request_id.clone(),
                conversation_id.clone(),
            ).await;
            
            tracing::info!(
                request_id = %request_id,
                conversation_id = %conversation_id,
                "SSE stream created successfully"
            );
            
            // Set SSE headers and return streaming response
            Ok(HttpResponse::Ok()
                .content_type("text/event-stream")
                .insert_header(("Cache-Control", "no-cache"))
                .insert_header(("Connection", "keep-alive"))
                .insert_header(("Access-Control-Allow-Origin", "*"))
                .insert_header(("Access-Control-Allow-Headers", "Cache-Control"))
                .streaming(stream))
        }
        Err(e) => {
            tracing::error!(
                request_id = %request_id,
                error = %e,
                "Failed to create signer for stream"
            );
            
            // Return error as SSE stream
            let error_event = sse::AgentEvent::Error {
                error: format!("Configuration error: {}", e),
                conversation_id,
                request_id,
                timestamp: chrono::Utc::now(),
            };
            
            let error_json = serde_json::to_string(&error_event)
                .unwrap_or_else(|_| "{}".to_string());
            
            Ok(HttpResponse::BadRequest()
                .content_type("text/event-stream")
                .body(format!("data: {}\n\n", error_json)))
        }
    }
}

/// Handle GET /health - Health check endpoint
pub async fn health() -> Result<HttpResponse> {
    #[derive(Serialize)]
    struct HealthResponse {
        status: String,
        timestamp: chrono::DateTime<chrono::Utc>,
        version: String,
    }
    
    let health = HealthResponse {
        status: "healthy".to_string(),
        timestamp: chrono::Utc::now(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    };
    
    Ok(HttpResponse::Ok().json(health))
}

/// Handle GET / - Server information
pub async fn index() -> Result<HttpResponse> {
    #[derive(Serialize)]
    struct ServerInfo {
        name: String,
        version: String,
        description: String,
        endpoints: Vec<EndpointInfo>,
    }
    
    #[derive(Serialize)]
    struct EndpointInfo {
        method: String,
        path: String,
        description: String,
    }
    
    let info = ServerInfo {
        name: "riglr-server".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        description: "HTTP server for serving rig agents over REST API".to_string(),
        endpoints: vec![
            EndpointInfo {
                method: "GET".to_string(),
                path: "/".to_string(),
                description: "Server information".to_string(),
            },
            EndpointInfo {
                method: "POST".to_string(),
                path: "/chat".to_string(),
                description: "Chat with agent (single-turn)".to_string(),
            },
            EndpointInfo {
                method: "GET".to_string(),
                path: "/stream".to_string(),
                description: "Server-Sent Events streaming".to_string(),
            },
            EndpointInfo {
                method: "GET".to_string(),
                path: "/health".to_string(),
                description: "Health check".to_string(),
            },
        ],
    };
    
    Ok(HttpResponse::Ok().json(info))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_chat_request_serialization() {
        let request = ChatRequest {
            message: "test".to_string(),
            signer_config: None,
            conversation_id: None,
            request_id: None,
        };
        
        // Test that the request can be serialized/deserialized
        let json = serde_json::to_string(&request).unwrap();
        let parsed: ChatRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.message, "test");
    }
    
    #[tokio::test]
    async fn test_chat_response_serialization() {
        let response = ChatResponse {
            response: "Hello!".to_string(),
            tool_calls: vec![],
            conversation_id: "conv-123".to_string(),
            request_id: "req-456".to_string(),
            timestamp: chrono::Utc::now(),
        };
        
        let json = serde_json::to_string(&response).unwrap();
        let parsed: ChatResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.response, "Hello!");
        assert_eq!(parsed.conversation_id, "conv-123");
    }
    
    #[tokio::test]
    async fn test_signer_config_serialization() {
        let config = crate::server::SignerConfig {
            solana_rpc_url: Some("https://api.devnet.solana.com".to_string()),
            evm_rpc_url: Some("https://eth.public-rpc.com".to_string()),
            user_id: Some("user123".to_string()),
            locale: Some("en".to_string()),
        };
        
        let json = serde_json::to_string(&config).unwrap();
        let parsed: crate::server::SignerConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.user_id, Some("user123".to_string()));
    }
}