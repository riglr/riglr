//! Utility functions for EVM operations

use crate::error::EvmToolError;
use alloy::network::Ethereum;
use alloy::primitives::Address;
use alloy::providers::{Provider, ProviderBuilder};
use alloy::rpc::types::TransactionRequest;
use anyhow::{anyhow, Result};
use riglr_core::SignerContext;
use std::future::Future;
use std::str::FromStr;
use std::sync::Arc;

/// Type alias for an Arc-wrapped Ethereum provider
pub type EvmProvider = Arc<dyn Provider<Ethereum>>;

/// Maps chain IDs to RPC URLs using convention-based environment variable lookup.
/// Uses format: RPC_URL_{CHAIN_ID}
/// Examples: RPC_URL_1 (Ethereum), RPC_URL_137 (Polygon), RPC_URL_42161 (Arbitrum)
pub fn chain_id_to_rpc_url(chain_id: u64) -> Result<String> {
    let env_var = format!("RPC_URL_{}", chain_id);

    match std::env::var(&env_var) {
        Ok(url) => {
            if url.trim().is_empty() {
                return Err(anyhow!(
                    "RPC URL for chain {} is empty. Set {} environment variable.",
                    chain_id,
                    env_var
                ));
            }

            // Validate URL format
            if !url.starts_with("http://")
                && !url.starts_with("https://")
                && !url.starts_with("wss://")
            {
                return Err(anyhow!(
                    "Invalid RPC URL format for chain {}: {}. Must start with http://, https://, or wss://",
                    chain_id, url
                ));
            }

            tracing::debug!(
                "✅ Found RPC URL for chain {}: {}",
                chain_id,
                &url[..std::cmp::min(50, url.len())]
            );
            Ok(url)
        }
        Err(_) => Err(anyhow!(
            "No RPC URL configured for chain ID {}. Set {} environment variable.\n\
                 Supported chains require RPC_URL_{{CHAIN_ID}} format.\n\
                 Example: RPC_URL_1=https://eth-mainnet.alchemyapi.io/v2/your-key",
            chain_id,
            env_var
        )),
    }
}

/// Helper function to check if a chain is supported (has RPC URL configured)
pub fn is_supported_chain(chain_id: u64) -> bool {
    let env_var = format!("RPC_URL_{}", chain_id);
    std::env::var(&env_var).is_ok()
}

/// Returns a list of all configured chain IDs by scanning environment variables
pub fn get_supported_chains() -> Vec<u64> {
    std::env::vars()
        .filter_map(|(key, _value)| {
            if key.starts_with("RPC_URL_") {
                key.strip_prefix("RPC_URL_")
                    .and_then(|chain_id_str| chain_id_str.parse::<u64>().ok())
            } else {
                None
            }
        })
        .collect()
}

/// Factory function for creating EVM providers
/// Centralizes provider creation and ensures consistent configuration
pub fn make_provider(chain_id: u64) -> Result<EvmProvider, EvmToolError> {
    let rpc_url =
        chain_id_to_rpc_url(chain_id).map_err(|_e| EvmToolError::UnsupportedChain(chain_id))?;

    let url = rpc_url
        .parse()
        .map_err(|e| EvmToolError::ProviderError(format!("Invalid RPC URL: {}", e)))?;

    let provider = ProviderBuilder::new().connect_http(url);

    Ok(Arc::new(provider) as EvmProvider)
}

/// Higher-order function to execute EVM transactions
/// Abstracts signer context retrieval and transaction signing
pub async fn execute_evm_transaction<F, Fut>(
    chain_id: u64,
    tx_creator: F,
) -> Result<String, EvmToolError>
where
    F: FnOnce(Address, EvmProvider) -> Fut + Send + 'static,
    Fut: Future<Output = Result<TransactionRequest, EvmToolError>> + Send + 'static,
{
    // Get signer from context
    let signer = SignerContext::current_as_evm()
        .await
        .map_err(EvmToolError::SignerError)?;

    // Get EVM address
    let address_str = signer.address();
    let address = Address::from_str(&address_str)
        .map_err(|e| EvmToolError::InvalidAddress(format!("Invalid address format: {}", e)))?;

    // Create provider
    let provider = make_provider(chain_id)?;

    // Execute transaction creator
    let tx = tx_creator(address, provider).await?;

    // Sign and send via signer context
    signer
        .sign_and_send_transaction(tx)
        .await
        .map_err(EvmToolError::SignerError)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_convention_based_chain_lookup() {
        // Test successful lookup
        env::set_var("RPC_URL_999", "https://test-rpc.example.com");
        let result = chain_id_to_rpc_url(999);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "https://test-rpc.example.com");

        // Clean up
        env::remove_var("RPC_URL_999");
    }

    #[test]
    fn test_missing_chain_error() {
        let result = chain_id_to_rpc_url(99999);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("No RPC URL configured"));
    }

    #[test]
    fn test_invalid_url_format() {
        env::set_var("RPC_URL_998", "invalid-url");
        let result = chain_id_to_rpc_url(998);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid RPC URL format"));

        env::remove_var("RPC_URL_998");
    }

    #[test]
    fn test_get_supported_chains() {
        env::set_var("RPC_URL_1", "https://eth.example.com");
        env::set_var("RPC_URL_137", "https://polygon.example.com");

        let chains = get_supported_chains();
        assert!(chains.contains(&1));
        assert!(chains.contains(&137));

        env::remove_var("RPC_URL_1");
        env::remove_var("RPC_URL_137");
    }

    #[test]
    fn test_chain_id_to_rpc_url_supported() {
        // Test each chain individually to avoid interference
        env::set_var("RPC_URL_1", "https://eth.example.com");
        let result1 = chain_id_to_rpc_url(1);
        assert!(result1.is_ok());
        env::remove_var("RPC_URL_1");

        env::set_var("RPC_URL_42161", "https://arb.example.com");
        let result2 = chain_id_to_rpc_url(42161);
        assert!(result2.is_ok());
        env::remove_var("RPC_URL_42161");

        env::set_var("RPC_URL_137", "https://polygon.example.com");
        let result3 = chain_id_to_rpc_url(137);
        assert!(result3.is_ok());
        env::remove_var("RPC_URL_137");

        env::set_var("RPC_URL_8453", "https://base.example.com");
        let result4 = chain_id_to_rpc_url(8453);
        assert!(result4.is_ok());
        env::remove_var("RPC_URL_8453");
    }

    #[test]
    fn test_is_supported_chain() {
        // Clean state first
        env::remove_var("RPC_URL_1");
        env::remove_var("RPC_URL_42161");
        env::remove_var("RPC_URL_137");
        env::remove_var("RPC_URL_8453");
        env::remove_var("RPC_URL_999999");

        // Test unsupported first
        assert!(!is_supported_chain(999999));

        // Test supported chains individually
        env::set_var("RPC_URL_1", "https://eth.example.com");
        assert!(is_supported_chain(1));
        env::remove_var("RPC_URL_1");

        env::set_var("RPC_URL_42161", "https://arb.example.com");
        assert!(is_supported_chain(42161));
        env::remove_var("RPC_URL_42161");

        env::set_var("RPC_URL_137", "https://polygon.example.com");
        assert!(is_supported_chain(137));
        env::remove_var("RPC_URL_137");

        env::set_var("RPC_URL_8453", "https://base.example.com");
        assert!(is_supported_chain(8453));
        env::remove_var("RPC_URL_8453");
    }

    #[test]
    fn test_chain_id_to_rpc_url_when_empty_url_should_return_err() {
        // Test empty URL
        env::set_var("RPC_URL_997", "");
        let result = chain_id_to_rpc_url(997);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("RPC URL for chain 997 is empty"));
        env::remove_var("RPC_URL_997");
    }

    #[test]
    fn test_chain_id_to_rpc_url_when_whitespace_only_url_should_return_err() {
        // Test whitespace-only URL
        env::set_var("RPC_URL_996", "   ");
        let result = chain_id_to_rpc_url(996);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("RPC URL for chain 996 is empty"));
        env::remove_var("RPC_URL_996");
    }

    #[test]
    fn test_chain_id_to_rpc_url_when_http_protocol_should_return_ok() {
        // Test HTTP protocol
        env::set_var("RPC_URL_995", "http://test-rpc.example.com");
        let result = chain_id_to_rpc_url(995);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "http://test-rpc.example.com");
        env::remove_var("RPC_URL_995");
    }

    #[test]
    fn test_chain_id_to_rpc_url_when_https_protocol_should_return_ok() {
        // Test HTTPS protocol
        env::set_var("RPC_URL_994", "https://test-rpc.example.com");
        let result = chain_id_to_rpc_url(994);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "https://test-rpc.example.com");
        env::remove_var("RPC_URL_994");
    }

    #[test]
    fn test_chain_id_to_rpc_url_when_wss_protocol_should_return_ok() {
        // Test WSS protocol
        env::set_var("RPC_URL_993", "wss://test-rpc.example.com");
        let result = chain_id_to_rpc_url(993);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "wss://test-rpc.example.com");
        env::remove_var("RPC_URL_993");
    }

    #[test]
    fn test_chain_id_to_rpc_url_when_ftp_protocol_should_return_err() {
        // Test invalid protocol (FTP)
        env::set_var("RPC_URL_992", "ftp://test-rpc.example.com");
        let result = chain_id_to_rpc_url(992);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid RPC URL format for chain 992"));
        env::remove_var("RPC_URL_992");
    }

    #[test]
    fn test_chain_id_to_rpc_url_when_no_protocol_should_return_err() {
        // Test URL without protocol
        env::set_var("RPC_URL_991", "test-rpc.example.com");
        let result = chain_id_to_rpc_url(991);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid RPC URL format for chain 991"));
        env::remove_var("RPC_URL_991");
    }

    #[test]
    fn test_get_supported_chains_when_no_rpc_vars_should_return_empty() {
        // Clean all RPC_URL_ vars first
        for (key, _) in env::vars() {
            if key.starts_with("RPC_URL_") {
                env::remove_var(&key);
            }
        }

        let chains = get_supported_chains();
        assert!(chains.is_empty());
    }

    #[test]
    fn test_get_supported_chains_when_malformed_chain_ids_should_ignore() {
        // Clean state first
        for (key, _) in env::vars() {
            if key.starts_with("RPC_URL_") {
                env::remove_var(&key);
            }
        }

        // Set valid and invalid chain IDs
        env::set_var("RPC_URL_1", "https://eth.example.com");
        env::set_var("RPC_URL_abc", "https://invalid.example.com"); // Invalid chain ID
        env::set_var("RPC_URL_", "https://empty.example.com"); // Empty chain ID
        env::set_var("RPC_URL_42", "https://valid.example.com");

        let chains = get_supported_chains();
        assert!(chains.contains(&1));
        assert!(chains.contains(&42));
        assert!(!chains.contains(&0)); // abc and empty should be ignored
        assert_eq!(chains.len(), 2);

        // Clean up
        env::remove_var("RPC_URL_1");
        env::remove_var("RPC_URL_abc");
        env::remove_var("RPC_URL_");
        env::remove_var("RPC_URL_42");
    }

    #[test]
    fn test_get_supported_chains_when_non_rpc_vars_should_ignore() {
        // Clean state first
        for (key, _) in env::vars() {
            if key.starts_with("RPC_URL_") {
                env::remove_var(&key);
            }
        }

        // Set non-RPC variables and one RPC variable
        env::set_var("OTHER_VAR_1", "value");
        env::set_var("RPC_URL_1", "https://eth.example.com");
        env::set_var("NOT_RPC_URL_2", "value");

        let chains = get_supported_chains();
        assert!(chains.contains(&1));
        assert_eq!(chains.len(), 1);

        // Clean up
        env::remove_var("OTHER_VAR_1");
        env::remove_var("RPC_URL_1");
        env::remove_var("NOT_RPC_URL_2");
    }

    #[test]
    fn test_make_provider_when_unsupported_chain_should_return_err() {
        // Test with chain that has no RPC URL configured
        env::remove_var("RPC_URL_999999");
        let result = make_provider(999999);
        assert!(result.is_err());
        if let Err(EvmToolError::UnsupportedChain(chain_id)) = result {
            assert_eq!(chain_id, 999999);
        } else {
            panic!("Expected UnsupportedChain error");
        }
    }

    #[test]
    fn test_make_provider_when_valid_chain_should_return_ok() {
        // Test with valid RPC URL
        env::set_var("RPC_URL_1337", "https://test-rpc.example.com");
        let result = make_provider(1337);
        assert!(result.is_ok());
        env::remove_var("RPC_URL_1337");
    }

    #[test]
    fn test_make_provider_when_invalid_url_format_should_return_err() {
        // Test with malformed URL that will fail parsing
        env::set_var("RPC_URL_1338", "https://[invalid-url");
        let result = make_provider(1338);
        assert!(result.is_err());
        if let Err(EvmToolError::ProviderError(msg)) = result {
            assert!(msg.contains("Invalid RPC URL"));
        } else {
            panic!("Expected ProviderError");
        }
        env::remove_var("RPC_URL_1338");
    }
}
