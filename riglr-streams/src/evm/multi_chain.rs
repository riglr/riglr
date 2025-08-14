use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};
use crate::evm::{EvmWebSocketStream, EvmStreamConfig, ChainId};
use crate::core::{StreamManager, StreamError, Stream};

/// Multi-chain EVM stream manager
pub struct MultiChainEvmManager {
    /// Streams by chain ID
    streams: Arc<RwLock<HashMap<ChainId, EvmWebSocketStream>>>,
    /// Stream manager reference
    stream_manager: Option<Arc<StreamManager>>,
}

impl MultiChainEvmManager {
    /// Create a new multi-chain EVM manager
    pub fn new() -> Self {
        Self {
            streams: Arc::new(RwLock::new(HashMap::new())),
            stream_manager: None,
        }
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
        self.streams.write().await.insert(chain_id, stream);
        
        // If we have a stream manager, register with it
        if let Some(manager) = &self.stream_manager {
            // Note: This would require the stream to be wrapped as DynamicStream
            // For now, we'll just log
            info!("Added EVM stream for chain: {}", chain_id);
        }
        
        Ok(())
    }
    
    /// Remove a chain
    pub async fn remove_chain(&self, chain_id: ChainId) -> Result<(), StreamError> {
        if let Some(mut stream) = self.streams.write().await.remove(&chain_id) {
            stream.stop().await?;
            info!("Removed EVM stream for chain: {}", chain_id);
        }
        Ok(())
    }
    
    /// Get WebSocket URL from environment variable
    fn get_websocket_url(&self, chain_id: ChainId) -> Result<String, StreamError> {
        let chain_id_num: u64 = chain_id.into();
        let rpc_url_key = format!("RPC_URL_{}", chain_id_num);
        
        let http_url = std::env::var(&rpc_url_key)
            .map_err(|_| StreamError::Configuration {
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
        let mut streams = self.streams.write().await;
        for (chain_id, mut stream) in streams.drain() {
            if let Err(e) = stream.stop().await {
                warn!("Failed to stop stream for chain {}: {}", chain_id, e);
            }
        }
        Ok(())
    }
    
    /// Get list of active chains
    pub async fn active_chains(&self) -> Vec<ChainId> {
        self.streams.read().await.keys().cloned().collect()
    }
}

impl Default for MultiChainEvmManager {
    fn default() -> Self {
        Self::new()
    }
}