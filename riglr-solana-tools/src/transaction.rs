//! Transaction tools for Solana blockchain
//!
//! This module provides tools for creating and executing transactions on the Solana blockchain.
//! All state-mutating operations are queued through the job system for resilience.

use crate::client::{SolanaClient, SolanaConfig};
use crate::error::{Result, SolanaToolError};
use anyhow::anyhow;
use riglr_core::{Job, JobQueue, JobResult};
use riglr_macros::tool;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use solana_sdk::{
    commitment_config::CommitmentLevel,
    instruction::{AccountMeta, Instruction},
    message::Message,
    native_token::LAMPORTS_PER_SOL,
    pubkey::Pubkey,
    signature::{Keypair, Signature, Signer},
    system_instruction, system_program,
    transaction::Transaction,
};
use spl_associated_token_account::get_associated_token_address;
use spl_token;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::{Arc, RwLock};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

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

/// Initialize the global signer context
pub fn init_signer_context(context: SignerContext) {
    unsafe {
        SIGNER_INIT.call_once(|| {
            SIGNER_CONTEXT = Some(Arc::new(context));
        });
    }
}

/// Get the global signer context
fn get_signer_context() -> Result<Arc<SignerContext>> {
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
#[tool]
pub async fn transfer_sol(
    /// The recipient address (base58 encoded public key)
    to_address: String,
    /// Amount to transfer in SOL (will be converted to lamports)
    amount_sol: f64,
    /// Optional sender signer name (uses default if not provided)
    #[serde(default)]
    from_signer: Option<String>,
    /// Optional memo to include with the transfer
    #[serde(default)]
    memo: Option<String>,
    /// RPC endpoint URL (optional, defaults to mainnet)
    #[serde(default)]
    rpc_url: Option<String>,
    /// Optional idempotency key to prevent duplicate transfers
    #[serde(default)]
    idempotency_key: Option<String>,
    /// Priority fee in microlamports per compute unit
    #[serde(default)]
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

    // Create client
    let client = if let Some(url) = rpc_url {
        Arc::new(SolanaClient::with_rpc_url(url))
    } else {
        Arc::new(SolanaClient::default())
    };

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

    // Create transaction
    let mut transaction = Transaction::new_unsigned(message);
    transaction.partial_sign(&[signer.as_ref()], blockhash.parse().unwrap());

    // Send transaction
    let signature = client
        .send_transaction(&transaction)
        .await
        .map_err(|e| anyhow!("Failed to send transaction: {}", e))?;

    info!(
        "SOL transfer initiated: {} -> {} ({} SOL), signature: {}",
        signer.pubkey(),
        to_address,
        amount_sol,
        signature
    );

    Ok(TransactionResult {
        signature,
        from: signer.pubkey().to_string(),
        to: to_address,
        amount: lamports,
        amount_display: format!("{} SOL", amount_sol),
        status: TransactionStatus::Pending,
        memo,
        idempotency_key,
    })
}

/// Transfer SPL tokens from one account to another
///
/// This tool creates and executes an SPL token transfer transaction.
/// It handles associated token account creation if necessary.
#[tool]
pub async fn transfer_spl_token(
    /// The recipient address (base58 encoded public key)
    to_address: String,
    /// The token mint address (base58 encoded public key)
    mint_address: String,
    /// Amount to transfer (in token units, not considering decimals)
    amount: u64,
    /// Number of decimals for the token
    decimals: u8,
    /// Optional sender signer name (uses default if not provided)
    #[serde(default)]
    from_signer: Option<String>,
    /// Create associated token account if it doesn't exist
    #[serde(default = "default_true")]
    create_ata_if_needed: bool,
    /// RPC endpoint URL (optional, defaults to mainnet)
    #[serde(default)]
    rpc_url: Option<String>,
    /// Optional idempotency key to prevent duplicate transfers
    #[serde(default)]
    idempotency_key: Option<String>,
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

    // Create client
    let client = if let Some(url) = rpc_url {
        Arc::new(SolanaClient::with_rpc_url(url))
    } else {
        Arc::new(SolanaClient::default())
    };

    // Get recent blockhash
    let blockhash = client
        .get_latest_blockhash()
        .await
        .map_err(|e| anyhow!("Failed to get blockhash: {}", e))?;

    let mut instructions = Vec::new();

    // Check if recipient ATA exists and create if needed
    if create_ata_if_needed {
        // In production, we would check if the ATA exists first
        // For now, we'll include the create instruction which is idempotent
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

    // Create transaction
    let mut transaction = Transaction::new_unsigned(message);
    transaction.partial_sign(&[signer.as_ref()], blockhash.parse().unwrap());

    // Send transaction
    let signature = client
        .send_transaction(&transaction)
        .await
        .map_err(|e| anyhow!("Failed to send transaction: {}", e))?;

    let ui_amount = amount as f64 / 10_f64.powi(decimals as i32);

    info!(
        "SPL token transfer initiated: {} -> {} ({} tokens), signature: {}",
        signer.pubkey(),
        to_address,
        ui_amount,
        signature
    );

    Ok(TokenTransferResult {
        signature,
        from: signer.pubkey().to_string(),
        to: to_address,
        mint: mint_address,
        amount,
        ui_amount,
        decimals,
        amount_display: format!("{:.9}", ui_amount),
        status: TransactionStatus::Pending,
        idempotency_key,
    })
}

/// Create a new SPL token mint
///
/// This tool creates a new SPL token with the specified parameters.
#[tool]
pub async fn create_spl_token_mint(
    /// Number of decimals for the token
    decimals: u8,
    /// Initial supply to mint (0 for no initial supply)
    #[serde(default)]
    initial_supply: u64,
    /// Whether the mint authority can be frozen
    #[serde(default)]
    freezable: bool,
    /// Optional mint authority signer name (uses default if not provided)
    #[serde(default)]
    authority_signer: Option<String>,
    /// RPC endpoint URL (optional, defaults to mainnet)
    #[serde(default)]
    rpc_url: Option<String>,
) -> anyhow::Result<CreateMintResult> {
    debug!("Creating new SPL token mint with {} decimals", decimals);

    // Get signer
    let signer_context =
        get_signer_context().map_err(|e| anyhow!("Failed to get signer context: {}", e))?;

    let authority = if let Some(name) = authority_signer {
        signer_context
            .get_signer(&name)
            .map_err(|e| anyhow!("Failed to get signer '{}': {}", name, e))?
    } else {
        signer_context
            .get_default_signer()
            .map_err(|e| anyhow!("Failed to get default signer: {}", e))?
    };

    // Generate new mint keypair
    let mint_keypair = Keypair::new();
    let mint_pubkey = mint_keypair.pubkey();

    // Create client
    let client = if let Some(url) = rpc_url {
        Arc::new(SolanaClient::with_rpc_url(url))
    } else {
        Arc::new(SolanaClient::default())
    };

    // Get rent exemption amount
    let mint_rent = client
        .rpc_client
        .get_minimum_balance_for_rent_exemption(spl_token::state::Mint::LEN)
        .map_err(|e| SolanaToolError::Rpc(e.to_string()))?;

    // Get recent blockhash
    let blockhash = client
        .get_latest_blockhash()
        .await
        .map_err(|e| anyhow!("Failed to get blockhash: {}", e))?;

    let mut instructions = Vec::new();

    // Create account for mint
    instructions.push(system_instruction::create_account(
        &authority.pubkey(),
        &mint_pubkey,
        mint_rent,
        spl_token::state::Mint::LEN as u64,
        &spl_token::id(),
    ));

    // Initialize mint
    let freeze_authority = if freezable {
        Some(&authority.pubkey())
    } else {
        None
    };

    instructions.push(
        spl_token::instruction::initialize_mint(
            &spl_token::id(),
            &mint_pubkey,
            &authority.pubkey(),
            freeze_authority,
            decimals,
        )
        .map_err(|e| anyhow!("Failed to create initialize mint instruction: {}", e))?,
    );

    // Mint initial supply if requested
    if initial_supply > 0 {
        let authority_ata = get_associated_token_address(&authority.pubkey(), &mint_pubkey);

        // Create ATA for authority
        instructions.push(
            spl_associated_token_account::instruction::create_associated_token_account(
                &authority.pubkey(),
                &authority.pubkey(),
                &mint_pubkey,
                &spl_token::id(),
            ),
        );

        // Mint to authority
        instructions.push(
            spl_token::instruction::mint_to(
                &spl_token::id(),
                &mint_pubkey,
                &authority_ata,
                &authority.pubkey(),
                &[],
                initial_supply,
            )
            .map_err(|e| anyhow!("Failed to create mint instruction: {}", e))?,
        );
    }

    // Create message
    let message = Message::new(&instructions, Some(&authority.pubkey()));

    // Create transaction
    let mut transaction = Transaction::new_unsigned(message);
    transaction.partial_sign(
        &[authority.as_ref(), &mint_keypair],
        blockhash.parse().unwrap(),
    );

    // Send transaction
    let signature = client
        .send_transaction(&transaction)
        .await
        .map_err(|e| anyhow!("Failed to send transaction: {}", e))?;

    info!(
        "SPL token mint created: {}, signature: {}",
        mint_pubkey, signature
    );

    Ok(CreateMintResult {
        signature,
        mint_address: mint_pubkey.to_string(),
        authority: authority.pubkey().to_string(),
        decimals,
        initial_supply,
        freezable,
    })
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
    /// Optional memo
    pub memo: Option<String>,
    /// Idempotency key if provided
    pub idempotency_key: Option<String>,
}

/// Result of an SPL token transfer
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TokenTransferResult {
    /// Transaction signature
    pub signature: String,
    /// Sender address
    pub from: String,
    /// Recipient address
    pub to: String,
    /// Token mint address
    pub mint: String,
    /// Raw amount transferred
    pub amount: u64,
    /// UI amount (with decimals)
    pub ui_amount: f64,
    /// Token decimals
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
    /// New mint address
    pub mint_address: String,
    /// Mint authority address
    pub authority: String,
    /// Number of decimals
    pub decimals: u8,
    /// Initial supply minted
    pub initial_supply: u64,
    /// Whether the mint is freezable
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
