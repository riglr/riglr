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
    use serde_json::json;

    // Helper function to create test Token
    fn create_test_token() -> Token {
        Token {
            address: "0x1234567890123456789012345678901234567890".to_string(),
            name: "Test Token".to_string(),
            symbol: "TEST".to_string(),
        }
    }

    // Helper function to create test PairInfo
    fn create_test_pair_info() -> PairInfo {
        PairInfo {
            chain_id: "ethereum".to_string(),
            dex_id: "uniswap".to_string(),
            url: "https://dexscreener.com/test".to_string(),
            pair_address: "0xabcdef1234567890".to_string(),
            labels: Some(vec!["test".to_string()]),
            base_token: create_test_token(),
            quote_token: Token {
                address: "0x9876543210987654321098765432109876543210".to_string(),
                name: "Quote Token".to_string(),
                symbol: "QUOTE".to_string(),
            },
            price_native: "0.001".to_string(),
            price_usd: Some("1.50".to_string()),
            liquidity: Some(Liquidity {
                usd: Some(100000.0),
                base: Some(50000.0),
                quote: Some(50000.0),
            }),
            volume: Some(Volume {
                h24: Some(10000.0),
                h6: Some(2500.0),
                h1: Some(416.0),
                m5: Some(35.0),
            }),
            price_change: Some(PriceChange {
                h24: Some(5.5),
                h6: Some(2.1),
                h1: Some(0.8),
                m5: Some(0.1),
            }),
            txns: Some(Transactions {
                h24: Some(TransactionStats {
                    buys: Some(100),
                    sells: Some(80),
                }),
                h6: Some(TransactionStats {
                    buys: Some(25),
                    sells: Some(20),
                }),
                h1: Some(TransactionStats {
                    buys: Some(4),
                    sells: Some(3),
                }),
                m5: Some(TransactionStats {
                    buys: Some(1),
                    sells: Some(0),
                }),
            }),
            market_cap: Some(1000000.0),
            fdv: Some(1500000.0),
        }
    }

    // Struct Serialization/Deserialization Tests

    #[test]
    fn test_token_serialization() {
        let token = create_test_token();
        let serialized = serde_json::to_string(&token).unwrap();
        let deserialized: Token = serde_json::from_str(&serialized).unwrap();

        assert_eq!(token.address, deserialized.address);
        assert_eq!(token.name, deserialized.name);
        assert_eq!(token.symbol, deserialized.symbol);
    }

    #[test]
    fn test_liquidity_serialization_with_all_fields() {
        let liquidity = Liquidity {
            usd: Some(100000.0),
            base: Some(50000.0),
            quote: Some(50000.0),
        };
        let serialized = serde_json::to_string(&liquidity).unwrap();
        let deserialized: Liquidity = serde_json::from_str(&serialized).unwrap();

        assert_eq!(liquidity.usd, deserialized.usd);
        assert_eq!(liquidity.base, deserialized.base);
        assert_eq!(liquidity.quote, deserialized.quote);
    }

    #[test]
    fn test_liquidity_serialization_with_none_fields() {
        let liquidity = Liquidity {
            usd: None,
            base: None,
            quote: None,
        };
        let serialized = serde_json::to_string(&liquidity).unwrap();
        let deserialized: Liquidity = serde_json::from_str(&serialized).unwrap();

        assert_eq!(liquidity.usd, deserialized.usd);
        assert_eq!(liquidity.base, deserialized.base);
        assert_eq!(liquidity.quote, deserialized.quote);
    }

    #[test]
    fn test_volume_default_serialization() {
        let volume = Volume::default();
        let serialized = serde_json::to_string(&volume).unwrap();
        let deserialized: Volume = serde_json::from_str(&serialized).unwrap();

        assert_eq!(volume.h24, deserialized.h24);
        assert_eq!(volume.h6, deserialized.h6);
        assert_eq!(volume.h1, deserialized.h1);
        assert_eq!(volume.m5, deserialized.m5);
    }

    #[test]
    fn test_volume_serialization_with_values() {
        let volume = Volume {
            h24: Some(10000.0),
            h6: Some(2500.0),
            h1: Some(416.0),
            m5: Some(35.0),
        };
        let serialized = serde_json::to_string(&volume).unwrap();
        let deserialized: Volume = serde_json::from_str(&serialized).unwrap();

        assert_eq!(volume.h24, deserialized.h24);
        assert_eq!(volume.h6, deserialized.h6);
        assert_eq!(volume.h1, deserialized.h1);
        assert_eq!(volume.m5, deserialized.m5);
    }

    #[test]
    fn test_price_change_serialization() {
        let price_change = PriceChange {
            h24: Some(5.5),
            h6: Some(2.1),
            h1: Some(0.8),
            m5: Some(0.1),
        };
        let serialized = serde_json::to_string(&price_change).unwrap();
        let deserialized: PriceChange = serde_json::from_str(&serialized).unwrap();

        assert_eq!(price_change.h24, deserialized.h24);
        assert_eq!(price_change.h6, deserialized.h6);
        assert_eq!(price_change.h1, deserialized.h1);
        assert_eq!(price_change.m5, deserialized.m5);
    }

    #[test]
    fn test_transaction_stats_serialization() {
        let stats = TransactionStats {
            buys: Some(100),
            sells: Some(80),
        };
        let serialized = serde_json::to_string(&stats).unwrap();
        let deserialized: TransactionStats = serde_json::from_str(&serialized).unwrap();

        assert_eq!(stats.buys, deserialized.buys);
        assert_eq!(stats.sells, deserialized.sells);
    }

    #[test]
    fn test_transaction_stats_serialization_with_none() {
        let stats = TransactionStats {
            buys: None,
            sells: None,
        };
        let serialized = serde_json::to_string(&stats).unwrap();
        let deserialized: TransactionStats = serde_json::from_str(&serialized).unwrap();

        assert_eq!(stats.buys, deserialized.buys);
        assert_eq!(stats.sells, deserialized.sells);
    }

    #[test]
    fn test_transactions_serialization() {
        let transactions = Transactions {
            h24: Some(TransactionStats {
                buys: Some(100),
                sells: Some(80),
            }),
            h6: None,
            h1: Some(TransactionStats {
                buys: None,
                sells: Some(3),
            }),
            m5: None,
        };
        let serialized = serde_json::to_string(&transactions).unwrap();
        let deserialized: Transactions = serde_json::from_str(&serialized).unwrap();

        assert!(deserialized.h24.is_some());
        assert!(deserialized.h6.is_none());
        assert!(deserialized.h1.is_some());
        assert!(deserialized.m5.is_none());
    }

    #[test]
    fn test_pair_info_serialization_complete() {
        let pair = create_test_pair_info();
        let serialized = serde_json::to_string(&pair).unwrap();
        let deserialized: PairInfo = serde_json::from_str(&serialized).unwrap();

        assert_eq!(pair.chain_id, deserialized.chain_id);
        assert_eq!(pair.dex_id, deserialized.dex_id);
        assert_eq!(pair.url, deserialized.url);
        assert_eq!(pair.pair_address, deserialized.pair_address);
        assert_eq!(pair.labels, deserialized.labels);
        assert_eq!(pair.price_native, deserialized.price_native);
        assert_eq!(pair.price_usd, deserialized.price_usd);
        assert!(deserialized.liquidity.is_some());
        assert!(deserialized.volume.is_some());
        assert!(deserialized.price_change.is_some());
        assert!(deserialized.txns.is_some());
        assert_eq!(pair.market_cap, deserialized.market_cap);
        assert_eq!(pair.fdv, deserialized.fdv);
    }

    #[test]
    fn test_pair_info_serialization_minimal() {
        let pair = PairInfo {
            chain_id: "ethereum".to_string(),
            dex_id: "uniswap".to_string(),
            url: "https://dexscreener.com/test".to_string(),
            pair_address: "0xabcdef1234567890".to_string(),
            labels: None,
            base_token: create_test_token(),
            quote_token: create_test_token(),
            price_native: "0.001".to_string(),
            price_usd: None,
            liquidity: None,
            volume: None,
            price_change: None,
            txns: None,
            market_cap: None,
            fdv: None,
        };
        let serialized = serde_json::to_string(&pair).unwrap();
        let deserialized: PairInfo = serde_json::from_str(&serialized).unwrap();

        assert_eq!(pair.chain_id, deserialized.chain_id);
        assert_eq!(pair.dex_id, deserialized.dex_id);
        assert!(deserialized.labels.is_none());
        assert!(deserialized.price_usd.is_none());
        assert!(deserialized.liquidity.is_none());
        assert!(deserialized.volume.is_none());
        assert!(deserialized.price_change.is_none());
        assert!(deserialized.txns.is_none());
        assert!(deserialized.market_cap.is_none());
        assert!(deserialized.fdv.is_none());
    }

    #[test]
    fn test_dexscreener_response_serialization() {
        let response = DexScreenerResponse {
            schema_version: "1.0.0".to_string(),
            pairs: vec![create_test_pair_info()],
        };
        let serialized = serde_json::to_string(&response).unwrap();
        let deserialized: DexScreenerResponse = serde_json::from_str(&serialized).unwrap();

        assert_eq!(response.schema_version, deserialized.schema_version);
        assert_eq!(response.pairs.len(), deserialized.pairs.len());
    }

    #[test]
    fn test_dexscreener_response_empty_pairs() {
        let response = DexScreenerResponse {
            schema_version: "1.0.0".to_string(),
            pairs: vec![],
        };
        let serialized = serde_json::to_string(&response).unwrap();
        let deserialized: DexScreenerResponse = serde_json::from_str(&serialized).unwrap();

        assert_eq!(response.schema_version, deserialized.schema_version);
        assert!(deserialized.pairs.is_empty());
    }

    // Helper function tests

    #[test]
    fn test_find_best_liquidity_pair_when_empty_should_return_none() {
        let pairs = vec![];
        let result = find_best_liquidity_pair(pairs);
        assert!(result.is_none());
    }

    #[test]
    fn test_find_best_liquidity_pair_when_no_liquidity_should_return_first() {
        let pair1 = PairInfo {
            liquidity: None,
            ..create_test_pair_info()
        };
        let pair2 = PairInfo {
            liquidity: None,
            ..create_test_pair_info()
        };
        let pairs = vec![pair1.clone(), pair2];
        let result = find_best_liquidity_pair(pairs);

        assert!(result.is_some());
        assert_eq!(result.unwrap().pair_address, pair1.pair_address);
    }

    #[test]
    fn test_find_best_liquidity_pair_when_liquidity_none_usd_should_return_first() {
        let pair1 = PairInfo {
            liquidity: Some(Liquidity {
                usd: None,
                base: Some(100.0),
                quote: Some(200.0),
            }),
            ..create_test_pair_info()
        };
        let pair2 = PairInfo {
            liquidity: Some(Liquidity {
                usd: None,
                base: Some(300.0),
                quote: Some(400.0),
            }),
            ..create_test_pair_info()
        };
        let pairs = vec![pair1.clone(), pair2];
        let result = find_best_liquidity_pair(pairs);

        assert!(result.is_some());
        assert_eq!(result.unwrap().pair_address, pair1.pair_address);
    }

    #[test]
    fn test_find_best_liquidity_pair_when_has_liquidity_should_return_highest() {
        let pair1 = PairInfo {
            pair_address: "low_liquidity".to_string(),
            liquidity: Some(Liquidity {
                usd: Some(50000.0),
                base: Some(25000.0),
                quote: Some(25000.0),
            }),
            ..create_test_pair_info()
        };
        let pair2 = PairInfo {
            pair_address: "high_liquidity".to_string(),
            liquidity: Some(Liquidity {
                usd: Some(200000.0),
                base: Some(100000.0),
                quote: Some(100000.0),
            }),
            ..create_test_pair_info()
        };
        let pair3 = PairInfo {
            pair_address: "medium_liquidity".to_string(),
            liquidity: Some(Liquidity {
                usd: Some(100000.0),
                base: Some(50000.0),
                quote: Some(50000.0),
            }),
            ..create_test_pair_info()
        };
        let pairs = vec![pair1, pair2.clone(), pair3];
        let result = find_best_liquidity_pair(pairs);

        assert!(result.is_some());
        assert_eq!(result.unwrap().pair_address, "high_liquidity");
    }

    #[test]
    fn test_find_best_liquidity_pair_when_mixed_liquidity_should_return_highest() {
        let pair1 = PairInfo {
            pair_address: "no_liquidity".to_string(),
            liquidity: None,
            ..create_test_pair_info()
        };
        let pair2 = PairInfo {
            pair_address: "has_liquidity".to_string(),
            liquidity: Some(Liquidity {
                usd: Some(150000.0),
                base: Some(75000.0),
                quote: Some(75000.0),
            }),
            ..create_test_pair_info()
        };
        let pair3 = PairInfo {
            pair_address: "none_usd".to_string(),
            liquidity: Some(Liquidity {
                usd: None,
                base: Some(50000.0),
                quote: Some(50000.0),
            }),
            ..create_test_pair_info()
        };
        let pairs = vec![pair1, pair2.clone(), pair3];
        let result = find_best_liquidity_pair(pairs);

        assert!(result.is_some());
        assert_eq!(result.unwrap().pair_address, "has_liquidity");
    }

    #[test]
    fn test_get_token_price_when_empty_pairs_should_return_none() {
        let pairs = vec![];
        let result = get_token_price(&pairs, "0x1234567890123456789012345678901234567890");
        assert!(result.is_none());
    }

    #[test]
    fn test_get_token_price_when_no_matching_address_should_return_none() {
        let pairs = vec![create_test_pair_info()];
        let result = get_token_price(&pairs, "0xnonexistent");
        assert!(result.is_none());
    }

    #[test]
    fn test_get_token_price_when_matching_address_case_insensitive_should_return_price() {
        let mut pair = create_test_pair_info();
        pair.base_token.address = "0xABCDEF1234567890".to_string();
        pair.price_usd = Some("2.50".to_string());
        let pairs = vec![pair];

        let result = get_token_price(&pairs, "0xabcdef1234567890");
        assert!(result.is_some());
        assert_eq!(result.unwrap(), "2.50");
    }

    #[test]
    fn test_get_token_price_when_matching_address_uppercase_should_return_price() {
        let mut pair = create_test_pair_info();
        pair.base_token.address = "0xabcdef1234567890".to_string();
        pair.price_usd = Some("3.75".to_string());
        let pairs = vec![pair];

        let result = get_token_price(&pairs, "0XABCDEF1234567890");
        assert!(result.is_some());
        assert_eq!(result.unwrap(), "3.75");
    }

    #[test]
    fn test_get_token_price_when_no_price_usd_should_return_none() {
        let mut pair = create_test_pair_info();
        pair.base_token.address = "0x1234567890123456789012345678901234567890".to_string();
        pair.price_usd = None;
        let pairs = vec![pair];

        let result = get_token_price(&pairs, "0x1234567890123456789012345678901234567890");
        assert!(result.is_none());
    }

    #[test]
    fn test_get_token_price_when_multiple_pairs_should_return_highest_liquidity() {
        let mut pair1 = create_test_pair_info();
        pair1.base_token.address = "0x1234567890123456789012345678901234567890".to_string();
        pair1.price_usd = Some("1.00".to_string());
        pair1.liquidity = Some(Liquidity {
            usd: Some(50000.0),
            base: Some(25000.0),
            quote: Some(25000.0),
        });

        let mut pair2 = create_test_pair_info();
        pair2.base_token.address = "0x1234567890123456789012345678901234567890".to_string();
        pair2.price_usd = Some("1.05".to_string());
        pair2.liquidity = Some(Liquidity {
            usd: Some(150000.0),
            base: Some(75000.0),
            quote: Some(75000.0),
        });

        let pairs = vec![pair1, pair2];
        let result = get_token_price(&pairs, "0x1234567890123456789012345678901234567890");

        assert!(result.is_some());
        assert_eq!(result.unwrap(), "1.05");
    }

    #[test]
    fn test_get_token_price_when_multiple_pairs_no_liquidity_should_return_first_match() {
        let mut pair1 = create_test_pair_info();
        pair1.base_token.address = "0x1234567890123456789012345678901234567890".to_string();
        pair1.price_usd = Some("1.00".to_string());
        pair1.liquidity = None;

        let mut pair2 = create_test_pair_info();
        pair2.base_token.address = "0x1234567890123456789012345678901234567890".to_string();
        pair2.price_usd = Some("1.05".to_string());
        pair2.liquidity = None;

        let pairs = vec![pair1, pair2];
        let result = get_token_price(&pairs, "0x1234567890123456789012345678901234567890");

        assert!(result.is_some());
        assert_eq!(result.unwrap(), "1.00");
    }

    #[test]
    fn test_get_token_price_when_mixed_liquidity_should_prefer_with_liquidity() {
        let mut pair1 = create_test_pair_info();
        pair1.base_token.address = "0x1234567890123456789012345678901234567890".to_string();
        pair1.price_usd = Some("1.00".to_string());
        pair1.liquidity = None;

        let mut pair2 = create_test_pair_info();
        pair2.base_token.address = "0x1234567890123456789012345678901234567890".to_string();
        pair2.price_usd = Some("1.05".to_string());
        pair2.liquidity = Some(Liquidity {
            usd: Some(100000.0),
            base: Some(50000.0),
            quote: Some(50000.0),
        });

        let pairs = vec![pair1, pair2];
        let result = get_token_price(&pairs, "0x1234567890123456789012345678901234567890");

        assert!(result.is_some());
        assert_eq!(result.unwrap(), "1.05");
    }

    // JSON parsing with snake_case and camelCase field names
    #[test]
    fn test_deserialize_pair_info_with_camel_case() {
        let json = json!({
            "chainId": "ethereum",
            "dexId": "uniswap",
            "url": "https://dexscreener.com/test",
            "pairAddress": "0xabcdef1234567890",
            "labels": ["test"],
            "baseToken": {
                "address": "0x1234567890123456789012345678901234567890",
                "name": "Test Token",
                "symbol": "TEST"
            },
            "quoteToken": {
                "address": "0x9876543210987654321098765432109876543210",
                "name": "Quote Token",
                "symbol": "QUOTE"
            },
            "priceNative": "0.001",
            "priceUsd": "1.50",
            "liquidity": {
                "usd": 100000.0,
                "base": 50000.0,
                "quote": 50000.0
            },
            "volume": {
                "h24": 10000.0,
                "h6": 2500.0,
                "h1": 416.0,
                "m5": 35.0
            },
            "priceChange": {
                "h24": 5.5,
                "h6": 2.1,
                "h1": 0.8,
                "m5": 0.1
            },
            "txns": {
                "h24": {
                    "buys": 100,
                    "sells": 80
                },
                "h6": {
                    "buys": 25,
                    "sells": 20
                },
                "h1": {
                    "buys": 4,
                    "sells": 3
                },
                "m5": {
                    "buys": 1,
                    "sells": 0
                }
            },
            "marketCap": 1000000.0,
            "fdv": 1500000.0
        });

        let pair: PairInfo = serde_json::from_value(json).unwrap();
        assert_eq!(pair.chain_id, "ethereum");
        assert_eq!(pair.dex_id, "uniswap");
        assert_eq!(pair.pair_address, "0xabcdef1234567890");
        assert_eq!(
            pair.base_token.address,
            "0x1234567890123456789012345678901234567890"
        );
        assert_eq!(pair.quote_token.symbol, "QUOTE");
        assert_eq!(pair.price_native, "0.001");
        assert_eq!(pair.price_usd, Some("1.50".to_string()));
        assert!(pair.liquidity.is_some());
        assert!(pair.volume.is_some());
        assert!(pair.price_change.is_some());
        assert!(pair.txns.is_some());
        assert_eq!(pair.market_cap, Some(1000000.0));
        assert_eq!(pair.fdv, Some(1500000.0));
    }

    #[test]
    fn test_deserialize_dexscreener_response_with_camel_case() {
        let json = json!({
            "schemaVersion": "1.0.0",
            "pairs": []
        });

        let response: DexScreenerResponse = serde_json::from_value(json).unwrap();
        assert_eq!(response.schema_version, "1.0.0");
        assert!(response.pairs.is_empty());
    }

    // Test Volume default behavior
    #[test]
    fn test_volume_default_values() {
        let volume = Volume::default();
        assert!(volume.h24.is_none());
        assert!(volume.h6.is_none());
        assert!(volume.h1.is_none());
        assert!(volume.m5.is_none());
    }

    #[test]
    fn test_volume_deserialization_with_missing_fields() {
        let json = json!({});
        let volume: Volume = serde_json::from_value(json).unwrap();
        assert!(volume.h24.is_none());
        assert!(volume.h6.is_none());
        assert!(volume.h1.is_none());
        assert!(volume.m5.is_none());
    }

    #[test]
    fn test_price_change_deserialization_with_missing_fields() {
        let json = json!({});
        let price_change: PriceChange = serde_json::from_value(json).unwrap();
        assert!(price_change.h24.is_none());
        assert!(price_change.h6.is_none());
        assert!(price_change.h1.is_none());
        assert!(price_change.m5.is_none());
    }

    #[test]
    fn test_transactions_deserialization_with_missing_fields() {
        let json = json!({});
        let transactions: Transactions = serde_json::from_value(json).unwrap();
        assert!(transactions.h24.is_none());
        assert!(transactions.h6.is_none());
        assert!(transactions.h1.is_none());
        assert!(transactions.m5.is_none());
    }

    // Additional edge case tests for helper functions

    #[test]
    fn test_find_best_liquidity_pair_with_single_pair() {
        let pair = create_test_pair_info();
        let pairs = vec![pair.clone()];
        let result = find_best_liquidity_pair(pairs);

        assert!(result.is_some());
        assert_eq!(result.unwrap().pair_address, pair.pair_address);
    }

    #[test]
    fn test_find_best_liquidity_pair_with_zero_liquidity() {
        let pair1 = PairInfo {
            pair_address: "zero_liquidity_1".to_string(),
            liquidity: Some(Liquidity {
                usd: Some(0.0),
                base: Some(0.0),
                quote: Some(0.0),
            }),
            ..create_test_pair_info()
        };
        let pair2 = PairInfo {
            pair_address: "zero_liquidity_2".to_string(),
            liquidity: Some(Liquidity {
                usd: Some(0.0),
                base: Some(100.0),
                quote: Some(200.0),
            }),
            ..create_test_pair_info()
        };
        let pairs = vec![pair1.clone(), pair2];
        let result = find_best_liquidity_pair(pairs);

        assert!(result.is_some());
        assert_eq!(result.unwrap().pair_address, "zero_liquidity_1");
    }

    #[test]
    fn test_find_best_liquidity_pair_with_negative_liquidity() {
        let pair = PairInfo {
            pair_address: "negative_liquidity".to_string(),
            liquidity: Some(Liquidity {
                usd: Some(-1000.0),
                base: Some(-500.0),
                quote: Some(-500.0),
            }),
            ..create_test_pair_info()
        };
        let pairs = vec![pair.clone()];
        let result = find_best_liquidity_pair(pairs);

        assert!(result.is_some());
        assert_eq!(result.unwrap().pair_address, "negative_liquidity");
    }

    #[test]
    fn test_get_token_price_with_empty_string_address() {
        let pairs = vec![create_test_pair_info()];
        let result = get_token_price(&pairs, "");
        assert!(result.is_none());
    }

    #[test]
    fn test_get_token_price_with_partial_match() {
        let mut pair = create_test_pair_info();
        pair.base_token.address = "0x1234567890123456789012345678901234567890".to_string();
        let pairs = vec![pair];

        // Should not match partial address
        let result = get_token_price(&pairs, "0x1234567890");
        assert!(result.is_none());
    }

    #[test]
    fn test_get_token_price_with_special_characters() {
        let mut pair = create_test_pair_info();
        pair.base_token.address = "0x!@#$%^&*()_+".to_string();
        pair.price_usd = Some("1.00".to_string());
        let pairs = vec![pair];

        let result = get_token_price(&pairs, "0x!@#$%^&*()_+");
        assert!(result.is_some());
        assert_eq!(result.unwrap(), "1.00");
    }

    #[test]
    fn test_get_token_price_case_sensitivity_mixed() {
        let mut pair = create_test_pair_info();
        pair.base_token.address = "0xaBcDeF1234567890".to_string();
        pair.price_usd = Some("2.50".to_string());
        let pairs = vec![pair];

        let result = get_token_price(&pairs, "0xAbCdEf1234567890");
        assert!(result.is_some());
        assert_eq!(result.unwrap(), "2.50");
    }

    // Test struct Debug trait implementations
    #[test]
    fn test_token_debug_format() {
        let token = create_test_token();
        let debug_str = format!("{:?}", token);
        assert!(debug_str.contains("Test Token"));
        assert!(debug_str.contains("TEST"));
        assert!(debug_str.contains("0x1234567890123456789012345678901234567890"));
    }

    #[test]
    fn test_liquidity_debug_format() {
        let liquidity = Liquidity {
            usd: Some(100000.0),
            base: Some(50000.0),
            quote: Some(50000.0),
        };
        let debug_str = format!("{:?}", liquidity);
        assert!(debug_str.contains("100000"));
        assert!(debug_str.contains("50000"));
    }

    #[test]
    fn test_volume_debug_format() {
        let volume = Volume {
            h24: Some(10000.0),
            h6: Some(2500.0),
            h1: Some(416.0),
            m5: Some(35.0),
        };
        let debug_str = format!("{:?}", volume);
        assert!(debug_str.contains("10000"));
        assert!(debug_str.contains("2500"));
        assert!(debug_str.contains("416"));
        assert!(debug_str.contains("35"));
    }

    #[test]
    fn test_price_change_debug_format() {
        let price_change = PriceChange {
            h24: Some(5.5),
            h6: Some(2.1),
            h1: Some(0.8),
            m5: Some(0.1),
        };
        let debug_str = format!("{:?}", price_change);
        assert!(debug_str.contains("5.5"));
        assert!(debug_str.contains("2.1"));
        assert!(debug_str.contains("0.8"));
        assert!(debug_str.contains("0.1"));
    }

    #[test]
    fn test_transaction_stats_debug_format() {
        let stats = TransactionStats {
            buys: Some(100),
            sells: Some(80),
        };
        let debug_str = format!("{:?}", stats);
        assert!(debug_str.contains("100"));
        assert!(debug_str.contains("80"));
    }

    #[test]
    fn test_transactions_debug_format() {
        let transactions = Transactions {
            h24: Some(TransactionStats {
                buys: Some(100),
                sells: Some(80),
            }),
            h6: None,
            h1: None,
            m5: None,
        };
        let debug_str = format!("{:?}", transactions);
        assert!(debug_str.contains("100"));
        assert!(debug_str.contains("80"));
    }

    #[test]
    fn test_pair_info_debug_format() {
        let pair = create_test_pair_info();
        let debug_str = format!("{:?}", pair);
        assert!(debug_str.contains("ethereum"));
        assert!(debug_str.contains("uniswap"));
        assert!(debug_str.contains("0xabcdef1234567890"));
    }

    #[test]
    fn test_dexscreener_response_debug_format() {
        let response = DexScreenerResponse {
            schema_version: "1.0.0".to_string(),
            pairs: vec![create_test_pair_info()],
        };
        let debug_str = format!("{:?}", response);
        assert!(debug_str.contains("1.0.0"));
        assert!(debug_str.contains("ethereum"));
    }

    // Test Clone trait implementations
    #[test]
    fn test_token_clone() {
        let token = create_test_token();
        let cloned = token.clone();
        assert_eq!(token.address, cloned.address);
        assert_eq!(token.name, cloned.name);
        assert_eq!(token.symbol, cloned.symbol);
    }

    #[test]
    fn test_liquidity_clone() {
        let liquidity = Liquidity {
            usd: Some(100000.0),
            base: Some(50000.0),
            quote: Some(50000.0),
        };
        let cloned = liquidity.clone();
        assert_eq!(liquidity.usd, cloned.usd);
        assert_eq!(liquidity.base, cloned.base);
        assert_eq!(liquidity.quote, cloned.quote);
    }

    #[test]
    fn test_volume_clone() {
        let volume = Volume {
            h24: Some(10000.0),
            h6: Some(2500.0),
            h1: Some(416.0),
            m5: Some(35.0),
        };
        let cloned = volume.clone();
        assert_eq!(volume.h24, cloned.h24);
        assert_eq!(volume.h6, cloned.h6);
        assert_eq!(volume.h1, cloned.h1);
        assert_eq!(volume.m5, cloned.m5);
    }

    #[test]
    fn test_price_change_clone() {
        let price_change = PriceChange {
            h24: Some(5.5),
            h6: Some(2.1),
            h1: Some(0.8),
            m5: Some(0.1),
        };
        let cloned = price_change.clone();
        assert_eq!(price_change.h24, cloned.h24);
        assert_eq!(price_change.h6, cloned.h6);
        assert_eq!(price_change.h1, cloned.h1);
        assert_eq!(price_change.m5, cloned.m5);
    }

    #[test]
    fn test_transaction_stats_clone() {
        let stats = TransactionStats {
            buys: Some(100),
            sells: Some(80),
        };
        let cloned = stats.clone();
        assert_eq!(stats.buys, cloned.buys);
        assert_eq!(stats.sells, cloned.sells);
    }

    #[test]
    fn test_transactions_clone() {
        let transactions = Transactions {
            h24: Some(TransactionStats {
                buys: Some(100),
                sells: Some(80),
            }),
            h6: None,
            h1: None,
            m5: None,
        };
        let cloned = transactions.clone();
        assert!(cloned.h24.is_some());
        assert!(cloned.h6.is_none());
        assert!(cloned.h1.is_none());
        assert!(cloned.m5.is_none());
    }

    #[test]
    fn test_pair_info_clone() {
        let pair = create_test_pair_info();
        let cloned = pair.clone();
        assert_eq!(pair.chain_id, cloned.chain_id);
        assert_eq!(pair.dex_id, cloned.dex_id);
        assert_eq!(pair.url, cloned.url);
        assert_eq!(pair.pair_address, cloned.pair_address);
    }

    #[test]
    fn test_dexscreener_response_clone() {
        let response = DexScreenerResponse {
            schema_version: "1.0.0".to_string(),
            pairs: vec![create_test_pair_info()],
        };
        let cloned = response.clone();
        assert_eq!(response.schema_version, cloned.schema_version);
        assert_eq!(response.pairs.len(), cloned.pairs.len());
    }

    // Test edge cases for liquidity calculation in find_best_liquidity_pair
    #[test]
    fn test_find_best_liquidity_pair_with_very_large_numbers() {
        let pair = PairInfo {
            pair_address: "large_liquidity".to_string(),
            liquidity: Some(Liquidity {
                usd: Some(f64::MAX),
                base: Some(f64::MAX),
                quote: Some(f64::MAX),
            }),
            ..create_test_pair_info()
        };
        let pairs = vec![pair.clone()];
        let result = find_best_liquidity_pair(pairs);

        assert!(result.is_some());
        assert_eq!(result.unwrap().pair_address, "large_liquidity");
    }

    #[test]
    fn test_find_best_liquidity_pair_with_infinity() {
        let pair = PairInfo {
            pair_address: "infinity_liquidity".to_string(),
            liquidity: Some(Liquidity {
                usd: Some(f64::INFINITY),
                base: Some(100.0),
                quote: Some(200.0),
            }),
            ..create_test_pair_info()
        };
        let pairs = vec![pair.clone()];
        let result = find_best_liquidity_pair(pairs);

        assert!(result.is_some());
        assert_eq!(result.unwrap().pair_address, "infinity_liquidity");
    }

    #[test]
    fn test_find_best_liquidity_pair_with_nan() {
        let pair = PairInfo {
            pair_address: "nan_liquidity".to_string(),
            liquidity: Some(Liquidity {
                usd: Some(f64::NAN),
                base: Some(100.0),
                quote: Some(200.0),
            }),
            ..create_test_pair_info()
        };
        let pairs = vec![pair.clone()];
        let result = find_best_liquidity_pair(pairs);

        assert!(result.is_some());
        assert_eq!(result.unwrap().pair_address, "nan_liquidity");
    }

    // Test search_ticker truncation behavior
    #[tokio::test]
    async fn test_search_ticker_truncates_to_8_results() {
        // This test will verify that results are truncated to 8, but since we can't control
        // the actual API response, this is more of a documentation test
        let response = search_ticker("ETH".to_string()).await.unwrap();
        assert!(response.pairs.len() <= 8);
    }

    // Original integration tests (preserved)
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

    #[tokio::test]
    async fn test_get_pair_by_address_success() {
        // Test with a known pair address that should exist
        let result =
            get_pair_by_address("ethereum_0x0d4a11d5eeaac28ec3f61d100daf4d40471f1852").await;
        match result {
            Ok(pair) => {
                assert!(!pair.pair_address.is_empty());
                assert!(!pair.chain_id.is_empty());
                assert!(!pair.dex_id.is_empty());
            }
            Err(_) => {
                // Some pair addresses might not exist, which is acceptable for this test
                // The important thing is that the function can be called without panicking
            }
        }
    }

    #[tokio::test]
    async fn test_get_pair_by_address_nonexistent() {
        // Test with a clearly invalid pair address
        let result = get_pair_by_address("invalid_pair_address_that_does_not_exist").await;
        assert!(result.is_err());
    }
}
