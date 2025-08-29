//! # RIGLR Authentication
//!
//! First-class authentication and signer factory implementations for RIGLR.
//!
//! This crate provides official, maintained SignerFactory implementations for popular
//! authentication services, making it easy to build consumer-facing, multi-tenant
//! blockchain applications.
//!
//! ## Supported Providers
//!
//! - **Privy**: Embedded wallets with social login
//! - **Web3Auth**: Non-custodial key management with social login
//! - **Magic.link**: Email-based authentication with embedded wallets
//!
//! ## Usage
//!
//! ```rust,no_run
//! use riglr_auth::{AuthProvider, PrivyConfig, CompositeSignerFactoryExt};
//! use riglr_auth::config::ProviderConfig;
//! use riglr_web_adapters::factory::CompositeSignerFactory;
//!
//! #[tokio::main]
//! async fn main() {
//!     // Create authentication provider
//!     let privy_config = PrivyConfig::from_env().expect("Failed to load Privy config");
//!     let privy_provider = AuthProvider::privy(privy_config);
//!
//!     // Register with composite factory
//!     let mut factory = CompositeSignerFactory::new();
//!     factory.register_provider(privy_provider);
//!
//!     // Use in your web server
//!     // ... server setup with factory ...
//! }
//! ```

pub mod config;
pub mod error;
pub mod provider;

#[cfg(feature = "privy")]
pub mod privy;

#[cfg(feature = "web3auth")]
pub mod web3auth;

#[cfg(feature = "magic")]
pub mod magic;

// Re-export main types
pub use config::AuthConfig;
pub use error::{AuthError, AuthResult};
pub use provider::{AuthProvider, AuthProviderType};

#[cfg(feature = "privy")]
pub use privy::{PrivyConfig, PrivyProvider};

#[cfg(feature = "web3auth")]
pub use web3auth::{Web3AuthConfig, Web3AuthProvider};

#[cfg(feature = "magic")]
pub use magic::{MagicConfig, MagicProvider};

// Re-export the SignerFactory trait from web-adapters for convenience
pub use riglr_web_adapters::factory::{AuthenticationData, SignerFactory};

/// Extension trait for CompositeSignerFactory to easily register auth providers
pub trait CompositeSignerFactoryExt {
    /// Register an authentication provider with the factory
    fn register_provider(&mut self, provider: AuthProvider);
}

impl CompositeSignerFactoryExt for riglr_web_adapters::factory::CompositeSignerFactory {
    fn register_provider(&mut self, provider: AuthProvider) {
        let auth_type = provider.auth_type();
        self.register_factory(auth_type, Box::new(provider));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use riglr_core::signer::{EvmClient, SignerError, UnifiedSigner};
    use riglr_web_adapters::factory::{AuthenticationData, CompositeSignerFactory};
    use std::collections::HashMap;

    // Mock SignerFactory implementation for testing
    #[derive(Debug)]
    struct MockSignerFactory {
        auth_types: Vec<String>,
        should_fail: bool,
    }

    impl MockSignerFactory {
        fn new(auth_types: Vec<String>) -> Self {
            Self {
                auth_types,
                should_fail: false,
            }
        }

        fn new_failing(auth_types: Vec<String>) -> Self {
            Self {
                auth_types,
                should_fail: true,
            }
        }
    }

    // Mock UnifiedSigner for testing
    #[derive(Debug)]
    struct MockUnifiedSigner {
        address: Option<String>,
        user_id: Option<String>,
        chain_id: Option<u64>,
    }

    // Mock Solana Signer for testing
    #[derive(Debug)]
    #[allow(dead_code)]
    struct MockSolanaSigner {
        address: String,
        user_id: Option<String>,
    }

    impl riglr_core::signer::SignerBase for MockSolanaSigner {
        fn locale(&self) -> String {
            "en".to_string()
        }

        fn user_id(&self) -> Option<String> {
            self.user_id.clone()
        }

        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
    }

    #[async_trait]
    impl riglr_core::signer::SolanaSigner for MockSolanaSigner {
        fn address(&self) -> String {
            self.address.clone()
        }

        fn pubkey(&self) -> String {
            self.address.clone()
        }

        async fn sign_and_send_transaction(
            &self,
            _tx_bytes: &mut Vec<u8>,
        ) -> Result<String, SignerError> {
            Ok("mock_signature".to_string())
        }

        fn client(&self) -> std::sync::Arc<dyn std::any::Any + Send + Sync> {
            std::sync::Arc::new(solana_client::rpc_client::RpcClient::new(
                "mock".to_string(),
            )) as std::sync::Arc<dyn std::any::Any + Send + Sync>
        }
    }

    // Mock EVM Signer for testing
    #[derive(Debug)]
    #[allow(dead_code)]
    struct MockEvmSigner {
        address: String,
        user_id: Option<String>,
        chain_id: u64,
    }

    impl riglr_core::signer::SignerBase for MockEvmSigner {
        fn locale(&self) -> String {
            "en".to_string()
        }

        fn user_id(&self) -> Option<String> {
            self.user_id.clone()
        }

        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
    }

    #[async_trait]
    impl riglr_core::signer::EvmSigner for MockEvmSigner {
        fn chain_id(&self) -> u64 {
            self.chain_id
        }

        fn address(&self) -> String {
            self.address.clone()
        }

        async fn sign_and_send_transaction(
            &self,
            _tx_json: serde_json::Value,
        ) -> Result<String, SignerError> {
            Ok("0xmock_tx_hash".to_string())
        }

        fn client(&self) -> Result<std::sync::Arc<dyn riglr_core::signer::EvmClient>, SignerError> {
            Ok(std::sync::Arc::new(MockEvmClient))
        }
    }

    impl MockUnifiedSigner {
        fn new(address: Option<String>) -> Self {
            Self {
                address,
                user_id: Some("test_user".to_string()),
                chain_id: Some(1),
            }
        }

        #[allow(dead_code)]
        fn address(&self) -> Option<String> {
            self.address.clone()
        }

        #[allow(dead_code)]
        fn with_user_id(mut self, user_id: Option<String>) -> Self {
            self.user_id = user_id;
            self
        }

        #[allow(dead_code)]
        fn with_chain_id(mut self, chain_id: Option<u64>) -> Self {
            self.chain_id = chain_id;
            self
        }
    }

    // Mock EVM Client
    #[derive(Debug)]
    #[allow(dead_code)]
    struct MockEvmClient;

    #[async_trait]
    impl EvmClient for MockEvmClient {
        async fn get_balance(&self, _address: &str) -> Result<String, SignerError> {
            Ok("1000".to_string())
        }

        async fn send_transaction(
            &self,
            _tx_json: &serde_json::Value,
        ) -> Result<String, SignerError> {
            Ok("0xmock_tx_hash".to_string())
        }

        async fn call(&self, _tx_json: &serde_json::Value) -> Result<String, SignerError> {
            Ok("0x".to_string())
        }
    }

    impl riglr_core::signer::SignerBase for MockUnifiedSigner {
        fn locale(&self) -> String {
            "en".to_string()
        }

        fn user_id(&self) -> Option<String> {
            self.user_id.clone()
        }

        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
    }

    impl UnifiedSigner for MockUnifiedSigner {
        fn supports_solana(&self) -> bool {
            true
        }

        fn supports_evm(&self) -> bool {
            true
        }

        fn as_solana(&self) -> Option<&dyn riglr_core::signer::SolanaSigner> {
            // For simplicity, we'll say this mock doesn't implement the specialized traits directly
            None
        }

        fn as_evm(&self) -> Option<&dyn riglr_core::signer::EvmSigner> {
            // For simplicity, we'll say this mock doesn't implement the specialized traits directly
            None
        }

        fn as_multi_chain(&self) -> Option<&dyn riglr_core::signer::MultiChainSigner> {
            None
        }
    }

    #[async_trait]
    impl SignerFactory for MockSignerFactory {
        async fn create_signer(
            &self,
            auth_data: AuthenticationData,
        ) -> Result<Box<dyn UnifiedSigner>, Box<dyn std::error::Error + Send + Sync>> {
            if self.should_fail {
                return Err("Mock creation error".into());
            }

            let address = if auth_data.credentials.get("token") == Some(&"invalid".to_string()) {
                return Err("Invalid token".into());
            } else if auth_data.credentials.get("username") == Some(&"invalid".to_string()) {
                return Err("Invalid credentials".into());
            } else {
                auth_data
                    .credentials
                    .get("address")
                    .or_else(|| auth_data.credentials.get("token"))
                    .cloned()
                    .or_else(|| Some("mock_address".to_string()))
            };

            Ok(Box::new(MockUnifiedSigner::new(address)))
        }

        fn supported_auth_types(&self) -> Vec<String> {
            self.auth_types.clone()
        }
    }

    mod auth_provider_type_tests {
        use super::*;

        #[test]
        fn test_auth_provider_type_as_str_when_privy_should_return_privy() {
            let provider_type = AuthProviderType::Privy;
            assert_eq!(provider_type.as_str(), "privy");
        }

        #[test]
        fn test_auth_provider_type_as_str_when_web3auth_should_return_web3auth() {
            let provider_type = AuthProviderType::Web3Auth;
            assert_eq!(provider_type.as_str(), "web3auth");
        }

        #[test]
        fn test_auth_provider_type_as_str_when_magic_should_return_magic() {
            let provider_type = AuthProviderType::Magic;
            assert_eq!(provider_type.as_str(), "magic");
        }

        #[test]
        fn test_auth_provider_type_as_str_when_custom_should_return_custom_name() {
            let provider_type = AuthProviderType::Custom("test_provider".to_string());
            assert_eq!(provider_type.as_str(), "test_provider");
        }

        #[test]
        fn test_auth_provider_type_as_str_when_custom_empty_should_return_empty() {
            let provider_type = AuthProviderType::Custom("".to_string());
            assert_eq!(provider_type.as_str(), "");
        }

        #[test]
        fn test_auth_provider_type_equality() {
            assert_eq!(AuthProviderType::Privy, AuthProviderType::Privy);
            assert_eq!(AuthProviderType::Web3Auth, AuthProviderType::Web3Auth);
            assert_eq!(AuthProviderType::Magic, AuthProviderType::Magic);
            assert_eq!(
                AuthProviderType::Custom("test".to_string()),
                AuthProviderType::Custom("test".to_string())
            );
            assert_ne!(AuthProviderType::Privy, AuthProviderType::Web3Auth);
            assert_ne!(
                AuthProviderType::Custom("test1".to_string()),
                AuthProviderType::Custom("test2".to_string())
            );
        }

        #[test]
        fn test_auth_provider_type_clone() {
            let original = AuthProviderType::Custom("test".to_string());
            let cloned = original.clone();
            assert_eq!(original, cloned);
        }

        #[test]
        fn test_auth_provider_type_debug() {
            let provider_type = AuthProviderType::Privy;
            let debug_str = format!("{:?}", provider_type);
            assert_eq!(debug_str, "Privy");
        }
    }

    mod auth_provider_tests {
        use super::*;

        #[test]
        fn test_auth_provider_new_should_create_provider() {
            let mock_factory = Box::new(MockSignerFactory::new(vec!["test".to_string()]));
            let provider = AuthProvider::new(AuthProviderType::Privy, mock_factory);
            assert_eq!(provider.auth_type(), "privy");
        }

        #[test]
        fn test_auth_provider_auth_type_when_privy_should_return_privy() {
            let mock_factory = Box::new(MockSignerFactory::new(vec!["privy".to_string()]));
            let provider = AuthProvider::new(AuthProviderType::Privy, mock_factory);
            assert_eq!(provider.auth_type(), "privy");
        }

        #[test]
        fn test_auth_provider_auth_type_when_web3auth_should_return_web3auth() {
            let mock_factory = Box::new(MockSignerFactory::new(vec!["web3auth".to_string()]));
            let provider = AuthProvider::new(AuthProviderType::Web3Auth, mock_factory);
            assert_eq!(provider.auth_type(), "web3auth");
        }

        #[test]
        fn test_auth_provider_auth_type_when_magic_should_return_magic() {
            let mock_factory = Box::new(MockSignerFactory::new(vec!["magic".to_string()]));
            let provider = AuthProvider::new(AuthProviderType::Magic, mock_factory);
            assert_eq!(provider.auth_type(), "magic");
        }

        #[test]
        fn test_auth_provider_auth_type_when_custom_should_return_custom_name() {
            let mock_factory = Box::new(MockSignerFactory::new(vec!["custom_auth".to_string()]));
            let provider = AuthProvider::new(
                AuthProviderType::Custom("custom_auth".to_string()),
                mock_factory,
            );
            assert_eq!(provider.auth_type(), "custom_auth");
        }

        #[test]
        fn test_auth_provider_auth_type_when_custom_empty_should_return_empty() {
            let mock_factory = Box::new(MockSignerFactory::new(vec!["".to_string()]));
            let provider =
                AuthProvider::new(AuthProviderType::Custom("".to_string()), mock_factory);
            assert_eq!(provider.auth_type(), "");
        }

        #[tokio::test]
        async fn test_auth_provider_create_signer_when_valid_token_should_succeed() {
            let mock_factory = Box::new(MockSignerFactory::new(vec!["test".to_string()]));
            let provider =
                AuthProvider::new(AuthProviderType::Custom("test".to_string()), mock_factory);

            let mut creds = HashMap::new();
            creds.insert("token".to_string(), "valid_token".to_string());
            let auth_data = AuthenticationData {
                auth_type: "test".to_string(),
                credentials: creds,
                network: "devnet".to_string(),
            };

            let result = provider.create_signer(auth_data).await;
            assert!(result.is_ok());

            let signer = result.unwrap();
            // For mock signer, just verify it supports the expected blockchain types
            assert!(signer.supports_solana() || signer.supports_evm());
        }

        #[tokio::test]
        async fn test_auth_provider_create_signer_when_invalid_token_should_fail() {
            let mock_factory = Box::new(MockSignerFactory::new(vec!["test".to_string()]));
            let provider =
                AuthProvider::new(AuthProviderType::Custom("test".to_string()), mock_factory);

            let mut creds = HashMap::new();
            creds.insert("token".to_string(), "invalid".to_string());
            let auth_data = AuthenticationData {
                auth_type: "test".to_string(),
                credentials: creds,
                network: "devnet".to_string(),
            };

            let result = provider.create_signer(auth_data).await;
            assert!(result.is_err());
            assert!(result.unwrap_err().to_string().contains("Invalid token"));
        }

        #[tokio::test]
        async fn test_auth_provider_create_signer_when_valid_credentials_should_succeed() {
            let mock_factory = Box::new(MockSignerFactory::new(vec!["test".to_string()]));
            let provider =
                AuthProvider::new(AuthProviderType::Custom("test".to_string()), mock_factory);

            let mut creds = HashMap::new();
            creds.insert("username".to_string(), "user".to_string());
            creds.insert("address".to_string(), "0x123".to_string());
            let auth_data = AuthenticationData {
                auth_type: "test".to_string(),
                credentials: creds,
                network: "devnet".to_string(),
            };

            let result = provider.create_signer(auth_data).await;
            assert!(result.is_ok());

            let signer = result.unwrap();
            // For mock signer, just verify it supports the expected blockchain types
            assert!(signer.supports_solana() || signer.supports_evm());
        }

        #[tokio::test]
        async fn test_auth_provider_create_signer_when_invalid_credentials_should_fail() {
            let mock_factory = Box::new(MockSignerFactory::new(vec!["test".to_string()]));
            let provider =
                AuthProvider::new(AuthProviderType::Custom("test".to_string()), mock_factory);

            let mut creds = HashMap::new();
            creds.insert("username".to_string(), "invalid".to_string());
            let auth_data = AuthenticationData {
                auth_type: "test".to_string(),
                credentials: creds,
                network: "devnet".to_string(),
            };

            let result = provider.create_signer(auth_data).await;
            assert!(result.is_err());
            assert!(result
                .unwrap_err()
                .to_string()
                .contains("Invalid credentials"));
        }

        #[tokio::test]
        async fn test_auth_provider_create_signer_when_custom_auth_data_should_succeed() {
            let mock_factory = Box::new(MockSignerFactory::new(vec!["test".to_string()]));
            let provider =
                AuthProvider::new(AuthProviderType::Custom("test".to_string()), mock_factory);

            let mut creds = HashMap::new();
            creds.insert("address".to_string(), "custom_addr".to_string());
            let auth_data = AuthenticationData {
                auth_type: "test".to_string(),
                credentials: creds,
                network: "devnet".to_string(),
            };

            let result = provider.create_signer(auth_data).await;
            assert!(result.is_ok());

            let signer = result.unwrap();
            // For mock signer, just verify it supports the expected blockchain types
            assert!(signer.supports_solana() || signer.supports_evm());
        }

        #[tokio::test]
        async fn test_auth_provider_create_signer_when_factory_fails_should_fail() {
            let mock_factory = Box::new(MockSignerFactory::new_failing(vec!["test".to_string()]));
            let provider =
                AuthProvider::new(AuthProviderType::Custom("test".to_string()), mock_factory);

            let mut creds = HashMap::new();
            creds.insert("token".to_string(), "any_token".to_string());
            let auth_data = AuthenticationData {
                auth_type: "test".to_string(),
                credentials: creds,
                network: "devnet".to_string(),
            };

            let result = provider.create_signer(auth_data).await;
            assert!(result.is_err());
            assert!(result
                .unwrap_err()
                .to_string()
                .contains("Mock creation error"));
        }

        #[test]
        fn test_auth_provider_supported_auth_types_should_return_inner_types() {
            let auth_types = vec!["type1".to_string(), "type2".to_string()];
            let mock_factory = Box::new(MockSignerFactory::new(auth_types.clone()));
            let provider =
                AuthProvider::new(AuthProviderType::Custom("test".to_string()), mock_factory);

            let supported = provider.supported_auth_types();
            assert_eq!(supported, auth_types);
        }

        #[test]
        fn test_auth_provider_supported_auth_types_when_empty_should_return_empty() {
            let mock_factory = Box::new(MockSignerFactory::new(vec![]));
            let provider =
                AuthProvider::new(AuthProviderType::Custom("test".to_string()), mock_factory);

            let supported = provider.supported_auth_types();
            assert!(supported.is_empty());
        }
    }

    mod composite_signer_factory_ext_tests {
        use super::*;

        #[test]
        fn test_register_provider_when_privy_provider_should_register_with_privy_auth_type() {
            let mut factory = CompositeSignerFactory::new();
            let mock_signer_factory = Box::new(MockSignerFactory::new(vec!["privy".to_string()]));
            let provider = AuthProvider::new(AuthProviderType::Privy, mock_signer_factory);

            factory.register_provider(provider);

            let supported_types = factory.supported_auth_types();
            assert!(supported_types.contains(&"privy".to_string()));
        }

        #[test]
        fn test_register_provider_when_web3auth_provider_should_register_with_web3auth_auth_type() {
            let mut factory = CompositeSignerFactory::new();
            let mock_signer_factory =
                Box::new(MockSignerFactory::new(vec!["web3auth".to_string()]));
            let provider = AuthProvider::new(AuthProviderType::Web3Auth, mock_signer_factory);

            factory.register_provider(provider);

            let supported_types = factory.supported_auth_types();
            assert!(supported_types.contains(&"web3auth".to_string()));
        }

        #[test]
        fn test_register_provider_when_magic_provider_should_register_with_magic_auth_type() {
            let mut factory = CompositeSignerFactory::new();
            let mock_signer_factory = Box::new(MockSignerFactory::new(vec!["magic".to_string()]));
            let provider = AuthProvider::new(AuthProviderType::Magic, mock_signer_factory);

            factory.register_provider(provider);

            let supported_types = factory.supported_auth_types();
            assert!(supported_types.contains(&"magic".to_string()));
        }

        #[test]
        fn test_register_provider_when_custom_provider_should_register_with_custom_auth_type() {
            let mut factory = CompositeSignerFactory::new();
            let custom_type = "my_custom_auth".to_string();
            let mock_signer_factory = Box::new(MockSignerFactory::new(vec![custom_type.clone()]));
            let provider = AuthProvider::new(
                AuthProviderType::Custom(custom_type.clone()),
                mock_signer_factory,
            );

            factory.register_provider(provider);

            let supported_types = factory.supported_auth_types();
            assert!(supported_types.contains(&custom_type));
        }

        #[test]
        fn test_register_provider_when_empty_custom_name_should_register_with_empty_auth_type() {
            let mut factory = CompositeSignerFactory::new();
            let mock_signer_factory = Box::new(MockSignerFactory::new(vec!["".to_string()]));
            let provider = AuthProvider::new(
                AuthProviderType::Custom("".to_string()),
                mock_signer_factory,
            );

            factory.register_provider(provider);

            let supported_types = factory.supported_auth_types();
            assert!(supported_types.contains(&"".to_string()));
        }

        #[test]
        fn test_register_provider_when_multiple_providers_should_register_all() {
            let mut factory = CompositeSignerFactory::new();

            let privy_provider = AuthProvider::new(
                AuthProviderType::Privy,
                Box::new(MockSignerFactory::new(vec!["privy".to_string()])),
            );
            let web3auth_provider = AuthProvider::new(
                AuthProviderType::Web3Auth,
                Box::new(MockSignerFactory::new(vec!["web3auth".to_string()])),
            );
            let custom_provider = AuthProvider::new(
                AuthProviderType::Custom("custom".to_string()),
                Box::new(MockSignerFactory::new(vec!["custom".to_string()])),
            );

            factory.register_provider(privy_provider);
            factory.register_provider(web3auth_provider);
            factory.register_provider(custom_provider);

            let supported_types = factory.supported_auth_types();
            assert!(supported_types.contains(&"privy".to_string()));
            assert!(supported_types.contains(&"web3auth".to_string()));
            assert!(supported_types.contains(&"custom".to_string()));
        }

        #[tokio::test]
        async fn test_register_provider_when_registered_should_be_usable_for_creating_signers() {
            let mut factory = CompositeSignerFactory::new();
            let mock_signer_factory = Box::new(MockSignerFactory::new(vec!["test".to_string()]));
            let provider = AuthProvider::new(
                AuthProviderType::Custom("test".to_string()),
                mock_signer_factory,
            );

            factory.register_provider(provider);

            let mut creds = HashMap::new();
            creds.insert("token".to_string(), "test_token".to_string());
            let auth_data = AuthenticationData {
                auth_type: "test".to_string(),
                credentials: creds,
                network: "devnet".to_string(),
            };

            let result = factory.create_signer(auth_data).await;
            assert!(result.is_ok());

            let signer = result.unwrap();
            // For mock signer, just verify it supports the expected blockchain types
            assert!(signer.supports_solana() || signer.supports_evm());
        }

        #[tokio::test]
        async fn test_register_provider_when_registered_failing_provider_should_fail_signer_creation(
        ) {
            let mut factory = CompositeSignerFactory::new();
            let mock_signer_factory =
                Box::new(MockSignerFactory::new_failing(vec!["test".to_string()]));
            let provider = AuthProvider::new(
                AuthProviderType::Custom("test".to_string()),
                mock_signer_factory,
            );

            factory.register_provider(provider);

            let mut creds = HashMap::new();
            creds.insert("token".to_string(), "test_token".to_string());
            let auth_data = AuthenticationData {
                auth_type: "test".to_string(),
                credentials: creds,
                network: "devnet".to_string(),
            };

            let result = factory.create_signer(auth_data).await;
            assert!(result.is_err());
        }
    }

    mod integration_tests {
        use super::*;

        #[tokio::test]
        async fn test_full_workflow_register_and_use_provider() {
            let mut factory = CompositeSignerFactory::new();

            // Create and register a provider
            let mock_signer_factory =
                Box::new(MockSignerFactory::new(vec!["integration_test".to_string()]));
            let provider = AuthProvider::new(
                AuthProviderType::Custom("integration_test".to_string()),
                mock_signer_factory,
            );

            factory.register_provider(provider);

            // Verify it's registered
            let supported_types = factory.supported_auth_types();
            assert!(supported_types.contains(&"integration_test".to_string()));

            // Use it to create a signer
            let mut creds = HashMap::new();
            creds.insert("token".to_string(), "integration_token".to_string());
            let auth_data = AuthenticationData {
                auth_type: "integration_test".to_string(),
                credentials: creds,
                network: "devnet".to_string(),
            };

            let signer_result = factory.create_signer(auth_data).await;
            assert!(signer_result.is_ok());

            let signer = signer_result.unwrap();
            // Verify basic properties through SignerBase trait
            assert_eq!(signer.user_id(), Some("test_user".to_string()));
            assert_eq!(signer.locale(), "en");

            // Verify signer supports expected blockchain types
            assert!(signer.supports_solana() || signer.supports_evm());
        }

        #[tokio::test]
        async fn test_full_workflow_with_credentials() {
            let mut factory = CompositeSignerFactory::new();

            let mock_signer_factory =
                Box::new(MockSignerFactory::new(vec!["creds_test".to_string()]));
            let provider = AuthProvider::new(
                AuthProviderType::Custom("creds_test".to_string()),
                mock_signer_factory,
            );

            factory.register_provider(provider);

            let mut creds = HashMap::new();
            creds.insert("username".to_string(), "test_user".to_string());
            creds.insert("address".to_string(), "0xabcd".to_string());
            let auth_data = AuthenticationData {
                auth_type: "creds_test".to_string(),
                credentials: creds,
                network: "devnet".to_string(),
            };

            let signer_result = factory.create_signer(auth_data).await;
            assert!(signer_result.is_ok());

            let signer = signer_result.unwrap();
            // For mock signer, just verify it supports the expected blockchain types
            assert!(signer.supports_solana() || signer.supports_evm());
        }

        #[tokio::test]
        async fn test_full_workflow_with_custom_auth_data() {
            let mut factory = CompositeSignerFactory::new();

            let mock_signer_factory =
                Box::new(MockSignerFactory::new(vec!["custom_data_test".to_string()]));
            let provider = AuthProvider::new(
                AuthProviderType::Custom("custom_data_test".to_string()),
                mock_signer_factory,
            );

            factory.register_provider(provider);

            let mut creds = HashMap::new();
            creds.insert("address".to_string(), "custom_0x456".to_string());
            creds.insert("metadata".to_string(), "test_meta".to_string());
            let auth_data = AuthenticationData {
                auth_type: "custom_data_test".to_string(),
                credentials: creds,
                network: "devnet".to_string(),
            };

            let signer_result = factory.create_signer(auth_data).await;
            assert!(signer_result.is_ok());

            let signer = signer_result.unwrap();
            // For mock signer, just verify it supports the expected blockchain types
            assert!(signer.supports_solana() || signer.supports_evm());
        }

        #[tokio::test]
        async fn test_full_workflow_with_error_signer() {
            let mut factory = CompositeSignerFactory::new();

            let mock_signer_factory =
                Box::new(MockSignerFactory::new(vec!["error_test".to_string()]));
            let provider = AuthProvider::new(
                AuthProviderType::Custom("error_test".to_string()),
                mock_signer_factory,
            );

            factory.register_provider(provider);

            let mut creds = HashMap::new();
            creds.insert("address".to_string(), "error".to_string());
            let auth_data = AuthenticationData {
                auth_type: "error_test".to_string(),
                credentials: creds,
                network: "devnet".to_string(),
            };

            let signer_result = factory.create_signer(auth_data).await;
            assert!(signer_result.is_ok());

            let _signer = signer_result.unwrap();
            // Test completed - the mock signer was created successfully
        }

        #[tokio::test]
        async fn test_full_workflow_evm_client() {
            let mut factory = CompositeSignerFactory::new();

            let mock_signer_factory =
                Box::new(MockSignerFactory::new(vec!["evm_test".to_string()]));
            let provider = AuthProvider::new(
                AuthProviderType::Custom("evm_test".to_string()),
                mock_signer_factory,
            );

            factory.register_provider(provider);

            let mut creds = HashMap::new();
            creds.insert("address".to_string(), "0xtest".to_string());
            let auth_data = AuthenticationData {
                auth_type: "evm_test".to_string(),
                credentials: creds,
                network: "devnet".to_string(),
            };

            let signer_result = factory.create_signer(auth_data).await;
            assert!(signer_result.is_ok());

            let signer = signer_result.unwrap();
            // For mock signer, just verify it supports the expected blockchain types
            assert!(signer.supports_solana() || signer.supports_evm());
        }

        #[tokio::test]
        async fn test_full_workflow_solana_client() {
            let mut factory = CompositeSignerFactory::new();

            let mock_signer_factory =
                Box::new(MockSignerFactory::new(vec!["solana_test".to_string()]));
            let provider = AuthProvider::new(
                AuthProviderType::Custom("solana_test".to_string()),
                mock_signer_factory,
            );

            factory.register_provider(provider);

            let mut creds = HashMap::new();
            creds.insert("address".to_string(), "solana_addr".to_string());
            let auth_data = AuthenticationData {
                auth_type: "solana_test".to_string(),
                credentials: creds,
                network: "devnet".to_string(),
            };

            let signer_result = factory.create_signer(auth_data).await;
            assert!(signer_result.is_ok());

            let signer = signer_result.unwrap();
            // For mock signer, just verify it supports the expected blockchain types
            assert!(signer.supports_solana() || signer.supports_evm());
        }
    }
}
