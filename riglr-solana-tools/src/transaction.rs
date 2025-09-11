//! Transaction tools for Solana blockchain
//!
//! This module provides tools for creating and executing transactions on the Solana blockchain.
//! All state-mutating operations are queued through the job system for resilience.

use crate::common::{
    create_associated_token_account_idempotent_v3, from_spl_token_pubkey,
    get_associated_token_address_v3, initialize_mint2_v3, mint_to_v3, spl_token_transfer_v3,
    system_create_account_v3, system_transfer_v3,
};
use crate::utils::send_transaction;
use riglr_core::{SignerContext, ToolError};
use riglr_macros::tool;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use solana_compute_budget_interface::ComputeBudgetInstruction;
use solana_sdk::signer::Signer;
#[allow(deprecated)]
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    native_token::LAMPORTS_PER_SOL,
    pubkey::Pubkey,
    transaction::Transaction,
};
use spl_token;
use std::str::FromStr;
use std::sync::Arc;
use tracing::{debug, info};

// Memo program ID constant
const MEMO_PROGRAM_ID: &str = "MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr";

/// Transfer SOL from one account to another
///
/// This tool creates, signs, and executes a SOL transfer transaction using the current signer context.
/// The transaction includes optional memo and priority fee support for faster confirmation.
///
/// # Arguments
///
/// * `to_address` - Recipient wallet address (base58 encoded public key)
/// * `amount_sol` - Amount to transfer in SOL (e.g., 0.001 for 1,000,000 lamports)
/// * `memo` - Optional memo to include in the transaction for record keeping
/// * `priority_fee` - Optional priority fee in microlamports for faster processing
///
/// # Returns
///
/// Returns `TransactionResult` containing:
/// - `signature`: Transaction signature hash
/// - `from`: Sender address from signer context
/// - `to`: Recipient address
/// - `amount`: Transfer amount in lamports
/// - `amount_display`: Human-readable amount in SOL
/// - `status`: Transaction confirmation status
/// - `memo`: Included memo (if any)
///
/// # Errors
///
/// * `ToolError::Permanent` - When amount is non-positive, addresses are invalid, or signer unavailable
/// * `ToolError::Permanent` - When transaction signing or submission fails
///
/// # Examples
///
/// ```rust,ignore
/// use riglr_solana_tools::transaction::transfer_sol;
/// use riglr_core::SignerContext;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Transfer 0.001 SOL with a memo
/// let result = transfer_sol(
///     "9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM".to_string(),
///     0.001, // 0.001 SOL
///     Some("Payment for services".to_string()),
///     Some(5000), // 5000 microlamports priority fee
/// ).await?;
///
/// println!("Transfer completed! Signature: {}", result.signature);
/// println!("Sent {} from {} to {}", result.amount_display, result.from, result.to);
/// # Ok(())
/// # }
/// ```
#[tool]
pub async fn transfer_sol(
    to_address: String,
    amount_sol: f64,
    memo: Option<String>,
    priority_fee: Option<u64>,
    _context: &riglr_core::provider::ApplicationContext,
) -> Result<TransactionResult, ToolError> {
    debug!(
        "Initiating SOL transfer of {} SOL to {}",
        amount_sol, to_address
    );

    // Validate inputs
    if amount_sol <= 0.0 {
        return Err(ToolError::permanent_string("Amount must be positive"));
    }

    let to_pubkey = Pubkey::from_str(&to_address)
        .map_err(|e| ToolError::permanent_string(format!("Invalid recipient address: {}", e)))?;

    // Convert SOL to lamports
    let lamports = (amount_sol * LAMPORTS_PER_SOL as f64) as u64;

    // Get signer from context
    let signer_context = SignerContext::current_as_solana()
        .await
        .map_err(|e| ToolError::permanent_string(format!("No Solana signer context: {}", e)))?;

    let from_pubkey = signer_context
        .pubkey()
        .parse::<Pubkey>()
        .map_err(|e| ToolError::permanent_string(format!("Invalid signer pubkey: {}", e)))?;

    // Create transfer instruction
    let mut instructions = vec![system_transfer_v3(&from_pubkey, &to_pubkey, lamports)];

    // Add priority fee if specified
    if let Some(fee) = priority_fee {
        instructions.insert(0, ComputeBudgetInstruction::set_compute_unit_price(fee));
    }

    // Add memo if provided
    if let Some(memo_text) = &memo {
        let memo_program_id = Pubkey::from_str(MEMO_PROGRAM_ID)
            .map_err(|e| ToolError::invalid_input_with_source(e, "Invalid memo program ID"))?;
        let memo_ix = Instruction::new_with_bytes(
            memo_program_id,
            memo_text.as_bytes(),
            vec![AccountMeta::new(from_pubkey, true)],
        );
        instructions.push(memo_ix);
    }

    // Create and send transaction with retry logic
    let mut transaction = Transaction::new_with_payer(&instructions, Some(&from_pubkey));
    let signature = send_transaction(
        &mut transaction,
        &format!("SOL Transfer ({} SOL)", amount_sol),
    )
    .await?;

    info!(
        "SOL transfer initiated: {} -> {} ({} SOL), signature: {}",
        from_pubkey, to_address, amount_sol, signature
    );

    Ok(TransactionResult {
        signature,
        from: from_pubkey.to_string(),
        to: to_address,
        amount: lamports,
        amount_display: format!("{} SOL", amount_sol),
        status: TransactionStatus::Pending,
        memo,
        idempotency_key: None,
    })
}

/// Transfer SPL tokens from one account to another
///
/// This tool creates, signs, and executes an SPL token transfer transaction. It automatically
/// handles Associated Token Account (ATA) creation if needed and supports any SPL token.
///
/// # Arguments
///
/// * `to_address` - Recipient wallet address (base58 encoded public key)
/// * `mint_address` - SPL token mint address (contract address)
/// * `amount` - Amount to transfer in token's smallest unit (before decimal adjustment)
/// * `decimals` - Number of decimal places for the token (e.g., 6 for USDC, 9 for most tokens)
/// * `create_ata_if_needed` - Whether to create recipient's ATA if it doesn't exist
///
/// # Returns
///
/// Returns `TokenTransferResult` containing transaction details and amount information
/// in both raw and UI-adjusted formats.
///
/// # Errors
///
/// * `ToolError::Permanent` - When addresses are invalid, signer unavailable, or transaction fails
///
/// # Examples
///
/// ```rust,ignore
/// use riglr_solana_tools::transaction::transfer_spl_token;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Transfer 1 USDC (6 decimals)
/// let result = transfer_spl_token(
///     "9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM".to_string(),
///     "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(), // USDC mint
///     1_000_000, // 1 USDC in microunits
///     6, // USDC has 6 decimals
///     true, // Create recipient ATA if needed
/// ).await?;
///
/// println!("Token transfer completed! Signature: {}", result.signature);
/// println!("Transferred {} tokens", result.ui_amount);
/// # Ok(())
/// # }
/// ```
#[tool]
pub async fn transfer_spl_token(
    to_address: String,
    mint_address: String,
    amount: u64,
    decimals: u8,
    create_ata_if_needed: bool,
    _context: &riglr_core::provider::ApplicationContext,
) -> Result<TokenTransferResult, ToolError> {
    debug!(
        "Initiating SPL token transfer of {} to {}",
        amount, to_address
    );

    // Validate inputs
    let to_pubkey = Pubkey::from_str(&to_address)
        .map_err(|e| ToolError::permanent_string(format!("Invalid recipient address: {}", e)))?;
    let mint_pubkey = Pubkey::from_str(&mint_address)
        .map_err(|e| ToolError::permanent_string(format!("Invalid mint address: {}", e)))?;

    // Get signer from context
    let signer_context = SignerContext::current_as_solana()
        .await
        .map_err(|e| ToolError::permanent_string(format!("No Solana signer context: {}", e)))?;

    let from_pubkey = signer_context
        .pubkey()
        .parse::<Pubkey>()
        .map_err(|e| ToolError::permanent_string(format!("Invalid signer pubkey: {}", e)))?;

    // Get associated token accounts
    let from_ata = get_associated_token_address_v3(&from_pubkey, &mint_pubkey);
    let to_ata = get_associated_token_address_v3(&to_pubkey, &mint_pubkey);

    let mut instructions = vec![];

    // Check if recipient ATA exists and create if needed
    if create_ata_if_needed {
        // The create_associated_token_account_idempotent instruction is safe to include
        // even if the account already exists
        let spl_token_id = from_spl_token_pubkey(&spl_token::id());
        instructions.push(create_associated_token_account_idempotent_v3(
            &from_pubkey,
            &to_pubkey,
            &mint_pubkey,
            &spl_token_id,
        ));
    }

    // Create transfer instruction
    let spl_token_id = from_spl_token_pubkey(&spl_token::id());
    instructions.push(
        spl_token_transfer_v3(&spl_token_id, &from_ata, &to_ata, &from_pubkey, &[], amount)
            .map_err(|e| {
                ToolError::permanent_string(format!("Failed to create transfer instruction: {}", e))
            })?,
    );

    // Create and send transaction with retry logic
    let mut transaction = Transaction::new_with_payer(&instructions, Some(&from_pubkey));
    let ui_amount = amount as f64 / 10_f64.powi(decimals as i32);
    let signature = send_transaction(
        &mut transaction,
        &format!("SPL Token Transfer ({:.9} tokens)", ui_amount),
    )
    .await?;

    info!(
        "SPL token transfer initiated: {} -> {} ({} tokens), signature: {}",
        from_pubkey, to_address, ui_amount, signature
    );

    Ok(TokenTransferResult {
        signature,
        from: from_pubkey.to_string(),
        to: to_address,
        mint: mint_address,
        amount,
        ui_amount,
        decimals,
        amount_display: format!("{:.9}", ui_amount),
        status: TransactionStatus::Pending,
        idempotency_key: None,
    })
}

/// Create a new SPL token mint
///
/// This tool creates a new SPL token mint account on the Solana blockchain with specified
/// configuration parameters. The mint authority is set to the current signer, enabling
/// future token supply management operations.
///
/// This function creates the mint account, sets up the mint and freeze authorities
/// (if applicable), and optionally mints an initial supply of tokens to the creator's
/// associated token account.
///
/// # Arguments
///
/// * `decimals` - Number of decimal places for the token (0-9, commonly 6 or 9)
/// * `initial_supply` - Initial number of tokens to mint (in smallest unit)
/// * `freezable` - Whether the token accounts can be frozen by the freeze authority
///
/// # Returns
///
/// Returns `CreateMintResult` containing:
/// - `mint_address`: The newly created token mint address
/// - `transaction_signature`: Transaction signature of the mint creation
/// - `initial_supply`: Number of tokens initially minted
/// - `decimals`: Decimal places configuration
/// - `authority`: The mint authority address (signer address)
///
/// # Errors
///
/// Returns `ToolError` in the following cases:
/// * `ToolError::Permanent` - Invalid decimals value (must be 0-9)
/// * `ToolError::Retriable` - Network connection issues, insufficient SOL balance
/// * `ToolError::Permanent` - Signer context not available
///
/// # Examples
///
/// ```rust,ignore
/// use riglr_solana_tools::transaction::create_spl_token_mint;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // This would create a USDC-like token with 6 decimals
/// let result = create_spl_token_mint(
///     6,             // 6 decimal places
///     1_000_000_000, // 1,000 tokens initial supply (1000 * 10^6)
///     true,          // Allow freezing accounts
/// ).await?;
///
/// println!("Created token mint: {}", result.mint_address);
/// println!("Initial supply: {} tokens", result.initial_supply);
/// println!("Transaction: {}", result.transaction_signature);
/// # Ok(())
/// # }
/// ```
#[tool]
pub async fn create_spl_token_mint(
    decimals: u8,
    initial_supply: u64,
    freezable: bool,
    context: &riglr_core::provider::ApplicationContext,
) -> Result<CreateMintResult, ToolError> {
    debug!(
        "Creating SPL token mint with {} decimals, {} initial supply, freezable: {}",
        decimals, initial_supply, freezable
    );

    // Validate inputs
    if decimals > 9 {
        return Err(ToolError::permanent_string(
            "Decimals must be between 0 and 9",
        ));
    }

    // Get signer from context
    let signer_context = SignerContext::current_as_solana()
        .await
        .map_err(|e| ToolError::permanent_string(format!("No Solana signer context: {}", e)))?;

    let payer_pubkey_str = signer_context.pubkey();
    let payer_pubkey = Pubkey::from_str(&payer_pubkey_str)
        .map_err(|e| ToolError::permanent_string(format!("Invalid pubkey format: {}", e)))?;

    // Generate a new mint keypair
    let mint_keypair = solana_sdk::signature::Keypair::new();
    let mint_pubkey = mint_keypair.pubkey();

    // Get RPC client from ApplicationContext
    let client = context
        .get_extension::<Arc<solana_client::rpc_client::RpcClient>>()
        .ok_or_else(|| ToolError::permanent_string("Solana RPC client not found in context"))?;

    // Calculate rent for mint account
    let mint_account_size = std::mem::size_of::<spl_token::state::Mint>();
    let rent = client
        .get_minimum_balance_for_rent_exemption(mint_account_size)
        .map_err(|e| ToolError::permanent_string(format!("Failed to get rent: {}", e)))?;

    // Get recent blockhash
    let blockhash = client
        .get_latest_blockhash()
        .map_err(|e| ToolError::retriable_string(format!("Failed to get blockhash: {}", e)))?;

    // Determine freeze authority
    let freeze_authority = if freezable { Some(payer_pubkey) } else { None };

    // Create instructions for:
    // 1. Create mint account
    // 2. Initialize mint
    // 3. Create associated token account for the payer
    // 4. Mint initial supply (if any)
    let mut instructions = vec![];

    // Create the mint account
    let spl_token_id = from_spl_token_pubkey(&spl_token::id());
    instructions.push(system_create_account_v3(
        &payer_pubkey,
        &mint_pubkey,
        rent,
        mint_account_size as u64,
        &spl_token_id,
    ));

    // Initialize the mint
    instructions.push(
        initialize_mint2_v3(
            &spl_token_id,
            &mint_pubkey,
            &payer_pubkey, // mint authority
            freeze_authority.as_ref(),
            decimals,
        )
        .map_err(|e| {
            ToolError::permanent_string(format!(
                "Failed to create initialize mint instruction: {}",
                e
            ))
        })?,
    );

    // If there's initial supply, create ATA and mint tokens
    if initial_supply > 0 {
        // Get or create associated token account for the payer
        let ata = get_associated_token_address_v3(&payer_pubkey, &mint_pubkey);

        // Create ATA instruction
        instructions.push(create_associated_token_account_idempotent_v3(
            &payer_pubkey,
            &payer_pubkey,
            &mint_pubkey,
            &spl_token_id,
        ));

        // Mint initial supply to ATA
        instructions.push(
            mint_to_v3(
                &spl_token_id,
                &mint_pubkey,
                &ata,
                &payer_pubkey,
                &[],
                initial_supply,
            )
            .map_err(|e| {
                ToolError::permanent_string(format!("Failed to create mint instruction: {}", e))
            })?,
        );
    }

    // Create and sign transaction
    let mut transaction = Transaction::new_with_payer(&instructions, Some(&payer_pubkey));
    transaction.message.recent_blockhash = blockhash;

    // Sign with the mint keypair (additional signer)
    transaction.sign(&[&mint_keypair], blockhash);

    // Send transaction through signer context
    // Serialize transaction to bytes
    let mut tx_bytes = bincode::serialize(&transaction).map_err(|e| {
        ToolError::permanent_string(format!("Failed to serialize transaction: {}", e))
    })?;
    let signature = signer_context
        .sign_and_send_transaction(&mut tx_bytes)
        .await
        .map_err(|e| ToolError::permanent_string(format!("Failed to send transaction: {}", e)))?;

    info!(
        "Created SPL token mint {} with signature {}",
        mint_pubkey, signature
    );

    Ok(CreateMintResult {
        signature: signature.to_string(),
        mint_address: mint_pubkey.to_string(),
        authority: payer_pubkey.to_string(),
        decimals,
        initial_supply,
        freezable,
    })
}

/// Result of a SOL transfer transaction
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TransactionResult {
    /// Transaction signature
    pub signature: String,
    /// Sender address
    pub from: String,
    /// Recipient address
    pub to: String,
    /// Amount transferred in lamports
    pub amount: u64,
    /// Human-readable amount display
    pub amount_display: String,
    /// Transaction status
    pub status: TransactionStatus,
    /// Optional memo included with the transaction
    pub memo: Option<String>,
    /// Idempotency key if provided
    pub idempotency_key: Option<String>,
}

/// Result of an SPL token transfer transaction
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TokenTransferResult {
    /// Transaction signature
    pub signature: String,
    /// Sender address
    pub from: String,
    /// Recipient address
    pub to: String,
    /// SPL token mint address
    pub mint: String,
    /// Raw amount transferred
    pub amount: u64,
    /// UI-formatted amount (adjusted for decimals)
    pub ui_amount: f64,
    /// Number of decimal places for the token
    pub decimals: u8,
    /// Human-readable amount display
    pub amount_display: String,
    /// Transaction status
    pub status: TransactionStatus,
    /// Idempotency key if provided
    pub idempotency_key: Option<String>,
}

/// Result of creating a new SPL token mint
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreateMintResult {
    /// Transaction signature
    pub signature: String,
    /// Address of the newly created token mint
    pub mint_address: String,
    /// Mint authority address
    pub authority: String,
    /// Number of decimal places for the token
    pub decimals: u8,
    /// Initial supply of tokens minted
    pub initial_supply: u64,
    /// Whether token accounts can be frozen
    pub freezable: bool,
}

/// Transaction status
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum TransactionStatus {
    /// Transaction is pending confirmation
    Pending,
    /// Transaction is confirmed
    Confirmed,
    /// Transaction is finalized
    Finalized,
    /// Transaction failed
    Failed(String),
}

// Implementation of the Tool trait for the generated structs to bridge RigTool to Tool
use crate::__riglr_tool_transfer_sol::TransferSolTool;

#[async_trait::async_trait]
impl riglr_core::Tool for TransferSolTool {
    type Args = serde_json::Value;
    type Output = riglr_core::JobResult;
    type Error = riglr_core::ToolError;

    fn name(&self) -> &'static str {
        "transfer_sol"
    }

    fn description(&self) -> &'static str {
        "Transfer SOL from one account to another"
    }

    fn schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "recipient": {
                    "type": "string",
                    "description": "Recipient wallet address (base58 encoded public key)"
                },
                "amount": {
                    "type": "number",
                    "description": "Amount to transfer in SOL (e.g., 0.001 for 1,000,000 lamports)"
                },
                "memo": {
                    "type": "string",
                    "description": "Optional memo to include in the transaction for record keeping"
                },
                "priority_fee": {
                    "type": "integer",
                    "description": "Optional priority fee in microlamports for faster processing"
                }
            },
            "required": ["recipient", "amount"]
        })
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        // Parse arguments from JSON
        let recipient: String = serde_json::from_value(
            args.get("recipient")
                .cloned()
                .unwrap_or(serde_json::Value::Null),
        )
        .map_err(|e| {
            riglr_core::ToolError::permanent_string(format!("Invalid recipient parameter: {}", e))
        })?;

        let amount: f64 = serde_json::from_value(
            args.get("amount")
                .cloned()
                .unwrap_or(serde_json::Value::Null),
        )
        .map_err(|e| {
            riglr_core::ToolError::permanent_string(format!("Invalid amount parameter: {}", e))
        })?;

        let memo: Option<String> = args
            .get("memo")
            .and_then(|v| serde_json::from_value(v.clone()).ok());

        let priority_fee: Option<u64> = args
            .get("priority_fee")
            .and_then(|v| serde_json::from_value(v.clone()).ok());

        // Call the actual tool function
        let result = transfer_sol(recipient, amount, memo, priority_fee, &self.context).await?;

        // Convert result to JobResult
        riglr_core::JobResult::success(&result).map_err(|e| {
            riglr_core::ToolError::permanent_string(format!("Failed to serialize result: {}", e))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use riglr_core::ToolError;
    use std::str::FromStr;
    use std::sync::Arc;

    // Helper function to create a test context with ApiClients
    fn create_test_context() -> riglr_core::provider::ApplicationContext {
        // Load .env.test for test environment
        dotenvy::from_filename(".env.test").ok();

        let config = riglr_config::Config::from_env();
        let context =
            riglr_core::provider::ApplicationContext::from_config(&Arc::new(config.clone()));

        // Create and inject ApiClients
        let api_clients = crate::clients::ApiClients::new(&config.providers);
        context.set_extension(Arc::new(api_clients));

        context
    }

    #[test]
    fn test_transaction_status_serialization() {
        let status = TransactionStatus::Pending;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"Pending\"");

        let status = TransactionStatus::Confirmed;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"Confirmed\"");

        let status = TransactionStatus::Finalized;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"Finalized\"");

        let status = TransactionStatus::Failed("error".to_string());
        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("Failed"));
        assert!(json.contains("error"));
    }

    #[test]
    fn test_transaction_status_deserialization() {
        let json = "\"Pending\"";
        let status: TransactionStatus = serde_json::from_str(json).unwrap();
        matches!(status, TransactionStatus::Pending);

        let json = "\"Confirmed\"";
        let status: TransactionStatus = serde_json::from_str(json).unwrap();
        matches!(status, TransactionStatus::Confirmed);

        let json = "\"Finalized\"";
        let status: TransactionStatus = serde_json::from_str(json).unwrap();
        matches!(status, TransactionStatus::Finalized);

        let json = "{\"Failed\":\"test error\"}";
        let status: TransactionStatus = serde_json::from_str(json).unwrap();
        if let TransactionStatus::Failed(msg) = status {
            assert_eq!(msg, "test error");
        } else {
            panic!("Expected Failed variant");
        }
    }

    #[test]
    fn test_transaction_result_creation() {
        let result = TransactionResult {
            signature: "test_signature".to_string(),
            from: "from_address".to_string(),
            to: "to_address".to_string(),
            amount: 1000000,
            amount_display: "0.001 SOL".to_string(),
            status: TransactionStatus::Pending,
            memo: Some("test memo".to_string()),
            idempotency_key: Some("test_key".to_string()),
        };

        assert_eq!(result.signature, "test_signature");
        assert_eq!(result.from, "from_address");
        assert_eq!(result.to, "to_address");
        assert_eq!(result.amount, 1000000);
        assert_eq!(result.amount_display, "0.001 SOL");
        assert_eq!(result.memo, Some("test memo".to_string()));
        assert_eq!(result.idempotency_key, Some("test_key".to_string()));
        matches!(result.status, TransactionStatus::Pending);
    }

    #[test]
    fn test_transaction_result_without_memo() {
        let result = TransactionResult {
            signature: "test_signature".to_string(),
            from: "from_address".to_string(),
            to: "to_address".to_string(),
            amount: 1000000,
            amount_display: "0.001 SOL".to_string(),
            status: TransactionStatus::Confirmed,
            memo: None,
            idempotency_key: None,
        };

        assert_eq!(result.memo, None);
        assert_eq!(result.idempotency_key, None);
        matches!(result.status, TransactionStatus::Confirmed);
    }

    #[test]
    fn test_token_transfer_result_creation() {
        let result = TokenTransferResult {
            signature: "token_signature".to_string(),
            from: "from_wallet".to_string(),
            to: "to_wallet".to_string(),
            mint: "mint_address".to_string(),
            amount: 1000000,
            ui_amount: 1.0,
            decimals: 6,
            amount_display: "1.000000000".to_string(),
            status: TransactionStatus::Finalized,
            idempotency_key: Some("token_key".to_string()),
        };

        assert_eq!(result.signature, "token_signature");
        assert_eq!(result.from, "from_wallet");
        assert_eq!(result.to, "to_wallet");
        assert_eq!(result.mint, "mint_address");
        assert_eq!(result.amount, 1000000);
        assert_eq!(result.ui_amount, 1.0);
        assert_eq!(result.decimals, 6);
        assert_eq!(result.amount_display, "1.000000000");
        matches!(result.status, TransactionStatus::Finalized);
    }

    #[test]
    fn test_create_mint_result_creation() {
        let result = CreateMintResult {
            signature: "mint_signature".to_string(),
            mint_address: "mint123".to_string(),
            authority: "authority123".to_string(),
            decimals: 9,
            initial_supply: 1000000000,
            freezable: true,
        };

        assert_eq!(result.signature, "mint_signature");
        assert_eq!(result.mint_address, "mint123");
        assert_eq!(result.authority, "authority123");
        assert_eq!(result.decimals, 9);
        assert_eq!(result.initial_supply, 1000000000);
        assert!(result.freezable);
    }

    #[test]
    fn test_create_mint_result_not_freezable() {
        let result = CreateMintResult {
            signature: "mint_signature".to_string(),
            mint_address: "mint123".to_string(),
            authority: "authority123".to_string(),
            decimals: 0,
            initial_supply: 0,
            freezable: false,
        };

        assert_eq!(result.decimals, 0);
        assert_eq!(result.initial_supply, 0);
        assert!(!result.freezable);
    }

    #[test]
    fn test_serialization_roundtrip_transaction_result() {
        let original = TransactionResult {
            signature: "sig123".to_string(),
            from: "from123".to_string(),
            to: "to123".to_string(),
            amount: 500000,
            amount_display: "0.0005 SOL".to_string(),
            status: TransactionStatus::Failed("Network error".to_string()),
            memo: Some("payment".to_string()),
            idempotency_key: Some("key123".to_string()),
        };

        let json = serde_json::to_string(&original).unwrap();
        let deserialized: TransactionResult = serde_json::from_str(&json).unwrap();

        assert_eq!(original.signature, deserialized.signature);
        assert_eq!(original.from, deserialized.from);
        assert_eq!(original.to, deserialized.to);
        assert_eq!(original.amount, deserialized.amount);
        assert_eq!(original.amount_display, deserialized.amount_display);
        assert_eq!(original.memo, deserialized.memo);
        assert_eq!(original.idempotency_key, deserialized.idempotency_key);

        if let (TransactionStatus::Failed(orig_msg), TransactionStatus::Failed(deser_msg)) =
            (&original.status, &deserialized.status)
        {
            assert_eq!(orig_msg, deser_msg);
        } else {
            panic!("Status should be Failed variant");
        }
    }

    #[test]
    fn test_serialization_roundtrip_token_transfer_result() {
        let original = TokenTransferResult {
            signature: "token_sig".to_string(),
            from: "from_addr".to_string(),
            to: "to_addr".to_string(),
            mint: "mint_addr".to_string(),
            amount: 1000000,
            ui_amount: 1.0,
            decimals: 6,
            amount_display: "1.000000".to_string(),
            status: TransactionStatus::Pending,
            idempotency_key: None,
        };

        let json = serde_json::to_string(&original).unwrap();
        let deserialized: TokenTransferResult = serde_json::from_str(&json).unwrap();

        assert_eq!(original.signature, deserialized.signature);
        assert_eq!(original.from, deserialized.from);
        assert_eq!(original.to, deserialized.to);
        assert_eq!(original.mint, deserialized.mint);
        assert_eq!(original.amount, deserialized.amount);
        assert_eq!(original.ui_amount, deserialized.ui_amount);
        assert_eq!(original.decimals, deserialized.decimals);
        assert_eq!(original.amount_display, deserialized.amount_display);
        assert_eq!(original.idempotency_key, deserialized.idempotency_key);
        matches!(deserialized.status, TransactionStatus::Pending);
    }

    #[test]
    fn test_serialization_roundtrip_create_mint_result() {
        let original = CreateMintResult {
            signature: "create_sig".to_string(),
            mint_address: "new_mint".to_string(),
            authority: "auth_addr".to_string(),
            decimals: 9,
            initial_supply: 5000000,
            freezable: true,
        };

        let json = serde_json::to_string(&original).unwrap();
        let deserialized: CreateMintResult = serde_json::from_str(&json).unwrap();

        assert_eq!(original.signature, deserialized.signature);
        assert_eq!(original.mint_address, deserialized.mint_address);
        assert_eq!(original.authority, deserialized.authority);
        assert_eq!(original.decimals, deserialized.decimals);
        assert_eq!(original.initial_supply, deserialized.initial_supply);
        assert_eq!(original.freezable, deserialized.freezable);
    }

    #[test]
    fn test_debug_formatting() {
        let status = TransactionStatus::Pending;
        let debug_str = format!("{:?}", status);
        assert!(debug_str.contains("Pending"));

        let status = TransactionStatus::Failed("test error".to_string());
        let debug_str = format!("{:?}", status);
        assert!(debug_str.contains("Failed"));
        assert!(debug_str.contains("test error"));

        let result = TransactionResult {
            signature: "test".to_string(),
            from: "from".to_string(),
            to: "to".to_string(),
            amount: 100,
            amount_display: "display".to_string(),
            status: TransactionStatus::Confirmed,
            memo: None,
            idempotency_key: None,
        };
        let debug_str = format!("{:?}", result);
        assert!(debug_str.contains("TransactionResult"));
    }

    #[test]
    fn test_clone_functionality() {
        let original_status = TransactionStatus::Failed("clone test".to_string());
        let cloned_status = original_status.clone();
        if let (TransactionStatus::Failed(orig), TransactionStatus::Failed(cloned)) =
            (&original_status, &cloned_status)
        {
            assert_eq!(orig, cloned);
        }

        let original_result = TransactionResult {
            signature: "clone_sig".to_string(),
            from: "clone_from".to_string(),
            to: "clone_to".to_string(),
            amount: 999,
            amount_display: "clone_display".to_string(),
            status: TransactionStatus::Finalized,
            memo: Some("clone memo".to_string()),
            idempotency_key: Some("clone_key".to_string()),
        };
        let cloned_result = original_result.clone();
        assert_eq!(original_result.signature, cloned_result.signature);
        assert_eq!(original_result.memo, cloned_result.memo);
    }

    // Note: The actual function tests for transfer_sol, transfer_spl_token, and create_spl_token_mint
    // cannot be easily unit tested in isolation because they depend on external services
    // (SignerContext, Solana RPC, send_transaction utility) that would require extensive mocking.
    // These functions are better suited for integration tests with a test environment.
    // However, we can test input validation logic by extracting it into separate testable functions
    // or testing error conditions with invalid inputs.

    #[tokio::test]
    async fn test_transfer_sol_negative_amount() {
        let context = create_test_context();
        let result = transfer_sol(
            "9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM".to_string(),
            -0.001,
            None,
            None,
            &context,
        )
        .await;

        assert!(result.is_err());
        if let Err(ToolError::Permanent { context, .. }) = result {
            assert!(context.contains("Amount must be positive"));
        } else {
            panic!("Expected permanent error for negative amount");
        }
    }

    #[tokio::test]
    async fn test_transfer_sol_zero_amount() {
        let context = create_test_context();
        let result = transfer_sol(
            "9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM".to_string(),
            0.0,
            None,
            None,
            &context,
        )
        .await;

        assert!(result.is_err());
        if let Err(ToolError::Permanent { context, .. }) = result {
            assert!(context.contains("Amount must be positive"));
        } else {
            panic!("Expected permanent error for zero amount");
        }
    }

    #[tokio::test]
    async fn test_transfer_sol_invalid_address() {
        let context = create_test_context();
        let result = transfer_sol("invalid_address".to_string(), 0.001, None, None, &context).await;

        assert!(result.is_err());
        if let Err(ToolError::Permanent { context, .. }) = result {
            assert!(context.contains("Invalid recipient address"));
        } else {
            panic!("Expected permanent error for invalid address");
        }
    }

    #[tokio::test]
    async fn test_transfer_sol_empty_address() {
        let context = create_test_context();
        let result = transfer_sol("".to_string(), 0.001, None, None, &context).await;

        assert!(result.is_err());
        if let Err(ToolError::Permanent { context, .. }) = result {
            assert!(context.contains("Invalid recipient address"));
        } else {
            panic!("Expected permanent error for empty address");
        }
    }

    #[tokio::test]
    async fn test_transfer_sol_very_large_amount() {
        let context = create_test_context();
        let result = transfer_sol(
            "9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM".to_string(),
            f64::MAX,
            None,
            None,
            &context,
        )
        .await;

        // Should fail when trying to get signer context, but amount validation should pass
        assert!(result.is_err());
        // We expect it to fail at signer context retrieval, not amount validation
        if let Err(ToolError::Permanent { context, .. }) = result {
            assert!(context.contains("No signer context") || context.contains("signer"));
        }
    }

    #[tokio::test]
    async fn test_transfer_spl_token_invalid_recipient() {
        let context = create_test_context();
        let result = transfer_spl_token(
            "invalid_recipient".to_string(),
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
            1000000,
            6,
            true,
            &context,
        )
        .await;

        assert!(result.is_err());
        if let Err(ToolError::Permanent { context, .. }) = result {
            assert!(context.contains("Invalid recipient address"));
        } else {
            panic!("Expected permanent error for invalid recipient");
        }
    }

    #[tokio::test]
    async fn test_transfer_spl_token_invalid_mint() {
        let context = create_test_context();
        let result = transfer_spl_token(
            "9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM".to_string(),
            "invalid_mint".to_string(),
            1000000,
            6,
            true,
            &context,
        )
        .await;

        assert!(result.is_err());
        if let Err(ToolError::Permanent { context, .. }) = result {
            assert!(context.contains("Invalid mint address"));
        } else {
            panic!("Expected permanent error for invalid mint");
        }
    }

    #[tokio::test]
    async fn test_transfer_spl_token_empty_addresses() {
        let context = create_test_context();
        let result =
            transfer_spl_token("".to_string(), "".to_string(), 1000000, 6, false, &context).await;

        assert!(result.is_err());
        if let Err(ToolError::Permanent { context, .. }) = result {
            assert!(context.contains("Invalid recipient address"));
        } else {
            panic!("Expected permanent error for empty recipient address");
        }
    }

    #[tokio::test]
    async fn test_transfer_spl_token_zero_amount() {
        let context = create_test_context();
        let result = transfer_spl_token(
            "9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM".to_string(),
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
            0,
            6,
            true,
            &context,
        )
        .await;

        // Zero amount is technically valid for SPL tokens, should fail at signer context
        assert!(result.is_err());
        if let Err(ToolError::Permanent { context, .. }) = result {
            assert!(context.contains("No signer context") || context.contains("signer"));
        }
    }

    #[tokio::test]
    async fn test_transfer_spl_token_max_amount() {
        let context = create_test_context();
        let result = transfer_spl_token(
            "9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM".to_string(),
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
            u64::MAX,
            6,
            true,
            &context,
        )
        .await;

        // Should fail at signer context, not amount validation
        assert!(result.is_err());
        if let Err(ToolError::Permanent { context, .. }) = result {
            assert!(context.contains("No signer context") || context.contains("signer"));
        }
    }

    #[tokio::test]
    async fn test_transfer_spl_token_max_decimals() {
        let context = create_test_context();
        let result = transfer_spl_token(
            "9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM".to_string(),
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
            1000000,
            255, // Max u8 value
            false,
            &context,
        )
        .await;

        // Should fail at signer context, decimals validation happens later
        assert!(result.is_err());
        if let Err(ToolError::Permanent { context, .. }) = result {
            assert!(context.contains("No signer context") || context.contains("signer"));
        }
    }

    #[tokio::test]
    async fn test_create_spl_token_mint_invalid_decimals() {
        let context = create_test_context();
        let result = create_spl_token_mint(
            10, // Invalid: > 9
            1000000, true, &context,
        )
        .await;

        assert!(result.is_err());
        if let Err(ToolError::Permanent { context, .. }) = result {
            assert!(context.contains("Decimals must be between 0 and 9"));
        } else {
            panic!("Expected permanent error for invalid decimals");
        }
    }

    #[tokio::test]
    async fn test_create_spl_token_mint_decimals_boundary_valid() {
        let context = create_test_context();
        // Test boundary values
        let result_0 = create_spl_token_mint(0, 1000000, true, &context).await;
        let result_9 = create_spl_token_mint(9, 1000000, false, &context).await;

        // Both should fail at signer context, not decimals validation
        assert!(result_0.is_err());
        assert!(result_9.is_err());

        if let Err(ToolError::Permanent { context, .. }) = result_0 {
            assert!(context.contains("No signer context") || context.contains("signer"));
        }
        if let Err(ToolError::Permanent { context, .. }) = result_9 {
            assert!(context.contains("No signer context") || context.contains("signer"));
        }
    }

    #[tokio::test]
    async fn test_create_spl_token_mint_decimals_boundary_invalid() {
        let context = create_test_context();
        let result = create_spl_token_mint(
            255, // Much larger than 9
            0, false, &context,
        )
        .await;

        assert!(result.is_err());
        if let Err(ToolError::Permanent { context, .. }) = result {
            assert!(context.contains("Decimals must be between 0 and 9"));
        }
    }

    #[tokio::test]
    async fn test_create_spl_token_mint_zero_initial_supply() {
        let context = create_test_context();
        let result = create_spl_token_mint(
            6, 0, // Zero initial supply should be valid
            true, &context,
        )
        .await;

        // Should fail at signer context, not supply validation
        assert!(result.is_err());
        if let Err(ToolError::Permanent { context, .. }) = result {
            assert!(context.contains("No signer context") || context.contains("signer"));
        }
    }

    #[tokio::test]
    async fn test_create_spl_token_mint_max_initial_supply() {
        let context = create_test_context();
        let result = create_spl_token_mint(9, u64::MAX, false, &context).await;

        // Should fail at signer context, not supply validation
        assert!(result.is_err());
        if let Err(ToolError::Permanent { context, .. }) = result {
            assert!(context.contains("No signer context") || context.contains("signer"));
        }
    }

    #[test]
    fn test_ui_amount_calculation() {
        // Test UI amount calculation logic (extracted from transfer_spl_token)
        let amount = 1000000u64;
        let decimals = 6u8;
        let ui_amount = amount as f64 / 10_f64.powi(decimals as i32);
        assert_eq!(ui_amount, 1.0);

        let amount = 1500000u64;
        let decimals = 6u8;
        let ui_amount = amount as f64 / 10_f64.powi(decimals as i32);
        assert_eq!(ui_amount, 1.5);

        let amount = 1u64;
        let decimals = 9u8;
        let ui_amount = amount as f64 / 10_f64.powi(decimals as i32);
        assert_eq!(ui_amount, 0.000000001);

        let amount = 0u64;
        let decimals = 0u8;
        let ui_amount = amount as f64 / 10_f64.powi(decimals as i32);
        assert_eq!(ui_amount, 0.0);
    }

    #[test]
    fn test_lamports_conversion() {
        // Test SOL to lamports conversion logic (extracted from transfer_sol)
        let amount_sol = 0.001f64;
        let lamports = (amount_sol * LAMPORTS_PER_SOL as f64) as u64;
        assert_eq!(lamports, 1000000);

        let amount_sol = 1.0f64;
        let lamports = (amount_sol * LAMPORTS_PER_SOL as f64) as u64;
        assert_eq!(lamports, LAMPORTS_PER_SOL);

        let amount_sol = 0.0f64;
        let lamports = (amount_sol * LAMPORTS_PER_SOL as f64) as u64;
        assert_eq!(lamports, 0);

        // Test edge case with very small amounts
        let amount_sol = 0.000000001f64; // 1 lamport
        let lamports = (amount_sol * LAMPORTS_PER_SOL as f64) as u64;
        assert_eq!(lamports, 1);
    }

    #[test]
    fn test_memo_program_pubkey() {
        // Test that the memo program pubkey is correct
        let memo_pubkey = Pubkey::from_str("MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr").unwrap();
        assert_eq!(
            memo_pubkey.to_string(),
            "MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr"
        );
    }

    #[test]
    fn test_address_validation_edge_cases() {
        // Test various invalid address formats that Pubkey::from_str should reject

        // Empty string
        let result = Pubkey::from_str("");
        assert!(result.is_err());

        // Too short
        let result = Pubkey::from_str("short");
        assert!(result.is_err());

        // Invalid characters
        let result = Pubkey::from_str("InvalidChars!@#$%^&*()");
        assert!(result.is_err());

        // Too long
        let result =
            Pubkey::from_str("ThisIsWayTooLongToBeAValidSolanaPubkeyAddressAndShouldFail123456789");
        assert!(result.is_err());

        // Valid address should work
        let result = Pubkey::from_str("9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM");
        assert!(result.is_ok());
    }

    #[test]
    fn test_transaction_status_variants_coverage() {
        // Test all variants of TransactionStatus
        let pending = TransactionStatus::Pending;
        let confirmed = TransactionStatus::Confirmed;
        let finalized = TransactionStatus::Finalized;
        let failed = TransactionStatus::Failed("test error".to_string());

        // Test Debug trait
        assert!(!format!("{:?}", pending).is_empty());
        assert!(!format!("{:?}", confirmed).is_empty());
        assert!(!format!("{:?}", finalized).is_empty());
        assert!(!format!("{:?}", failed).is_empty());

        // Test Clone trait
        let _cloned_pending = pending.clone();
        let _cloned_confirmed = confirmed.clone();
        let _cloned_finalized = finalized.clone();
        let _cloned_failed = failed.clone();
    }

    #[test]
    fn test_result_structs_field_access() {
        // Test that all fields are accessible and have correct types
        let tx_result = TransactionResult {
            signature: "test_signature".to_string(),
            from: "test_from".to_string(),
            to: "test_to".to_string(),
            amount: 0u64,
            amount_display: "test_display".to_string(),
            status: TransactionStatus::Pending,
            memo: None,
            idempotency_key: None,
        };
        let _: String = tx_result.signature;
        let _: u64 = tx_result.amount;
        let _: Option<String> = tx_result.memo;

        let token_result = TokenTransferResult {
            signature: "test_token_sig".to_string(),
            from: "test_from".to_string(),
            to: "test_to".to_string(),
            mint: "test_mint".to_string(),
            amount: 0u64,
            ui_amount: 0.0f64,
            decimals: 0u8,
            amount_display: "test_display".to_string(),
            status: TransactionStatus::Pending,
            idempotency_key: None,
        };
        let _: f64 = token_result.ui_amount;
        let _: u8 = token_result.decimals;

        let mint_result = CreateMintResult {
            signature: "test_mint_sig".to_string(),
            mint_address: "test_mint_addr".to_string(),
            authority: "test_authority".to_string(),
            decimals: 0u8,
            initial_supply: 0u64,
            freezable: false,
        };
        let _: bool = mint_result.freezable;
        let _: u8 = mint_result.decimals;
    }
}
