//! Chain management utilities shared across riglr crates
//!
//! This module provides unified chain ID mapping, RPC URL management,
//! and chain validation functions that eliminate duplication between
//! riglr-evm-tools and riglr-cross-chain-tools.

use crate::error::{EvmCommonError, EvmResult};

/// Chain information structure
#[derive(Debug, Clone)]
pub struct ChainInfo {
    pub chain_id: u64,
    pub name: String,
    pub symbol: String,
    pub block_explorer: Option<String>,
    pub default_rpc: Option<String>,
}

/// Get chain information by chain ID
pub fn get_chain_info(chain_id: u64) -> Option<ChainInfo> {
    match chain_id {
        1 => Some(ChainInfo {
            chain_id: 1,
            name: "Ethereum".to_string(),
            symbol: "ETH".to_string(),
            block_explorer: Some("https://etherscan.io".to_string()),
            default_rpc: Some("https://eth.llamarpc.com".to_string()),
        }),
        137 => Some(ChainInfo {
            chain_id: 137,
            name: "Polygon".to_string(),
            symbol: "MATIC".to_string(),
            block_explorer: Some("https://polygonscan.com".to_string()),
            default_rpc: Some("https://polygon-rpc.com".to_string()),
        }),
        42161 => Some(ChainInfo {
            chain_id: 42161,
            name: "Arbitrum".to_string(),
            symbol: "ETH".to_string(),
            block_explorer: Some("https://arbiscan.io".to_string()),
            default_rpc: Some("https://arb1.arbitrum.io/rpc".to_string()),
        }),
        10 => Some(ChainInfo {
            chain_id: 10,
            name: "Optimism".to_string(),
            symbol: "ETH".to_string(),
            block_explorer: Some("https://optimistic.etherscan.io".to_string()),
            default_rpc: Some("https://mainnet.optimism.io".to_string()),
        }),
        8453 => Some(ChainInfo {
            chain_id: 8453,
            name: "Base".to_string(),
            symbol: "ETH".to_string(),
            block_explorer: Some("https://basescan.org".to_string()),
            default_rpc: Some("https://mainnet.base.org".to_string()),
        }),
        56 => Some(ChainInfo {
            chain_id: 56,
            name: "BNB Smart Chain".to_string(),
            symbol: "BNB".to_string(),
            block_explorer: Some("https://bscscan.com".to_string()),
            default_rpc: Some("https://bsc-dataseed.binance.org".to_string()),
        }),
        43114 => Some(ChainInfo {
            chain_id: 43114,
            name: "Avalanche".to_string(),
            symbol: "AVAX".to_string(),
            block_explorer: Some("https://snowtrace.io".to_string()),
            default_rpc: Some("https://api.avax.network/ext/bc/C/rpc".to_string()),
        }),
        250 => Some(ChainInfo {
            chain_id: 250,
            name: "Fantom".to_string(),
            symbol: "FTM".to_string(),
            block_explorer: Some("https://ftmscan.com".to_string()),
            default_rpc: Some("https://rpc.ftm.tools".to_string()),
        }),
        _ => None,
    }
}

/// Maps chain IDs to RPC URLs using convention-based environment variable lookup.
/// Uses format: RPC_URL_{CHAIN_ID}
///
/// This is the UNIFIED approach that eliminates conflicts between different
/// chain management systems across riglr crates.
///
/// # Arguments
/// * `chain_id` - Numeric chain ID (e.g., 1 for Ethereum, 137 for Polygon)
///
/// # Returns
/// * RPC URL string from environment or error if not configured
///
/// # Environment Variables
/// * `RPC_URL_1` - Ethereum mainnet
/// * `RPC_URL_137` - Polygon
/// * `RPC_URL_42161` - Arbitrum
/// * `RPC_URL_10` - Optimism
/// * `RPC_URL_8453` - Base
/// * etc.
///
/// # Examples
/// ```rust,ignore
/// use riglr_evm_common::chain::chain_id_to_rpc_url;
///
/// // Configure environment
/// std::env::set_var("RPC_URL_1", "https://eth.llamarpc.com");
///
/// let url = chain_id_to_rpc_url(1)?;
/// assert_eq!(url, "https://eth.llamarpc.com");
/// ```
pub fn chain_id_to_rpc_url(chain_id: u64) -> EvmResult<String> {
    let env_var = format!("RPC_URL_{}", chain_id);

    match std::env::var(&env_var) {
        Ok(url) => {
            if url.trim().is_empty() {
                return Err(EvmCommonError::InvalidConfig(format!(
                    "RPC URL for chain {} is empty. Set {} environment variable.",
                    chain_id, env_var
                )));
            }

            // Validate URL format
            validate_rpc_url(&url, chain_id)?;

            tracing::debug!(
                "✅ Found RPC URL for chain {}: {}",
                chain_id,
                &url[..std::cmp::min(50, url.len())]
            );
            Ok(url)
        }
        Err(_) => {
            // Try to use default RPC if available
            if let Some(chain_info) = get_chain_info(chain_id) {
                if let Some(default_rpc) = chain_info.default_rpc {
                    tracing::warn!("⚠️  Using default RPC for chain {}: {}. Consider setting {} for production use.",
                                   chain_id, default_rpc, env_var);
                    return Ok(default_rpc);
                }
            }

            Err(EvmCommonError::UnsupportedChain(chain_id))
        }
    }
}

/// Validate RPC URL format
fn validate_rpc_url(url: &str, chain_id: u64) -> EvmResult<()> {
    if !url.starts_with("http://") && !url.starts_with("https://") && !url.starts_with("wss://") {
        return Err(EvmCommonError::InvalidConfig(format!(
            "Invalid RPC URL format for chain {}: {}. Must start with http://, https://, or wss://",
            chain_id, url
        )));
    }
    Ok(())
}

/// Convert chain name to chain ID
///
/// This provides a bridge between human-readable names and numeric IDs,
/// useful for cross-chain operations and user interfaces.
///
/// # Arguments
/// * `name` - Chain name (case-insensitive)
///
/// # Returns
/// * Numeric chain ID
///
/// # Supported Names
/// * "ethereum", "eth" → 1
/// * "polygon", "matic" → 137
/// * "arbitrum", "arb" → 42161
/// * "optimism", "op" → 10
/// * "base" → 8453
/// * "bsc", "binance" → 56
/// * "avalanche", "avax" → 43114
/// * "fantom", "ftm" → 250
///
/// # Examples
/// ```rust,ignore
/// use riglr_evm_common::chain::chain_name_to_id;
///
/// assert_eq!(chain_name_to_id("ethereum")?, 1);
/// assert_eq!(chain_name_to_id("ETH")?, 1);
/// assert_eq!(chain_name_to_id("polygon")?, 137);
/// ```
pub fn chain_name_to_id(name: &str) -> EvmResult<u64> {
    let normalized = name.to_lowercase();
    match normalized.as_str() {
        "ethereum" | "eth" => Ok(1),
        "polygon" | "matic" => Ok(137),
        "arbitrum" | "arb" => Ok(42161),
        "optimism" | "op" => Ok(10),
        "base" => Ok(8453),
        "bsc" | "binance" => Ok(56),
        "avalanche" | "avax" => Ok(43114),
        "fantom" | "ftm" => Ok(250),
        _ => Err(EvmCommonError::InvalidChainName(format!(
            "Unsupported chain name: {}. Supported: ethereum, polygon, arbitrum, optimism, base, bsc, avalanche, fantom",
            name
        ))),
    }
}

/// Convert chain ID to human-readable name
///
/// # Arguments
/// * `id` - Numeric chain ID
///
/// # Returns
/// * Human-readable chain name
///
/// # Examples
/// ```rust,ignore
/// use riglr_evm_common::chain::chain_id_to_name;
///
/// assert_eq!(chain_id_to_name(1)?, "ethereum");
/// assert_eq!(chain_id_to_name(137)?, "polygon");
/// ```
pub fn chain_id_to_name(id: u64) -> EvmResult<String> {
    match id {
        1 => Ok("ethereum".to_string()),
        137 => Ok("polygon".to_string()),
        42161 => Ok("arbitrum".to_string()),
        10 => Ok("optimism".to_string()),
        8453 => Ok("base".to_string()),
        56 => Ok("bsc".to_string()),
        43114 => Ok("avalanche".to_string()),
        250 => Ok("fantom".to_string()),
        _ => Err(EvmCommonError::UnsupportedChain(id)),
    }
}

/// Check if a chain is supported (has RPC URL configured or has default)
///
/// # Arguments
/// * `chain_id` - Numeric chain ID to check
///
/// # Returns
/// * `true` if chain is supported, `false` otherwise
///
/// # Examples
/// ```rust,ignore
/// use riglr_evm_common::chain::is_supported_chain;
///
/// // If RPC_URL_1 is configured or Ethereum has defaults
/// assert!(is_supported_chain(1));
///
/// // Unsupported chain
/// assert!(!is_supported_chain(999999));
/// ```
pub fn is_supported_chain(chain_id: u64) -> bool {
    chain_id_to_rpc_url(chain_id).is_ok()
}

/// Get list of all supported chain IDs
///
/// Scans environment variables for RPC_URL_* patterns and includes
/// chains with default RPC endpoints.
///
/// # Returns
/// * Vector of supported chain IDs
///
/// # Examples
/// ```rust,ignore
/// use riglr_evm_common::chain::get_supported_chains;
///
/// let chains = get_supported_chains();
/// if chains.contains(&1) {
///     println!("Ethereum is supported!");
/// }
/// ```
pub fn get_supported_chains() -> Vec<u64> {
    let mut chains: Vec<u64> = std::env::vars()
        .filter_map(|(key, _value)| {
            if key.starts_with("RPC_URL_") {
                key.strip_prefix("RPC_URL_")
                    .and_then(|chain_id_str| chain_id_str.parse::<u64>().ok())
            } else {
                None
            }
        })
        .collect();

    // Add chains with default RPCs that aren't already configured
    let default_chains = [1, 137, 42161, 10, 8453, 56, 43114, 250];
    for chain_id in default_chains {
        if !chains.contains(&chain_id) {
            // Check if this chain has default RPC
            if get_chain_info(chain_id)
                .and_then(|info| info.default_rpc)
                .is_some()
            {
                chains.push(chain_id);
            }
        }
    }

    chains.sort();
    chains
}

/// Get block explorer URL for a chain
///
/// # Arguments
/// * `chain_id` - Numeric chain ID
///
/// # Returns
/// * Block explorer base URL if known
///
/// # Examples
/// ```rust,ignore
/// use riglr_evm_common::chain::get_block_explorer_url;
///
/// let url = get_block_explorer_url(1)?;
/// assert_eq!(url, "https://etherscan.io");
/// ```
pub fn get_block_explorer_url(chain_id: u64) -> EvmResult<String> {
    get_chain_info(chain_id)
        .and_then(|info| info.block_explorer)
        .ok_or(EvmCommonError::UnsupportedChain(chain_id))
}

/// Get transaction URL for a specific transaction
///
/// # Arguments
/// * `chain_id` - Numeric chain ID
/// * `tx_hash` - Transaction hash (with or without 0x prefix)
///
/// # Returns
/// * Full URL to view transaction in block explorer
///
/// # Examples
/// ```rust,ignore
/// use riglr_evm_common::chain::get_transaction_url;
///
/// let url = get_transaction_url(1, "0x123abc...")?;
/// // Returns: https://etherscan.io/tx/0x123abc...
/// ```
pub fn get_transaction_url(chain_id: u64, tx_hash: &str) -> EvmResult<String> {
    let base_url = get_block_explorer_url(chain_id)?;
    let hash = if tx_hash.starts_with("0x") {
        tx_hash
    } else {
        &format!("0x{}", tx_hash)
    };
    Ok(format!("{}/tx/{}", base_url, hash))
}

/// Get address URL for viewing an address in block explorer
///
/// # Arguments
/// * `chain_id` - Numeric chain ID
/// * `address` - Address (with or without 0x prefix)
///
/// # Returns
/// * Full URL to view address in block explorer
///
/// # Examples
/// ```rust,ignore
/// use riglr_evm_common::chain::get_address_url;
///
/// let url = get_address_url(1, "0x742d35Cc...")?;
/// // Returns: https://etherscan.io/address/0x742d35Cc...
/// ```
pub fn get_address_url(chain_id: u64, address: &str) -> EvmResult<String> {
    let base_url = get_block_explorer_url(chain_id)?;
    let addr = crate::address::ensure_0x_prefix(address);
    Ok(format!("{}/address/{}", base_url, addr))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_chain_name_conversion() {
        assert_eq!(chain_name_to_id("ethereum").unwrap(), 1);
        assert_eq!(chain_name_to_id("ETH").unwrap(), 1);
        assert_eq!(chain_name_to_id("polygon").unwrap(), 137);
        assert_eq!(chain_name_to_id("ARBITRUM").unwrap(), 42161);

        // Test invalid name
        assert!(chain_name_to_id("invalid").is_err());
    }

    #[test]
    fn test_chain_id_to_name() {
        assert_eq!(chain_id_to_name(1).unwrap(), "ethereum");
        assert_eq!(chain_id_to_name(137).unwrap(), "polygon");
        assert_eq!(chain_id_to_name(42161).unwrap(), "arbitrum");

        // Test invalid ID
        assert!(chain_id_to_name(999999).is_err());
    }

    #[test]
    fn test_chain_info() {
        let eth_info = get_chain_info(1).unwrap();
        assert_eq!(eth_info.name, "Ethereum");
        assert_eq!(eth_info.symbol, "ETH");
        assert!(eth_info.block_explorer.is_some());

        // Test unknown chain
        assert!(get_chain_info(999999).is_none());
    }

    #[test]
    fn test_rpc_url_resolution() {
        // Test with environment variable
        env::set_var("RPC_URL_999", "https://test-rpc.example.com");
        let result = chain_id_to_rpc_url(999);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "https://test-rpc.example.com");
        env::remove_var("RPC_URL_999");

        // Test with default (Ethereum should have default)
        // Clear any existing env var to test default behavior
        env::remove_var("RPC_URL_1");
        let result = chain_id_to_rpc_url(1);
        assert!(result.is_ok()); // Should use default RPC
    }

    #[test]
    fn test_invalid_rpc_url() {
        env::set_var("RPC_URL_998", "invalid-url");
        let result = chain_id_to_rpc_url(998);
        assert!(result.is_err());
        env::remove_var("RPC_URL_998");
    }

    #[test]
    fn test_supported_chains() {
        let chains = get_supported_chains();
        assert!(!chains.is_empty());

        // Should include major chains with defaults
        assert!(chains.contains(&1)); // Ethereum
    }

    #[test]
    fn test_block_explorer_urls() {
        let eth_url = get_block_explorer_url(1).unwrap();
        assert_eq!(eth_url, "https://etherscan.io");

        let polygon_url = get_block_explorer_url(137).unwrap();
        assert_eq!(polygon_url, "https://polygonscan.com");

        // Test invalid chain
        assert!(get_block_explorer_url(999999).is_err());
    }

    #[test]
    fn test_transaction_url() {
        let url = get_transaction_url(1, "0x123abc").unwrap();
        assert_eq!(url, "https://etherscan.io/tx/0x123abc");

        // Test without 0x prefix
        let url = get_transaction_url(1, "123abc").unwrap();
        assert_eq!(url, "https://etherscan.io/tx/0x123abc");
    }

    #[test]
    fn test_address_url() {
        let url = get_address_url(1, "0x742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e").unwrap();
        assert_eq!(
            url,
            "https://etherscan.io/address/0x742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e"
        );

        // Test without 0x prefix
        let url = get_address_url(1, "742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e").unwrap();
        assert_eq!(
            url,
            "https://etherscan.io/address/0x742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e"
        );
    }
}
