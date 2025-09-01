//! EVM provider utilities
//!
//! This module contains provider-specific functionality that depends on
//! riglr-core and tool-specific error types.

use crate::error::EvmToolError;
use alloy::network::Ethereum;
use alloy::primitives::Address;
use alloy::providers::{Provider, ProviderBuilder};
use alloy::rpc::types::TransactionRequest;
use riglr_core::SignerContext;
use riglr_evm_common::chain_id_to_rpc_url;
use std::future::Future;
use std::str::FromStr;
use std::sync::Arc;

/// Type alias for an Arc-wrapped Ethereum provider
pub type EvmProvider = Arc<dyn Provider<Ethereum>>;

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
    let tx_json = serde_json::to_value(&tx)
        .map_err(|e| EvmToolError::Generic(format!("Failed to serialize transaction: {}", e)))?;
    signer
        .sign_and_send_transaction(tx_json)
        .await
        .map_err(EvmToolError::SignerError)
}
