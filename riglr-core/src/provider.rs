//! RPC Provider for read-only blockchain operations
//!
//! This module provides a centralized way to manage RPC clients for different chains,
//! enabling read-only operations without requiring signer contexts.

use dashmap::DashMap;
use riglr_config::Config;
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;

// Environment variable constants
const SOLANA_RPC_URL: &str = "SOLANA_RPC_URL";
const SOLANA_DEVNET_RPC_URL: &str = "SOLANA_DEVNET_RPC_URL";
const SOLANA_TESTNET_RPC_URL: &str = "SOLANA_TESTNET_RPC_URL";

/// RPC Provider that manages Solana RPC clients
///
/// This provider holds initialized Solana RPC clients and can be
/// shared across the application via Arc. It's designed for read-only operations
/// that don't require transaction signing.
///
/// For other blockchain networks (e.g., EVM), clients should be created and injected
/// directly into the ApplicationContext's extensions by the application.
#[derive(Clone, Default)]
pub struct RpcProvider {
    /// Solana RPC clients by network name
    solana_clients: HashMap<String, Arc<solana_client::rpc_client::RpcClient>>,
}

impl RpcProvider {
    /// Create a new RPC provider from configuration
    #[must_use]
    pub fn new() -> Self {
        Self {
            solana_clients: HashMap::new(),
        }
    }

    /// Add a Solana RPC client for a specific network
    pub fn add_solana_network(&mut self, network: String, rpc_url: String) {
        use solana_client::rpc_client::RpcClient;
        let client = Arc::new(RpcClient::new(rpc_url));
        self.solana_clients.insert(network, client);
    }

    /// Get a Solana RPC client for a specific network
    pub fn get_solana_client(
        &self,
        network: &str,
    ) -> Option<Arc<solana_client::rpc_client::RpcClient>> {
        self.solana_clients.get(network).cloned()
    }

    /// Create an RPC provider from configuration
    ///
    /// This method uses the Config system to get RPC URLs for various networks.
    pub fn from_config(config: &Config) -> Self {
        let mut provider = Self::new();

        // Add Solana networks from configuration
        // Main network from config
        provider.add_solana_network("mainnet".to_string(), config.network.solana_rpc_url.clone());

        // If the config URL contains "devnet", also add as devnet
        if config.network.solana_rpc_url.contains("devnet") {
            provider
                .add_solana_network("devnet".to_string(), config.network.solana_rpc_url.clone());
        } else {
            // Use default devnet URL
            provider.add_solana_network(
                "devnet".to_string(),
                "https://api.devnet.solana.com".to_string(),
            );
        }

        // Add testnet with default
        provider.add_solana_network(
            "testnet".to_string(),
            "https://api.testnet.solana.com".to_string(),
        );

        provider
    }

    /// Create an RPC provider from environment (backward compatibility)
    ///
    /// This method first tries to load configuration from Config::from_env(),
    /// falling back to direct environment variable reading if that fails.
    pub fn from_env() -> Self {
        // Try to use global config if available
        if let Some(config) = Config::try_global() {
            return Self::from_config(&config);
        }

        // Otherwise fall back to reading environment variables directly
        // This maintains backward compatibility
        let mut provider = Self::new();

        // Add Solana networks with fallback to defaults
        provider.add_solana_network(
            "mainnet".to_string(),
            std::env::var(SOLANA_RPC_URL)
                .unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string()),
        );

        provider.add_solana_network(
            "devnet".to_string(),
            std::env::var(SOLANA_DEVNET_RPC_URL)
                .unwrap_or_else(|_| "https://api.devnet.solana.com".to_string()),
        );

        provider.add_solana_network(
            "testnet".to_string(),
            std::env::var(SOLANA_TESTNET_RPC_URL)
                .unwrap_or_else(|_| "https://api.testnet.solana.com".to_string()),
        );

        provider
    }

    /// Get the default Solana client (mainnet)
    pub fn default_solana_client(&self) -> Option<Arc<solana_client::rpc_client::RpcClient>> {
        self.get_solana_client("mainnet")
    }

    /// List all available Solana networks
    pub fn list_solana_networks(&self) -> Vec<String> {
        self.solana_clients.keys().cloned().collect()
    }
}

impl std::fmt::Debug for RpcProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RpcProvider")
            .field("solana_networks", &self.list_solana_networks())
            .finish()
    }
}

/// Application context that includes RPC providers and other shared resources
///
/// This context can be passed to tools and workers to provide access to
/// blockchain RPC clients and other shared application resources.
#[derive(Clone)]
pub struct ApplicationContext {
    /// RPC provider for read-only blockchain operations
    pub rpc_provider: Arc<RpcProvider>,
    /// Configuration for application settings
    pub config: Arc<Config>,
    /// Extensions for crate-specific context (e.g., API clients)
    extensions: DashMap<TypeId, Arc<dyn Any + Send + Sync>>,
}

impl ApplicationContext {
    /// Create a new application context
    pub fn new(rpc_provider: Arc<RpcProvider>, config: Arc<Config>) -> Self {
        Self {
            rpc_provider,
            config,
            extensions: DashMap::new(),
        }
    }

    /// Create an application context from configuration
    pub fn from_config(config: &Config) -> Self {
        let rpc_provider = Arc::new(RpcProvider::from_config(config));
        Self::new(rpc_provider, Arc::new(config.clone()))
    }

    /// Create an application context from environment
    pub fn from_env() -> Self {
        let config = Config::from_env();
        let rpc_provider = Arc::new(RpcProvider::from_config(&config));
        Self::new(rpc_provider, config)
    }

    /// Get the providers configuration for creating API clients
    pub fn providers_config(&self) -> &riglr_config::ProvidersConfig {
        &self.config.providers
    }

    /// Add an extension to the context
    ///
    /// This allows higher-level crates to store their specific context
    /// (like API clients) without creating circular dependencies.
    pub fn set_extension<T: Send + Sync + 'static>(&self, data: T) {
        self.extensions.insert(TypeId::of::<T>(), Arc::new(data));
    }

    /// Get an extension from the context
    ///
    /// Returns None if the extension wasn't registered or if the type doesn't match.
    pub fn get_extension<T: Send + Sync + 'static>(&self) -> Option<Arc<T>> {
        self.extensions
            .get(&TypeId::of::<T>())
            .and_then(|entry| entry.value().clone().downcast::<T>().ok())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rpc_provider_new() {
        let provider = RpcProvider::default();
        assert!(provider.solana_clients.is_empty());
    }

    #[test]
    fn test_add_solana_network() {
        let mut provider = RpcProvider::default();
        provider.add_solana_network("testnet".to_string(), "https://test.com".to_string());

        assert!(provider.get_solana_client("testnet").is_some());
        assert!(provider.get_solana_client("mainnet").is_none());
    }

    #[test]
    fn test_list_solana_networks() {
        let mut provider = RpcProvider::default();
        provider.add_solana_network("mainnet".to_string(), "https://main.com".to_string());
        provider.add_solana_network("devnet".to_string(), "https://dev.com".to_string());

        let solana_networks = provider.list_solana_networks();
        assert_eq!(solana_networks.len(), 2);
        assert!(solana_networks.contains(&"mainnet".to_string()));
        assert!(solana_networks.contains(&"devnet".to_string()));
    }

    #[test]
    fn test_from_env_defaults() {
        // Clear relevant env vars to test defaults
        std::env::remove_var(SOLANA_RPC_URL);
        std::env::remove_var(SOLANA_DEVNET_RPC_URL);
        std::env::remove_var(SOLANA_TESTNET_RPC_URL);

        let provider = RpcProvider::from_env();

        // Should have default Solana networks
        assert!(provider.get_solana_client("mainnet").is_some());
        assert!(provider.get_solana_client("devnet").is_some());
        assert!(provider.get_solana_client("testnet").is_some());
    }

    #[test]
    fn test_from_env_with_custom_urls() {
        std::env::set_var(SOLANA_RPC_URL, "https://custom.solana.com");

        let provider = RpcProvider::from_env();

        assert!(provider.get_solana_client("mainnet").is_some());

        // Cleanup
        std::env::remove_var(SOLANA_RPC_URL);
    }

    #[test]
    fn test_application_context() {
        let provider = Arc::new(RpcProvider::default());
        let config = Config::from_env();
        let context = ApplicationContext::new(provider.clone(), config.clone());

        assert!(Arc::ptr_eq(&context.rpc_provider, &provider));
        assert!(Arc::ptr_eq(&context.config, &config));
    }

    #[test]
    fn test_debug_implementation() {
        let mut provider = RpcProvider::default();
        provider.add_solana_network("mainnet".to_string(), "https://main.com".to_string());

        let debug_str = format!("{:?}", provider);
        assert!(debug_str.contains("RpcProvider"));
        assert!(debug_str.contains("mainnet"));
    }
}
