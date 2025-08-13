//! Network configuration types for blockchain integrations
//!
//! This module contains only the minimal configuration types needed
//! by other crates in the riglr ecosystem. Application-specific configuration
//! should be defined in the application crate, not in the core library.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Type-safe RPC configuration for blockchain networks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcConfig {
    pub evm_networks: HashMap<String, EvmNetworkConfig>,
    pub solana_networks: HashMap<String, SolanaNetworkConfig>,
}

/// EVM network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvmNetworkConfig {
    pub name: String,
    pub chain_id: u64,
    pub rpc_url: String,
    pub explorer_url: Option<String>,
}

/// Solana network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolanaNetworkConfig {
    pub name: String,
    pub rpc_url: String,
    pub explorer_url: Option<String>,
}

impl Default for RpcConfig {
    fn default() -> Self {
        let mut evm_networks = HashMap::new();
        evm_networks.insert("ethereum".to_string(), EvmNetworkConfig {
            name: "Ethereum Mainnet".to_string(),
            chain_id: 1,
            rpc_url: "https://eth.llamarpc.com".to_string(),
            explorer_url: Some("https://etherscan.io".to_string()),
        });
        evm_networks.insert("polygon".to_string(), EvmNetworkConfig {
            name: "Polygon".to_string(),
            chain_id: 137,
            rpc_url: "https://polygon.llamarpc.com".to_string(),
            explorer_url: Some("https://polygonscan.com".to_string()),
        });
        evm_networks.insert("arbitrum".to_string(), EvmNetworkConfig {
            name: "Arbitrum One".to_string(),
            chain_id: 42161,
            rpc_url: "https://arbitrum.llamarpc.com".to_string(),
            explorer_url: Some("https://arbiscan.io".to_string()),
        });
        evm_networks.insert("optimism".to_string(), EvmNetworkConfig {
            name: "Optimism".to_string(),
            chain_id: 10,
            rpc_url: "https://optimism.llamarpc.com".to_string(),
            explorer_url: Some("https://optimistic.etherscan.io".to_string()),
        });
        evm_networks.insert("base".to_string(), EvmNetworkConfig {
            name: "Base".to_string(),
            chain_id: 8453,
            rpc_url: "https://base.llamarpc.com".to_string(),
            explorer_url: Some("https://basescan.org".to_string()),
        });
        evm_networks.insert("bsc".to_string(), EvmNetworkConfig {
            name: "BNB Smart Chain".to_string(),
            chain_id: 56,
            rpc_url: "https://bsc.llamarpc.com".to_string(),
            explorer_url: Some("https://bscscan.com".to_string()),
        });
        evm_networks.insert("avalanche".to_string(), EvmNetworkConfig {
            name: "Avalanche C-Chain".to_string(),
            chain_id: 43114,
            rpc_url: "https://avalanche.llamarpc.com".to_string(),
            explorer_url: Some("https://snowtrace.io".to_string()),
        });
        evm_networks.insert("gnosis".to_string(), EvmNetworkConfig {
            name: "Gnosis".to_string(),
            chain_id: 100,
            rpc_url: "https://gnosis.llamarpc.com".to_string(),
            explorer_url: Some("https://gnosisscan.io".to_string()),
        });
        evm_networks.insert("fantom".to_string(), EvmNetworkConfig {
            name: "Fantom".to_string(),
            chain_id: 250,
            rpc_url: "https://fantom.llamarpc.com".to_string(),
            explorer_url: Some("https://ftmscan.com".to_string()),
        });
        evm_networks.insert("linea".to_string(), EvmNetworkConfig {
            name: "Linea".to_string(),
            chain_id: 59144,
            rpc_url: "https://linea.blockpi.network/v1/rpc/public".to_string(),
            explorer_url: Some("https://lineascan.build".to_string()),
        });
        evm_networks.insert("scroll".to_string(), EvmNetworkConfig {
            name: "Scroll".to_string(),
            chain_id: 534352,
            rpc_url: "https://rpc.scroll.io".to_string(),
            explorer_url: Some("https://scrollscan.com".to_string()),
        });
        evm_networks.insert("blast".to_string(), EvmNetworkConfig {
            name: "Blast".to_string(),
            chain_id: 81457,
            rpc_url: "https://blast.blockpi.network/v1/rpc/public".to_string(),
            explorer_url: Some("https://blastscan.io".to_string()),
        });
        evm_networks.insert("mode".to_string(), EvmNetworkConfig {
            name: "Mode".to_string(),
            chain_id: 34443,
            rpc_url: "https://mainnet.mode.network".to_string(),
            explorer_url: Some("https://modescan.io".to_string()),
        });
        evm_networks.insert("mantle".to_string(), EvmNetworkConfig {
            name: "Mantle".to_string(),
            chain_id: 5000,
            rpc_url: "https://mantle.publicnode.com".to_string(),
            explorer_url: Some("https://explorer.mantle.xyz".to_string()),
        });
        evm_networks.insert("celo".to_string(), EvmNetworkConfig {
            name: "Celo".to_string(),
            chain_id: 42220,
            rpc_url: "https://forno.celo.org".to_string(),
            explorer_url: Some("https://celoscan.io".to_string()),
        });
        evm_networks.insert("cronos".to_string(), EvmNetworkConfig {
            name: "Cronos".to_string(),
            chain_id: 25,
            rpc_url: "https://node.cronos.org/rpc".to_string(),
            explorer_url: Some("https://cronoscan.com".to_string()),
        });

        let mut solana_networks = HashMap::new();
        solana_networks.insert("mainnet".to_string(), SolanaNetworkConfig {
            name: "Solana Mainnet".to_string(),
            rpc_url: "https://api.mainnet-beta.solana.com".to_string(),
            explorer_url: Some("https://explorer.solana.com".to_string()),
        });
        solana_networks.insert("devnet".to_string(), SolanaNetworkConfig {
            name: "Solana Devnet".to_string(),
            rpc_url: "https://api.devnet.solana.com".to_string(),
            explorer_url: Some("https://explorer.solana.com".to_string()),
        });

        RpcConfig {
            evm_networks,
            solana_networks,
        }
    }
}

impl EvmNetworkConfig {
    /// Return the CAIP-2 identifier for this EVM network, e.g. "eip155:1".
    pub fn caip2(&self) -> String {
        format!("eip155:{}", self.chain_id)
    }
}

impl RpcConfig {
    /// Get CAIP-2 for an EVM network by name (case-insensitive key lookup).
    pub fn evm_caip2_for(&self, name: &str) -> Option<String> {
        let key = name.to_lowercase();
        self.evm_networks.get(&key).map(|n| n.caip2())
    }

    /// Add or update an EVM network configuration dynamically.
    pub fn add_evm_network(
        &mut self,
        name: impl Into<String>,
        chain_id: u64,
        rpc_url: impl Into<String>,
        explorer_url: Option<String>,
    ) -> &mut Self {
        let key = name.into().to_lowercase();
        let display_name = key.clone();
        self.evm_networks.insert(key, EvmNetworkConfig { 
            name: display_name, 
            chain_id, 
            rpc_url: rpc_url.into(), 
            explorer_url 
        });
        self
    }

    /// Override or extend EVM networks from env like RPC_URL_{CHAIN_ID}.
    /// If chain_id exists, updates rpc_url; otherwise adds as "chain_{id}".
    pub fn with_env_overrides(mut self) -> Self {
        for (k, v) in std::env::vars() {
            if let Some(cid_str) = k.strip_prefix("RPC_URL_") {
                if let Ok(cid) = cid_str.parse::<u64>() {
                    // Try to find an existing network with same chain_id
                    if let Some((existing_key, _)) = self
                        .evm_networks
                        .iter()
                        .find(|(_, cfg)| cfg.chain_id == cid)
                        .map(|(k, v)| (k.clone(), v.clone()))
                    {
                        if let Some(cfg) = self.evm_networks.get_mut(&existing_key) {
                            cfg.rpc_url = v.clone();
                        }
                    } else {
                        let name = format!("chain_{}", cid);
                        self.add_evm_network(name, cid, v.clone(), None);
                    }
                }
            }
        }
        self
    }
}