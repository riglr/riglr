//! riglr-web-adapters: Framework-agnostic web adapters for riglr agents
//!
//! This crate provides a library-first approach to serving riglr agents over HTTP,
//! with framework-agnostic core handlers and specific adapters for different web frameworks.
//!
//! ## Design Philosophy
//!
//! - **Library-first**: Core logic is framework-agnostic and reusable
//! - **Framework adapters**: Thin wrappers around core handlers for specific frameworks
//! - **SignerContext integration**: Proper signer isolation per request
//! - **Type safety**: Leverages Rust's type system for correctness
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────┐
//! │   Your App      │
//! └─────────────────┘
//!          │
//! ┌─────────────────┐
//! │ Framework       │ ← actix.rs, axum.rs
//! │ Adapters        │
//! └─────────────────┘
//!          │
//! ┌─────────────────┐
//! │ Core Handlers   │ ← core.rs (framework-agnostic)
//! └─────────────────┘
//!          │
//! ┌─────────────────┐
//! │ riglr-core      │
//! └─────────────────┘
//! ```
//!
//! ## Usage
//!
//! ### With Actix Web
//!
//! ```rust,ignore
//! use riglr_web_adapters::actix::{sse_handler, completion_handler};
//! use actix_web::{web, App, HttpServer};
//! use rig::providers::anthropic::Client;
//!
//! #[actix_web::main]
//! async fn main() -> std::io::Result<()> {
//!     let client = Client::from_env();
//!     let agent = client.agent("claude-3-5-sonnet").build();
//!
//!     HttpServer::new(move || {
//!         App::new()
//!             .app_data(web::Data::new(agent.clone()))
//!             .route("/api/v1/sse", web::post().to(sse_handler))
//!             .route("/api/v1/completion", web::post().to(completion_handler))
//!     })
//!     .bind("0.0.0.0:8080")?
//!     .run()
//!     .await
//! }
//! ```
//!
//! ### With Axum
//!
//! ```rust,ignore
//! use riglr_web_adapters::axum::{sse_handler, completion_handler};
//! use axum::{routing::post, Router};
//! use rig::providers::anthropic::Client;
//!
//! #[tokio::main]
//! async fn main() {
//!     let client = Client::from_env();
//!     let agent = client.agent("claude-3-5-sonnet").build();
//!
//!     let app = Router::new()
//!         .route("/api/v1/sse", post(sse_handler))
//!         .route("/api/v1/completion", post(completion_handler))
//!         .with_state(agent);
//!
//!     axum::Server::bind(&"0.0.0.0:8080".parse().unwrap())
//!         .serve(app.into_make_service())
//!         .await
//!         .unwrap();
//! }
//! ```

pub mod core;
pub mod factory;

#[cfg(feature = "actix")]
pub mod actix;

#[cfg(feature = "axum")]
pub mod axum;

// Re-export commonly used types
pub use core::{Agent, AgentStream, CompletionResponse, PromptRequest};

// Re-export factory traits for easy use
pub use factory::{AuthenticationData, CompositeSignerFactory, SignerFactory};

// Re-export new adapter types for easy use
#[cfg(feature = "actix")]
pub use actix::ActixRiglrAdapter;

#[cfg(feature = "axum")]
pub use axum::AxumRiglrAdapter;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_library_compiles() {
        // Basic compilation test to ensure the library structure is valid
        // The fact that this test compiles is the test itself
    }

    #[test]
    fn test_core_module_exists() {
        // Test that we can access the core module
        // This ensures the module declaration is valid
        // The fact that this test compiles proves module access works
    }

    #[test]
    fn test_factory_module_exists() {
        // Test that we can access the factory module
        // This ensures the module declaration is valid
        // The fact that this test compiles proves module access works
    }

    #[cfg(feature = "actix")]
    #[test]
    fn test_actix_module_exists_when_feature_enabled() {
        // Test that actix module is available when feature is enabled
        // The fact that this test compiles proves module access works
    }

    #[cfg(feature = "axum")]
    #[test]
    fn test_axum_module_exists_when_feature_enabled() {
        // Test that axum module is available when feature is enabled
        // The fact that this test compiles proves module access works
    }

    #[test]
    fn test_core_re_exports_available() {
        // Test that re-exported types from core are accessible

        // Create a PromptRequest to test accessibility
        let request = PromptRequest {
            text: "test".to_string(),
            conversation_id: None,
            request_id: None,
        };

        assert_eq!(request.text, "test");
        assert_eq!(request.conversation_id, None);
        assert_eq!(request.request_id, None);
    }

    #[test]
    fn test_factory_re_exports_available() {
        // Test that re-exported types from factory are accessible

        let auth_data = AuthenticationData {
            auth_type: "test".to_string(),
            credentials: std::collections::HashMap::new(),
            network: "testnet".to_string(),
        };

        assert_eq!(auth_data.auth_type, "test");
        assert_eq!(auth_data.network, "testnet");
        assert!(auth_data.credentials.is_empty());
    }

    #[test]
    fn test_composite_signer_factory_re_export() {
        // Test that CompositeSignerFactory is accessible via re-export
        let factory = CompositeSignerFactory::new();
        assert!(factory.get_registered_auth_types().is_empty());
    }

    #[cfg(feature = "actix")]
    #[test]
    fn test_actix_adapter_re_export_available() {
        // Test that ActixRiglrAdapter is available when actix feature is enabled
        use std::sync::Arc;

        // Create a minimal signer factory for testing
        #[derive(Debug)]
        struct TestSignerFactory;

        #[async_trait::async_trait]
        impl SignerFactory for TestSignerFactory {
            async fn create_signer(
                &self,
                _auth_data: AuthenticationData,
            ) -> Result<
                Box<dyn riglr_core::signer::UnifiedSigner>,
                Box<dyn std::error::Error + Send + Sync>,
            > {
                Err("Test factory".into())
            }

            fn supported_auth_types(&self) -> Vec<String> {
                vec!["test".to_string()]
            }
        }

        let factory = Arc::new(TestSignerFactory);
        let _adapter = ActixRiglrAdapter::new(factory);

        // The fact that this compiles proves the re-export works
    }

    #[cfg(feature = "axum")]
    #[test]
    fn test_axum_adapter_re_export_available() {
        // Test that AxumRiglrAdapter is available when axum feature is enabled
        use std::sync::Arc;

        // Create a minimal signer factory for testing
        #[derive(Debug)]
        struct TestSignerFactory;

        #[async_trait::async_trait]
        impl SignerFactory for TestSignerFactory {
            async fn create_signer(
                &self,
                _auth_data: AuthenticationData,
            ) -> Result<
                Box<dyn riglr_core::signer::UnifiedSigner>,
                Box<dyn std::error::Error + Send + Sync>,
            > {
                Err("Test factory".into())
            }

            fn supported_auth_types(&self) -> Vec<String> {
                vec!["test".to_string()]
            }
        }

        let factory = Arc::new(TestSignerFactory);
        let _adapter = AxumRiglrAdapter::new(factory);

        // The fact that this compiles proves the re-export works
    }

    #[test]
    fn test_prompt_request_re_export_functionality() {
        // Test that PromptRequest re-export works with all field combinations

        // Test with all fields populated
        let request_full = PromptRequest {
            text: "Hello world".to_string(),
            conversation_id: Some("conv-123".to_string()),
            request_id: Some("req-456".to_string()),
        };

        assert_eq!(request_full.text, "Hello world");
        assert_eq!(request_full.conversation_id, Some("conv-123".to_string()));
        assert_eq!(request_full.request_id, Some("req-456".to_string()));

        // Test with minimal fields
        let request_minimal = PromptRequest {
            text: "".to_string(),
            conversation_id: None,
            request_id: None,
        };

        assert_eq!(request_minimal.text, "");
        assert_eq!(request_minimal.conversation_id, None);
        assert_eq!(request_minimal.request_id, None);

        // Test cloning functionality
        let cloned = request_full.clone();
        assert_eq!(cloned.text, request_full.text);
        assert_eq!(cloned.conversation_id, request_full.conversation_id);
        assert_eq!(cloned.request_id, request_full.request_id);
    }

    #[test]
    fn test_completion_response_re_export_functionality() {
        // Test that CompletionResponse re-export works properly
        let timestamp = chrono::Utc::now();

        let response = CompletionResponse {
            response: "AI response".to_string(),
            model: "test-model".to_string(),
            conversation_id: "conv-789".to_string(),
            request_id: "req-101".to_string(),
            timestamp,
        };

        assert_eq!(response.response, "AI response");
        assert_eq!(response.model, "test-model");
        assert_eq!(response.conversation_id, "conv-789");
        assert_eq!(response.request_id, "req-101");
        assert_eq!(response.timestamp, timestamp);
    }

    #[test]
    fn test_authentication_data_re_export_functionality() {
        // Test that AuthenticationData re-export works with various configurations

        // Test with empty credentials
        let auth_empty = AuthenticationData {
            auth_type: "privy".to_string(),
            credentials: std::collections::HashMap::new(),
            network: "mainnet".to_string(),
        };

        assert_eq!(auth_empty.auth_type, "privy");
        assert_eq!(auth_empty.network, "mainnet");
        assert!(auth_empty.credentials.is_empty());

        // Test with populated credentials
        let mut credentials = std::collections::HashMap::new();
        credentials.insert("token".to_string(), "abc123".to_string());
        credentials.insert("user_id".to_string(), "user456".to_string());

        let auth_populated = AuthenticationData {
            auth_type: "web3auth".to_string(),
            credentials,
            network: "devnet".to_string(),
        };

        assert_eq!(auth_populated.auth_type, "web3auth");
        assert_eq!(auth_populated.network, "devnet");
        assert_eq!(auth_populated.credentials.len(), 2);
        assert_eq!(
            auth_populated.credentials.get("token"),
            Some(&"abc123".to_string())
        );
        assert_eq!(
            auth_populated.credentials.get("user_id"),
            Some(&"user456".to_string())
        );

        // Test cloning
        let cloned = auth_populated.clone();
        assert_eq!(cloned.auth_type, auth_populated.auth_type);
        assert_eq!(cloned.network, auth_populated.network);
        assert_eq!(cloned.credentials, auth_populated.credentials);
    }

    #[test]
    fn test_signer_factory_trait_re_export() {
        // Test that SignerFactory trait is properly re-exported and usable
        #[derive(Debug)]
        struct MockFactory;

        #[async_trait::async_trait]
        impl SignerFactory for MockFactory {
            async fn create_signer(
                &self,
                _auth_data: AuthenticationData,
            ) -> Result<
                Box<dyn riglr_core::signer::UnifiedSigner>,
                Box<dyn std::error::Error + Send + Sync>,
            > {
                Err("Mock implementation".into())
            }

            fn supported_auth_types(&self) -> Vec<String> {
                vec!["mock".to_string(), "test".to_string()]
            }
        }

        let factory = MockFactory;
        let supported_types = factory.supported_auth_types();

        assert_eq!(supported_types.len(), 2);
        assert!(supported_types.contains(&"mock".to_string()));
        assert!(supported_types.contains(&"test".to_string()));
    }

    #[test]
    fn test_composite_signer_factory_full_functionality() {
        // Test CompositeSignerFactory through re-export with all methods
        use std::sync::Arc;

        #[derive(Debug)]
        struct TestFactory1;
        #[derive(Debug)]
        struct TestFactory2;

        #[async_trait::async_trait]
        impl SignerFactory for TestFactory1 {
            async fn create_signer(
                &self,
                _auth_data: AuthenticationData,
            ) -> Result<
                Box<dyn riglr_core::signer::UnifiedSigner>,
                Box<dyn std::error::Error + Send + Sync>,
            > {
                Err("Factory 1".into())
            }

            fn supported_auth_types(&self) -> Vec<String> {
                vec!["type1".to_string()]
            }
        }

        #[async_trait::async_trait]
        impl SignerFactory for TestFactory2 {
            async fn create_signer(
                &self,
                _auth_data: AuthenticationData,
            ) -> Result<
                Box<dyn riglr_core::signer::UnifiedSigner>,
                Box<dyn std::error::Error + Send + Sync>,
            > {
                Err("Factory 2".into())
            }

            fn supported_auth_types(&self) -> Vec<String> {
                vec!["type2".to_string(), "type3".to_string()]
            }
        }

        let mut composite = CompositeSignerFactory::new();

        // Test initial state
        assert!(composite.get_registered_auth_types().is_empty());
        assert!(composite.supported_auth_types().is_empty());

        // Test register_factory method
        composite.register_factory("type1".to_string(), Box::new(TestFactory1));

        let registered = composite.get_registered_auth_types();
        assert_eq!(registered.len(), 1);
        assert!(registered.contains(&"type1".to_string()));

        // Test add_factory method
        composite.add_factory("type2".to_string(), Arc::new(TestFactory2));

        let registered = composite.get_registered_auth_types();
        assert_eq!(registered.len(), 2);
        assert!(registered.contains(&"type1".to_string()));
        assert!(registered.contains(&"type2".to_string()));

        // Test supported_auth_types aggregation
        let supported = composite.supported_auth_types();
        assert!(supported.len() >= 2); // At least type1, type2, type3
        assert!(supported.contains(&"type1".to_string()));
        assert!(supported.contains(&"type2".to_string()));
        assert!(supported.contains(&"type3".to_string()));
    }

    #[tokio::test]
    async fn test_composite_signer_factory_create_signer_success() {
        // Test successful signer creation through re-export
        use riglr_solana_tools::signer::LocalSolanaSigner;
        use solana_sdk::signature::Keypair;

        #[derive(Debug)]
        struct WorkingFactory;

        #[async_trait::async_trait]
        impl SignerFactory for WorkingFactory {
            async fn create_signer(
                &self,
                _auth_data: AuthenticationData,
            ) -> Result<
                Box<dyn riglr_core::signer::UnifiedSigner>,
                Box<dyn std::error::Error + Send + Sync>,
            > {
                let keypair = Keypair::new();
                let signer = LocalSolanaSigner::from_keypair_with_url(
                    keypair,
                    "https://api.devnet.solana.com".to_string(),
                );
                Ok(Box::new(signer))
            }

            fn supported_auth_types(&self) -> Vec<String> {
                vec!["working".to_string()]
            }
        }

        let mut composite = CompositeSignerFactory::new();
        composite.register_factory("working".to_string(), Box::new(WorkingFactory));

        let auth_data = AuthenticationData {
            auth_type: "working".to_string(),
            credentials: std::collections::HashMap::new(),
            network: "devnet".to_string(),
        };

        let result = composite.create_signer(auth_data).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_composite_signer_factory_create_signer_failure() {
        // Test failed signer creation through re-export
        let composite = CompositeSignerFactory::new();

        let auth_data = AuthenticationData {
            auth_type: "nonexistent".to_string(),
            credentials: std::collections::HashMap::new(),
            network: "testnet".to_string(),
        };

        let result = composite.create_signer(auth_data).await;

        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Unsupported auth type"));
        assert!(error_msg.contains("nonexistent"));
    }

    #[test]
    fn test_all_public_re_exports_accessible() {
        // Comprehensive test that all public re-exports are accessible and functional

        // Test that we can create and use types through re-exports
        let _prompt = PromptRequest {
            text: "test".to_string(),
            conversation_id: None,
            request_id: None,
        };

        let _response = CompletionResponse {
            response: "response".to_string(),
            model: "model".to_string(),
            conversation_id: "conv".to_string(),
            request_id: "req".to_string(),
            timestamp: chrono::Utc::now(),
        };

        let _auth_data = AuthenticationData {
            auth_type: "test".to_string(),
            credentials: std::collections::HashMap::new(),
            network: "testnet".to_string(),
        };

        let _factory = CompositeSignerFactory::new();

        // This test ensures all main re-exports compile and are accessible
    }

    #[test]
    fn test_debug_trait_implementations() {
        // Test that Debug trait works for re-exported types that support it

        let request = PromptRequest {
            text: "debug test".to_string(),
            conversation_id: Some("debug-conv".to_string()),
            request_id: Some("debug-req".to_string()),
        };

        let debug_str = format!("{:?}", request);
        assert!(debug_str.contains("debug test"));
        assert!(debug_str.contains("debug-conv"));
        assert!(debug_str.contains("debug-req"));

        let auth_data = AuthenticationData {
            auth_type: "debug-auth".to_string(),
            credentials: std::collections::HashMap::new(),
            network: "debug-net".to_string(),
        };

        let debug_str = format!("{:?}", auth_data);
        assert!(debug_str.contains("debug-auth"));
        assert!(debug_str.contains("debug-net"));

        let response = CompletionResponse {
            response: "debug response".to_string(),
            model: "debug-model".to_string(),
            conversation_id: "debug-conv".to_string(),
            request_id: "debug-req".to_string(),
            timestamp: chrono::Utc::now(),
        };

        let debug_str = format!("{:?}", response);
        assert!(debug_str.contains("debug response"));
        assert!(debug_str.contains("debug-model"));
        assert!(debug_str.contains("debug-conv"));
        assert!(debug_str.contains("debug-req"));
    }

    #[test]
    fn test_edge_cases_empty_strings() {
        // Test edge cases with empty strings and default values

        let request = PromptRequest {
            text: String::default(),
            conversation_id: Some(String::default()),
            request_id: Some(String::default()),
        };

        assert_eq!(request.text, "");
        assert_eq!(request.conversation_id, Some(String::default()));
        assert_eq!(request.request_id, Some(String::default()));

        let auth_data = AuthenticationData {
            auth_type: String::default(),
            credentials: std::collections::HashMap::new(),
            network: String::default(),
        };

        assert_eq!(auth_data.auth_type, "");
        assert_eq!(auth_data.network, "");
        assert!(auth_data.credentials.is_empty());
    }

    #[test]
    fn test_large_string_values() {
        // Test with large string values to ensure no size limitations
        let large_text = "a".repeat(10000);
        let large_auth_type = "b".repeat(1000);
        let large_network = "c".repeat(500);

        let request = PromptRequest {
            text: large_text.clone(),
            conversation_id: Some("conv".to_string()),
            request_id: Some("req".to_string()),
        };

        assert_eq!(request.text.len(), 10000);
        assert_eq!(request.text, large_text);

        let auth_data = AuthenticationData {
            auth_type: large_auth_type.clone(),
            credentials: std::collections::HashMap::new(),
            network: large_network.clone(),
        };

        assert_eq!(auth_data.auth_type.len(), 1000);
        assert_eq!(auth_data.network.len(), 500);
        assert_eq!(auth_data.auth_type, large_auth_type);
        assert_eq!(auth_data.network, large_network);
    }

    #[test]
    fn test_unicode_string_handling() {
        // Test Unicode string handling in re-exported types
        let unicode_text = "Hello 世界 🌍 émojis";
        let unicode_auth = "テスト";
        let unicode_network = "ñetwork";

        let request = PromptRequest {
            text: unicode_text.to_string(),
            conversation_id: Some("conv-🎯".to_string()),
            request_id: Some("req-✅".to_string()),
        };

        assert_eq!(request.text, unicode_text);
        assert_eq!(request.conversation_id, Some("conv-🎯".to_string()));
        assert_eq!(request.request_id, Some("req-✅".to_string()));

        let auth_data = AuthenticationData {
            auth_type: unicode_auth.to_string(),
            credentials: std::collections::HashMap::new(),
            network: unicode_network.to_string(),
        };

        assert_eq!(auth_data.auth_type, unicode_auth);
        assert_eq!(auth_data.network, unicode_network);
    }
}
