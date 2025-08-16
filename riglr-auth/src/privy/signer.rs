//! Privy-specific signer implementations

#[cfg(any(feature = "solana", feature = "evm"))]
use async_trait::async_trait;

#[cfg(any(feature = "solana", feature = "evm"))]
use base64::{engine::general_purpose::STANDARD, Engine as _};

#[cfg(any(feature = "solana", feature = "evm"))]
use riglr_core::signer::{SignerError, TransactionSigner};

#[cfg(feature = "solana")]
use riglr_core::config::SolanaNetworkConfig;

#[cfg(feature = "evm")]
use riglr_core::config::EvmNetworkConfig;

#[cfg(any(feature = "solana", feature = "evm"))]
use riglr_core::signer::EvmClient;

#[cfg(any(feature = "solana", feature = "evm"))]
use std::sync::Arc;

#[cfg(any(feature = "solana", feature = "evm"))]
use tracing::{debug, error, info};

#[cfg(feature = "solana")]
use solana_client::rpc_client::RpcClient;
#[cfg(feature = "solana")]
use solana_sdk::transaction::Transaction;

#[cfg(feature = "evm")]
use alloy::rpc::types::TransactionRequest;

#[cfg(any(feature = "solana", feature = "evm"))]
use super::types::{PrivyRpcRequest, PrivyRpcResponse};

#[cfg(feature = "solana")]
use super::types::PrivySolanaTransactionParams;

#[cfg(feature = "evm")]
use super::types::PrivyEvmTransactionParams;

/// Privy Solana signer implementation
#[cfg(feature = "solana")]
#[derive(Clone)]
pub struct PrivySolanaSigner {
    client: reqwest::Client,
    address: String,
    rpc: Arc<RpcClient>,
    network: SolanaNetworkConfig,
}

#[cfg(feature = "solana")]
impl std::fmt::Debug for PrivySolanaSigner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PrivySolanaSigner")
            .field("address", &self.address)
            .field("network", &self.network.name)
            .finish()
    }
}

#[cfg(feature = "solana")]
impl PrivySolanaSigner {
    pub fn new(client: reqwest::Client, address: String, network: SolanaNetworkConfig) -> Self {
        let rpc = Arc::new(RpcClient::new(network.rpc_url.clone()));
        Self {
            client,
            address,
            rpc,
            network,
        }
    }
}

#[cfg(feature = "solana")]
#[async_trait]
impl TransactionSigner for PrivySolanaSigner {
    fn address(&self) -> Option<String> {
        Some(self.address.clone())
    }

    fn pubkey(&self) -> Option<String> {
        Some(self.address.clone())
    }

    async fn sign_and_send_solana_transaction(
        &self,
        tx: &mut Transaction,
    ) -> Result<String, SignerError> {
        info!("Signing and sending Solana transaction via Privy");

        // Serialize and encode transaction
        let tx_bytes = bincode::serialize(tx)
            .map_err(|e| SignerError::Signing(format!("Failed to serialize transaction: {}", e)))?;
        let tx_base64 = STANDARD.encode(&tx_bytes);

        // Determine CAIP-2 identifier based on network
        let caip2 = match self.network.name.as_str() {
            "mainnet" | "mainnet-beta" => "solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp",
            "devnet" => "solana:EtWTRABZaYq6iMfeYKouRu166VU2xqa1",
            "testnet" => "solana:4uhcVJyU9pJkvQyS88uRDiswHXSCkY3z",
            _ => "solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp", // Default to mainnet
        };

        let params = PrivySolanaTransactionParams {
            transaction: tx_base64,
            encoding: "base64".to_string(),
        };

        let request = PrivyRpcRequest {
            address: self.address.clone(),
            chain_type: "solana".to_string(),
            method: "signAndSendTransaction".to_string(),
            caip2: caip2.to_string(),
            params: serde_json::to_value(params)
                .map_err(|e| SignerError::Signing(format!("Failed to serialize params: {}", e)))?,
        };

        debug!("Sending RPC request to Privy");

        let response = self
            .client
            .post("https://api.privy.io/v1/wallets/rpc")
            .json(&request)
            .send()
            .await
            .map_err(|e| SignerError::TransactionFailed(format!("Privy request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            error!("Privy RPC error: {} - {}", status, body);
            return Err(SignerError::TransactionFailed(format!(
                "Privy error: {} - {}",
                status, body
            )));
        }

        let rpc_response: PrivyRpcResponse = response
            .json()
            .await
            .map_err(|e| SignerError::TransactionFailed(format!("Invalid response: {}", e)))?;

        // Extract transaction hash from response
        let hash = rpc_response
            .data
            .get("hash")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                SignerError::TransactionFailed("No transaction hash in response".to_string())
            })?;

        info!("Transaction sent successfully: {}", hash);
        Ok(hash.to_string())
    }

    #[cfg(feature = "evm")]
    async fn sign_and_send_evm_transaction(
        &self,
        _tx: TransactionRequest,
    ) -> Result<String, SignerError> {
        Err(SignerError::UnsupportedOperation(
            "EVM not supported by Solana signer".to_string(),
        ))
    }

    #[cfg(not(feature = "evm"))]
    async fn sign_and_send_evm_transaction(
        &self,
        _tx: alloy::rpc::types::TransactionRequest,
    ) -> Result<String, SignerError> {
        Err(SignerError::UnsupportedOperation(
            "EVM support not compiled".to_string(),
        ))
    }

    fn solana_client(&self) -> Option<Arc<solana_client::rpc_client::RpcClient>> {
        Some(self.rpc.clone())
    }

    fn evm_client(&self) -> Result<Arc<dyn EvmClient>, SignerError> {
        Err(SignerError::UnsupportedOperation(
            "EVM client not available for Solana signer".to_string(),
        ))
    }
}

/// Privy EVM signer implementation
#[cfg(feature = "evm")]
#[derive(Debug, Clone)]
pub struct PrivyEvmSigner {
    client: reqwest::Client,
    address: String,
    wallet_id: String,
    network: EvmNetworkConfig,
}

#[cfg(feature = "evm")]
impl PrivyEvmSigner {
    pub fn new(
        client: reqwest::Client,
        address: String,
        wallet_id: String,
        network: EvmNetworkConfig,
    ) -> Self {
        Self {
            client,
            address,
            wallet_id,
            network,
        }
    }
}

#[cfg(feature = "evm")]
#[async_trait]
impl TransactionSigner for PrivyEvmSigner {
    fn address(&self) -> Option<String> {
        Some(self.address.clone())
    }

    #[cfg(feature = "solana")]
    async fn sign_and_send_solana_transaction(
        &self,
        _tx: &mut Transaction,
    ) -> Result<String, SignerError> {
        Err(SignerError::UnsupportedOperation(
            "Solana not supported by EVM signer".to_string(),
        ))
    }

    #[cfg(not(feature = "solana"))]
    async fn sign_and_send_solana_transaction(
        &self,
        _tx: &mut solana_sdk::transaction::Transaction,
    ) -> Result<String, SignerError> {
        Err(SignerError::UnsupportedOperation(
            "Solana support not compiled".to_string(),
        ))
    }

    async fn sign_and_send_evm_transaction(
        &self,
        tx: TransactionRequest,
    ) -> Result<String, SignerError> {
        info!("Signing and sending EVM transaction via Privy");

        // Convert transaction to Privy format
        let from = tx
            .from
            .map(|a| format!("0x{:x}", a))
            .unwrap_or_else(|| self.address.clone());

        let to = if let Some(to) = tx.to {
            // Handle TxKind enum
            format!("0x{:?}", to) // This will need proper handling based on actual TxKind structure
        } else {
            // Contract creation
            "0x0000000000000000000000000000000000000000".to_string()
        };

        let value = tx.value.map(|v| format!("0x{:x}", v));
        let data = tx
            .input
            .data
            .as_ref()
            .map(|d| format!("0x{}", hex::encode(d)));
        let gas_limit = tx.gas.map(|g| format!("0x{:x}", g));

        let params = PrivyEvmTransactionParams {
            from,
            to,
            value,
            data,
            gas_limit,
            gas_price: None, // Let Privy handle gas pricing
            tx_type: None,
        };

        let request = PrivyRpcRequest {
            address: self.address.clone(),
            chain_type: "ethereum".to_string(),
            method: "eth_sendTransaction".to_string(),
            caip2: self.network.caip2(),
            params: serde_json::to_value(vec![params])
                .map_err(|e| SignerError::Signing(format!("Failed to serialize params: {}", e)))?,
        };

        debug!("Sending RPC request to Privy for wallet {}", self.wallet_id);

        let response = self
            .client
            .post(format!(
                "https://api.privy.io/v1/wallets/{}/rpc",
                self.wallet_id
            ))
            .json(&request)
            .send()
            .await
            .map_err(|e| SignerError::TransactionFailed(format!("Privy request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            error!("Privy RPC error: {} - {}", status, body);
            return Err(SignerError::TransactionFailed(format!(
                "Privy error: {} - {}",
                status, body
            )));
        }

        let rpc_response: PrivyRpcResponse = response
            .json()
            .await
            .map_err(|e| SignerError::TransactionFailed(format!("Invalid response: {}", e)))?;

        // Extract transaction hash from response
        let hash = rpc_response
            .data
            .get("hash")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                SignerError::TransactionFailed("No transaction hash in response".to_string())
            })?;

        info!("Transaction sent successfully: {}", hash);
        Ok(hash.to_string())
    }

    fn solana_client(&self) -> Option<Arc<solana_client::rpc_client::RpcClient>> {
        // EVM signer doesn't need Solana client
        None
    }

    fn evm_client(&self) -> Result<Arc<dyn EvmClient>, SignerError> {
        // Privy handles RPC internally, so we don't provide direct client access
        Err(SignerError::UnsupportedOperation(
            "Direct EVM client not available for Privy signer".to_string(),
        ))
    }
}

// Stub implementations when features are disabled
#[cfg(not(feature = "solana"))]
pub struct PrivySolanaSigner;

#[cfg(not(feature = "evm"))]
pub struct PrivyEvmSigner;
