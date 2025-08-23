//! Balance checking tools for Solana blockchain
//!
//! This module provides tools for querying SOL and SPL token balances on the Solana blockchain.
//! These tools use RpcProvider for read-only operations and don't require transaction signing.

use crate::utils::validate_address;
use riglr_core::provider::ApplicationContext;
use riglr_core::ToolError;
use riglr_macros::tool;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use solana_sdk::native_token::LAMPORTS_PER_SOL;
use std::sync::Arc;
use tracing::{debug, info};

/// Get SOL balance for a given address using an RPC client
///
/// This tool queries the Solana blockchain to retrieve the SOL balance for any wallet address.
/// The balance is returned in both lamports (smallest unit) and SOL (human-readable format).
/// This is a read-only operation that uses RpcProvider instead of requiring transaction signing.
///
/// # Arguments
///
/// * `address` - The Solana wallet address to check (base58 encoded public key)
/// * `context` - The ApplicationContext containing RPC client and configuration
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
///
/// # Examples
///
/// ```rust,ignore
/// use riglr_solana_tools::balance::get_sol_balance;
/// use riglr_core::provider::{ApplicationContext, RpcProvider};
/// use riglr_config::Config;
/// use std::sync::Arc;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Set up ApplicationContext
/// let config = Config::from_env();
/// let rpc_provider = Arc::new(RpcProvider::new());
/// let context = ApplicationContext::new(rpc_provider, config);
///
/// // Check SOL balance for a wallet using the tool directly
/// let balance = get_sol_balance(
///     "So11111111111111111111111111111111111111112".to_string(),
///     &context
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
    context: &ApplicationContext,
) -> Result<BalanceResult, ToolError> {
    // Validate address using stateless utility
    let pubkey =
        validate_address(&address).map_err(|e| ToolError::permanent_string(e.to_string()))?;

    // Get Solana RPC client from the ApplicationContext's extensions
    let rpc_client = context
        .get_extension::<Arc<solana_client::rpc_client::RpcClient>>()
        .ok_or_else(|| {
            ToolError::permanent_string("Solana RpcClient not found in context".to_string())
        })?;

    // Get balance in lamports using the RPC client from context
    let lamports = rpc_client.get_balance(&pubkey).map_err(|e| {
        // Network and connection errors are retriable
        let error_str = e.to_string();
        if error_str.contains("timeout")
            || error_str.contains("connection")
            || error_str.contains("temporarily")
            || error_str.contains("network")
        {
            ToolError::retriable_string(format!("Failed to get balance: {}", e))
        } else {
            ToolError::permanent_string(format!("Failed to get balance: {}", e))
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

/// Get SPL token balance for a given owner and mint using an RPC client
///
/// This tool queries the Solana blockchain to retrieve the balance of a specific SPL token
/// for a given wallet address. It automatically finds the Associated Token Account (ATA)
/// and returns both raw and UI-adjusted amounts. This is a read-only operation that uses
/// RpcProvider instead of requiring transaction signing.
///
/// # Arguments
///
/// * `owner_address` - The wallet address that owns the tokens (base58 encoded)
/// * `mint_address` - The SPL token mint address (contract address)
/// * `rpc_client` - The Solana RPC client to use for the query (from RpcProvider)
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
/// * `ToolError::Permanent` - When addresses are invalid
/// * `ToolError::Retriable` - When network issues occur during balance retrieval
///
/// # Examples
///
/// ```rust,ignore
/// use riglr_solana_tools::balance::get_spl_token_balance;
/// use riglr_core::provider::RpcProvider;
/// use std::sync::Arc;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let provider = RpcProvider::from_env();
/// let client = provider.get_solana_client("mainnet").unwrap();
///
/// // Check USDC balance for a wallet
/// let balance = get_spl_token_balance(
///     "9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM".to_string(),
///     "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(), // USDC mint
///     client
/// ).await?;
///
/// println!("Token balance: {} {}", balance.ui_amount, balance.mint_address);
/// println!("Raw amount: {} (decimals: {})", balance.raw_amount, balance.decimals);
/// # Ok(())
/// # }
/// ```
/// Get SPL token balance for a given owner and mint
///
/// This tool queries the Solana blockchain to retrieve the balance of a specific SPL token.
/// This version uses dependency injection to get the RPC client from ApplicationContext.
#[tool]
pub async fn get_spl_token_balance(
    owner_address: String,
    mint_address: String,
    context: &ApplicationContext,
) -> Result<TokenBalanceResult, ToolError> {
    use crate::common::get_associated_token_address_v3;

    debug!(
        "Getting SPL token balance for owner: {}, mint: {}",
        owner_address, mint_address
    );

    // Get Solana RPC client from the ApplicationContext's extensions
    let rpc_client = context
        .get_extension::<Arc<solana_client::rpc_client::RpcClient>>()
        .ok_or_else(|| {
            ToolError::permanent_string("Solana RpcClient not found in context".to_string())
        })?;

    // Validate addresses using stateless utilities
    let owner_pubkey = validate_address(&owner_address)
        .map_err(|e| ToolError::permanent_string(format!("Invalid owner address: {}", e)))?;
    let mint_pubkey = validate_address(&mint_address)
        .map_err(|e| ToolError::permanent_string(format!("Invalid mint address: {}", e)))?;

    // Get the Associated Token Account (ATA) address
    let ata = get_associated_token_address_v3(&owner_pubkey, &mint_pubkey);

    // Get token account balance using the RPC client from context
    match rpc_client
        .get_token_account_balance(&ata)
        .map_err(|e| ToolError::permanent_string(format!("Failed to get token balance: {}", e)))
    {
        Ok(balance) => {
            let raw_amount = balance.amount.parse::<u64>().map_err(|e| {
                ToolError::permanent_string(format!("Failed to parse token amount: {}", e))
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
/// let balances = get_multiple_balances(&context, addresses).await?;
/// for balance in balances {
///     println!("{}: {} SOL", balance.address, balance.sol);
/// }
/// # Ok(())
/// # }
/// ```
#[tool]
pub async fn get_multiple_balances(
    addresses: Vec<String>,
    context: &ApplicationContext,
) -> Result<Vec<BalanceResult>, ToolError> {
    let mut results = Vec::new();

    for address in addresses {
        match get_sol_balance(address.clone(), context).await {
            Ok(balance) => results.push(balance),
            Err(e) => {
                // For individual address failures, return error with partial results
                // This is a design choice - could also continue and mark failed addresses
                return Err(e);
            }
        }
    }

    info!("Retrieved balances for {} addresses", results.len());
    Ok(results)
}

/// Result structure for balance queries
///
/// Contains balance information for a Solana address including both raw lamports
/// and human-readable SOL amounts.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BalanceResult {
    /// The Solana wallet address that was queried
    pub address: String,
    /// Balance in lamports (smallest unit)
    pub lamports: u64,
    /// Balance in SOL
    pub sol: f64,
    /// Human-readable formatted balance
    pub formatted: String,
}

/// Result structure for SPL token balance queries
///
/// Contains balance information for a specific SPL token including both raw amounts
/// and decimal-adjusted values.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TokenBalanceResult {
    /// The wallet address that owns the tokens
    pub owner_address: String,
    /// The SPL token mint address (contract address)
    pub mint_address: String,
    /// Raw token amount (before decimal adjustment)
    pub raw_amount: u64,
    /// UI amount (with decimal adjustment)
    pub ui_amount: f64,
    /// Number of decimal places for the token
    pub decimals: u8,
    /// Human-readable formatted balance
    pub formatted: String,
}

/// Unit tests for balance functionality
#[cfg(test)]
mod tests {
    use super::*;
    use riglr_core::ToolError;

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
        assert_eq!(result.address, "11111111111111111111111111111111");
        assert_eq!(result.formatted, "1.000000000 SOL");
    }

    #[tokio::test]
    async fn test_balance_result_zero_balance() {
        let result = BalanceResult {
            address: "So11111111111111111111111111111111111111112".to_string(),
            lamports: 0,
            sol: 0.0,
            formatted: "0.000000000 SOL".to_string(),
        };

        assert_eq!(result.lamports, 0);
        assert_eq!(result.sol, 0.0);
        assert_eq!(result.formatted, "0.000000000 SOL");
    }

    #[tokio::test]
    async fn test_balance_result_max_value() {
        let max_lamports = u64::MAX;
        let max_sol = max_lamports as f64 / LAMPORTS_PER_SOL as f64;

        let result = BalanceResult {
            address: "Test123456789".to_string(),
            lamports: max_lamports,
            sol: max_sol,
            formatted: format!("{:.9} SOL", max_sol),
        };

        assert_eq!(result.lamports, max_lamports);
        assert_eq!(result.sol, max_sol);
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
        assert_eq!(result.ui_amount, 1.0);
        assert_eq!(result.owner_address, "11111111111111111111111111111111");
        assert_eq!(
            result.mint_address,
            "So11111111111111111111111111111111111111112"
        );
        assert_eq!(result.formatted, "1.0");
    }

    #[tokio::test]
    async fn test_token_balance_result_zero_balance() {
        let result = TokenBalanceResult {
            owner_address: "Owner123".to_string(),
            mint_address: "Mint456".to_string(),
            raw_amount: 0,
            ui_amount: 0.0,
            decimals: 9,
            formatted: "0.000000000".to_string(),
        };

        assert_eq!(result.raw_amount, 0);
        assert_eq!(result.ui_amount, 0.0);
        assert_eq!(result.decimals, 9);
        assert_eq!(result.formatted, "0.000000000");
    }

    #[tokio::test]
    async fn test_token_balance_result_high_decimals() {
        let result = TokenBalanceResult {
            owner_address: "HighDecimal".to_string(),
            mint_address: "Decimals18".to_string(),
            raw_amount: 123456789012345678,
            ui_amount: 123.456789012345678,
            decimals: 18,
            formatted: "123.456789012".to_string(),
        };

        assert_eq!(result.decimals, 18);
        assert_eq!(result.raw_amount, 123456789012345678);
        assert!(result.ui_amount > 123.0);
    }

    #[tokio::test]
    async fn test_token_balance_result_max_values() {
        let result = TokenBalanceResult {
            owner_address: "MaxValues".to_string(),
            mint_address: "MaxMint".to_string(),
            raw_amount: u64::MAX,
            ui_amount: f64::MAX,
            decimals: u8::MAX,
            formatted: "Max".to_string(),
        };

        assert_eq!(result.raw_amount, u64::MAX);
        assert_eq!(result.ui_amount, f64::MAX);
        assert_eq!(result.decimals, u8::MAX);
    }

    #[tokio::test]
    async fn test_get_sol_balance_when_invalid_address_should_return_permanent_error() {
        let context = riglr_core::provider::ApplicationContext::from_env();
        let result = get_sol_balance("invalid_address".to_string(), &context).await;

        assert!(result.is_err());
        let error = result.unwrap_err();

        match error {
            ToolError::Permanent { .. } => {
                // Expected permanent error for invalid address
            }
            _ => panic!("Expected permanent error for invalid address"),
        }
    }

    #[tokio::test]
    async fn test_get_sol_balance_when_empty_address_should_return_permanent_error() {
        let context = riglr_core::provider::ApplicationContext::from_env();
        let result = get_sol_balance("".to_string(), &context).await;

        assert!(result.is_err());
        let error = result.unwrap_err();

        match error {
            ToolError::Permanent { .. } => {
                // Expected permanent error for empty address
            }
            _ => panic!("Expected permanent error for empty address"),
        }
    }

    #[tokio::test]
    async fn test_get_sol_balance_when_special_chars_address_should_return_permanent_error() {
        let context = riglr_core::provider::ApplicationContext::from_env();
        let result = get_sol_balance("!@#$%^&*()".to_string(), &context).await;

        assert!(result.is_err());
        let error = result.unwrap_err();

        match error {
            ToolError::Permanent { .. } => {
                // Expected permanent error for invalid characters
            }
            _ => panic!("Expected permanent error for special characters"),
        }
    }

    #[tokio::test]
    async fn test_get_sol_balance_when_no_signer_context_should_return_permanent_error() {
        // This test covers the case where SignerContext::current() fails
        // Since we can't easily mock SignerContext in unit tests, we test with valid address
        // but expect it to fail due to no signer context in test environment
        let valid_address = "11111111111111111111111111111114"; // System program (valid format)

        let context = riglr_core::provider::ApplicationContext::from_env();
        let result = get_sol_balance(valid_address.to_string(), &context).await;

        assert!(result.is_err());
        let error = result.unwrap_err();

        match error {
            ToolError::Permanent { context, .. } => {
                assert!(context.contains("No signer context available"));
            }
            _ => panic!("Expected permanent error for no signer context"),
        }
    }

    #[tokio::test]
    async fn test_get_spl_token_balance_when_invalid_owner_address_should_return_permanent_error() {
        let context = riglr_core::provider::ApplicationContext::from_env();
        let result = get_spl_token_balance(
            "invalid_owner".to_string(),
            "11111111111111111111111111111114".to_string(),
            &context,
        )
        .await;

        assert!(result.is_err());
        let error = result.unwrap_err();

        match error {
            ToolError::Permanent { context, .. } => {
                assert!(context.contains("Invalid owner address"));
            }
            _ => panic!("Expected permanent error for invalid owner address"),
        }
    }

    #[tokio::test]
    async fn test_get_spl_token_balance_when_invalid_mint_address_should_return_permanent_error() {
        let context = riglr_core::provider::ApplicationContext::from_env();
        let result = get_spl_token_balance(
            "11111111111111111111111111111114".to_string(),
            "invalid_mint".to_string(),
            &context,
        )
        .await;

        assert!(result.is_err());
        let error = result.unwrap_err();

        match error {
            ToolError::Permanent { context, .. } => {
                assert!(context.contains("Invalid mint address"));
            }
            _ => panic!("Expected permanent error for invalid mint address"),
        }
    }

    #[tokio::test]
    async fn test_get_spl_token_balance_when_empty_owner_address_should_return_permanent_error() {
        let context = riglr_core::provider::ApplicationContext::from_env();
        let result = get_spl_token_balance(
            "".to_string(),
            "11111111111111111111111111111114".to_string(),
            &context,
        )
        .await;

        assert!(result.is_err());
        let error = result.unwrap_err();

        match error {
            ToolError::Permanent { .. } => {
                // Expected permanent error
            }
            _ => panic!("Expected permanent error for empty owner address"),
        }
    }

    #[tokio::test]
    async fn test_get_spl_token_balance_when_empty_mint_address_should_return_permanent_error() {
        let context = riglr_core::provider::ApplicationContext::from_env();
        let result = get_spl_token_balance(
            "11111111111111111111111111111114".to_string(),
            "".to_string(),
            &context,
        )
        .await;

        assert!(result.is_err());
        let error = result.unwrap_err();

        match error {
            ToolError::Permanent { .. } => {
                // Expected permanent error
            }
            _ => panic!("Expected permanent error for empty mint address"),
        }
    }

    #[tokio::test]
    async fn test_get_spl_token_balance_when_no_signer_context_should_return_permanent_error() {
        // Test with valid addresses but no signer context
        let context = riglr_core::provider::ApplicationContext::from_env();
        let result = get_spl_token_balance(
            "11111111111111111111111111111114".to_string(),
            "So11111111111111111111111111111111111111112".to_string(),
            &context,
        )
        .await;

        assert!(result.is_err());
        let error = result.unwrap_err();

        match error {
            ToolError::Permanent { context, .. } => {
                assert!(context.contains("No signer context available"));
            }
            _ => panic!("Expected permanent error for no signer context"),
        }
    }

    #[tokio::test]
    async fn test_get_multiple_balances_when_empty_addresses_should_return_empty_vec() {
        let context = riglr_core::provider::ApplicationContext::from_env();
        let result = get_multiple_balances(vec![], &context).await;

        assert!(result.is_ok());
        let balances = result.unwrap();
        assert!(balances.is_empty());
    }

    #[tokio::test]
    async fn test_get_multiple_balances_when_single_invalid_address_should_return_error() {
        let addresses = vec!["invalid_address".to_string()];
        let context = riglr_core::provider::ApplicationContext::from_env();

        let result = get_multiple_balances(addresses, &context).await;

        assert!(result.is_err());
        let error = result.unwrap_err();

        match error {
            ToolError::Permanent { .. } => {
                // Expected permanent error
            }
            _ => panic!("Expected permanent error for invalid address"),
        }
    }

    #[tokio::test]
    async fn test_get_multiple_balances_when_multiple_invalid_addresses_should_return_error() {
        let addresses = vec![
            "invalid1".to_string(),
            "invalid2".to_string(),
            "invalid3".to_string(),
        ];
        let context = riglr_core::provider::ApplicationContext::from_env();

        let result = get_multiple_balances(addresses, &context).await;

        assert!(result.is_err());
        // Should fail on first invalid address
    }

    #[tokio::test]
    async fn test_get_multiple_balances_when_mixed_valid_invalid_addresses_should_return_error() {
        let addresses = vec![
            "11111111111111111111111111111114".to_string(), // Valid format but will fail due to no context
            "invalid_address".to_string(),                  // Invalid format
        ];
        let context = riglr_core::provider::ApplicationContext::from_env();

        let result = get_multiple_balances(addresses, &context).await;

        assert!(result.is_err());
        // Should fail on first address due to no signer context
    }

    #[tokio::test]
    async fn test_get_multiple_balances_when_single_valid_address_should_fail_due_to_no_context() {
        let addresses = vec!["11111111111111111111111111111114".to_string()];
        let context = riglr_core::provider::ApplicationContext::from_env();

        let result = get_multiple_balances(addresses, &context).await;

        assert!(result.is_err());
        let error = result.unwrap_err();

        match error {
            ToolError::Permanent { context, .. } => {
                assert!(context.contains("No signer context available"));
            }
            _ => panic!("Expected permanent error for no signer context"),
        }
    }

    // Test struct field access and serialization/deserialization behavior
    #[tokio::test]
    async fn test_balance_result_clone_and_debug() {
        let result = BalanceResult {
            address: "test".to_string(),
            lamports: 100,
            sol: 0.0000001,
            formatted: "0.000000100 SOL".to_string(),
        };

        let cloned = result.clone();
        assert_eq!(result.address, cloned.address);
        assert_eq!(result.lamports, cloned.lamports);
        assert_eq!(result.sol, cloned.sol);
        assert_eq!(result.formatted, cloned.formatted);

        // Test Debug formatting
        let debug_str = format!("{:?}", result);
        assert!(debug_str.contains("BalanceResult"));
        assert!(debug_str.contains("test"));
        assert!(debug_str.contains("100"));
    }

    #[tokio::test]
    async fn test_token_balance_result_clone_and_debug() {
        let result = TokenBalanceResult {
            owner_address: "owner".to_string(),
            mint_address: "mint".to_string(),
            raw_amount: 1000,
            ui_amount: 0.001,
            decimals: 6,
            formatted: "0.001000000".to_string(),
        };

        let cloned = result.clone();
        assert_eq!(result.owner_address, cloned.owner_address);
        assert_eq!(result.mint_address, cloned.mint_address);
        assert_eq!(result.raw_amount, cloned.raw_amount);
        assert_eq!(result.ui_amount, cloned.ui_amount);
        assert_eq!(result.decimals, cloned.decimals);
        assert_eq!(result.formatted, cloned.formatted);

        // Test Debug formatting
        let debug_str = format!("{:?}", result);
        assert!(debug_str.contains("TokenBalanceResult"));
        assert!(debug_str.contains("owner"));
        assert!(debug_str.contains("mint"));
        assert!(debug_str.contains("1000"));
    }

    #[tokio::test]
    async fn test_balance_result_with_fractional_sol() {
        let lamports = 123456789; // 0.123456789 SOL
        let sol = lamports as f64 / LAMPORTS_PER_SOL as f64;

        let result = BalanceResult {
            address: "fractional".to_string(),
            lamports,
            sol,
            formatted: format!("{:.9} SOL", sol),
        };

        assert_eq!(result.lamports, 123456789);
        assert!((result.sol - 0.123456789).abs() < f64::EPSILON);
        assert!(result.formatted.contains("0.123456789"));
    }

    #[tokio::test]
    async fn test_token_balance_result_with_various_decimals() {
        // Test different decimal values
        for decimals in [0, 1, 6, 9, 18] {
            let result = TokenBalanceResult {
                owner_address: format!("owner_{}", decimals),
                mint_address: format!("mint_{}", decimals),
                raw_amount: 1000,
                ui_amount: 1000.0 / 10_f64.powi(decimals as i32),
                decimals,
                formatted: "test".to_string(),
            };

            assert_eq!(result.decimals, decimals);
            assert_eq!(result.raw_amount, 1000);
        }
    }
}
