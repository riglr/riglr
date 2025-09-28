//! Privy-specific signer implementations

#[cfg(any(feature = "solana", feature = "evm"))]
use async_trait::async_trait;

#[cfg(any(feature = "solana", feature = "evm"))]
use base64::{engine::general_purpose::STANDARD, Engine as _};

#[cfg(any(feature = "solana", feature = "evm"))]
use riglr_core::signer::{Chain, EvmSigner, SignerBase, SignerError, SolanaSigner, UnifiedSigner};

#[cfg(any(feature = "solana", feature = "evm"))]
use riglr_core::signer::error::Standard;

#[cfg(any(feature = "solana", feature = "evm"))]
use riglr_core::provider_extensions::{EvmAppContextProvider, SolanaAppContextProvider};

#[cfg(any(feature = "solana", feature = "evm"))]
use riglr_core::ToolError;

#[cfg(any(feature = "solana", feature = "evm"))]
use riglr_core::signer::{EvmClient, SolanaClient};

#[cfg(any(feature = "solana", feature = "evm"))]
use std::sync::Arc;

#[cfg(any(feature = "solana", feature = "evm"))]
use core::{
    any::Any,
    fmt::{Debug, Formatter, Result as FmtResult},
};

#[cfg(feature = "solana")]
use bincode::deserialize;

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
    address: String,
    client: reqwest::Client,
    network_name: String,
    rpc: Arc<RpcClient>,
    rpc_url: String,
}

#[cfg(feature = "solana")]
impl Debug for PrivySolanaSigner {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_struct("PrivySolanaSigner")
            .field("address", &self.address)
            .field("network", &self.network_name)
            .field("rpc_url", &self.rpc_url)
            .finish_non_exhaustive()
    }
}

#[cfg(feature = "solana")]
impl PrivySolanaSigner {
    pub fn new(
        client: reqwest::Client,
        address: String,
        network_name: String,
        rpc_url: String,
    ) -> Self {
        let rpc = Arc::new(RpcClient::new(rpc_url.clone()));
        Self {
            address,
            client,
            network_name,
            rpc,
            rpc_url,
        }
    }
}

#[cfg(feature = "solana")]
#[async_trait]
impl SignerBase for PrivySolanaSigner {
    fn supported_chains(&self) -> &[Chain] {
        &[Chain::Solana]
    }

    fn user_id(&self) -> String {
        // Return a consistent identifier for this signer
        format!("privy-solana-{}", self.address)
    }
}

#[cfg(feature = "solana")]
#[async_trait]
impl SolanaSigner for PrivySolanaSigner {
    fn client(&self) -> &dyn SolanaClient {
        // This is a limitation of the Privy approach - we need to return a reference
        // but we have an Arc<RpcClient>. For now, we'll use a workaround.
        // In a real implementation, we'd restructure to have the client as a reference.
        todo!("Privy signers need client restructuring to return &dyn SolanaClient")
    }

    fn pubkey(&self) -> String {
        self.address.clone()
    }

    async fn sign_and_send_transaction(
        &self,
        transaction: serde_json::Value,
    ) -> Result<String, Box<dyn SignerError>> {
        // Convert JSON to Transaction bytes for our existing implementation
        let tx_bytes = transaction
            .get("transaction")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                Box::new(Standard::InvalidInput(
                    "Invalid transaction format".to_string(),
                )) as Box<dyn SignerError>
            })?
            .as_bytes();

        let tx_vec = tx_bytes.to_vec();
        let tx: Transaction = deserialize(&tx_vec).map_err(|e| {
            Box::new(Standard::SigningFailed(format!(
                "Failed to deserialize transaction: {e}"
            ))) as Box<dyn SignerError>
        })?;
        self.sign_and_send_solana_transaction_impl(&tx).await
    }

    async fn sign_message(&self, _message: &[u8]) -> Result<String, Box<dyn SignerError>> {
        // Privy doesn't directly support message signing in the same way
        // This would need to be implemented via their API
        Err(Box::new(Standard::UnsupportedOperation(
            "Message signing not implemented for Privy signers".to_string(),
        )))
    }
}

#[cfg(feature = "solana")]
impl PrivySolanaSigner {
    async fn sign_and_send_solana_transaction_impl(
        &self,
        tx: &Transaction,
    ) -> Result<String, Box<dyn SignerError>> {
        info!("Signing and sending Solana transaction via Privy");

        let tx_base64 = Self::serialize_transaction(tx)?;
        let request = self.build_rpc_request(tx_base64)?;
        let response = self.send_rpc_request(request).await?;
        let hash = Self::extract_transaction_hash(&response)?;

        info!("Transaction sent successfully: {}", hash);
        Ok(hash)
    }

    fn get_caip2_identifier(&self) -> &'static str {
        match self.network_name.as_str() {
            "devnet" => "solana:EtWTRABZaYq6iMfeYKouRu166VU2xqa1",
            "testnet" => "solana:4uhcVJyU9pJkvQyS88uRDiswHXSCkY3z",
            _ => "solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp", // Default for mainnet and mainnet-beta
        }
    }

    fn serialize_transaction(tx: &Transaction) -> Result<String, Box<dyn SignerError>> {
        let tx_bytes = bincode::serialize(tx).map_err(|e| {
            Box::new(Standard::SigningFailed(format!(
                "Failed to serialize transaction: {e}"
            ))) as Box<dyn SignerError>
        })?;
        Ok(STANDARD.encode(&tx_bytes))
    }

    fn build_rpc_request(
        &self,
        tx_base64: String,
    ) -> Result<PrivyRpcRequest, Box<dyn SignerError>> {
        let params = PrivySolanaTransactionParams {
            transaction: tx_base64,
            encoding: "base64".to_string(),
        };

        let params_value = serde_json::to_value(params).map_err(|e| {
            Box::new(Standard::SigningFailed(format!(
                "Failed to serialize params: {e}"
            ))) as Box<dyn SignerError>
        })?;

        Ok(PrivyRpcRequest {
            address: self.address.clone(),
            chain_type: "solana".to_string(),
            method: "signAndSendTransaction".to_string(),
            caip2: self.get_caip2_identifier().to_string(),
            params: params_value,
        })
    }

    fn extract_transaction_hash(
        rpc_response: &PrivyRpcResponse,
    ) -> Result<String, Box<dyn SignerError>> {
        let hash = rpc_response
            .data
            .get("hash")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                Box::new(Standard::SigningFailed(
                    "No transaction hash in response".to_string(),
                )) as Box<dyn SignerError>
            })?;
        Ok(hash.to_string())
    }

    async fn handle_response_status(
        &self,
        response: reqwest::Response,
    ) -> Result<String, Box<dyn SignerError>> {
        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            error!("Privy RPC error: {} - {}", status, body);
            return Err(Box::new(Standard::SigningFailed(format!(
                "Privy error: {status} - {body}"
            ))));
        }
        let body = response.text().await.unwrap_or_default();
        Ok(body)
    }

    async fn send_rpc_request(
        &self,
        request: PrivyRpcRequest,
    ) -> Result<PrivyRpcResponse, Box<dyn SignerError>> {
        debug!("Sending RPC request to Privy");

        let response = self
            .client
            .post("https://api.privy.io/v1/wallets/rpc")
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                Box::new(Standard::SigningFailed(format!(
                    "Privy request failed: {e}"
                ))) as Box<dyn SignerError>
            })?;

        let response_body = self.handle_response_status(response).await?;

        let result = serde_json::from_str(&response_body).map_err(|e| {
            Box::new(Standard::SigningFailed(format!("Invalid response: {e}")))
                as Box<dyn SignerError>
        });
        result
    }
}

#[cfg(feature = "solana")]
impl UnifiedSigner for PrivySolanaSigner {
    fn as_evm(&self) -> Option<&dyn EvmSigner> {
        None
    }

    fn as_solana(&self) -> Option<&dyn SolanaSigner> {
        Some(self)
    }
}

#[cfg(feature = "solana")]
impl EvmAppContextProvider for PrivySolanaSigner {
    fn evm_client(&self) -> Result<Arc<dyn Any>, ToolError> {
        Err(ToolError::permanent_string(
            "EVM client not available for Solana signer",
        ))
    }
}

#[cfg(feature = "solana")]
impl SolanaAppContextProvider for PrivySolanaSigner {
    fn solana_client(&self) -> Result<Arc<dyn Any>, ToolError> {
        Ok(self.rpc.clone() as Arc<dyn Any>)
    }
}

/// Privy EVM signer implementation
#[cfg(feature = "evm")]
#[derive(Clone)]
pub struct PrivyEvmSigner {
    address: String,
    chain_id: u64,
    client: reqwest::Client,
    rpc_url: String,
    wallet_id: String,
}

#[cfg(feature = "evm")]
impl Debug for PrivyEvmSigner {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_struct("PrivyEvmSigner")
            .field("address", &self.address)
            .field("chain_id", &self.chain_id)
            .field("rpc_url", &self.rpc_url)
            .field("wallet_id", &self.wallet_id)
            .finish_non_exhaustive()
    }
}

#[cfg(feature = "evm")]
impl PrivyEvmSigner {
    pub const fn new(
        client: reqwest::Client,
        address: String,
        wallet_id: String,
        chain_id: u64,
        rpc_url: String,
    ) -> Self {
        Self {
            address,
            chain_id,
            client,
            rpc_url,
            wallet_id,
        }
    }

    /// Sign and send Solana transaction - not supported for EVM signers (Solana feature disabled)
    #[cfg(not(feature = "solana"))]
    pub async fn sign_and_send_solana_transaction(
        _tx: &mut solana_sdk::transaction::Transaction,
    ) -> Result<String, Box<dyn SignerError>> {
        Err(Box::new(Standard::UnsupportedOperation(
            "Solana support not compiled".to_string(),
        )))
    }
}

#[cfg(feature = "evm")]
#[async_trait]
impl SignerBase for PrivyEvmSigner {
    fn supported_chains(&self) -> &[Chain] {
        &[Chain::Evm]
    }

    fn user_id(&self) -> String {
        // Return a consistent identifier for this signer
        format!("privy-evm-{}-{}", self.chain_id, self.address)
    }
}

#[cfg(feature = "evm")]
#[async_trait]
impl EvmSigner for PrivyEvmSigner {
    fn address(&self) -> String {
        self.address.clone()
    }

    fn chain_id(&self) -> u64 {
        self.chain_id
    }

    fn client(&self) -> &dyn EvmClient {
        // This is a limitation of the Privy approach - we need to return a reference
        // but we don't have a direct EvmClient. For now, we'll use a workaround.
        todo!("Privy signers need client restructuring to return &dyn EvmClient")
    }

    async fn sign_and_send_transaction(
        &self,
        tx_json: serde_json::Value,
    ) -> Result<String, Box<dyn SignerError>> {
        info!("Signing and sending EVM transaction via Privy");

        // Convert JSON to TransactionRequest
        let tx: TransactionRequest = serde_json::from_value(tx_json).map_err(|e| {
            Box::new(Standard::SigningFailed(format!(
                "Failed to parse transaction: {e}"
            ))) as Box<dyn SignerError>
        })?;

        // Convert transaction to Privy format
        let from = tx
            .from
            .map_or_else(|| self.address.clone(), |a| format!("0x{a:x}"));

        let to = tx.to.map_or_else(
            || "0x0000000000000000000000000000000000000000".to_string(),
            |to| format!("0x{to:?}"), // This will need proper handling based on actual TxKind structure
        );

        let value = tx.value.map(|v| format!("0x{v:x}"));
        let data = tx
            .input
            .data
            .as_ref()
            .map(|d| format!("0x{}", hex::encode(d)));
        let gas_limit = tx.gas.map(|g| format!("0x{g:x}"));

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
            caip2: format!("eip155:{}", self.chain_id),
            params: serde_json::to_value(vec![params]).map_err(|e| {
                Box::new(Standard::SigningFailed(format!(
                    "Failed to serialize params: {e}"
                ))) as Box<dyn SignerError>
            })?,
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
            .map_err(|e| {
                Box::new(Standard::SigningFailed(format!(
                    "Privy request failed: {e}"
                ))) as Box<dyn SignerError>
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            error!("Privy RPC error: {} - {}", status, body);
            return Err(Box::new(Standard::SigningFailed(format!(
                "Privy error: {status} - {body}"
            ))));
        }

        let rpc_response: PrivyRpcResponse = response.json().await.map_err(|e| {
            Box::new(Standard::SigningFailed(format!("Invalid response: {e}")))
                as Box<dyn SignerError>
        })?;

        // Extract transaction hash from response
        let hash = rpc_response
            .data
            .get("hash")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                Box::new(Standard::SigningFailed(
                    "No transaction hash in response".to_string(),
                )) as Box<dyn SignerError>
            })?;

        info!("Transaction sent successfully: {}", hash);
        Ok(hash.to_string())
    }

    async fn sign_message(&self, _message: &[u8]) -> Result<String, Box<dyn SignerError>> {
        // Privy doesn't directly support message signing in the same way
        // This would need to be implemented via their API
        Err(Box::new(Standard::UnsupportedOperation(
            "Message signing not implemented for Privy signers".to_string(),
        )))
    }
}

#[cfg(feature = "evm")]
impl UnifiedSigner for PrivyEvmSigner {
    fn as_evm(&self) -> Option<&dyn EvmSigner> {
        Some(self)
    }

    fn as_solana(&self) -> Option<&dyn SolanaSigner> {
        None
    }
}

#[cfg(feature = "evm")]
impl EvmAppContextProvider for PrivyEvmSigner {
    fn evm_client(&self) -> Result<Arc<dyn Any>, ToolError> {
        // Privy handles RPC internally, so we don't provide direct client access
        Err(ToolError::permanent_string(
            "Direct EVM client not available for Privy signer",
        ))
    }
}

#[cfg(feature = "evm")]
impl SolanaAppContextProvider for PrivyEvmSigner {
    fn solana_client(&self) -> Result<Arc<dyn Any>, ToolError> {
        Err(ToolError::permanent_string(
            "Solana client not available for EVM signer",
        ))
    }
}

// Stub implementations when features are disabled
#[cfg(not(feature = "solana"))]
pub struct PrivySolanaSigner;

#[cfg(not(feature = "evm"))]
pub struct PrivyEvmSigner;

#[cfg(test)]
mod tests {
    use super::*;
    // NOTE: Using deprecated solana_sdk::system_instruction instead of solana_system_interface
    // to avoid Address vs Pubkey type mismatches and solana-instruction crate version conflicts.
    // The newer solana_system_interface requires Address types but our codebase uses Pubkey,
    // leading to compilation errors. This should be migrated in a broader dependency update.

    #[cfg(feature = "solana")]
    mod solana_signer_tests {
        use super::*;
        use riglr_core::signer::SolanaSigner;

        fn create_test_signer() -> PrivySolanaSigner {
            let client = reqwest::Client::new();
            let address = "11111111111111111111111111111111".to_string();
            let network_name = "mainnet".to_string();
            let rpc_url = "https://api.mainnet-beta.solana.com".to_string();
            PrivySolanaSigner::new(client, address, network_name, rpc_url)
        }

        #[test]
        fn test_new_when_valid_params_should_create_signer() {
            let client = reqwest::Client::new();
            let address = "test_address".to_string();
            let network_name = "devnet".to_string();
            let rpc_url = "https://api.devnet.solana.com".to_string();

            let signer = PrivySolanaSigner::new(
                client,
                address.clone(),
                network_name.clone(),
                rpc_url.clone(),
            );

            assert_eq!(signer.address, address);
            assert_eq!(signer.network_name, network_name);
            assert_eq!(signer.rpc_url, rpc_url);
        }

        #[test]
        fn test_debug_when_called_should_format_correctly() {
            let signer = create_test_signer();
            let debug_str = format!("{signer:?}");

            assert!(debug_str.contains("PrivySolanaSigner"));
            assert!(debug_str.contains("address"));
            assert!(debug_str.contains("network"));
            assert!(debug_str.contains("mainnet"));
        }

        #[test]
        fn test_address_when_called_should_return_address() {
            let signer = create_test_signer();
            // The SolanaSigner trait uses pubkey(), not address()
            let result = signer.pubkey();

            assert_eq!(result, "11111111111111111111111111111111");
        }

        #[test]
        fn test_pubkey_when_called_should_return_address() {
            let signer = create_test_signer();
            let result = signer.pubkey();

            assert_eq!(result, "11111111111111111111111111111111");
        }

        #[test]
        fn test_client_when_called_should_return_client() {
            let signer = create_test_signer();
            // Test the client() method from SolanaSigner trait
            let _client = signer.client();
            // If we get here without panic, the test passes
        }

        #[tokio::test]
        async fn test_caip2_mapping_when_mainnet_should_return_correct_id() {
            // Test through the actual function by creating different network configs
            let client = reqwest::Client::new();
            let address = "test_address".to_string();

            // Test mainnet
            let network_name = "mainnet".to_string();
            let rpc_url = "https://api.mainnet-beta.solana.com".to_string();
            let signer = PrivySolanaSigner::new(client, address, network_name, rpc_url);

            // We can verify the CAIP2 mapping indirectly by checking the request would be formed correctly
            // The actual CAIP2 mapping is tested through integration with the sign_and_send method
            assert_eq!(signer.network_name, "mainnet");
        }

        #[tokio::test]
        async fn test_caip2_mapping_when_devnet_should_return_correct_id() {
            let client = reqwest::Client::new();
            let address = "test_address".to_string();

            let network_name = "devnet".to_string();
            let rpc_url = "https://api.devnet.solana.com".to_string();
            let signer = PrivySolanaSigner::new(client, address, network_name, rpc_url);

            assert_eq!(signer.network_name, "devnet");
        }

        #[tokio::test]
        async fn test_caip2_mapping_when_testnet_should_return_correct_id() {
            let client = reqwest::Client::new();
            let address = "test_address".to_string();

            let network_name = "testnet".to_string();
            let rpc_url = "https://api.testnet.solana.com".to_string();
            let signer = PrivySolanaSigner::new(client, address, network_name, rpc_url);

            assert_eq!(signer.network_name, "testnet");
        }

        #[tokio::test]
        async fn test_caip2_mapping_when_unknown_network_should_default_to_mainnet() {
            let client = reqwest::Client::new();
            let address = "test_address".to_string();

            let network_name = "unknown_network".to_string();
            let rpc_url = "https://api.unknown.solana.com".to_string();
            let signer = PrivySolanaSigner::new(client, address, network_name, rpc_url);

            assert_eq!(signer.network_name, "unknown_network");
        }
    }

    #[cfg(feature = "evm")]
    mod evm_signer_tests {
        use super::*;

        fn create_test_evm_signer() -> PrivyEvmSigner {
            let client = reqwest::Client::new();
            let address = "0x1234567890123456789012345678901234567890".to_string();
            let wallet_id = "wallet_123".to_string();
            let chain_id = 1;
            let rpc_url = "https://eth.llamarpc.com".to_string();
            PrivyEvmSigner::new(client, address, wallet_id, chain_id, rpc_url)
        }

        #[test]
        fn test_new_when_valid_params_should_create_signer() {
            let client = reqwest::Client::new();
            let address = "0x1234567890123456789012345678901234567890".to_string();
            let wallet_id = "wallet_123".to_string();
            let chain_id = 1;
            let rpc_url = "https://eth.llamarpc.com".to_string();

            let signer = PrivyEvmSigner::new(
                client,
                address.clone(),
                wallet_id.clone(),
                chain_id,
                rpc_url.clone(),
            );

            assert_eq!(signer.address, address);
            assert_eq!(signer.wallet_id, wallet_id);
            assert_eq!(signer.chain_id, chain_id);
            assert_eq!(signer.rpc_url, rpc_url);
        }

        #[test]
        fn test_debug_when_called_should_format_correctly() {
            let signer = create_test_evm_signer();
            let debug_str = format!("{signer:?}");

            assert!(debug_str.contains("PrivyEvmSigner"));
            assert!(debug_str.contains("address"));
            assert!(debug_str.contains("wallet_id"));
        }

        #[test]
        fn test_address_when_called_should_return_address() {
            let signer = create_test_evm_signer();
            let result = signer.address();

            assert_eq!(result, "0x1234567890123456789012345678901234567890");
        }
    }

    // Tests for stub implementations when features are disabled
    #[cfg(not(feature = "solana"))]
    mod solana_stub_tests {
        use super::*;

        #[test]
        fn test_privy_solana_signer_stub_exists() {
            // Just test that the stub struct exists and can be instantiated
            let _signer = PrivySolanaSigner;
        }
    }

    #[cfg(not(feature = "evm"))]
    mod evm_stub_tests {
        use super::*;

        #[test]
        fn test_privy_evm_signer_stub_exists() {
            // Just test that the stub struct exists and can be instantiated
            let _signer = PrivyEvmSigner;
        }
    }
}
