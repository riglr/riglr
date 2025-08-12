//! Actix Web adapter for riglr agents
//!
//! This module provides Actix-specific handlers that wrap the framework-agnostic
//! core handlers. It handles Privy authentication, request/response conversion,
//! and SSE streaming in the Actix Web ecosystem.

use actix_web::{web, HttpRequest, HttpResponse, Result as ActixResult};
use futures_util::StreamExt;
use crate::core::Agent;
use riglr_core::signer::TransactionSigner;
use crate::core::{handle_agent_stream, handle_agent_completion, PromptRequest};

/// Extract authorization token from Authorization header
///
/// This function parses the Authorization header to extract a Bearer token.
/// In a real implementation, this would create a PrivySigner or other signer
/// implementation based on the token.
///
/// # Arguments
/// * `req` - The HTTP request containing headers
///
/// # Returns
/// The bearer token or an HTTP error response
pub async fn extract_auth_token(req: &HttpRequest) -> Result<String, HttpResponse> {
    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| {
            tracing::warn!("Missing Authorization header");
            HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Missing Authorization header",
                "code": "MISSING_AUTH_HEADER"
            }))
        })?;
    
    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| {
            tracing::warn!("Invalid Authorization header format");
            HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Invalid Authorization format. Expected: Bearer <token>",
                "code": "INVALID_AUTH_FORMAT"
            }))
        })?;
    
    tracing::debug!("Extracted Bearer token from Authorization header");
    Ok(token.to_string())
}

/// Create a signer from an auth token
///
/// This is a mock implementation for demonstration purposes.
/// In a real application, this would:
/// 1. Validate the token with Privy
/// 2. Extract user wallet information
/// 3. Create appropriate signer implementation
pub async fn create_signer_from_token(token: &str) -> Result<std::sync::Arc<dyn TransactionSigner>, HttpResponse> {
    // Mock implementation - in production this would validate the token
    // and create a proper PrivySigner or other implementation
    use riglr_solana_tools::signer::LocalSolanaSigner;
    use solana_sdk::signature::Keypair;
    
    tracing::info!(token_len = token.len(), "Creating mock signer from token");
    
    let keypair = Keypair::new();
    let signer = LocalSolanaSigner::new(keypair, "https://api.devnet.solana.com".to_string());
    
    Ok(std::sync::Arc::new(signer))
}

/// Actix handler for Server-Sent Events streaming
///
/// This handler extracts the Privy signer from the request, uses the core
/// handler to process the agent stream, and converts the results to Actix SSE format.
///
/// # Arguments
/// * `req` - HTTP request for signer extraction
/// * `agent` - The rig agent wrapped in Actix Data
/// * `prompt` - JSON prompt request body
///
/// # Returns
/// An SSE streaming HTTP response
pub async fn sse_handler<A>(
    req: HttpRequest,
    agent: web::Data<A>,
    prompt: web::Json<PromptRequest>,
) -> ActixResult<HttpResponse>
where
    A: Agent + Clone + Send + Sync + 'static,
    A::Error: std::fmt::Display + std::fmt::Debug + Send + Sync + 'static,
{
    tracing::info!(
        prompt_len = prompt.text.len(),
        conversation_id = ?prompt.conversation_id,
        request_id = ?prompt.request_id,
        "Processing SSE request"
    );

    // Extract token and create signer
    let token = match extract_auth_token(&req).await {
        Ok(t) => t,
        Err(response) => return Ok(response),
    };
    
    let signer = match create_signer_from_token(&token).await {
        Ok(s) => s,
        Err(response) => return Ok(response),
    };
    
    // Handle stream using framework-agnostic core
    let stream_result = handle_agent_stream(
        agent.get_ref().clone(),
        signer,
        prompt.into_inner(),
    ).await;
    
    match stream_result {
        Ok(stream) => {
            tracing::info!("Agent stream created successfully");
            
            // Convert to Actix SSE stream
            let sse_stream = stream.map(|chunk| {
                match chunk {
                    Ok(data) => Ok(actix_web::web::Bytes::from(format!("data: {}\n\n", data))),
                    Err(e) => {
                        tracing::error!(error = %e, "Stream error");
                        Err(actix_web::error::ErrorInternalServerError(e))
                    }
                }
            });
            
            Ok(HttpResponse::Ok()
                .content_type("text/event-stream")
                .insert_header(("Cache-Control", "no-cache"))
                .insert_header(("Connection", "keep-alive"))
                .insert_header(("Access-Control-Allow-Origin", "*"))
                .insert_header(("Access-Control-Allow-Headers", "Cache-Control, Authorization"))
                .streaming(sse_stream))
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to create agent stream");
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": e.to_string(),
                "code": "AGENT_STREAM_ERROR"
            })))
        }
    }
}

/// Actix handler for one-shot completion
///
/// This handler extracts the Privy signer from the request, uses the core
/// handler to process the agent completion, and returns the JSON response.
///
/// # Arguments
/// * `req` - HTTP request for signer extraction
/// * `agent` - The rig agent wrapped in Actix Data
/// * `prompt` - JSON prompt request body
///
/// # Returns
/// A JSON completion response
pub async fn completion_handler<A>(
    req: HttpRequest,
    agent: web::Data<A>,
    prompt: web::Json<PromptRequest>,
) -> ActixResult<HttpResponse>
where
    A: Agent + Clone + Send + Sync + 'static,
    A::Error: std::fmt::Display + std::fmt::Debug + Send + Sync + 'static,
{
    tracing::info!(
        prompt_len = prompt.text.len(),
        conversation_id = ?prompt.conversation_id,
        request_id = ?prompt.request_id,
        "Processing completion request"
    );

    // Extract token and create signer
    let token = match extract_auth_token(&req).await {
        Ok(t) => t,
        Err(response) => return Ok(response),
    };
    
    let signer = match create_signer_from_token(&token).await {
        Ok(s) => s,
        Err(response) => return Ok(response),
    };
    
    // Handle completion using framework-agnostic core
    match handle_agent_completion(
        agent.get_ref().clone(),
        signer,
        prompt.into_inner(),
    ).await {
        Ok(response) => {
            tracing::info!(
                conversation_id = %response.conversation_id,
                request_id = %response.request_id,
                response_len = response.response.len(),
                "Completion request processed successfully"
            );
            Ok(HttpResponse::Ok().json(response))
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to process completion");
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": e.to_string(),
                "code": "AGENT_COMPLETION_ERROR"
            })))
        }
    }
}

/// Health check handler
pub async fn health_handler() -> ActixResult<HttpResponse> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "service": "riglr-web-adapters",
        "version": env!("CARGO_PKG_VERSION")
    })))
}

/// Information handler  
pub async fn info_handler() -> ActixResult<HttpResponse> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "service": "riglr-web-adapters",
        "version": env!("CARGO_PKG_VERSION"),
        "description": "Framework-agnostic web adapters for riglr agents",
        "endpoints": [
            {
                "method": "POST",
                "path": "/api/v1/sse",
                "description": "Server-Sent Events streaming with agent"
            },
            {
                "method": "POST", 
                "path": "/api/v1/completion",
                "description": "One-shot completion with agent"
            },
            {
                "method": "GET",
                "path": "/health",
                "description": "Health check"
            }
        ]
    })))
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, web, App};
    use riglr_solana_tools::signer::LocalSolanaSigner;
    use solana_sdk::signature::Keypair;
    use std::error::Error as StdError;

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
            let chunks = vec!["Hello", " ", "world"];
            let stream = futures_util::stream::iter(chunks)
                .map(|chunk| Ok(chunk.to_string()));
            Ok(Box::pin(stream))
        }
    }

    #[actix_web::test]
    async fn test_health_handler() {
        let app = test::init_service(
            App::new().route("/health", web::get().to(health_handler))
        ).await;

        let req = test::TestRequest::get()
            .uri("/health")
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let body = test::read_body(resp).await;
        let health_response: serde_json::Value = serde_json::from_slice(&body).unwrap();
        
        assert_eq!(health_response["status"], "healthy");
        assert!(health_response["timestamp"].is_string());
    }

    #[actix_web::test] 
    async fn test_info_handler() {
        let app = test::init_service(
            App::new().route("/", web::get().to(info_handler))
        ).await;

        let req = test::TestRequest::get()
            .uri("/")
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let body = test::read_body(resp).await;
        let info_response: serde_json::Value = serde_json::from_slice(&body).unwrap();
        
        assert_eq!(info_response["service"], "riglr-web-adapters");
        assert!(info_response["version"].is_string());
        assert!(info_response["endpoints"].is_array());
    }

    #[actix_web::test]
    async fn test_extract_auth_token_missing_header() {
        let req = test::TestRequest::get().to_http_request();
        let result = extract_auth_token(&req).await;
        assert!(result.is_err());
    }

    #[actix_web::test]
    async fn test_extract_auth_token_invalid_format() {
        let req = test::TestRequest::get()
            .insert_header(("Authorization", "InvalidFormat"))
            .to_http_request();
        let result = extract_auth_token(&req).await;
        assert!(result.is_err());
    }
}