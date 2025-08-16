//! Integration tests for authentication providers

#[cfg(test)]
mod tests {
    use riglr_auth::{AuthProvider, MagicConfig, PrivyConfig, Web3AuthConfig};
    use riglr_core::config::RpcConfig;
    use riglr_web_adapters::factory::{AuthenticationData, SignerFactory};
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_privy_provider_creation() {
        let config = PrivyConfig::new("test_app_id".to_string(), "test_app_secret".to_string());

        let provider = AuthProvider::privy(config);
        let auth_types = provider.supported_auth_types();

        assert_eq!(auth_types, vec!["privy"]);
    }

    #[tokio::test]
    async fn test_web3auth_provider_creation() {
        let config = Web3AuthConfig::new("test_client_id".to_string(), "test_verifier".to_string());

        let provider = AuthProvider::web3auth(config);
        let auth_types = provider.supported_auth_types();

        assert_eq!(auth_types, vec!["web3auth"]);
    }

    #[tokio::test]
    async fn test_magic_provider_creation() {
        let config = MagicConfig::new(
            "test_publishable_key".to_string(),
            "test_secret_key".to_string(),
        );

        let provider = AuthProvider::magic(config);
        let auth_types = provider.supported_auth_types();

        assert_eq!(auth_types, vec!["magic"]);
    }

    #[tokio::test]
    async fn test_privy_missing_token_error() {
        let config = PrivyConfig::new("test_app_id".to_string(), "test_app_secret".to_string());

        let provider = AuthProvider::privy(config);

        let auth_data = AuthenticationData {
            auth_type: "privy".to_string(),
            credentials: HashMap::new(), // No token
            network: "mainnet".to_string(),
        };

        let rpc_config = RpcConfig::default();
        let result = provider.create_signer(auth_data, &rpc_config).await;

        assert!(result.is_err());
        let error = result.unwrap_err().to_string();
        assert!(error.contains("Missing required credential") || error.contains("token"));
    }

    #[tokio::test]
    async fn test_composite_factory_with_auth_providers() {
        use riglr_auth::CompositeSignerFactoryExt;
        use riglr_web_adapters::factory::CompositeSignerFactory;

        let mut factory = CompositeSignerFactory::new();

        // Register Privy provider
        let privy_config =
            PrivyConfig::new("test_app_id".to_string(), "test_app_secret".to_string());
        factory.register_provider(AuthProvider::privy(privy_config));

        // Register Web3Auth provider
        let web3auth_config =
            Web3AuthConfig::new("test_client_id".to_string(), "test_verifier".to_string());
        factory.register_provider(AuthProvider::web3auth(web3auth_config));

        // Check registered types
        let types = factory.get_registered_auth_types();
        assert!(types.contains(&"privy".to_string()));
        assert!(types.contains(&"web3auth".to_string()));
    }

    #[test]
    fn test_config_validation() {
        use riglr_auth::config::ProviderConfig;

        // Valid Privy config
        let valid_privy = PrivyConfig::new("app_id".to_string(), "app_secret".to_string());
        assert!(valid_privy.validate().is_ok());

        // Invalid Privy config (empty app_id)
        let invalid_privy = PrivyConfig::new("".to_string(), "app_secret".to_string());
        assert!(invalid_privy.validate().is_err());

        // Valid Web3Auth config
        let valid_web3auth = Web3AuthConfig::new("client_id".to_string(), "verifier".to_string());
        assert!(valid_web3auth.validate().is_ok());

        // Invalid Web3Auth config (empty verifier)
        let invalid_web3auth = Web3AuthConfig::new("client_id".to_string(), "".to_string());
        assert!(invalid_web3auth.validate().is_err());

        // Valid Magic config
        let valid_magic = MagicConfig::new("publishable_key".to_string(), "secret_key".to_string());
        assert!(valid_magic.validate().is_ok());

        // Invalid Magic config (empty secret_key)
        let invalid_magic = MagicConfig::new("publishable_key".to_string(), "".to_string());
        assert!(invalid_magic.validate().is_err());
    }

    #[test]
    fn test_auth_error_retriable() {
        use riglr_auth::AuthError;

        let api_error = AuthError::ApiError("Network timeout".to_string());
        assert!(api_error.is_retriable());

        let validation_error = AuthError::TokenValidation("Invalid token".to_string());
        assert!(!validation_error.is_retriable());

        let config_error = AuthError::ConfigError("Missing config".to_string());
        assert!(!config_error.is_retriable());
    }
}
