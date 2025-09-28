//! Configuration types for authentication providers

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Base configuration for authentication providers
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[expect(clippy::module_name_repetitions)]
pub struct AuthConfig {
    /// Cache configuration
    #[serde(default)]
    pub cache_config: CacheConfig,

    /// Network-specific overrides
    #[serde(default)]
    pub network_overrides: HashMap<String, NetworkOverride>,

    /// Provider-specific configuration
    pub provider_config: HashMap<String, String>,
}

/// Cache configuration for authentication providers
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[expect(clippy::module_name_repetitions)]
pub struct CacheConfig {
    /// Enable caching of user data
    #[serde(default = "default_cache_enabled")]
    pub enabled: bool,

    /// Maximum cache size
    #[serde(default = "default_cache_size")]
    pub max_size: usize,

    /// Cache TTL in seconds
    #[serde(default = "default_cache_ttl")]
    pub ttl_seconds: u64,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled: default_cache_enabled(),
            max_size: default_cache_size(),
            ttl_seconds: default_cache_ttl(),
        }
    }
}
const fn default_cache_enabled() -> bool {
    true
}
const fn default_cache_ttl() -> u64 {
    300
} // 5 minutes
const fn default_cache_size() -> usize {
    1000
}

/// Network-specific configuration overrides
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NetworkOverride {
    /// Override chain ID
    pub chain_id: Option<u64>,

    /// Custom parameters
    #[serde(default)]
    pub custom_params: HashMap<String, String>,

    /// Override RPC URL for this network
    pub rpc_url: Option<String>,
}

/// Common trait for provider-specific configurations
#[expect(clippy::module_name_repetitions)]
pub trait ProviderConfig: Send + Sync {
    /// Create from environment variables
    ///
    /// # Errors
    ///
    /// Returns error if required environment variables are missing or invalid
    fn from_env() -> Result<Self, crate::AuthError>
    where
        Self: Sized;

    /// Get provider name
    fn provider_name(&self) -> &'static str;

    /// Validate the configuration
    ///
    /// # Errors
    ///
    /// Returns error if the configuration is invalid or required fields are missing
    fn validate(&self) -> Result<(), crate::AuthError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AuthError;
    use std::collections::HashMap;

    // Helper test implementation of ProviderConfig
    #[derive(Debug, Clone)]
    struct TestProviderConfig {
        name: &'static str,
        valid: bool,
    }

    impl ProviderConfig for TestProviderConfig {
        fn from_env() -> Result<Self, AuthError> {
            Ok(Self {
                name: "test",
                valid: true,
            })
        }
        fn provider_name(&self) -> &'static str {
            self.name
        }
        fn validate(&self) -> Result<(), AuthError> {
            if self.valid {
                Ok(())
            } else {
                Err(AuthError::ConfigError("Invalid test config".to_string()))
            }
        }
    }

    #[test]
    fn test_auth_config_creation() {
        let mut provider_config = HashMap::new();
        provider_config.insert("key1".to_string(), "value1".to_string());
        provider_config.insert("key2".to_string(), "value2".to_string());

        let mut network_overrides = HashMap::new();
        network_overrides.insert(
            "mainnet".to_string(),
            NetworkOverride {
                chain_id: Some(1),
                rpc_url: Some("https://mainnet.example.com".to_string()),
                custom_params: HashMap::new(),
            },
        );

        let config = AuthConfig {
            cache_config: CacheConfig::default(),
            provider_config: provider_config.clone(),
            network_overrides: network_overrides.clone(),
        };

        assert_eq!(config.provider_config, provider_config);
        assert_eq!(config.network_overrides, network_overrides);
    }

    #[test]
    fn test_auth_config_empty() {
        let config = AuthConfig {
            cache_config: CacheConfig::default(),
            provider_config: HashMap::new(),
            network_overrides: HashMap::new(),
        };

        assert!(config.provider_config.is_empty());
        assert!(config.network_overrides.is_empty());
    }

    #[test]
    fn test_cache_config_default() {
        let config = CacheConfig::default();

        assert!(config.enabled);
        assert_eq!(config.ttl_seconds, 300);
        assert_eq!(config.max_size, 1000);
    }

    #[test]
    fn test_cache_config_custom_values() {
        let config = CacheConfig {
            enabled: false,
            max_size: 2000,
            ttl_seconds: 600,
        };

        assert!(!config.enabled);
        assert_eq!(config.ttl_seconds, 600);
        assert_eq!(config.max_size, 2000);
    }

    #[test]
    fn test_default_cache_enabled() {
        assert!(default_cache_enabled());
    }

    #[test]
    fn test_default_cache_ttl() {
        assert_eq!(default_cache_ttl(), 300);
    }

    #[test]
    fn test_default_cache_size() {
        assert_eq!(default_cache_size(), 1000);
    }

    #[test]
    fn test_network_override_with_all_fields() {
        let mut custom_params = HashMap::new();
        custom_params.insert("param1".to_string(), "value1".to_string());
        custom_params.insert("param2".to_string(), "value2".to_string());

        let override_config = NetworkOverride {
            chain_id: Some(42),
            rpc_url: Some("https://example.com/rpc".to_string()),
            custom_params: custom_params.clone(),
        };

        assert_eq!(
            override_config.rpc_url,
            Some("https://example.com/rpc".to_string())
        );
        assert_eq!(override_config.chain_id, Some(42));
        assert_eq!(override_config.custom_params, custom_params);
    }

    #[test]
    fn test_network_override_with_none_values() {
        let override_config = NetworkOverride {
            chain_id: None,
            rpc_url: None,
            custom_params: HashMap::new(),
        };

        assert_eq!(override_config.rpc_url, None);
        assert_eq!(override_config.chain_id, None);
        assert!(override_config.custom_params.is_empty());
    }

    #[test]
    fn test_network_override_with_empty_custom_params() {
        let override_config = NetworkOverride {
            chain_id: Some(1),
            rpc_url: Some("https://test.com".to_string()),
            custom_params: HashMap::new(),
        };

        assert!(override_config.custom_params.is_empty());
    }

    #[test]
    fn test_provider_config_trait_validate_success() {
        let config = TestProviderConfig {
            name: "test_provider",
            valid: true,
        };

        assert!(config.validate().is_ok());
    }

    #[test]
    #[expect(clippy::panic)]
    fn test_provider_config_trait_validate_failure() {
        let config = TestProviderConfig {
            name: "test_provider",
            valid: false,
        };

        let result = config.validate();
        assert!(result.is_err());
        match result {
            Err(AuthError::ConfigError(msg)) => assert_eq!(msg, "Invalid test config"),
            Err(other) => panic!("Expected ConfigError but got: {other:?}"),
            Ok(()) => panic!("Expected validation to fail"),
        }
    }

    #[test]
    fn test_provider_config_trait_provider_name() {
        let config = TestProviderConfig {
            name: "my_provider",
            valid: true,
        };

        assert_eq!(config.provider_name(), "my_provider");
    }

    #[test]
    #[expect(clippy::panic)]
    fn test_provider_config_trait_from_env() {
        let config = TestProviderConfig::from_env()
            .unwrap_or_else(|e| panic!("TestProviderConfig::from_env should succeed: {e}"));
        assert!(config.valid);
        assert_eq!(config.name, "test");
    }

    #[test]
    #[expect(clippy::panic)]
    fn test_auth_config_serialization() {
        let mut provider_config = HashMap::new();
        provider_config.insert("api_key".to_string(), "test_key".to_string());

        let config = AuthConfig {
            cache_config: CacheConfig {
                enabled: false,
                max_size: 500,
                ttl_seconds: 120,
            },
            provider_config,
            network_overrides: HashMap::new(),
        };

        let serialized = serde_json::to_string(&config)
            .unwrap_or_else(|e| panic!("Serialization should succeed: {e}"));
        let deserialized: AuthConfig = serde_json::from_str(&serialized)
            .unwrap_or_else(|e| panic!("Deserialization should succeed: {e}"));

        assert_eq!(config.provider_config, deserialized.provider_config);
        assert_eq!(
            config.cache_config.enabled,
            deserialized.cache_config.enabled
        );
        assert_eq!(
            config.cache_config.ttl_seconds,
            deserialized.cache_config.ttl_seconds
        );
        assert_eq!(
            config.cache_config.max_size,
            deserialized.cache_config.max_size
        );
    }

    #[test]
    #[expect(clippy::panic)]
    fn test_cache_config_serialization_with_defaults() {
        let config = CacheConfig {
            enabled: true,
            max_size: 1000,
            ttl_seconds: 300,
        };

        let serialized = serde_json::to_string(&config)
            .unwrap_or_else(|e| panic!("Serialization should succeed: {e}"));
        let deserialized: CacheConfig = serde_json::from_str(&serialized)
            .unwrap_or_else(|e| panic!("Deserialization should succeed: {e}"));

        assert_eq!(config.enabled, deserialized.enabled);
        assert_eq!(config.ttl_seconds, deserialized.ttl_seconds);
        assert_eq!(config.max_size, deserialized.max_size);
    }

    #[test]
    #[expect(clippy::panic)]
    fn test_cache_config_deserialization_with_missing_fields() {
        let json = "{}";
        let config: CacheConfig = serde_json::from_str(json)
            .unwrap_or_else(|e| panic!("Deserialization should succeed: {e}"));

        assert!(config.enabled);
        assert_eq!(config.ttl_seconds, 300);
        assert_eq!(config.max_size, 1000);
    }

    #[test]
    #[expect(clippy::panic)]
    fn test_network_override_serialization() {
        let mut custom_params = HashMap::new();
        custom_params.insert("timeout".to_string(), "30".to_string());

        let override_config = NetworkOverride {
            chain_id: Some(137),
            rpc_url: Some("https://rpc.example.com".to_string()),
            custom_params,
        };

        let serialized = serde_json::to_string(&override_config)
            .unwrap_or_else(|e| panic!("Serialization should succeed: {e}"));
        let deserialized: NetworkOverride = serde_json::from_str(&serialized)
            .unwrap_or_else(|e| panic!("Deserialization should succeed: {e}"));

        assert_eq!(override_config.rpc_url, deserialized.rpc_url);
        assert_eq!(override_config.chain_id, deserialized.chain_id);
        assert_eq!(override_config.custom_params, deserialized.custom_params);
    }

    #[test]
    #[expect(clippy::panic)]
    fn test_network_override_deserialization_with_missing_custom_params() {
        let json = r#"{"rpc_url": "https://test.com", "chain_id": 1}"#;
        let config: NetworkOverride = serde_json::from_str(json)
            .unwrap_or_else(|e| panic!("Deserialization should succeed: {e}"));

        assert_eq!(config.rpc_url, Some("https://test.com".to_string()));
        assert_eq!(config.chain_id, Some(1));
        assert!(config.custom_params.is_empty());
    }

    #[test]
    fn test_auth_config_debug_format() {
        let config = AuthConfig {
            cache_config: CacheConfig::default(),
            provider_config: HashMap::new(),
            network_overrides: HashMap::new(),
        };

        let debug_str = format!("{config:?}");
        assert!(debug_str.contains("AuthConfig"));
    }

    #[test]
    fn test_cache_config_debug_format() {
        let config = CacheConfig::default();
        let debug_str = format!("{config:?}");
        assert!(debug_str.contains("CacheConfig"));
    }

    #[test]
    fn test_network_override_debug_format() {
        let config = NetworkOverride {
            chain_id: None,
            rpc_url: None,
            custom_params: HashMap::new(),
        };
        let debug_str = format!("{config:?}");
        assert!(debug_str.contains("NetworkOverride"));
    }

    #[test]
    fn test_auth_config_clone() {
        let mut provider_config = HashMap::new();
        provider_config.insert("key".to_string(), "value".to_string());

        let config = AuthConfig {
            cache_config: CacheConfig::default(),
            provider_config,
            network_overrides: HashMap::new(),
        };

        let cloned = config.clone();
        assert_eq!(config.provider_config, cloned.provider_config);
        assert_eq!(config.cache_config.enabled, cloned.cache_config.enabled);
    }

    #[test]
    fn test_cache_config_clone() {
        let config = CacheConfig {
            enabled: false,
            max_size: 2000,
            ttl_seconds: 600,
        };

        let cloned = config.clone();
        assert_eq!(config.enabled, cloned.enabled);
        assert_eq!(config.ttl_seconds, cloned.ttl_seconds);
        assert_eq!(config.max_size, cloned.max_size);
    }

    #[test]
    fn test_network_override_clone() {
        let config = NetworkOverride {
            chain_id: Some(42),
            rpc_url: Some("https://test.com".to_string()),
            custom_params: HashMap::new(),
        };

        let cloned = config.clone();
        assert_eq!(config.rpc_url, cloned.rpc_url);
        assert_eq!(config.chain_id, cloned.chain_id);
        assert_eq!(config.custom_params, cloned.custom_params);
    }

    #[test]
    fn test_cache_config_extreme_values() {
        let config = CacheConfig {
            enabled: true,
            max_size: usize::MAX,
            ttl_seconds: u64::MAX,
        };

        assert_eq!(config.ttl_seconds, u64::MAX);
        assert_eq!(config.max_size, usize::MAX);
    }

    #[test]
    fn test_cache_config_zero_values() {
        let config = CacheConfig {
            enabled: false,
            max_size: 0,
            ttl_seconds: 0,
        };

        assert_eq!(config.ttl_seconds, 0);
        assert_eq!(config.max_size, 0);
    }

    #[test]
    fn test_network_override_zero_chain_id() {
        let config = NetworkOverride {
            chain_id: Some(0),
            rpc_url: Some("https://localhost:8545".to_string()),
            custom_params: HashMap::new(),
        };

        assert_eq!(config.chain_id, Some(0));
    }

    #[test]
    fn test_network_override_max_chain_id() {
        let config = NetworkOverride {
            chain_id: Some(u64::MAX),
            rpc_url: Some("https://test.com".to_string()),
            custom_params: HashMap::new(),
        };

        assert_eq!(config.chain_id, Some(u64::MAX));
    }

    #[test]
    fn test_auth_config_with_unicode_strings() {
        let mut provider_config = HashMap::new();
        provider_config.insert("unicode_key_🔑".to_string(), "unicode_value_✨".to_string());

        let config = AuthConfig {
            cache_config: CacheConfig::default(),
            provider_config: provider_config.clone(),
            network_overrides: HashMap::new(),
        };

        assert_eq!(config.provider_config, provider_config);
    }

    #[test]
    #[expect(clippy::expect_used)]
    fn test_network_override_with_unicode_rpc_url() {
        let config = NetworkOverride {
            chain_id: Some(1),
            rpc_url: Some("https://example.com/path/with/émojí🌟".to_string()),
            custom_params: HashMap::new(),
        };

        let url = config.rpc_url.as_ref().expect("RPC URL should be Some");
        assert!(url.contains("émojí🌟"));
    }

    #[test]
    fn test_empty_strings_in_provider_config() {
        let mut provider_config = HashMap::new();
        provider_config.insert(String::new(), String::new());
        provider_config.insert("empty_value".to_string(), String::new());
        provider_config.insert(String::new(), "empty_key".to_string());

        let config = AuthConfig {
            cache_config: CacheConfig::default(),
            provider_config: provider_config.clone(),
            network_overrides: HashMap::new(),
        };

        assert_eq!(config.provider_config, provider_config);
    }

    #[test]
    fn test_network_override_with_empty_rpc_url() {
        let config = NetworkOverride {
            chain_id: Some(1),
            rpc_url: Some(String::new()),
            custom_params: HashMap::new(),
        };

        assert_eq!(config.rpc_url, Some(String::new()));
    }
}
