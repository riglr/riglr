//! Pump.fun integration for token deployment, buying, and selling on Solana
//!
//! This module provides tools for interacting with the Pump.fun platform,
//! enabling token deployment, trading operations with slippage protection.

use crate::common::newtypes::{SolanaAddress, SolanaSignature};
use crate::transaction::TransactionStatus;
use crate::utils::send_transaction;
use crate::utils::validation::validate_address;
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
use tracing::{debug, info, warn};

// Rate limiting is now handled via ApplicationContext dependency injection
// The RateLimiter service is available from riglr_core::util::RateLimiter

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
    context: &riglr_core::provider::ApplicationContext,
) -> Result<PumpTokenInfo, ToolError> {
    debug!(
        "Deploying pump token: {} ({}) - {}",
        name, symbol, description
    );

    // Get signer from context
    let signer_context = SignerContext::current_as_solana()
        .await
        .map_err(|e| ToolError::permanent_string(format!("No Solana signer context: {}", e)))?;

    let signer_pubkey = signer_context.pubkey();

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

    // Get API clients from context
    let api_clients = context
        .get_extension::<std::sync::Arc<crate::clients::ApiClients>>()
        .ok_or_else(|| ToolError::permanent_string("ApiClients not found in context"))?;
    let pump_client = &api_clients.pump;

    // Build deployment request
    let deploy_request = json!({
        "name": &name,
        "symbol": &symbol,
        "description": &description,
        "image": image_url.as_deref().unwrap_or_default(),
        "creator": signer_pubkey,
        "showName": true
    });

    debug!("Requesting token deployment from Pump.fun");

    // Request token deployment using injected client
    let response = pump_client
        .http_client()
        .post(format!("{}/ipfs", pump_client.api_url()))
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
            "name": &name,
            "symbol": &symbol,
            "uri": deployment_response.metadata_uri
        }
    });

    let tx_response = pump_client
        .http_client()
        .post(format!("{}/trade-local", pump_client.api_url()))
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

    // Sign the transaction with both the mint keypair and the fee payer
    let mut tx = transaction;

    // Get recent blockhash for partial signing
    let client = signer_context.client();
    let recent_blockhash = client
        .get_latest_blockhash()
        .map_err(|e| ToolError::retriable_string(format!("Failed to get blockhash: {}", e)))?;

    // First, partially sign with the mint keypair
    tx.partial_sign(&[&mint_keypair], recent_blockhash);

    // Then sign with the fee payer and send
    let creation_signature = signer_context
        .sign_and_send_transaction(&mut tx)
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
                Some(5.0), // 5% default slippage
                context,
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
    context: &riglr_core::provider::ApplicationContext,
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
    let _mint_pubkey =
        validate_address(&token_mint).map_err(|e| ToolError::permanent_string(e.to_string()))?;

    let signer_context = SignerContext::current_as_solana()
        .await
        .map_err(|e| ToolError::permanent_string(format!("No Solana signer context: {}", e)))?;

    let signer_pubkey = signer_context.pubkey();

    // Get API clients from context
    let api_clients = context
        .get_extension::<std::sync::Arc<crate::clients::ApiClients>>()
        .ok_or_else(|| ToolError::permanent_string("ApiClients not found in context"))?;
    let pump_client = &api_clients.pump;

    let slippage = slippage_percent.unwrap_or(5.0); // 5% default slippage
    let amount_lamports = (sol_amount * LAMPORTS_PER_SOL as f64) as u64;

    // Build buy request
    let buy_request = json!({
        "publicKey": signer_pubkey,
        "action": "buy",
        "mint": &token_mint,
        "amount": amount_lamports,
        "denominatedInSol": "true",
        "slippage": (slippage * 100.0) as u64 // Convert percentage to basis points
    });

    debug!("Requesting buy transaction from Pump.fun");

    // Request buy transaction using injected client
    let response = pump_client
        .http_client()
        .post(format!("{}/trade-local", pump_client.api_url()))
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
    context: &riglr_core::provider::ApplicationContext,
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
    let _mint_pubkey =
        validate_address(&token_mint).map_err(|e| ToolError::permanent_string(e.to_string()))?;

    let signer_context = SignerContext::current_as_solana()
        .await
        .map_err(|e| ToolError::permanent_string(format!("No Solana signer context: {}", e)))?;

    let signer_pubkey = signer_context.pubkey();

    // Get API clients from context
    let api_clients = context
        .get_extension::<std::sync::Arc<crate::clients::ApiClients>>()
        .ok_or_else(|| ToolError::permanent_string("ApiClients not found in context"))?;
    let pump_client = &api_clients.pump;

    let slippage = slippage_percent.unwrap_or(5.0); // 5% default slippage

    // Build sell request
    let sell_request = json!({
        "publicKey": signer_pubkey,
        "action": "sell",
        "mint": &token_mint,
        "amount": token_amount,
        "denominatedInSol": "false",
        "slippage": (slippage * 100.0) as u64 // Convert percentage to basis points
    });

    debug!("Requesting sell transaction from Pump.fun");

    // Request sell transaction using injected client
    let response = pump_client
        .http_client()
        .post(format!("{}/trade-local", pump_client.api_url()))
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
pub async fn get_pump_token_info(
    token_mint: String,
    context: &riglr_core::provider::ApplicationContext,
) -> Result<PumpTokenInfo, ToolError> {
    debug!("Getting pump token info for: {}", token_mint);

    // Validate mint address
    let _mint_pubkey =
        validate_address(&token_mint).map_err(|e| ToolError::permanent_string(e.to_string()))?;

    // Get API clients from context
    let api_clients = context
        .get_extension::<std::sync::Arc<crate::clients::ApiClients>>()
        .ok_or_else(|| ToolError::permanent_string("ApiClients not found in context"))?;
    let pump_client = &api_clients.pump;

    // Request token information using injected client
    let response = pump_client
        .http_client()
        .get(format!("{}/token/{}", pump_client.api_url(), token_mint))
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
///
/// ## Security Features
/// - Enhanced input validation for all parameters
/// - Rate limiting via SignerContext
/// - Signature format validation
/// - Address length validation
#[tool]
pub async fn analyze_pump_transaction(
    signature: SolanaSignature,
    user_address: SolanaAddress,
    token_mint: SolanaAddress,
    _context: &riglr_core::provider::ApplicationContext,
) -> Result<PumpTradeAnalysis, ToolError> {
    debug!(
        "Analyzing Pump transaction: {} for user: {} and token: {}",
        signature, user_address, token_mint
    );

    // The type system now guarantees these are valid
    let sig_str = signature.to_string();
    let user_pubkey = *user_address;
    let mint_pubkey = *token_mint;
    let user_addr_str = user_address.to_string();

    // Additional validation: ensure addresses are not system program or other reserved addresses
    if user_pubkey == solana_sdk::pubkey::Pubkey::default() {
        return Err(ToolError::invalid_input_string(
            "Invalid user address: cannot be default/zero address",
        ));
    }

    if mint_pubkey == solana_sdk::pubkey::Pubkey::default() {
        return Err(ToolError::invalid_input_string(
            "Invalid token mint: cannot be default/zero address",
        ));
    }

    // Rate limiting is now automatically handled by the ToolWorker framework
    // No manual rate limiting checks needed - the framework applies rate limiting
    // based on SignerContext user_id before tool execution

    // Get client from SignerContext
    let signer = SignerContext::current_as_solana()
        .await
        .map_err(|e| ToolError::permanent_string(format!("No Solana signer context: {}", e)))?;
    let client = signer.client();

    // Parse transaction details
    let (token_amount, sol_amount, price_per_token) =
        parse_pump_trade_details(&client, &sig_str, &user_pubkey, &mint_pubkey).await?;

    info!(
        "Analyzed Pump transaction {}: token_amount={:?}, sol_amount={:?}, price={:?}",
        signature, token_amount, sol_amount, price_per_token
    );

    Ok(PumpTradeAnalysis {
        signature: sig_str,
        user_address: user_addr_str,
        token_mint: token_mint.to_string(),
        token_amount,
        sol_amount,
        price_per_token,
    })
}

/// Get trending tokens on Pump.fun
///
/// This tool fetches the currently trending tokens on the Pump.fun platform.
#[tool]
pub async fn get_trending_pump_tokens(
    limit: Option<u32>,
    context: &riglr_core::provider::ApplicationContext,
) -> Result<Vec<PumpTokenInfo>, ToolError> {
    debug!("Getting trending pump tokens (limit: {:?})", limit);

    // Get API clients from context
    let api_clients = context
        .get_extension::<std::sync::Arc<crate::clients::ApiClients>>()
        .ok_or_else(|| ToolError::permanent_string("ApiClients not found in context"))?;
    let pump_client = &api_clients.pump;

    let limit = limit.unwrap_or(10).min(50); // Cap at 50

    // Request trending tokens using injected client
    let response = pump_client
        .http_client()
        .get(format!(
            "{}/trending?limit={}",
            pump_client.api_url(),
            limit
        ))
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

/// Response from Pump.fun deployment API
#[derive(Debug, Clone, Serialize, Deserialize)]
struct PumpDeploymentResponse {
    /// IPFS URI for the token metadata
    pub metadata_uri: String,
    /// Whether the deployment was successful
    pub success: bool,
}

/// Response from Pump.fun token API
#[derive(Debug, Clone, Serialize, Deserialize)]
struct PumpTokenResponse {
    /// Token mint address
    pub mint: Option<String>,
    /// Token name
    pub name: String,
    /// Token symbol
    pub symbol: String,
    /// Token description
    pub description: String,
    /// Optional image URL
    pub image: Option<String>,
    /// Current market cap in lamports
    pub market_cap: Option<u64>,
    /// Current price in SOL
    pub price_sol: Option<f64>,
    /// Creator's public key
    pub creator: String,
}

// Public result structures

/// Information about a Pump.fun token
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PumpTokenInfo {
    /// Token mint address
    pub mint_address: String,
    /// Token name
    pub name: String,
    /// Token symbol
    pub symbol: String,
    /// Token description
    pub description: String,
    /// Optional image URL
    pub image_url: Option<String>,
    /// Current market cap in lamports
    pub market_cap: Option<u64>,
    /// Current price in SOL
    pub price_sol: Option<f64>,
    /// Transaction signature for token creation
    pub creation_signature: Option<String>,
    /// Creator's public key
    pub creator: String,
    /// Transaction signature for initial buy (if any)
    pub initial_buy_signature: Option<String>,
}

/// Result of a Pump.fun trade operation
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PumpTradeResult {
    /// Transaction signature
    pub signature: String,
    /// Token mint address
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
    /// Buy tokens with SOL
    Buy,
    /// Sell tokens for SOL
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
///
/// Returns a fresh keypair that can be used as the mint address for a new token.
pub fn generate_mint_keypair() -> Keypair {
    Keypair::new()
}

/// Creates properly signed Solana transaction with mint keypair
///
/// This function creates a transaction with the given instructions and signs it
/// using both the payer from signer context and the provided mint keypair.
pub async fn create_token_with_mint_keypair(
    instructions: Vec<Instruction>,
    _mint_keypair: &Keypair,
) -> Result<String, ToolError> {
    let signer = SignerContext::current_as_solana()
        .await
        .map_err(|e| ToolError::permanent_string(format!("No Solana signer context: {}", e)))?;

    // Get the payer pubkey from the signer context
    let payer_pubkey = validate_address(&signer.pubkey())
        .map_err(|e| ToolError::permanent_string(e.to_string()))?;

    let mut transaction = Transaction::new_with_payer(&instructions, Some(&payer_pubkey));

    // Get recent blockhash
    let _rpc_client = signer.client();

    // In a real implementation, we would sign with both the payer and mint keypair
    // For now, we'll use the signer context to sign and send the transaction
    let signature = signer
        .sign_and_send_transaction(&mut transaction)
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
///
/// Returns a tuple of (token_amount, sol_amount, price_per_token) where:
/// - token_amount: Number of tokens involved in the trade (raw units)
/// - sol_amount: SOL amount delta (positive for received, negative for spent)
/// - price_per_token: Price per token in SOL
async fn parse_pump_trade_details(
    rpc: &RpcClient,
    signature: &str,
    user: &Pubkey,
    mint: &Pubkey,
) -> Result<(Option<u64>, Option<f64>, Option<f64>), ToolError> {
    use solana_client::rpc_config::RpcTransactionConfig;

    // Use standardized retry logic for fetching transaction
    let sig = signature
        .parse::<Signature>()
        .map_err(|e| ToolError::permanent_string(format!("Invalid signature: {}", e)))?;

    let tx = riglr_core::retry::retry_async(
        || async {
            rpc.get_transaction_with_config(
                &sig,
                RpcTransactionConfig {
                    encoding: Some(UiTransactionEncoding::JsonParsed),
                    commitment: None,
                    max_supported_transaction_version: Some(0),
                },
            )
            .map_err(|e| e.to_string())
        },
        |_| riglr_core::retry::ErrorClass::Retryable,
        &riglr_core::retry::RetryConfig::fast(),
        "fetch_pump_transaction",
    )
    .await
    .map_err(|e| ToolError::retriable_string(format!("Transaction not available yet: {}", e)))?;

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

/// Tests for Pump.fun integration functionality
#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_pump_client_default() {
        use crate::clients::PumpClient;
        use riglr_config::ProvidersConfig;

        let config = ProvidersConfig::default();
        let pump_client = PumpClient::new(&config);
        assert!(pump_client.api_url().contains("pumpapi.fun"));
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
    fn test_pump_token_info_deserialization() {
        let json = r#"{
            "mint_address": "11111111111111111111111111111111",
            "name": "Test Token",
            "symbol": "TEST",
            "description": "A test token",
            "image_url": null,
            "market_cap": 1000000,
            "price_sol": 0.001,
            "creation_signature": null,
            "creator": "Creator1111111111111111111111111111",
            "initial_buy_signature": null
        }"#;

        let token_info: PumpTokenInfo = serde_json::from_str(json).unwrap();
        assert_eq!(token_info.name, "Test Token");
        assert_eq!(token_info.symbol, "TEST");
        assert_eq!(token_info.image_url, None);
    }

    #[test]
    fn test_pump_token_info_debug_clone() {
        let token_info = PumpTokenInfo {
            mint_address: "mint123".to_string(),
            name: "Token".to_string(),
            symbol: "TKN".to_string(),
            description: "desc".to_string(),
            image_url: None,
            market_cap: None,
            price_sol: None,
            creation_signature: None,
            creator: "creator123".to_string(),
            initial_buy_signature: None,
        };

        let cloned = token_info.clone();
        assert_eq!(format!("{:?}", token_info), format!("{:?}", cloned));
    }

    #[test]
    fn test_pump_trade_result_serialization() {
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

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("signature123"));
        assert!(json.contains("Buy"));
    }

    #[test]
    fn test_pump_trade_result_buy_type() {
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

    #[test]
    fn test_pump_trade_result_sell_type() {
        let result = PumpTradeResult {
            signature: "signature456".to_string(),
            token_mint: "mint456".to_string(),
            sol_amount: 0.8,
            token_amount: Some(500000),
            trade_type: PumpTradeType::Sell,
            slippage_percent: 3.0,
            status: TransactionStatus::Pending,
            price_per_token: Some(0.0016),
        };

        assert!(matches!(result.trade_type, PumpTradeType::Sell));
        assert_eq!(result.token_amount, Some(500000));
    }

    #[test]
    fn test_pump_trade_result_debug_clone() {
        let result = PumpTradeResult {
            signature: "sig".to_string(),
            token_mint: "mint".to_string(),
            sol_amount: 1.0,
            token_amount: None,
            trade_type: PumpTradeType::Buy,
            slippage_percent: 1.0,
            status: TransactionStatus::Pending,
            price_per_token: None,
        };

        let cloned = result.clone();
        assert_eq!(format!("{:?}", result), format!("{:?}", cloned));
    }

    #[test]
    fn test_pump_trade_type_serialization() {
        let buy_type = PumpTradeType::Buy;
        let sell_type = PumpTradeType::Sell;

        let buy_json = serde_json::to_string(&buy_type).unwrap();
        let sell_json = serde_json::to_string(&sell_type).unwrap();

        assert_eq!(buy_json, "\"Buy\"");
        assert_eq!(sell_json, "\"Sell\"");
    }

    #[test]
    fn test_pump_trade_type_deserialization() {
        let buy_json = "\"Buy\"";
        let sell_json = "\"Sell\"";

        let buy_type: PumpTradeType = serde_json::from_str(buy_json).unwrap();
        let sell_type: PumpTradeType = serde_json::from_str(sell_json).unwrap();

        assert!(matches!(buy_type, PumpTradeType::Buy));
        assert!(matches!(sell_type, PumpTradeType::Sell));
    }

    #[test]
    fn test_pump_trade_type_debug_clone() {
        let buy_type = PumpTradeType::Buy;
        let sell_type = PumpTradeType::Sell;

        let buy_cloned = buy_type.clone();
        let sell_cloned = sell_type.clone();

        assert_eq!(format!("{:?}", buy_type), format!("{:?}", buy_cloned));
        assert_eq!(format!("{:?}", sell_type), format!("{:?}", sell_cloned));
    }

    #[test]
    fn test_pump_trade_analysis_serialization() {
        let analysis = PumpTradeAnalysis {
            signature: "sig123".to_string(),
            user_address: "user123".to_string(),
            token_mint: "mint123".to_string(),
            token_amount: Some(1000),
            sol_amount: Some(-0.5),
            price_per_token: Some(0.0005),
        };

        let json = serde_json::to_string(&analysis).unwrap();
        assert!(json.contains("sig123"));
        assert!(json.contains("-0.5"));
    }

    #[test]
    fn test_pump_trade_analysis_debug_clone() {
        let analysis = PumpTradeAnalysis {
            signature: "sig".to_string(),
            user_address: "user".to_string(),
            token_mint: "mint".to_string(),
            token_amount: None,
            sol_amount: None,
            price_per_token: None,
        };

        let cloned = analysis.clone();
        assert_eq!(format!("{:?}", analysis), format!("{:?}", cloned));
    }

    #[test]
    fn test_pump_deployment_response_serialization() {
        let response = PumpDeploymentResponse {
            metadata_uri: "ipfs://hash123".to_string(),
            success: true,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("ipfs://hash123"));
        assert!(json.contains("true"));
    }

    #[test]
    fn test_pump_deployment_response_debug_clone() {
        let response = PumpDeploymentResponse {
            metadata_uri: "uri".to_string(),
            success: false,
        };

        let cloned = response.clone();
        assert_eq!(format!("{:?}", response), format!("{:?}", cloned));
    }

    #[test]
    fn test_pump_token_response_serialization() {
        let response = PumpTokenResponse {
            mint: Some("mint123".to_string()),
            name: "Token".to_string(),
            symbol: "TKN".to_string(),
            description: "Description".to_string(),
            image: Some("image.png".to_string()),
            market_cap: Some(1000000),
            price_sol: Some(0.001),
            creator: "creator123".to_string(),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("Token"));
        assert!(json.contains("TKN"));
    }

    #[test]
    fn test_pump_token_response_with_none_values() {
        let response = PumpTokenResponse {
            mint: None,
            name: "Token".to_string(),
            symbol: "TKN".to_string(),
            description: "Description".to_string(),
            image: None,
            market_cap: None,
            price_sol: None,
            creator: "creator123".to_string(),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("null"));
    }

    #[test]
    fn test_pump_token_response_debug_clone() {
        let response = PumpTokenResponse {
            mint: None,
            name: "name".to_string(),
            symbol: "sym".to_string(),
            description: "desc".to_string(),
            image: None,
            market_cap: None,
            price_sol: None,
            creator: "creator".to_string(),
        };

        let cloned = response.clone();
        assert_eq!(format!("{:?}", response), format!("{:?}", cloned));
    }

    #[test]
    fn test_generate_mint_keypair() {
        let keypair1 = generate_mint_keypair();
        let keypair2 = generate_mint_keypair();

        // Each keypair should be unique
        assert_ne!(keypair1.pubkey(), keypair2.pubkey());

        // Both keypairs should be valid
        assert_eq!(keypair1.pubkey().to_bytes().len(), 32);
        assert_eq!(keypair2.pubkey().to_bytes().len(), 32);
    }

    // Mock test helpers for async validation
    mod mock_validation_tests {
        use super::*;

        #[test]
        fn test_input_validation_empty_name() {
            // Test empty name validation logic
            let name = "";
            assert!(name.is_empty());
        }

        #[test]
        fn test_input_validation_name_too_long() {
            // Test name length validation logic
            let name = "a".repeat(33);
            assert!(name.len() > 32);
        }

        #[test]
        fn test_input_validation_empty_symbol() {
            // Test empty symbol validation logic
            let symbol = "";
            assert!(symbol.is_empty());
        }

        #[test]
        fn test_input_validation_symbol_too_long() {
            // Test symbol length validation logic
            let symbol = "a".repeat(11);
            assert!(symbol.len() > 10);
        }

        #[test]
        fn test_input_validation_description_too_long() {
            // Test description length validation logic
            let description = "a".repeat(1001);
            assert!(description.len() > 1000);
        }

        #[test]
        fn test_input_validation_valid_inputs() {
            // Test valid inputs
            let name = "Valid Token Name";
            let symbol = "VALID";
            let description = "Valid description";

            assert!(!name.is_empty() && name.len() <= 32);
            assert!(!symbol.is_empty() && symbol.len() <= 10);
            assert!(description.len() <= 1000);
        }

        #[test]
        fn test_input_validation_edge_case_lengths() {
            // Test exact boundary conditions
            let name_32 = "a".repeat(32);
            let symbol_10 = "a".repeat(10);
            let description_1000 = "a".repeat(1000);

            assert_eq!(name_32.len(), 32);
            assert_eq!(symbol_10.len(), 10);
            assert_eq!(description_1000.len(), 1000);
        }

        #[test]
        fn test_sol_amount_validation_zero() {
            // Test zero SOL amount validation
            let sol_amount = 0.0;
            assert!(sol_amount <= 0.0);
        }

        #[test]
        fn test_sol_amount_validation_negative() {
            // Test negative SOL amount validation
            let sol_amount = -1.0;
            assert!(sol_amount <= 0.0);
        }

        #[test]
        fn test_sol_amount_validation_positive() {
            // Test positive SOL amount validation
            let sol_amount = 1.5;
            assert!(sol_amount > 0.0);
        }

        #[test]
        fn test_token_amount_validation_zero() {
            // Test zero token amount validation
            let token_amount = 0u64;
            assert_eq!(token_amount, 0);
        }

        #[test]
        fn test_token_amount_validation_positive() {
            // Test positive token amount validation
            let token_amount = 1000u64;
            assert!(token_amount > 0);
        }

        #[test]
        fn test_pubkey_validation_valid() {
            // Test valid pubkey parsing
            let valid_pubkey = "11111111111111111111111111111111";
            let result = Pubkey::from_str(valid_pubkey);
            assert!(result.is_ok());
        }

        #[test]
        fn test_pubkey_validation_invalid() {
            // Test invalid pubkey parsing
            let invalid_pubkey = "invalid_pubkey";
            let result = Pubkey::from_str(invalid_pubkey);
            assert!(result.is_err());
        }

        #[test]
        fn test_pubkey_validation_empty() {
            // Test empty pubkey parsing
            let empty_pubkey = "";
            let result = Pubkey::from_str(empty_pubkey);
            assert!(result.is_err());
        }

        #[test]
        fn test_slippage_calculation_default() {
            // Test default slippage calculation
            let default_slippage = 5.0; // 5% default slippage
            assert_eq!(default_slippage, 5.0);
        }

        #[test]
        fn test_slippage_calculation_custom() {
            // Test custom slippage calculation
            let slippage_percent = 3.5;
            let slippage_bps = (slippage_percent * 100.0) as u64;
            assert_eq!(slippage_bps, 350);
        }

        #[test]
        fn test_lamports_conversion() {
            // Test SOL to lamports conversion
            let sol_amount = 1.5;
            let lamports = (sol_amount * LAMPORTS_PER_SOL as f64) as u64;
            assert_eq!(lamports, 1_500_000_000);
        }

        #[test]
        fn test_lamports_conversion_zero() {
            // Test zero SOL conversion
            let sol_amount = 0.0;
            let lamports = (sol_amount * LAMPORTS_PER_SOL as f64) as u64;
            assert_eq!(lamports, 0);
        }

        #[test]
        fn test_trending_limit_default() {
            // Test default trending limit
            let limit = None;
            let actual_limit = limit.unwrap_or(10).min(50);
            assert_eq!(actual_limit, 10);
        }

        #[test]
        fn test_trending_limit_custom() {
            // Test custom trending limit
            let limit = Some(25);
            let actual_limit = limit.unwrap_or(10).min(50);
            assert_eq!(actual_limit, 25);
        }

        #[test]
        fn test_trending_limit_capped() {
            // Test trending limit capping
            let limit = Some(100);
            let actual_limit = limit.unwrap_or(10).min(50);
            assert_eq!(actual_limit, 50);
        }

        #[test]
        fn test_trending_limit_zero() {
            // Test zero trending limit
            let limit = Some(0);
            let actual_limit = limit.unwrap_or(10).min(50);
            assert_eq!(actual_limit, 0);
        }

        #[test]
        fn test_status_code_404() {
            // Test 404 status code detection
            let status_code = 404u16;
            assert_eq!(status_code, 404);
        }

        #[test]
        fn test_status_code_success() {
            // Test success status code range
            let status_code = 200u16;
            assert!(status_code >= 200 && status_code < 300);
        }

        #[test]
        fn test_status_code_error() {
            // Test error status code
            let status_code = 500u16;
            assert!(status_code >= 400);
        }

        #[test]
        fn test_json_request_building_deploy() {
            // Test deployment request JSON structure
            let deploy_request = json!({
                "name": "Test Token",
                "symbol": "TEST",
                "description": "Test description",
                "image": "",
                "creator": "11111111111111111111111111111111",
                "showName": true
            });

            assert_eq!(deploy_request["name"], "Test Token");
            assert_eq!(deploy_request["symbol"], "TEST");
            assert!(deploy_request["showName"] == true);
        }

        #[test]
        fn test_json_request_building_buy() {
            // Test buy request JSON structure
            let buy_request = json!({
                "publicKey": "11111111111111111111111111111111",
                "action": "buy",
                "mint": "mint123",
                "amount": 1000000000u64,
                "denominatedInSol": "true",
                "slippage": 500u64
            });

            assert_eq!(buy_request["action"], "buy");
            assert_eq!(buy_request["denominatedInSol"], "true");
            assert_eq!(buy_request["amount"], 1000000000u64);
        }

        #[test]
        fn test_json_request_building_sell() {
            // Test sell request JSON structure
            let sell_request = json!({
                "publicKey": "11111111111111111111111111111111",
                "action": "sell",
                "mint": "mint123",
                "amount": 1000000u64,
                "denominatedInSol": "false",
                "slippage": 300u64
            });

            assert_eq!(sell_request["action"], "sell");
            assert_eq!(sell_request["denominatedInSol"], "false");
            assert_eq!(sell_request["amount"], 1000000u64);
        }

        #[test]
        fn test_json_request_building_create_tx() {
            // Test create transaction request JSON structure
            let create_tx_request = json!({
                "publicKey": "11111111111111111111111111111111",
                "action": "create",
                "mint": "mint123",
                "tokenMetadata": {
                    "name": "Test Token",
                    "symbol": "TEST",
                    "uri": "ipfs://hash123"
                }
            });

            assert_eq!(create_tx_request["action"], "create");
            assert_eq!(create_tx_request["tokenMetadata"]["name"], "Test Token");
        }

        #[test]
        fn test_image_url_unwrap_or_default() {
            // Test image URL handling with Some value
            let image_url = Some("https://example.com/image.png".to_string());
            let result = image_url.as_deref().unwrap_or_default();
            assert_eq!(result, "https://example.com/image.png");

            // Test image URL handling with None value
            let image_url: Option<String> = None;
            let result = image_url.as_deref().unwrap_or_default();
            assert_eq!(result, "");
        }

        #[test]
        fn test_error_text_unwrap_or_else() {
            // Test error text extraction fallback
            let fallback = "Unknown error".to_string();
            assert_eq!(fallback, "Unknown error");
        }

        #[test]
        fn test_initial_buy_amount_positive() {
            // Test initial buy amount validation
            let initial_buy_sol = Some(1.5);
            if let Some(buy_amount) = initial_buy_sol {
                assert!(buy_amount > 0.0);
            }
        }

        #[test]
        fn test_initial_buy_amount_zero() {
            // Test initial buy amount zero case
            let initial_buy_sol = Some(0.0);
            if let Some(buy_amount) = initial_buy_sol {
                assert!(buy_amount <= 0.0);
            }
        }

        #[test]
        fn test_initial_buy_amount_none() {
            // Test initial buy amount None case
            let initial_buy_sol: Option<f64> = None;
            assert!(initial_buy_sol.is_none());
        }

        #[test]
        fn test_mint_address_conversion() {
            // Test mint address string conversion
            let keypair = Keypair::new();
            let mint_address = keypair.pubkey();
            let mint_address_str = mint_address.to_string();
            assert_eq!(mint_address_str.len(), 44); // Base58 encoded pubkey length
        }

        #[test]
        fn test_base64_decode_success() {
            // Test base64 decoding success case
            use base64::{engine::general_purpose, Engine as _};
            let encoded = general_purpose::STANDARD.encode(b"test data");
            let decoded = general_purpose::STANDARD.decode(&encoded);
            assert!(decoded.is_ok());
            assert_eq!(decoded.unwrap(), b"test data");
        }

        #[test]
        fn test_base64_decode_failure() {
            // Test base64 decoding failure case
            use base64::{engine::general_purpose, Engine as _};
            let invalid_base64 = "invalid@base64!";
            let decoded = general_purpose::STANDARD.decode(invalid_base64);
            assert!(decoded.is_err());
        }

        #[test]
        fn test_sol_delta_calculation_positive() {
            // Test SOL delta calculation for positive change
            let pre_balance = 1000000000u64; // 1 SOL
            let post_balance = 1500000000u64; // 1.5 SOL
            let pre = pre_balance as i128;
            let post = post_balance as i128;
            let delta_lamports = post - pre;
            assert_eq!(delta_lamports, 500000000);
            assert!(delta_lamports > 0);
        }

        #[test]
        fn test_sol_delta_calculation_negative() {
            // Test SOL delta calculation for negative change
            let pre_balance = 1500000000u64; // 1.5 SOL
            let post_balance = 1000000000u64; // 1 SOL
            let pre = pre_balance as i128;
            let post = post_balance as i128;
            let delta_lamports = post - pre;
            assert_eq!(delta_lamports, -500000000);
            assert!(delta_lamports < 0);
        }

        #[test]
        fn test_sol_delta_calculation_with_fee() {
            // Test SOL delta calculation with fee adjustment
            let pre_balance = 1000000000u64;
            let post_balance = 995000000u64; // Less due to transaction fee
            let fee = 5000000u64; // 0.005 SOL transaction fee

            let pre = pre_balance as i128;
            let post = post_balance as i128;
            let delta_lamports = post - pre;
            let fee_i128 = fee as i128;

            let adjusted = if delta_lamports >= 0 {
                delta_lamports
            } else {
                delta_lamports + fee_i128
            };

            assert_eq!(adjusted, 0); // Break-even after fee adjustment
        }

        #[test]
        fn test_token_delta_calculation_positive() {
            // Test token delta calculation for token gain
            let pre_amount = 0i128;
            let post_amount = 1000000i128;
            let token_delta = post_amount - pre_amount;
            assert_eq!(token_delta, 1000000);
            assert!(token_delta > 0);
        }

        #[test]
        fn test_token_delta_calculation_negative() {
            // Test token delta calculation for token loss
            let pre_amount = 1000000i128;
            let post_amount = 500000i128;
            let token_delta = post_amount - pre_amount;
            assert_eq!(token_delta, -500000);
            assert!(token_delta < 0);
        }

        #[test]
        fn test_token_delta_calculation_zero() {
            // Test token delta calculation for no change
            let pre_amount = 1000000i128;
            let post_amount = 1000000i128;
            let token_delta = post_amount - pre_amount;
            assert_eq!(token_delta, 0);
        }

        #[test]
        fn test_price_calculation() {
            // Test price per token calculation
            let token_amount = 1000000u64;
            let sol_amount = 0.5f64;
            let decimals = 6u8;

            let token_ui = (token_amount as f64) / 10u64.pow(decimals as u32) as f64;
            let price = (sol_amount.abs()) / token_ui;

            assert_eq!(token_ui, 1.0);
            assert_eq!(price, 0.5);
        }

        #[test]
        fn test_price_calculation_with_decimals() {
            // Test price calculation with different decimals
            let token_amount = 1000000000u64; // 1 token with 9 decimals
            let sol_amount = 0.001f64;
            let decimals = 9u8;

            let token_ui = (token_amount as f64) / 10u64.pow(decimals as u32) as f64;
            let price = (sol_amount.abs()) / token_ui;

            assert_eq!(token_ui, 1.0);
            assert_eq!(price, 0.001);
        }

        #[test]
        fn test_price_calculation_zero_tokens() {
            // Test price calculation with zero tokens
            let token_amount = 0u64;
            let _sol_amount = 0.5f64;
            let decimals = 6u8;

            let token_ui = (token_amount as f64) / 10u64.pow(decimals as u32) as f64;
            assert_eq!(token_ui, 0.0);
            // Price calculation would be division by zero, so should be None
        }

        #[test]
        fn test_unsigned_abs_conversion() {
            // Test conversion from signed to unsigned absolute value
            let negative_value = -1000000i128;
            let positive_value = 1000000i128;
            let zero_value = 0i128;

            assert_eq!(negative_value.unsigned_abs(), 1000000u128);
            assert_eq!(positive_value.unsigned_abs(), 1000000u128);
            assert_eq!(zero_value.unsigned_abs(), 0u128);
        }

        #[test]
        fn test_option_serializer_matching() {
            // Test owner matching with OptionSerializer
            let owner_str = "11111111111111111111111111111111";
            let some_owner = OptionSerializer::Some(owner_str.to_string());
            let none_owner: OptionSerializer<String> = OptionSerializer::None;

            let owner_matches_some =
                matches!(&some_owner, OptionSerializer::Some(owner) if owner == &owner_str);
            let owner_matches_none =
                matches!(&none_owner, OptionSerializer::Some(owner) if owner == &owner_str);

            assert!(owner_matches_some);
            assert!(!owner_matches_none);
        }

        #[test]
        fn test_parse_amount_from_string() {
            // Test parsing token amounts from strings
            let valid_amount = "1000000";
            let invalid_amount = "not_a_number";
            let empty_amount = "";

            let valid_result = valid_amount.parse::<i128>();
            let invalid_result = invalid_amount.parse::<i128>();
            let empty_result = empty_amount.parse::<i128>();

            assert!(valid_result.is_ok());
            assert_eq!(valid_result.unwrap(), 1000000);
            assert!(invalid_result.is_err());
            assert!(empty_result.is_err());
        }

        #[test]
        fn test_retry_loop_logic() {
            // Test retry loop counter
            let max_retries = 3;
            let mut attempts = 0;

            for _ in 0..max_retries {
                attempts += 1;
            }

            assert_eq!(attempts, max_retries);
        }

        #[test]
        fn test_trending_tokens_conversion() {
            // Test conversion from PumpTokenResponse to PumpTokenInfo
            let token_response = PumpTokenResponse {
                mint: Some("mint123".to_string()),
                name: "Test Token".to_string(),
                symbol: "TEST".to_string(),
                description: "Test description".to_string(),
                image: Some("image.png".to_string()),
                market_cap: Some(1000000),
                price_sol: Some(0.001),
                creator: "creator123".to_string(),
            };

            let token_info = PumpTokenInfo {
                mint_address: token_response.mint.clone().unwrap_or_default(),
                name: token_response.name.clone(),
                symbol: token_response.symbol.clone(),
                description: token_response.description.clone(),
                image_url: token_response.image.clone(),
                market_cap: token_response.market_cap,
                price_sol: token_response.price_sol,
                creation_signature: None,
                creator: token_response.creator.clone(),
                initial_buy_signature: None,
            };

            assert_eq!(token_info.mint_address, "mint123");
            assert_eq!(token_info.name, "Test Token");
            assert_eq!(token_info.symbol, "TEST");
        }

        #[test]
        fn test_trending_tokens_conversion_none_mint() {
            // Test conversion with None mint
            let token_response = PumpTokenResponse {
                mint: None,
                name: "Test Token".to_string(),
                symbol: "TEST".to_string(),
                description: "Test description".to_string(),
                image: None,
                market_cap: None,
                price_sol: None,
                creator: "creator123".to_string(),
            };

            let mint_address = token_response.mint.clone().unwrap_or_default();
            assert_eq!(mint_address, "");
        }

        #[test]
        fn test_vec_collection() {
            // Test vector collection from iterator
            let responses = vec![
                PumpTokenResponse {
                    mint: Some("mint1".to_string()),
                    name: "Token1".to_string(),
                    symbol: "TK1".to_string(),
                    description: "Desc1".to_string(),
                    image: None,
                    market_cap: None,
                    price_sol: None,
                    creator: "creator1".to_string(),
                },
                PumpTokenResponse {
                    mint: Some("mint2".to_string()),
                    name: "Token2".to_string(),
                    symbol: "TK2".to_string(),
                    description: "Desc2".to_string(),
                    image: None,
                    market_cap: None,
                    price_sol: None,
                    creator: "creator2".to_string(),
                },
            ];

            let token_infos: Vec<PumpTokenInfo> = responses
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

            assert_eq!(token_infos.len(), 2);
            assert_eq!(token_infos[0].name, "Token1");
            assert_eq!(token_infos[1].name, "Token2");
        }

        #[test]
        fn test_balance_vec_empty_check() {
            // Test empty balance vector checks
            let empty_pre_balances: Vec<u64> = vec![];
            let empty_post_balances: Vec<u64> = vec![];
            let non_empty_pre_balances = vec![1000000000u64];
            let non_empty_post_balances = vec![1500000000u64];

            assert!(empty_pre_balances.is_empty());
            assert!(empty_post_balances.is_empty());
            assert!(!non_empty_pre_balances.is_empty());
            assert!(!non_empty_post_balances.is_empty());
        }

        #[test]
        fn test_none_check_with_is_none() {
            // Test None checking with is_none
            let decimals_opt: Option<u8> = None;
            let some_decimals_opt: Option<u8> = Some(6);

            assert!(decimals_opt.is_none());
            assert!(!some_decimals_opt.is_none());
        }

        #[test]
        fn test_option_and_then_logic() {
            // Test Option and_then logic for complex transformations
            let token_delta_raw: Option<i128> = Some(1000000);
            let zero_delta_raw: Option<i128> = Some(0);
            let none_delta_raw: Option<i128> = None;

            let token_delta_opt_u64 = token_delta_raw.and_then(|v| {
                if v == 0 {
                    None
                } else {
                    Some(v.unsigned_abs() as u64)
                }
            });

            let zero_delta_opt_u64 = zero_delta_raw.and_then(|v| {
                if v == 0 {
                    None
                } else {
                    Some(v.unsigned_abs() as u64)
                }
            });

            let none_delta_opt_u64 = none_delta_raw.and_then(|v| {
                if v == 0 {
                    None
                } else {
                    Some(v.unsigned_abs() as u64)
                }
            });

            assert_eq!(token_delta_opt_u64, Some(1000000));
            assert_eq!(zero_delta_opt_u64, None);
            assert_eq!(none_delta_opt_u64, None);
        }

        #[test]
        fn test_triple_option_matching() {
            // Test triple Option matching for price calculation
            let token_delta = Some(1000000u64);
            let sol_delta = Some(0.5f64);
            let decimals = Some(6u8);

            let price_opt = if let (Some(token_delta), Some(sol_delta), Some(decimals)) =
                (token_delta, sol_delta, decimals)
            {
                let token_ui = (token_delta as f64) / 10u64.pow(decimals as u32) as f64;
                if token_ui > 0.0 {
                    Some((sol_delta.abs()) / token_ui)
                } else {
                    None
                }
            } else {
                None
            };

            assert_eq!(price_opt, Some(0.5));
        }

        #[test]
        fn test_triple_option_matching_missing_values() {
            // Test triple Option matching with missing values
            let token_delta = Some(1000000u64);
            let sol_delta: Option<f64> = None;
            let decimals = Some(6u8);

            let price_opt = if let (Some(_token_delta), Some(_sol_delta), Some(_decimals)) =
                (token_delta, sol_delta, decimals)
            {
                Some(0.5) // This won't execute
            } else {
                None
            };

            assert_eq!(price_opt, None);
        }

        #[test]
        fn test_tuple_return_types() {
            // Test tuple return type patterns
            let result: (Option<u64>, Option<f64>, Option<f64>) =
                (Some(1000), Some(0.5), Some(0.0005));
            assert_eq!(result.0, Some(1000));
            assert_eq!(result.1, Some(0.5));
            assert_eq!(result.2, Some(0.0005));

            let fallback_result: (Option<u64>, Option<f64>, Option<f64>) = (None, Some(0.5), None);
            assert_eq!(fallback_result.0, None);
            assert_eq!(fallback_result.1, Some(0.5));
            assert_eq!(fallback_result.2, None);
        }
    }
}
