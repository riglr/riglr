use async_trait::async_trait;
use std::sync::Arc;
use super::error::SignerError;

/// A trait for transaction signing across multiple blockchain networks.
/// This trait provides a unified interface for signing transactions on different chains
/// while maintaining secure context isolation.
#[async_trait]
pub trait TransactionSigner: Send + Sync + std::fmt::Debug {
    /// User locale for localized responses
    fn locale(&self) -> String {
        "en".to_string()
    }
    
    /// Optional user identifier for multi-tenant scenarios
    fn user_id(&self) -> Option<String> {
        None
    }
    
    /// Solana wallet address (base58 encoded)
    fn address(&self) -> Option<String> {
        None
    }
    
    /// Solana public key string (base58 encoded)
    fn pubkey(&self) -> Option<String> {
        None
    }
    
    /// Sign and send a Solana transaction
    /// The transaction should be properly constructed with recent blockhash
    async fn sign_and_send_solana_transaction(
        &self,
        tx: &mut solana_sdk::transaction::Transaction,
    ) -> Result<String, SignerError>;
    
    /// Sign and send an EVM transaction
    /// The transaction request should include all necessary fields (to, value, gas, etc.)
    async fn sign_and_send_evm_transaction(
        &self,
        tx: alloy::rpc::types::TransactionRequest,
    ) -> Result<String, SignerError>;
    
    /// Get Solana RPC client (derived from signer configuration)
    /// This client should be configured with the appropriate RPC endpoint
    fn solana_client(&self) -> Arc<solana_client::rpc_client::RpcClient>;
    
    /// Get EVM RPC client (derived from signer configuration)  
    /// This client should be configured with the appropriate RPC endpoint and chain ID
    fn evm_client(&self) -> Result<std::sync::Arc<dyn std::any::Any + Send + Sync>, SignerError>;
}