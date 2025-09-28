//! Actix Web adapter for riglr agents
//!
//! This module provides Actix-specific handlers that wrap the framework-agnostic
//! core handlers. It handles authentication via pluggable `SignerFactory` implementations,
//! request/response conversion, and SSE streaming in the Actix Web ecosystem.

use crate::core::Agent;
use crate::core::{
    handle_agent_completion, handle_agent_stream, CompletionResponse, PromptRequest,
};
use crate::factory::{AuthenticationData, SignerFactory};
use actix_web::{error, web, HttpRequest, HttpResponse, Result as ActixResult};
use core::{error::Error as StdError, fmt};
use futures_util::{stream, StreamExt};
use riglr_core::signer::UnifiedSigner;
use std::sync::Arc;

/// Actix Web adapter that uses `SignerFactory` for authentication
#[derive(Clone)]
#[expect(clippy::module_name_repetitions)]
pub struct ActixRiglrAdapter {
    signer_factory: Arc<dyn SignerFactory>,
}

impl fmt::Debug for ActixRiglrAdapter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ActixRiglrAdapter")
            .field("signer_factory", &"Arc<dyn SignerFactory>")
            .finish()
    }
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
    ) -> Result<String, Box<dyn StdError + Send + Sync>> {
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
                supported_types.into_iter().next().map_or_else(
                    || Err("Unexpected error: vector reported length 1 but was empty".into()),
                    Ok,
                )
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
    ) -> Result<AuthenticationData, Box<dyn StdError + Send + Sync>> {
        let auth_header = req
            .headers()
            .get("authorization")
            .ok_or("Missing authorization header")?
            .to_str()?;

        // Parse auth header to determine type and extract credentials
        if auth_header.starts_with("Bearer ") {
            let token = auth_header
                .strip_prefix("Bearer ")
                .ok_or("Invalid Bearer token format")?;
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
    #[expect(clippy::future_not_send)]
    async fn authenticate_request(
        &self,
        req: &HttpRequest,
    ) -> Result<Box<dyn UnifiedSigner>, Box<dyn StdError + Send + Sync>> {
        // Extract authentication data from request headers
        let auth_data = self.extract_auth_data(req)?;

        // Use factory to create appropriate signer
        let signer = self.signer_factory.create_signer(auth_data).await?;

        Ok(signer)
    }

    /// Create authentication error response
    fn create_auth_error_response(error: &dyn StdError) -> HttpResponse {
        tracing::error!(error = %error, "Authentication failed");
        HttpResponse::Unauthorized().json(serde_json::json!({
            "error": error.to_string(),
            "code": "AUTHENTICATION_FAILED"
        }))
    }

    /// Create stream error response
    fn create_stream_error_response(error: &dyn StdError) -> HttpResponse {
        tracing::error!(error = %error, "Failed to create agent stream");
        HttpResponse::InternalServerError().json(serde_json::json!({
            "error": error.to_string(),
            "code": "AGENT_STREAM_ERROR"
        }))
    }

    /// Create completion error response
    fn create_completion_error_response(error: &dyn StdError) -> HttpResponse {
        tracing::error!(error = %error, "Failed to process completion");
        HttpResponse::InternalServerError().json(serde_json::json!({
            "error": error.to_string(),
            "code": "AGENT_COMPLETION_ERROR"
        }))
    }

    /// Configure SSE response headers
    fn configure_sse_headers(
        mut response: actix_web::HttpResponseBuilder,
    ) -> actix_web::HttpResponseBuilder {
        response
            .content_type("text/event-stream")
            .insert_header(("Cache-Control", "no-cache"))
            .insert_header(("Connection", "keep-alive"))
            .insert_header(("Access-Control-Allow-Origin", "*"))
            .insert_header((
                "Access-Control-Allow-Headers",
                "Cache-Control, Authorization",
            ));
        response
    }

    /// Convert agent stream to Actix SSE stream
    fn convert_to_sse_stream<S, E>(
        stream: S,
    ) -> impl stream::Stream<Item = Result<web::Bytes, error::Error>>
    where
        S: stream::Stream<Item = Result<String, E>>,
        E: fmt::Display + fmt::Debug + 'static,
    {
        stream.map(|chunk| match chunk {
            Ok(data) => Ok(web::Bytes::from(format!("data: {data}\n\n"))),
            Err(e) => {
                tracing::error!(error = %e, "Stream error");
                Err(error::ErrorInternalServerError(e))
            }
        })
    }

    /// SSE handler using `SignerFactory` pattern
    ///
    /// # Errors
    /// Returns an error if:
    /// - Authentication fails due to missing or invalid headers
    /// - Signer creation fails during authentication
    /// - Agent stream creation fails
    /// - HTTP response building fails
    #[expect(clippy::future_not_send)]
    pub async fn sse_handler<A>(
        &self,
        req: &HttpRequest,
        agent: &A,
        prompt: PromptRequest,
    ) -> ActixResult<HttpResponse>
    where
        A: Agent + Clone + Send + Sync + 'static,
        A::Error: fmt::Display + fmt::Debug + Send + Sync + 'static,
    {
        tracing::info!(
            prompt_len = prompt.text.len(),
            conversation_id = ?prompt.conversation_id,
            request_id = ?prompt.request_id,
            "Processing SSE request with SignerFactory"
        );

        // Extract authentication data and create signer
        let signer = match self.authenticate_request(req).await {
            Ok(s) => Arc::<dyn UnifiedSigner>::from(s),
            Err(e) => return Ok(Self::create_auth_error_response(&*e)),
        };

        // Handle stream using framework-agnostic core
        let stream_result = handle_agent_stream(agent.clone(), signer, prompt).await;

        match stream_result {
            Ok(stream) => {
                tracing::info!("Agent stream created successfully");
                let sse_stream = Self::convert_to_sse_stream(stream);
                let response =
                    Self::configure_sse_headers(HttpResponse::Ok()).streaming(sse_stream);
                Ok(response)
            }
            Err(e) => Ok(Self::create_stream_error_response(&*e)),
        }
    }

    /// Log successful completion response
    fn log_completion_success(response: &CompletionResponse) {
        tracing::info!(
            conversation_id = %response.conversation_id,
            request_id = %response.request_id,
            response_len = response.response.len(),
            "Completion request processed successfully"
        );
    }

    /// Completion handler using `SignerFactory` pattern
    ///
    /// # Errors
    /// Returns an error if:
    /// - Authentication fails due to missing or invalid headers
    /// - Signer creation fails during authentication
    /// - Agent completion processing fails
    /// - HTTP response building fails
    #[expect(clippy::future_not_send)]
    pub async fn completion_handler<A>(
        &self,
        req: &HttpRequest,
        agent: &A,
        prompt: PromptRequest,
    ) -> ActixResult<HttpResponse>
    where
        A: Agent + Clone + Send + Sync + 'static,
        A::Error: fmt::Display + fmt::Debug + Send + Sync + 'static,
    {
        tracing::info!(
            prompt_len = prompt.text.len(),
            conversation_id = ?prompt.conversation_id,
            request_id = ?prompt.request_id,
            "Processing completion request with SignerFactory"
        );

        // Extract authentication data and create signer
        let signer = match self.authenticate_request(req).await {
            Ok(s) => Arc::<dyn UnifiedSigner>::from(s),
            Err(e) => return Ok(Self::create_auth_error_response(&*e)),
        };

        // Handle completion using framework-agnostic core
        match handle_agent_completion(agent.clone(), signer, prompt).await {
            Ok(response) => {
                Self::log_completion_success(&response);
                Ok(HttpResponse::Ok().json(response))
            }
            Err(e) => Ok(Self::create_completion_error_response(&*e)),
        }
    }
}

/// Health check handler
///
/// # Errors
///
/// This handler currently never fails but returns a Result for API consistency
#[allow(clippy::allow_attributes, clippy::unused_async)] // Framework compatibility: async required for handler signature
pub async fn health_handler() -> ActixResult<HttpResponse> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "service": "riglr-web-adapters",
        "version": env!("CARGO_PKG_VERSION")
    })))
}

/// Information handler
///
/// # Errors
///
/// This handler currently never fails but returns a Result for API consistency
#[allow(clippy::allow_attributes, clippy::unused_async)] // Framework compatibility: async required for handler signature
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
#[expect(clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use actix_web::http::StatusCode;
    use actix_web::{test, web, App};
    use core::error::Error as StdError;
    use riglr_core::signer::{
        error::Standard as SignerStandard, Chain, EvmClient, EvmSigner, SignerBase, SignerError,
        SolanaClient, SolanaSigner, UnifiedSigner,
    };
    use std::io;

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
            let chunks = vec!["Hello", " ", "world"];
            let stream = stream::iter(chunks).map(|chunk| Ok(chunk.to_string()));
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

        #[expect(
            dead_code,
            reason = "Mock utility for testing - may be used in future test scenarios"
        )]
        fn new_failing() -> Self {
            Self { should_fail: true }
        }
    }

    #[async_trait::async_trait]
    impl SignerBase for MockSigner {
        fn supported_chains(&self) -> &[Chain] {
            &[Chain::Solana, Chain::Evm]
        }

        fn user_id(&self) -> String {
            "mock_user_123".to_string()
        }
    }

    #[async_trait::async_trait]
    impl SolanaSigner for MockSigner {
        fn pubkey(&self) -> String {
            "mock_public_key".to_string()
        }

        fn client(&self) -> &dyn SolanaClient {
            // This is a mock implementation - in real code this would return a proper client
            panic!("MockSigner client() should not be called in tests")
        }

        async fn sign_message(&self, _message: &[u8]) -> Result<String, Box<dyn SignerError>> {
            if self.should_fail {
                return Err(Box::new(SignerStandard::SigningFailed(
                    "Mock Solana message signing error".to_string(),
                )));
            }
            Ok("mock_solana_message_signature".to_string())
        }

        async fn sign_and_send_transaction(
            &self,
            _transaction: serde_json::Value,
        ) -> Result<String, Box<dyn SignerError>> {
            if self.should_fail {
                return Err(Box::new(SignerStandard::SigningFailed(
                    "Mock Solana signing error".to_string(),
                )));
            }
            Ok("mock_solana_signature".to_string())
        }
    }

    #[async_trait::async_trait]
    impl EvmSigner for MockSigner {
        fn address(&self) -> String {
            "0xmock_evm_address".to_string()
        }

        fn chain_id(&self) -> u64 {
            1337
        }

        fn client(&self) -> &dyn EvmClient {
            // This is a mock implementation - in real code this would return a proper client
            panic!("MockSigner client() should not be called in tests")
        }

        async fn sign_message(&self, _message: &[u8]) -> Result<String, Box<dyn SignerError>> {
            if self.should_fail {
                return Err(Box::new(SignerStandard::SigningFailed(
                    "Mock EVM message signing error".to_string(),
                )));
            }
            Ok("0xmock_evm_message_signature".to_string())
        }

        async fn sign_and_send_transaction(
            &self,
            _transaction_request: serde_json::Value,
        ) -> Result<String, Box<dyn SignerError>> {
            if self.should_fail {
                return Err(Box::new(SignerStandard::SigningFailed(
                    "Mock EVM signing error".to_string(),
                )));
            }
            Ok("0xmock_evm_hash".to_string())
        }
    }

    impl UnifiedSigner for MockSigner {
        fn as_solana(&self) -> Option<&dyn SolanaSigner> {
            Some(self)
        }

        fn as_evm(&self) -> Option<&dyn EvmSigner> {
            Some(self)
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
        let health_response: serde_json::Value =
            serde_json::from_slice(&body).expect("Failed to parse health response JSON");

        assert_eq!(
            health_response
                .get("status")
                .expect("status field should exist"),
            "healthy"
        );
        assert!(health_response
            .get("timestamp")
            .expect("timestamp field should exist")
            .is_string());
    }

    #[actix_web::test]
    async fn test_info_handler() {
        let app = test::init_service(App::new().route("/", web::get().to(info_handler))).await;

        let req = test::TestRequest::get().uri("/").to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let body = test::read_body(resp).await;
        let info_response: serde_json::Value =
            serde_json::from_slice(&body).expect("Failed to parse info response JSON");

        assert_eq!(
            info_response
                .get("service")
                .expect("service field should exist"),
            "riglr-web-adapters"
        );
        assert!(info_response
            .get("version")
            .expect("version field should exist")
            .is_string());
        assert!(info_response
            .get("endpoints")
            .expect("endpoints field should exist")
            .is_array());
    }

    // Mock SignerFactory for testing
    #[derive(Clone, Debug)]
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

        let adapter = ActixRiglrAdapter::new(signer_factory);

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

        let auth_type = adapter
            .detect_auth_type(&req)
            .expect("Failed to detect auth type with explicit header");
        assert_eq!(auth_type, "custom");
    }

    #[test]
    async fn test_detect_auth_type_fallback_to_first_supported() {
        let signer_factory = Arc::new(MockSignerFactory::new());
        let adapter = ActixRiglrAdapter::new(signer_factory);

        let req = test::TestRequest::default().to_http_request();

        let auth_type = adapter
            .detect_auth_type(&req)
            .expect("Failed to detect auth type with single provider");
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
            result
                .expect_err("Expected error for no auth providers")
                .to_string(),
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
            result
                .expect_err("Expected error for multiple providers without header")
                .to_string(),
            "`x-auth-type` header is required when multiple auth providers are configured"
        );
    }

    #[test]
    async fn test_detect_auth_type_single_provider_missing_header_should_succeed() {
        let signer_factory = Arc::new(MockSignerFactory::new());
        let adapter = ActixRiglrAdapter::new(signer_factory);
        let req = test::TestRequest::default().to_http_request();
        let auth_type = adapter
            .detect_auth_type(&req)
            .expect("Failed to detect auth type with single provider");
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

        let auth_data = adapter
            .extract_auth_data(&req)
            .expect("Failed to extract auth data with bearer token");
        assert_eq!(auth_data.auth_type, "privy");
        assert_eq!(
            auth_data
                .credentials
                .get("token")
                .expect("Token should be present"),
            "test_token"
        );
        assert_eq!(auth_data.network, "testnet");
    }

    #[test]
    async fn test_extract_auth_data_success_default_network() {
        let signer_factory = Arc::new(MockSignerFactory::new());
        let adapter = ActixRiglrAdapter::new(signer_factory);

        let req = test::TestRequest::default()
            .insert_header(("authorization", "Bearer test_token"))
            .to_http_request();

        let auth_data = adapter
            .extract_auth_data(&req)
            .expect("Failed to extract auth data with default network");
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
            result
                .expect_err("Expected error for missing authorization header in extract_auth_data")
                .to_string(),
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
            result
                .expect_err("Expected error for unsupported auth format")
                .to_string(),
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

        let auth_data = adapter
            .extract_auth_data(&req)
            .expect("Failed to extract auth data with invalid network header");
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
            result
                .expect_err(
                    "Expected error for missing authorization header in authenticate_request"
                )
                .to_string(),
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
        assert_eq!(
            result
                .expect_err("Expected signer creation failure")
                .to_string(),
            "Failed to create signer"
        );
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

        let result = adapter
            .sse_handler(&req, &agent, prompt)
            .await
            .expect("SSE handler should handle auth failure");
        assert_eq!(result.status(), StatusCode::UNAUTHORIZED);
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

        let result = adapter
            .sse_handler(&req, &agent, prompt)
            .await
            .expect("SSE handler should succeed");
        assert_eq!(result.status(), StatusCode::OK);
        assert_eq!(
            result
                .headers()
                .get("content-type")
                .expect("Content-Type header should be present"),
            "text/event-stream"
        );
        assert_eq!(
            result
                .headers()
                .get("cache-control")
                .expect("Cache-Control header should be present"),
            "no-cache"
        );
        assert_eq!(
            result
                .headers()
                .get("connection")
                .expect("Connection header should be present"),
            "keep-alive"
        );
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
            .expect("Completion handler should handle auth failure");
        assert_eq!(result.status(), StatusCode::UNAUTHORIZED);
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
            .expect("Completion handler should succeed");
        assert_eq!(result.status(), StatusCode::OK);
    }

    // Test for failing agent to trigger stream error path
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

        let result = adapter
            .sse_handler(&req, &agent, prompt)
            .await
            .expect("SSE handler should handle agent stream error");
        assert_eq!(result.status(), StatusCode::OK);
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
            .expect("Completion handler should handle agent error");
        assert_eq!(result.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }
}
