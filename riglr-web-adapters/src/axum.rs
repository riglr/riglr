//! Axum adapter for riglr agents
//!
//! This module provides Axum-specific handlers that wrap the framework-agnostic
//! core handlers. It handles authentication via pluggable SignerFactory implementations.

use axum::{
    extract::{Request, State},
    http::{StatusCode, HeaderMap},
    response::{Response, Sse, sse::Event},
    Json,
};
use futures_util::StreamExt;
use crate::core::Agent;
use crate::factory::{SignerFactory, AuthenticationData};
use riglr_core::config::RpcConfig;
use riglr_core::signer::TransactionSigner;
use crate::core::{handle_agent_stream, handle_agent_completion, PromptRequest, CompletionResponse};
use std::sync::Arc;

/// Axum adapter that uses SignerFactory for authentication
pub struct AxumRiglrAdapter {
    signer_factory: Arc<dyn SignerFactory>,
    rpc_config: RpcConfig,
}

impl AxumRiglrAdapter {
    /// Create a new Axum adapter with the given signer factory and RPC config
    pub fn new(signer_factory: Arc<dyn SignerFactory>, rpc_config: RpcConfig) -> Self {
        Self {
            signer_factory,
            rpc_config,
        }
    }
    
    /// Extract authentication data from request headers
    fn extract_auth_data(&self, headers: &HeaderMap) -> Result<AuthenticationData, StatusCode> {
        let auth_header = headers
            .get("authorization")
            .and_then(|h| h.to_str().ok())
            .ok_or_else(|| {
                tracing::warn!("Missing Authorization header");
                StatusCode::UNAUTHORIZED
            })?;
        
        // Parse auth header to determine type and extract credentials
        if auth_header.starts_with("Bearer ") {
            let token = auth_header.strip_prefix("Bearer ").unwrap();
            
            Ok(AuthenticationData {
                auth_type: "privy".to_string(), // Could be detected from token format
                credentials: [("token".to_string(), token.to_string())].into(),
                network: headers
                    .get("x-network")
                    .and_then(|h| h.to_str().ok())
                    .unwrap_or("mainnet")
                    .to_string(),
            })
        } else {
            tracing::warn!("Invalid Authorization header format");
            Err(StatusCode::UNAUTHORIZED)
        }
    }
    
    /// Authenticate request and create appropriate signer
    async fn authenticate_request(
        &self,
        headers: &HeaderMap,
    ) -> Result<Box<dyn TransactionSigner>, StatusCode> {
        // Extract authentication data from request headers
        let auth_data = self.extract_auth_data(headers)?;
        
        // Use factory to create appropriate signer
        let signer = self.signer_factory
            .create_signer(auth_data, &self.rpc_config)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        
        Ok(signer)
    }
    
    /// SSE handler using SignerFactory pattern
    pub async fn sse_handler<A>(
        &self,
        headers: HeaderMap,
        agent: A,
        prompt: PromptRequest,
    ) -> Result<Sse<impl futures_util::Stream<Item = Result<Event, axum::Error>>>, StatusCode>
    where
        A: Agent + Clone + Send + Sync + 'static,
        A::Error: std::fmt::Display + std::fmt::Debug + Send + Sync + 'static,
    {
        tracing::info!(
            prompt_len = prompt.text.len(),
            conversation_id = ?prompt.conversation_id,
            request_id = ?prompt.request_id,
            "Processing Axum SSE request with SignerFactory"
        );

        // Extract authentication data and create signer
        let signer = Arc::new(self.authenticate_request(&headers).await?);
        
        // Handle stream using framework-agnostic core
        let stream_result = handle_agent_stream(agent, signer, prompt).await;
        
        match stream_result {
            Ok(stream) => {
                tracing::info!("Agent stream created successfully");
                
                // Convert to Axum SSE stream
                let sse_stream = stream.map(|chunk| {
                    match chunk {
                        Ok(data) => Ok(Event::default().data(data)),
                        Err(e) => {
                            tracing::error!(error = %e, "Stream error");
                            Err(axum::Error::new(e))
                        }
                    }
                });
                
                Ok(Sse::new(sse_stream))
            }
            Err(e) => {
                tracing::error!(error = %e, "Failed to create agent stream");
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }
    
    /// Completion handler using SignerFactory pattern
    pub async fn completion_handler<A>(
        &self,
        headers: HeaderMap,
        agent: A,
        prompt: PromptRequest,
    ) -> Result<Json<CompletionResponse>, StatusCode>
    where
        A: Agent + Clone + Send + Sync + 'static,
        A::Error: std::fmt::Display + std::fmt::Debug + Send + Sync + 'static,
    {
        tracing::info!(
            prompt_len = prompt.text.len(),
            conversation_id = ?prompt.conversation_id,
            request_id = ?prompt.request_id,
            "Processing Axum completion request with SignerFactory"
        );

        // Extract authentication data and create signer
        let signer = Arc::new(self.authenticate_request(&headers).await?);
        
        // Handle completion using framework-agnostic core
        match handle_agent_completion(agent, signer, prompt).await {
            Ok(response) => {
                tracing::info!(
                    conversation_id = %response.conversation_id,
                    request_id = %response.request_id,
                    response_len = response.response.len(),
                    "Completion request processed successfully"
                );
                Ok(Json(response))
            }
            Err(e) => {
                tracing::error!(error = %e, "Failed to process completion");
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }
}


/// Health check handler for Axum
pub async fn health_handler() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "service": "riglr-web-adapters",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

/// Information handler for Axum
pub async fn info_handler() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "service": "riglr-web-adapters",
        "version": env!("CARGO_PKG_VERSION"),
        "description": "Framework-agnostic web adapters for riglr agents",
        "framework": "axum",
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
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{routing::get, Router};
    use tower::ServiceExt;
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

    #[tokio::test]
    async fn test_health_handler() {
        let app = Router::new().route("/health", get(health_handler));

        let response = app
            .oneshot(
                axum::http::Request::builder()
                    .uri("/health")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_info_handler() {
        let app = Router::new().route("/", get(info_handler));

        let response = app
            .oneshot(
                axum::http::Request::builder()
                    .uri("/")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

}