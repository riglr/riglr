//! Complete DexScreener API implementation with real market data fetching
//!
//! This module provides production-ready integration with the DexScreener API
//! for fetching token prices, liquidity, and market data.

use reqwest::Client;
use serde::{Deserialize, Serialize};
use anyhow::Result;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DexScreenerResponse {
    #[serde(rename = "schemaVersion")]
    pub schema_version: String,
    pub pairs: Vec<PairInfo>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PairInfo {
    #[serde(rename = "chainId")]
    pub chain_id: String,
    #[serde(rename = "dexId")]
    pub dex_id: String,
    pub url: String,
    #[serde(rename = "pairAddress")]
    pub pair_address: String,
    pub labels: Option<Vec<String>>,
    #[serde(rename = "baseToken")]
    pub base_token: Token,
    #[serde(rename = "quoteToken")]
    pub quote_token: Token,
    #[serde(rename = "priceNative")]
    pub price_native: String,
    #[serde(rename = "priceUsd")]
    pub price_usd: Option<String>,
    pub liquidity: Option<Liquidity>,
    pub volume: Option<Volume>,
    #[serde(rename = "priceChange")]
    pub price_change: Option<PriceChange>,
    #[serde(rename = "txns")]
    pub txns: Option<Transactions>,
    #[serde(rename = "marketCap")]
    pub market_cap: Option<f64>,
    #[serde(rename = "fdv")]
    pub fdv: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Liquidity {
    pub usd: Option<f64>,
    pub base: Option<f64>,
    pub quote: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Volume {
    #[serde(default)]
    pub h24: Option<f64>,
    #[serde(default)]
    pub h6: Option<f64>,
    #[serde(default)]
    pub h1: Option<f64>,
    #[serde(default)]
    pub m5: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PriceChange {
    #[serde(default)]
    pub h24: Option<f64>,
    #[serde(default)]
    pub h6: Option<f64>,
    #[serde(default)]
    pub h1: Option<f64>,
    #[serde(default)]
    pub m5: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Transactions {
    #[serde(default)]
    pub h24: Option<TransactionStats>,
    #[serde(default)]
    pub h6: Option<TransactionStats>,
    #[serde(default)]
    pub h1: Option<TransactionStats>,
    #[serde(default)]
    pub m5: Option<TransactionStats>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TransactionStats {
    pub buys: Option<u64>,
    pub sells: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Token {
    pub address: String,
    pub name: String,
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

    pair_response.pairs
        .into_iter()
        .next()
        .ok_or_else(|| anyhow::anyhow!("No pair found for address: {}", pair_address))
}

/// Find the best liquidity pair for a token
pub fn find_best_liquidity_pair(pairs: Vec<PairInfo>) -> Option<PairInfo> {
    pairs.into_iter()
        .max_by_key(|p| {
            p.liquidity
                .as_ref()
                .and_then(|l| l.usd)
                .map(|usd| (usd * 1000.0) as u64)
                .unwrap_or(0)
        })
}

/// Extract token price from the best pair
pub fn get_token_price(pairs: &[PairInfo], token_address: &str) -> Option<String> {
    pairs.iter()
        .filter(|p| p.base_token.address.eq_ignore_ascii_case(token_address))
        .max_by_key(|p| {
            p.liquidity
                .as_ref()
                .and_then(|l| l.usd)
                .map(|usd| (usd * 1000.0) as u64)
                .unwrap_or(0)
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
            "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263".to_string() // BONK token
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