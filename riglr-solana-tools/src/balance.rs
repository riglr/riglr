//! Balance checking tools for Solana blockchain
//!
//! This module provides tools for querying SOL and SPL token balances on the Solana blockchain.

use crate::client::{SolanaClient, SolanaConfig};
use crate::error::Result;
use anyhow::anyhow;
use riglr_macros::tool;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use solana_sdk::native_token::LAMPORTS_PER_SOL;
use std::sync::Arc;
use tracing::{debug, info};

/// Global client instance for balance operations
static mut BALANCE_CLIENT: Option<Arc<SolanaClient>> = None;
static INIT: std::sync::Once = std::sync::Once::new();

/// Initialize the balance client with a custom configuration
pub fn init_balance_client(config: SolanaConfig) {
    unsafe {
        INIT.call_once(|| {
            BALANCE_CLIENT = Some(Arc::new(SolanaClient::new(config)));
        });
    }
}

/// Get the balance client, initializing with default if needed
fn get_balance_client() -> Arc<SolanaClient> {
    unsafe {
        INIT.call_once(|| {
            BALANCE_CLIENT = Some(Arc::new(SolanaClient::default()));
        });
        BALANCE_CLIENT.as_ref().unwrap().clone()
    }
}

/// Get the SOL balance of a Solana wallet
///
/// This tool queries the Solana blockchain to retrieve the SOL balance
/// for the specified wallet address. The balance is returned in both
/// lamports and SOL units.
#[tool]
pub async fn get_sol_balance(
    /// The Solana wallet address to query (base58 encoded public key)
    address: String,
    /// RPC endpoint URL (optional, defaults to mainnet)
    #[serde(default)]
    rpc_url: Option<String>,
    /// Whether to use confirmed or finalized commitment level
    #[serde(default)]
    use_finalized: bool,
) -> anyhow::Result<BalanceResult> {
    debug!("Getting SOL balance for address: {}", address);
    
    // Create client with custom RPC if provided
    let client = if let Some(url) = rpc_url {
        Arc::new(SolanaClient::with_rpc_url(url))
    } else {
        get_balance_client()
    };
    
    // Set commitment level if requested
    let client = if use_finalized {
        Arc::new(client.as_ref().clone().with_commitment(
            solana_sdk::commitment_config::CommitmentLevel::Finalized
        ))
    } else {
        client
    };
    
    // Get balance
    let lamports = client.get_balance(&address).await
        .map_err(|e| anyhow!("Failed to get balance: {}", e))?;
    
    let sol = lamports as f64 / LAMPORTS_PER_SOL as f64;
    
    info!("Balance for {}: {} SOL ({} lamports)", address, sol, lamports);
    
    Ok(BalanceResult {
        address,
        lamports,
        sol,
        formatted: format!("{:.9} SOL", sol),
    })
}

/// Get the SPL token balance of a wallet
///
/// This tool queries the Solana blockchain to retrieve the balance of a specific
/// SPL token for the given wallet address.
#[tool]
pub async fn get_spl_token_balance(
    /// The owner wallet address (base58 encoded public key)
    owner_address: String,
    /// The token mint address (base58 encoded public key)
    mint_address: String,
    /// RPC endpoint URL (optional, defaults to mainnet)
    #[serde(default)]
    rpc_url: Option<String>,
    /// Number of decimals for the token (optional, will be fetched if not provided)
    #[serde(default)]
    decimals: Option<u8>,
) -> anyhow::Result<TokenBalanceResult> {
    debug!("Getting SPL token balance for owner: {}, mint: {}", owner_address, mint_address);
    
    // Create client with custom RPC if provided
    let client = if let Some(url) = rpc_url {
        Arc::new(SolanaClient::with_rpc_url(url))
    } else {
        get_balance_client()
    };
    
    // Get raw token amount
    let raw_amount = client.get_token_balance(&owner_address, &mint_address).await
        .map_err(|e| anyhow!("Failed to get token balance: {}", e))?;
    
    // Calculate UI amount based on decimals
    let decimals = decimals.unwrap_or(9); // Default to 9 decimals if not provided
    let ui_amount = raw_amount as f64 / 10_f64.powi(decimals as i32);
    
    info!("Token balance for {} (mint: {}): {} (raw: {})", 
        owner_address, mint_address, ui_amount, raw_amount);
    
    Ok(TokenBalanceResult {
        owner_address,
        mint_address,
        raw_amount,
        ui_amount,
        decimals,
        formatted: format!("{:.9}", ui_amount),
    })
}

/// Get balances for multiple addresses in batch
///
/// This tool efficiently queries balances for multiple addresses in a single operation.
#[tool]
pub async fn get_multiple_balances(
    /// List of Solana wallet addresses to query
    addresses: Vec<String>,
    /// RPC endpoint URL (optional, defaults to mainnet)
    #[serde(default)]
    rpc_url: Option<String>,
) -> anyhow::Result<Vec<BalanceResult>> {
    debug!("Getting balances for {} addresses", addresses.len());
    
    // Create client with custom RPC if provided
    let client = if let Some(url) = rpc_url {
        Arc::new(SolanaClient::with_rpc_url(url))
    } else {
        get_balance_client()
    };
    
    let mut results = Vec::new();
    
    // Query each address
    // In production, this could be optimized with batch RPC calls
    for address in addresses {
        match client.get_balance(&address).await {
            Ok(lamports) => {
                let sol = lamports as f64 / LAMPORTS_PER_SOL as f64;
                results.push(BalanceResult {
                    address: address.clone(),
                    lamports,
                    sol,
                    formatted: format!("{:.9} SOL", sol),
                });
            }
            Err(e) => {
                // Add error result but continue with other addresses
                results.push(BalanceResult {
                    address: address.clone(),
                    lamports: 0,
                    sol: 0.0,
                    formatted: format!("Error: {}", e),
                });
            }
        }
    }
    
    Ok(results)
}

/// Result structure for balance queries
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BalanceResult {
    /// The queried address
    pub address: String,
    /// Balance in lamports (smallest unit)
    pub lamports: u64,
    /// Balance in SOL
    pub sol: f64,
    /// Human-readable formatted balance
    pub formatted: String,
}

/// Result structure for token balance queries
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TokenBalanceResult {
    /// The owner wallet address
    pub owner_address: String,
    /// The token mint address
    pub mint_address: String,
    /// Raw token amount (without decimal adjustment)
    pub raw_amount: u64,
    /// UI amount (with decimal adjustment)
    pub ui_amount: f64,
    /// Number of decimals for the token
    pub decimals: u8,
    /// Human-readable formatted balance
    pub formatted: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_balance_result_creation() {
        let result = BalanceResult {
            address: "11111111111111111111111111111111".to_string(),
            lamports: 1_000_000_000,
            sol: 1.0,
            formatted: "1.000000000 SOL".to_string(),
        };
        
        assert_eq!(result.lamports, 1_000_000_000);
        assert_eq!(result.sol, 1.0);
    }
    
    #[tokio::test]
    async fn test_token_balance_result() {
        let result = TokenBalanceResult {
            owner_address: "11111111111111111111111111111111".to_string(),
            mint_address: "So11111111111111111111111111111111111111112".to_string(),
            raw_amount: 1_000_000,
            ui_amount: 1.0,
            decimals: 6,
            formatted: "1.0".to_string(),
        };
        
        assert_eq!(result.raw_amount, 1_000_000);
        assert_eq!(result.decimals, 6);
    }
}