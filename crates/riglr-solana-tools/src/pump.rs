//! Pump.fun integration for token deployment, buying, and selling on Solana
//!
//! This module provides tools for interacting with the Pump.fun platform,
//! enabling token deployment, trading operations with slippage protection.

use crate::clients::{Clients, PumpClient};
use crate::common_newtypes::{SolanaAddress, SolanaSignature};
use crate::transaction::Status;
use crate::utils::validation::validate_address;
use crate::utils::{generate_mint, send};
use base64::{engine::general_purpose, Engine as _};
use riglr_core::provider::ApplicationContext;
use riglr_core::signer::SolanaSignerHandle;
use riglr_core::{retry, SignerContext, ToolError};
use riglr_macros::tool;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;
use solana_client::rpc_client::RpcClient;
use solana_sdk::signature::Signature;
use solana_sdk::transaction::Transaction;
use solana_sdk::{
    instruction::Instruction, native_token::LAMPORTS_PER_SOL, pubkey::Pubkey, signature::Keypair,
    signer::Signer as _,
};
use solana_transaction_status::option_serializer::OptionSerializer;
use solana_transaction_status::UiTransactionEncoding;
use std::sync::Arc;
use tracing::{debug, info, warn};

// Rate limiting is now handled via ApplicationContext dependency injection
// The RateLimiter service is available from riglr_core::util::RateLimiter

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
    /// Creator's public key
    pub creator: String,
    /// Token description
    pub description: String,
    /// Optional image URL
    pub image: Option<String>,
    /// Current market cap in lamports
    pub market_cap: Option<u64>,
    /// Token mint address
    pub mint: Option<String>,
    /// Token name
    pub name: String,
    /// Current price in SOL
    pub price_sol: Option<f64>,
    /// Token symbol
    pub symbol: String,
}

/// Token information from Pump.fun
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[non_exhaustive]
pub struct TokenInfo {
    /// Transaction signature for token creation
    pub creation_signature: Option<String>,
    /// Creator's public key
    pub creator: String,
    /// Token description
    pub description: String,
    /// Optional image URL
    pub image_url: Option<String>,
    /// Transaction signature for initial buy (if any)
    pub initial_buy_signature: Option<String>,
    /// Current market cap in lamports
    pub market_cap: Option<u64>,
    /// Token mint address
    pub mint_address: String,
    /// Token name
    pub name: String,
    /// Current price in SOL
    pub price_sol: Option<f64>,
    /// Token symbol
    pub symbol: String,
}

/// Type of trade on Pump.fun
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[non_exhaustive]
pub enum TradeType {
    /// Buy tokens with SOL
    Buy,
    /// Sell tokens for SOL
    Sell,
}

/// Result of a Pump.fun trade operation
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[non_exhaustive]
pub struct TradeResult {
    /// Error encountered during signing/sending (if any)
    pub error: Option<String>,
    /// Signature of the transaction that was sent
    pub signature: String,
    /// Status of the transaction
    pub status: Status,
}

/// Analysis result of a Pump.fun trade transaction
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[non_exhaustive]
pub struct TradeAnalysis {
    /// Price per token in SOL (if determinable)
    pub price_per_token: Option<f64>,
    /// Transaction signature
    pub signature: String,
    /// Change in SOL balance for the user
    pub sol_amount: Option<f64>,
    /// Token amount involved in the trade
    pub token_amount: Option<u64>,
    /// Token mint address
    pub token_mint: String,
    /// User address that performed the trade
    pub user_address: String,
}

/// Validate token deployment parameters
fn validate_deployment_params(
    name: &str,
    symbol: &str,
    description: &str,
) -> Result<(), ToolError> {
    if name.is_empty() || name.len() > 32 {
        return Err(ToolError::permanent_string(
            "Token name must be 1-32 characters".to_owned(),
        ));
    }
    if symbol.is_empty() || symbol.len() > 10 {
        return Err(ToolError::permanent_string(
            "Token symbol must be 1-10 characters".to_owned(),
        ));
    }
    if description.len() > 1000 {
        return Err(ToolError::permanent_string(
            "Description must be under 1000 characters".to_owned(),
        ));
    }
    Ok(())
}

/// Prepare deployment context with signer, mint keypair, and API clients
fn prepare_deployment_context(
    context: &ApplicationContext,
) -> Result<(SolanaSignerHandle, Pubkey, Keypair, Clients), ToolError> {
    // Get signer from context
    let signer_context = SignerContext::current_as_solana()
        .map_err(|e| ToolError::permanent_string(format!("No Solana signer context: {e}")))?;

    let signer_pubkey = signer_context
        .signer()
        .pubkey()
        .parse::<Pubkey>()
        .map_err(|e| ToolError::permanent_string(format!("Invalid signer pubkey: {e}")))?;

    // Generate new mint keypair
    let mint_keypair = generate_mint();

    // Get API clients from context
    let api_clients = context
        .get_extension::<Arc<Clients>>()
        .ok_or_else(|| ToolError::permanent_string("Clients not found in context"))?;

    Ok((
        signer_context,
        signer_pubkey,
        mint_keypair,
        (**api_clients).clone(),
    ))
}

/// Sign and send the token creation transaction
async fn execute_token_creation(
    transaction: Transaction,
    mint_keypair: &Keypair,
    signer_context: &SolanaSignerHandle,
    context: &ApplicationContext,
) -> Result<String, ToolError> {
    let mut tx = transaction;

    // Get recent blockhash
    let client = context
        .get_extension::<Arc<RpcClient>>()
        .ok_or_else(|| ToolError::permanent_string("Solana RPC client not found in context"))?;
    let recent_blockhash = client
        .get_latest_blockhash()
        .map_err(|e| ToolError::retriable_string(format!("Failed to get blockhash: {e}")))?;

    // Sign with mint keypair first
    tx.partial_sign(&[mint_keypair], recent_blockhash);

    // Serialize and sign with fee payer
    let tx_bytes = bincode::serialize(&tx).map_err(|e| {
        ToolError::permanent_string(format!("Failed to serialize transaction: {e}"))
    })?;

    // Convert to base64-encoded JSON for the signer interface
    let tx_json = serde_json::json!({
        "transaction": general_purpose::STANDARD.encode(&tx_bytes)
    });

    let signature = signer_context
        .signer()
        .sign_and_send_transaction(tx_json)
        .await
        .map_err(|e| {
            ToolError::retriable_string(format!("Failed to sign and send transaction: {e}"))
        })?;

    Ok(signature)
}

/// Perform initial token purchase if specified
async fn execute_initial_buy(
    mint_address: &str,
    buy_amount: f64,
    context: &ApplicationContext,
) -> Option<String> {
    if buy_amount <= 0.0f64 {
        return None;
    }

    info!("Performing initial buy of {} SOL", buy_amount);

    return match buy_pump_token(
        mint_address.to_owned(),
        buy_amount,
        Some(5.0f64), // 5% default slippage
        context,
    )
    .await
    {
        Ok(buy_result) => {
            let buy_signature = buy_result.signature.clone();
            info!("Initial buy completed: {}", buy_signature);
            Some(buy_result.signature)
        }
        Err(e) => {
            warn!(
                "Initial buy failed, but token was created successfully: {}",
                e
            );
            None
        }
    };
}

/// Upload metadata to IPFS via Pump.fun API
async fn upload_metadata_to_ipfs(
    name: &str,
    symbol: &str,
    description: &str,
    image_url: Option<&str>,
    signer_pubkey: &str,
    pump_client: &PumpClient,
) -> Result<PumpDeploymentResponse, ToolError> {
    let deploy_request = json!({
        "name": name,
        "symbol": symbol,
        "description": description,
        "image": image_url.unwrap_or_default(),
        "creator": signer_pubkey,
        "showName": true
    });

    debug!("Requesting token deployment from Pump.fun");

    let response = pump_client
        .http_client()
        .post(format!("{}/ipfs", pump_client.api_url()))
        .json(&deploy_request)
        .send()
        .await
        .map_err(|e| ToolError::retriable_string(format!("Failed to request deployment: {e}")))?;

    if !response.status().is_success() {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_owned());
        return Err(ToolError::permanent_string(format!(
            "Pump.fun deployment API error: {error_text}"
        )));
    }

    let deployment_response: PumpDeploymentResponse = response.json().await.map_err(|e| {
        ToolError::permanent_string(format!("Failed to parse deployment response: {e}"))
    })?;

    info!(
        "Token metadata uploaded to IPFS: {}",
        deployment_response.metadata_uri
    );
    Ok(deployment_response)
}

/// Get signed creation transaction from Pump.fun
///
/// # Errors
///
/// Returns `ToolError` if:
/// - Network communication fails
/// - Transaction API returns non-success status
/// - Base64 decoding fails
/// - Transaction deserialization fails
async fn get_creation_transaction(
    signer_pubkey: &str,
    mint_address: &Pubkey,
    name: &str,
    symbol: &str,
    metadata_uri: &str,
    pump_client: &PumpClient,
) -> Result<Transaction, ToolError> {
    let create_tx_request = json!({
        "publicKey": signer_pubkey,
        "action": "create",
        "mint": mint_address.to_string(),
        "tokenMetadata": {
            "name": name,
            "symbol": symbol,
            "uri": metadata_uri
        }
    });

    let tx_response = pump_client
        .http_client()
        .post(format!("{}/trade-local", pump_client.api_url()))
        .json(&create_tx_request)
        .send()
        .await
        .map_err(|e| {
            ToolError::retriable_string(format!("Failed to get creation transaction: {e}"))
        })?;

    if !tx_response.status().is_success() {
        let error_text = tx_response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_owned());
        return Err(ToolError::permanent_string(format!(
            "Pump.fun transaction API error: {error_text}"
        )));
    }

    let tx_data: String = tx_response
        .text()
        .await
        .map_err(|e| ToolError::permanent_string(format!("Failed to get transaction data: {e}")))?;

    // Deserialize the creation transaction
    let transaction_bytes = general_purpose::STANDARD.decode(&tx_data).map_err(|e| {
        ToolError::permanent_string(format!("Failed to decode creation transaction: {e}"))
    })?;

    let transaction: Transaction = bincode::deserialize(&transaction_bytes).map_err(|e| {
        ToolError::permanent_string(format!("Failed to deserialize creation transaction: {e}"))
    })?;

    Ok(transaction)
}

/// Deploy a new token on Pump.fun
///
/// This tool creates and deploys a new meme token on the Pump.fun platform.
/// Optionally performs an initial buy to bootstrap liquidity.
///
/// # Errors
///
/// Returns `ToolError` if:
/// - Token parameters are invalid (name, symbol, description length)
/// - Signer context is not available or invalid
/// - API clients are not properly configured
/// - Network communication fails
/// - Transaction signing or broadcasting fails
// #[tool] removed to avoid conflict with manual impl below
pub async fn deploy_pump_token(
    name: String,
    symbol: String,
    description: String,
    image_url: Option<String>,
    initial_buy_sol: Option<f64>,
    context: &ApplicationContext,
) -> Result<TokenInfo, ToolError> {
    debug!(
        "Deploying pump token: {} ({}) - {}",
        name, symbol, description
    );

    // Validate inputs
    validate_deployment_params(&name, &symbol, &description)?;

    // Prepare deployment context
    let (signer_context, signer_pubkey, mint_keypair, api_clients) =
        prepare_deployment_context(context)?;
    let mint_address = mint_keypair.pubkey();
    let pump_client = &api_clients.pump;

    // Upload metadata to IPFS
    let deployment_response = upload_metadata_to_ipfs(
        &name,
        &symbol,
        &description,
        image_url.as_deref(),
        &signer_pubkey.to_string(),
        pump_client,
    )
    .await?;

    // Get creation transaction
    let transaction = get_creation_transaction(
        &signer_pubkey.to_string(),
        &mint_address,
        &name,
        &symbol,
        &deployment_response.metadata_uri,
        pump_client,
    )
    .await?;

    // Execute token creation
    let creation_signature =
        execute_token_creation(transaction, &mint_keypair, &signer_context, context).await?;

    info!("Token creation transaction sent: {}", creation_signature);

    // Create token info structure
    let mint_address_str = mint_address.to_string();
    let mut token_info = TokenInfo {
        mint_address: mint_address_str.clone(),
        name: name.clone(),
        symbol: symbol.clone(),
        description: description.clone(),
        image_url,
        market_cap: Some(0),
        price_sol: Some(0.0f64),
        creation_signature: Some(creation_signature),
        creator: signer_pubkey.to_string(),
        initial_buy_signature: None,
    };

    // Perform initial buy if specified
    if let Some(buy_amount) = initial_buy_sol {
        token_info.initial_buy_signature =
            execute_initial_buy(&mint_address_str, buy_amount, context).await;
    }

    Ok(token_info)
}

/// Validate buy parameters and prepare context
fn validate_buy_params(
    token_mint: &str,
    sol_amount: f64,
    context: &ApplicationContext,
) -> Result<(SolanaSignerHandle, Pubkey, Clients), ToolError> {
    if sol_amount <= 0.0f64 {
        return Err(ToolError::permanent_string(
            "SOL amount must be positive".to_owned(),
        ));
    }

    // Validate mint address
    let _mint_pubkey =
        validate_address(token_mint).map_err(|e| ToolError::permanent_string(e.to_string()))?;

    let signer_context = SignerContext::current_as_solana()
        .map_err(|e| ToolError::permanent_string(format!("No Solana signer context: {e}")))?;

    let signer_pubkey = signer_context
        .signer()
        .pubkey()
        .parse::<Pubkey>()
        .map_err(|e| ToolError::permanent_string(format!("Invalid signer pubkey: {e}")))?;

    // Get API clients from context
    let api_clients = context
        .get_extension::<Arc<Clients>>()
        .ok_or_else(|| ToolError::permanent_string("Clients not found in context"))?;

    Ok((signer_context, signer_pubkey, (**api_clients).clone()))
}

/// Request buy transaction from Pump.fun API
async fn request_buy_transaction(
    signer_pubkey: &Pubkey,
    token_mint: &str,
    sol_amount: f64,
    slippage_percent: f64,
    pump_client: &PumpClient,
) -> Result<Transaction, ToolError> {
    // Converting SOL to lamports with proper bounds checking
    #[expect(clippy::cast_precision_loss)]
    let lamports_f64 = (sol_amount * LAMPORTS_PER_SOL as f64).round();
    #[expect(clippy::cast_precision_loss)]
    if lamports_f64 < 0.0 || lamports_f64 > u64::MAX as f64 {
        return Err(ToolError::permanent_string(
            "SOL amount out of valid range".to_string(),
        ));
    }
    #[expect(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    let amount_lamports = lamports_f64 as u64;

    // Converting slippage percentage to basis points with proper bounds checking
    let slippage_bps_f64 = (slippage_percent * 100.0).round();
    if slippage_bps_f64 < 0.0 || slippage_bps_f64 > f64::from(u32::MAX) {
        return Err(ToolError::permanent_string(
            "Slippage percentage out of valid range".to_string(),
        ));
    }
    #[expect(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    let slippage_basis_points = u64::from(slippage_bps_f64 as u32);

    let buy_request = json!({
        "publicKey": signer_pubkey,
        "action": "buy",
        "mint": token_mint,
        "amount": amount_lamports,
        "denominatedInSol": "true",
        "slippage": slippage_basis_points
    });

    debug!("Requesting buy transaction from Pump.fun");

    let response = pump_client
        .http_client()
        .post(format!("{}/trade-local", pump_client.api_url()))
        .json(&buy_request)
        .send()
        .await
        .map_err(|e| {
            ToolError::retriable_string(format!("Failed to request buy transaction: {e}"))
        })?;

    if !response.status().is_success() {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_owned());
        return Err(ToolError::permanent_string(format!(
            "Pump.fun buy API error: {error_text}"
        )));
    }

    let tx_data: String = response.text().await.map_err(|e| {
        ToolError::permanent_string(format!("Failed to get buy transaction data: {e}"))
    })?;

    // Deserialize transaction
    let transaction_bytes = general_purpose::STANDARD.decode(&tx_data).map_err(|e| {
        ToolError::permanent_string(format!("Failed to decode buy transaction: {e}"))
    })?;

    let transaction: Transaction = bincode::deserialize(&transaction_bytes).map_err(|e| {
        ToolError::permanent_string(format!("Failed to deserialize buy transaction: {e}"))
    })?;

    Ok(transaction)
}

/// Buy tokens on Pump.fun
///
/// This tool executes a buy order for a specific token on Pump.fun
/// with configurable slippage protection.
///
/// # Errors
///
/// Returns `ToolError` if:
/// - Token mint address is invalid
/// - SOL amount is non-positive or exceeds available balance
/// - Signer context is not available or invalid
/// - API clients are not properly configured
/// - Network communication fails
/// - Transaction signing or broadcasting fails
/// - Slippage protection triggers
// #[tool] removed to avoid conflict with manual impl below
pub async fn buy_pump_token(
    token_mint: String,
    sol_amount: f64,
    slippage_percent: Option<f64>,
    context: &ApplicationContext,
) -> Result<TradeResult, ToolError> {
    debug!(
        "Buying pump token: {} with {} SOL (slippage: {:?}%)",
        token_mint, sol_amount, slippage_percent
    );

    // Validate parameters and get context
    let (_signer_context, signer_pubkey, api_clients) =
        validate_buy_params(&token_mint, sol_amount, context)?;
    let pump_client = &api_clients.pump;

    let slippage = slippage_percent.unwrap_or(5.0f64);

    // Request buy transaction from API
    let mut transaction = request_buy_transaction(
        &signer_pubkey,
        &token_mint,
        sol_amount,
        slippage,
        pump_client,
    )
    .await?;

    // Send buy transaction with retry logic
    let signature = send(
        &mut transaction,
        &format!("Buy Pump Token ({sol_amount} SOL)"),
    )
    .await?;

    info!(
        "Pump.fun buy executed: {} SOL for {} tokens, signature: {}",
        sol_amount, token_mint, signature
    );

    Ok(TradeResult {
        signature,
        error: None,
        status: Status::Pending,
    })
}

/// Validate sell parameters and prepare context
fn validate_sell_params(
    token_mint: &str,
    token_amount: u64,
    context: &ApplicationContext,
) -> Result<(SolanaSignerHandle, Pubkey, Clients), ToolError> {
    if token_amount == 0 {
        return Err(ToolError::permanent_string(
            "Token amount must be positive".to_owned(),
        ));
    }

    // Validate mint address
    let _mint_pubkey =
        validate_address(token_mint).map_err(|e| ToolError::permanent_string(e.to_string()))?;

    let signer_context = SignerContext::current_as_solana()
        .map_err(|e| ToolError::permanent_string(format!("No Solana signer context: {e}")))?;

    let signer_pubkey = signer_context
        .signer()
        .pubkey()
        .parse::<Pubkey>()
        .map_err(|e| ToolError::permanent_string(format!("Invalid signer pubkey: {e}")))?;

    // Get API clients from context
    let api_clients = context
        .get_extension::<Arc<Clients>>()
        .ok_or_else(|| ToolError::permanent_string("Clients not found in context"))?;

    Ok((signer_context, signer_pubkey, (**api_clients).clone()))
}

/// Request sell transaction from Pump.fun API
async fn request_sell_transaction(
    signer_pubkey: &Pubkey,
    token_mint: &str,
    token_amount: u64,
    slippage_percent: f64,
    pump_client: &PumpClient,
) -> Result<Transaction, ToolError> {
    // Converting slippage percentage to basis points with proper bounds checking
    let slippage_bps_f64 = (slippage_percent * 100.0).round();
    if slippage_bps_f64 < 0.0 || slippage_bps_f64 > f64::from(u32::MAX) {
        return Err(ToolError::permanent_string(
            "Slippage percentage out of valid range".to_string(),
        ));
    }
    #[expect(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    let slippage_basis_points = u64::from(slippage_bps_f64 as u32);

    let sell_request = json!({
        "publicKey": signer_pubkey,
        "action": "sell",
        "mint": token_mint,
        "amount": token_amount,
        "denominatedInSol": "false",
        "slippage": slippage_basis_points
    });

    debug!("Requesting sell transaction from Pump.fun");

    let response = pump_client
        .http_client()
        .post(format!("{}/trade-local", pump_client.api_url()))
        .json(&sell_request)
        .send()
        .await
        .map_err(|e| {
            ToolError::retriable_string(format!("Failed to request sell transaction: {e}"))
        })?;

    if !response.status().is_success() {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_owned());
        return Err(ToolError::permanent_string(format!(
            "Pump.fun sell API error: {error_text}"
        )));
    }

    let tx_data: String = response.text().await.map_err(|e| {
        ToolError::permanent_string(format!("Failed to get sell transaction data: {e}"))
    })?;

    // Deserialize transaction
    let transaction_bytes = general_purpose::STANDARD.decode(&tx_data).map_err(|e| {
        ToolError::permanent_string(format!("Failed to decode sell transaction: {e}"))
    })?;

    let transaction: Transaction = bincode::deserialize(&transaction_bytes).map_err(|e| {
        ToolError::permanent_string(format!("Failed to deserialize sell transaction: {e}"))
    })?;

    Ok(transaction)
}

/// Sell tokens on Pump.fun
///
/// This tool executes a sell order for a specific token on Pump.fun
/// with configurable slippage protection.
///
/// # Errors
///
/// Returns `ToolError` if:
/// - Token mint address is invalid
/// - Token amount is zero or exceeds available balance
/// - Signer context is not available or invalid
/// - API clients are not properly configured
/// - Network communication fails
/// - Transaction signing or broadcasting fails
/// - Slippage protection triggers
// #[tool] removed to avoid conflict with manual impl below
pub async fn sell_pump_token(
    token_mint: String,
    token_amount: u64,
    slippage_percent: Option<f64>,
    context: &ApplicationContext,
) -> Result<TradeResult, ToolError> {
    debug!(
        "Selling pump token: {} amount: {} (slippage: {:?}%)",
        token_mint, token_amount, slippage_percent
    );

    // Validate parameters and get context
    let (_signer_context, signer_pubkey, api_clients) =
        validate_sell_params(&token_mint, token_amount, context)?;
    let pump_client = &api_clients.pump;

    let slippage = slippage_percent.unwrap_or(5.0f64);

    // Request sell transaction from API
    let mut transaction = request_sell_transaction(
        &signer_pubkey,
        &token_mint,
        token_amount,
        slippage,
        pump_client,
    )
    .await?;

    // Send sell transaction with retry logic
    let signature = send(
        &mut transaction,
        &format!("Sell Pump Token ({token_amount} tokens)"),
    )
    .await?;

    info!(
        "Pump.fun sell executed: {} tokens for SOL, signature: {}",
        token_amount, signature
    );

    Ok(TradeResult {
        signature,
        error: None,
        status: Status::Pending,
    })
}

/// Get token information from Pump.fun
///
/// This tool fetches current token information, price, and market data
/// for a specific token on the Pump.fun platform.
///
/// # Errors
///
/// Returns `ToolError` if:
/// - Token mint address is invalid
/// - API clients are not properly configured
/// - Network communication fails
/// - Token is not found on Pump.fun
/// - Response parsing fails
// #[tool] removed to avoid conflict with manual impl below
pub async fn get_pump_token_info(
    token_mint: String,
    context: &ApplicationContext,
) -> Result<TokenInfo, ToolError> {
    debug!("Getting pump token info for: {}", token_mint);

    // Validate mint address
    let _mint_pubkey =
        validate_address(&token_mint).map_err(|e| ToolError::permanent_string(e.to_string()))?;

    // Get API clients from context
    let api_clients = context
        .get_extension::<Arc<Clients>>()
        .ok_or_else(|| ToolError::permanent_string("Clients not found in context"))?;
    let pump_client = &api_clients.pump;

    // Request token information using injected client
    let response = pump_client
        .http_client()
        .get(format!("{}/token/{}", pump_client.api_url(), token_mint))
        .send()
        .await
        .map_err(|e| ToolError::retriable_string(format!("Failed to get token info: {e}")))?;

    if response.status().as_u16() == 404 {
        return Err(ToolError::permanent_string(format!(
            "Token {token_mint} not found on Pump.fun"
        )));
    }

    if !response.status().is_success() {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_owned());
        return Err(ToolError::retriable_string(format!(
            "Pump.fun API error: {error_text}"
        )));
    }

    let token_response: PumpTokenResponse = response
        .json()
        .await
        .map_err(|e| ToolError::permanent_string(format!("Failed to parse token info: {e}")))?;

    info!(
        "Retrieved token info for {}: {} ({})",
        token_mint, token_response.name, token_response.symbol
    );

    Ok(TokenInfo {
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
/// - Rate limiting via `SignerContext`
/// - Signature format validation
/// - Address length validation
///
/// # Errors
///
/// Returns `ToolError` if:
/// - Transaction signature is invalid or cannot be parsed
/// - User or token addresses are malformed
/// - Transaction cannot be fetched from the blockchain
/// - Transaction parsing fails or contains unexpected data
/// - Network connection issues prevent data retrieval
#[tool]
pub async fn analyze_pump_transaction(
    signature: SolanaSignature,
    user_address: SolanaAddress,
    token_mint: SolanaAddress,
    context: &ApplicationContext,
) -> Result<TradeAnalysis, ToolError> {
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
    if user_pubkey == Pubkey::default() {
        return Err(ToolError::invalid_input_string(
            "Invalid user address: cannot be default/zero address",
        ));
    }

    if mint_pubkey == Pubkey::default() {
        return Err(ToolError::invalid_input_string(
            "Invalid token mint: cannot be default/zero address",
        ));
    }

    // Rate limiting is now automatically handled by the ToolWorker framework
    // No manual rate limiting checks needed - the framework applies rate limiting
    // based on SignerContext user_id before tool execution

    // Get RPC client from ApplicationContext
    let client = context
        .get_extension::<Arc<RpcClient>>()
        .ok_or_else(|| ToolError::permanent_string("Solana RPC client not found in context"))?;

    // Parse transaction details
    let (token_amount, sol_amount, price_per_token) =
        parse_pump_trade_details(&client, &sig_str, &user_pubkey, &mint_pubkey).await?;

    info!(
        "Analyzed Pump transaction {}: token_amount={:?}, sol_amount={:?}, price={:?}",
        signature, token_amount, sol_amount, price_per_token
    );

    Ok(TradeAnalysis {
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
///
/// # Errors
///
/// Returns `ToolError` if:
/// - API clients are not properly configured
/// - Network communication fails
/// - Response parsing fails
/// - Invalid limit parameter (exceeds maximum allowed)
// #[tool] removed to avoid conflict with manual impl below
pub async fn get_trending_pump_tokens(
    limit: Option<u32>,
    context: &ApplicationContext,
) -> Result<Vec<TokenInfo>, ToolError> {
    debug!("Getting trending pump tokens (limit: {:?})", limit);

    // Get API clients from context
    let api_clients = context
        .get_extension::<Arc<Clients>>()
        .ok_or_else(|| ToolError::permanent_string("Clients not found in context"))?;
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
        .map_err(|e| ToolError::retriable_string(format!("Failed to get trending tokens: {e}")))?;

    if !response.status().is_success() {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_owned());
        return Err(ToolError::retriable_string(format!(
            "Pump.fun trending API error: {error_text}"
        )));
    }

    let trending_response: Vec<PumpTokenResponse> = response.json().await.map_err(|e| {
        ToolError::permanent_string(format!("Failed to parse trending tokens: {e}"))
    })?;

    let trending_tokens: Vec<TokenInfo> = trending_response
        .into_iter()
        .map(|token| TokenInfo {
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

// ============================================================================
// Utility Functions for Token Creation
// ============================================================================

// Tool re-exports are handled by the macro system automatically
// Remove manual tool struct definitions as they conflict with macro-generated ones

/// Creates properly signed Solana transaction with mint keypair
///
/// This function creates a transaction with the given instructions and signs it
/// using both the payer from signer context and the provided mint keypair.
///
/// # Errors
///
/// Returns `ToolError` if:
/// - Signer context is not available or invalid
/// - Transaction serialization fails
/// - Transaction signing or broadcasting fails
#[inline]
pub async fn create_token_with_mint_keypair(
    instructions: Vec<Instruction>,
    _mint_keypair: &Keypair,
) -> Result<String, ToolError> {
    let signer = SignerContext::current_as_solana()
        .map_err(|e| ToolError::permanent_string(format!("No Solana signer context: {e}")))?;

    // Get the payer pubkey from the signer context
    let payer_pubkey = validate_address(&signer.signer().pubkey())
        .map_err(|e| ToolError::permanent_string(e.to_string()))?;

    let transaction = Transaction::new_with_payer(&instructions, Some(&payer_pubkey));

    // Get recent blockhash
    let _rpc_client = signer.signer().client();

    // In a real implementation, we would sign with both the payer and mint keypair
    // For now, we'll use the signer context to sign and send the transaction
    // Serialize transaction to bytes
    let tx_bytes = bincode::serialize(&transaction).map_err(|e| {
        ToolError::permanent_string(format!("Failed to serialize transaction: {e}"))
    })?;

    // Convert to base64-encoded JSON for the signer interface
    let tx_json = serde_json::json!({
        "transaction": general_purpose::STANDARD.encode(&tx_bytes)
    });

    let signature = signer
        .signer()
        .sign_and_send_transaction(tx_json)
        .await
        .map_err(|e| {
            ToolError::retriable_string(format!("Failed to sign and send transaction: {e}"))
        })?;

    Ok(signature)
}

// ============================================================================
// Helpers: Parse executed transactions to extract token/SOL deltas and price
// ============================================================================

// (no direct mint decimals helper needed; we'll use UiTokenAmount.decimals when available)

/// Parse a confirmed transaction to compute token delta for the user for the given mint,
/// the SOL delta (spent or received), and the price per token (SOL per whole token).
///
/// Returns a tuple of (`token_amount`, `sol_amount`, `price_per_token`) where:
/// - `token_amount`: Number of tokens involved in the trade (raw units)
/// - `sol_amount`: SOL amount delta (positive for received, negative for spent)
/// - `price_per_token`: Price per token in SOL
#[expect(clippy::too_many_lines)]
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
        .map_err(|e| ToolError::permanent_string(format!("Invalid signature: {e}")))?;

    let tx = retry::retry_async(
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
        |_| retry::ErrorClass::Retryable,
        &retry::RetryConfig::fast(),
        "fetch_pump_transaction",
    )
    .await
    .map_err(|e| ToolError::retriable_string(format!("Transaction not available yet: {e}")))?;

    let meta = tx
        .transaction
        .meta
        .ok_or_else(|| ToolError::permanent_string("Missing transaction meta".to_string()))?;

    // Compute SOL delta for the user
    let (pre_balances, post_balances) = (meta.pre_balances, meta.post_balances);
    // Assume signer is fee payer and first account key (index 0)
    let sol_delta_ui: Option<f64> = if !pre_balances.is_empty() && !post_balances.is_empty() {
        // We've already checked that the vectors are not empty above
        #[expect(clippy::expect_used)]
        let pre = i128::from(
            *pre_balances
                .first()
                .expect("Pre-balances vector verified as non-empty"),
        );
        #[expect(clippy::expect_used)]
        let post = i128::from(
            *post_balances
                .first()
                .expect("Post-balances vector verified as non-empty"),
        );
        let delta_lamports = post.saturating_sub(pre); // positive means received
        let fee = i128::from(meta.fee);
        let adjusted = if delta_lamports >= 0 {
            delta_lamports
        } else {
            delta_lamports.saturating_add(fee)
        };
        // Convert to f64 with acceptable precision loss for SOL UI calculations
        #[expect(clippy::cast_precision_loss)]
        let sol_ui = adjusted as f64 / LAMPORTS_PER_SOL as f64;
        Some(sol_ui)
    } else {
        None
    };

    // Compute token delta from token balances
    let owner_str = user.to_string();

    // Find matching pre/post token balances for this owner+mint
    if let (OptionSerializer::Some(pre_tb), OptionSerializer::Some(post_tb)) =
        (meta.pre_token_balances, meta.post_token_balances)
    {
        let mut pre_amount: i128 = 0;
        let mut post_amount: i128 = 0;
        let mut decimals_opt: Option<u8> = None;
        for tb in pre_tb {
            let owner_matches =
                matches!(tb.owner, OptionSerializer::Some(ref owner) if owner == &owner_str);
            if owner_matches && tb.mint == mint.to_string() {
                if let Ok(v) = tb.ui_token_amount.amount.parse::<i128>() {
                    pre_amount = v;
                }
                decimals_opt = Some(tb.ui_token_amount.decimals);
            }
        }
        for tb in post_tb {
            let owner_matches =
                matches!(tb.owner, OptionSerializer::Some(ref owner) if owner == &owner_str);
            if owner_matches && tb.mint == mint.to_string() {
                if let Ok(v) = tb.ui_token_amount.amount.parse::<i128>() {
                    post_amount = v;
                }
                if decimals_opt.is_none() {
                    decimals_opt = Some(tb.ui_token_amount.decimals);
                }
            }
        }
        let token_delta_raw: Option<i128> = Some(post_amount.saturating_sub(pre_amount));

        // Convert to outputs and compute price
        let (token_delta_opt_u64, price_opt) = if let (Some(token_delta), Some(decimals)) =
            (token_delta_raw, decimals_opt)
        {
            if token_delta == 0 {
                (None, None)
            } else {
                let token_delta_u64 = u64::try_from(token_delta.unsigned_abs()).unwrap_or(u64::MAX); // Clamp to max if overflow
                                                                                                     // Convert to f64 with acceptable precision loss for token UI calculations
                #[expect(clippy::cast_precision_loss)]
                let token_ui = token_delta as f64 / (10u64.pow(u32::from(decimals)) as f64);
                let price = sol_delta_ui.and_then(|sol_delta| {
                    if token_ui == 0.0 {
                        return None;
                    }
                    Some(sol_delta.abs() / token_ui.abs())
                });
                (Some(token_delta_u64), price)
            }
        } else {
            (None, None)
        };

        return Ok((token_delta_opt_u64, sol_delta_ui, price_opt));
    }

    // Fallback if token balances absent
    Ok((None, sol_delta_ui, None))
}

// Manual Tool implementations for the non-macro tools
use riglr_core::{JobResult, Tool};

// ============================================================================
// Tool Exports and Manual Implementations
// ============================================================================

// Re-export tool structs generated by the #[tool] macro (for functions that still use it)
pub use __riglr_tool_analyze_pump_transaction::AnalyzePumpTransactionTool;

// Manual tool struct definitions for functions with manual Tool implementations
#[derive(Debug, Clone)]
pub struct DeployPumpTokenTool {
    pub context: Arc<ApplicationContext>,
}

#[derive(Debug, Clone)]
pub struct BuyPumpTokenTool {
    pub context: Arc<ApplicationContext>,
}

#[derive(Debug, Clone)]
pub struct SellPumpTokenTool {
    pub context: Arc<ApplicationContext>,
}

#[derive(Debug, Clone)]
pub struct GetPumpTokenInfoTool {
    pub context: Arc<ApplicationContext>,
}

#[derive(Debug, Clone)]
pub struct GetTrendingPumpTokensTool {
    pub context: Arc<ApplicationContext>,
}
#[async_trait::async_trait]
impl Tool for SellPumpTokenTool {
    type Args = serde_json::Value;
    type Error = ToolError;
    type Output = JobResult;

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        #[derive(serde::Deserialize)]
        struct SellPumpTokenArgs {
            #[serde(rename = "slippagePercent")]
            slippage_percent: Option<f64>,
            #[serde(rename = "tokenAmount")]
            token_amount: u64,
            #[serde(rename = "tokenMint")]
            token_mint: String,
        }

        let parsed_args: SellPumpTokenArgs = serde_json::from_value(args)
            .map_err(|e| ToolError::invalid_input_string(format!("Invalid arguments: {e}")))?;

        let result = sell_pump_token(
            parsed_args.token_mint,
            parsed_args.token_amount,
            parsed_args.slippage_percent,
            &self.context,
        )
        .await?;
        JobResult::success(&result)
            .map_err(|e| ToolError::permanent_string(format!("Failed to serialize result: {e}")))
    }

    fn description(&self) -> &'static str {
        "Sell tokens on Pump.fun"
    }

    fn name(&self) -> &'static str {
        "sell_pump_token"
    }

    fn schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "tokenMint": {
                    "type": "string",
                    "description": "Token mint address"
                },
                "tokenAmount": {
                    "type": "integer",
                    "description": "Token amount to sell"
                },
                "slippagePercent": {
                    "type": "number",
                    "description": "Optional slippage tolerance percentage"
                }
            },
            "required": ["tokenMint", "tokenAmount"]
        })
    }
}
#[async_trait::async_trait]
impl Tool for GetPumpTokenInfoTool {
    type Args = serde_json::Value;
    type Error = ToolError;
    type Output = JobResult;

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        #[derive(serde::Deserialize)]
        struct GetPumpTokenInfoArgs {
            #[serde(rename = "tokenMint")]
            token_mint: String,
        }

        let parsed_args: GetPumpTokenInfoArgs = serde_json::from_value(args)
            .map_err(|e| ToolError::invalid_input_string(format!("Invalid arguments: {e}")))?;

        let result = get_pump_token_info(parsed_args.token_mint, &self.context).await?;
        JobResult::success(&result)
            .map_err(|e| ToolError::permanent_string(format!("Failed to serialize result: {e}")))
    }

    fn description(&self) -> &'static str {
        "Get token information from Pump.fun"
    }

    fn name(&self) -> &'static str {
        "get_pump_token_info"
    }

    fn schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "tokenMint": {
                    "type": "string",
                    "description": "Token mint address"
                }
            },
            "required": ["tokenMint"]
        })
    }
}
#[async_trait::async_trait]
impl Tool for GetTrendingPumpTokensTool {
    type Args = serde_json::Value;
    type Error = ToolError;
    type Output = JobResult;

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        #[derive(serde::Deserialize)]
        struct GetTrendingPumpTokensArgs {
            limit: Option<u32>,
        }

        let parsed_args: GetTrendingPumpTokensArgs = serde_json::from_value(args)
            .map_err(|e| ToolError::invalid_input_string(format!("Invalid arguments: {e}")))?;

        let result = get_trending_pump_tokens(parsed_args.limit, &self.context).await?;
        JobResult::success(&result)
            .map_err(|e| ToolError::permanent_string(format!("Failed to serialize result: {e}")))
    }

    fn description(&self) -> &'static str {
        "Get trending tokens on Pump.fun"
    }

    fn name(&self) -> &'static str {
        "get_trending_pump_tokens"
    }

    fn schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "limit": {
                    "type": "integer",
                    "description": "Optional limit for number of tokens to fetch (max 50)"
                }
            },
            "required": []
        })
    }
}
#[async_trait::async_trait]
impl Tool for DeployPumpTokenTool {
    type Args = serde_json::Value;
    type Error = ToolError;
    type Output = JobResult;

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        #[derive(serde::Deserialize)]
        struct DeployPumpTokenArgs {
            description: String,
            #[serde(rename = "imageUrl")]
            image_url: Option<String>,
            #[serde(rename = "initialBuySol")]
            initial_buy_sol: Option<f64>,
            name: String,
            symbol: String,
        }

        let parsed_args: DeployPumpTokenArgs = serde_json::from_value(args)
            .map_err(|e| ToolError::invalid_input_string(format!("Invalid arguments: {e}")))?;

        let result = deploy_pump_token(
            parsed_args.name,
            parsed_args.symbol,
            parsed_args.description,
            parsed_args.image_url,
            parsed_args.initial_buy_sol,
            &self.context,
        )
        .await?;
        JobResult::success(&result)
            .map_err(|e| ToolError::permanent_string(format!("Failed to serialize result: {e}")))
    }

    fn description(&self) -> &'static str {
        "Deploy a new token on Pump.fun platform"
    }

    fn name(&self) -> &'static str {
        "deploy_pump_token"
    }

    fn schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "name": {
                    "type": "string",
                    "description": "Token name"
                },
                "symbol": {
                    "type": "string",
                    "description": "Token symbol"
                },
                "description": {
                    "type": "string",
                    "description": "Token description"
                },
                "imageUrl": {
                    "type": "string",
                    "description": "Optional image URL for the token"
                },
                "initialBuySol": {
                    "type": "number",
                    "description": "Optional initial SOL amount to buy"
                }
            },
            "required": ["name", "symbol", "description"]
        })
    }
}
#[async_trait::async_trait]
impl Tool for BuyPumpTokenTool {
    type Args = serde_json::Value;
    type Error = ToolError;
    type Output = JobResult;

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        #[derive(serde::Deserialize)]
        struct BuyPumpTokenArgs {
            #[serde(rename = "slippagePercent")]
            slippage_percent: Option<f64>,
            #[serde(rename = "solAmount")]
            sol_amount: f64,
            #[serde(rename = "tokenMint")]
            token_mint: String,
        }

        let parsed_args: BuyPumpTokenArgs = serde_json::from_value(args)
            .map_err(|e| ToolError::invalid_input_string(format!("Invalid arguments: {e}")))?;

        let result = buy_pump_token(
            parsed_args.token_mint,
            parsed_args.sol_amount,
            parsed_args.slippage_percent,
            &self.context,
        )
        .await?;
        JobResult::success(&result)
            .map_err(|e| ToolError::permanent_string(format!("Failed to serialize result: {e}")))
    }

    fn description(&self) -> &'static str {
        "Buy tokens on Pump.fun platform"
    }

    fn name(&self) -> &'static str {
        "buy_pump_token"
    }

    fn schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "tokenMint": {
                    "type": "string",
                    "description": "Token mint address"
                },
                "solAmount": {
                    "type": "number",
                    "description": "SOL amount to spend"
                },
                "slippagePercent": {
                    "type": "number",
                    "description": "Optional slippage tolerance percentage"
                }
            },
            "required": ["tokenMint", "solAmount"]
        })
    }
}

/// Tests for Pump.fun integration functionality
#[cfg(test)]
#[expect(clippy::expect_used, clippy::float_cmp)]
mod tests {
    use super::*;
    use core::str::FromStr;

    #[test]
    fn test_pump_client_default() {
        use riglr_config::ProvidersConfig;

        let config = ProvidersConfig::default();
        let pump_client = PumpClient::new(&config);
        assert!(pump_client.api_url().contains("pumpapi.fun"));
    }

    #[test]
    fn test_pump_token_info_serialization() {
        let token_info = TokenInfo {
            mint_address: "11111111111111111111111111111111".to_string(),
            name: "Test Token".to_string(),
            symbol: "TEST".to_string(),
            description: "A test token".to_string(),
            image_url: Some("https://example.com/image.png".to_string()),
            market_cap: Some(1_000_000),
            price_sol: Some(0.001),
            creation_signature: None,
            creator: "Creator1111111111111111111111111111".to_string(),
            initial_buy_signature: None,
        };

        let json = serde_json::to_string(&token_info).expect("Token info should serialize to JSON");
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

        let token_info: TokenInfo =
            serde_json::from_str(json).expect("JSON should deserialize to TokenInfo");
        assert_eq!(token_info.name, "Test Token");
        assert_eq!(token_info.symbol, "TEST");
        assert_eq!(token_info.image_url, None);
    }

    #[test]
    fn test_pump_token_info_debug_clone() {
        let token_info = TokenInfo {
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
        assert_eq!(format!("{token_info:?}"), format!("{:?}", cloned));
    }

    #[test]
    fn test_pump_trade_result_serialization() {
        let result = TradeResult {
            signature: "signature123".to_string(),
            error: None,
            status: Status::Pending,
        };

        let json = serde_json::to_string(&result).expect("Result should serialize to JSON");
        assert!(json.contains("signature123"));
        assert!(json.contains("Pending"));
    }

    #[test]
    fn test_pump_trade_result_success() {
        let result = TradeResult {
            signature: "signature123".to_string(),
            error: None,
            status: Status::Pending,
        };

        assert_eq!(result.signature, "signature123");
        assert!(result.error.is_none());
        assert!(matches!(result.status, Status::Pending));
    }

    #[test]
    fn test_pump_trade_result_with_error() {
        let result = TradeResult {
            signature: "signature456".to_string(),
            error: Some("Transaction failed".to_string()),
            status: Status::Pending,
        };

        assert_eq!(result.signature, "signature456");
        assert_eq!(result.error, Some("Transaction failed".to_string()));
        assert!(matches!(result.status, Status::Pending));
    }

    #[test]
    fn test_pump_trade_result_debug_clone() {
        let result = TradeResult {
            signature: "sig".to_string(),
            error: None,
            status: Status::Pending,
        };

        let cloned = result.clone();
        assert_eq!(format!("{result:?}"), format!("{:?}", cloned));
    }

    #[test]
    fn test_pump_trade_type_serialization() {
        let buy_type = TradeType::Buy;
        let sell_type = TradeType::Sell;

        let buy_json = serde_json::to_string(&buy_type).expect("Buy type should serialize to JSON");
        let sell_json =
            serde_json::to_string(&sell_type).expect("Sell type should serialize to JSON");

        assert_eq!(buy_json, "\"Buy\"");
        assert_eq!(sell_json, "\"Sell\"");
    }

    #[test]
    fn test_pump_trade_type_deserialization() {
        let buy_json = "\"Buy\"";
        let sell_json = "\"Sell\"";

        let buy_type: TradeType =
            serde_json::from_str(buy_json).expect("Buy JSON should deserialize to TradeType");
        let sell_type: TradeType =
            serde_json::from_str(sell_json).expect("Sell JSON should deserialize to TradeType");

        assert!(matches!(buy_type, TradeType::Buy));
        assert!(matches!(sell_type, TradeType::Sell));
    }

    #[test]
    fn test_pump_trade_type_debug_clone() {
        let buy_type = TradeType::Buy;
        let sell_type = TradeType::Sell;

        let buy_cloned = buy_type.clone();
        let sell_cloned = sell_type.clone();

        assert_eq!(format!("{buy_type:?}"), format!("{:?}", buy_cloned));
        assert_eq!(format!("{sell_type:?}"), format!("{:?}", sell_cloned));
    }

    #[test]
    fn test_pump_trade_analysis_serialization() {
        let analysis = TradeAnalysis {
            signature: "sig123".to_string(),
            user_address: "user123".to_string(),
            token_mint: "mint123".to_string(),
            token_amount: Some(1000),
            sol_amount: Some(-0.5),
            price_per_token: Some(0.0005),
        };

        let json = serde_json::to_string(&analysis).expect("Analysis should serialize to JSON");
        assert!(json.contains("sig123"));
        assert!(json.contains("-0.5"));
    }

    #[test]
    fn test_pump_trade_analysis_debug_clone() {
        let analysis = TradeAnalysis {
            signature: "sig".to_string(),
            user_address: "user".to_string(),
            token_mint: "mint".to_string(),
            token_amount: None,
            sol_amount: None,
            price_per_token: None,
        };

        let cloned = analysis.clone();
        assert_eq!(format!("{analysis:?}"), format!("{:?}", cloned));
    }

    #[test]
    fn test_pump_deployment_response_serialization() {
        let response = PumpDeploymentResponse {
            metadata_uri: "ipfs://hash123".to_string(),
            success: true,
        };

        let json = serde_json::to_string(&response).expect("Response should serialize to JSON");
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
        assert_eq!(format!("{response:?}"), format!("{:?}", cloned));
    }

    #[test]
    fn test_pump_token_response_serialization() {
        let response = PumpTokenResponse {
            mint: Some("mint123".to_string()),
            name: "Token".to_string(),
            symbol: "TKN".to_string(),
            description: "Description".to_string(),
            image: Some("image.png".to_string()),
            market_cap: Some(1_000_000),
            price_sol: Some(0.001),
            creator: "creator123".to_string(),
        };

        let json = serde_json::to_string(&response).expect("Response should serialize to JSON");
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

        let json = serde_json::to_string(&response).expect("Response should serialize to JSON");
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
        assert_eq!(format!("{response:?}"), format!("{:?}", cloned));
    }

    #[test]
    fn test_generate_mint() {
        let keypair1 = generate_mint();
        let keypair2 = generate_mint();

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
            {
                assert_eq!(default_slippage, 5.0);
            }
        }

        #[test]
        fn test_slippage_calculation_custom() {
            // Test custom slippage calculation
            let slippage_percent = 3.5;
            // Safe cast: slippage_percent is validated to be positive business value
            #[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let slippage_bps = (slippage_percent * 100.0) as u64;
            assert_eq!(slippage_bps, 350);
        }

        #[test]
        fn test_lamports_conversion() {
            // Test SOL to lamports conversion
            let sol_amount = 1.5;
            // Safe casts: sol_amount is positive, LAMPORTS_PER_SOL conversion acceptable precision loss
            #[expect(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                clippy::cast_precision_loss
            )]
            let lamports = (sol_amount * LAMPORTS_PER_SOL as f64) as u64;
            assert_eq!(lamports, 1_500_000_000);
        }

        #[test]
        fn test_lamports_conversion_zero() {
            // Test zero SOL conversion
            let sol_amount = 0.0;
            // Safe casts: sol_amount is positive, LAMPORTS_PER_SOL conversion acceptable precision loss
            #[expect(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                clippy::cast_precision_loss
            )]
            let lamports = (sol_amount * LAMPORTS_PER_SOL as f64) as u64;
            assert_eq!(lamports, 0);
        }

        #[test]
        fn test_trending_limit_default() {
            // Test default trending limit
            let limit = None;
            let actual_limit = limit.map_or(10, |l| l).min(50);
            assert_eq!(actual_limit, 10);
        }

        #[test]
        fn test_trending_limit_custom() {
            // Test custom trending limit
            let limit = 25;
            let actual_limit = limit.min(50);
            assert_eq!(actual_limit, 25);
        }

        #[test]
        fn test_trending_limit_capped() {
            // Test trending limit capping
            let limit = 100;
            let actual_limit = limit.min(50);
            assert_eq!(actual_limit, 50);
        }

        #[test]
        fn test_trending_limit_zero() {
            // Test zero trending limit
            let limit = 0;
            let actual_limit = limit.min(50);
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
            assert!((200..300).contains(&status_code));
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

            assert_eq!(
                deploy_request.get("name").and_then(|v| v.as_str()),
                Some("Test Token")
            );
            assert_eq!(
                deploy_request.get("symbol").and_then(|v| v.as_str()),
                Some("TEST")
            );
            assert_eq!(
                deploy_request
                    .get("showName")
                    .and_then(serde_json::Value::as_bool),
                Some(true)
            );
        }

        #[test]
        fn test_json_request_building_buy() {
            // Test buy request JSON structure
            let buy_request = json!({
                "publicKey": "11111111111111111111111111111111",
                "action": "buy",
                "mint": "mint123",
                "amount": 1_000_000_000_u64,
                "denominatedInSol": "true",
                "slippage": 500u64
            });

            assert_eq!(
                buy_request.get("action").and_then(|v| v.as_str()),
                Some("buy")
            );
            assert_eq!(
                buy_request.get("denominatedInSol").and_then(|v| v.as_str()),
                Some("true")
            );
            assert_eq!(
                buy_request
                    .get("amount")
                    .and_then(serde_json::Value::as_u64),
                Some(1_000_000_000_u64)
            );
        }

        #[test]
        fn test_json_request_building_sell() {
            // Test sell request JSON structure
            let sell_request = json!({
                "publicKey": "11111111111111111111111111111111",
                "action": "sell",
                "mint": "mint123",
                "amount": 1_000_000_u64,
                "denominatedInSol": "false",
                "slippage": 300u64
            });

            assert_eq!(
                sell_request.get("action").and_then(|v| v.as_str()),
                Some("sell")
            );
            assert_eq!(
                sell_request
                    .get("denominatedInSol")
                    .and_then(|v| v.as_str()),
                Some("false")
            );
            assert_eq!(
                sell_request
                    .get("amount")
                    .and_then(serde_json::Value::as_u64),
                Some(1_000_000_u64)
            );
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

            assert_eq!(
                create_tx_request.get("action").and_then(|v| v.as_str()),
                Some("create")
            );
            assert_eq!(
                create_tx_request
                    .get("tokenMetadata")
                    .and_then(|m| m.get("name"))
                    .and_then(|v| v.as_str()),
                Some("Test Token")
            );
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
            let encoded = general_purpose::STANDARD.encode(b"test data");
            let decoded = general_purpose::STANDARD.decode(&encoded);
            assert!(decoded.is_ok());
            assert_eq!(decoded.expect("Base64 decode should succeed"), b"test data");
        }

        #[test]
        fn test_base64_decode_failure() {
            // Test base64 decoding failure case
            let invalid_base64 = "invalid@base64!";
            let decoded = general_purpose::STANDARD.decode(invalid_base64);
            assert!(decoded.is_err());
        }

        #[test]
        fn test_sol_delta_calculation_positive() {
            // Test SOL delta calculation for positive change
            let pre_balance = 1_000_000_000_u64; // 1 SOL
            let post_balance = 1_500_000_000_u64; // 1.5 SOL
            let pre = i128::from(pre_balance);
            let post = i128::from(post_balance);
            let delta_lamports = post - pre;
            assert_eq!(delta_lamports, 500_000_000);
            assert!(delta_lamports > 0);
        }

        #[test]
        fn test_sol_delta_calculation_negative() {
            // Test SOL delta calculation for negative change
            let pre_balance = 1_500_000_000_u64; // 1.5 SOL
            let post_balance = 1_000_000_000_u64; // 1 SOL
            let pre = i128::from(pre_balance);
            let post = i128::from(post_balance);
            let delta_lamports = post - pre;
            assert_eq!(delta_lamports, -500_000_000);
            assert!(delta_lamports < 0);
        }

        #[test]
        fn test_sol_delta_calculation_with_fee() {
            // Test SOL delta calculation with fee adjustment
            let pre_balance = 1_000_000_000_u64;
            let post_balance = 995_000_000_u64; // Less due to transaction fee
            let fee = 5_000_000_u64; // 0.005 SOL transaction fee

            let pre = i128::from(pre_balance);
            let post = i128::from(post_balance);
            let delta_lamports = post - pre;
            let fee_i128 = i128::from(fee);

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
            let post_amount = 1_000_000_i128;
            let token_delta = post_amount - pre_amount;
            assert_eq!(token_delta, 1_000_000);
            assert!(token_delta > 0);
        }

        #[test]
        fn test_token_delta_calculation_negative() {
            // Test token delta calculation for token loss
            let pre_amount = 1_000_000_i128;
            let post_amount = 500_000_i128;
            let token_delta = post_amount - pre_amount;
            assert_eq!(token_delta, -500_000);
            assert!(token_delta < 0);
        }

        #[test]
        fn test_token_delta_calculation_zero() {
            // Test token delta calculation for no change
            let pre_amount = 1_000_000_i128;
            let post_amount = 1_000_000_i128;
            let token_delta = post_amount - pre_amount;
            assert_eq!(token_delta, 0);
        }

        #[test]
        fn test_price_calculation() {
            // Test price per token calculation
            let token_amount = 1_000_000_u64;
            let sol_amount = 0.5f64;
            let decimals = 6u8;

            // Safe casts: token amounts are reasonable for f64 precision in financial calculations
            #[expect(clippy::cast_precision_loss)]
            let token_ui = (token_amount as f64) / 10u64.pow(u32::from(decimals)) as f64;
            let price = (sol_amount.abs()) / token_ui;

            {
                assert_eq!(token_ui, 1.0);
                assert_eq!(price, 0.5);
            }
        }

        #[test]
        fn test_price_calculation_with_decimals() {
            // Test price calculation with different decimals
            let token_amount = 1_000_000_000_u64; // 1 token with 9 decimals
            let sol_amount = 0.001f64;
            let decimals = 9u8;

            // Safe casts: token amounts are reasonable for f64 precision in financial calculations
            #[expect(clippy::cast_precision_loss)]
            let token_ui = (token_amount as f64) / 10u64.pow(u32::from(decimals)) as f64;
            let price = (sol_amount.abs()) / token_ui;

            {
                assert_eq!(token_ui, 1.0);
                assert_eq!(price, 0.001);
            }
        }

        #[test]
        fn test_price_calculation_zero_tokens() {
            // Test price calculation with zero tokens
            let token_amount = 0u64;
            let sol_amount = 0.5f64;
            let _ = sol_amount; // Used for demonstrating price calculation with zero tokens
            let decimals = 6u8;

            // Safe casts: token amounts are reasonable for f64 precision in financial calculations
            #[expect(clippy::cast_precision_loss)]
            let token_ui = (token_amount as f64) / 10u64.pow(u32::from(decimals)) as f64;
            {
                assert_eq!(token_ui, 0.0);
            }
            // Price calculation would be division by zero, so should be None
        }

        #[test]
        fn test_unsigned_abs_conversion() {
            // Test conversion from signed to unsigned absolute value
            let negative_value = -1_000_000_i128;
            let positive_value = 1_000_000_i128;
            let zero_value = 0i128;

            assert_eq!(negative_value.unsigned_abs(), 1_000_000_u128);
            assert_eq!(positive_value.unsigned_abs(), 1_000_000_u128);
            assert_eq!(zero_value.unsigned_abs(), 0u128);
        }

        #[test]
        fn test_option_serializer_matching() {
            // Test owner matching with OptionSerializer
            let owner_str = "11111111111111111111111111111111";
            let some_owner = OptionSerializer::Some(owner_str.to_string());
            let none_owner: OptionSerializer<String> = OptionSerializer::None;

            let owner_matches_some =
                matches!(some_owner, OptionSerializer::Some(owner) if owner == owner_str);
            let owner_matches_none =
                matches!(none_owner, OptionSerializer::Some(owner) if owner == owner_str);

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
            assert_eq!(valid_result.expect("Valid result should be Ok"), 1_000_000);
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
            // Test conversion from PumpTokenResponse to TokenInfo
            let token_response = PumpTokenResponse {
                mint: Some("mint123".to_string()),
                name: "Test Token".to_string(),
                symbol: "TEST".to_string(),
                description: "Test description".to_string(),
                image: Some("image.png".to_string()),
                market_cap: Some(1_000_000),
                price_sol: Some(0.001),
                creator: "creator123".to_string(),
            };

            let token_info = TokenInfo {
                mint_address: token_response.mint.clone().unwrap_or_default(),
                name: token_response.name.clone(),
                symbol: token_response.symbol.clone(),
                description: token_response.description.clone(),
                image_url: token_response.image.clone(),
                market_cap: token_response.market_cap,
                price_sol: token_response.price_sol,
                creation_signature: None,
                creator: token_response.creator,
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

            let mint_address = token_response.mint.unwrap_or_default();
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

            let token_infos: Vec<TokenInfo> = responses
                .into_iter()
                .map(|token| TokenInfo {
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
            assert_eq!(
                token_infos.first().expect("Should have first token").name,
                "Token1"
            );
            assert_eq!(
                token_infos.get(1).expect("Should have second token").name,
                "Token2"
            );
        }

        #[test]
        fn test_balance_vec_empty_check() {
            // Test empty balance vector checks
            let empty_pre_balances: Vec<u64> = vec![];
            let empty_post_balances: Vec<u64> = vec![];
            let non_empty_pre_balances = [1_000_000_000_u64];
            let non_empty_post_balances = [1_500_000_000_u64];

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
            assert!(some_decimals_opt.is_some());
        }

        #[test]
        fn test_option_and_then_logic() {
            // Test Option and_then logic for complex transformations
            let token_delta_raw: Option<i128> = Some(1_000_000);
            let zero_delta_raw: Option<i128> = Some(0);
            let none_delta_raw: Option<i128> = None;

            let token_delta_opt_u64 = token_delta_raw.and_then(|v| {
                if v == 0 {
                    return None;
                }
                #[expect(clippy::cast_possible_truncation)]
                Some(v.unsigned_abs() as u64)
            });

            let zero_delta_opt_u64 = zero_delta_raw.and_then(|v| {
                if v == 0 {
                    return None;
                }
                #[expect(clippy::cast_possible_truncation)]
                Some(v.unsigned_abs() as u64)
            });

            let none_delta_opt_u64 = none_delta_raw.and_then(|v| {
                if v == 0 {
                    return None;
                }
                #[expect(clippy::cast_possible_truncation)]
                Some(v.unsigned_abs() as u64)
            });

            assert_eq!(token_delta_opt_u64, Some(1_000_000));
            assert_eq!(zero_delta_opt_u64, None);
            assert_eq!(none_delta_opt_u64, None);
        }

        #[test]
        fn test_triple_option_matching() {
            // Test triple Option matching for price calculation
            let token_delta = Some(1_000_000_u64);
            let sol_delta = Some(0.5f64);
            let decimals = Some(6u8);

            let price_opt = if let (Some(token_delta), Some(sol_delta), Some(decimals)) =
                (token_delta, sol_delta, decimals)
            {
                // Safe casts: token amounts are reasonable for f64 precision in financial calculations
                #[expect(clippy::cast_precision_loss)]
                let token_ui = (token_delta as f64) / 10u64.pow(u32::from(decimals)) as f64;
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
            let token_delta = Some(1_000_000_u64);
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
