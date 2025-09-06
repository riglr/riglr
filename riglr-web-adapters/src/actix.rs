//! Actix Web adapter for riglr agents
//!
//! This module provides Actix-specific handlers that wrap the framework-agnostic
//! core handlers. It handles authentication via pluggable SignerFactory implementations,
//! request/response conversion, and SSE streaming in the Actix Web ecosystem.

use crate::core::Agent;
use crate::core::{handle_agent_completion, handle_agent_stream, PromptRequest};
use crate::factory::{AuthenticationData, SignerFactory};
use actix_web::{HttpRequest, HttpResponse, Result as ActixResult};
use futures_util::StreamExt;
use riglr_core::signer::UnifiedSigner;
use std::sync::Arc;

/// Actix Web adapter that uses SignerFactory for authentication
#[derive(Clone)]
pub struct ActixRiglrAdapter {
    signer_factory: Arc<dyn SignerFactory>,
}

impl ActixRiglrAdapter {
    /// Create a new Actix adapter with the given signer factory
    pub fn new(signer_factory: Arc<dyn SignerFactory>) -> Self {
        Self { signer_factory }
    }

    /// Detect authentication type from request headers.
    ///
    /// The detection logic is as follows:
    /// 1. If the `x-auth-type` header is present, its value is used.
    /// 2. If the header is absent and exactly one `SignerFactory` is registered in the
    ///    `CompositeSignerFactory`, that factory's auth type is used as the default.
    /// 3. If the header is absent and multiple factories are registered, an error is
    ///    returned, as the auth type is ambiguous.
    /// 4. If no factories are registered, an error is returned.
    fn detect_auth_type(
        &self,
        req: &HttpRequest,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        // First check for explicit auth type header
        if let Some(auth_type) = req.headers().get("x-auth-type") {
            return Ok(auth_type.to_str()?.to_string());
        }

        // Check how many auth providers are registered
        let supported_types = self.signer_factory.supported_auth_types();

        match supported_types.len() {
            0 => {
                // No providers configured
                Err("No authentication providers registered".into())
            }
            1 => {
                // Single provider - use as default
                Ok(supported_types[0].clone())
            }
            _ => {
                // Multiple providers - require explicit header
                Err(
                    "`x-auth-type` header is required when multiple auth providers are configured"
                        .into(),
                )
            }
        }
    }

    /// Extract authentication data from request headers
    fn extract_auth_data(
        &self,
        req: &HttpRequest,
    ) -> Result<AuthenticationData, Box<dyn std::error::Error + Send + Sync>> {
        let auth_header = req
            .headers()
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
                network: req
                    .headers()
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
    ) -> Result<Box<dyn UnifiedSigner>, Box<dyn std::error::Error + Send + Sync>> {
        // Extract authentication data from request headers
        let auth_data = self.extract_auth_data(req)?;

        // Use factory to create appropriate signer
        let signer = self.signer_factory.create_signer(auth_data).await?;

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
            Ok(s) => std::sync::Arc::<dyn riglr_core::signer::UnifiedSigner>::from(s),
            Err(e) => {
                tracing::error!(error = %e, "Authentication failed");
                return Ok(HttpResponse::Unauthorized().json(serde_json::json!({
                    "error": e.to_string(),
                    "code": "AUTHENTICATION_FAILED"
                })));
            }
        };

        // Handle stream using framework-agnostic core
        let stream_result = handle_agent_stream(agent.clone(), signer, prompt).await;

        match stream_result {
            Ok(stream) => {
                tracing::info!("Agent stream created successfully");

                // Convert to Actix SSE stream
                let sse_stream = stream.map(|chunk| match chunk {
                    Ok(data) => Ok(actix_web::web::Bytes::from(format!("data: {}\n\n", data))),
                    Err(e) => {
                        tracing::error!(error = %e, "Stream error");
                        Err(actix_web::error::ErrorInternalServerError(e))
                    }
                });

                Ok(HttpResponse::Ok()
                    .content_type("text/event-stream")
                    .insert_header(("Cache-Control", "no-cache"))
                    .insert_header(("Connection", "keep-alive"))
                    .insert_header(("Access-Control-Allow-Origin", "*"))
                    .insert_header((
                        "Access-Control-Allow-Headers",
                        "Cache-Control, Authorization",
                    ))
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
            Ok(s) => std::sync::Arc::<dyn riglr_core::signer::UnifiedSigner>::from(s),
            Err(e) => {
                tracing::error!(error = %e, "Authentication failed");
                return Ok(HttpResponse::Unauthorized().json(serde_json::json!({
                    "error": e.to_string(),
                    "code": "AUTHENTICATION_FAILED"
                })));
            }
        };

        // Handle completion using framework-agnostic core
        match handle_agent_completion(agent.clone(), signer, prompt).await {
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
    use riglr_core::signer::{
        EvmSigner, MultiChainSigner, SignerBase, SolanaSigner, UnifiedSigner,
    };
    #[allow(unused_imports)]
    use riglr_solana_tools::signer::LocalSolanaSigner;
    #[allow(unused_imports)]
    use solana_sdk::signature::Keypair;
    use std::any::Any;
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

        async fn prompt_stream(
            &self,
            _prompt: &str,
        ) -> Result<futures_util::stream::BoxStream<'_, Result<String, Self::Error>>, Self::Error>
        {
            let chunks = vec!["Hello", " ", "world"];
            let stream = futures_util::stream::iter(chunks).map(|chunk| Ok(chunk.to_string()));
            Ok(Box::pin(stream))
        }
    }

    // Mock signer for testing
    #[derive(Debug)]
    struct MockSigner {
        should_fail: bool,
    }

    impl MockSigner {
        fn new() -> Self {
            Self { should_fail: false }
        }

        #[allow(dead_code)]
        fn new_failing() -> Self {
            Self { should_fail: true }
        }
    }

    impl SignerBase for MockSigner {
        fn as_any(&self) -> &dyn Any {
            self
        }
    }

    #[async_trait::async_trait]
    impl SolanaSigner for MockSigner {
        fn address(&self) -> String {
            "mock_address".to_string()
        }

        fn pubkey(&self) -> String {
            "mock_public_key".to_string()
        }

        async fn sign_and_send_transaction(
            &self,
            _tx: &mut Vec<u8>,
        ) -> Result<String, riglr_core::signer::SignerError> {
            if self.should_fail {
                return Err(riglr_core::signer::SignerError::Signing(
                    "Mock Solana signing error".to_string(),
                ));
            }
            Ok("mock_solana_signature".to_string())
        }

        fn client(&self) -> std::sync::Arc<dyn std::any::Any + Send + Sync> {
            let client = solana_client::rpc_client::RpcClient::new("http://localhost:8899");
            std::sync::Arc::new(client)
        }
    }

    #[async_trait::async_trait]
    impl EvmSigner for MockSigner {
        fn chain_id(&self) -> u64 {
            1337
        }

        fn address(&self) -> String {
            "0xmock_evm_address".to_string()
        }

        async fn sign_and_send_transaction(
            &self,
            _tx: serde_json::Value,
        ) -> Result<String, riglr_core::signer::SignerError> {
            if self.should_fail {
                return Err(riglr_core::signer::SignerError::Signing(
                    "Mock EVM signing error".to_string(),
                ));
            }
            Ok("0xmock_evm_hash".to_string())
        }

        fn client(
            &self,
        ) -> Result<
            std::sync::Arc<dyn riglr_core::signer::traits::EvmClient>,
            riglr_core::signer::SignerError,
        > {
            Err(riglr_core::signer::SignerError::UnsupportedOperation(
                "Mock EVM client not available".to_string(),
            ))
        }
    }

    impl UnifiedSigner for MockSigner {
        fn supports_solana(&self) -> bool {
            true
        }

        fn supports_evm(&self) -> bool {
            true
        }

        fn as_solana(&self) -> Option<&dyn SolanaSigner> {
            Some(self)
        }

        fn as_evm(&self) -> Option<&dyn EvmSigner> {
            Some(self)
        }

        fn as_multi_chain(&self) -> Option<&dyn MultiChainSigner> {
            None
        }
    }

    #[actix_web::test]
    async fn test_health_handler() {
        let app =
            test::init_service(App::new().route("/health", web::get().to(health_handler))).await;

        let req = test::TestRequest::get().uri("/health").to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let body = test::read_body(resp).await;
        let health_response: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(health_response["status"], "healthy");
        assert!(health_response["timestamp"].is_string());
    }

    #[actix_web::test]
    async fn test_info_handler() {
        let app = test::init_service(App::new().route("/", web::get().to(info_handler))).await;

        let req = test::TestRequest::get().uri("/").to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let body = test::read_body(resp).await;
        let info_response: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(info_response["service"], "riglr-web-adapters");
        assert!(info_response["version"].is_string());
        assert!(info_response["endpoints"].is_array());
    }

    // Mock SignerFactory for testing
    #[derive(Clone)]
    struct MockSignerFactory {
        supported_types: Vec<String>,
        should_fail_create_signer: bool,
    }

    impl MockSignerFactory {
        fn new() -> Self {
            Self {
                supported_types: vec!["privy".to_string()],
                should_fail_create_signer: false,
            }
        }

        fn with_no_auth_types() -> Self {
            Self {
                supported_types: vec![],
                should_fail_create_signer: false,
            }
        }

        fn with_signer_creation_failure() -> Self {
            Self {
                supported_types: vec!["privy".to_string()],
                should_fail_create_signer: true,
            }
        }

        fn with_multiple_auth_types() -> Self {
            Self {
                supported_types: vec![
                    "privy".to_string(),
                    "web3auth".to_string(),
                    "magic".to_string(),
                ],
                should_fail_create_signer: false,
            }
        }
    }

    #[async_trait::async_trait]
    impl SignerFactory for MockSignerFactory {
        fn supported_auth_types(&self) -> Vec<String> {
            self.supported_types.clone()
        }

        async fn create_signer(
            &self,
            _auth_data: AuthenticationData,
        ) -> Result<Box<dyn UnifiedSigner>, Box<dyn StdError + Send + Sync>> {
            if self.should_fail_create_signer {
                return Err("Failed to create signer".into());
            }

            Ok(Box::new(MockSigner::new()))
        }
    }

    #[test]
    async fn test_actix_riglr_adapter_new() {
        let signer_factory = Arc::new(MockSignerFactory::new());

        let adapter = ActixRiglrAdapter::new(signer_factory.clone());

        // Just verify the adapter was created successfully
        assert_eq!(
            adapter.signer_factory.supported_auth_types(),
            vec!["privy".to_string()]
        );
    }

    #[test]
    async fn test_detect_auth_type_with_explicit_header() {
        let signer_factory = Arc::new(MockSignerFactory::new());
        let adapter = ActixRiglrAdapter::new(signer_factory);

        let req = test::TestRequest::default()
            .insert_header(("x-auth-type", "custom"))
            .to_http_request();

        let auth_type = adapter.detect_auth_type(&req).unwrap();
        assert_eq!(auth_type, "custom");
    }

    #[test]
    async fn test_detect_auth_type_fallback_to_first_supported() {
        let signer_factory = Arc::new(MockSignerFactory::new());
        let adapter = ActixRiglrAdapter::new(signer_factory);

        let req = test::TestRequest::default().to_http_request();

        let auth_type = adapter.detect_auth_type(&req).unwrap();
        assert_eq!(auth_type, "privy");
    }

    #[test]
    async fn test_detect_auth_type_no_providers_registered() {
        let signer_factory = Arc::new(MockSignerFactory::with_no_auth_types());
        let adapter = ActixRiglrAdapter::new(signer_factory);

        let req = test::TestRequest::default().to_http_request();

        let result = adapter.detect_auth_type(&req);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "No authentication providers registered"
        );
    }

    #[test]
    async fn test_detect_auth_type_invalid_header_value() {
        let signer_factory = Arc::new(MockSignerFactory::new());
        let adapter = ActixRiglrAdapter::new(signer_factory);

        let req = test::TestRequest::default()
            .insert_header(("x-auth-type", "invalid-type"))
            .to_http_request();

        let result = adapter.detect_auth_type(&req);
        assert!(result.is_ok());
    }

    #[test]
    async fn test_detect_auth_type_multiple_providers_missing_header_should_fail() {
        let signer_factory = Arc::new(MockSignerFactory::with_multiple_auth_types());
        let adapter = ActixRiglrAdapter::new(signer_factory);
        let req = test::TestRequest::default().to_http_request();
        let result = adapter.detect_auth_type(&req);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "`x-auth-type` header is required when multiple auth providers are configured"
        );
    }

    #[test]
    async fn test_detect_auth_type_single_provider_missing_header_should_succeed() {
        let signer_factory = Arc::new(MockSignerFactory::new());
        let adapter = ActixRiglrAdapter::new(signer_factory);
        let req = test::TestRequest::default().to_http_request();
        let auth_type = adapter.detect_auth_type(&req).unwrap();
        assert_eq!(auth_type, "privy");
    }

    #[test]
    async fn test_extract_auth_data_success_with_bearer() {
        let signer_factory = Arc::new(MockSignerFactory::new());
        let adapter = ActixRiglrAdapter::new(signer_factory);

        let req = test::TestRequest::default()
            .insert_header(("authorization", "Bearer test_token"))
            .insert_header(("x-network", "testnet"))
            .to_http_request();

        let auth_data = adapter.extract_auth_data(&req).unwrap();
        assert_eq!(auth_data.auth_type, "privy");
        assert_eq!(auth_data.credentials.get("token").unwrap(), "test_token");
        assert_eq!(auth_data.network, "testnet");
    }

    #[test]
    async fn test_extract_auth_data_success_default_network() {
        let signer_factory = Arc::new(MockSignerFactory::new());
        let adapter = ActixRiglrAdapter::new(signer_factory);

        let req = test::TestRequest::default()
            .insert_header(("authorization", "Bearer test_token"))
            .to_http_request();

        let auth_data = adapter.extract_auth_data(&req).unwrap();
        assert_eq!(auth_data.network, "mainnet");
    }

    #[test]
    async fn test_extract_auth_data_missing_authorization_header() {
        let signer_factory = Arc::new(MockSignerFactory::new());
        let adapter = ActixRiglrAdapter::new(signer_factory);

        let req = test::TestRequest::default().to_http_request();

        let result = adapter.extract_auth_data(&req);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Missing authorization header"
        );
    }

    #[test]
    async fn test_extract_auth_data_unsupported_auth_format() {
        let signer_factory = Arc::new(MockSignerFactory::new());
        let adapter = ActixRiglrAdapter::new(signer_factory);

        let req = test::TestRequest::default()
            .insert_header(("authorization", "Basic dGVzdDp0ZXN0"))
            .to_http_request();

        let result = adapter.extract_auth_data(&req);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Unsupported authentication format"
        );
    }

    #[test]
    async fn test_extract_auth_data_invalid_header_value() {
        let signer_factory = Arc::new(MockSignerFactory::new());
        let adapter = ActixRiglrAdapter::new(signer_factory);

        let req = test::TestRequest::default()
            .insert_header(("authorization", "Invalid-Auth-Header"))
            .to_http_request();

        let result = adapter.extract_auth_data(&req);
        assert!(result.is_err());
    }

    #[test]
    async fn test_extract_auth_data_invalid_network_header() {
        let signer_factory = Arc::new(MockSignerFactory::new());
        let adapter = ActixRiglrAdapter::new(signer_factory);

        let req = test::TestRequest::default()
            .insert_header(("authorization", "Bearer test_token"))
            .insert_header(("x-network", "invalid-network"))
            .to_http_request();

        let auth_data = adapter.extract_auth_data(&req).unwrap();
        // Should use the provided network value
        assert_eq!(auth_data.network, "invalid-network");
    }

    #[actix_web::test]
    async fn test_authenticate_request_success() {
        let signer_factory = Arc::new(MockSignerFactory::new());
        let adapter = ActixRiglrAdapter::new(signer_factory);

        let req = test::TestRequest::default()
            .insert_header(("authorization", "Bearer test_token"))
            .to_http_request();

        let result = adapter.authenticate_request(&req).await;
        assert!(result.is_ok());
    }

    #[actix_web::test]
    async fn test_authenticate_request_extract_auth_data_failure() {
        let signer_factory = Arc::new(MockSignerFactory::new());
        let adapter = ActixRiglrAdapter::new(signer_factory);

        let req = test::TestRequest::default().to_http_request();

        let result = adapter.authenticate_request(&req).await;
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Missing authorization header"
        );
    }

    #[actix_web::test]
    async fn test_authenticate_request_signer_creation_failure() {
        let signer_factory = Arc::new(MockSignerFactory::with_signer_creation_failure());
        let adapter = ActixRiglrAdapter::new(signer_factory);

        let req = test::TestRequest::default()
            .insert_header(("authorization", "Bearer test_token"))
            .to_http_request();

        let result = adapter.authenticate_request(&req).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Failed to create signer");
    }

    #[actix_web::test]
    async fn test_sse_handler_authentication_failure() {
        let signer_factory = Arc::new(MockSignerFactory::new());
        let adapter = ActixRiglrAdapter::new(signer_factory);
        let agent = MockAgent::new("test response".to_string());
        let prompt = PromptRequest {
            text: "test prompt".to_string(),
            conversation_id: Some("conv-123".to_string()),
            request_id: Some("req-456".to_string()),
        };

        let req = test::TestRequest::default().to_http_request();

        let result = adapter.sse_handler(&req, &agent, prompt).await.unwrap();
        assert_eq!(result.status(), actix_web::http::StatusCode::UNAUTHORIZED);
    }

    #[actix_web::test]
    async fn test_sse_handler_success() {
        let signer_factory = Arc::new(MockSignerFactory::new());
        let adapter = ActixRiglrAdapter::new(signer_factory);
        let agent = MockAgent::new("test response".to_string());
        let prompt = PromptRequest {
            text: "test prompt".to_string(),
            conversation_id: Some("conv-123".to_string()),
            request_id: Some("req-456".to_string()),
        };

        let req = test::TestRequest::default()
            .insert_header(("authorization", "Bearer test_token"))
            .to_http_request();

        let result = adapter.sse_handler(&req, &agent, prompt).await.unwrap();
        assert_eq!(result.status(), actix_web::http::StatusCode::OK);
        assert_eq!(
            result.headers().get("content-type").unwrap(),
            "text/event-stream"
        );
        assert_eq!(result.headers().get("cache-control").unwrap(), "no-cache");
        assert_eq!(result.headers().get("connection").unwrap(), "keep-alive");
    }

    #[actix_web::test]
    async fn test_completion_handler_authentication_failure() {
        let signer_factory = Arc::new(MockSignerFactory::new());
        let adapter = ActixRiglrAdapter::new(signer_factory);
        let agent = MockAgent::new("test response".to_string());
        let prompt = PromptRequest {
            text: "test prompt".to_string(),
            conversation_id: Some("conv-123".to_string()),
            request_id: Some("req-456".to_string()),
        };

        let req = test::TestRequest::default().to_http_request();

        let result = adapter
            .completion_handler(&req, &agent, prompt)
            .await
            .unwrap();
        assert_eq!(result.status(), actix_web::http::StatusCode::UNAUTHORIZED);
    }

    #[actix_web::test]
    async fn test_completion_handler_success() {
        let signer_factory = Arc::new(MockSignerFactory::new());
        let adapter = ActixRiglrAdapter::new(signer_factory);
        let agent = MockAgent::new("test response".to_string());
        let prompt = PromptRequest {
            text: "test prompt".to_string(),
            conversation_id: Some("conv-123".to_string()),
            request_id: Some("req-456".to_string()),
        };

        let req = test::TestRequest::default()
            .insert_header(("authorization", "Bearer test_token"))
            .to_http_request();

        let result = adapter
            .completion_handler(&req, &agent, prompt)
            .await
            .unwrap();
        assert_eq!(result.status(), actix_web::http::StatusCode::OK);
    }

    // Test for failing agent to trigger stream error path
    #[derive(Clone)]
    struct FailingMockAgent;

    #[async_trait::async_trait]
    impl Agent for FailingMockAgent {
        type Error = std::io::Error;

        async fn prompt(&self, _prompt: &str) -> Result<String, Self::Error> {
            Err(std::io::Error::other("Agent failed"))
        }

        async fn prompt_stream(
            &self,
            _prompt: &str,
        ) -> Result<futures_util::stream::BoxStream<'_, Result<String, Self::Error>>, Self::Error>
        {
            Err(std::io::Error::other("Stream failed"))
        }
    }

    #[actix_web::test]
    async fn test_sse_handler_agent_stream_error() {
        let signer_factory = Arc::new(MockSignerFactory::new());
        let adapter = ActixRiglrAdapter::new(signer_factory);
        let agent = FailingMockAgent;
        let prompt = PromptRequest {
            text: "test prompt".to_string(),
            conversation_id: Some("conv-123".to_string()),
            request_id: Some("req-456".to_string()),
        };

        let req = test::TestRequest::default()
            .insert_header(("authorization", "Bearer test_token"))
            .to_http_request();

        let result = adapter.sse_handler(&req, &agent, prompt).await.unwrap();
        assert_eq!(result.status(), actix_web::http::StatusCode::OK);
    }

    #[actix_web::test]
    async fn test_completion_handler_agent_error() {
        let signer_factory = Arc::new(MockSignerFactory::new());
        let adapter = ActixRiglrAdapter::new(signer_factory);
        let agent = FailingMockAgent;
        let prompt = PromptRequest {
            text: "test prompt".to_string(),
            conversation_id: Some("conv-123".to_string()),
            request_id: Some("req-456".to_string()),
        };

        let req = test::TestRequest::default()
            .insert_header(("authorization", "Bearer test_token"))
            .to_http_request();

        let result = adapter
            .completion_handler(&req, &agent, prompt)
            .await
            .unwrap();
        assert_eq!(
            result.status(),
            actix_web::http::StatusCode::INTERNAL_SERVER_ERROR
        );
    }
}
