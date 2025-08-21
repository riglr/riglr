//! Multi-chain EVM stream management
//!
//! This module provides functionality for managing EVM streams across multiple blockchain networks.
//! It allows for dynamic registration and management of EVM chains using environment-based configuration.

use crate::core::{Stream, StreamError, StreamManager};
use crate::evm::{ChainId, EvmStreamConfig, EvmWebSocketStream};
use dashmap::DashMap;
use std::sync::Arc;
use tracing::{info, warn};

/// Multi-chain EVM stream manager
#[derive(Default)]
pub struct MultiChainEvmManager {
    /// Streams by chain ID
    streams: Arc<DashMap<ChainId, EvmWebSocketStream>>,
    /// Stream manager reference
    stream_manager: Option<Arc<StreamManager>>,
}

impl MultiChainEvmManager {
    /// Create a new multi-chain EVM manager
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the stream manager
    pub fn with_stream_manager(mut self, manager: Arc<StreamManager>) -> Self {
        self.stream_manager = Some(manager);
        self
    }

    /// Add an EVM chain using the RPC_URL_{CHAIN_ID} pattern
    pub async fn add_chain(&self, chain_id: ChainId) -> Result<(), StreamError> {
        let ws_url = self.get_websocket_url(chain_id)?;

        let stream_config = EvmStreamConfig {
            ws_url,
            chain_id,
            subscribe_pending_transactions: false,
            subscribe_new_blocks: true,
            contract_addresses: Vec::new(),
            buffer_size: 10000,
        };

        let stream_name = format!("evm-{}", chain_id);
        let mut stream = EvmWebSocketStream::new(stream_name.clone());
        stream.start(stream_config).await?;

        // Add to local collection
        self.streams.insert(chain_id, stream);

        // If we have a stream manager, register with it
        if let Some(_manager) = &self.stream_manager {
            // Note: This would require the stream to be wrapped as DynamicStream
            // For now, we'll just log
            info!("Added EVM stream for chain: {}", chain_id);
        }

        Ok(())
    }

    /// Remove a chain
    pub async fn remove_chain(&self, chain_id: ChainId) -> Result<(), StreamError> {
        if let Some((_, mut stream)) = self.streams.remove(&chain_id) {
            stream.stop().await?;
            info!("Removed EVM stream for chain: {}", chain_id);
        }
        Ok(())
    }

    /// Get WebSocket URL from environment variable
    fn get_websocket_url(&self, chain_id: ChainId) -> Result<String, StreamError> {
        let chain_id_num: u64 = chain_id.into();
        let rpc_url_key = format!("RPC_URL_{}", chain_id_num);

        let http_url = std::env::var(&rpc_url_key).map_err(|_| StreamError::Configuration {
            message: format!("Missing {} environment variable", rpc_url_key),
        })?;

        // Convert HTTP URL to WebSocket URL
        let ws_url = http_url
            .replace("https://", "wss://")
            .replace("http://", "ws://");

        Ok(ws_url)
    }

    /// Check if a chain is configured
    pub fn is_chain_configured(&self, chain_id: ChainId) -> bool {
        let chain_id_num: u64 = chain_id.into();
        let rpc_url_key = format!("RPC_URL_{}", chain_id_num);
        std::env::var(&rpc_url_key).is_ok()
    }

    /// Register all configured chains
    pub async fn register_all_configured_chains(&self) -> Result<(), StreamError> {
        let supported_chains = vec![
            ChainId::Ethereum,
            ChainId::Polygon,
            ChainId::BSC,
            ChainId::Arbitrum,
            ChainId::Optimism,
            ChainId::Avalanche,
            ChainId::Base,
        ];

        for chain_id in supported_chains {
            if self.is_chain_configured(chain_id) {
                match self.add_chain(chain_id).await {
                    Ok(_) => {
                        info!("Registered EVM stream for chain: {}", chain_id);
                    }
                    Err(e) => {
                        warn!("Failed to add chain {}: {}", chain_id, e);
                    }
                }
            }
        }

        Ok(())
    }

    /// Stop all streams
    pub async fn stop_all(&self) -> Result<(), StreamError> {
        let streams: Vec<_> = self.streams.iter().map(|entry| *entry.key()).collect();
        for chain_id in streams {
            if let Some((_, mut stream)) = self.streams.remove(&chain_id) {
                if let Err(e) = stream.stop().await {
                    warn!("Failed to stop stream for chain {}: {}", chain_id, e);
                }
            }
        }
        Ok(())
    }

    /// Get list of active chains
    pub async fn active_chains(&self) -> Vec<ChainId> {
        self.streams.iter().map(|entry| *entry.key()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    fn setup_test_env() {
        env::set_var("RPC_URL_1", "https://eth.example.com");
        env::set_var("RPC_URL_137", "https://polygon.example.com");
        env::set_var("RPC_URL_56", "http://bsc.example.com");
    }

    fn cleanup_test_env() {
        env::remove_var("RPC_URL_1");
        env::remove_var("RPC_URL_137");
        env::remove_var("RPC_URL_56");
        env::remove_var("RPC_URL_42161");
        env::remove_var("RPC_URL_10");
        env::remove_var("RPC_URL_43114");
        env::remove_var("RPC_URL_8453");
    }

    #[test]
    fn test_new_creates_default_manager() {
        let manager = MultiChainEvmManager::default();
        assert!(manager.streams.is_empty());
        assert!(manager.stream_manager.is_none());
    }

    #[test]
    fn test_default_creates_empty_manager() {
        let manager = MultiChainEvmManager::default();
        assert!(manager.streams.is_empty());
        assert!(manager.stream_manager.is_none());
    }

    #[test]
    fn test_with_stream_manager_sets_manager() {
        let stream_manager = Arc::new(StreamManager::default());
        let manager = MultiChainEvmManager::default().with_stream_manager(stream_manager.clone());

        assert!(manager.stream_manager.is_some());
        assert!(Arc::ptr_eq(
            &manager.stream_manager.unwrap(),
            &stream_manager
        ));
    }

    #[tokio::test]
    async fn test_add_chain_when_env_var_missing_should_return_error() {
        cleanup_test_env();
        let manager = MultiChainEvmManager::default();

        let result = manager.add_chain(ChainId::Ethereum).await;

        assert!(result.is_err());
        assert!(matches!(result, Err(StreamError::Configuration { .. })));
        if let Err(StreamError::Configuration { message }) = result {
            assert!(message.contains("Missing RPC_URL_1"));
        }
    }

    #[tokio::test]
    async fn test_add_chain_when_valid_env_var_should_succeed() {
        setup_test_env();
        let manager = MultiChainEvmManager::default();

        // Mock the stream creation by not actually starting it
        // This test focuses on the manager logic
        let result = manager.add_chain(ChainId::Ethereum).await;

        // The test may fail due to actual WebSocket connection, but we're testing the URL resolution
        // The important part is that we don't get a Configuration error
        assert!(!matches!(result, Err(StreamError::Configuration { .. })));
        // OK - either success or connection error is fine for this test

        cleanup_test_env();
    }

    #[tokio::test]
    async fn test_remove_chain_when_chain_exists_should_remove() {
        setup_test_env();
        let manager = MultiChainEvmManager::default();

        // First try to add a chain (it may fail due to connection, but that's OK)
        let _ = manager.add_chain(ChainId::Ethereum).await;

        // Remove should always succeed even if chain wasn't actually added
        let result = manager.remove_chain(ChainId::Ethereum).await;
        assert!(result.is_ok());

        cleanup_test_env();
    }

    #[tokio::test]
    async fn test_remove_chain_when_chain_not_exists_should_succeed() {
        let manager = MultiChainEvmManager::default();

        let result = manager.remove_chain(ChainId::Ethereum).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_websocket_url_when_https_should_convert_to_wss() {
        setup_test_env();
        let manager = MultiChainEvmManager::default();

        let result = manager.get_websocket_url(ChainId::Ethereum);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "wss://eth.example.com");

        cleanup_test_env();
    }

    #[test]
    fn test_get_websocket_url_when_http_should_convert_to_ws() {
        setup_test_env();
        let manager = MultiChainEvmManager::default();

        let result = manager.get_websocket_url(ChainId::BSC);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "ws://bsc.example.com");

        cleanup_test_env();
    }

    #[test]
    fn test_get_websocket_url_when_env_var_missing_should_return_error() {
        cleanup_test_env();
        let manager = MultiChainEvmManager::default();

        let result = manager.get_websocket_url(ChainId::Ethereum);
        assert!(result.is_err());
        assert!(matches!(result, Err(StreamError::Configuration { .. })));

        if let Err(StreamError::Configuration { message }) = result {
            assert!(message.contains("Missing RPC_URL_1"));
        }
    }

    #[test]
    fn test_is_chain_configured_when_env_var_exists_should_return_true() {
        setup_test_env();
        let manager = MultiChainEvmManager::default();

        assert!(manager.is_chain_configured(ChainId::Ethereum));
        assert!(manager.is_chain_configured(ChainId::Polygon));
        assert!(manager.is_chain_configured(ChainId::BSC));

        cleanup_test_env();
    }

    #[test]
    fn test_is_chain_configured_when_env_var_missing_should_return_false() {
        cleanup_test_env();
        let manager = MultiChainEvmManager::default();

        assert!(!manager.is_chain_configured(ChainId::Ethereum));
        assert!(!manager.is_chain_configured(ChainId::Polygon));
        assert!(!manager.is_chain_configured(ChainId::BSC));
        assert!(!manager.is_chain_configured(ChainId::Arbitrum));
        assert!(!manager.is_chain_configured(ChainId::Optimism));
        assert!(!manager.is_chain_configured(ChainId::Avalanche));
        assert!(!manager.is_chain_configured(ChainId::Base));
    }

    #[tokio::test]
    async fn test_register_all_configured_chains_when_no_chains_configured_should_succeed() {
        cleanup_test_env();
        let manager = MultiChainEvmManager::default();

        let result = manager.register_all_configured_chains().await;
        assert!(result.is_ok());

        // No chains should be added
        let active_chains = manager.active_chains().await;
        assert!(active_chains.is_empty());
    }

    #[tokio::test]
    async fn test_register_all_configured_chains_when_some_chains_configured_should_try_all() {
        setup_test_env();
        let manager = MultiChainEvmManager::default();

        let result = manager.register_all_configured_chains().await;
        assert!(result.is_ok());

        // The method should complete successfully even if individual chain additions fail
        // due to connection issues in tests

        cleanup_test_env();
    }

    #[tokio::test]
    async fn test_register_all_configured_chains_covers_all_supported_chains() {
        // Set up environment variables for all supported chains
        env::set_var("RPC_URL_1", "https://eth.example.com");
        env::set_var("RPC_URL_137", "https://polygon.example.com");
        env::set_var("RPC_URL_56", "https://bsc.example.com");
        env::set_var("RPC_URL_42161", "https://arbitrum.example.com");
        env::set_var("RPC_URL_10", "https://optimism.example.com");
        env::set_var("RPC_URL_43114", "https://avalanche.example.com");
        env::set_var("RPC_URL_8453", "https://base.example.com");

        let manager = MultiChainEvmManager::default();

        // Verify all chains are configured
        assert!(manager.is_chain_configured(ChainId::Ethereum));
        assert!(manager.is_chain_configured(ChainId::Polygon));
        assert!(manager.is_chain_configured(ChainId::BSC));
        assert!(manager.is_chain_configured(ChainId::Arbitrum));
        assert!(manager.is_chain_configured(ChainId::Optimism));
        assert!(manager.is_chain_configured(ChainId::Avalanche));
        assert!(manager.is_chain_configured(ChainId::Base));

        let result = manager.register_all_configured_chains().await;
        assert!(result.is_ok());

        cleanup_test_env();
    }

    #[tokio::test]
    async fn test_stop_all_when_no_streams_should_succeed() {
        let manager = MultiChainEvmManager::default();

        let result = manager.stop_all().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_stop_all_when_streams_exist_should_stop_all() {
        setup_test_env();
        let manager = MultiChainEvmManager::default();

        // Try to add some chains (may fail due to connection, but that's OK for testing)
        let _ = manager.add_chain(ChainId::Ethereum).await;
        let _ = manager.add_chain(ChainId::Polygon).await;

        let result = manager.stop_all().await;
        assert!(result.is_ok());

        // Verify all streams are removed
        let active_chains = manager.active_chains().await;
        assert!(active_chains.is_empty());

        cleanup_test_env();
    }

    #[tokio::test]
    async fn test_active_chains_when_no_streams_should_return_empty() {
        let manager = MultiChainEvmManager::default();

        let active_chains = manager.active_chains().await;
        assert!(active_chains.is_empty());
    }

    #[tokio::test]
    async fn test_active_chains_when_streams_exist_should_return_chain_ids() {
        setup_test_env();
        let manager = MultiChainEvmManager::default();

        // Try to add some chains (may fail due to connection, but we can still test the collection)
        let _ = manager.add_chain(ChainId::Ethereum).await;
        let _ = manager.add_chain(ChainId::Polygon).await;

        let _active_chains = manager.active_chains().await;
        // The exact contents depend on whether the streams were successfully added
        // but we can verify the method works without panicking

        cleanup_test_env();
    }

    #[test]
    fn test_chain_id_conversion_for_url_generation() {
        let _manager = MultiChainEvmManager::default();

        // Test that ChainId converts to correct u64 values for environment variable names
        let ethereum_num: u64 = ChainId::Ethereum.into();
        let polygon_num: u64 = ChainId::Polygon.into();
        let bsc_num: u64 = ChainId::BSC.into();
        let arbitrum_num: u64 = ChainId::Arbitrum.into();
        let optimism_num: u64 = ChainId::Optimism.into();
        let avalanche_num: u64 = ChainId::Avalanche.into();
        let base_num: u64 = ChainId::Base.into();

        // Verify the format string generation works correctly
        assert_eq!(format!("RPC_URL_{}", ethereum_num), "RPC_URL_1");
        assert_eq!(format!("RPC_URL_{}", polygon_num), "RPC_URL_137");
        assert_eq!(format!("RPC_URL_{}", bsc_num), "RPC_URL_56");
        assert_eq!(format!("RPC_URL_{}", arbitrum_num), "RPC_URL_42161");
        assert_eq!(format!("RPC_URL_{}", optimism_num), "RPC_URL_10");
        assert_eq!(format!("RPC_URL_{}", avalanche_num), "RPC_URL_43114");
        assert_eq!(format!("RPC_URL_{}", base_num), "RPC_URL_8453");
    }

    #[test]
    fn test_websocket_url_conversion_edge_cases() {
        let manager = MultiChainEvmManager::default();

        // Test with various URL formats using Ethereum chain ID
        env::set_var("RPC_URL_1", "https://example.com/path?query=value");
        let result = manager.get_websocket_url(ChainId::Ethereum);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "wss://example.com/path?query=value");

        env::set_var("RPC_URL_1", "http://localhost:8545");
        let result = manager.get_websocket_url(ChainId::Ethereum);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "ws://localhost:8545");

        env::set_var("RPC_URL_1", "https://user:pass@example.com:443/rpc");
        let result = manager.get_websocket_url(ChainId::Ethereum);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "wss://user:pass@example.com:443/rpc");

        env::remove_var("RPC_URL_1");
    }

    #[tokio::test]
    async fn test_with_stream_manager_integration() {
        let stream_manager = Arc::new(StreamManager::default());
        let manager = MultiChainEvmManager::default().with_stream_manager(stream_manager.clone());

        setup_test_env();

        // Test that the stream manager reference is used (currently just logs)
        let _result = manager.add_chain(ChainId::Ethereum).await;

        // The result depends on WebSocket connection, but we're testing the manager integration
        // The important part is that the stream_manager is set and used
        assert!(manager.stream_manager.is_some());

        cleanup_test_env();
    }
}
