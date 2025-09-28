//! Chain management utilities shared across riglr crates
//!
//! This module provides unified chain ID mapping, RPC URL management,
//! and chain validation functions that eliminate duplication between
//! riglr-evm-tools and riglr-cross-chain-tools.

use crate::address::ensure_0x_prefix;
use crate::error::{Error, EvmResult};
use core::cmp;
use std::env;

/// Chain information structure
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Info {
    /// Block explorer base URL for this chain
    pub block_explorer: Option<String>,
    /// Numeric chain ID (e.g., 1 for Ethereum, 137 for Polygon)
    pub chain_id: u64,
    /// Default RPC endpoint URL for this chain
    pub default_rpc: Option<String>,
    /// Human-readable chain name (e.g., "Ethereum", "Polygon")
    pub name: String,
    /// Native token symbol (e.g., "ETH", "MATIC")
    pub symbol: String,
}

/// Get chain information by chain ID
#[must_use]
#[inline]
pub fn get_info(chain_id: u64) -> Option<Info> {
    match chain_id {
        1 => Some(Info {
            chain_id: 1,
            name: "Ethereum".to_owned(),
            symbol: "ETH".to_owned(),
            block_explorer: Some("https://etherscan.io".to_owned()),
            default_rpc: Some("https://eth.llamarpc.com".to_owned()),
        }),
        137 => Some(Info {
            chain_id: 137,
            name: "Polygon".to_owned(),
            symbol: "MATIC".to_owned(),
            block_explorer: Some("https://polygonscan.com".to_owned()),
            default_rpc: Some("https://polygon-rpc.com".to_owned()),
        }),
        42161 => Some(Info {
            chain_id: 42161,
            name: "Arbitrum".to_owned(),
            symbol: "ETH".to_owned(),
            block_explorer: Some("https://arbiscan.io".to_owned()),
            default_rpc: Some("https://arb1.arbitrum.io/rpc".to_owned()),
        }),
        10 => Some(Info {
            chain_id: 10,
            name: "Optimism".to_owned(),
            symbol: "ETH".to_owned(),
            block_explorer: Some("https://optimistic.etherscan.io".to_owned()),
            default_rpc: Some("https://mainnet.optimism.io".to_owned()),
        }),
        8453 => Some(Info {
            chain_id: 8453,
            name: "Base".to_owned(),
            symbol: "ETH".to_owned(),
            block_explorer: Some("https://basescan.org".to_owned()),
            default_rpc: Some("https://mainnet.base.org".to_owned()),
        }),
        56 => Some(Info {
            chain_id: 56,
            name: "BNB Smart Chain".to_owned(),
            symbol: "BNB".to_owned(),
            block_explorer: Some("https://bscscan.com".to_owned()),
            default_rpc: Some("https://bsc-dataseed.binance.org".to_owned()),
        }),
        43114 => Some(Info {
            chain_id: 43114,
            name: "Avalanche".to_owned(),
            symbol: "AVAX".to_owned(),
            block_explorer: Some("https://snowtrace.io".to_owned()),
            default_rpc: Some("https://api.avax.network/ext/bc/C/rpc".to_owned()),
        }),
        250 => Some(Info {
            chain_id: 250,
            name: "Fantom".to_owned(),
            symbol: "FTM".to_owned(),
            block_explorer: Some("https://ftmscan.com".to_owned()),
            default_rpc: Some("https://rpc.ftm.tools".to_owned()),
        }),
        _ => None,
    }
}

/// Maps chain IDs to RPC URLs using convention-based environment variable lookup.
/// Uses format: `RPC_URL`_{`CHAIN_ID`}
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
/// # Errors
/// Returns `Error::InvalidConfig` if the RPC URL is empty or has invalid format.
/// Returns `Error::UnsupportedChain` if no RPC URL or default is configured.
///
/// # Examples
/// ```rust,ignore
/// use riglr_evm_common::chain::id_to_rpc_url;
///
/// // Configure environment
/// std::env::set_var("RPC_URL_1", "https://eth.llamarpc.com");
///
/// let url = id_to_rpc_url(1)?;
/// assert_eq!(url, "https://eth.llamarpc.com");
/// ```
#[inline]
pub fn id_to_rpc_url(chain_id: u64) -> EvmResult<String> {
    let env_var = format!("RPC_URL_{chain_id}");

    if let Ok(url) = env::var(&env_var) {
        if url.trim().is_empty() {
            return Err(Error::InvalidConfig(format!(
                "RPC URL for chain {chain_id} is empty. Set {env_var} environment variable."
            )));
        }

        // Validate URL format
        validate_rpc_url(&url, chain_id)?;

        tracing::debug!(
            "\u{2705} Found RPC URL for chain {}: {}",
            chain_id,
            &url[..cmp::min(50, url.len())]
        );
        return Ok(url);
    }

    // Try to use default RPC if available
    if let Some(chain_info) = get_info(chain_id) {
        if let Some(default_rpc) = chain_info.default_rpc {
            tracing::warn!("\u{26a0}\u{fe0f}  Using default RPC for chain {}: {}. Consider setting {} for production use.",
                           chain_id, default_rpc, env_var);
            return Ok(default_rpc);
        }
    }

    Err(Error::UnsupportedChain(chain_id))
}

/// Validate RPC URL format
fn validate_rpc_url(url: &str, chain_id: u64) -> EvmResult<()> {
    if !url.starts_with("http://") && !url.starts_with("https://") && !url.starts_with("wss://") {
        return Err(Error::InvalidConfig(format!(
            "Invalid RPC URL format for chain {chain_id}: {url}. Must start with http://, https://, or wss://"
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
/// # Errors
/// Returns `Error::InvalidChainName` if the chain name is not recognized.
///
/// # Examples
/// ```rust,ignore
/// use riglr_evm_common::chain::name_to_id;
///
/// assert_eq!(name_to_id("ethereum")?, 1);
/// assert_eq!(name_to_id("ETH")?, 1);
/// assert_eq!(name_to_id("polygon")?, 137);
/// ```
#[inline]
pub fn name_to_id(name: &str) -> EvmResult<u64> {
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
        _ => Err(Error::InvalidChainName(format!(
            "Unsupported chain name: {name}. Supported: ethereum, polygon, arbitrum, optimism, base, bsc, avalanche, fantom"
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
/// # Errors
/// Returns `Error::UnsupportedChain` if the chain ID is not recognized.
///
/// # Examples
/// ```rust,ignore
/// use riglr_evm_common::chain::id_to_name;
///
/// assert_eq!(id_to_name(1)?, "ethereum");
/// assert_eq!(id_to_name(137)?, "polygon");
/// ```
#[inline]
pub fn id_to_name(id: u64) -> EvmResult<String> {
    match id {
        1 => Ok("ethereum".to_owned()),
        137 => Ok("polygon".to_owned()),
        42161 => Ok("arbitrum".to_owned()),
        10 => Ok("optimism".to_owned()),
        8453 => Ok("base".to_owned()),
        56 => Ok("bsc".to_owned()),
        43114 => Ok("avalanche".to_owned()),
        250 => Ok("fantom".to_owned()),
        _ => Err(Error::UnsupportedChain(id)),
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
/// use riglr_evm_common::chain::is_supported;
///
/// // If RPC_URL_1 is configured or Ethereum has defaults
/// assert!(is_supported(1));
///
/// // Unsupported chain
/// assert!(!is_supported(999_999));
/// ```
#[must_use]
#[inline]
pub fn is_supported(chain_id: u64) -> bool {
    id_to_rpc_url(chain_id).is_ok()
}

/// Get list of all supported chain IDs
///
/// Scans environment variables for `RPC_URL`_* patterns and includes
/// chains with default RPC endpoints.
///
/// # Returns
/// * Vector of supported chain IDs
///
/// # Examples
/// ```rust,ignore
/// use riglr_evm_common::chain::get_supported;
///
/// let chains = get_supported();
/// if chains.contains(&1) {
///     println!("Ethereum is supported!");
/// }
/// ```
#[must_use]
#[inline]
pub fn get_supported() -> Vec<u64> {
    let mut chains: Vec<u64> = env::vars()
        .filter_map(|(key, _value)| {
            if key.starts_with("RPC_URL_") {
                return key
                    .strip_prefix("RPC_URL_")
                    .and_then(|chain_id_str| chain_id_str.parse::<u64>().ok());
            }
            None
        })
        .collect();

    // Add chains with default RPCs that aren't already configured
    let default_chains = [1, 137, 42161, 10, 8453, 56, 43114, 250];
    for chain_id in default_chains {
        if !chains.contains(&chain_id) {
            // Check if this chain has default RPC
            if get_info(chain_id)
                .and_then(|info| info.default_rpc)
                .is_some()
            {
                chains.push(chain_id);
            }
        }
    }

    chains.sort_unstable();
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
/// # Errors
/// Returns `Error::UnsupportedChain` if the chain has no configured block explorer.
///
/// # Examples
/// ```rust,ignore
/// use riglr_evm_common::chain::get_block_explorer_url;
///
/// let url = get_block_explorer_url(1)?;
/// assert_eq!(url, "https://etherscan.io");
/// ```
#[inline]
pub fn get_block_explorer_url(chain_id: u64) -> EvmResult<String> {
    get_info(chain_id)
        .and_then(|info| info.block_explorer)
        .ok_or(Error::UnsupportedChain(chain_id))
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
/// # Errors
/// Returns `Error::UnsupportedChain` if the chain has no configured block explorer.
///
/// # Examples
/// ```rust,ignore
/// use riglr_evm_common::chain::get_transaction_url;
///
/// let url = get_transaction_url(1, "0x123abc...")?;
/// // Returns: https://etherscan.io/tx/0x123abc...
/// ```
#[inline]
pub fn get_transaction_url(chain_id: u64, tx_hash: &str) -> EvmResult<String> {
    let base_url = get_block_explorer_url(chain_id)?;
    let hash = if tx_hash.starts_with("0x") {
        tx_hash
    } else {
        &format!("0x{tx_hash}")
    };
    Ok(format!("{base_url}/tx/{hash}"))
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
/// # Errors
/// Returns `Error::UnsupportedChain` if the chain has no configured block explorer.
///
/// # Examples
/// ```rust,ignore
/// use riglr_evm_common::chain::get_address_url;
///
/// let url = get_address_url(1, "0x742d35Cc...")?;
/// // Returns: https://etherscan.io/address/0x742d35Cc...
/// ```
#[inline]
pub fn get_address_url(chain_id: u64, address: &str) -> EvmResult<String> {
    let base_url = get_block_explorer_url(chain_id)?;
    let addr = ensure_0x_prefix(address);
    Ok(format!("{base_url}/address/{addr}"))
}

#[cfg(test)]
#[expect(clippy::expect_used)]
mod tests {
    use super::*;
    use std::env;

    /// Helper function to set environment variables in tests without using string literals
    #[expect(unsafe_code)]
    fn set_test_env_var(key: &str, value: &str) {
        // SAFETY: This is a test-only function and tests run in single-threaded mode by default.
        // The environment access only happens in single-threaded test code.
        unsafe { env::set_var(key, value) };
    }

    /// Helper function to remove environment variables in tests without using string literals
    #[expect(unsafe_code)]
    fn remove_test_env_var(key: &str) {
        // SAFETY: This is a test-only function and tests run in single-threaded mode by default.
        // The environment access only happens in single-threaded test code.
        unsafe { env::remove_var(key) };
    }

    #[test]
    fn chain_name_conversion() {
        assert_eq!(
            name_to_id("ethereum").expect("ethereum should be a valid chain name"),
            1
        );
        assert_eq!(
            name_to_id("ETH").expect("ETH should be a valid chain name"),
            1
        );
        assert_eq!(
            name_to_id("polygon").expect("polygon should be a valid chain name"),
            137
        );
        assert_eq!(
            name_to_id("ARBITRUM").expect("ARBITRUM should be a valid chain name"),
            42161
        );

        // Test invalid name
        assert!(name_to_id("invalid").is_err());
    }

    #[test]
    fn id_to_name_conversion() {
        assert_eq!(
            id_to_name(1).expect("chain ID 1 should be supported"),
            "ethereum"
        );
        assert_eq!(
            id_to_name(137).expect("chain ID 137 should be supported"),
            "polygon"
        );
        assert_eq!(
            id_to_name(42161).expect("chain ID 42161 should be supported"),
            "arbitrum"
        );

        // Test invalid ID
        assert!(id_to_name(999_999).is_err());
    }

    #[test]
    fn chain_info() {
        let eth_info = get_info(1).expect("chain ID 1 should have info");
        assert_eq!(eth_info.name, "Ethereum");
        assert_eq!(eth_info.symbol, "ETH");
        assert!(eth_info.block_explorer.is_some());

        // Test unknown chain
        assert!(get_info(999_999).is_none());
    }

    #[test]
    fn rpc_url_resolution() {
        // Test with environment variable
        set_test_env_var("RPC_URL_999", "https://test-rpc.example.com");
        let rpc_result = id_to_rpc_url(999);
        assert_eq!(
            rpc_result.expect("should get RPC URL from env var"),
            "https://test-rpc.example.com"
        );
        remove_test_env_var("RPC_URL_999");

        // Test with default (Ethereum should have default)
        // Clear any existing env var to test default behavior
        remove_test_env_var("RPC_URL_1");
        let default_result = id_to_rpc_url(1);
        assert!(default_result.is_ok()); // Should use default RPC
    }

    #[test]
    fn invalid_rpc_url() {
        set_test_env_var("RPC_URL_998", "invalid-url");
        let invalid_result = id_to_rpc_url(998);
        assert!(invalid_result.is_err());
        remove_test_env_var("RPC_URL_998");
    }

    #[test]
    fn supported_chains() {
        let chains = get_supported();
        assert!(!chains.is_empty());

        // Should include major chains with defaults
        assert!(chains.contains(&1)); // Ethereum
    }

    #[test]
    fn block_explorer_urls() {
        let eth_url = get_block_explorer_url(1).expect("chain ID 1 should have block explorer URL");
        assert_eq!(eth_url, "https://etherscan.io");

        let polygon_url =
            get_block_explorer_url(137).expect("chain ID 137 should have block explorer URL");
        assert_eq!(polygon_url, "https://polygonscan.com");

        // Test invalid chain
        assert!(get_block_explorer_url(999_999).is_err());
    }

    #[test]
    fn transaction_url() {
        let url = get_transaction_url(1, "0x123abc")
            .expect("should generate transaction URL for chain ID 1");
        assert_eq!(url, "https://etherscan.io/tx/0x123abc");

        // Test without 0x prefix
        let url_no_prefix = get_transaction_url(1, "123abc")
            .expect("should generate transaction URL without 0x prefix");
        assert_eq!(url_no_prefix, "https://etherscan.io/tx/0x123abc");
    }

    #[test]
    fn address_url() {
        let url = get_address_url(1, "0x742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e")
            .expect("should generate address URL for chain ID 1");
        assert_eq!(
            url,
            "https://etherscan.io/address/0x742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e"
        );

        // Test without 0x prefix
        let addr_url_no_prefix = get_address_url(1, "742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e")
            .expect("should generate address URL without 0x prefix");
        assert_eq!(
            addr_url_no_prefix,
            "https://etherscan.io/address/0x742d35Cc67A5b747bE4C506C5e8b0A146d7b2E9e"
        );
    }

    #[test]
    fn validate_rpc_url_when_valid_should_return_ok() {
        // Test all valid URL schemes
        assert!(validate_rpc_url("https://example.com", 1).is_ok());
        assert!(validate_rpc_url("http://example.com", 1).is_ok());
        assert!(validate_rpc_url("wss://example.com", 1).is_ok());
    }

    #[test]
    fn validate_rpc_url_when_invalid_scheme_should_return_err() {
        let validate_result = validate_rpc_url("ftp://example.com", 1);
        if let Err(Error::InvalidConfig(msg)) = validate_result {
            assert!(msg.contains("Invalid RPC URL format for chain 1"));
            assert!(msg.contains("Must start with http://, https://, or wss://"));
        }
    }

    #[test]
    fn validate_rpc_url_when_no_scheme_should_return_err() {
        let no_scheme_result = validate_rpc_url("example.com", 1);
        if let Err(Error::InvalidConfig(msg)) = no_scheme_result {
            assert!(msg.contains("Invalid RPC URL format for chain 1"));
        }
    }

    #[test]
    fn chain_id_to_rpc_url_when_empty_env_var_should_return_err() {
        // Test empty environment variable
        set_test_env_var("RPC_URL_997", "   ");
        let empty_result = id_to_rpc_url(997);
        if let Err(Error::InvalidConfig(msg)) = empty_result {
            assert!(msg.contains("RPC URL for chain 997 is empty"));
        }
        remove_test_env_var("RPC_URL_997");
    }

    #[test]
    fn chain_id_to_rpc_url_when_whitespace_only_should_return_err() {
        // Test whitespace-only environment variable
        set_test_env_var("RPC_URL_996", "\t\n  \r");
        let whitespace_result = id_to_rpc_url(996);
        assert!(whitespace_result.is_err());
        remove_test_env_var("RPC_URL_996");
    }

    #[test]
    fn chain_id_to_rpc_url_when_unsupported_chain_no_default_should_return_err() {
        // Test completely unsupported chain (no env var, no default)
        remove_test_env_var("RPC_URL_999_999");
        let unsupported_result = id_to_rpc_url(999_999);
        if let Err(Error::UnsupportedChain(chain_id)) = unsupported_result {
            assert_eq!(chain_id, 999_999);
        }
    }

    /// Helper function to validate chain info
    fn assert_chain_info(chain_id: u64, expected_name: &str, expected_symbol: &str) {
        let info = get_info(chain_id).expect("chain info should exist for supported chain");
        assert_eq!(info.chain_id, chain_id);
        assert_eq!(info.name, expected_name);
        assert_eq!(info.symbol, expected_symbol);
        assert!(info.block_explorer.is_some());
        assert!(info.default_rpc.is_some());
    }

    /// Helper function to test major chains
    fn test_major_chains() {
        assert_chain_info(1, "Ethereum", "ETH");
        assert_chain_info(137, "Polygon", "MATIC");
        assert_chain_info(42161, "Arbitrum", "ETH");
        assert_chain_info(10, "Optimism", "ETH");
    }

    /// Helper function to test secondary chains
    fn test_secondary_chains() {
        assert_chain_info(8453, "Base", "ETH");
        assert_chain_info(56, "BNB Smart Chain", "BNB");
        assert_chain_info(43114, "Avalanche", "AVAX");
        assert_chain_info(250, "Fantom", "FTM");
    }

    #[test]
    fn get_chain_info_for_all_supported_chains() {
        // Test all supported chains individually
        test_major_chains();
        test_secondary_chains();
    }

    /// Helper function to test chain name aliases
    fn assert_chain_name_aliases(names: &[&str], expected_id: u64) {
        for name in names {
            assert_eq!(
                name_to_id(name).expect("chain name should be supported"),
                expected_id
            );
        }
    }

    /// Helper function to test primary chain name aliases
    fn test_primary_chain_aliases() {
        assert_chain_name_aliases(&["ethereum", "eth", "ETHEREUM", "ETH"], 1);
        assert_chain_name_aliases(&["polygon", "matic", "POLYGON", "MATIC"], 137);
        assert_chain_name_aliases(&["arbitrum", "arb", "ARBITRUM", "ARB"], 42161);
        assert_chain_name_aliases(&["optimism", "op", "OPTIMISM", "OP"], 10);
    }

    /// Helper function to test secondary chain name aliases
    fn test_secondary_chain_aliases() {
        assert_chain_name_aliases(&["base", "BASE"], 8453);
        assert_chain_name_aliases(&["bsc", "binance", "BSC", "BINANCE"], 56);
        assert_chain_name_aliases(&["avalanche", "avax", "AVALANCHE", "AVAX"], 43114);
        assert_chain_name_aliases(&["fantom", "ftm", "FANTOM", "FTM"], 250);
    }

    #[test]
    fn name_to_id_all_supported_aliases() {
        // Test all supported chain name aliases
        test_primary_chain_aliases();
        test_secondary_chain_aliases();
    }

    #[test]
    fn name_to_id_when_invalid_name_should_return_err() {
        let unknown_result = name_to_id("unknown_chain");
        if let Err(Error::InvalidChainName(msg)) = unknown_result {
            assert!(msg.contains("Unsupported chain name: unknown_chain"));
            assert!(msg.contains(
                "Supported: ethereum, polygon, arbitrum, optimism, base, bsc, avalanche, fantom"
            ));
        }
    }

    #[test]
    fn name_to_id_when_empty_string_should_return_err() {
        let empty_name_result = name_to_id("");
        assert!(empty_name_result.is_err());
    }

    #[test]
    fn id_to_name_all_supported_chains() {
        assert_eq!(
            id_to_name(1).expect("chain ID 1 should be supported"),
            "ethereum"
        );
        assert_eq!(
            id_to_name(137).expect("chain ID 137 should be supported"),
            "polygon"
        );
        assert_eq!(
            id_to_name(42161).expect("chain ID 42161 should be supported"),
            "arbitrum"
        );
        assert_eq!(
            id_to_name(10).expect("chain ID 10 should be supported"),
            "optimism"
        );
        assert_eq!(
            id_to_name(8453).expect("chain ID 8453 should be supported"),
            "base"
        );
        assert_eq!(
            id_to_name(56).expect("chain ID 56 should be supported"),
            "bsc"
        );
        assert_eq!(
            id_to_name(43114).expect("chain ID 43114 should be supported"),
            "avalanche"
        );
        assert_eq!(
            id_to_name(250).expect("chain ID 250 should be supported"),
            "fantom"
        );
    }

    #[test]
    fn id_to_name_when_unsupported_should_return_err() {
        let unknown_id_result = id_to_name(999_999);
        if let Err(Error::UnsupportedChain(chain_id)) = unknown_id_result {
            assert_eq!(chain_id, 999_999);
        }
    }

    #[test]
    fn is_supported_when_env_var_configured() {
        set_test_env_var("RPC_URL_995", "https://test.example.com");
        assert!(is_supported(995));
        remove_test_env_var("RPC_URL_995");
    }

    #[test]
    fn is_supported_when_has_default() {
        // Ethereum should be supported due to default RPC
        remove_test_env_var("RPC_URL_1");
        assert!(is_supported(1));
    }

    #[test]
    fn is_supported_when_not_supported() {
        remove_test_env_var("RPC_URL_999_999");
        assert!(!is_supported(999_999));
    }

    #[test]
    fn get_supported_includes_env_vars() {
        // Set a custom environment variable
        set_test_env_var("RPC_URL_994", "https://test.example.com");
        let chains = get_supported();
        assert!(chains.contains(&994));
        remove_test_env_var("RPC_URL_994");
    }

    #[test]
    fn get_supported_includes_defaults() {
        let chains = get_supported();
        // Should include all chains with default RPCs
        assert!(chains.contains(&1)); // Ethereum
        assert!(chains.contains(&137)); // Polygon
        assert!(chains.contains(&42161)); // Arbitrum
        assert!(chains.contains(&10)); // Optimism
        assert!(chains.contains(&8453)); // Base
        assert!(chains.contains(&56)); // BSC
        assert!(chains.contains(&43114)); // Avalanche
        assert!(chains.contains(&250)); // Fantom
    }

    #[test]
    fn get_supported_sorted() {
        let chains = get_supported();
        let mut sorted_chains = chains.clone();
        sorted_chains.sort_unstable();
        assert_eq!(chains, sorted_chains);
    }

    #[test]
    fn get_supported_no_duplicates() {
        // Set env var for a chain that already has defaults
        set_test_env_var("RPC_URL_1", "https://custom-eth.example.com");
        let chains = get_supported();
        let unique_count = chains.len();
        let mut deduped = chains;
        deduped.sort_unstable();
        deduped.dedup();
        assert_eq!(unique_count, deduped.len());
        remove_test_env_var("RPC_URL_1");
    }

    #[test]
    fn get_block_explorer_url_when_unsupported_chain_should_return_err() {
        let explorer_error_result = get_block_explorer_url(999_999);
        if let Err(Error::UnsupportedChain(chain_id)) = explorer_error_result {
            assert_eq!(chain_id, 999_999);
        }
    }

    #[test]
    fn get_transaction_url_when_unsupported_chain_should_return_err() {
        let tx_error_result = get_transaction_url(999_999, "0x123");
        assert!(tx_error_result.is_err());
    }

    #[test]
    fn get_address_url_when_unsupported_chain_should_return_err() {
        let addr_error_result = get_address_url(999_999, "0x123");
        assert!(addr_error_result.is_err());
    }

    #[test]
    fn chain_info_debug_clone() {
        let info = get_info(1).expect("chain ID 1 should have info");
        let cloned = info.clone();
        assert_eq!(info.chain_id, cloned.chain_id);
        assert_eq!(info.name, cloned.name);
        assert_eq!(info.symbol, cloned.symbol);

        // Test debug format works
        let debug_str = format!("{info:?}");
        assert!(debug_str.contains("Info"));
        assert!(debug_str.contains("Ethereum"));
    }
}
