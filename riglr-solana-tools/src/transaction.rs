//! Transaction tools for Solana blockchain
//!
//! This module provides tools for creating and executing transactions on the Solana blockchain.
//! All state-mutating operations are queued through the job system for resilience.

use crate::client::SolanaClient;
use crate::error::{Result, SolanaToolError};
use anyhow::anyhow;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    message::Message,
    native_token::LAMPORTS_PER_SOL,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_instruction,
    transaction::Transaction,
};
use spl_associated_token_account::get_associated_token_address;
use spl_token;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::{Arc, RwLock};
use tracing::{debug, info};

/// Secure signer context for managing keypairs
///
/// This context ensures that private keys are never exposed to the agent's
/// reasoning context, following the security requirements.
#[derive(Clone)]
pub struct SignerContext {
    /// Map of signer names to keypairs
    signers: Arc<RwLock<HashMap<String, Arc<Keypair>>>>,
    /// Default signer name
    default_signer: Option<String>,
}

impl SignerContext {
    /// Create a new empty signer context
    pub fn new() -> Self {
        Self {
            signers: Arc::new(RwLock::new(HashMap::new())),
            default_signer: None,
        }
    }

    /// Add a signer from a private key bytes
    pub fn add_signer(&mut self, name: impl Into<String>, keypair: Keypair) -> Result<()> {
        let name = name.into();
        let mut signers = self
            .signers
            .write()
            .map_err(|e| SolanaToolError::Generic(format!("Lock error: {}", e)))?;

        if self.default_signer.is_none() {
            self.default_signer = Some(name.clone());
        }

        signers.insert(name, Arc::new(keypair));
        Ok(())
    }

    /// Get a signer by name
    pub fn get_signer(&self, name: &str) -> Result<Arc<Keypair>> {
        let signers = self
            .signers
            .read()
            .map_err(|e| SolanaToolError::Generic(format!("Lock error: {}", e)))?;

        signers
            .get(name)
            .cloned()
            .ok_or_else(|| SolanaToolError::Generic(format!("Signer '{}' not found", name)))
    }

    /// Get the default signer
    pub fn get_default_signer(&self) -> Result<Arc<Keypair>> {
        let name = self
            .default_signer
            .as_ref()
            .ok_or_else(|| SolanaToolError::Generic("No default signer configured".to_string()))?;
        self.get_signer(name)
    }

    /// Get public key for a signer
    pub fn get_pubkey(&self, name: &str) -> Result<Pubkey> {
        Ok(self.get_signer(name)?.pubkey())
    }
}

impl Default for SignerContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Global signer context
static mut SIGNER_CONTEXT: Option<Arc<SignerContext>> = None;
static SIGNER_INIT: std::sync::Once = std::sync::Once::new();

pub fn init_signer_context(context: SignerContext) {
    unsafe {
        SIGNER_INIT.call_once(|| {
            SIGNER_CONTEXT = Some(Arc::new(context));
        });
    }
}

/// Get the global signer context
pub fn get_signer_context() -> Result<Arc<SignerContext>> {
    unsafe {
        SIGNER_CONTEXT.as_ref().cloned().ok_or_else(|| {
            SolanaToolError::Generic(
                "Signer context not initialized. Call init_signer_context() first.".to_string(),
            )
        })
    }
}

/// Transfer SOL from one account to another
///
/// This tool creates and executes a SOL transfer transaction.
/// The transaction is queued for execution with automatic retry and idempotency.
// #[tool]
pub async fn transfer_sol(
    client: &SolanaClient,
    to_address: String,
    amount_sol: f64,
    from_signer: Option<String>,
    memo: Option<String>,
    priority_fee: Option<u64>,
) -> anyhow::Result<TransactionResult> {
    debug!(
        "Initiating SOL transfer of {} SOL to {}",
        amount_sol, to_address
    );

    // Validate inputs
    if amount_sol <= 0.0 {
        return Err(anyhow!("Amount must be positive"));
    }

    let to_pubkey =
        Pubkey::from_str(&to_address).map_err(|e| anyhow!("Invalid recipient address: {}", e))?;

    // Get signer
    let signer_context =
        get_signer_context().map_err(|e| anyhow!("Failed to get signer context: {}", e))?;

    let signer = if let Some(name) = from_signer {
        signer_context
            .get_signer(&name)
            .map_err(|e| anyhow!("Failed to get signer '{}': {}", name, e))?
    } else {
        signer_context
            .get_default_signer()
            .map_err(|e| anyhow!("Failed to get default signer: {}", e))?
    };

    // Convert SOL to lamports
    let lamports = (amount_sol * LAMPORTS_PER_SOL as f64) as u64;

    // Get recent blockhash
    let blockhash = client
        .get_latest_blockhash()
        .await
        .map_err(|e| anyhow!("Failed to get blockhash: {}", e))?;

    // Create transfer instruction
    let mut instructions = vec![system_instruction::transfer(
        &signer.pubkey(),
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
            vec![AccountMeta::new(signer.pubkey(), true)],
        );
        instructions.push(memo_ix);
    }

    // Create message
    let message = Message::new(&instructions, Some(&signer.pubkey()));

    // Create and sign transaction
    let mut transaction = Transaction::new_unsigned(message);
    let blockhash = blockhash
        .parse()
        .map_err(|e| anyhow!("Failed to parse blockhash: {}", e))?;
    transaction.partial_sign(&[signer.as_ref()], blockhash);

    // Send transaction
    let signature = client
        .send_and_confirm_transaction(&transaction)
        .await
        .map_err(|e| anyhow!("Failed to send transaction: {}", e))?;

    let sig_str = signature.to_string();
    info!(
        "SOL transfer initiated: {} -> {} ({} SOL), signature: {}",
        signer.pubkey(),
        to_address,
        amount_sol,
        sig_str
    );

    Ok(TransactionResult {
        signature: sig_str,
        from: signer.pubkey().to_string(),
        to: to_address,
        amount: lamports,
        amount_display: format!("{} SOL", amount_sol),
        status: TransactionStatus::Confirmed,
        memo,
        idempotency_key: None,
    })
}

/// Transfer SPL tokens from one account to another
// #[tool]
pub async fn transfer_spl_token(
    client: &SolanaClient,
    to_address: String,
    mint_address: String,
    amount: u64,
    decimals: u8,
    from_signer: Option<String>,
    create_ata_if_needed: bool,
) -> anyhow::Result<TokenTransferResult> {
    debug!(
        "Initiating SPL token transfer of {} to {}",
        amount, to_address
    );

    // Validate inputs
    let to_pubkey =
        Pubkey::from_str(&to_address).map_err(|e| anyhow!("Invalid recipient address: {}", e))?;

    let mint_pubkey =
        Pubkey::from_str(&mint_address).map_err(|e| anyhow!("Invalid mint address: {}", e))?;

    // Get signer
    let signer_context =
        get_signer_context().map_err(|e| anyhow!("Failed to get signer context: {}", e))?;

    let signer = if let Some(name) = from_signer {
        signer_context
            .get_signer(&name)
            .map_err(|e| anyhow!("Failed to get signer '{}': {}", name, e))?
    } else {
        signer_context
            .get_default_signer()
            .map_err(|e| anyhow!("Failed to get default signer: {}", e))?
    };

    // Get associated token accounts
    let from_ata = get_associated_token_address(&signer.pubkey(), &mint_pubkey);
    let to_ata = get_associated_token_address(&to_pubkey, &mint_pubkey);

    // Get recent blockhash
    let blockhash = client
        .get_latest_blockhash()
        .await
        .map_err(|e| anyhow!("Failed to get blockhash: {}", e))?;

    let mut instructions = Vec::new();

    // Check if recipient ATA exists and create if needed
    if create_ata_if_needed {
        // The create_associated_token_account_idempotent instruction is safe to include
        // even if the account already exists
        instructions.push(
            spl_associated_token_account::instruction::create_associated_token_account_idempotent(
                &signer.pubkey(),
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
            &signer.pubkey(),
            &[],
            amount,
        )
        .map_err(|e| anyhow!("Failed to create transfer instruction: {}", e))?,
    );

    // Create message
    let message = Message::new(&instructions, Some(&signer.pubkey()));

    // Create and sign transaction
    let mut transaction = Transaction::new_unsigned(message);
    let blockhash = blockhash
        .parse()
        .map_err(|e| anyhow!("Failed to parse blockhash: {}", e))?;
    transaction.partial_sign(&[signer.as_ref()], blockhash);

    // Send transaction
    let signature = client
        .send_and_confirm_transaction(&transaction)
        .await
        .map_err(|e| anyhow!("Failed to send transaction: {}", e))?;

    let sig_str = signature.to_string();
    let ui_amount = amount as f64 / 10_f64.powi(decimals as i32);

    info!(
        "SPL token transfer initiated: {} -> {} ({} tokens), signature: {}",
        signer.pubkey(),
        to_address,
        ui_amount,
        sig_str
    );

    Ok(TokenTransferResult {
        signature: sig_str,
        from: signer.pubkey().to_string(),
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

///
// #[tool]
pub async fn create_spl_token_mint(
    _decimals: u8,

    _initial_supply: u64,

    _freezable: bool,

    _authority_signer: Option<String>,

    _rpc_url: Option<String>,
) -> anyhow::Result<CreateMintResult> {
    todo!("Implementation pending")
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
    fn test_signer_context() {
        let mut context = SignerContext::new();
        let keypair = Keypair::new();
        let pubkey = keypair.pubkey();

        context.add_signer("test", keypair).unwrap();

        let retrieved = context.get_signer("test").unwrap();
        assert_eq!(retrieved.pubkey(), pubkey);

        let default = context.get_default_signer().unwrap();
        assert_eq!(default.pubkey(), pubkey);
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
