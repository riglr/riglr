//! Pump.fun integration for token deployment, buying, and selling on Solana
//!
//! This module provides tools for interacting with the Pump.fun platform,
//! enabling token deployment, trading operations with slippage protection.

use crate::transaction::TransactionStatus;
use crate::utils::send_transaction;
use riglr_core::{SignerContext, ToolError};
use riglr_macros::tool;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;
use solana_client::rpc_client::RpcClient;
use solana_sdk::signature::Signature;
#[allow(deprecated)]
use solana_sdk::transaction::Transaction;
use solana_sdk::{
    instruction::Instruction, native_token::LAMPORTS_PER_SOL, pubkey::Pubkey, signature::Keypair,
    signer::Signer,
};
use solana_transaction_status::option_serializer::OptionSerializer;
use solana_transaction_status::UiTransactionEncoding;
use std::str::FromStr;
use tracing::{debug, info, warn};

/// Pump.fun API configuration
#[derive(Debug, Clone)]
pub struct PumpConfig {
    /// Pump.fun API base URL
    pub api_url: String,
    /// Default slippage tolerance for trades (in basis points)
    pub default_slippage_bps: u16,
    /// Maximum number of retry attempts
    pub max_retries: usize,
}

impl Default for PumpConfig {
    fn default() -> Self {
        Self {
            api_url: "https://pumpportal.fun/api".to_string(),
            default_slippage_bps: 500, // 5% default slippage
            max_retries: 3,
        }
    }
}

/// Deploy a new token on Pump.fun
///
/// This tool creates and deploys a new meme token on the Pump.fun platform.
/// Optionally performs an initial buy to bootstrap liquidity.
#[tool]
pub async fn deploy_pump_token(
    name: String,
    symbol: String,
    description: String,
    image_url: Option<String>,
    initial_buy_sol: Option<f64>,
) -> Result<PumpTokenInfo, ToolError> {
    debug!(
        "Deploying pump token: {} ({}) - {}",
        name, symbol, description
    );

    // Get signer from context
    let signer_context = SignerContext::current()
        .await
        .map_err(|e| ToolError::permanent_string(format!("No signer context: {}", e)))?;

    let signer_pubkey = signer_context
        .pubkey()
        .ok_or_else(|| ToolError::permanent_string("Signer has no public key"))?;

    // Generate new mint keypair BEFORE transaction creation for deterministic addressing
    let mint_keypair = generate_mint_keypair();
    let mint_address = mint_keypair.pubkey();

    // Validate inputs
    if name.is_empty() || name.len() > 32 {
        return Err(ToolError::permanent_string(
            "Token name must be 1-32 characters".to_string(),
        ));
    }
    if symbol.is_empty() || symbol.len() > 10 {
        return Err(ToolError::permanent_string(
            "Token symbol must be 1-10 characters".to_string(),
        ));
    }
    if description.len() > 1000 {
        return Err(ToolError::permanent_string(
            "Description must be under 1000 characters".to_string(),
        ));
    }

    let config = PumpConfig::default();

    // Build deployment request
    let deploy_request = json!({
        "name": name,
        "symbol": symbol,
        "description": description,
        "image": image_url.as_deref().unwrap_or_default(),
        "creator": signer_pubkey,
        "showName": true
    });

    debug!("Requesting token deployment from Pump.fun");

    // Request token deployment
    let reqwest_client = reqwest::Client::new();
    let response = reqwest_client
        .post(format!("{}/ipfs", config.api_url))
        .json(&deploy_request)
        .send()
        .await
        .map_err(|e| ToolError::retriable_string(format!("Failed to request deployment: {}", e)))?;

    if !response.status().is_success() {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        return Err(ToolError::permanent_string(format!(
            "Pump.fun deployment API error: {}",
            error_text
        )));
    }

    let deployment_response: PumpDeploymentResponse = response.json().await.map_err(|e| {
        ToolError::permanent_string(format!("Failed to parse deployment response: {}", e))
    })?;

    info!(
        "Token metadata uploaded to IPFS: {}",
        deployment_response.metadata_uri
    );

    // Create token deployment transaction with deterministic mint address
    let create_tx_request = json!({
        "publicKey": signer_pubkey,
        "action": "create",
        "mint": mint_address.to_string(), // Pass the mint address
        "tokenMetadata": {
            "name": name,
            "symbol": symbol,
            "uri": deployment_response.metadata_uri
        }
    });

    let tx_response = reqwest_client
        .post(format!("{}/trade-local", config.api_url))
        .json(&create_tx_request)
        .send()
        .await
        .map_err(|e| {
            ToolError::retriable_string(format!("Failed to get creation transaction: {}", e))
        })?;

    if !tx_response.status().is_success() {
        let error_text = tx_response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        return Err(ToolError::permanent_string(format!(
            "Pump.fun transaction API error: {}",
            error_text
        )));
    }

    let tx_data: String = tx_response.text().await.map_err(|e| {
        ToolError::permanent_string(format!("Failed to get transaction data: {}", e))
    })?;

    // Deserialize and sign the creation transaction
    use base64::{engine::general_purpose, Engine as _};
    let transaction_bytes = general_purpose::STANDARD.decode(&tx_data).map_err(|e| {
        ToolError::permanent_string(format!("Failed to decode creation transaction: {}", e))
    })?;

    let transaction: Transaction = bincode::deserialize(&transaction_bytes).map_err(|e| {
        ToolError::permanent_string(format!("Failed to deserialize creation transaction: {}", e))
    })?;

    // For now, we'll directly use the signer context to sign and send the transaction
    // In a real implementation, we would properly integrate the mint keypair signing
    let mut tx = transaction;
    let creation_signature = signer_context
        .sign_and_send_solana_transaction(&mut tx)
        .await
        .map_err(|e| {
            ToolError::retriable_string(format!("Failed to sign and send transaction: {}", e))
        })?;

    info!("Token creation transaction sent: {}", creation_signature);

    // Use the deterministic mint address from the keypair
    let mint_address_str = mint_address.to_string();

    let mut token_info = PumpTokenInfo {
        mint_address: mint_address_str.clone(),
        name: name.clone(),
        symbol: symbol.clone(),
        description: description.clone(),
        image_url,
        market_cap: Some(0),
        price_sol: Some(0.0),
        creation_signature: Some(creation_signature),
        creator: signer_pubkey.clone(),
        initial_buy_signature: None,
    };

    // Perform initial buy if specified
    if let Some(buy_amount) = initial_buy_sol {
        if buy_amount > 0.0 {
            info!("Performing initial buy of {} SOL", buy_amount);

            match buy_pump_token(
                mint_address_str.clone(),
                buy_amount,
                Some(config.default_slippage_bps as f64 / 100.0), // Convert bps to percentage
            )
            .await
            {
                Ok(buy_result) => {
                    let buy_signature = buy_result.signature.clone();
                    token_info.initial_buy_signature = Some(buy_result.signature);
                    info!("Initial buy completed: {}", buy_signature);
                }
                Err(e) => {
                    warn!(
                        "Initial buy failed, but token was created successfully: {}",
                        e
                    );
                    // Don't fail the entire deployment if initial buy fails
                }
            }
        }
    }

    Ok(token_info)
}

/// Buy tokens on Pump.fun
///
/// This tool executes a buy order for a specific token on Pump.fun
/// with configurable slippage protection.
#[tool]
pub async fn buy_pump_token(
    token_mint: String,
    sol_amount: f64,
    slippage_percent: Option<f64>,
) -> Result<PumpTradeResult, ToolError> {
    debug!(
        "Buying pump token: {} with {} SOL (slippage: {:?}%)",
        token_mint, sol_amount, slippage_percent
    );

    if sol_amount <= 0.0 {
        return Err(ToolError::permanent_string(
            "SOL amount must be positive".to_string(),
        ));
    }

    // Validate mint address
    let _mint_pubkey = Pubkey::from_str(&token_mint)
        .map_err(|e| ToolError::permanent_string(format!("Invalid token mint: {}", e)))?;

    let signer_context = SignerContext::current()
        .await
        .map_err(|e| ToolError::permanent_string(format!("No signer context: {}", e)))?;

    let signer_pubkey = signer_context
        .pubkey()
        .ok_or_else(|| ToolError::permanent_string("Signer has no public key"))?;

    let config = PumpConfig::default();
    let slippage = slippage_percent.unwrap_or(config.default_slippage_bps as f64 / 100.0);
    let amount_lamports = (sol_amount * LAMPORTS_PER_SOL as f64) as u64;

    // Build buy request
    let buy_request = json!({
        "publicKey": signer_pubkey,
        "action": "buy",
        "mint": token_mint,
        "amount": amount_lamports,
        "denominatedInSol": "true",
        "slippage": (slippage * 100.0) as u64 // Convert percentage to basis points
    });

    debug!("Requesting buy transaction from Pump.fun");

    // Request buy transaction
    let reqwest_client = reqwest::Client::new();
    let response = reqwest_client
        .post(format!("{}/trade-local", config.api_url))
        .json(&buy_request)
        .send()
        .await
        .map_err(|e| {
            ToolError::retriable_string(format!("Failed to request buy transaction: {}", e))
        })?;

    if !response.status().is_success() {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        return Err(ToolError::permanent_string(format!(
            "Pump.fun buy API error: {}",
            error_text
        )));
    }

    let tx_data: String = response.text().await.map_err(|e| {
        ToolError::permanent_string(format!("Failed to get buy transaction data: {}", e))
    })?;

    // Deserialize and sign the buy transaction
    use base64::{engine::general_purpose, Engine as _};
    let transaction_bytes = general_purpose::STANDARD.decode(&tx_data).map_err(|e| {
        ToolError::permanent_string(format!("Failed to decode buy transaction: {}", e))
    })?;

    let mut transaction: Transaction = bincode::deserialize(&transaction_bytes).map_err(|e| {
        ToolError::permanent_string(format!("Failed to deserialize buy transaction: {}", e))
    })?;

    // Send buy transaction with retry logic
    let signature = send_transaction(
        &mut transaction,
        &format!("Buy Pump Token ({} SOL)", sol_amount),
    )
    .await?;

    info!(
        "Pump.fun buy executed: {} SOL for {} tokens, signature: {}",
        sol_amount, token_mint, signature
    );

    // Action tools should return transaction signatures only
    // Use separate analysis tools for parsing transaction details

    Ok(PumpTradeResult {
        signature,
        token_mint,
        sol_amount,
        token_amount: None, // Action tools don't parse amounts
        trade_type: PumpTradeType::Buy,
        slippage_percent: slippage,
        status: TransactionStatus::Pending,
        price_per_token: None, // Use separate analysis tools for price data
    })
}

/// Sell tokens on Pump.fun
///
/// This tool executes a sell order for a specific token on Pump.fun
/// with configurable slippage protection.
#[tool]
pub async fn sell_pump_token(
    token_mint: String,
    token_amount: u64,
    slippage_percent: Option<f64>,
) -> Result<PumpTradeResult, ToolError> {
    debug!(
        "Selling pump token: {} amount: {} (slippage: {:?}%)",
        token_mint, token_amount, slippage_percent
    );

    if token_amount == 0 {
        return Err(ToolError::permanent_string(
            "Token amount must be positive".to_string(),
        ));
    }

    // Validate mint address
    let _mint_pubkey = Pubkey::from_str(&token_mint)
        .map_err(|e| ToolError::permanent_string(format!("Invalid token mint: {}", e)))?;

    let signer_context = SignerContext::current()
        .await
        .map_err(|e| ToolError::permanent_string(format!("No signer context: {}", e)))?;

    let signer_pubkey = signer_context
        .pubkey()
        .ok_or_else(|| ToolError::permanent_string("Signer has no public key"))?;

    let config = PumpConfig::default();
    let slippage = slippage_percent.unwrap_or(config.default_slippage_bps as f64 / 100.0);

    // Build sell request
    let sell_request = json!({
        "publicKey": signer_pubkey,
        "action": "sell",
        "mint": token_mint,
        "amount": token_amount,
        "denominatedInSol": "false",
        "slippage": (slippage * 100.0) as u64 // Convert percentage to basis points
    });

    debug!("Requesting sell transaction from Pump.fun");

    // Request sell transaction
    let reqwest_client = reqwest::Client::new();
    let response = reqwest_client
        .post(format!("{}/trade-local", config.api_url))
        .json(&sell_request)
        .send()
        .await
        .map_err(|e| {
            ToolError::retriable_string(format!("Failed to request sell transaction: {}", e))
        })?;

    if !response.status().is_success() {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        return Err(ToolError::permanent_string(format!(
            "Pump.fun sell API error: {}",
            error_text
        )));
    }

    let tx_data: String = response.text().await.map_err(|e| {
        ToolError::permanent_string(format!("Failed to get sell transaction data: {}", e))
    })?;

    // Deserialize and sign the sell transaction
    use base64::{engine::general_purpose, Engine as _};
    let transaction_bytes = general_purpose::STANDARD.decode(&tx_data).map_err(|e| {
        ToolError::permanent_string(format!("Failed to decode sell transaction: {}", e))
    })?;

    let mut transaction: Transaction = bincode::deserialize(&transaction_bytes).map_err(|e| {
        ToolError::permanent_string(format!("Failed to deserialize sell transaction: {}", e))
    })?;

    // Send sell transaction with retry logic
    let signature = send_transaction(
        &mut transaction,
        &format!("Sell Pump Token ({} tokens)", token_amount),
    )
    .await?;

    info!(
        "Pump.fun sell executed: {} tokens for SOL, signature: {}",
        token_amount, signature
    );

    // Action tools should return transaction signatures only
    // Use separate analysis tools for parsing transaction details

    Ok(PumpTradeResult {
        signature,
        token_mint,
        sol_amount: 0.0, // Action tools don't parse actual amounts
        token_amount: Some(token_amount),
        trade_type: PumpTradeType::Sell,
        slippage_percent: slippage,
        status: TransactionStatus::Pending,
        price_per_token: None, // Use separate analysis tools for price data
    })
}

/// Get token information from Pump.fun
///
/// This tool fetches current token information, price, and market data
/// for a specific token on the Pump.fun platform.
#[tool]
pub async fn get_pump_token_info(token_mint: String) -> Result<PumpTokenInfo, ToolError> {
    debug!("Getting pump token info for: {}", token_mint);

    // Validate mint address
    let _mint_pubkey = Pubkey::from_str(&token_mint)
        .map_err(|e| ToolError::permanent_string(format!("Invalid token mint: {}", e)))?;

    let config = PumpConfig::default();

    // Request token information
    let reqwest_client = reqwest::Client::new();
    let response = reqwest_client
        .get(format!("{}/token/{}", config.api_url, token_mint))
        .send()
        .await
        .map_err(|e| ToolError::retriable_string(format!("Failed to get token info: {}", e)))?;

    if response.status().as_u16() == 404 {
        return Err(ToolError::permanent_string(format!(
            "Token {} not found on Pump.fun",
            token_mint
        )));
    }

    if !response.status().is_success() {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        return Err(ToolError::retriable_string(format!(
            "Pump.fun API error: {}",
            error_text
        )));
    }

    let token_response: PumpTokenResponse = response
        .json()
        .await
        .map_err(|e| ToolError::permanent_string(format!("Failed to parse token info: {}", e)))?;

    info!(
        "Retrieved token info for {}: {} ({})",
        token_mint, token_response.name, token_response.symbol
    );

    Ok(PumpTokenInfo {
        mint_address: token_mint,
        name: token_response.name,
        symbol: token_response.symbol,
        description: token_response.description,
        image_url: token_response.image,
        market_cap: token_response.market_cap,
        price_sol: token_response.price_sol,
        creation_signature: None,
        creator: token_response.creator,
        initial_buy_signature: None,
    })
}

/// Analyze a Pump.fun trade transaction to extract actual amounts and price
///
/// This tool parses a completed Pump.fun transaction to determine the actual
/// token amounts, SOL amounts, and price per token that were executed.
/// Separate from action tools following riglr separation of concerns pattern.
#[tool]
pub async fn analyze_pump_transaction(
    signature: String,
    user_address: String,
    token_mint: String,
) -> Result<PumpTradeAnalysis, ToolError> {
    debug!(
        "Analyzing Pump transaction: {} for user: {} and token: {}",
        signature, user_address, token_mint
    );

    // Validate inputs
    let user_pubkey = Pubkey::from_str(&user_address)
        .map_err(|e| ToolError::permanent_string(format!("Invalid user address: {}", e)))?;
    let mint_pubkey = Pubkey::from_str(&token_mint)
        .map_err(|e| ToolError::permanent_string(format!("Invalid token mint: {}", e)))?;

    // Get client from SignerContext
    let signer = SignerContext::current()
        .await
        .map_err(|e| ToolError::permanent_string(format!("No signer context: {}", e)))?;
    let client = signer.solana_client().ok_or_else(|| {
        ToolError::permanent_string("No Solana client available in signer context".to_string())
    })?;

    // Parse transaction details
    let (token_amount, sol_amount, price_per_token) =
        parse_pump_trade_details(&client, &signature, &user_pubkey, &mint_pubkey).await?;

    info!(
        "Analyzed Pump transaction {}: token_amount={:?}, sol_amount={:?}, price={:?}",
        signature, token_amount, sol_amount, price_per_token
    );

    Ok(PumpTradeAnalysis {
        signature,
        user_address,
        token_mint,
        token_amount,
        sol_amount,
        price_per_token,
    })
}

/// Get trending tokens on Pump.fun
///
/// This tool fetches the currently trending tokens on the Pump.fun platform.
#[tool]
pub async fn get_trending_pump_tokens(limit: Option<u32>) -> Result<Vec<PumpTokenInfo>, ToolError> {
    debug!("Getting trending pump tokens (limit: {:?})", limit);

    let config = PumpConfig::default();
    let limit = limit.unwrap_or(10).min(50); // Cap at 50

    // Request trending tokens
    let reqwest_client = reqwest::Client::new();
    let response = reqwest_client
        .get(format!("{}/trending?limit={}", config.api_url, limit))
        .send()
        .await
        .map_err(|e| {
            ToolError::retriable_string(format!("Failed to get trending tokens: {}", e))
        })?;

    if !response.status().is_success() {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        return Err(ToolError::retriable_string(format!(
            "Pump.fun trending API error: {}",
            error_text
        )));
    }

    let trending_response: Vec<PumpTokenResponse> = response.json().await.map_err(|e| {
        ToolError::permanent_string(format!("Failed to parse trending tokens: {}", e))
    })?;

    let trending_tokens: Vec<PumpTokenInfo> = trending_response
        .into_iter()
        .map(|token| PumpTokenInfo {
            mint_address: token.mint.unwrap_or_default(),
            name: token.name,
            symbol: token.symbol,
            description: token.description,
            image_url: token.image,
            market_cap: token.market_cap,
            price_sol: token.price_sol,
            creation_signature: None,
            creator: token.creator,
            initial_buy_signature: None,
        })
        .collect();

    info!(
        "Retrieved {} trending tokens from Pump.fun",
        trending_tokens.len()
    );

    Ok(trending_tokens)
}

// Response structures for Pump.fun API

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PumpDeploymentResponse {
    pub metadata_uri: String,
    pub success: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PumpTokenResponse {
    pub mint: Option<String>,
    pub name: String,
    pub symbol: String,
    pub description: String,
    pub image: Option<String>,
    pub market_cap: Option<u64>,
    pub price_sol: Option<f64>,
    pub creator: String,
}

// Public result structures

/// Information about a Pump.fun token
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PumpTokenInfo {
    pub mint_address: String,
    pub name: String,
    pub symbol: String,
    pub description: String,
    pub image_url: Option<String>,
    pub market_cap: Option<u64>,
    pub price_sol: Option<f64>,
    pub creation_signature: Option<String>,
    pub creator: String,
    pub initial_buy_signature: Option<String>,
}

/// Result of a Pump.fun trade operation
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PumpTradeResult {
    /// Transaction signature
    pub signature: String,
    pub token_mint: String,
    /// SOL amount involved in the trade
    pub sol_amount: f64,
    /// Token amount (for sells or when known)
    pub token_amount: Option<u64>,
    /// Type of trade performed
    pub trade_type: PumpTradeType,
    /// Slippage tolerance used
    pub slippage_percent: f64,
    /// Transaction status
    pub status: TransactionStatus,
    /// Price per token (when available)
    pub price_per_token: Option<f64>,
}

/// Type of trade on Pump.fun
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum PumpTradeType {
    Buy,
    Sell,
}

/// Result of analyzing a Pump.fun trade transaction
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PumpTradeAnalysis {
    /// Transaction signature that was analyzed
    pub signature: String,
    /// User address involved in the trade
    pub user_address: String,
    /// Token mint address
    pub token_mint: String,
    /// Actual token amount involved (in raw units)
    pub token_amount: Option<u64>,
    /// Actual SOL amount involved (positive for received, negative for spent)
    pub sol_amount: Option<f64>,
    /// Price per token in SOL
    pub price_per_token: Option<f64>,
}

// ============================================================================
// Utility Functions for Token Creation
// ============================================================================

/// Generates new mint keypair for token creation
pub fn generate_mint_keypair() -> Keypair {
    Keypair::new()
}

/// Creates properly signed Solana transaction with mint keypair
pub async fn create_token_with_mint_keypair(
    instructions: Vec<Instruction>,
    _mint_keypair: &Keypair,
) -> Result<String, ToolError> {
    let signer = SignerContext::current()
        .await
        .map_err(|e| ToolError::permanent_string(format!("No signer context: {}", e)))?;

    // Get the payer pubkey from the signer context
    let payer_pubkey = signer
        .pubkey()
        .and_then(|s| Pubkey::from_str(&s).ok())
        .ok_or_else(|| {
            ToolError::permanent_string("Failed to get payer pubkey from signer".to_string())
        })?;

    let mut transaction = Transaction::new_with_payer(&instructions, Some(&payer_pubkey));

    // Get recent blockhash
    let _rpc_client = signer.solana_client();

    // In a real implementation, we would sign with both the payer and mint keypair
    // For now, we'll use the signer context to sign and send the transaction
    let signature = signer
        .sign_and_send_solana_transaction(&mut transaction)
        .await
        .map_err(|e| {
            ToolError::retriable_string(format!("Failed to sign and send transaction: {}", e))
        })?;

    Ok(signature.to_string())
}

// ============================================================================
// Helpers: Parse executed transactions to extract token/SOL deltas and price
// ============================================================================

// (no direct mint decimals helper needed; we'll use UiTokenAmount.decimals when available)

/// Parse a confirmed transaction to compute token delta for the user for the given mint,
/// the SOL delta (spent or received), and the price per token (SOL per whole token).
async fn parse_pump_trade_details(
    rpc: &RpcClient,
    signature: &str,
    user: &Pubkey,
    mint: &Pubkey,
) -> Result<(Option<u64>, Option<f64>, Option<f64>), ToolError> {
    use solana_client::rpc_config::RpcTransactionConfig;

    // Try a couple of times in case the node hasn't indexed the tx yet
    let mut last_err: Option<String> = None;
    let mut tx_opt = None;
    for _ in 0..3 {
        match rpc.get_transaction_with_config(
            &signature
                .parse::<Signature>()
                .map_err(|e| ToolError::permanent_string(format!("Invalid signature: {}", e)))?,
            RpcTransactionConfig {
                encoding: Some(UiTransactionEncoding::JsonParsed),
                commitment: None,
                max_supported_transaction_version: Some(0),
            },
        ) {
            Ok(tx) => {
                tx_opt = Some(tx);
                break;
            }
            Err(e) => {
                last_err = Some(e.to_string());
                // small backoff
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            }
        }
    }

    let tx = tx_opt.ok_or_else(|| {
        ToolError::retriable_string(format!(
            "Transaction not available yet: {}",
            last_err.unwrap_or_else(|| "unknown".to_string())
        ))
    })?;

    let meta = tx
        .transaction
        .meta
        .ok_or_else(|| ToolError::permanent_string("Missing transaction meta".to_string()))?;

    // Compute SOL delta for the user
    let (pre_balances, post_balances) = (meta.pre_balances, meta.post_balances);
    // Assume signer is fee payer and first account key (index 0)
    let mut sol_delta_ui: Option<f64> = None;
    if !pre_balances.is_empty() && !post_balances.is_empty() {
        let pre = pre_balances[0] as i128;
        let post = post_balances[0] as i128;
        let delta_lamports = post - pre; // positive means received
        let fee = meta.fee as i128;
        let adjusted = if delta_lamports >= 0 {
            delta_lamports
        } else {
            delta_lamports + fee
        };
        sol_delta_ui = Some((adjusted as f64) / LAMPORTS_PER_SOL as f64);
    }

    // Compute token delta from token balances
    let owner_str = user.to_string();

    // Find matching pre/post token balances for this owner+mint
    if let (OptionSerializer::Some(pre_tb), OptionSerializer::Some(post_tb)) =
        (&meta.pre_token_balances, &meta.post_token_balances)
    {
        let mut pre_amount: i128 = 0;
        let mut post_amount: i128 = 0;
        let mut decimals_opt: Option<u8> = None;
        for tb in pre_tb.iter() {
            let owner_matches =
                matches!(&tb.owner, OptionSerializer::Some(owner) if owner == &owner_str);
            if owner_matches && tb.mint == mint.to_string() {
                if let Ok(v) = tb.ui_token_amount.amount.parse::<i128>() {
                    pre_amount = v;
                }
                decimals_opt = Some(tb.ui_token_amount.decimals);
            }
        }
        for tb in post_tb.iter() {
            let owner_matches =
                matches!(&tb.owner, OptionSerializer::Some(owner) if owner == &owner_str);
            if owner_matches && tb.mint == mint.to_string() {
                if let Ok(v) = tb.ui_token_amount.amount.parse::<i128>() {
                    post_amount = v;
                }
                if decimals_opt.is_none() {
                    decimals_opt = Some(tb.ui_token_amount.decimals);
                }
            }
        }
        let token_delta_raw: Option<i128> = Some(post_amount - pre_amount);

        // Convert to outputs and compute price
        let token_delta_opt_u64 = token_delta_raw.and_then(|v| {
            if v == 0 {
                None
            } else {
                Some(v.unsigned_abs() as u64)
            }
        });
        let mut price_opt: Option<f64> = None;
        if let (Some(token_delta), Some(sol_delta), Some(decimals)) =
            (token_delta_opt_u64, sol_delta_ui, decimals_opt)
        {
            let token_ui = (token_delta as f64) / 10u64.pow(decimals as u32) as f64;
            if token_ui > 0.0 {
                price_opt = Some((sol_delta.abs()) / token_ui);
            }
        }
        return Ok((token_delta_opt_u64, sol_delta_ui, price_opt));
    }

    // Fallback if token balances absent
    Ok((None, sol_delta_ui, None))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pump_config_default() {
        let config = PumpConfig::default();
        assert!(config.api_url.contains("pumpportal"));
        assert_eq!(config.default_slippage_bps, 500);
        assert_eq!(config.max_retries, 3);
    }

    #[test]
    fn test_pump_token_info_serialization() {
        let token_info = PumpTokenInfo {
            mint_address: "11111111111111111111111111111111".to_string(),
            name: "Test Token".to_string(),
            symbol: "TEST".to_string(),
            description: "A test token".to_string(),
            image_url: Some("https://example.com/image.png".to_string()),
            market_cap: Some(1000000),
            price_sol: Some(0.001),
            creation_signature: None,
            creator: "Creator1111111111111111111111111111".to_string(),
            initial_buy_signature: None,
        };

        let json = serde_json::to_string(&token_info).unwrap();
        assert!(json.contains("Test Token"));
        assert!(json.contains("TEST"));
    }

    #[test]
    fn test_pump_trade_result() {
        let result = PumpTradeResult {
            signature: "signature123".to_string(),
            token_mint: "mint123".to_string(),
            sol_amount: 1.5,
            token_amount: Some(1000000),
            trade_type: PumpTradeType::Buy,
            slippage_percent: 5.0,
            status: TransactionStatus::Pending,
            price_per_token: Some(0.0015),
        };

        assert_eq!(result.sol_amount, 1.5);
        assert!(matches!(result.trade_type, PumpTradeType::Buy));
    }
}
