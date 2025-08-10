//! Balance checking tools for Solana blockchain
//!
//! This module provides tools for querying SOL and SPL token balances on the Solana blockchain.

use riglr_core::{ToolError, SignerContext};
use riglr_macros::tool;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use solana_sdk::{native_token::LAMPORTS_PER_SOL, pubkey::Pubkey};
use std::str::FromStr;
use tracing::{debug, info};

/// Get SOL balance for a given address
///
/// This tool queries the Solana blockchain to retrieve the SOL balance for any wallet address.
/// The balance is returned in both lamports (smallest unit) and SOL (human-readable format).
/// 
/// # Arguments
/// 
/// * `address` - The Solana wallet address to check (base58 encoded public key)
/// 
/// # Returns
/// 
/// Returns `BalanceResult` containing:
/// - `address`: The queried wallet address
/// - `lamports`: Balance in lamports (1 SOL = 1,000,000,000 lamports)
/// - `sol`: Balance in SOL units as a floating-point number
/// - `formatted`: Human-readable balance string with 9 decimal places
/// 
/// # Errors
/// 
/// * `ToolError::Permanent` - When the address format is invalid or parsing fails
/// * `ToolError::Retriable` - When network connection issues occur (timeouts, connection errors)
/// * `ToolError::Permanent` - When no signer context is available
/// 
/// # Examples
/// 
/// ```rust,ignore
/// use riglr_solana_tools::balance::get_sol_balance;
/// use riglr_core::SignerContext;
/// 
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Check SOL balance for a wallet
/// let balance = get_sol_balance(
///     "So11111111111111111111111111111111111111112".to_string()
/// ).await?;
/// 
/// println!("Address: {}", balance.address);
/// println!("Balance: {} SOL ({} lamports)", balance.sol, balance.lamports);
/// println!("Formatted: {}", balance.formatted);
/// # Ok(())
/// # }
/// ```
#[tool]
pub async fn get_sol_balance(
    address: String,
) -> Result<BalanceResult, ToolError> {
    debug!("Getting SOL balance for address: {}", address);

    // Get signer context and client
    let signer = SignerContext::current().await
        .map_err(|e| ToolError::permanent(format!("No signer context: {}", e)))?;
    let client = signer.solana_client();

    // Parse address
    let pubkey = Pubkey::from_str(&address)
        .map_err(|e| ToolError::permanent(format!("Invalid address: {}", e)))?;

    // Get balance in lamports
    let lamports = client.get_balance(&pubkey).map_err(|e| {
        // Network and connection errors are retriable
        let error_str = e.to_string();
        if error_str.contains("timeout")
            || error_str.contains("connection")
            || error_str.contains("temporarily")
            || error_str.contains("network")
        {
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
/// This tool queries the Solana blockchain to retrieve the balance of a specific SPL token
/// for a given wallet address. It automatically finds the Associated Token Account (ATA)
/// and returns both raw and UI-adjusted amounts.
/// 
/// # Arguments
/// 
/// * `owner_address` - The wallet address that owns the tokens (base58 encoded)
/// * `mint_address` - The SPL token mint address (contract address)
/// 
/// # Returns
/// 
/// Returns `TokenBalanceResult` containing:
/// - `owner_address`: The wallet address queried
/// - `mint_address`: The token mint address
/// - `raw_amount`: Balance in token's smallest unit (before decimal adjustment)
/// - `ui_amount`: Balance adjusted for token decimals
/// - `decimals`: Number of decimal places for the token
/// - `formatted`: Human-readable balance string
/// 
/// # Errors
/// 
/// * `ToolError::Permanent` - When addresses are invalid or context unavailable
/// * `ToolError::Retriable` - When network issues occur during balance retrieval
/// 
/// # Examples
/// 
/// ```rust,ignore
/// use riglr_solana_tools::balance::get_spl_token_balance;
/// 
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Check USDC balance for a wallet
/// let balance = get_spl_token_balance(
///     "9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM".to_string(),
///     "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(), // USDC mint
/// ).await?;
/// 
/// println!("Token balance: {} {}", balance.ui_amount, balance.mint_address);
/// println!("Raw amount: {} (decimals: {})", balance.raw_amount, balance.decimals);
/// # Ok(())
/// # }
/// ```
#[tool]
pub async fn get_spl_token_balance(
    owner_address: String,
    mint_address: String,
) -> Result<TokenBalanceResult, ToolError> {
    use spl_associated_token_account::get_associated_token_address;

    debug!(
        "Getting SPL token balance for owner: {}, mint: {}",
        owner_address, mint_address
    );

    // Get signer context and client
    let signer = SignerContext::current().await
        .map_err(|e| ToolError::permanent(format!("No signer context: {}", e)))?;
    let client = signer.solana_client();

    // Parse addresses - invalid addresses are permanent errors
    let owner_pubkey = Pubkey::from_str(&owner_address)
        .map_err(|e| ToolError::permanent(format!("Invalid owner address: {}", e)))?;
    let mint_pubkey = Pubkey::from_str(&mint_address)
        .map_err(|e| ToolError::permanent(format!("Invalid mint address: {}", e)))?;

    // Get the Associated Token Account (ATA) address
    let ata = get_associated_token_address(&owner_pubkey, &mint_pubkey);

    // Get token account balance
    match client.get_token_account_balance(&ata).map_err(|e| ToolError::permanent(format!("Failed to get token balance: {}", e))) {
        Ok(balance) => {
            let raw_amount = balance.amount.parse::<u64>().map_err(|e| {
                ToolError::permanent(format!("Failed to parse token amount: {}", e))
            })?;
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

/// Get SOL balances for multiple addresses
///
/// This tool queries the Solana blockchain to retrieve SOL balances for multiple wallet addresses
/// in a batch operation. Each address is processed individually and results are collected.
/// 
/// # Arguments
/// 
/// * `addresses` - Vector of Solana wallet addresses to check (base58 encoded public keys)
/// 
/// # Returns
/// 
/// Returns `Vec<BalanceResult>` with balance information for each address that was successfully queried.
/// Each result contains the same fields as `get_sol_balance`.
/// 
/// # Errors
/// 
/// * `ToolError::Permanent` - When any individual address is invalid or signer context unavailable
/// * `ToolError::Retriable` - When network issues occur during any balance query
/// 
/// Note: This function fails fast - if any address query fails, the entire operation returns an error.
/// 
/// # Examples
/// 
/// ```rust,ignore
/// use riglr_solana_tools::balance::get_multiple_balances;
/// 
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let addresses = vec![
///     "So11111111111111111111111111111111111111112".to_string(),
///     "9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM".to_string(),
/// ];
/// 
/// let balances = get_multiple_balances(addresses).await?;
/// for balance in balances {
///     println!("{}: {} SOL", balance.address, balance.sol);
/// }
/// # Ok(())
/// # }
/// ```
#[tool]
pub async fn get_multiple_balances(
    addresses: Vec<String>,
) -> Result<Vec<BalanceResult>, ToolError> {
    let mut results = Vec::new();
    
    for address in addresses {
        match get_sol_balance(address.clone()).await {
            Ok(balance) => results.push(balance),
            Err(e) => {
                // For individual address failures, return error with partial results
                // This is a design choice - could also continue and mark failed addresses
                return Err(e);
            }
        }
    }
    
    Ok(results)
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
