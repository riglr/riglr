//! Utility functions for EVM operations

use alloy::providers::{Provider, ProviderBuilder};
use alloy::network::Ethereum;
use alloy::primitives::Address;
use alloy::rpc::types::TransactionRequest;
use anyhow::{anyhow, Result};
use riglr_core::SignerContext;
use crate::error::EvmToolError;
use std::future::Future;
use std::str::FromStr;
use std::sync::Arc;

pub type EvmProvider = Arc<dyn Provider<Ethereum>>;

/// Maps chain IDs to RPC URLs from environment variables
pub fn chain_id_to_rpc_url(chain_id: u64) -> Result<String> {
    match chain_id {
        1 => Ok(riglr_core::util::must_get_env("ETHEREUM_RPC_URL")),
        42161 => Ok(riglr_core::util::must_get_env("ARBITRUM_RPC_URL")), 
        137 => Ok(riglr_core::util::must_get_env("POLYGON_RPC_URL")),
        8453 => Ok(riglr_core::util::must_get_env("BASE_RPC_URL")),
        _ => Err(anyhow!("Unsupported chain ID: {}", chain_id)),
    }
}

/// Validates if a chain ID is supported
pub fn is_supported_chain(chain_id: u64) -> bool {
    matches!(chain_id, 1 | 42161 | 137 | 8453)
}

/// Factory function for creating EVM providers
/// Centralizes provider creation and ensures consistent configuration
pub fn make_provider(chain_id: u64) -> Result<EvmProvider, EvmToolError> {
    let rpc_url = chain_id_to_rpc_url(chain_id)
        .map_err(|_e| EvmToolError::UnsupportedChain(chain_id))?;
    
    let url = rpc_url.parse().map_err(|e| {
        EvmToolError::ProviderError(format!("Invalid RPC URL: {}", e))
    })?;
    
    let provider = ProviderBuilder::new()
        .connect_http(url);
    
    Ok(Arc::new(provider) as EvmProvider)
}

/// Higher-order function to execute EVM transactions
/// Abstracts signer context retrieval and transaction signing
pub async fn execute_evm_transaction<F, Fut>(
    chain_id: u64,
    tx_creator: F
) -> Result<String, EvmToolError>
where
    F: FnOnce(Address, EvmProvider) -> Fut + Send + 'static,
    Fut: Future<Output = Result<TransactionRequest, EvmToolError>> + Send + 'static,
{
    // Get signer from context
    let signer = SignerContext::current().await
        .map_err(|e| EvmToolError::SignerError(e.into()))?;
    
    // Get EVM address
    let address_str = signer.address()
        .ok_or_else(|| EvmToolError::InvalidAddress("No EVM address in signer context".to_string()))?;
    let address = Address::from_str(&address_str)
        .map_err(|e| EvmToolError::InvalidAddress(format!("Invalid address format: {}", e)))?;
    
    // Create provider
    let provider = make_provider(chain_id)?;
    
    // Execute transaction creator
    let tx = tx_creator(address, provider).await?;
    
    // Sign and send via signer context
    signer.sign_and_send_evm_transaction(tx).await
        .map_err(|e| EvmToolError::SignerError(e.into()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_chain_id_to_rpc_url_supported() {
        env::set_var("ETHEREUM_RPC_URL", "https://eth.example.com");
        env::set_var("ARBITRUM_RPC_URL", "https://arb.example.com");
        env::set_var("POLYGON_RPC_URL", "https://polygon.example.com");
        env::set_var("BASE_RPC_URL", "https://base.example.com");

        assert!(chain_id_to_rpc_url(1).is_ok());
        assert!(chain_id_to_rpc_url(42161).is_ok());
        assert!(chain_id_to_rpc_url(137).is_ok());
        assert!(chain_id_to_rpc_url(8453).is_ok());

        // Clean up
        env::remove_var("ETHEREUM_RPC_URL");
        env::remove_var("ARBITRUM_RPC_URL");
        env::remove_var("POLYGON_RPC_URL");
        env::remove_var("BASE_RPC_URL");
    }

    #[test]
    fn test_chain_id_to_rpc_url_unsupported() {
        let result = chain_id_to_rpc_url(999999);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unsupported chain ID"));
    }

    #[test]
    fn test_is_supported_chain() {
        assert!(is_supported_chain(1));     // Ethereum
        assert!(is_supported_chain(42161)); // Arbitrum
        assert!(is_supported_chain(137));   // Polygon
        assert!(is_supported_chain(8453));  // Base
        assert!(!is_supported_chain(999999)); // Unsupported
    }
}