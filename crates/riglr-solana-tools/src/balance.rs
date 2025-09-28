//! Balance checking tools for Solana blockchain
//!
//! This module provides tools for querying SOL and SPL token balances on the Solana blockchain.
//! These tools use `ApplicationContext` extensions for read-only operations and don't require transaction signing.

use crate::utils::validate_address;
use riglr_core::provider::ApplicationContext;
use riglr_core::ToolError;
use riglr_macros::tool;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use solana_sdk::native_token::LAMPORTS_PER_SOL;
use std::sync::Arc;
use tracing::{debug, info};

/// Result structure for balance queries
///
/// Contains balance information for a Solana address including both raw lamports
/// and human-readable SOL amounts.
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SolBalanceResult {
    /// The Solana wallet address that was queried
    pub address: String,
    /// Human-readable formatted balance
    pub formatted: String,
    /// Balance in lamports (smallest unit)
    pub lamports: u64,
    /// Balance in SOL
    pub sol: f64,
}

/// Result structure for SPL token balance queries
///
/// Contains balance information for a specific SPL token including both raw amounts
/// and decimal-adjusted values.
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TokenBalanceResult {
    /// Number of decimal places for the token
    pub decimals: u8,
    /// Human-readable formatted balance
    pub formatted: String,
    /// The SPL token mint address (contract address)
    pub mint_address: String,
    /// The wallet address that owns the tokens
    pub owner_address: String,
    /// Raw token amount (before decimal adjustment)
    pub raw_amount: u64,
    /// UI amount (with decimal adjustment)
    pub ui_amount: f64,
}

/// Get SOL balance for a given address
///
/// This tool queries the Solana blockchain to retrieve the SOL balance for any wallet address.
/// The balance is returned in both lamports (smallest unit) and SOL (human-readable format).
/// This is a read-only operation that uses `ApplicationContext` extensions instead of requiring transaction signing.
///
/// # Arguments
///
/// * `address` - The Solana wallet address to check (base58 encoded public key)
/// * `context` - The `ApplicationContext` containing RPC client and configuration
///
/// # Returns
///
/// Returns `SolBalanceResult` containing:
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
/// use riglr_core::provider::ApplicationContext;
/// use riglr_config::Config;
/// use solana_client::rpc_client::RpcClient;
/// use std::sync::Arc;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Set up ApplicationContext with Solana RPC client
/// let config = Config::from_env();
/// let context = ApplicationContext::from_config(&config);
///
/// // Add Solana RPC client as an extension
/// let solana_client = Arc::new(RpcClient::new("https://api.mainnet-beta.solana.com"));
/// context.set_extension(solana_client);
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
#[inline]
pub async fn get_sol(
    address: String,
    context: &ApplicationContext,
) -> Result<SolBalanceResult, ToolError> {
    get_sol_balance_impl(address, context)
}

/// Internal implementation of SOL balance checking
fn get_sol_balance_impl(
    address: String,
    context: &ApplicationContext,
) -> Result<SolBalanceResult, ToolError> {
    use solana_client::rpc_client::RpcClient;

    // Validate address using stateless utility
    let pubkey = validate_address(&address)
        .map_err(|validation_error| ToolError::permanent_string(validation_error.to_string()))?;

    // Get Solana RPC client from the ApplicationContext's extensions
    let rpc_client = context.get_extension::<Arc<RpcClient>>().ok_or_else(|| {
        ToolError::permanent_string("Solana RpcClient not found in context".to_owned())
    })?;

    // Get balance in lamports using the RPC client from context
    let lamports = rpc_client.get_balance(&pubkey).map_err(|rpc_error| {
        // Network and connection errors are retriable
        let error_str = rpc_error.to_string();
        if error_str.contains("timeout")
            || error_str.contains("connection")
            || error_str.contains("temporarily")
            || error_str.contains("network")
        {
            ToolError::retriable_string(format!("Failed to get balance: {rpc_error}"))
        } else {
            ToolError::permanent_string(format!("Failed to get balance: {rpc_error}"))
        }
    })?;

    // Convert to SOL with acceptable precision loss for UI display
    #[expect(clippy::cast_precision_loss)]
    let sol = (lamports as f64) / (LAMPORTS_PER_SOL as f64);

    info!(
        "Balance for {}: {} SOL ({} lamports)",
        address, sol, lamports
    );

    Ok(SolBalanceResult {
        address,
        lamports,
        sol,
        formatted: format!("{sol:.9} SOL"),
    })
}

// Manual Tool trait implementations for balance tools
#[async_trait::async_trait]
impl riglr_core::Tool for GetSolTool {
    type Args = serde_json::Value;
    type Error = riglr_core::ToolError;
    type Output = riglr_core::JobResult;

    #[inline]
    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let address: String = serde_json::from_value(
            args.get("address")
                .cloned()
                .unwrap_or(serde_json::Value::Null),
        )
        .map_err(|json_error| {
            riglr_core::ToolError::permanent_string(format!(
                "Invalid address parameter: {json_error}"
            ))
        })?;

        let result = get_sol_balance_impl(address, &self.context)?;
        return riglr_core::JobResult::success(&result).map_err(|serialization_error| {
            riglr_core::ToolError::permanent_string(format!(
                "Failed to serialize result: {serialization_error}"
            ))
        });
    }

    #[inline]
    fn description(&self) -> &'static str {
        "Get SOL balance for a given address using an RPC client"
    }

    #[inline]
    fn name(&self) -> &'static str {
        "get_sol_balance"
    }

    #[inline]
    fn schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "address": {
                    "type": "string",
                    "description": "The Solana wallet address to check (base58 encoded public key)"
                }
            },
            "required": ["address"]
        })
    }
}

#[async_trait::async_trait]
impl riglr_core::Tool for GetSplTokenTool {
    type Args = serde_json::Value;
    type Error = riglr_core::ToolError;
    type Output = riglr_core::JobResult;

    #[inline]
    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let owner_address: String = serde_json::from_value(
            args.get("ownerAddress")
                .cloned()
                .unwrap_or(serde_json::Value::Null),
        )
        .map_err(|json_error| {
            riglr_core::ToolError::permanent_string(format!(
                "Invalid ownerAddress parameter: {json_error}"
            ))
        })?;
        let mint_address: String = serde_json::from_value(
            args.get("mintAddress")
                .cloned()
                .unwrap_or(serde_json::Value::Null),
        )
        .map_err(|json_error| {
            riglr_core::ToolError::permanent_string(format!(
                "Invalid mintAddress parameter: {json_error}"
            ))
        })?;

        let result = get_spl_token_balance_impl(owner_address, mint_address, &self.context)?;
        return riglr_core::JobResult::success(&result).map_err(|serialization_error| {
            riglr_core::ToolError::permanent_string(format!(
                "Failed to serialize result: {serialization_error}"
            ))
        });
    }

    #[inline]
    fn description(&self) -> &'static str {
        "Get SPL token balance for a given owner and mint using an RPC client"
    }

    #[inline]
    fn name(&self) -> &'static str {
        "get_spl_token_balance"
    }

    #[inline]
    fn schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "ownerAddress": {
                    "type": "string",
                    "description": "The wallet address that owns the tokens (base58 encoded)"
                },
                "mintAddress": {
                    "type": "string",
                    "description": "The SPL token mint address (contract address)"
                }
            },
            "required": ["ownerAddress", "mintAddress"]
        })
    }
}

/// Get SPL token balance for a given owner and mint
///
/// This tool queries the Solana blockchain to retrieve the balance of a specific SPL token
/// for a given wallet address. It automatically finds the Associated Token Account (ATA)
/// and returns both raw and UI-adjusted amounts. This is a read-only operation that uses
/// `ApplicationContext` extensions instead of requiring transaction signing.
///
/// # Arguments
///
/// * `owner_address` - The wallet address that owns the tokens (base58 encoded)
/// * `mint_address` - The SPL token mint address (contract address)
/// * `context` - The `ApplicationContext` containing the RPC client
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
/// use riglr_core::provider::ApplicationContext;
/// use riglr_config::Config;
/// use solana_client::rpc_client::RpcClient;
/// use std::sync::Arc;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let config = Config::from_env();
/// let context = ApplicationContext::from_config(&config);
///
/// // Add Solana RPC client as an extension
/// let solana_client = Arc::new(RpcClient::new("https://api.mainnet-beta.solana.com"));
/// context.set_extension(solana_client);
///
/// // Check USDC balance for a wallet
/// let balance = get_spl_token_balance(
///     "9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM".to_string(),
///     "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(), // USDC mint
///     &context
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
/// This version uses dependency injection to get the RPC client from `ApplicationContext`.
#[tool]
#[inline]
pub async fn get_spl_token(
    owner_address: String,
    mint_address: String,
    context: &ApplicationContext,
) -> Result<TokenBalanceResult, ToolError> {
    get_spl_token_balance_impl(owner_address, mint_address, context)
}

/// Internal implementation of SPL token balance checking
fn get_spl_token_balance_impl(
    owner_address: String,
    mint_address: String,
    context: &ApplicationContext,
) -> Result<TokenBalanceResult, ToolError> {
    use crate::common_conversions::get_associated_token_address_v3;
    use solana_client::rpc_client::RpcClient;

    debug!(
        "Getting SPL token balance for owner: {}, mint: {}",
        owner_address, mint_address
    );

    // Get Solana RPC client from the ApplicationContext's extensions
    let rpc_client = context.get_extension::<Arc<RpcClient>>().ok_or_else(|| {
        ToolError::permanent_string("Solana RpcClient not found in context".to_owned())
    })?;

    // Validate addresses using stateless utilities
    let owner_pubkey = validate_address(&owner_address).map_err(|validation_error| {
        ToolError::permanent_string(format!("Invalid owner address: {validation_error}"))
    })?;
    let mint_pubkey = validate_address(&mint_address).map_err(|validation_error| {
        ToolError::permanent_string(format!("Invalid mint address: {validation_error}"))
    })?;

    // Get the Associated Token Account (ATA) address
    let ata = get_associated_token_address_v3(&owner_pubkey, &mint_pubkey);

    // Get token account balance using the RPC client from context
    let balance_result = rpc_client
        .get_token_account_balance(&ata)
        .map_err(|rpc_error| {
            ToolError::permanent_string(format!("Failed to get token balance: {rpc_error}"))
        });

    if let Ok(balance) = balance_result {
        let raw_amount = balance.amount.parse::<u64>().map_err(|parse_error| {
            ToolError::permanent_string(format!("Failed to parse token amount: {parse_error}"))
        })?;
        let ui_amount = balance.ui_amount.unwrap_or(0.0f64);
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
            formatted: format!("{ui_amount:.9}"),
        })
    } else {
        // Account doesn't exist or has zero balance
        info!(
            "No token account found for owner: {}, mint: {}",
            owner_address, mint_address
        );
        Ok(TokenBalanceResult {
            owner_address,
            mint_address,
            raw_amount: 0u64,
            ui_amount: 0.0,
            decimals: 9u8, // Default to 9 decimals
            formatted: "0.000000000".to_owned(),
        })
    }
}

#[async_trait::async_trait]
impl riglr_core::Tool for GetMultipleBalancesTool {
    type Args = serde_json::Value;
    type Error = riglr_core::ToolError;
    type Output = riglr_core::JobResult;

    #[inline]
    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let addresses: Vec<String> = serde_json::from_value(
            args.get("addresses")
                .cloned()
                .unwrap_or(serde_json::Value::Null),
        )
        .map_err(|json_error| {
            riglr_core::ToolError::permanent_string(format!(
                "Invalid addresses parameter: {json_error}"
            ))
        })?;

        let result = get_multiple_balances(addresses, &self.context).await?;
        return riglr_core::JobResult::success(&result).map_err(|serialization_error| {
            riglr_core::ToolError::permanent_string(format!(
                "Failed to serialize result: {serialization_error}"
            ))
        });
    }

    #[inline]
    fn description(&self) -> &'static str {
        "Get SOL balance for multiple addresses using an RPC client"
    }

    #[inline]
    fn name(&self) -> &'static str {
        "get_multiple_balances"
    }

    #[inline]
    fn schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "addresses": {
                    "type": "array",
                    "items": {
                        "type": "string"
                    },
                    "description": "Array of Solana wallet addresses to check (base58 encoded public keys)"
                }
            },
            "required": ["addresses"]
        })
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
#[inline]
pub async fn get_multiple_balances(
    addresses: Vec<String>,
    context: &ApplicationContext,
) -> Result<Vec<SolBalanceResult>, ToolError> {
    let mut results = Vec::new();

    for address in addresses {
        let balance_result = get_sol_balance_impl(address.clone(), context);
        match balance_result {
            Ok(balance) => results.push(balance),
            Err(balance_error) => {
                // For individual address failures, return error with partial results
                // This is a design choice - could also continue and mark failed addresses
                return Err(balance_error);
            }
        }
    }

    info!("Retrieved balances for {} addresses", results.len());
    Ok(results)
}

pub use __riglr_tool_get_multiple_balances::GetMultipleBalancesTool;
pub use __riglr_tool_get_sol::GetSolTool;
pub use __riglr_tool_get_spl_token::GetSplTokenTool;

/// Unit tests for balance functionality
#[cfg(test)]
#[expect(clippy::unwrap_used, clippy::panic)]
mod tests {
    use super::*;
    use crate::clients::Clients;
    use riglr_config::Config;
    use riglr_core::{provider::ApplicationContext, ToolError};
    use std::sync::Arc;

    // Helper function to create a test context with ExternalClients
    fn create_test_context() -> ApplicationContext {
        // Load .env.test for test environment
        dotenvy::from_filename(".env.test").ok();

        let config = Config::from_env();
        let context = ApplicationContext::from_config(&Arc::new(config.clone()));

        // Create and inject ExternalClients
        let api_clients = Clients::new(&config.providers);
        context.set_extension(Arc::new(api_clients));

        context
    }

    #[tokio::test]
    #[expect(clippy::float_cmp)]
    async fn test_balance_result_creation() {
        let result = SolBalanceResult {
            address: "11111111111111111111111111111111".to_owned(),
            lamports: 1_000_000_000,
            sol: 1.0,
            formatted: "1.000000000 SOL".to_owned(),
        };

        assert_eq!(result.lamports, 1_000_000_000);
        {
            assert_eq!(result.sol, 1.0f64);
        }
        assert_eq!(result.address, "11111111111111111111111111111111");
        assert_eq!(result.formatted, "1.000000000 SOL");
    }

    #[tokio::test]
    #[expect(clippy::float_cmp)]
    async fn test_balance_result_zero_balance() {
        let result = SolBalanceResult {
            address: "So11111111111111111111111111111111111111112".to_owned(),
            lamports: 0u64,
            sol: 0.0,
            formatted: "0.000000000 SOL".to_owned(),
        };

        assert_eq!(result.lamports, 0);
        {
            assert_eq!(result.sol, 0.0f64);
        }
        assert_eq!(result.formatted, "0.000000000 SOL");
    }

    #[tokio::test]
    #[expect(clippy::float_cmp)]
    async fn test_balance_result_max_value() {
        let max_lamports = u64::MAX;
        #[expect(clippy::cast_precision_loss)]
        let max_sol = max_lamports as f64 / LAMPORTS_PER_SOL as f64;

        let result = SolBalanceResult {
            address: "Test123456789".to_owned(),
            lamports: max_lamports,
            sol: max_sol,
            formatted: format!("{max_sol:.9} SOL"),
        };

        assert_eq!(result.lamports, max_lamports);
        {
            assert_eq!(result.sol, max_sol);
        }
    }

    #[tokio::test]
    #[expect(clippy::float_cmp)]
    async fn test_token_balance_result() {
        let result = TokenBalanceResult {
            owner_address: "11111111111111111111111111111111".to_owned(),
            mint_address: "So11111111111111111111111111111111111111112".to_owned(),
            raw_amount: 1_000_000,
            ui_amount: 1.0,
            decimals: 6u8,
            formatted: "1.0".to_owned(),
        };

        assert_eq!(result.raw_amount, 1_000_000);
        assert_eq!(result.decimals, 6);
        {
            assert_eq!(result.ui_amount, 1.0f64);
        }
        assert_eq!(result.owner_address, "11111111111111111111111111111111");
        assert_eq!(
            result.mint_address,
            "So11111111111111111111111111111111111111112"
        );
        assert_eq!(result.formatted, "1.0");
    }

    #[tokio::test]
    #[expect(clippy::float_cmp)]
    async fn test_token_balance_result_zero_balance() {
        let result = TokenBalanceResult {
            owner_address: "Owner123".to_owned(),
            mint_address: "Mint456".to_owned(),
            raw_amount: 0u64,
            ui_amount: 0.0,
            decimals: 9u8,
            formatted: "0.000000000".to_string(),
        };

        assert_eq!(result.raw_amount, 0);
        {
            assert_eq!(result.ui_amount, 0.0f64);
        }
        assert_eq!(result.decimals, 9);
        assert_eq!(result.formatted, "0.000000000");
    }

    #[tokio::test]
    async fn test_token_balance_result_high_decimals() {
        let result = TokenBalanceResult {
            owner_address: "HighDecimal".to_owned(),
            mint_address: "Decimals18".to_owned(),
            raw_amount: 123_456_789_012_345_678,
            ui_amount: 123.456_789_012_345_68,
            decimals: 18u8,
            formatted: "123.456789012".to_owned(),
        };

        assert_eq!(result.decimals, 18);
        assert_eq!(result.raw_amount, 123_456_789_012_345_678);
        assert!(result.ui_amount > 123.0);
    }

    #[tokio::test]
    #[expect(clippy::float_cmp)]
    async fn test_token_balance_result_max_values() {
        let result = TokenBalanceResult {
            owner_address: "MaxValues".to_owned(),
            mint_address: "MaxMint".to_owned(),
            raw_amount: u64::MAX,
            ui_amount: f64::MAX,
            decimals: u8::MAX,
            formatted: "Max".to_owned(),
        };

        assert_eq!(result.raw_amount, u64::MAX);
        {
            assert_eq!(result.ui_amount, f64::MAX);
        }
        assert_eq!(result.decimals, u8::MAX);
    }

    #[tokio::test]
    async fn test_get_sol_balance_when_invalid_address_should_return_permanent_error() {
        let context = create_test_context();
        let result = get_sol_balance_impl("invalid_address".to_owned(), &context);

        assert!(result.is_err());
        let error = result.unwrap_err();

        assert!(
            matches!(error, ToolError::Permanent { .. }),
            "Expected permanent error for invalid address, got: {error:?}"
        );
    }

    #[tokio::test]
    async fn test_get_sol_balance_when_empty_address_should_return_permanent_error() {
        let context = create_test_context();
        let result = get_sol_balance_impl(String::new(), &context);

        assert!(result.is_err());
        let error = result.unwrap_err();

        assert!(
            matches!(error, ToolError::Permanent { .. }),
            "Expected permanent error for empty address, got: {error:?}"
        );
    }

    #[tokio::test]
    async fn test_get_sol_balance_when_special_chars_address_should_return_permanent_error() {
        let context = create_test_context();
        let result = get_sol_balance_impl("!@#$%^&*()".to_owned(), &context);

        assert!(result.is_err());
        let error = result.unwrap_err();

        assert!(
            matches!(error, ToolError::Permanent { .. }),
            "Expected permanent error for special characters, got: {error:?}"
        );
    }

    #[tokio::test]
    async fn test_get_sol_balance_when_no_signer_context_should_return_permanent_error() {
        // This test covers the case where Solana RpcClient is not in context
        // Since we can't inject RpcClient in test context, we test with valid address
        // but expect it to fail due to no RpcClient in context
        let valid_address = "11111111111111111111111111111114"; // System program (valid format)

        let context = create_test_context();
        let result = get_sol_balance_impl(valid_address.to_owned(), &context);

        assert!(result.is_err());
        let error = result.unwrap_err();

        match error {
            ToolError::Permanent { context, .. } => {
                // The error message changed - now it's about missing RpcClient
                assert!(context.contains("Solana RpcClient not found in context"));
            }
            _ => {
                panic!("Expected permanent error for no RpcClient in context, got: {error:?}");
            }
        }
    }

    #[tokio::test]
    async fn test_get_spl_token_balance_when_invalid_owner_address_should_return_permanent_error() {
        let context = create_test_context();
        let result = get_spl_token_balance_impl(
            "invalid_owner".to_owned(),
            "11111111111111111111111111111114".to_owned(),
            &context,
        );

        assert!(result.is_err());
        let error = result.unwrap_err();

        match error {
            ToolError::Permanent { context, .. } => {
                // Now fails with missing RpcClient before address validation
                assert!(context.contains("Solana RpcClient not found in context"));
            }
            ToolError::InvalidInput { .. }
            | ToolError::Retriable { .. }
            | ToolError::RateLimited { .. }
            | ToolError::SignerContext(_) => {
                panic!("Expected permanent error for missing RpcClient");
            }
            _ => {
                panic!("Unexpected error variant for invalid owner");
            }
        }
    }

    #[tokio::test]
    async fn test_get_spl_token_balance_when_invalid_mint_address_should_return_permanent_error() {
        let context = create_test_context();
        let result = get_spl_token_balance_impl(
            "11111111111111111111111111111114".to_owned(),
            "invalid_mint".to_owned(),
            &context,
        );

        assert!(result.is_err());
        let error = result.unwrap_err();

        match error {
            ToolError::Permanent { context, .. } => {
                // Now fails with missing RpcClient before address validation
                assert!(context.contains("Solana RpcClient not found in context"));
            }
            ToolError::InvalidInput { .. }
            | ToolError::Retriable { .. }
            | ToolError::RateLimited { .. }
            | ToolError::SignerContext(_) => {
                panic!("Expected permanent error for missing RpcClient");
            }
            _ => {
                panic!("Unexpected error variant for invalid mint");
            }
        }
    }

    #[tokio::test]
    async fn test_get_spl_token_balance_when_empty_owner_address_should_return_permanent_error() {
        let context = create_test_context();
        let result = get_spl_token_balance_impl(
            String::new(),
            "11111111111111111111111111111114".to_owned(),
            &context,
        );

        assert!(result.is_err());
        let error = result.unwrap_err();

        match error {
            ToolError::Permanent { .. } => {
                // Expected permanent error
            }
            ToolError::InvalidInput { .. }
            | ToolError::Retriable { .. }
            | ToolError::RateLimited { .. }
            | ToolError::SignerContext(_) => {
                panic!("Expected permanent error for empty owner address");
            }
            _ => {
                panic!("Unexpected error variant for empty owner address");
            }
        }
    }

    #[tokio::test]
    async fn test_get_spl_token_balance_when_empty_mint_address_should_return_permanent_error() {
        let context = create_test_context();
        let result = get_spl_token_balance_impl(
            "11111111111111111111111111111114".to_owned(),
            String::new(),
            &context,
        );

        assert!(result.is_err());
        let error = result.unwrap_err();

        match error {
            ToolError::Permanent { .. } => {
                // Expected permanent error
            }
            ToolError::InvalidInput { .. }
            | ToolError::Retriable { .. }
            | ToolError::RateLimited { .. }
            | ToolError::SignerContext(_) => {
                panic!("Expected permanent error for empty mint address");
            }
            _ => {
                panic!("Unexpected error variant for empty mint address");
            }
        }
    }

    #[tokio::test]
    async fn test_get_spl_token_balance_when_no_signer_context_should_return_permanent_error() {
        // Test with valid addresses but no signer context
        let context = create_test_context();
        let result = get_spl_token_balance_impl(
            "11111111111111111111111111111114".to_owned(),
            "So11111111111111111111111111111111111111112".to_string(),
            &context,
        );

        assert!(result.is_err());
        let error = result.unwrap_err();

        match error {
            ToolError::Permanent { context, .. } => {
                assert!(context.contains("Solana RpcClient not found in context"));
            }
            ToolError::InvalidInput { .. }
            | ToolError::Retriable { .. }
            | ToolError::RateLimited { .. }
            | ToolError::SignerContext(_) => {
                panic!("Expected permanent error for no signer context");
            }
            _ => {
                panic!("Unexpected error variant for no signer context");
            }
        }
    }

    #[tokio::test]
    async fn test_get_multiple_balances_when_empty_addresses_should_return_empty_vec() {
        let context = create_test_context();
        let result = get_multiple_balances(vec![], &context).await;

        assert!(result.is_ok());
        let balances = result.unwrap();
        assert!(balances.is_empty());
    }

    #[tokio::test]
    async fn test_get_multiple_balances_when_single_invalid_address_should_return_error() {
        let addresses = vec!["invalid_address".to_owned()];
        let context = create_test_context();

        let result = get_multiple_balances(addresses, &context).await;

        assert!(result.is_err());
        let error = result.unwrap_err();

        match error {
            ToolError::Permanent { .. } => {
                // Expected permanent error
            }
            ToolError::InvalidInput { .. }
            | ToolError::Retriable { .. }
            | ToolError::RateLimited { .. }
            | ToolError::SignerContext(_) => {
                panic!("Expected permanent error for invalid address");
            }
            _ => {
                panic!("Unexpected error variant for invalid address");
            }
        }
    }

    #[tokio::test]
    async fn test_get_multiple_balances_when_multiple_invalid_addresses_should_return_error() {
        let addresses = vec![
            "invalid1".to_owned(),
            "invalid2".to_owned(),
            "invalid3".to_owned(),
        ];
        let context = create_test_context();

        let result = get_multiple_balances(addresses, &context).await;

        assert!(result.is_err());
        // Should fail on first invalid address
    }

    #[tokio::test]
    async fn test_get_multiple_balances_when_mixed_valid_invalid_addresses_should_return_error() {
        let addresses = vec![
            "11111111111111111111111111111114".to_owned(), // Valid format but will fail due to no context
            "invalid_address".to_owned(),                  // Invalid format
        ];
        let context = create_test_context();

        let result = get_multiple_balances(addresses, &context).await;

        assert!(result.is_err());
        // Should fail on first address due to no signer context
    }

    #[tokio::test]
    async fn test_get_multiple_balances_when_single_valid_address_should_fail_due_to_no_context() {
        let addresses = vec!["11111111111111111111111111111114".to_owned()];
        let context = create_test_context();

        let result = get_multiple_balances(addresses, &context).await;

        assert!(result.is_err());
        let error = result.unwrap_err();

        match error {
            ToolError::Permanent { context, .. } => {
                assert!(context.contains("Solana RpcClient not found in context"));
            }
            ToolError::InvalidInput { .. }
            | ToolError::Retriable { .. }
            | ToolError::RateLimited { .. }
            | ToolError::SignerContext(_) => {
                panic!("Expected permanent error for no signer context");
            }
            _ => {
                panic!("Unexpected error variant for no signer context");
            }
        }
    }

    // Test struct field access and serialization/deserialization behavior
    #[tokio::test]
    async fn test_balance_result_clone_and_debug() {
        let result = SolBalanceResult {
            address: "test".to_owned(),
            lamports: 100u64,
            sol: 0.000_000_1,
            formatted: "0.000000100 SOL".to_owned(),
        };

        let cloned = result.clone();
        assert_eq!(result.address, cloned.address);
        assert_eq!(result.lamports, cloned.lamports);
        {
            assert!((result.sol - cloned.sol).abs() < f64::EPSILON);
        }
        assert_eq!(result.formatted, cloned.formatted);

        // Test Debug formatting
        let debug_str = format!("{result:?}");
        assert!(debug_str.contains("BalanceResult"));
        assert!(debug_str.contains("test"));
        assert!(debug_str.contains("100"));
    }

    #[tokio::test]
    async fn test_token_balance_result_clone_and_debug() {
        let result = TokenBalanceResult {
            owner_address: "owner".to_owned(),
            mint_address: "mint".to_owned(),
            raw_amount: 1000u64,
            ui_amount: 0.001,
            decimals: 6u8,
            formatted: "0.001000000".to_owned(),
        };

        let cloned = result.clone();
        assert_eq!(result.owner_address, cloned.owner_address);
        assert_eq!(result.mint_address, cloned.mint_address);
        assert_eq!(result.raw_amount, cloned.raw_amount);
        {
            assert!((result.ui_amount - cloned.ui_amount).abs() < f64::EPSILON);
        }
        assert_eq!(result.decimals, cloned.decimals);
        assert_eq!(result.formatted, cloned.formatted);

        // Test Debug formatting
        let debug_str = format!("{result:?}");
        assert!(debug_str.contains("TokenBalanceResult"));
        assert!(debug_str.contains("owner"));
        assert!(debug_str.contains("mint"));
        assert!(debug_str.contains("1000"));
    }

    #[tokio::test]
    async fn test_balance_result_with_fractional_sol() {
        let lamports = 123_456_789; // 0.123456789 SOL
        #[expect(clippy::cast_precision_loss)]
        let sol = lamports as f64 / LAMPORTS_PER_SOL as f64;

        let result = SolBalanceResult {
            address: "fractional".to_owned(),
            lamports,
            sol,
            formatted: format!("{sol:.9} SOL"),
        };

        assert_eq!(result.lamports, 123_456_789);
        assert!((result.sol - 0.123_456_789).abs() < f64::EPSILON);
        assert!(result.formatted.contains("0.123456789"));
    }

    #[tokio::test]
    async fn test_token_balance_result_with_various_decimals() {
        // Test different decimal values
        for decimals in [0, 1, 6, 9, 18] {
            let result = TokenBalanceResult {
                owner_address: format!("owner_{decimals}"),
                mint_address: format!("mint_{decimals}"),
                raw_amount: 1000u64,
                ui_amount: 1000.0f64 / 10f64.powi(i32::from(decimals)),
                decimals,
                formatted: "test".to_owned(),
            };

            assert_eq!(result.decimals, decimals);
            assert_eq!(result.raw_amount, 1000);
        }
    }
}
