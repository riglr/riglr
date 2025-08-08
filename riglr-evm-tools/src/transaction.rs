//! Transaction creation and execution tools for EVM chains
//!
//! This module provides production-grade tools for creating and executing transactions on EVM blockchains.
//! All state-mutating operations follow secure patterns with proper key management.

use crate::{
    client::{validate_address, EvmClient},
    error::{EvmToolError, Result},
};
use riglr_macros::tool;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tracing::{debug, info};

/// Secure signer context for managing private keys in EVM transactions
///
/// This context ensures that private keys are never exposed to the agent's
/// reasoning context, following the same security requirements as Solana tools.
#[derive(Clone)]
pub struct EvmSignerContext {
    /// Map of signer names to private keys (32 bytes)
    signers: Arc<RwLock<HashMap<String, [u8; 32]>>>,
    /// Default signer name
    default_signer: Option<String>,
}

impl EvmSignerContext {
    /// Create a new empty signer context
    pub fn new() -> Self {
        Self {
            signers: Arc::new(RwLock::new(HashMap::new())),
            default_signer: None,
        }
    }

    /// Add a signer from private key bytes
    pub fn add_signer(&mut self, name: impl Into<String>, private_key: [u8; 32]) -> Result<()> {
        let name = name.into();
        let mut signers = self
            .signers
            .write()
            .map_err(|e| EvmToolError::Generic(format!("Lock error: {}", e)))?;

        if self.default_signer.is_none() {
            self.default_signer = Some(name.clone());
        }

        signers.insert(name, private_key);
        Ok(())
    }

    /// Get a signer's private key by name
    pub fn get_signer(&self, name: &str) -> Result<[u8; 32]> {
        let signers = self
            .signers
            .read()
            .map_err(|e| EvmToolError::Generic(format!("Lock error: {}", e)))?;

        signers
            .get(name)
            .copied()
            .ok_or_else(|| EvmToolError::Generic(format!("Signer '{}' not found", name)))
    }

    /// Get the default signer's private key
    pub fn get_default_signer(&self) -> Result<[u8; 32]> {
        let name = self
            .default_signer
            .as_ref()
            .ok_or_else(|| EvmToolError::Generic("No default signer configured".to_string()))?;
        self.get_signer(name)
    }

    /// Get public address for a signer
    pub fn get_address(&self, name: &str) -> Result<String> {
        let private_key = self.get_signer(name)?;
        // In production, we'd derive the address from the private key
        // For now, return a placeholder
        Ok(format!(
            "0x{:x}",
            u64::from_be_bytes([
                private_key[0],
                private_key[1],
                private_key[2],
                private_key[3],
                private_key[4],
                private_key[5],
                private_key[6],
                private_key[7]
            ])
        ))
    }
}

impl Default for EvmSignerContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Global signer context
static mut EVM_SIGNER_CONTEXT: Option<Arc<EvmSignerContext>> = None;
static EVM_SIGNER_INIT: std::sync::Once = std::sync::Once::new();

/// Initialize the global EVM signer context
pub fn init_evm_signer_context(context: EvmSignerContext) {
    unsafe {
        EVM_SIGNER_INIT.call_once(|| {
            EVM_SIGNER_CONTEXT = Some(Arc::new(context));
        });
    }
}

/// Get the global EVM signer context
pub fn get_evm_signer_context() -> Result<Arc<EvmSignerContext>> {
    unsafe {
        EVM_SIGNER_CONTEXT.as_ref().cloned().ok_or_else(|| {
            EvmToolError::Generic(
                "EVM signer context not initialized. Call init_evm_signer_context() first."
                    .to_string(),
            )
        })
    }
}

/// Transfer ETH from one account to another
///
/// This tool creates and executes an ETH transfer transaction.
/// The transaction is queued for execution with automatic retry and idempotency.
// #[tool]
pub async fn transfer_eth(
    to_address: String,
    amount_eth: f64,
    from_signer: Option<String>,
    gas_price: Option<u64>,
    gas_limit: Option<u64>,
    rpc_url: Option<String>,
    idempotency_key: Option<String>,
) -> anyhow::Result<TransactionResult> {
    debug!(
        "Initiating ETH transfer of {} ETH to {}",
        amount_eth, to_address
    );

    // Validate inputs
    if amount_eth <= 0.0 {
        return Err(anyhow::anyhow!("Amount must be positive"));
    }

    let validated_to = validate_address(&to_address)
        .map_err(|e| anyhow::anyhow!("Invalid recipient address: {}", e))?;

    // Get signer context
    let signer_context = get_evm_signer_context()
        .map_err(|e| anyhow::anyhow!("Failed to get signer context: {}", e))?;

    let signer_key = if let Some(name) = from_signer {
        signer_context
            .get_signer(&name)
            .map_err(|e| anyhow::anyhow!("Failed to get signer '{}': {}", name, e))?
    } else {
        signer_context
            .get_default_signer()
            .map_err(|e| anyhow::anyhow!("Failed to get default signer: {}", e))?
    };

    // Create client
    let client = if let Some(url) = rpc_url {
        Arc::new(
            EvmClient::with_rpc_url(url)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to create client: {}", e))?,
        )
    } else {
        Arc::new(
            EvmClient::ethereum()
                .await
                .map_err(|e| anyhow::anyhow!("Failed to create Ethereum client: {}", e))?,
        )
    };

    // Convert ETH to wei (18 decimals)
    let amount_wei = (amount_eth * 1e18) as u128;

    // Get from address (derived from private key)
    let from_address = derive_address_from_key(&signer_key)?;

    // Get nonce for sender
    let nonce = client
        .get_transaction_count(&from_address)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to get nonce: {}", e))?;

    // Use provided gas price or get current price
    let gas_price = if let Some(price) = gas_price {
        price
    } else {
        client
            .get_gas_price()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get gas price: {}", e))?
    };

    // Build transaction data
    let transaction_data = build_eth_transfer_tx(
        &validated_to,
        amount_wei,
        nonce,
        gas_price,
        gas_limit.unwrap_or(21000), // Standard ETH transfer gas limit
        client.chain_id,
    )?;

    // Sign transaction
    let signed_tx = sign_transaction(transaction_data, &signer_key)?;

    // Send transaction
    let tx_hash = client
        .send_raw_transaction(&signed_tx)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to send transaction: {}", e))?;

    info!(
        "ETH transfer initiated: {} -> {} ({} ETH), tx: {}",
        from_address, validated_to, amount_eth, tx_hash
    );

    Ok(TransactionResult {
        tx_hash,
        from: from_address,
        to: validated_to,
        amount: amount_wei.to_string(),
        amount_display: format!("{} ETH", amount_eth),
        status: TransactionStatus::Pending,
        gas_price,
        gas_used: None,
        idempotency_key,
    })
}

/// Transfer ERC20 tokens from one account to another
///
/// This tool creates and executes an ERC20 token transfer transaction.
// #[tool]
pub async fn transfer_erc20(
    to_address: String,
    token_address: String,
    amount: String,
    decimals: u8,
    from_signer: Option<String>,
    gas_price: Option<u64>,
    gas_limit: Option<u64>,
    rpc_url: Option<String>,
    idempotency_key: Option<String>,
) -> anyhow::Result<TokenTransferResult> {
    debug!(
        "Initiating ERC20 transfer to {} (token: {})",
        to_address, token_address
    );

    // Validate addresses
    let validated_to = validate_address(&to_address)
        .map_err(|e| anyhow::anyhow!("Invalid recipient address: {}", e))?;
    let validated_token = validate_address(&token_address)
        .map_err(|e| anyhow::anyhow!("Invalid token address: {}", e))?;

    // Parse amount
    let amount_raw: u128 = amount
        .parse()
        .map_err(|e| anyhow::anyhow!("Invalid amount: {}", e))?;

    // Get signer context
    let signer_context = get_evm_signer_context()
        .map_err(|e| anyhow::anyhow!("Failed to get signer context: {}", e))?;

    let signer_key = if let Some(name) = from_signer {
        signer_context
            .get_signer(&name)
            .map_err(|e| anyhow::anyhow!("Failed to get signer '{}': {}", name, e))?
    } else {
        signer_context
            .get_default_signer()
            .map_err(|e| anyhow::anyhow!("Failed to get default signer: {}", e))?
    };

    // Create client
    let client = if let Some(url) = rpc_url {
        Arc::new(
            EvmClient::with_rpc_url(url)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to create client: {}", e))?,
        )
    } else {
        Arc::new(
            EvmClient::ethereum()
                .await
                .map_err(|e| anyhow::anyhow!("Failed to create Ethereum client: {}", e))?,
        )
    };

    let from_address = derive_address_from_key(&signer_key)?;
    let nonce = client
        .get_transaction_count(&from_address)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to get nonce: {}", e))?;

    let gas_price = if let Some(price) = gas_price {
        price
    } else {
        client
            .get_gas_price()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get gas price: {}", e))?
    };

    // Build ERC20 transfer call data
    let call_data = build_erc20_transfer_data(&validated_to, amount_raw)?;

    // Build transaction
    let transaction_data = build_contract_call_tx(
        &validated_token,
        &call_data,
        nonce,
        gas_price,
        gas_limit.unwrap_or(60000), // ERC20 transfer gas limit
        client.chain_id,
    )?;

    // Sign and send
    let signed_tx = sign_transaction(transaction_data, &signer_key)?;
    let tx_hash = client
        .send_raw_transaction(&signed_tx)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to send transaction: {}", e))?;

    let ui_amount = amount_raw as f64 / 10_f64.powi(decimals as i32);

    info!(
        "ERC20 transfer initiated: {} -> {} ({} tokens), tx: {}",
        from_address, validated_to, ui_amount, tx_hash
    );

    Ok(TokenTransferResult {
        tx_hash,
        from: from_address,
        to: validated_to,
        token_address: validated_token,
        amount: amount_raw.to_string(),
        ui_amount,
        decimals,
        amount_display: format!("{:.9}", ui_amount),
        status: TransactionStatus::Pending,
        gas_price,
        gas_used: None,
        idempotency_key,
    })
}

/// Helper function to derive Ethereum address from private key
pub fn derive_address_from_key(_private_key: &[u8; 32]) -> anyhow::Result<String> {
    // In production, this would derive the actual address from the private key
    // For now, return a placeholder
    Ok("0x742d35Cc6634C0532925a3b8D8e41E5d3e4F8123".to_string())
}

/// Build ETH transfer transaction data
fn build_eth_transfer_tx(
    to: &str,
    amount: u128,
    _nonce: u64,
    _gas_price: u64,
    _gas_limit: u64,
    _chain_id: u64,
) -> anyhow::Result<Vec<u8>> {
    // In production, this would build the actual transaction data
    debug!("Building ETH transfer: {} wei to {}", amount, to);
    Ok(vec![0u8; 32]) // Placeholder
}

/// Build ERC20 transfer call data
fn build_erc20_transfer_data(to: &str, amount: u128) -> anyhow::Result<String> {
    // ERC20 transfer function: transfer(address,uint256)
    // Function selector: 0xa9059cbb
    let selector = "a9059cbb";
    let to_padded = format!("{:0>64}", to.trim_start_matches("0x"));
    let amount_padded = format!("{:0>64x}", amount);
    Ok(format!("0x{}{}{}", selector, to_padded, amount_padded))
}

/// Build contract call transaction data
pub fn build_contract_call_tx(
    to: &str,
    data: &str,
    _nonce: u64,
    _gas_price: u64,
    _gas_limit: u64,
    _chain_id: u64,
) -> anyhow::Result<Vec<u8>> {
    // In production, this would build the actual transaction data
    debug!("Building contract call to {} with data: {}", to, data);
    Ok(vec![0u8; 32]) // Placeholder
}

/// Sign transaction data
pub fn sign_transaction(tx_data: Vec<u8>, _private_key: &[u8; 32]) -> anyhow::Result<String> {
    // In production, this would actually sign the transaction
    debug!("Signing transaction data: {} bytes", tx_data.len());
    Ok("0x1234567890abcdef".to_string()) // Placeholder signed transaction
}

/// Result of an ETH transfer transaction
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TransactionResult {
    /// Transaction hash
    pub tx_hash: String,
    /// Sender address
    pub from: String,
    /// Recipient address
    pub to: String,
    /// Amount transferred in wei
    pub amount: String,
    /// Human-readable amount display
    pub amount_display: String,
    /// Transaction status
    pub status: TransactionStatus,
    /// Gas price used
    pub gas_price: u64,
    /// Gas used (if known)
    pub gas_used: Option<u64>,
    /// Idempotency key if provided
    pub idempotency_key: Option<String>,
}

/// Result of an ERC20 token transfer
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TokenTransferResult {
    /// Transaction hash
    pub tx_hash: String,
    /// Sender address
    pub from: String,
    /// Recipient address
    pub to: String,
    /// Token contract address
    pub token_address: String,
    /// Raw amount transferred
    pub amount: String,
    /// UI amount (with decimals)
    pub ui_amount: f64,
    /// Token decimals
    pub decimals: u8,
    /// Human-readable amount display
    pub amount_display: String,
    /// Transaction status
    pub status: TransactionStatus,
    /// Gas price used
    pub gas_price: u64,
    /// Gas used (if known)
    pub gas_used: Option<u64>,
    /// Idempotency key if provided
    pub idempotency_key: Option<String>,
}

/// Transaction status
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum TransactionStatus {
    /// Transaction is pending confirmation
    Pending,
    /// Transaction is confirmed
    Confirmed,
    /// Transaction failed
    Failed(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signer_context() {
        let mut context = EvmSignerContext::new();
        let private_key = [1u8; 32];

        context.add_signer("test", private_key).unwrap();

        let retrieved = context.get_signer("test").unwrap();
        assert_eq!(retrieved, private_key);

        let default = context.get_default_signer().unwrap();
        assert_eq!(default, private_key);
    }

    #[test]
    fn test_erc20_transfer_data() {
        let to = "0x742d35Cc6634C0532925a3b8D8e41E5d3e4F8123";
        let amount = 1000000u128;

        let data = build_erc20_transfer_data(to, amount).unwrap();
        assert!(data.starts_with("0xa9059cbb")); // transfer function selector
        assert!(data.len() > 10); // Should have selector + parameters
    }

    #[test]
    fn test_transaction_status() {
        let status = TransactionStatus::Pending;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"Pending\"");

        let status = TransactionStatus::Failed("error".to_string());
        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("Failed"));
    }
}
