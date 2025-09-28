//! Axum adapter for riglr agents
//!
//! This module provides Axum-specific handlers that wrap the framework-agnostic
//! core handlers. It handles authentication via pluggable `SignerFactory` implementations.

use crate::core::Agent;
use crate::core::{
    handle_agent_completion, handle_agent_stream, CompletionResponse, PromptRequest,
};
use crate::factory::{AuthenticationData, SignerFactory};
use axum::{
    http::{HeaderMap, StatusCode},
    response::{sse::Event, Sse},
    Json,
};
use core::fmt;
use futures_util::StreamExt;
use riglr_core::signer::UnifiedSigner;
use std::{string::ToString, sync::Arc};

/// Axum adapter that uses `SignerFactory` for authentication
#[derive(Clone)]
#[expect(clippy::module_name_repetitions)]
pub struct AxumRiglrAdapter {
    signer_factory: Arc<dyn SignerFactory>,
}

impl fmt::Debug for AxumRiglrAdapter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AxumRiglrAdapter")
            .field("signer_factory", &"Arc<dyn SignerFactory>")
            .finish()
    }
}

impl AxumRiglrAdapter {
    /// Create a new Axum adapter with the given signer factory and RPC config
    pub fn new(signer_factory: Arc<dyn SignerFactory>) -> Self {
        Self { signer_factory }
    }

    /// Get a reference to the signer factory
    #[must_use]
    pub fn signer_factory(&self) -> &Arc<dyn SignerFactory> {
        &self.signer_factory
    }

    /// Detect authentication type from request headers.
    ///
    /// The detection logic is as follows:
    /// 1. If the `x-auth-type` header is present, its value is used.
    /// 2. If the header is absent and exactly one `SignerFactory` is registered in the
    ///    `CompositeSignerFactory`, that factory's auth type is used as the default.
    /// 3. If the header is absent and multiple factories are registered, a `BAD_REQUEST`
    ///    error is returned, as the auth type is ambiguous.
    /// 4. If no factories are registered, an `INTERNAL_SERVER_ERROR` is returned.
    fn detect_auth_type(&self, headers: &HeaderMap) -> Result<String, StatusCode> {
        // First check for explicit auth type header
        if let Some(auth_type) = headers.get("x-auth-type") {
            return auth_type
                .to_str()
                .map(ToString::to_string)
                .map_err(|_| StatusCode::BAD_REQUEST);
        }

        // Check how many auth providers are registered
        let supported_types = self.signer_factory.supported_auth_types();

        match supported_types.len() {
            0 => {
                // No providers configured
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
            1 => {
                // Single provider - use as default
                supported_types
                    .into_iter()
                    .next()
                    .ok_or(StatusCode::INTERNAL_SERVER_ERROR)
            }
            _ => {
                // Multiple providers - require explicit header
                Err(StatusCode::BAD_REQUEST)
            }
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
            let token = auth_header
                .strip_prefix("Bearer ")
                .ok_or(StatusCode::UNAUTHORIZED)?;
            let auth_type = self.detect_auth_type(headers)?;

            Ok(AuthenticationData {
                auth_type,
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
    ) -> Result<Box<dyn UnifiedSigner>, StatusCode> {
        // Extract authentication data from request headers
        let auth_data = self.extract_auth_data(headers)?;

        // Use factory to create appropriate signer
        let signer = self
            .signer_factory
            .create_signer(auth_data)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        Ok(signer)
    }

    /// SSE handler using `SignerFactory` pattern
    ///
    /// # Errors
    /// Returns an error if:
    /// - Authentication fails due to missing, invalid, or unsupported auth headers
    /// - No authentication providers are registered
    /// - Multiple auth providers require explicit `x-auth-type` header
    /// - Signer creation fails during authentication
    /// - Agent stream creation fails
    pub async fn sse_handler<A>(
        &self,
        headers: HeaderMap,
        agent: A,
        prompt: PromptRequest,
    ) -> Result<
        Sse<impl futures_util::Stream<Item = Result<Event, axum::Error>> + use<A>>,
        StatusCode,
    >
    where
        A: Agent + Clone + Send + Sync + 'static,
        A::Error: fmt::Display + fmt::Debug + Send + Sync + 'static,
    {
        tracing::info!(
            prompt_len = prompt.text.len(),
            conversation_id = ?prompt.conversation_id,
            request_id = ?prompt.request_id,
            "Processing Axum SSE request with SignerFactory"
        );

        // Extract authentication data and create signer
        let signer: Arc<dyn UnifiedSigner> = Arc::from(self.authenticate_request(&headers).await?);

        // Handle stream using framework-agnostic core
        let stream_result = handle_agent_stream(agent, signer, prompt).await;

        match stream_result {
            Ok(stream) => {
                tracing::info!("Agent stream created successfully");

                // Convert to Axum SSE stream
                let sse_stream = stream.map(|chunk| match chunk {
                    Ok(data) => Ok(Event::default().data(data)),
                    Err(e) => {
                        tracing::error!(error = %e, "Stream error");
                        Err(axum::Error::new(e))
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

    /// Completion handler using `SignerFactory` pattern
    ///
    /// # Errors
    /// Returns an error if:
    /// - Authentication fails due to missing, invalid, or unsupported auth headers
    /// - No authentication providers are registered
    /// - Multiple auth providers require explicit `x-auth-type` header
    /// - Signer creation fails during authentication
    /// - Agent completion processing fails
    pub async fn completion_handler<A>(
        &self,
        headers: HeaderMap,
        agent: A,
        prompt: PromptRequest,
    ) -> Result<Json<CompletionResponse>, StatusCode>
    where
        A: Agent + Clone + Send + Sync + 'static,
        A::Error: fmt::Display + fmt::Debug + Send + Sync + 'static,
    {
        tracing::info!(
            prompt_len = prompt.text.len(),
            conversation_id = ?prompt.conversation_id,
            request_id = ?prompt.request_id,
            "Processing Axum completion request with SignerFactory"
        );

        // Extract authentication data and create signer
        let signer: Arc<dyn UnifiedSigner> = Arc::from(self.authenticate_request(&headers).await?);

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
#[allow(
    clippy::allow_attributes,
    clippy::unused_async,
    reason = "Handler must be async for Axum compatibility, #[expect] doesn't work reliably across all build targets"
)]
pub async fn health_handler() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "service": "riglr-web-adapters",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

/// Information handler for Axum
#[allow(
    clippy::allow_attributes,
    clippy::unused_async,
    reason = "Handler must be async for Axum compatibility, #[expect] doesn't work reliably across all build targets"
)]
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
#[expect(clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use axum::body::{self, Body};
    use axum::http::{HeaderValue, Request};
    use axum::{routing::get, Router};
    use core::error::Error as StdError;
    use futures_util::stream;
    use riglr_core::signer::{
        error::Standard as SignerStandard, Chain, EvmClient, EvmSigner, SignerBase, SignerError,
        SolanaClient, SolanaSigner, UnifiedSigner,
    };
    use std::io;
    use tower::ServiceExt;

    // Mock agent for testing
    #[derive(Clone)]
    struct MockAgent {
        response: String,
        should_fail: bool,
    }

    impl MockAgent {
        fn new(response: String) -> Self {
            Self {
                response,
                should_fail: false,
            }
        }

        fn new_failing() -> Self {
            Self {
                response: String::new(),
                should_fail: true,
            }
        }
    }

    #[async_trait::async_trait]
    impl Agent for MockAgent {
        type Error = io::Error;

        async fn prompt(&self, _prompt: &str) -> Result<String, Self::Error> {
            if self.should_fail {
                return Err(io::Error::other("Mock error"));
            }
            Ok(self.response.clone())
        }

        async fn prompt_stream(
            &self,
            _prompt: &str,
        ) -> Result<stream::BoxStream<'_, Result<String, Self::Error>>, Self::Error> {
            if self.should_fail {
                return Err(io::Error::other("Mock stream error"));
            }
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

    impl SignerBase for MockSigner {
        fn supported_chains(&self) -> &[Chain] {
            &[Chain::Solana, Chain::Evm]
        }

        fn user_id(&self) -> String {
            "test_user_123".to_string()
        }
    }

    struct MockSolanaClient;

    #[async_trait]
    impl SolanaClient for MockSolanaClient {
        async fn get_balance(&self, _pubkey: &str) -> Result<String, Box<dyn SignerError>> {
            Ok("1000000000".to_string()) // 1 SOL in lamports
        }

        async fn get_latest_blockhash(&self) -> Result<String, Box<dyn SignerError>> {
            Ok("11111111111111111111111111111111".to_string())
        }

        async fn send_transaction(
            &self,
            _signed_transaction: &str,
        ) -> Result<String, Box<dyn SignerError>> {
            Ok("mock_signature_123".to_string())
        }

        async fn get_fee_for_transaction(
            &self,
            _transaction: &serde_json::Value,
        ) -> Result<String, Box<dyn SignerError>> {
            Ok("5000".to_string()) // 5000 lamports
        }

        async fn get_account_info(
            &self,
            _pubkey: &str,
        ) -> Result<Option<serde_json::Value>, Box<dyn SignerError>> {
            Ok(None)
        }

        async fn confirm_transaction(
            &self,
            _signature: &str,
        ) -> Result<serde_json::Value, Box<dyn SignerError>> {
            Ok(serde_json::json!({"confirmed": true}))
        }

        async fn get_transaction(
            &self,
            _signature: &str,
        ) -> Result<Option<serde_json::Value>, Box<dyn SignerError>> {
            Ok(None)
        }

        async fn get_slot(&self) -> Result<String, Box<dyn SignerError>> {
            Ok("123456".to_string())
        }

        async fn get_minimum_balance_for_rent_exemption(
            &self,
            _data_len: usize,
        ) -> Result<String, Box<dyn SignerError>> {
            Ok("2039280".to_string()) // Standard rent exemption amount
        }
    }

    #[async_trait]
    impl SolanaSigner for MockSigner {
        fn pubkey(&self) -> String {
            "mock_public_key".to_string()
        }

        fn client(&self) -> &dyn SolanaClient {
            &MockSolanaClient
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

    struct MockEvmClient;

    #[async_trait]
    impl EvmClient for MockEvmClient {
        async fn call(
            &self,
            _call_request: &serde_json::Value,
            _block: Option<&str>,
        ) -> Result<String, Box<dyn SignerError>> {
            Ok("0x".to_string())
        }

        async fn estimate_gas(
            &self,
            _transaction: &serde_json::Value,
        ) -> Result<String, Box<dyn SignerError>> {
            Ok("21000".to_string())
        }

        async fn get_balance(&self, _address: &str) -> Result<String, Box<dyn SignerError>> {
            Ok("1000000000000000000".to_string()) // 1 ETH in wei
        }

        async fn get_block_number(&self) -> Result<String, Box<dyn SignerError>> {
            Ok("123456".to_string())
        }

        async fn get_chain_id(&self) -> Result<String, Box<dyn SignerError>> {
            Ok("1337".to_string())
        }

        async fn get_gas_price(&self) -> Result<String, Box<dyn SignerError>> {
            Ok("20000000000".to_string()) // 20 gwei
        }

        async fn get_nonce(&self, _address: &str) -> Result<String, Box<dyn SignerError>> {
            Ok("0".to_string())
        }

        async fn get_transaction(
            &self,
            _tx_hash: &str,
        ) -> Result<Option<serde_json::Value>, Box<dyn SignerError>> {
            Ok(None)
        }

        async fn get_transaction_receipt(
            &self,
            _tx_hash: &str,
        ) -> Result<Option<serde_json::Value>, Box<dyn SignerError>> {
            Ok(None)
        }

        async fn send_raw_transaction(
            &self,
            _signed_tx: &str,
        ) -> Result<String, Box<dyn SignerError>> {
            Ok("0xmock_tx_hash".to_string())
        }
    }

    #[async_trait]
    impl EvmSigner for MockSigner {
        fn address(&self) -> String {
            "0xmock_evm_address".to_string()
        }

        fn chain_id(&self) -> u64 {
            1337
        }

        fn client(&self) -> &dyn EvmClient {
            &MockEvmClient
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

    // Mock signer factory for testing
    #[derive(Debug)]
    struct MockSignerFactory {
        supported_types: Vec<String>,
        should_fail: bool,
    }

    impl MockSignerFactory {
        fn new(supported_types: Vec<String>) -> Self {
            Self {
                supported_types,
                should_fail: false,
            }
        }

        fn new_failing() -> Self {
            Self {
                supported_types: vec!["test".to_string()],
                should_fail: true,
            }
        }

        fn new_empty() -> Self {
            Self {
                supported_types: vec![],
                should_fail: false,
            }
        }

        fn new_with_multiple_types() -> Self {
            Self {
                supported_types: vec![
                    "privy".to_string(),
                    "web3auth".to_string(),
                    "magic".to_string(),
                ],
                should_fail: false,
            }
        }
    }

    #[async_trait]
    impl SignerFactory for MockSignerFactory {
        async fn create_signer(
            &self,
            _auth_data: AuthenticationData,
        ) -> Result<Box<dyn UnifiedSigner>, Box<dyn StdError + Send + Sync>> {
            if self.should_fail {
                return Err(Box::new(io::Error::other("Mock auth error")));
            }
            Ok(Box::new(MockSigner::new()))
        }

        fn supported_auth_types(&self) -> Vec<String> {
            self.supported_types.clone()
        }
    }

    fn create_test_prompt() -> PromptRequest {
        PromptRequest {
            text: "Test prompt".to_string(),
            conversation_id: Some("conv_123".to_string()),
            request_id: Some("req_456".to_string()),
        }
    }

    // Test AxumRiglrAdapter::new
    #[test]
    fn test_axum_riglr_adapter_new() {
        let factory = Arc::new(MockSignerFactory::new(vec!["test".to_string()]));

        let adapter = AxumRiglrAdapter::new(factory);

        // Verify the adapter was created successfully
        assert_eq!(
            adapter.signer_factory.supported_auth_types(),
            vec!["test".to_string()]
        );
    }

    // Test detect_auth_type with explicit header
    #[test]
    fn test_detect_auth_type_when_explicit_header_should_return_type() {
        let factory = Arc::new(MockSignerFactory::new(vec!["default".to_string()]));
        let adapter = AxumRiglrAdapter::new(factory);

        let mut headers = HeaderMap::new();
        headers.insert(
            "x-auth-type",
            "custom"
                .parse()
                .expect("Failed to parse 'custom' as header value"),
        );

        let result = adapter.detect_auth_type(&headers);
        assert!(result.is_ok());
        assert_eq!(
            result.expect("Auth type detection should succeed with explicit header"),
            "custom"
        );
    }

    // Test detect_auth_type with invalid header value
    #[test]
    fn test_detect_auth_type_when_invalid_header_should_return_bad_request() {
        let factory = Arc::new(MockSignerFactory::new(vec!["default".to_string()]));
        let adapter = AxumRiglrAdapter::new(factory);

        let mut headers = HeaderMap::new();
        // Insert invalid UTF-8 bytes
        headers.insert(
            "x-auth-type",
            HeaderValue::from_bytes(&[0xFF, 0xFE])
                .expect("Failed to create invalid header value for test"),
        );

        let result = adapter.detect_auth_type(&headers);
        assert!(result.is_err());
        assert_eq!(
            result.expect_err("Invalid header should cause BAD_REQUEST error"),
            StatusCode::BAD_REQUEST
        );
    }

    // Test detect_auth_type fallback to default
    #[test]
    fn test_detect_auth_type_when_multiple_providers_and_no_header_should_fail() {
        let factory = Arc::new(MockSignerFactory::new_with_multiple_types());
        let adapter = AxumRiglrAdapter::new(factory);

        let headers = HeaderMap::new();

        let result = adapter.detect_auth_type(&headers);
        assert!(result.is_err());
        assert_eq!(
            result.expect_err(
                "Multiple providers without explicit header should cause BAD_REQUEST error"
            ),
            StatusCode::BAD_REQUEST
        );
    }

    #[test]
    fn test_detect_auth_type_when_single_provider_and_no_header_should_succeed() {
        let factory = Arc::new(MockSignerFactory::new(vec!["privy".to_string()]));
        let adapter = AxumRiglrAdapter::new(factory);

        let headers = HeaderMap::new();

        let result = adapter.detect_auth_type(&headers);
        assert!(result.is_ok());
        assert_eq!(
            result.expect("Single provider should succeed without explicit header"),
            "privy"
        );
    }

    // Test detect_auth_type with empty supported types
    #[test]
    fn test_detect_auth_type_when_no_supported_types_should_return_internal_error() {
        let factory = Arc::new(MockSignerFactory::new_empty());
        let adapter = AxumRiglrAdapter::new(factory);

        let headers = HeaderMap::new();

        let result = adapter.detect_auth_type(&headers);
        assert!(result.is_err());
        assert_eq!(
            result.expect_err("No supported auth types should cause INTERNAL_SERVER_ERROR"),
            StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    // Test extract_auth_data with valid bearer token
    #[test]
    fn test_extract_auth_data_when_valid_bearer_should_return_auth_data() {
        let factory = Arc::new(MockSignerFactory::new(vec!["test".to_string()]));
        let adapter = AxumRiglrAdapter::new(factory);

        let mut headers = HeaderMap::new();
        headers.insert(
            "authorization",
            "Bearer test_token_123"
                .parse()
                .expect("Failed to parse Bearer token header value"),
        );
        headers.insert(
            "x-network",
            "testnet"
                .parse()
                .expect("Failed to parse network header value"),
        );

        let result = adapter.extract_auth_data(&headers);
        assert!(result.is_ok());

        let auth_data =
            result.expect("Auth data extraction should succeed with valid Bearer token");
        assert_eq!(auth_data.auth_type, "test");
        assert_eq!(
            auth_data.credentials.get("token"),
            Some(&"test_token_123".to_string())
        );
        assert_eq!(auth_data.network, "testnet");
    }

    // Test extract_auth_data with bearer token and no network header (default)
    #[test]
    fn test_extract_auth_data_when_no_network_header_should_use_mainnet() {
        let factory = Arc::new(MockSignerFactory::new(vec!["test".to_string()]));
        let adapter = AxumRiglrAdapter::new(factory);

        let mut headers = HeaderMap::new();
        headers.insert(
            "authorization",
            "Bearer test_token_123"
                .parse()
                .expect("Failed to parse Bearer token header value"),
        );

        let result = adapter.extract_auth_data(&headers);
        assert!(result.is_ok());

        let auth_data =
            result.expect("Auth data extraction should succeed with valid Bearer token");
        assert_eq!(auth_data.network, "mainnet");
    }

    // Test extract_auth_data with invalid network header value
    #[test]
    fn test_extract_auth_data_when_invalid_network_header_should_use_mainnet() {
        let factory = Arc::new(MockSignerFactory::new(vec!["test".to_string()]));
        let adapter = AxumRiglrAdapter::new(factory);

        let mut headers = HeaderMap::new();
        headers.insert(
            "authorization",
            "Bearer test_token_123"
                .parse()
                .expect("Failed to parse Bearer token header value"),
        );
        headers.insert(
            "x-network",
            HeaderValue::from_bytes(&[0xFF, 0xFE])
                .expect("Failed to create invalid header value for test"),
        );

        let result = adapter.extract_auth_data(&headers);
        assert!(result.is_ok());

        let auth_data =
            result.expect("Auth data extraction should succeed with valid Bearer token");
        assert_eq!(auth_data.network, "mainnet");
    }

    // Test extract_auth_data with missing authorization header
    #[test]
    fn test_extract_auth_data_when_missing_auth_header_should_return_unauthorized() {
        let factory = Arc::new(MockSignerFactory::new(vec!["test".to_string()]));
        let adapter = AxumRiglrAdapter::new(factory);

        let headers = HeaderMap::new();

        let result = adapter.extract_auth_data(&headers);
        assert!(result.is_err());
        assert_eq!(
            result.expect_err("Missing authorization header should cause UNAUTHORIZED error"),
            StatusCode::UNAUTHORIZED
        );
    }

    // Test extract_auth_data with invalid authorization header value
    #[test]
    fn test_extract_auth_data_when_invalid_auth_header_value_should_return_unauthorized() {
        let factory = Arc::new(MockSignerFactory::new(vec!["test".to_string()]));
        let adapter = AxumRiglrAdapter::new(factory);

        let mut headers = HeaderMap::new();
        headers.insert(
            "authorization",
            HeaderValue::from_bytes(&[0xFF, 0xFE])
                .expect("Failed to create invalid header value for test"),
        );

        let result = adapter.extract_auth_data(&headers);
        assert!(result.is_err());
        assert_eq!(
            result.expect_err("Invalid authorization header value should cause UNAUTHORIZED error"),
            StatusCode::UNAUTHORIZED
        );
    }

    // Test extract_auth_data with non-bearer authorization
    #[test]
    fn test_extract_auth_data_when_non_bearer_auth_should_return_unauthorized() {
        let factory = Arc::new(MockSignerFactory::new(vec!["test".to_string()]));
        let adapter = AxumRiglrAdapter::new(factory);

        let mut headers = HeaderMap::new();
        headers.insert(
            "authorization",
            "Basic dGVzdDp0ZXN0"
                .parse()
                .expect("Failed to parse Basic auth header value"),
        );

        let result = adapter.extract_auth_data(&headers);
        assert!(result.is_err());
        assert_eq!(
            result.expect_err("Non-Bearer authorization should cause UNAUTHORIZED error"),
            StatusCode::UNAUTHORIZED
        );
    }

    // Test authenticate_request success
    #[tokio::test]
    async fn test_authenticate_request_when_valid_should_return_signer() {
        let factory = Arc::new(MockSignerFactory::new(vec!["test".to_string()]));
        let adapter = AxumRiglrAdapter::new(factory);

        let mut headers = HeaderMap::new();
        headers.insert(
            "authorization",
            "Bearer test_token_123"
                .parse()
                .expect("Failed to parse Bearer token header value"),
        );

        let result = adapter.authenticate_request(&headers).await;
        assert!(result.is_ok());

        let signer = result.expect("Authentication should succeed with valid Bearer token");
        // Test that the signer supports Solana and get the pubkey through the proper trait
        assert!(signer.supports_solana());
        if let Some(solana_signer) = signer.as_solana() {
            assert_eq!(solana_signer.pubkey(), "mock_public_key".to_string());
        } else {
            panic!("MockSigner should support Solana");
        }
    }

    // Test authenticate_request with signer factory failure
    #[tokio::test]
    async fn test_authenticate_request_when_factory_fails_should_return_internal_error() {
        let factory = Arc::new(MockSignerFactory::new_failing());
        let adapter = AxumRiglrAdapter::new(factory);

        let mut headers = HeaderMap::new();
        headers.insert(
            "authorization",
            "Bearer test_token_123"
                .parse()
                .expect("Failed to parse Bearer token header value"),
        );

        let result = adapter.authenticate_request(&headers).await;
        assert!(result.is_err());
        assert_eq!(
            result.expect_err("Failing factory should cause INTERNAL_SERVER_ERROR"),
            StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    // Test sse_handler success
    #[tokio::test]
    async fn test_sse_handler_when_valid_should_return_sse_stream() {
        let factory = Arc::new(MockSignerFactory::new(vec!["test".to_string()]));
        let adapter = AxumRiglrAdapter::new(factory);

        let mut headers = HeaderMap::new();
        headers.insert(
            "authorization",
            "Bearer test_token_123"
                .parse()
                .expect("Failed to parse Bearer token header value"),
        );

        let agent = MockAgent::new("Test response".to_string());
        let prompt = create_test_prompt();

        let result = adapter.sse_handler(headers, agent, prompt).await;
        assert!(result.is_ok());
    }

    // Test sse_handler with authentication failure
    #[tokio::test]
    async fn test_sse_handler_when_auth_fails_should_return_error() {
        let factory = Arc::new(MockSignerFactory::new(vec!["test".to_string()]));
        let adapter = AxumRiglrAdapter::new(factory);

        let headers = HeaderMap::new(); // Missing auth header

        let agent = MockAgent::new("Test response".to_string());
        let prompt = create_test_prompt();

        let result = adapter.sse_handler(headers, agent, prompt).await;
        assert!(result.is_err());
        assert_eq!(
            result.expect_err("Missing auth header should cause UNAUTHORIZED error"),
            StatusCode::UNAUTHORIZED
        );
    }

    // Test sse_handler with agent stream failure
    #[tokio::test]
    async fn test_sse_handler_when_agent_fails_should_return_internal_error() {
        let factory = Arc::new(MockSignerFactory::new(vec!["test".to_string()]));
        let adapter = AxumRiglrAdapter::new(factory);

        let mut headers = HeaderMap::new();
        headers.insert(
            "authorization",
            "Bearer test_token_123"
                .parse()
                .expect("Failed to parse Bearer token header value"),
        );

        let agent = MockAgent::new_failing();
        let prompt = create_test_prompt();

        let result = adapter.sse_handler(headers, agent, prompt).await;
        assert!(result.is_err());
        assert_eq!(
            result.expect_err("Failing agent should cause INTERNAL_SERVER_ERROR"),
            StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    // Test completion_handler success
    #[tokio::test]
    async fn test_completion_handler_when_valid_should_return_json_response() {
        let factory = Arc::new(MockSignerFactory::new(vec!["test".to_string()]));
        let adapter = AxumRiglrAdapter::new(factory);

        let mut headers = HeaderMap::new();
        headers.insert(
            "authorization",
            "Bearer test_token_123"
                .parse()
                .expect("Failed to parse Bearer token header value"),
        );

        let agent = MockAgent::new("Test response".to_string());
        let prompt = create_test_prompt();

        let result = adapter.completion_handler(headers, agent, prompt).await;
        assert!(result.is_ok());

        let response = result.expect("Completion handler should succeed with valid request");
        assert_eq!(response.response, "Test response");
    }

    // Test completion_handler with authentication failure
    #[tokio::test]
    async fn test_completion_handler_when_auth_fails_should_return_error() {
        let factory = Arc::new(MockSignerFactory::new(vec!["test".to_string()]));
        let adapter = AxumRiglrAdapter::new(factory);

        let headers = HeaderMap::new(); // Missing auth header

        let agent = MockAgent::new("Test response".to_string());
        let prompt = create_test_prompt();

        let result = adapter.completion_handler(headers, agent, prompt).await;
        assert!(result.is_err());
        assert_eq!(
            result.expect_err("Missing auth header should cause UNAUTHORIZED error"),
            StatusCode::UNAUTHORIZED
        );
    }

    // Test completion_handler with agent failure
    #[tokio::test]
    async fn test_completion_handler_when_agent_fails_should_return_internal_error() {
        let factory = Arc::new(MockSignerFactory::new(vec!["test".to_string()]));
        let adapter = AxumRiglrAdapter::new(factory);

        let mut headers = HeaderMap::new();
        headers.insert(
            "authorization",
            "Bearer test_token_123"
                .parse()
                .expect("Failed to parse Bearer token header value"),
        );

        let agent = MockAgent::new_failing();
        let prompt = create_test_prompt();

        let result = adapter.completion_handler(headers, agent, prompt).await;
        assert!(result.is_err());
        assert_eq!(
            result.expect_err("Failing agent should cause INTERNAL_SERVER_ERROR"),
            StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    // Enhanced health handler test that checks response content
    #[tokio::test]
    async fn test_health_handler_content() {
        let app = Router::new().route("/health", get(health_handler));

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .expect("Failed to build test request for /health endpoint"),
            )
            .await
            .expect("Health handler request should succeed");

        assert_eq!(response.status(), StatusCode::OK);

        let body = body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("Failed to read response body bytes");
        let body_str =
            String::from_utf8(body.to_vec()).expect("Response body should be valid UTF-8");
        let json: serde_json::Value =
            serde_json::from_str(&body_str).expect("Response body should be valid JSON");

        assert_eq!(
            json.get("status").expect("status field should exist"),
            "healthy"
        );
        assert_eq!(
            json.get("service").expect("service field should exist"),
            "riglr-web-adapters"
        );
        assert_eq!(
            json.get("version").expect("version field should exist"),
            env!("CARGO_PKG_VERSION")
        );
        assert!(json
            .get("timestamp")
            .expect("timestamp field should exist")
            .is_string());
    }

    // Enhanced info handler test that checks response content
    #[tokio::test]
    async fn test_info_handler_content() {
        let app = Router::new().route("/", get(info_handler));

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/")
                    .body(Body::empty())
                    .expect("Failed to build test request for / endpoint"),
            )
            .await
            .expect("Info handler request should succeed");

        assert_eq!(response.status(), StatusCode::OK);

        let body = body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("Failed to read response body bytes");
        let body_str =
            String::from_utf8(body.to_vec()).expect("Response body should be valid UTF-8");
        let json: serde_json::Value =
            serde_json::from_str(&body_str).expect("Response body should be valid JSON");

        assert_eq!(
            json.get("service").expect("service field should exist"),
            "riglr-web-adapters"
        );
        assert_eq!(
            json.get("version").expect("version field should exist"),
            env!("CARGO_PKG_VERSION")
        );
        assert_eq!(
            json.get("description")
                .expect("description field should exist"),
            "Framework-agnostic web adapters for riglr agents"
        );
        assert_eq!(
            json.get("framework").expect("framework field should exist"),
            "axum"
        );
        assert!(json
            .get("endpoints")
            .expect("endpoints field should exist")
            .is_array());
        assert_eq!(
            json.get("endpoints")
                .expect("endpoints field should exist")
                .as_array()
                .expect("endpoints should be an array")
                .len(),
            3
        );
    }

    #[tokio::test]
    async fn test_health_handler() {
        let app = Router::new().route("/health", get(health_handler));

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .expect("Failed to build test request for /health endpoint"),
            )
            .await
            .expect("Health handler request should succeed");

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_info_handler() {
        let app = Router::new().route("/", get(info_handler));

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/")
                    .body(Body::empty())
                    .expect("Failed to build test request for / endpoint"),
            )
            .await
            .expect("Info handler request should succeed");

        assert_eq!(response.status(), StatusCode::OK);
    }
}
