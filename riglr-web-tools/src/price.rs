//! Price fetching tools using DexScreener API
//!
//! This module provides specialized tools for fetching token prices using real DexScreener
//! integration. It focuses on finding the most reliable price data by selecting pairs
//! with highest liquidity for accuracy.

use crate::client::WebClient;
use futures::future;
use riglr_core::ToolError;
use riglr_macros::tool;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

/// Response from DexScreener API
#[derive(Debug, Deserialize)]
struct DexScreenerResponse {
    pairs: Option<Vec<PairInfo>>,
}

/// Pair information from DexScreener
#[derive(Debug, Deserialize)]
struct PairInfo {
    #[serde(rename = "priceUsd")]
    price_usd: Option<String>,
    liquidity: Option<LiquidityInfo>,
    #[serde(rename = "baseToken")]
    base_token: TokenInfo,
    #[serde(rename = "dexId")]
    dex_id: String,
    #[serde(rename = "pairAddress")]
    pair_address: String,
}

/// Liquidity information
#[derive(Debug, Deserialize)]
struct LiquidityInfo {
    usd: Option<f64>,
}

/// Token information
#[derive(Debug, Deserialize)]
struct TokenInfo {
    _address: String,
    symbol: String,
}

/// Price result with additional metadata
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TokenPriceResult {
    /// Token address that was queried
    pub token_address: String,
    /// Token symbol
    pub token_symbol: Option<String>,
    /// Current price in USD
    pub price_usd: String,
    /// DEX where price was sourced
    pub source_dex: Option<String>,
    /// Pair address used for pricing
    pub source_pair: Option<String>,
    /// Liquidity in USD of the source pair
    pub source_liquidity_usd: Option<f64>,
    /// Chain name where token exists
    pub chain: Option<String>,
    /// Timestamp of price fetch
    pub fetched_at: chrono::DateTime<chrono::Utc>,
}

/// Get token price from DexScreener with highest liquidity pair
///
/// This tool fetches the most reliable token price by finding the trading pair
/// with the highest liquidity on DexScreener. Using the highest liquidity pair
/// ensures the most accurate and stable price data.
///
/// # Arguments
///
/// * `token_address` - Token contract address to get price for
/// * `chain` - Optional chain name (e.g., "ethereum", "bsc", "polygon", "solana")
///
/// # Returns
///
/// Returns `TokenPriceResult` containing:
/// - `token_address`: The queried token address
/// - `token_symbol`: Token symbol if available
/// - `price_usd`: Current price in USD as string
/// - `source_dex`: DEX where price was sourced from
/// - `source_pair`: Trading pair address used
/// - `source_liquidity_usd`: Liquidity of the source pair
/// - `chain`: Chain name
/// - `fetched_at`: Timestamp when price was fetched
///
/// # Errors
///
/// * `ToolError::InvalidInput` - When token address format is invalid
/// * `ToolError::Retriable` - When DexScreener API request fails
/// * `ToolError::Permanent` - When no trading pairs found for token
///
/// # Examples
///
/// ```rust,ignore
/// use riglr_web_tools::price::get_token_price;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Get USDC price on Ethereum
/// let price = get_token_price(
///     "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string(),
///     Some("ethereum".to_string()),
/// ).await?;
///
/// println!("USDC price: ${}", price.price_usd);
/// println!("Source: {} (${:.2} liquidity)",
///          price.source_dex.unwrap_or_default(),
///          price.source_liquidity_usd.unwrap_or(0.0));
/// # Ok(())
/// # }
/// ```
#[tool]
pub async fn get_token_price(
    _context: &riglr_core::provider::ApplicationContext,
    token_address: String,
    chain: Option<String>,
) -> Result<TokenPriceResult, ToolError> {
    debug!(
        "Getting token price for address: {} on chain: {:?}",
        token_address, chain
    );

    // Validate token address format
    if token_address.is_empty() {
        return Err(ToolError::invalid_input_string(
            "Token address cannot be empty",
        ));
    }

    // Build query string
    let query = if let Some(chain_name) = &chain {
        format!("{}:{}", chain_name, token_address)
    } else {
        token_address.clone()
    };

    let url = format!("https://api.dexscreener.com/latest/dex/search/?q={}", query);

    debug!("Fetching price data from: {}", url);

    // Use WebClient for HTTP request with retry logic
    let client = WebClient::default();

    let response_text = client
        .get(&url)
        .await
        .map_err(|e| ToolError::retriable_string(format!("DexScreener request failed: {}", e)))?;

    let data: DexScreenerResponse = serde_json::from_str(&response_text)
        .map_err(|e| ToolError::retriable_string(format!("Failed to parse response: {}", e)))?;

    // Find pair with highest liquidity for most reliable price
    let best_pair = data
        .pairs
        .and_then(|pairs| {
            if pairs.is_empty() {
                None
            } else {
                pairs
                    .into_iter()
                    .filter(|pair| pair.price_usd.is_some()) // Only consider pairs with price data
                    .max_by(|a, b| {
                        let liquidity_a = a.liquidity.as_ref().and_then(|l| l.usd).unwrap_or(0.0);
                        let liquidity_b = b.liquidity.as_ref().and_then(|l| l.usd).unwrap_or(0.0);
                        liquidity_a
                            .partial_cmp(&liquidity_b)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    })
            }
        })
        .ok_or_else(|| ToolError::permanent_string("No trading pairs found for token"))?;

    let price = best_pair
        .price_usd
        .ok_or_else(|| ToolError::permanent_string("No price data available"))?;

    let result = TokenPriceResult {
        token_address: token_address.clone(),
        token_symbol: Some(best_pair.base_token.symbol),
        price_usd: price,
        source_dex: Some(best_pair.dex_id),
        source_pair: Some(best_pair.pair_address),
        source_liquidity_usd: best_pair.liquidity.and_then(|l| l.usd),
        chain: chain.clone(),
        fetched_at: chrono::Utc::now(),
    };

    info!(
        "Found price for {} ({}): ${} from {} DEX with ${:.2} liquidity",
        token_address,
        result
            .token_symbol
            .as_ref()
            .unwrap_or(&"Unknown".to_string()),
        result.price_usd,
        result.source_dex.as_ref().unwrap_or(&"Unknown".to_string()),
        result.source_liquidity_usd.unwrap_or(0.0)
    );

    Ok(result)
}

/// Get multiple token prices in a batch request
///
/// This tool fetches prices for multiple tokens efficiently by making multiple
/// requests concurrently. Useful for portfolio tracking or multi-token analysis.
///
/// # Arguments
///
/// * `token_addresses` - List of token addresses to get prices for
/// * `chain` - Optional chain name to apply to all tokens
///
/// # Returns
///
/// Returns `Vec<TokenPriceResult>` with prices for all found tokens.
/// Tokens without available price data are omitted from results.
///
/// # Errors
///
/// * `ToolError::InvalidInput` - When token addresses list is empty
/// * `ToolError::Retriable` - When API requests fail
///
/// # Examples
///
/// ```rust,ignore
/// use riglr_web_tools::price::get_token_prices_batch;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let tokens = vec![
///     "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string(), // USDC
///     "0xdAC17F958D2ee523a2206206994597C13D831ec7".to_string(), // USDT
/// ];
///
/// let prices = get_token_prices_batch(tokens, Some("ethereum".to_string())).await?;
///
/// for price in prices {
///     println!("{}: ${}", price.token_symbol.unwrap_or_default(), price.price_usd);
/// }
/// # Ok(())
/// # }
/// ```
#[tool]
pub async fn get_token_prices_batch(
    context: &riglr_core::provider::ApplicationContext,
    token_addresses: Vec<String>,
    chain: Option<String>,
) -> Result<Vec<TokenPriceResult>, ToolError> {
    if token_addresses.is_empty() {
        return Err(ToolError::invalid_input_string(
            "Token addresses list cannot be empty",
        ));
    }

    debug!("Getting batch prices for {} tokens", token_addresses.len());

    // Create futures for concurrent requests
    let futures: Vec<_> = token_addresses
        .into_iter()
        .map(|addr| get_token_price(context, addr, chain.clone()))
        .collect();

    // Execute all requests concurrently
    let results = future::join_all(futures).await;

    // Collect successful results
    let mut prices = Vec::new();
    for (i, result) in results.into_iter().enumerate() {
        match result {
            Ok(price) => prices.push(price),
            Err(e) => {
                warn!("Failed to get price for token {}: {}", i, e);
                // Continue with other tokens
            }
        }
    }

    info!("Successfully retrieved {} token prices", prices.len());
    Ok(prices)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_context() -> riglr_core::provider::ApplicationContext {
        // Create a minimal config with bridging disabled to avoid LIFI_API_KEY requirement
        let mut features = riglr_core::FeaturesConfig::default();
        features.enable_bridging = false; // Disable bridging to avoid requiring LIFI_API_KEY

        let config = riglr_config::ConfigBuilder::new()
            .features(features)
            .build()
            .expect("Test config should be valid");

        riglr_core::provider::ApplicationContext::from_config(&config)
    }

    #[test]
    fn test_token_price_result_creation() {
        let result = TokenPriceResult {
            token_address: "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string(),
            token_symbol: Some("USDC".to_string()),
            price_usd: "1.0000".to_string(),
            source_dex: Some("uniswap_v2".to_string()),
            source_pair: Some("0xB4e16d0168e52d35CaCD2c6185b44281Ec28C9Dc".to_string()),
            source_liquidity_usd: Some(10000000.0),
            chain: Some("ethereum".to_string()),
            fetched_at: chrono::Utc::now(),
        };

        assert_eq!(result.token_symbol, Some("USDC".to_string()));
        assert_eq!(result.price_usd, "1.0000");
        assert!(result.source_liquidity_usd.unwrap() > 0.0);
    }

    #[tokio::test]
    async fn test_empty_token_address_validation() {
        let context = create_test_context();
        let result = get_token_price(&context, "".to_string(), None).await;
        assert!(result.is_err());
        assert!(matches!(result, Err(ToolError::InvalidInput { .. })));
    }

    #[tokio::test]
    async fn test_batch_empty_addresses() {
        let context = create_test_context();
        let result = get_token_prices_batch(&context, vec![], None).await;
        assert!(result.is_err());
        assert!(matches!(result, Err(ToolError::InvalidInput { .. })));
    }

    #[tokio::test]
    async fn test_get_token_price_with_chain() {
        // This will test the query building with chain
        let context = create_test_context();
        let result =
            get_token_price(&context, "0x123".to_string(), Some("ethereum".to_string())).await;
        // This will likely fail due to no mock, but tests the path with chain
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_token_price_without_chain() {
        // This will test the query building without chain
        let context = create_test_context();
        let result = get_token_price(&context, "0x123".to_string(), None).await;
        // This will likely fail due to no mock, but tests the path without chain
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_batch_with_single_address() {
        let context = create_test_context();
        let addresses = vec!["0x123".to_string()];
        let result = get_token_prices_batch(&context, addresses, None).await;
        // Will test the batch functionality with one address
        assert!(result.is_ok()); // Should return empty vec due to failed individual requests
        assert_eq!(result.unwrap().len(), 0);
    }

    #[tokio::test]
    async fn test_batch_with_multiple_addresses() {
        let context = create_test_context();
        let addresses = vec![
            "0x123".to_string(),
            "0x456".to_string(),
            "0x789".to_string(),
        ];
        let result =
            get_token_prices_batch(&context, addresses, Some("ethereum".to_string())).await;
        // Will test the batch functionality with multiple addresses
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0); // All will fail but function succeeds
    }

    #[test]
    fn test_dexscreener_response_deserialization_empty_pairs() {
        let json = r#"{"pairs": []}"#;
        let response: DexScreenerResponse = serde_json::from_str(json).unwrap();
        assert!(response.pairs.is_some());
        assert!(response.pairs.unwrap().is_empty());
    }

    #[test]
    fn test_dexscreener_response_deserialization_no_pairs() {
        let json = r#"{"pairs": null}"#;
        let response: DexScreenerResponse = serde_json::from_str(json).unwrap();
        assert!(response.pairs.is_none());
    }

    #[test]
    fn test_pair_info_deserialization_complete() {
        let json = r#"{
            "priceUsd": "1.0000",
            "liquidity": {"usd": 10000.0},
            "baseToken": {"address": "0x123", "symbol": "TEST"},
            "dexId": "uniswap_v2",
            "pairAddress": "0x456"
        }"#;
        let pair: PairInfo = serde_json::from_str(json).unwrap();
        assert_eq!(pair.price_usd, Some("1.0000".to_string()));
        assert_eq!(pair.liquidity.unwrap().usd, Some(10000.0));
        assert_eq!(pair.base_token.symbol, "TEST");
        assert_eq!(pair.dex_id, "uniswap_v2");
        assert_eq!(pair.pair_address, "0x456");
    }

    #[test]
    fn test_pair_info_deserialization_minimal() {
        let json = r#"{
            "priceUsd": null,
            "liquidity": null,
            "baseToken": {"address": "0x123", "symbol": "TEST"},
            "dexId": "uniswap_v2",
            "pairAddress": "0x456"
        }"#;
        let pair: PairInfo = serde_json::from_str(json).unwrap();
        assert_eq!(pair.price_usd, None);
        assert!(pair.liquidity.is_none());
        assert_eq!(pair.base_token.symbol, "TEST");
    }

    #[test]
    fn test_liquidity_info_deserialization_with_usd() {
        let json = r#"{"usd": 50000.0}"#;
        let liquidity: LiquidityInfo = serde_json::from_str(json).unwrap();
        assert_eq!(liquidity.usd, Some(50000.0));
    }

    #[test]
    fn test_liquidity_info_deserialization_without_usd() {
        let json = r#"{"usd": null}"#;
        let liquidity: LiquidityInfo = serde_json::from_str(json).unwrap();
        assert_eq!(liquidity.usd, None);
    }

    #[test]
    fn test_token_info_deserialization() {
        let json = r#"{"address": "0x123", "symbol": "BTC"}"#;
        let token: TokenInfo = serde_json::from_str(json).unwrap();
        assert_eq!(token._address, "0x123");
        assert_eq!(token.symbol, "BTC");
    }

    #[test]
    fn test_token_price_result_serialization() {
        let result = TokenPriceResult {
            token_address: "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string(),
            token_symbol: Some("USDC".to_string()),
            price_usd: "1.0000".to_string(),
            source_dex: Some("uniswap_v2".to_string()),
            source_pair: Some("0xB4e16d0168e52d35CaCD2c6185b44281Ec28C9Dc".to_string()),
            source_liquidity_usd: Some(10000000.0),
            chain: Some("ethereum".to_string()),
            fetched_at: chrono::DateTime::parse_from_rfc3339("2023-01-01T00:00:00Z")
                .unwrap()
                .with_timezone(&chrono::Utc),
        };

        let serialized = serde_json::to_string(&result).unwrap();
        assert!(serialized.contains("USDC"));
        assert!(serialized.contains("1.0000"));
        assert!(serialized.contains("ethereum"));
    }

    #[test]
    fn test_token_price_result_deserialization() {
        let json = r#"{
            "token_address": "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
            "token_symbol": "USDC",
            "price_usd": "1.0000",
            "source_dex": "uniswap_v2",
            "source_pair": "0xB4e16d0168e52d35CaCD2c6185b44281Ec28C9Dc",
            "source_liquidity_usd": 10000000.0,
            "chain": "ethereum",
            "fetched_at": "2023-01-01T00:00:00Z"
        }"#;

        let result: TokenPriceResult = serde_json::from_str(json).unwrap();
        assert_eq!(
            result.token_address,
            "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"
        );
        assert_eq!(result.token_symbol, Some("USDC".to_string()));
        assert_eq!(result.price_usd, "1.0000");
        assert_eq!(result.source_dex, Some("uniswap_v2".to_string()));
        assert_eq!(
            result.source_pair,
            Some("0xB4e16d0168e52d35CaCD2c6185b44281Ec28C9Dc".to_string())
        );
        assert_eq!(result.source_liquidity_usd, Some(10000000.0));
        assert_eq!(result.chain, Some("ethereum".to_string()));
    }

    #[test]
    fn test_token_price_result_with_optional_none_fields() {
        let result = TokenPriceResult {
            token_address: "0x123".to_string(),
            token_symbol: None,
            price_usd: "0.5".to_string(),
            source_dex: None,
            source_pair: None,
            source_liquidity_usd: None,
            chain: None,
            fetched_at: chrono::Utc::now(),
        };

        assert_eq!(result.token_symbol, None);
        assert_eq!(result.source_dex, None);
        assert_eq!(result.source_pair, None);
        assert_eq!(result.source_liquidity_usd, None);
        assert_eq!(result.chain, None);
        assert_eq!(result.price_usd, "0.5");
    }

    #[test]
    fn test_clone_and_debug_traits() {
        let result = TokenPriceResult {
            token_address: "0x123".to_string(),
            token_symbol: Some("TEST".to_string()),
            price_usd: "1.0".to_string(),
            source_dex: Some("test_dex".to_string()),
            source_pair: Some("0x456".to_string()),
            source_liquidity_usd: Some(1000.0),
            chain: Some("test_chain".to_string()),
            fetched_at: chrono::Utc::now(),
        };

        // Test Clone trait
        let cloned = result.clone();
        assert_eq!(result.token_address, cloned.token_address);
        assert_eq!(result.token_symbol, cloned.token_symbol);
        assert_eq!(result.price_usd, cloned.price_usd);

        // Test Debug trait
        let debug_str = format!("{:?}", result);
        assert!(debug_str.contains("TokenPriceResult"));
        assert!(debug_str.contains("TEST"));
    }
}
