//! EVM provider utilities
//!
//! This module contains provider-specific functionality that depends on
//! riglr-core and tool-specific error types.

extern crate alloc;

use crate::error::Error as EvmToolError;
use alloc::sync::Arc;
use alloy::primitives::Address;
use alloy::providers::{ProviderBuilder, RootProvider};
use alloy::rpc::types::TransactionRequest;
use alloy::transports::http::Http;
use core::future::Future;
use core::str::FromStr as _;
use riglr_core::{signer::error::Standard, SignerContext};
use riglr_evm_common::id_to_rpc_url;

/// Type alias for an Arc-wrapped Ethereum HTTP client
pub type HttpClient = Arc<RootProvider<Http<reqwest::Client>>>;

/// Factory function for creating EVM providers
/// Centralizes provider creation and ensures consistent configuration
///
/// # Errors
///
/// Returns `EvmToolError::UnsupportedChain` if the chain ID is not supported.
/// Returns `EvmToolError::ProviderError` if the RPC URL is malformed.
#[inline]
pub fn create_http_client(chain_id: u64) -> Result<HttpClient, EvmToolError> {
    let rpc_url =
        id_to_rpc_url(chain_id).map_err(|_error| EvmToolError::UnsupportedChain(chain_id))?;

    let url = rpc_url
        .parse()
        .map_err(|error| EvmToolError::ProviderError(format!("Invalid RPC URL: {error}")))?;

    let provider = ProviderBuilder::new().on_http(url);

    Ok(Arc::new(provider))
}

/// Higher-order function to execute EVM transactions
/// Abstracts signer context retrieval and transaction signing
///
/// # Errors
///
/// Returns `EvmToolError::SignerError` if signer context is unavailable or signing fails.
/// Returns `EvmToolError::InvalidAddress` if the address format is invalid.
/// Returns `EvmToolError::Generic` if transaction serialization fails.
/// Returns errors from the provided transaction creator function.
#[inline]
pub async fn execute_evm_transaction<F, Fut>(
    chain_id: u64,
    tx_creator: F,
) -> Result<String, EvmToolError>
where
    F: FnOnce(Address, HttpClient) -> Fut + Send + 'static,
    Fut: Future<Output = Result<TransactionRequest, EvmToolError>> + Send + 'static,
{
    // Get signer from context
    let signer = SignerContext::current_as_evm()
        .map_err(|error| EvmToolError::SignerError(Standard::Generic(error.to_string())))?;

    // Get EVM address
    let address_str = signer.signer().address();
    let address = Address::from_str(&address_str).map_err(|error| {
        EvmToolError::InvalidAddress(format!("Invalid address format: {error}"))
    })?;

    // Create provider
    let provider = create_http_client(chain_id)?;

    // Execute transaction creator
    let tx = tx_creator(address, provider).await?;

    // Sign and send via signer context
    let tx_json = serde_json::to_value(&tx).map_err(|error| {
        EvmToolError::Generic(format!("Failed to serialize transaction: {error}"))
    })?;
    signer
        .signer()
        .sign_and_send_transaction(tx_json)
        .await
        .map_err(|error| EvmToolError::SignerError(Standard::Generic(error.to_string())))
}
