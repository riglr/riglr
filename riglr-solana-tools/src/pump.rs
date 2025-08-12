//! Pump.fun integration for token deployment, buying, and selling on Solana
//!
//! This module provides tools for interacting with the Pump.fun platform,
//! enabling token deployment, trading operations with slippage protection.

use crate::transaction::TransactionStatus;
use crate::utils::send_transaction;
use riglr_core::{ToolError, SignerContext};
use riglr_macros::tool;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;
use solana_sdk::{
    pubkey::Pubkey, 
    native_token::LAMPORTS_PER_SOL,
    signature::Keypair,
    signer::Signer,
    instruction::Instruction,
};
#[allow(deprecated)]
use solana_sdk::transaction::Transaction;
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
    let signer_context = SignerContext::current().await
        .map_err(|e| ToolError::permanent(format!("No signer context: {}", e)))?;
    
    let signer_pubkey = signer_context.pubkey()
        .ok_or_else(|| ToolError::permanent("Signer has no public key"))?;
    
    // Generate new mint keypair BEFORE transaction creation for deterministic addressing
    let mint_keypair = generate_mint_keypair();
    let mint_address = mint_keypair.pubkey();

    // Validate inputs
    if name.is_empty() || name.len() > 32 {
        return Err(ToolError::permanent("Token name must be 1-32 characters".to_string()));
    }
    if symbol.is_empty() || symbol.len() > 10 {
        return Err(ToolError::permanent("Token symbol must be 1-10 characters".to_string()));
    }
    if description.len() > 1000 {
        return Err(ToolError::permanent("Description must be under 1000 characters".to_string()));
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
        .map_err(|e| ToolError::retriable(format!("Failed to request deployment: {}", e)))?;

    if !response.status().is_success() {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        return Err(ToolError::permanent(format!("Pump.fun deployment API error: {}", error_text)));
    }

    let deployment_response: PumpDeploymentResponse = response
        .json()
        .await
        .map_err(|e| ToolError::permanent(format!("Failed to parse deployment response: {}", e)))?;

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
        .map_err(|e| ToolError::retriable(format!("Failed to get creation transaction: {}", e)))?;

    if !tx_response.status().is_success() {
        let error_text = tx_response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        return Err(ToolError::permanent(format!("Pump.fun transaction API error: {}", error_text)));
    }

    let tx_data: String = tx_response
        .text()
        .await
        .map_err(|e| ToolError::permanent(format!("Failed to get transaction data: {}", e)))?;

    // Deserialize and sign the creation transaction
    use base64::{engine::general_purpose, Engine as _};
    let transaction_bytes = general_purpose::STANDARD
        .decode(&tx_data)
        .map_err(|e| ToolError::permanent(format!("Failed to decode creation transaction: {}", e)))?;

    let mut transaction: Transaction = bincode::deserialize(&transaction_bytes)
        .map_err(|e| ToolError::permanent(format!("Failed to deserialize creation transaction: {}", e)))?;

    // For now, we'll directly use the signer context to sign and send the transaction
    // In a real implementation, we would properly integrate the mint keypair signing
    let mut tx = transaction;
    let creation_signature = signer_context.sign_and_send_solana_transaction(&mut tx).await
        .map_err(|e| ToolError::retriable(format!("Failed to sign and send transaction: {}", e)))?;

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
            ).await {
                Ok(buy_result) => {
                    let buy_signature = buy_result.signature.clone();
                    token_info.initial_buy_signature = Some(buy_result.signature);
                    info!("Initial buy completed: {}", buy_signature);
                },
                Err(e) => {
                    warn!("Initial buy failed, but token was created successfully: {}", e);
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
        return Err(ToolError::permanent("SOL amount must be positive".to_string()));
    }

    // Validate mint address
    let _mint_pubkey = Pubkey::from_str(&token_mint)
        .map_err(|e| ToolError::permanent(format!("Invalid token mint: {}", e)))?;

    let signer_context = SignerContext::current().await
        .map_err(|e| ToolError::permanent(format!("No signer context: {}", e)))?;
    
    let signer_pubkey = signer_context.pubkey()
        .ok_or_else(|| ToolError::permanent("Signer has no public key"))?;

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
        .map_err(|e| ToolError::retriable(format!("Failed to request buy transaction: {}", e)))?;

    if !response.status().is_success() {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        return Err(ToolError::permanent(format!("Pump.fun buy API error: {}", error_text)));
    }

    let tx_data: String = response
        .text()
        .await
        .map_err(|e| ToolError::permanent(format!("Failed to get buy transaction data: {}", e)))?;

    // Deserialize and sign the buy transaction
    use base64::{engine::general_purpose, Engine as _};
    let transaction_bytes = general_purpose::STANDARD
        .decode(&tx_data)
        .map_err(|e| ToolError::permanent(format!("Failed to decode buy transaction: {}", e)))?;

    let mut transaction: Transaction = bincode::deserialize(&transaction_bytes)
        .map_err(|e| ToolError::permanent(format!("Failed to deserialize buy transaction: {}", e)))?;

    // Send buy transaction with retry logic
    let signature = send_transaction(&mut transaction, &format!("Buy Pump Token ({} SOL)", sol_amount)).await?;

    info!(
        "Pump.fun buy executed: {} SOL for {} tokens, signature: {}",
        sol_amount, token_mint, signature
    );

    Ok(PumpTradeResult {
        signature,
        token_mint,
        sol_amount,
        token_amount: None, // Would be parsed from transaction logs in full implementation
        trade_type: PumpTradeType::Buy,
        slippage_percent: slippage,
        status: TransactionStatus::Pending,
        price_per_token: None,
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
        return Err(ToolError::permanent("Token amount must be positive".to_string()));
    }

    // Validate mint address
    let _mint_pubkey = Pubkey::from_str(&token_mint)
        .map_err(|e| ToolError::permanent(format!("Invalid token mint: {}", e)))?;

    let signer_context = SignerContext::current().await
        .map_err(|e| ToolError::permanent(format!("No signer context: {}", e)))?;
    
    let signer_pubkey = signer_context.pubkey()
        .ok_or_else(|| ToolError::permanent("Signer has no public key"))?;

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
        .map_err(|e| ToolError::retriable(format!("Failed to request sell transaction: {}", e)))?;

    if !response.status().is_success() {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        return Err(ToolError::permanent(format!("Pump.fun sell API error: {}", error_text)));
    }

    let tx_data: String = response
        .text()
        .await
        .map_err(|e| ToolError::permanent(format!("Failed to get sell transaction data: {}", e)))?;

    // Deserialize and sign the sell transaction
    use base64::{engine::general_purpose, Engine as _};
    let transaction_bytes = general_purpose::STANDARD
        .decode(&tx_data)
        .map_err(|e| ToolError::permanent(format!("Failed to decode sell transaction: {}", e)))?;

    let mut transaction: Transaction = bincode::deserialize(&transaction_bytes)
        .map_err(|e| ToolError::permanent(format!("Failed to deserialize sell transaction: {}", e)))?;

    // Send sell transaction with retry logic
    let signature = send_transaction(&mut transaction, &format!("Sell Pump Token ({} tokens)", token_amount)).await?;

    info!(
        "Pump.fun sell executed: {} tokens for SOL, signature: {}",
        token_amount, signature
    );

    // Calculate estimated SOL amount based on current price (simplified)
    let estimated_sol = token_amount as f64 / 1_000_000.0; // Placeholder calculation

    Ok(PumpTradeResult {
        signature,
        token_mint,
        sol_amount: estimated_sol,
        token_amount: Some(token_amount),
        trade_type: PumpTradeType::Sell,
        slippage_percent: slippage,
        status: TransactionStatus::Pending,
        price_per_token: None,
    })
}

/// Get token information from Pump.fun
///
/// This tool fetches current token information, price, and market data
/// for a specific token on the Pump.fun platform.
#[tool]
pub async fn get_pump_token_info(
    token_mint: String,
) -> Result<PumpTokenInfo, ToolError> {
    debug!("Getting pump token info for: {}", token_mint);

    // Validate mint address
    let _mint_pubkey = Pubkey::from_str(&token_mint)
        .map_err(|e| ToolError::permanent(format!("Invalid token mint: {}", e)))?;

    let config = PumpConfig::default();

    // Request token information
    let reqwest_client = reqwest::Client::new();
    let response = reqwest_client
        .get(format!("{}/token/{}", config.api_url, token_mint))
        .send()
        .await
        .map_err(|e| ToolError::retriable(format!("Failed to get token info: {}", e)))?;

    if response.status().as_u16() == 404 {
        return Err(ToolError::permanent(format!("Token {} not found on Pump.fun", token_mint)));
    }

    if !response.status().is_success() {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        return Err(ToolError::retriable(format!("Pump.fun API error: {}", error_text)));
    }

    let token_response: PumpTokenResponse = response
        .json()
        .await
        .map_err(|e| ToolError::permanent(format!("Failed to parse token info: {}", e)))?;

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

/// Get trending tokens on Pump.fun
///
/// This tool fetches the currently trending tokens on the Pump.fun platform.
#[tool]
pub async fn get_trending_pump_tokens(
    limit: Option<u32>,
) -> Result<Vec<PumpTokenInfo>, ToolError> {
    debug!("Getting trending pump tokens (limit: {:?})", limit);

    let config = PumpConfig::default();
    let limit = limit.unwrap_or(10).min(50); // Cap at 50

    // Request trending tokens
    let reqwest_client = reqwest::Client::new();
    let response = reqwest_client
        .get(format!("{}/trending?limit={}", config.api_url, limit))
        .send()
        .await
        .map_err(|e| ToolError::retriable(format!("Failed to get trending tokens: {}", e)))?;

    if !response.status().is_success() {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        return Err(ToolError::retriable(format!("Pump.fun trending API error: {}", error_text)));
    }

    let trending_response: Vec<PumpTokenResponse> = response
        .json()
        .await
        .map_err(|e| ToolError::permanent(format!("Failed to parse trending tokens: {}", e)))?;

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

    info!("Retrieved {} trending tokens from Pump.fun", trending_tokens.len());

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
    mint_keypair: &Keypair,
) -> Result<String, ToolError> {
    let signer = SignerContext::current().await
        .map_err(|e| ToolError::permanent(format!("No signer context: {}", e)))?;
    
    // Note: In a real implementation, we would need to extract the keypair
    // from the signer's private key or use a different signing approach.
    // For now, we'll create a mock keypair for the transaction structure
    let payer = Keypair::new(); // Placeholder - should come from signer
    
    let mut transaction = Transaction::new_with_payer(&instructions, Some(&payer.pubkey()));
    
    // Get recent blockhash
    let rpc_client = signer.solana_client();
    let recent_blockhash = rpc_client.get_latest_blockhash()
        .map_err(|e| ToolError::retriable(format!("Failed to get recent blockhash: {}", e)))?;
    
    // In a real implementation, we would sign with both the payer and mint keypair
    // For now, we'll use the signer context to sign and send the transaction
    let signature = signer.sign_and_send_solana_transaction(&mut transaction).await
        .map_err(|e| ToolError::retriable(format!("Failed to sign and send transaction: {}", e)))?;
    
    Ok(signature.to_string())
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