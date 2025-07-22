//! Actix Web adapter for riglr agents
//!
//! This module provides Actix-specific handlers that wrap the framework-agnostic
//! core handlers. It handles authentication via pluggable SignerFactory implementations,
//! request/response conversion, and SSE streaming in the Actix Web ecosystem.

use actix_web::{HttpRequest, HttpResponse, Result as ActixResult};
use futures_util::StreamExt;
use crate::core::Agent;
use crate::factory::{SignerFactory, AuthenticationData};
use riglr_core::config::RpcConfig;
use riglr_core::signer::TransactionSigner;
use crate::core::{handle_agent_stream, handle_agent_completion, PromptRequest};
use std::sync::Arc;

/// Actix Web adapter that uses SignerFactory for authentication
#[derive(Clone)]
pub struct ActixRiglrAdapter {
    signer_factory: Arc<dyn SignerFactory>,
    rpc_config: RpcConfig,
}

impl ActixRiglrAdapter {
    /// Create a new Actix adapter with the given signer factory and RPC config
    pub fn new(signer_factory: Arc<dyn SignerFactory>, rpc_config: RpcConfig) -> Self {
        Self {
            signer_factory,
            rpc_config,
        }
    }
    
    /// Detect authentication type from request headers
    fn detect_auth_type(&self, req: &HttpRequest) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        // First check for explicit auth type header
        if let Some(auth_type) = req.headers().get("x-auth-type") {
            return Ok(auth_type.to_str()?.to_string());
        }
        
        // Fall back to registered auth types - use first one as default
        let supported_types = self.signer_factory.supported_auth_types();
        if supported_types.is_empty() {
            return Err("No authentication providers registered".into());
        }
        
        // For now, return the first registered type
        // In the future, this could inspect the JWT to determine the issuer
        Ok(supported_types[0].clone())
    }
    
    /// Extract authentication data from request headers
    fn extract_auth_data(&self, req: &HttpRequest) -> Result<AuthenticationData, Box<dyn std::error::Error + Send + Sync>> {
        let auth_header = req.headers()
            .get("authorization")
            .ok_or("Missing authorization header")?
            .to_str()?;
        
        // Parse auth header to determine type and extract credentials
        if auth_header.starts_with("Bearer ") {
            let token = auth_header.strip_prefix("Bearer ").unwrap();
            let auth_type = self.detect_auth_type(req)?;
            
            Ok(AuthenticationData {
                auth_type,
                credentials: [("token".to_string(), token.to_string())].into(),
                network: req.headers()
                    .get("x-network")
                    .and_then(|h| h.to_str().ok())
                    .unwrap_or("mainnet")
                    .to_string(),
            })
        } else {
            Err("Unsupported authentication format".into())
        }
    }
    
    /// Authenticate request and create appropriate signer
    async fn authenticate_request(
        &self,
        req: &HttpRequest,
    ) -> Result<Box<dyn TransactionSigner>, Box<dyn std::error::Error + Send + Sync>> {
        // Extract authentication data from request headers
        let auth_data = self.extract_auth_data(req)?;
        
        // Use factory to create appropriate signer
        let signer = self.signer_factory
            .create_signer(auth_data, &self.rpc_config)
            .await?;
        
        Ok(signer)
    }
    
    /// SSE handler using SignerFactory pattern
    pub async fn sse_handler<A>(
        &self,
        req: &HttpRequest,
        agent: &A,
        prompt: PromptRequest,
    ) -> ActixResult<HttpResponse>
    where
        A: Agent + Clone + Send + Sync + 'static,
        A::Error: std::fmt::Display + std::fmt::Debug + Send + Sync + 'static,
    {
        tracing::info!(
            prompt_len = prompt.text.len(),
            conversation_id = ?prompt.conversation_id,
            request_id = ?prompt.request_id,
            "Processing SSE request with SignerFactory"
        );

        // Extract authentication data and create signer
        let signer = match self.authenticate_request(req).await {
            Ok(s) => std::sync::Arc::<dyn riglr_core::TransactionSigner>::from(s),
            Err(e) => {
                tracing::error!(error = %e, "Authentication failed");
                return Ok(HttpResponse::Unauthorized().json(serde_json::json!({
                    "error": e.to_string(),
                    "code": "AUTHENTICATION_FAILED"
                })));
            }
        };
        
        // Handle stream using framework-agnostic core
        let stream_result = handle_agent_stream(
            agent.clone(),
            signer,
            prompt,
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
    
    /// Completion handler using SignerFactory pattern
    pub async fn completion_handler<A>(
        &self,
        req: &HttpRequest,
        agent: &A,
        prompt: PromptRequest,
    ) -> ActixResult<HttpResponse>
    where
        A: Agent + Clone + Send + Sync + 'static,
        A::Error: std::fmt::Display + std::fmt::Debug + Send + Sync + 'static,
    {
        tracing::info!(
            prompt_len = prompt.text.len(),
            conversation_id = ?prompt.conversation_id,
            request_id = ?prompt.request_id,
            "Processing completion request with SignerFactory"
        );

        // Extract authentication data and create signer
        let signer = match self.authenticate_request(req).await {
            Ok(s) => std::sync::Arc::<dyn riglr_core::TransactionSigner>::from(s),
            Err(e) => {
                tracing::error!(error = %e, "Authentication failed");
                return Ok(HttpResponse::Unauthorized().json(serde_json::json!({
                    "error": e.to_string(),
                    "code": "AUTHENTICATION_FAILED"
                })));
            }
        };
        
        // Handle completion using framework-agnostic core
        match handle_agent_completion(
            agent.clone(),
            signer,
            prompt,
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
    #[allow(unused_imports)]
    use riglr_solana_tools::signer::LocalSolanaSigner;
    #[allow(unused_imports)]
    use solana_sdk::signature::Keypair;
    #[allow(unused_imports)]
    use std::error::Error as StdError;

    // Mock agent for testing
    #[derive(Clone)]
    #[allow(dead_code)]
    struct MockAgent {
        response: String,
    }

    impl MockAgent {
        #[allow(dead_code)]
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

}