//! SignerFactory trait for abstracting authentication and signer creation
//!
//! This module provides the core abstraction for creating signers from HTTP request
//! authentication data. It enables web adapters to work with any authentication
//! provider by implementing the SignerFactory trait.

use async_trait::async_trait;
use riglr_core::signer::UnifiedSigner;
use std::collections::HashMap;

/// Abstract factory for creating signers from authentication data
#[async_trait]
pub trait SignerFactory: Send + Sync {
    /// Create a signer from authentication data
    ///
    /// # Arguments
    /// * `auth_data` - Authentication data extracted from HTTP request
    ///
    /// # Returns
    /// A boxed transaction signer or an error
    async fn create_signer(
        &self,
        auth_data: AuthenticationData,
    ) -> Result<Box<dyn UnifiedSigner>, Box<dyn std::error::Error + Send + Sync>>;

    /// Get list of supported authentication types
    fn supported_auth_types(&self) -> Vec<String>;
}

/// Authentication data extracted from HTTP requests
#[derive(Debug, Clone)]
pub struct AuthenticationData {
    /// Type of authentication (e.g., "privy", "web3auth", "magic")
    pub auth_type: String,
    /// Key-value pairs of authentication credentials
    pub credentials: HashMap<String, String>,
    /// Target blockchain network
    pub network: String,
}

/// Composite factory that can hold multiple authentication providers
#[derive(Default)]
pub struct CompositeSignerFactory {
    factories: HashMap<String, std::sync::Arc<dyn SignerFactory>>,
}

impl CompositeSignerFactory {
    /// Create a new composite factory
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a signer factory for a specific auth type
    ///
    /// # Arguments
    /// * `auth_type` - Authentication type identifier
    /// * `factory` - Factory implementation for this auth type
    pub fn register_factory(&mut self, auth_type: String, factory: Box<dyn SignerFactory>) {
        self.factories
            .insert(auth_type, std::sync::Arc::from(factory));
    }

    /// Convenience: add a factory wrapped in Arc
    pub fn add_factory(&mut self, auth_type: String, factory: std::sync::Arc<dyn SignerFactory>) {
        self.factories.insert(auth_type, factory);
    }

    /// Get all registered auth types
    pub fn get_registered_auth_types(&self) -> Vec<String> {
        self.factories.keys().cloned().collect()
    }
}

#[async_trait]
impl SignerFactory for CompositeSignerFactory {
    async fn create_signer(
        &self,
        auth_data: AuthenticationData,
    ) -> Result<Box<dyn UnifiedSigner>, Box<dyn std::error::Error + Send + Sync>> {
        let factory = self
            .factories
            .get(&auth_data.auth_type)
            .ok_or_else(|| format!("Unsupported auth type: {}", auth_data.auth_type))?;

        factory.create_signer(auth_data).await
    }

    fn supported_auth_types(&self) -> Vec<String> {
        let mut all_types = Vec::new();
        for factory in self.factories.values() {
            all_types.extend(factory.supported_auth_types());
        }
        all_types
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use riglr_core::signer::UnifiedSigner;
    use riglr_solana_tools::signer::LocalSolanaSigner;
    use solana_sdk::signature::Keypair;

    // Mock signer factory for testing
    struct MockSignerFactory;

    #[async_trait]
    impl SignerFactory for MockSignerFactory {
        async fn create_signer(
            &self,
            _auth_data: AuthenticationData,
        ) -> Result<Box<dyn UnifiedSigner>, Box<dyn std::error::Error + Send + Sync>> {
            let keypair = Keypair::new();
            let signer =
                LocalSolanaSigner::new(keypair, "https://api.devnet.solana.com".to_string());
            Ok(Box::new(signer))
        }

        fn supported_auth_types(&self) -> Vec<String> {
            vec!["mock".to_string()]
        }
    }

    #[tokio::test]
    async fn test_composite_factory_creation() {
        let factory = CompositeSignerFactory::default();
        assert!(factory.get_registered_auth_types().is_empty());
    }

    #[tokio::test]
    async fn test_factory_registration() {
        let mut composite = CompositeSignerFactory::default();
        let mock_factory = Box::new(MockSignerFactory);

        composite.register_factory("mock".to_string(), mock_factory);

        let registered_types = composite.get_registered_auth_types();
        assert_eq!(registered_types.len(), 1);
        assert!(registered_types.contains(&"mock".to_string()));
    }

    #[tokio::test]
    async fn test_signer_creation() {
        let mut composite = CompositeSignerFactory::default();
        let mock_factory = Box::new(MockSignerFactory);

        composite.register_factory("mock".to_string(), mock_factory);

        let auth_data = AuthenticationData {
            auth_type: "mock".to_string(),
            credentials: [("token".to_string(), "test_token".to_string())].into(),
            network: "devnet".to_string(),
        };

        let result = composite.create_signer(auth_data).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_unsupported_auth_type() {
        let composite = CompositeSignerFactory::default();

        let auth_data = AuthenticationData {
            auth_type: "unsupported".to_string(),
            credentials: HashMap::default(),
            network: "devnet".to_string(),
        };

        let result = composite.create_signer(auth_data).await;

        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Unsupported auth type"));
    }

    #[test]
    fn test_authentication_data_creation() {
        let mut credentials = HashMap::default();
        credentials.insert("token".to_string(), "test_token".to_string());
        credentials.insert("user_id".to_string(), "test_user".to_string());

        let auth_data = AuthenticationData {
            auth_type: "privy".to_string(),
            credentials,
            network: "mainnet".to_string(),
        };

        assert_eq!(auth_data.auth_type, "privy");
        assert_eq!(auth_data.network, "mainnet");
        assert_eq!(
            auth_data.credentials.get("token"),
            Some(&"test_token".to_string())
        );
        assert_eq!(
            auth_data.credentials.get("user_id"),
            Some(&"test_user".to_string())
        );
    }

    #[test]
    fn test_composite_factory_new() {
        let factory = CompositeSignerFactory::default();
        assert!(factory.get_registered_auth_types().is_empty());
        assert_eq!(factory.factories.len(), 0);
    }

    #[test]
    fn test_add_factory_method() {
        let mut composite = CompositeSignerFactory::default();
        let mock_factory = std::sync::Arc::new(MockSignerFactory);

        composite.add_factory("mock".to_string(), mock_factory);

        let registered_types = composite.get_registered_auth_types();
        assert_eq!(registered_types.len(), 1);
        assert!(registered_types.contains(&"mock".to_string()));
    }

    #[tokio::test]
    async fn test_supported_auth_types_with_multiple_factories() {
        let mut composite = CompositeSignerFactory::default();

        // Mock factory that returns multiple auth types
        struct MultiAuthFactory;

        #[async_trait]
        impl SignerFactory for MultiAuthFactory {
            async fn create_signer(
                &self,
                _auth_data: AuthenticationData,
            ) -> Result<Box<dyn UnifiedSigner>, Box<dyn std::error::Error + Send + Sync>>
            {
                let keypair = Keypair::new();
                let signer =
                    LocalSolanaSigner::new(keypair, "https://api.devnet.solana.com".to_string());
                Ok(Box::new(signer))
            }

            fn supported_auth_types(&self) -> Vec<String> {
                vec!["auth1".to_string(), "auth2".to_string()]
            }
        }

        composite.add_factory("multi".to_string(), std::sync::Arc::new(MultiAuthFactory));
        composite.add_factory("mock".to_string(), std::sync::Arc::new(MockSignerFactory));

        let all_types = composite.supported_auth_types();
        assert!(all_types.len() >= 3); // At least auth1, auth2, and mock
        assert!(all_types.contains(&"auth1".to_string()));
        assert!(all_types.contains(&"auth2".to_string()));
        assert!(all_types.contains(&"mock".to_string()));
    }

    #[test]
    fn test_authentication_data_clone() {
        let mut credentials = HashMap::default();
        credentials.insert("key".to_string(), "value".to_string());

        let auth_data = AuthenticationData {
            auth_type: "test".to_string(),
            credentials,
            network: "testnet".to_string(),
        };

        let cloned_data = auth_data.clone();
        assert_eq!(auth_data.auth_type, cloned_data.auth_type);
        assert_eq!(auth_data.network, cloned_data.network);
        assert_eq!(auth_data.credentials, cloned_data.credentials);
    }

    #[test]
    fn test_authentication_data_debug() {
        let auth_data = AuthenticationData {
            auth_type: "debug_test".to_string(),
            credentials: HashMap::default(),
            network: "debug_net".to_string(),
        };

        let debug_string = format!("{:?}", auth_data);
        assert!(debug_string.contains("debug_test"));
        assert!(debug_string.contains("debug_net"));
    }

    #[test]
    fn test_authentication_data_with_empty_credentials() {
        let auth_data = AuthenticationData {
            auth_type: "empty".to_string(),
            credentials: HashMap::default(),
            network: "test".to_string(),
        };

        assert_eq!(auth_data.auth_type, "empty");
        assert_eq!(auth_data.network, "test");
        assert!(auth_data.credentials.is_empty());
    }

    #[test]
    fn test_authentication_data_with_empty_strings() {
        let auth_data = AuthenticationData {
            auth_type: "".to_string(),
            credentials: HashMap::default(),
            network: "".to_string(),
        };

        assert_eq!(auth_data.auth_type, "");
        assert_eq!(auth_data.network, "");
        assert!(auth_data.credentials.is_empty());
    }

    #[tokio::test]
    async fn test_composite_factory_with_no_registered_factories() {
        let composite = CompositeSignerFactory::default();

        let auth_data = AuthenticationData {
            auth_type: "nonexistent".to_string(),
            credentials: HashMap::default(),
            network: "test".to_string(),
        };

        let result = composite.create_signer(auth_data).await;

        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Unsupported auth type: nonexistent"));
    }

    #[tokio::test]
    async fn test_composite_factory_with_multiple_registered_factories() {
        let mut composite = CompositeSignerFactory::default();

        // Register multiple factories
        composite.register_factory("factory1".to_string(), Box::new(MockSignerFactory));
        composite.add_factory(
            "factory2".to_string(),
            std::sync::Arc::new(MockSignerFactory),
        );

        let registered_types = composite.get_registered_auth_types();
        assert_eq!(registered_types.len(), 2);
        assert!(registered_types.contains(&"factory1".to_string()));
        assert!(registered_types.contains(&"factory2".to_string()));

        // Test creating signer with first factory
        let auth_data1 = AuthenticationData {
            auth_type: "factory1".to_string(),
            credentials: HashMap::default(),
            network: "test".to_string(),
        };

        let result1 = composite.create_signer(auth_data1).await;
        assert!(result1.is_ok());

        // Test creating signer with second factory
        let auth_data2 = AuthenticationData {
            auth_type: "factory2".to_string(),
            credentials: HashMap::default(),
            network: "test".to_string(),
        };

        let result2 = composite.create_signer(auth_data2).await;
        assert!(result2.is_ok());
    }

    #[test]
    fn test_get_registered_auth_types_empty() {
        let composite = CompositeSignerFactory::default();
        let types = composite.get_registered_auth_types();
        assert!(types.is_empty());
    }

    #[test]
    fn test_supported_auth_types_empty() {
        let composite = CompositeSignerFactory::default();
        let types = composite.supported_auth_types();
        assert!(types.is_empty());
    }

    #[test]
    fn test_authentication_data_with_special_characters() {
        let mut credentials = HashMap::default();
        credentials.insert("sp3c!@l".to_string(), "v@lue#123".to_string());

        let auth_data = AuthenticationData {
            auth_type: "sp3c!@l_type".to_string(),
            credentials,
            network: "n3tw0rk_#123".to_string(),
        };

        assert_eq!(auth_data.auth_type, "sp3c!@l_type");
        assert_eq!(auth_data.network, "n3tw0rk_#123");
        assert_eq!(
            auth_data.credentials.get("sp3c!@l"),
            Some(&"v@lue#123".to_string())
        );
    }
}
