//! Core authentication provider abstraction

use crate::error::AuthResult;
use async_trait::async_trait;
use riglr_core::config::RpcConfig;
use riglr_core::signer::TransactionSigner;
use riglr_web_adapters::factory::{AuthenticationData, SignerFactory};

/// Authentication provider types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthProviderType {
    Privy,
    Web3Auth,
    Magic,
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
        config: &RpcConfig,
    ) -> Result<Box<dyn TransactionSigner>, Box<dyn std::error::Error + Send + Sync>> {
        self.inner.create_signer(auth_data, config).await
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
