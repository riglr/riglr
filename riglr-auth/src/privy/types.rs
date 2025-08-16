//! Privy-specific types and data structures

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Privy user data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivyUserData {
    /// User ID
    pub id: String,

    /// Linked accounts
    pub linked_accounts: Vec<LinkedAccount>,

    /// Whether user is verified
    #[serde(default)]
    pub verified: bool,

    /// User metadata
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

impl PrivyUserData {
    /// Get Solana wallet if available
    pub fn solana_wallet(&self) -> Option<&PrivyWallet> {
        self.linked_accounts
            .iter()
            .filter_map(|account| match account {
                LinkedAccount::Wallet(w) if w.chain_type == "solana" && w.delegated => Some(w),
                _ => None,
            })
            .next()
    }

    /// Get EVM wallet if available
    pub fn evm_wallet(&self) -> Option<&PrivyWallet> {
        self.linked_accounts
            .iter()
            .filter_map(|account| match account {
                LinkedAccount::Wallet(w) if w.chain_type == "ethereum" && w.delegated => Some(w),
                _ => None,
            })
            .next()
    }

    /// Get email if available
    pub fn email(&self) -> Option<String> {
        self.linked_accounts
            .iter()
            .filter_map(|account| match account {
                LinkedAccount::Email { address, .. } => Some(address.clone()),
                _ => None,
            })
            .next()
    }
}

/// Linked account types in Privy
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum LinkedAccount {
    /// Wallet account
    Wallet(PrivyWallet),

    /// Email account
    Email {
        address: String,
        #[serde(default)]
        verified: bool,
    },

    /// Phone account
    Phone {
        number: String,
        #[serde(default)]
        verified: bool,
    },

    /// Social account
    Social {
        provider: String,
        username: Option<String>,
        #[serde(default)]
        verified: bool,
    },

    /// Other account types
    #[serde(other)]
    Other,
}

/// Privy wallet information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivyWallet {
    /// Wallet ID
    #[serde(default)]
    pub id: Option<String>,

    /// Wallet address
    pub address: String,

    /// Chain type (solana, ethereum, etc.)
    pub chain_type: String,

    /// Wallet client type
    #[serde(default)]
    pub wallet_client: String,

    /// Whether this is a delegated wallet
    #[serde(default)]
    pub delegated: bool,

    /// Whether wallet is imported
    #[serde(default)]
    pub imported: bool,
}

/// JWT claims for Privy tokens
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct PrivyClaims {
    /// Subject (user ID)
    pub sub: String,

    /// Audience (app ID)
    pub aud: String,

    /// Issuer
    pub iss: String,

    /// Session ID
    #[serde(default)]
    pub sid: String,

    /// Expiration time
    pub exp: i64,

    /// Issued at
    pub iat: i64,
}

/// Privy RPC request structure
#[derive(Debug, Serialize)]
pub struct PrivyRpcRequest {
    /// Wallet address
    pub address: String,

    /// Chain type
    pub chain_type: String,

    /// RPC method
    pub method: String,

    /// CAIP-2 chain identifier
    pub caip2: String,

    /// Method parameters
    pub params: serde_json::Value,
}

/// Privy RPC response structure
#[derive(Debug, Deserialize)]
pub struct PrivyRpcResponse {
    /// Response data
    pub data: serde_json::Value,
}

/// Privy transaction parameters for Solana
#[derive(Debug, Serialize)]
pub struct PrivySolanaTransactionParams {
    /// Base64-encoded transaction
    pub transaction: String,

    /// Encoding type
    pub encoding: String,
}

/// Privy transaction parameters for EVM
#[derive(Debug, Serialize)]
pub struct PrivyEvmTransactionParams {
    /// From address
    pub from: String,

    /// To address
    pub to: String,

    /// Value in hex
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,

    /// Data in hex
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,

    /// Gas limit
    #[serde(skip_serializing_if = "Option::is_none", rename = "gasLimit")]
    pub gas_limit: Option<String>,

    /// Gas price
    #[serde(skip_serializing_if = "Option::is_none", rename = "gasPrice")]
    pub gas_price: Option<String>,

    /// Transaction type
    #[serde(skip_serializing_if = "Option::is_none", rename = "type")]
    pub tx_type: Option<u8>,
}
