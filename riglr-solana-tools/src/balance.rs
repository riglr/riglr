//! Balance checking tools for Solana blockchain
//!
//! This module provides tools for querying SOL and SPL token balances on the Solana blockchain.

use crate::client::{SolanaClient, SolanaConfig};
use riglr_core::ToolError;
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

/// Get SOL balance for a given address
///
/// This tool queries the Solana blockchain to retrieve the SOL balance
/// in both lamports and SOL units.
// #[tool]
pub async fn get_sol_balance(
    client: &SolanaClient,
    address: String,
) -> Result<BalanceResult, ToolError> {
    debug!("Getting SOL balance for address: {}", address);

    // Get balance in lamports
    let lamports = client
        .get_balance(&address)
        .await
        .map_err(|e| {
            // Network and connection errors are retriable
            let error_str = e.to_string();
            if error_str.contains("timeout") || 
               error_str.contains("connection") || 
               error_str.contains("temporarily") ||
               error_str.contains("network") {
                ToolError::retriable(format!("Failed to get balance: {}", e))
            } else {
                ToolError::permanent(format!("Failed to get balance: {}", e))
            }
        })?;

    // Convert to SOL
    let sol = lamports as f64 / LAMPORTS_PER_SOL as f64;

    info!(
        "Balance for {}: {} SOL ({} lamports)",
        address, sol, lamports
    );

    Ok(BalanceResult {
        address,
        lamports,
        sol,
        formatted: format!("{:.9} SOL", sol),
    })
}

/// Get SPL token balance for a given owner and mint
///
/// This tool queries the Solana blockchain to retrieve the balance of a specific
/// SPL token for a given owner address.
// #[tool]
pub async fn get_spl_token_balance(
    client: &SolanaClient,
    owner_address: String,
    mint_address: String,
) -> Result<TokenBalanceResult, ToolError> {
    use solana_sdk::pubkey::Pubkey;
    use spl_associated_token_account::get_associated_token_address;
    use std::str::FromStr;

    debug!(
        "Getting SPL token balance for owner: {}, mint: {}",
        owner_address, mint_address
    );

    // Parse addresses - invalid addresses are permanent errors
    let owner_pubkey = Pubkey::from_str(&owner_address)
        .map_err(|e| ToolError::permanent(format!("Invalid owner address: {}", e)))?;
    let mint_pubkey = Pubkey::from_str(&mint_address)
        .map_err(|e| ToolError::permanent(format!("Invalid mint address: {}", e)))?;

    // Get the Associated Token Account (ATA) address
    let ata = get_associated_token_address(&owner_pubkey, &mint_pubkey);

    // Get token account balance
    match client.get_token_account_balance(&ata.to_string()).await {
        Ok(balance) => {
            let raw_amount = balance
                .amount
                .parse::<u64>()
                .map_err(|e| ToolError::permanent(format!("Failed to parse token amount: {}", e)))?;
            let ui_amount = balance.ui_amount.unwrap_or(0.0);
            let decimals = balance.decimals;

            info!(
                "Token balance for {} (mint: {}): {} (raw: {})",
                owner_address, mint_address, ui_amount, raw_amount
            );

            Ok(TokenBalanceResult {
                owner_address,
                mint_address,
                raw_amount,
                ui_amount,
                decimals,
                formatted: format!("{:.9}", ui_amount),
            })
        }
        Err(_) => {
            // Account doesn't exist or has zero balance
            info!(
                "No token account found for owner: {}, mint: {}",
                owner_address, mint_address
            );
            Ok(TokenBalanceResult {
                owner_address,
                mint_address,
                raw_amount: 0,
                ui_amount: 0.0,
                decimals: 9, // Default to 9 decimals
                formatted: "0.000000000".to_string(),
            })
        }
    }
}

///
// #[tool]
pub async fn get_multiple_balances(
    _addresses: Vec<String>,

    _rpc_url: Option<String>,
) -> Result<Vec<BalanceResult>, ToolError> {
    Err(ToolError::permanent("get_multiple_balances not yet implemented"))
}

/// Result structure for balance queries
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BalanceResult {
    pub address: String,
    /// Balance in lamports (smallest unit)
    pub lamports: u64,
    /// Balance in SOL
    pub sol: f64,
    /// Human-readable formatted balance
    pub formatted: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TokenBalanceResult {
    pub owner_address: String,
    pub mint_address: String,
    pub raw_amount: u64,
    /// UI amount (with decimal adjustment)
    pub ui_amount: f64,
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
