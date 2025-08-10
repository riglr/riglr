//! Transaction tools for Solana blockchain
//!
//! This module provides tools for creating and executing transactions on the Solana blockchain.
//! All state-mutating operations are queued through the job system for resilience.

use riglr_core::{ToolError, SignerContext};
use riglr_macros::tool;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
#[allow(deprecated)]
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    native_token::LAMPORTS_PER_SOL,
    pubkey::Pubkey,
    system_instruction,
    transaction::Transaction,
};
use spl_associated_token_account::get_associated_token_address;
use spl_token;
use std::str::FromStr;
use tracing::{debug, info};


/// Transfer SOL from one account to another
///
/// This tool creates and executes a SOL transfer transaction.
#[tool]
pub async fn transfer_sol(
    to_address: String,
    amount_sol: f64,
    memo: Option<String>,
    priority_fee: Option<u64>,
) -> Result<TransactionResult, ToolError> {
    debug!(
        "Initiating SOL transfer of {} SOL to {}",
        amount_sol, to_address
    );

    // Validate inputs
    if amount_sol <= 0.0 {
        return Err(ToolError::permanent("Amount must be positive"));
    }

    let to_pubkey = Pubkey::from_str(&to_address)
        .map_err(|e| ToolError::permanent(format!("Invalid recipient address: {}", e)))?;

    // Convert SOL to lamports
    let lamports = (amount_sol * LAMPORTS_PER_SOL as f64) as u64;

    // Get signer from context
    let signer_context = SignerContext::current().await
        .map_err(|e| ToolError::permanent(format!("No signer context: {}", e)))?;
    
    let from_pubkey = signer_context.pubkey()
        .ok_or_else(|| ToolError::permanent("Signer has no public key"))?
        .parse::<Pubkey>()
        .map_err(|e| ToolError::permanent(format!("Invalid signer pubkey: {}", e)))?;

    // Create transfer instruction
    let mut instructions = vec![system_instruction::transfer(
        &from_pubkey,
        &to_pubkey,
        lamports,
    )];

    // Add priority fee if specified
    if let Some(fee) = priority_fee {
        instructions.insert(
            0,
            solana_sdk::compute_budget::ComputeBudgetInstruction::set_compute_unit_price(fee),
        );
    }

    // Add memo if provided
    if let Some(memo_text) = &memo {
        let memo_ix = Instruction::new_with_bytes(
            Pubkey::from_str("MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr").unwrap(),
            memo_text.as_bytes(),
            vec![AccountMeta::new(from_pubkey, true)],
        );
        instructions.push(memo_ix);
    }

    // Create and sign transaction
    let mut transaction = Transaction::new_with_payer(&instructions, Some(&from_pubkey));

    // Sign and send through the signer context
    let signature = signer_context.sign_and_send_solana_transaction(&mut transaction).await
        .map_err(|e| ToolError::permanent(format!("Failed to send transaction: {}", e)))?;

    info!(
        "SOL transfer initiated: {} -> {} ({} SOL), signature: {}",
        from_pubkey,
        to_address,
        amount_sol,
        signature
    );

    Ok(TransactionResult {
        signature,
        from: from_pubkey.to_string(),
        to: to_address,
        amount: lamports,
        amount_display: format!("{} SOL", amount_sol),
        status: TransactionStatus::Confirmed,
        memo,
        idempotency_key: None,
    })
}

/// Transfer SPL tokens from one account to another
#[tool]
pub async fn transfer_spl_token(
    to_address: String,
    mint_address: String,
    amount: u64,
    decimals: u8,
    create_ata_if_needed: bool,
) -> Result<TokenTransferResult, ToolError> {
    debug!(
        "Initiating SPL token transfer of {} to {}",
        amount, to_address
    );

    // Validate inputs
    let to_pubkey = Pubkey::from_str(&to_address)
        .map_err(|e| ToolError::permanent(format!("Invalid recipient address: {}", e)))?;
    let mint_pubkey = Pubkey::from_str(&mint_address)
        .map_err(|e| ToolError::permanent(format!("Invalid mint address: {}", e)))?;

    // Get signer from context
    let signer_context = SignerContext::current().await
        .map_err(|e| ToolError::permanent(format!("No signer context: {}", e)))?;
    
    let from_pubkey = signer_context.pubkey()
        .ok_or_else(|| ToolError::permanent("Signer has no public key"))?
        .parse::<Pubkey>()
        .map_err(|e| ToolError::permanent(format!("Invalid signer pubkey: {}", e)))?;

    // Get associated token accounts
    let from_ata = get_associated_token_address(&from_pubkey, &mint_pubkey);
    let to_ata = get_associated_token_address(&to_pubkey, &mint_pubkey);

    let mut instructions = Vec::new();

    // Check if recipient ATA exists and create if needed
    if create_ata_if_needed {
        // The create_associated_token_account_idempotent instruction is safe to include
        // even if the account already exists
        instructions.push(
            spl_associated_token_account::instruction::create_associated_token_account_idempotent(
                &from_pubkey,
                &to_pubkey,
                &mint_pubkey,
                &spl_token::id(),
            ),
        );
    }

    // Create transfer instruction
    instructions.push(
        spl_token::instruction::transfer(
            &spl_token::id(),
            &from_ata,
            &to_ata,
            &from_pubkey,
            &[],
            amount,
        )
        .map_err(|e| ToolError::permanent(format!("Failed to create transfer instruction: {}", e)))?,
    );

    // Create and sign transaction
    let mut transaction = Transaction::new_with_payer(&instructions, Some(&from_pubkey));

    // Sign and send through the signer context
    let signature = signer_context.sign_and_send_solana_transaction(&mut transaction).await
        .map_err(|e| ToolError::permanent(format!("Failed to send transaction: {}", e)))?;
    let ui_amount = amount as f64 / 10_f64.powi(decimals as i32);

    info!(
        "SPL token transfer initiated: {} -> {} ({} tokens), signature: {}",
        from_pubkey,
        to_address,
        ui_amount,
        signature
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
        status: TransactionStatus::Confirmed,
        idempotency_key: None,
    })
}

/// Create a new SPL token mint
#[tool]
pub async fn create_spl_token_mint(
    decimals: u8,
    initial_supply: u64,
    freezable: bool,
) -> Result<CreateMintResult, ToolError> {
    let _ = (decimals, initial_supply, freezable);
    Err(ToolError::permanent("Implementation pending"))
}

/// Helper function for default true value
fn default_true() -> bool {
    true
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
    pub memo: Option<String>,
    /// Idempotency key if provided
    pub idempotency_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TokenTransferResult {
    /// Transaction signature
    pub signature: String,
    /// Sender address
    pub from: String,
    /// Recipient address
    pub to: String,
    pub mint: String,
    /// Raw amount transferred
    pub amount: u64,
    pub ui_amount: f64,
    pub decimals: u8,
    /// Human-readable amount display
    pub amount_display: String,
    /// Transaction status
    pub status: TransactionStatus,
    /// Idempotency key if provided
    pub idempotency_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreateMintResult {
    /// Transaction signature
    pub signature: String,
    pub mint_address: String,
    pub authority: String,
    pub decimals: u8,
    pub initial_supply: u64,
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

#[cfg(test)]
mod tests {
    use super::*;

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
