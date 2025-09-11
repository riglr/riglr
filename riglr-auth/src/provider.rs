//! Core authentication provider abstraction

use crate::error::AuthResult;
use async_trait::async_trait;
use riglr_core::signer::UnifiedSigner;
// NOTE: Temporarily defining these types locally to avoid circular dependency
use std::collections::HashMap;

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

/// Abstract factory for creating signers from authentication data
#[async_trait]
pub trait SignerFactory: Send + Sync + std::fmt::Debug {
    /// Create a signer from authentication data
    async fn create_signer(
        &self,
        auth_data: AuthenticationData,
    ) -> Result<Box<dyn UnifiedSigner>, Box<dyn std::error::Error + Send + Sync>>;

    /// Get list of supported authentication types
    fn supported_auth_types(&self) -> Vec<String>;
}

/// Authentication provider types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthProviderType {
    /// Privy authentication provider
    Privy,
    /// Web3Auth authentication provider
    Web3Auth,
    /// Magic.link authentication provider
    Magic,
    /// Custom authentication provider with a name
    Custom(String),
}

impl AuthProviderType {
    /// Get the string identifier for this provider type
    pub fn as_str(&self) -> &str {
        match self {
            Self::Privy => "privy",
            Self::Web3Auth => "web3auth",
            Self::Magic => "magic",
            Self::Custom(s) => s,
        }
    }
}

/// Main authentication provider wrapper
#[derive(Debug)]
pub struct AuthProvider {
    provider_type: AuthProviderType,
    inner: Box<dyn SignerFactory>,
}

impl AuthProvider {
    /// Create a new authentication provider
    pub fn new(provider_type: AuthProviderType, inner: Box<dyn SignerFactory>) -> Self {
        Self {
            provider_type,
            inner,
        }
    }

    /// Get the authentication type string
    pub fn auth_type(&self) -> String {
        self.provider_type.as_str().to_string()
    }

    /// Create a Privy authentication provider
    #[cfg(feature = "privy")]
    pub fn privy(config: crate::privy::PrivyConfig) -> Self {
        Self::new(
            AuthProviderType::Privy,
            Box::new(crate::privy::PrivyProvider::new(config)),
        )
    }

    /// Create a Web3Auth authentication provider
    #[cfg(feature = "web3auth")]
    pub fn web3auth(config: crate::web3auth::Web3AuthConfig) -> Self {
        Self::new(
            AuthProviderType::Web3Auth,
            Box::new(crate::web3auth::Web3AuthProvider::new(config)),
        )
    }

    /// Create a Magic.link authentication provider
    #[cfg(feature = "magic")]
    pub fn magic(config: crate::magic::MagicConfig) -> Self {
        Self::new(
            AuthProviderType::Magic,
            Box::new(crate::magic::MagicProvider::new(config)),
        )
    }
}

#[async_trait]
impl SignerFactory for AuthProvider {
    async fn create_signer(
        &self,
        auth_data: AuthenticationData,
    ) -> Result<Box<dyn UnifiedSigner>, Box<dyn std::error::Error + Send + Sync>> {
        self.inner.create_signer(auth_data).await
    }

    fn supported_auth_types(&self) -> Vec<String> {
        self.inner.supported_auth_types()
    }
}

/// Base trait for authentication providers with additional functionality
#[async_trait]
pub trait AuthenticationProvider: SignerFactory {
    /// Validate a token and return user information
    async fn validate_token(&self, token: &str) -> AuthResult<UserInfo>;

    /// Refresh a token if supported
    async fn refresh_token(&self, _token: &str) -> AuthResult<String> {
        Err(crate::AuthError::UnsupportedOperation(
            "Token refresh not supported".to_string(),
        ))
    }

    /// Revoke a token if supported
    async fn revoke_token(&self, _token: &str) -> AuthResult<()> {
        Err(crate::AuthError::UnsupportedOperation(
            "Token revocation not supported".to_string(),
        ))
    }
}

/// User information returned from authentication providers
#[derive(Debug, Clone)]
pub struct UserInfo {
    /// Unique user identifier
    pub id: String,

    /// Email address if available
    pub email: Option<String>,

    /// Solana wallet address if available
    pub solana_address: Option<String>,

    /// EVM wallet address if available
    pub evm_address: Option<String>,

    /// Whether the user is verified
    pub verified: bool,

    /// Additional metadata
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use riglr_core::signer::granular_traits::{
        EvmSigner, MultiChainSigner, SignerBase, SolanaSigner,
    };
    use riglr_core::signer::UnifiedSigner;
    use std::collections::HashMap;

    // Mock SignerFactory for testing
    #[derive(Debug)]
    struct MockSignerFactory {
        auth_types: Vec<String>,
        should_error: bool,
    }

    impl MockSignerFactory {
        fn new(auth_types: Vec<String>) -> Self {
            Self {
                auth_types,
                should_error: false,
            }
        }

        fn with_error() -> Self {
            Self {
                auth_types: vec!["test".to_string()],
                should_error: true,
            }
        }
    }

    // Mock UnifiedSigner for testing
    #[derive(Debug)]
    struct MockUnifiedSigner;

    impl SignerBase for MockUnifiedSigner {
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
    }

    impl UnifiedSigner for MockUnifiedSigner {
        fn supports_solana(&self) -> bool {
            false
        }

        fn supports_evm(&self) -> bool {
            false
        }

        fn as_solana(&self) -> Option<&dyn SolanaSigner> {
            None
        }

        fn as_evm(&self) -> Option<&dyn EvmSigner> {
            None
        }

        fn as_multi_chain(&self) -> Option<&dyn MultiChainSigner> {
            None
        }
    }

    #[async_trait]
    impl SignerFactory for MockSignerFactory {
        async fn create_signer(
            &self,
            _auth_data: AuthenticationData,
        ) -> Result<Box<dyn UnifiedSigner>, Box<dyn std::error::Error + Send + Sync>> {
            if self.should_error {
                Err("Mock error".into())
            } else {
                Ok(Box::new(MockUnifiedSigner))
            }
        }

        fn supported_auth_types(&self) -> Vec<String> {
            self.auth_types.clone()
        }
    }

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
        let custom_name = "custom_provider".to_string();
        let provider_type = AuthProviderType::Custom(custom_name.clone());
        assert_eq!(provider_type.as_str(), &custom_name);
    }

    #[test]
    fn test_auth_provider_type_debug_should_format_correctly() {
        assert_eq!(format!("{:?}", AuthProviderType::Privy), "Privy");
        assert_eq!(format!("{:?}", AuthProviderType::Web3Auth), "Web3Auth");
        assert_eq!(format!("{:?}", AuthProviderType::Magic), "Magic");
        assert_eq!(
            format!("{:?}", AuthProviderType::Custom("test".to_string())),
            "Custom(\"test\")"
        );
    }

    #[test]
    fn test_auth_provider_type_clone_should_create_identical_copy() {
        let original = AuthProviderType::Custom("test".to_string());
        let cloned = original.clone();
        assert_eq!(original, cloned);
    }

    #[test]
    fn test_auth_provider_type_partial_eq_when_same_should_be_equal() {
        assert_eq!(AuthProviderType::Privy, AuthProviderType::Privy);
        assert_eq!(AuthProviderType::Web3Auth, AuthProviderType::Web3Auth);
        assert_eq!(AuthProviderType::Magic, AuthProviderType::Magic);
        assert_eq!(
            AuthProviderType::Custom("test".to_string()),
            AuthProviderType::Custom("test".to_string())
        );
    }

    #[test]
    fn test_auth_provider_type_partial_eq_when_different_should_not_be_equal() {
        assert_ne!(AuthProviderType::Privy, AuthProviderType::Web3Auth);
        assert_ne!(AuthProviderType::Privy, AuthProviderType::Magic);
        assert_ne!(
            AuthProviderType::Custom("test1".to_string()),
            AuthProviderType::Custom("test2".to_string())
        );
        assert_ne!(
            AuthProviderType::Privy,
            AuthProviderType::Custom("privy".to_string())
        );
    }

    #[test]
    fn test_auth_provider_new_should_create_provider_with_correct_fields() {
        let provider_type = AuthProviderType::Privy;
        let mock_factory = MockSignerFactory::new(vec!["test".to_string()]);
        let provider = AuthProvider::new(provider_type.clone(), Box::new(mock_factory));

        assert_eq!(provider.provider_type, provider_type);
        assert_eq!(provider.auth_type(), "privy");
    }

    #[test]
    fn test_auth_provider_auth_type_when_privy_should_return_privy_string() {
        let mock_factory = MockSignerFactory::new(vec!["test".to_string()]);
        let provider = AuthProvider::new(AuthProviderType::Privy, Box::new(mock_factory));
        assert_eq!(provider.auth_type(), "privy");
    }

    #[test]
    fn test_auth_provider_auth_type_when_web3auth_should_return_web3auth_string() {
        let mock_factory = MockSignerFactory::new(vec!["test".to_string()]);
        let provider = AuthProvider::new(AuthProviderType::Web3Auth, Box::new(mock_factory));
        assert_eq!(provider.auth_type(), "web3auth");
    }

    #[test]
    fn test_auth_provider_auth_type_when_magic_should_return_magic_string() {
        let mock_factory = MockSignerFactory::new(vec!["test".to_string()]);
        let provider = AuthProvider::new(AuthProviderType::Magic, Box::new(mock_factory));
        assert_eq!(provider.auth_type(), "magic");
    }

    #[test]
    fn test_auth_provider_auth_type_when_custom_should_return_custom_string() {
        let custom_name = "my_custom_provider";
        let mock_factory = MockSignerFactory::new(vec!["test".to_string()]);
        let provider = AuthProvider::new(
            AuthProviderType::Custom(custom_name.to_string()),
            Box::new(mock_factory),
        );
        assert_eq!(provider.auth_type(), custom_name);
    }

    #[tokio::test]
    async fn test_auth_provider_create_signer_when_success_should_return_signer() {
        let mock_factory = MockSignerFactory::new(vec!["test".to_string()]);
        let provider = AuthProvider::new(AuthProviderType::Privy, Box::new(mock_factory));

        let mut credentials = std::collections::HashMap::new();
        credentials.insert("token".to_string(), "test_token".to_string());
        credentials.insert("user_id".to_string(), "test_user".to_string());

        let auth_data = AuthenticationData {
            auth_type: "test".to_string(),
            credentials,
            network: "mainnet".to_string(),
        };

        let result = provider.create_signer(auth_data).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_auth_provider_create_signer_when_error_should_return_error() {
        let mock_factory = MockSignerFactory::with_error();
        let provider = AuthProvider::new(AuthProviderType::Privy, Box::new(mock_factory));

        let mut credentials = std::collections::HashMap::new();
        credentials.insert("token".to_string(), "test_token".to_string());
        credentials.insert("user_id".to_string(), "test_user".to_string());

        let auth_data = AuthenticationData {
            auth_type: "test".to_string(),
            credentials,
            network: "mainnet".to_string(),
        };

        let result = provider.create_signer(auth_data).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_auth_provider_supported_auth_types_should_return_inner_types() {
        let expected_types = vec!["type1".to_string(), "type2".to_string()];
        let mock_factory = MockSignerFactory::new(expected_types.clone());
        let provider = AuthProvider::new(AuthProviderType::Privy, Box::new(mock_factory));

        let result = provider.supported_auth_types();
        assert_eq!(result, expected_types);
    }

    #[test]
    fn test_auth_provider_supported_auth_types_when_empty_should_return_empty_vec() {
        let mock_factory = MockSignerFactory::new(vec![]);
        let provider = AuthProvider::new(AuthProviderType::Privy, Box::new(mock_factory));

        let result = provider.supported_auth_types();
        assert!(result.is_empty());
    }

    #[test]
    fn test_user_info_debug_should_format_correctly() {
        let mut metadata = HashMap::new();
        metadata.insert(
            "key".to_string(),
            serde_json::Value::String("value".to_string()),
        );

        let user_info = UserInfo {
            id: "test_id".to_string(),
            email: Some("test@example.com".to_string()),
            solana_address: Some("solana_addr".to_string()),
            evm_address: Some("evm_addr".to_string()),
            verified: true,
            metadata,
        };

        let debug_str = format!("{:?}", user_info);
        assert!(debug_str.contains("test_id"));
        assert!(debug_str.contains("test@example.com"));
        assert!(debug_str.contains("solana_addr"));
        assert!(debug_str.contains("evm_addr"));
        assert!(debug_str.contains("true"));
    }

    #[test]
    fn test_user_info_clone_should_create_identical_copy() {
        let mut metadata = HashMap::new();
        metadata.insert(
            "key".to_string(),
            serde_json::Value::String("value".to_string()),
        );

        let original = UserInfo {
            id: "test_id".to_string(),
            email: Some("test@example.com".to_string()),
            solana_address: None,
            evm_address: None,
            verified: false,
            metadata,
        };

        let cloned = original.clone();
        assert_eq!(original.id, cloned.id);
        assert_eq!(original.email, cloned.email);
        assert_eq!(original.solana_address, cloned.solana_address);
        assert_eq!(original.evm_address, cloned.evm_address);
        assert_eq!(original.verified, cloned.verified);
        assert_eq!(original.metadata.len(), cloned.metadata.len());
    }

    #[test]
    fn test_user_info_with_none_values_should_work() {
        let user_info = UserInfo {
            id: "test_id".to_string(),
            email: None,
            solana_address: None,
            evm_address: None,
            verified: false,
            metadata: HashMap::new(),
        };

        assert_eq!(user_info.id, "test_id");
        assert!(user_info.email.is_none());
        assert!(user_info.solana_address.is_none());
        assert!(user_info.evm_address.is_none());
        assert!(!user_info.verified);
        assert!(user_info.metadata.is_empty());
    }

    #[test]
    fn test_user_info_with_all_values_should_work() {
        let mut metadata = HashMap::new();
        metadata.insert(
            "role".to_string(),
            serde_json::Value::String("admin".to_string()),
        );
        metadata.insert(
            "score".to_string(),
            serde_json::Value::Number(serde_json::Number::from(100)),
        );

        let user_info = UserInfo {
            id: "user123".to_string(),
            email: Some("admin@example.com".to_string()),
            solana_address: Some("So11111111111111111111111111111111111111112".to_string()),
            evm_address: Some("0x742d35Cc6634C0532925a3b8D36C9FC2F2373458".to_string()),
            verified: true,
            metadata,
        };

        assert_eq!(user_info.id, "user123");
        assert_eq!(user_info.email.as_ref().unwrap(), "admin@example.com");
        assert_eq!(
            user_info.solana_address.as_ref().unwrap(),
            "So11111111111111111111111111111111111111112"
        );
        assert_eq!(
            user_info.evm_address.as_ref().unwrap(),
            "0x742d35Cc6634C0532925a3b8D36C9FC2F2373458"
        );
        assert!(user_info.verified);
        assert_eq!(user_info.metadata.len(), 2);
    }

    // Mock AuthenticationProvider for testing default implementations
    #[derive(Debug)]
    struct MockAuthProvider;

    #[async_trait]
    impl SignerFactory for MockAuthProvider {
        async fn create_signer(
            &self,
            _auth_data: AuthenticationData,
        ) -> Result<Box<dyn UnifiedSigner>, Box<dyn std::error::Error + Send + Sync>> {
            Ok(Box::new(MockUnifiedSigner))
        }

        fn supported_auth_types(&self) -> Vec<String> {
            vec!["test".to_string()]
        }
    }

    #[async_trait]
    impl AuthenticationProvider for MockAuthProvider {
        async fn validate_token(&self, _token: &str) -> crate::error::AuthResult<UserInfo> {
            Ok(UserInfo {
                id: "test_user".to_string(),
                email: None,
                solana_address: None,
                evm_address: None,
                verified: true,
                metadata: HashMap::new(),
            })
        }
    }

    #[tokio::test]
    async fn test_authentication_provider_refresh_token_default_should_return_unsupported_error() {
        let provider = MockAuthProvider;
        let result = provider.refresh_token("test_token").await;

        assert!(result.is_err());
        match result.unwrap_err() {
            crate::AuthError::UnsupportedOperation(msg) => {
                assert_eq!(msg, "Token refresh not supported");
            }
            _ => panic!("Expected UnsupportedOperation error"),
        }
    }

    #[tokio::test]
    async fn test_authentication_provider_revoke_token_default_should_return_unsupported_error() {
        let provider = MockAuthProvider;
        let result = provider.revoke_token("test_token").await;

        assert!(result.is_err());
        match result.unwrap_err() {
            crate::AuthError::UnsupportedOperation(msg) => {
                assert_eq!(msg, "Token revocation not supported");
            }
            _ => panic!("Expected UnsupportedOperation error"),
        }
    }

    #[tokio::test]
    async fn test_authentication_provider_validate_token_should_work() {
        let provider = MockAuthProvider;
        let result = provider.validate_token("test_token").await;

        assert!(result.is_ok());
        let user_info = result.unwrap();
        assert_eq!(user_info.id, "test_user");
        assert!(user_info.verified);
    }
}
