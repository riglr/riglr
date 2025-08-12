//! SignerFactory trait for abstracting authentication and signer creation
//!
//! This module provides the core abstraction for creating signers from HTTP request
//! authentication data. It enables web adapters to work with any authentication
//! provider by implementing the SignerFactory trait.

use riglr_core::signer::TransactionSigner;
use riglr_core::config::RpcConfig;
use async_trait::async_trait;
use std::collections::HashMap;

/// Abstract factory for creating signers from authentication data
#[async_trait]
pub trait SignerFactory: Send + Sync {
    /// Create a signer from authentication data
    /// 
    /// # Arguments
    /// * `auth_data` - Authentication data extracted from HTTP request
    /// * `config` - RPC configuration for blockchain networks
    /// 
    /// # Returns
    /// A boxed transaction signer or an error
    async fn create_signer(
        &self,
        auth_data: AuthenticationData,
        config: &RpcConfig,
    ) -> Result<Box<dyn TransactionSigner>, Box<dyn std::error::Error + Send + Sync>>;
    
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
pub struct CompositeSignerFactory {
    factories: HashMap<String, Box<dyn SignerFactory>>,
}

impl CompositeSignerFactory {
    /// Create a new composite factory
    pub fn new() -> Self {
        Self {
            factories: HashMap::new(),
        }
    }
    
    /// Register a signer factory for a specific auth type
    /// 
    /// # Arguments
    /// * `auth_type` - Authentication type identifier
    /// * `factory` - Factory implementation for this auth type
    pub fn register_factory(&mut self, auth_type: String, factory: Box<dyn SignerFactory>) {
        self.factories.insert(auth_type, factory);
    }
    
    /// Get all registered auth types
    pub fn get_registered_auth_types(&self) -> Vec<String> {
        self.factories.keys().cloned().collect()
    }
}

impl Default for CompositeSignerFactory {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SignerFactory for CompositeSignerFactory {
    async fn create_signer(
        &self,
        auth_data: AuthenticationData,
        config: &RpcConfig,
    ) -> Result<Box<dyn TransactionSigner>, Box<dyn std::error::Error + Send + Sync>> {
        let factory = self.factories.get(&auth_data.auth_type)
            .ok_or_else(|| format!("Unsupported auth type: {}", auth_data.auth_type))?;
        
        factory.create_signer(auth_data, config).await
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
    use riglr_core::signer::TransactionSigner;
    use riglr_solana_tools::signer::LocalSolanaSigner;
    use solana_sdk::signature::Keypair;
    
    // Mock signer factory for testing
    struct MockSignerFactory;
    
    #[async_trait]
    impl SignerFactory for MockSignerFactory {
        async fn create_signer(
            &self,
            _auth_data: AuthenticationData,
            _config: &RpcConfig,
        ) -> Result<Box<dyn TransactionSigner>, Box<dyn std::error::Error + Send + Sync>> {
            let keypair = Keypair::new();
            let signer = LocalSolanaSigner::new(keypair, "https://api.devnet.solana.com".to_string());
            Ok(Box::new(signer))
        }
        
        fn supported_auth_types(&self) -> Vec<String> {
            vec!["mock".to_string()]
        }
    }
    
    #[tokio::test]
    async fn test_composite_factory_creation() {
        let factory = CompositeSignerFactory::new();
        assert!(factory.get_registered_auth_types().is_empty());
    }
    
    #[tokio::test]
    async fn test_factory_registration() {
        let mut composite = CompositeSignerFactory::new();
        let mock_factory = Box::new(MockSignerFactory);
        
        composite.register_factory("mock".to_string(), mock_factory);
        
        let registered_types = composite.get_registered_auth_types();
        assert_eq!(registered_types.len(), 1);
        assert!(registered_types.contains(&"mock".to_string()));
    }
    
    #[tokio::test]
    async fn test_signer_creation() {
        let mut composite = CompositeSignerFactory::new();
        let mock_factory = Box::new(MockSignerFactory);
        
        composite.register_factory("mock".to_string(), mock_factory);
        
        let auth_data = AuthenticationData {
            auth_type: "mock".to_string(),
            credentials: [("token".to_string(), "test_token".to_string())].into(),
            network: "devnet".to_string(),
        };
        
        let config = RpcConfig::default();
        let result = composite.create_signer(auth_data, &config).await;
        
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_unsupported_auth_type() {
        let composite = CompositeSignerFactory::new();
        
        let auth_data = AuthenticationData {
            auth_type: "unsupported".to_string(),
            credentials: HashMap::new(),
            network: "devnet".to_string(),
        };
        
        let config = RpcConfig::default();
        let result = composite.create_signer(auth_data, &config).await;
        
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Unsupported auth type"));
    }
    
    #[test]
    fn test_authentication_data_creation() {
        let mut credentials = HashMap::new();
        credentials.insert("token".to_string(), "test_token".to_string());
        credentials.insert("user_id".to_string(), "test_user".to_string());
        
        let auth_data = AuthenticationData {
            auth_type: "privy".to_string(),
            credentials,
            network: "mainnet".to_string(),
        };
        
        assert_eq!(auth_data.auth_type, "privy");
        assert_eq!(auth_data.network, "mainnet");
        assert_eq!(auth_data.credentials.get("token"), Some(&"test_token".to_string()));
        assert_eq!(auth_data.credentials.get("user_id"), Some(&"test_user".to_string()));
    }
}