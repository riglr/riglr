//! Complete DexScreener API implementation with real market data fetching
//!
//! This module provides production-ready integration with the DexScreener API
//! for fetching token prices, liquidity, and market data.

use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};

/// Response from DexScreener API containing token pair information
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DexScreenerResponse {
    /// Schema version of the API response
    #[serde(rename = "schemaVersion")]
    pub schema_version: String,
    /// List of token pairs returned by the API
    pub pairs: Vec<PairInfo>,
}

/// Information about a trading pair from DexScreener
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PairInfo {
    /// Blockchain network identifier
    #[serde(rename = "chainId")]
    pub chain_id: String,
    /// Decentralized exchange identifier
    #[serde(rename = "dexId")]
    pub dex_id: String,
    /// URL to view this pair on DexScreener
    pub url: String,
    /// Smart contract address of the trading pair
    #[serde(rename = "pairAddress")]
    pub pair_address: String,
    /// Optional labels associated with this pair
    pub labels: Option<Vec<String>>,
    /// Base token information
    #[serde(rename = "baseToken")]
    pub base_token: Token,
    /// Quote token information
    #[serde(rename = "quoteToken")]
    pub quote_token: Token,
    /// Price in native chain token (e.g., ETH, SOL)
    #[serde(rename = "priceNative")]
    pub price_native: String,
    /// Price in USD
    #[serde(rename = "priceUsd")]
    pub price_usd: Option<String>,
    /// Liquidity information for this pair
    pub liquidity: Option<Liquidity>,
    /// Trading volume statistics
    pub volume: Option<Volume>,
    /// Price change statistics
    #[serde(rename = "priceChange")]
    pub price_change: Option<PriceChange>,
    /// Transaction statistics
    #[serde(rename = "txns")]
    pub txns: Option<Transactions>,
    /// Market capitalization in USD
    #[serde(rename = "marketCap")]
    pub market_cap: Option<f64>,
    /// Fully diluted valuation in USD
    #[serde(rename = "fdv")]
    pub fdv: Option<f64>,
}

/// Liquidity information for a trading pair
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Liquidity {
    /// Total liquidity in USD
    pub usd: Option<f64>,
    /// Liquidity of the base token
    pub base: Option<f64>,
    /// Liquidity of the quote token
    pub quote: Option<f64>,
}

/// Trading volume statistics over different time periods
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Volume {
    /// Trading volume in the last 24 hours
    #[serde(default)]
    pub h24: Option<f64>,
    /// Trading volume in the last 6 hours
    #[serde(default)]
    pub h6: Option<f64>,
    /// Trading volume in the last 1 hour
    #[serde(default)]
    pub h1: Option<f64>,
    /// Trading volume in the last 5 minutes
    #[serde(default)]
    pub m5: Option<f64>,
}

/// Price change statistics over different time periods
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PriceChange {
    /// Price change percentage in the last 24 hours
    #[serde(default)]
    pub h24: Option<f64>,
    /// Price change percentage in the last 6 hours
    #[serde(default)]
    pub h6: Option<f64>,
    /// Price change percentage in the last 1 hour
    #[serde(default)]
    pub h1: Option<f64>,
    /// Price change percentage in the last 5 minutes
    #[serde(default)]
    pub m5: Option<f64>,
}

/// Transaction statistics over different time periods
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Transactions {
    /// Transaction statistics for the last 24 hours
    #[serde(default)]
    pub h24: Option<TransactionStats>,
    /// Transaction statistics for the last 6 hours
    #[serde(default)]
    pub h6: Option<TransactionStats>,
    /// Transaction statistics for the last 1 hour
    #[serde(default)]
    pub h1: Option<TransactionStats>,
    /// Transaction statistics for the last 5 minutes
    #[serde(default)]
    pub m5: Option<TransactionStats>,
}

/// Buy and sell transaction statistics
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TransactionStats {
    /// Number of buy transactions
    pub buys: Option<u64>,
    /// Number of sell transactions
    pub sells: Option<u64>,
}

/// Token information
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Token {
    /// Token contract address
    pub address: String,
    /// Full name of the token
    pub name: String,
    /// Token symbol/ticker
    pub symbol: String,
}

/// Search for tokens or pairs on DexScreener
pub async fn search_ticker(ticker: String) -> Result<DexScreenerResponse> {
    let client = Client::new();
    let url = format!(
        "https://api.dexscreener.com/latest/dex/search/?q={}&limit=8",
        ticker
    );

    let response = client.get(&url).send().await?;

    if response.status().is_client_error() {
        let res = response.text().await?;
        tracing::error!("DexScreener API error: {:?}", res);
        return Err(anyhow::anyhow!("DexScreener API error: {:?}", res));
    }

    let data: serde_json::Value = response.json().await?;
    let mut dex_response: DexScreenerResponse = serde_json::from_value(data)?;

    // Limit results to 8
    dex_response.pairs.truncate(8);

    Ok(dex_response)
}

/// Get token pairs by token address
pub async fn get_pairs_by_token(token_address: &str) -> Result<DexScreenerResponse> {
    let client = Client::new();
    let url = format!(
        "https://api.dexscreener.com/latest/dex/tokens/{}",
        token_address
    );

    let response = client.get(&url).send().await?;

    if !response.status().is_success() {
        let res = response.text().await?;
        return Err(anyhow::anyhow!("Failed to fetch token pairs: {}", res));
    }

    let data: serde_json::Value = response.json().await?;
    let dex_response: DexScreenerResponse = serde_json::from_value(data)?;

    Ok(dex_response)
}

/// Get pairs by pair address
pub async fn get_pair_by_address(pair_address: &str) -> Result<PairInfo> {
    let client = Client::new();
    let url = format!(
        "https://api.dexscreener.com/latest/dex/pairs/{}",
        pair_address
    );

    let response = client.get(&url).send().await?;

    if !response.status().is_success() {
        let res = response.text().await?;
        return Err(anyhow::anyhow!("Failed to fetch pair: {}", res));
    }

    let data: serde_json::Value = response.json().await?;
    let pair_response: DexScreenerResponse = serde_json::from_value(data)?;

    pair_response
        .pairs
        .into_iter()
        .next()
        .ok_or_else(|| anyhow::anyhow!("No pair found for address: {}", pair_address))
}

/// Find the best liquidity pair for a token
pub fn find_best_liquidity_pair(pairs: Vec<PairInfo>) -> Option<PairInfo> {
    pairs.into_iter().max_by_key(|p| {
        p.liquidity
            .as_ref()
            .and_then(|l| l.usd)
            .map_or(0, |usd| (usd * 1000.0) as u64)
    })
}

/// Extract token price from the best pair
pub fn get_token_price(pairs: &[PairInfo], token_address: &str) -> Option<String> {
    pairs
        .iter()
        .filter(|p| p.base_token.address.eq_ignore_ascii_case(token_address))
        .max_by_key(|p| {
            p.liquidity
                .as_ref()
                .and_then(|l| l.usd)
                .map_or(0, |usd| (usd * 1000.0) as u64)
        })
        .and_then(|p| p.price_usd.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_search_ticker() {
        let response = search_ticker("BONK".to_string()).await.unwrap();
        assert_eq!(response.schema_version, "1.0.0");
        assert!(!response.pairs.is_empty());
    }

    #[tokio::test]
    async fn test_search_by_mint() {
        let response = search_ticker(
            "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263".to_string(), // BONK token
        )
        .await
        .unwrap();
        assert_eq!(response.schema_version, "1.0.0");
        assert!(!response.pairs.is_empty());
    }

    #[tokio::test]
    async fn test_get_pairs_by_token() {
        let response = get_pairs_by_token("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48") // USDC
            .await
            .unwrap();
        assert!(!response.pairs.is_empty());
    }
}
