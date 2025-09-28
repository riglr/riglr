//! Event enrichment pipeline for metadata augmentation
//!
//! This module provides functionality to enrich parsed events with additional
//! metadata such as token information, price data, and transaction context.

extern crate alloc;
use crate::zero_copy::ZeroCopyEvent;

use alloc::sync::Arc;
use core::fmt::{Debug, Formatter, Result as FmtResult};
use core::time::Duration;
use dashmap::DashMap;
use serde_json::Value;
use solana_sdk::pubkey::Pubkey;
use std::time::Instant;

#[cfg(test)]
use crate::types::{EventType, ProtocolType};
#[cfg(test)]
use std::thread;
// UnifiedEvent trait has been removed

/// Configuration for event enrichment
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Config {
    /// Timeout for external API calls
    pub api_timeout: Duration,
    /// Enable price data enrichment
    pub enable_price_data: bool,
    /// Enable token metadata enrichment
    pub enable_token_metadata: bool,
    /// Enable transaction context enrichment
    pub enable_transaction_context: bool,
    /// Maximum cache size
    pub max_cache_size: usize,
    /// Cache TTL for metadata
    pub metadata_cache_ttl: Duration,
}

impl Default for Config {
    #[inline]
    fn default() -> Self {
        Self {
            api_timeout: Duration::from_secs(5),
            enable_price_data: true,
            enable_token_metadata: true,
            enable_transaction_context: true,
            max_cache_size: 10_000,
            metadata_cache_ttl: Duration::from_secs(300), // 5 minutes
        }
    }
}

/// Token metadata information
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct TokenMetadata {
    /// Token decimals
    pub decimals: u8,
    /// Token description
    pub description: Option<String>,
    /// When this metadata was fetched
    pub fetched_at: Instant,
    /// Token logo URI
    pub logo_uri: Option<String>,
    /// Token mint address
    pub mint: Pubkey,
    /// Token name
    pub name: Option<String>,
    /// Token symbol
    pub symbol: Option<String>,
}

/// Price information for a token
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct PriceData {
    /// When this price data was fetched
    pub fetched_at: Instant,
    /// Market cap in USD
    pub market_cap: Option<f64>,
    /// Token mint address
    pub mint: Pubkey,
    /// Price in SOL
    pub sol_price: Option<f64>,
    /// Price in USD
    pub usd_price: Option<f64>,
    /// 24h volume in USD
    pub volume_24h: Option<f64>,
}

/// Transaction context information
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct TransactionContext {
    /// Block height
    pub block_height: Option<u64>,
    /// Block time
    pub block_time: Option<i64>,
    /// Compute units consumed
    pub compute_units_consumed: Option<u64>,
    /// Fee paid for the transaction
    pub fee: Option<u64>,
    /// Transaction signature
    pub signature: String,
    /// Slot number
    pub slot: u64,
}

/// Cache entry for metadata with TTL
#[derive(Debug, Clone)]
struct CacheEntry<T> {
    /// Cached data
    data: T,
    /// Expiration timestamp
    expires_at: Instant,
}

impl<T> CacheEntry<T> {
    /// Check if cache entry has expired
    fn is_expired(&self) -> bool {
        Instant::now() > self.expires_at
    }

    /// Create a new cache entry with TTL
    fn new(data: T, ttl: Duration) -> Self {
        Self {
            data,
            expires_at: Instant::now().checked_add(ttl).unwrap_or_else(Instant::now),
        }
    }
}

/// Event enricher with caching and external API integration
pub struct EventEnricher {
    /// Enrichment configuration
    config: Config,
    /// HTTP client for external API calls
    http_client: reqwest::Client,
    /// Price data cache
    price_cache: Arc<DashMap<Pubkey, CacheEntry<PriceData>>>,
    /// Token metadata cache
    token_cache: Arc<DashMap<Pubkey, CacheEntry<TokenMetadata>>>,
}

impl Debug for EventEnricher {
    #[inline]
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        formatter
            .debug_struct("EventEnricher")
            .field("config", &self.config)
            .field("token_cache", &self.token_cache)
            .field("price_cache", &self.price_cache)
            .field("http_client", &"<reqwest::Client>")
            .finish()
    }
}

impl EventEnricher {
    /// Enrich a single event with additional metadata
    ///
    /// # Errors
    ///
    /// Returns `EnrichmentError` if external API calls fail or data processing fails
    #[inline]
    pub fn enrich_event(
        &self,
        mut event: ZeroCopyEvent<'static>,
    ) -> Result<ZeroCopyEvent<'static>, Error> {
        let start_time = Instant::now();

        // Extract relevant token addresses from the event
        let token_addresses = Self::extract_token_addresses(&event);

        // Enrich with token metadata
        if self.config.enable_token_metadata && !token_addresses.is_empty() {
            self.enrich_with_token_metadata(&mut event, &token_addresses);
        }

        // Enrich with price data
        if self.config.enable_price_data && !token_addresses.is_empty() {
            self.enrich_with_price_data(&mut event, &token_addresses);
        }

        // Enrich with transaction context
        if self.config.enable_transaction_context {
            Self::enrich_with_transaction_context(&mut event);
        }

        // Add enrichment timing
        if let Some(json_data) = event.get_json_data() {
            if let Some(json_object) = json_data.as_object() {
                let mut json_map = json_object.clone();
                json_map.insert(
                    "enrichment_time_ms".to_owned(),
                    Value::Number(serde_json::Number::from(
                        start_time.elapsed().as_millis().try_into().unwrap_or(0),
                    )),
                );
                event.set_json_data(Value::Object(json_map));
            }
        }

        Ok(event)
    }

    /// Enrich multiple events in parallel
    ///
    /// # Errors
    ///
    /// Returns `EnrichmentError` if any individual event enrichment fails
    #[inline]
    pub fn enrich_events(
        &self,
        events: Vec<ZeroCopyEvent<'static>>,
    ) -> Result<Vec<ZeroCopyEvent<'static>>, Error> {
        let mut enriched_events = Vec::new();
        for event in events {
            let enriched = self.enrich_event(event)?;
            enriched_events.push(enriched);
        }
        Ok(enriched_events)
    }

    /// Enrich event with price data
    fn enrich_with_price_data(
        &self,
        event: &mut ZeroCopyEvent<'static>,
        token_addresses: &[Pubkey],
    ) {
        let mut price_data = Vec::new();

        for &token_address in token_addresses {
            if let Some(price) = self.get_price_data(token_address) {
                price_data.push(price);
            }
        }

        if !price_data.is_empty() {
            // Add price data to JSON
            if let Some(json_data) = event.get_json_data() {
                if let Some(json_object) = json_data.as_object() {
                    let mut json_map = json_object.clone();
                    let price_json: Vec<Value> = price_data
                        .iter()
                        .map(|price| {
                            serde_json::json!({
                                "mint": price.mint.to_string(),
                                "usd_price": price.usd_price,
                                "sol_price": price.sol_price,
                                "volume_24h": price.volume_24h,
                                "market_cap": price.market_cap
                            })
                        })
                        .collect();

                    json_map.insert("price_data".to_owned(), Value::Array(price_json));
                    event.set_json_data(Value::Object(json_map));
                }
            }
        }
    }

    /// Enrich event with token metadata
    fn enrich_with_token_metadata(
        &self,
        event: &mut ZeroCopyEvent<'static>,
        token_addresses: &[Pubkey],
    ) {
        let mut token_metadata = Vec::new();

        for &token_address in token_addresses {
            if let Some(metadata) = self.get_token_metadata(token_address) {
                token_metadata.push(metadata);
            }
        }

        if !token_metadata.is_empty() {
            // Add token metadata to JSON
            if let Some(json_data) = event.get_json_data() {
                if let Some(json_object) = json_data.as_object() {
                    let mut json_map = json_object.clone();
                    let metadata_json: Vec<Value> = token_metadata
                        .iter()
                        .map(|meta| {
                            serde_json::json!({
                                "mint": meta.mint.to_string(),
                                "name": meta.name,
                                "symbol": meta.symbol,
                                "decimals": meta.decimals,
                                "logo_uri": meta.logo_uri
                            })
                        })
                        .collect();

                    json_map.insert("token_metadata".to_owned(), Value::Array(metadata_json));
                    event.set_json_data(Value::Object(json_map));
                }
            }
        }
    }

    /// Enrich event with transaction context
    fn enrich_with_transaction_context(event: &mut ZeroCopyEvent<'static>) {
        // In a real implementation, this would fetch additional transaction details
        // For now, we'll use the existing metadata

        if let Some(json_data) = event.get_json_data() {
            if let Some(json_object) = json_data.as_object() {
                let mut json_map = json_object.clone();

                json_map.insert(
                    "transaction_context".to_owned(),
                    serde_json::json!({
                        "signature": event.signature(),
                        "slot": event.slot(),
                        "block_time": event.block_number(),
                        "enriched": true
                    }),
                );

                event.set_json_data(Value::Object(json_map));
            }
        }
    }

    /// Recursively extract Pubkey-like strings from JSON
    fn extract_pubkeys_from_json(value: &Value, addresses: &mut Vec<Pubkey>) {
        match *value {
            Value::String(ref string_value) => {
                if let Ok(pubkey) = string_value.parse::<Pubkey>() {
                    addresses.push(pubkey);
                }
            }
            Value::Object(ref map) => {
                for (key, val) in map {
                    if key.contains("mint") || key.contains("token") || key.contains("address") {
                        Self::extract_pubkeys_from_json(val, addresses);
                    }
                }
            }
            Value::Array(ref arr) => {
                for val in arr {
                    Self::extract_pubkeys_from_json(val, addresses);
                }
            }
            Value::Null | Value::Bool(_) | Value::Number(_) => {}
        }
    }

    /// Extract token addresses from an event
    fn extract_token_addresses(event: &ZeroCopyEvent<'_>) -> Vec<Pubkey> {
        let mut addresses = Vec::new();

        // Look for token addresses in the JSON data
        if let Some(json_data) = event.get_json_data() {
            Self::extract_pubkeys_from_json(json_data, &mut addresses);
        }

        // Protocol-specific token extraction would be implemented here
        // Currently, we rely on generic JSON parsing above

        addresses
    }

    /// Fetch price data from external API
    fn fetch_price_data_from_api(mint: Pubkey) -> PriceData {
        // Placeholder implementation - would integrate with CoinGecko, Jupiter Price API, etc.
        // For now, return mock data for testing
        PriceData {
            mint,
            usd_price: Some(1.0),
            sol_price: Some(0.01),
            volume_24h: Some(1_000_000.0),
            market_cap: Some(10_000_000.0),
            fetched_at: Instant::now(),
        }
    }

    /// Fetch token metadata from external API
    fn fetch_token_metadata_from_api(mint: Pubkey) -> TokenMetadata {
        // Placeholder implementation - would integrate with Jupiter API, Solscan, etc.
        // For now, return mock data for testing
        TokenMetadata {
            mint,
            name: Some(format!("Token {mint}")),
            symbol: Some("TKN".to_owned()),
            decimals: 9,
            logo_uri: None,
            description: Some("Mock token metadata".to_owned()),
            fetched_at: Instant::now(),
        }
    }

    /// Get cache statistics
    #[must_use]
    #[inline]
    pub fn get_cache_stats(&self) -> CacheStats {
        CacheStats {
            price_cache_hit_rate: 0.0, // Would be tracked in real implementation
            price_cache_size: self.price_cache.len(),
            token_cache_hit_rate: 0.0, // Would be tracked in real implementation
            token_cache_size: self.token_cache.len(),
        }
    }

    /// Get price data with caching
    fn get_price_data(&self, mint: Pubkey) -> Option<PriceData> {
        // Check cache first
        if let Some(entry) = self.price_cache.get(&mint) {
            if !entry.is_expired() {
                return Some(entry.data.clone());
            }
        }

        // Fetch from external API (placeholder implementation)
        let price_data = Some(Self::fetch_price_data_from_api(mint));

        // Update cache
        if let Some(ref price) = price_data {
            // Collect expired keys first to avoid holding locks during iteration
            let expired_keys: Vec<Pubkey> = self
                .price_cache
                .iter()
                .filter(|entry| entry.value().is_expired())
                .map(|entry| *entry.key())
                .collect();

            // Remove expired entries
            for key in expired_keys {
                self.price_cache.remove(&key);
            }

            // If still at capacity after cleanup, remove oldest entries to make space
            if self.price_cache.len() >= self.config.max_cache_size {
                // Collect all keys first to avoid deadlock
                let keys_to_remove: Vec<Pubkey> = self
                    .price_cache
                    .iter()
                    .take(
                        self.price_cache
                            .len()
                            .saturating_sub(self.config.max_cache_size)
                            .saturating_add(1),
                    )
                    .map(|entry| *entry.key())
                    .collect();

                // Remove the collected keys
                for key in keys_to_remove {
                    self.price_cache.remove(&key);
                }
            }

            self.price_cache.insert(
                mint,
                CacheEntry::new(price.clone(), self.config.metadata_cache_ttl),
            );
        }

        price_data
    }

    /// Get token metadata with caching
    fn get_token_metadata(&self, mint: Pubkey) -> Option<TokenMetadata> {
        // Check cache first
        if let Some(entry) = self.token_cache.get(&mint) {
            if !entry.is_expired() {
                return Some(entry.data.clone());
            }
        }

        // Fetch from external API (placeholder implementation)
        let metadata = Some(Self::fetch_token_metadata_from_api(mint));

        // Update cache
        if let Some(ref meta) = metadata {
            // Collect expired keys first to avoid holding locks during iteration
            let expired_keys: Vec<Pubkey> = self
                .token_cache
                .iter()
                .filter(|entry| entry.value().is_expired())
                .map(|entry| *entry.key())
                .collect();

            // Remove expired entries
            for key in expired_keys {
                self.token_cache.remove(&key);
            }

            // If still at capacity after cleanup, remove oldest entries to make space
            if self.token_cache.len() >= self.config.max_cache_size {
                // Collect all keys first to avoid deadlock
                let keys_to_remove: Vec<Pubkey> = self
                    .token_cache
                    .iter()
                    .take(
                        self.token_cache
                            .len()
                            .saturating_sub(self.config.max_cache_size)
                            .saturating_add(1),
                    )
                    .map(|entry| *entry.key())
                    .collect();

                // Remove the collected keys
                for key in keys_to_remove {
                    self.token_cache.remove(&key);
                }
            }

            self.token_cache.insert(
                mint,
                CacheEntry::new(meta.clone(), self.config.metadata_cache_ttl),
            );
        }

        metadata
    }

    /// Create a new event enricher
    ///
    /// # Errors
    ///
    /// Returns `EnrichmentError` if HTTP client creation fails
    #[inline]
    pub fn new(config: Config) -> Result<Self, Error> {
        let http_client = reqwest::Client::builder()
            .timeout(config.api_timeout)
            .build()
            .map_err(Error::HttpError)?;

        Ok(Self {
            config,
            http_client,
            price_cache: Arc::new(DashMap::new()),
            token_cache: Arc::new(DashMap::new()),
        })
    }
}

impl Clone for EventEnricher {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            http_client: self.http_client.clone(),
            price_cache: self.price_cache.clone(),
            token_cache: self.token_cache.clone(),
        }
    }

    fn clone_from(&mut self, source: &Self) {
        self.config.clone_from(&source.config);
        self.http_client.clone_from(&source.http_client);
        self.price_cache.clone_from(&source.price_cache);
        self.token_cache.clone_from(&source.token_cache);
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct CacheStats {
    /// Price cache hit rate (0.0 to 1.0)
    pub price_cache_hit_rate: f64,
    /// Number of entries in price data cache
    pub price_cache_size: usize,
    /// Token cache hit rate (0.0 to 1.0)
    pub token_cache_hit_rate: f64,
    /// Number of entries in token metadata cache
    pub token_cache_size: usize,
}

/// Error type for enrichment operations
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// Cache error
    #[error("Cache error: {0}")]
    CacheError(String),

    /// HTTP request failed
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    /// API rate limit exceeded
    #[error("API rate limit exceeded")]
    RateLimitExceeded,

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    /// Task execution error
    #[error("Task execution error: {0}")]
    TaskError(String),
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use crate::solana_metadata::SolanaEventMetadata;

    #[tokio::test]
    async fn test_enricher_creation() {
        let config = Config::default();
        let enricher = EventEnricher::new(config).expect("Failed to create enricher");

        let stats = enricher.get_cache_stats();
        assert_eq!(stats.token_cache_size, 0);
        assert_eq!(stats.price_cache_size, 0);
    }

    #[tokio::test]
    async fn test_token_address_extraction() {
        let config = Config::default();
        let _enricher = EventEnricher::new(config).expect("Failed to create enricher");

        let metadata = SolanaEventMetadata::default();
        let mut event = ZeroCopyEvent::new_owned(metadata, vec![]);

        // Add JSON data with token addresses
        let json = serde_json::json!({
            "input_mint": "So11111111111111111111111111111111111111112",
            "output_mint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"
        });
        event.set_json_data(json);

        let addresses = EventEnricher::extract_token_addresses(&event);
        assert_eq!(addresses.len(), 2);
    }

    #[test]
    fn test_cache_entry_expiration() {
        let metadata = TokenMetadata {
            mint: Pubkey::default(),
            name: Some("Test".to_string()),
            symbol: Some("TST".to_string()),
            decimals: 9,
            logo_uri: None,
            description: None,
            fetched_at: Instant::now(),
        };

        let entry = CacheEntry::new(metadata, Duration::from_millis(1));
        assert!(!entry.is_expired());

        thread::sleep(Duration::from_millis(2));
        assert!(entry.is_expired());
    }

    #[test]
    fn test_enrichment_config_default() {
        let config = Config::default();
        assert!(config.enable_token_metadata);
        assert!(config.enable_price_data);
        assert!(config.enable_transaction_context);
        assert_eq!(config.metadata_cache_ttl, Duration::from_secs(300));
        assert_eq!(config.max_cache_size, 10000);
        assert_eq!(config.api_timeout, Duration::from_secs(5));
    }

    #[test]
    fn test_enrichment_config_clone() {
        let config = Config {
            enable_token_metadata: false,
            enable_price_data: true,
            enable_transaction_context: false,
            metadata_cache_ttl: Duration::from_secs(100),
            max_cache_size: 5000,
            api_timeout: Duration::from_secs(10),
        };
        let cloned = config.clone();
        assert_eq!(config.enable_token_metadata, cloned.enable_token_metadata);
        assert_eq!(config.enable_price_data, cloned.enable_price_data);
        assert_eq!(
            config.enable_transaction_context,
            cloned.enable_transaction_context
        );
        assert_eq!(config.metadata_cache_ttl, cloned.metadata_cache_ttl);
        assert_eq!(config.max_cache_size, cloned.max_cache_size);
        assert_eq!(config.api_timeout, cloned.api_timeout);
    }

    #[test]
    fn test_token_metadata_clone() {
        let metadata = TokenMetadata {
            mint: Pubkey::default(),
            name: Some("Test Token".to_string()),
            symbol: Some("TEST".to_string()),
            decimals: 6,
            logo_uri: Some("https://example.com/logo.png".to_string()),
            description: Some("A test token".to_string()),
            fetched_at: Instant::now(),
        };
        let cloned = metadata.clone();
        assert_eq!(metadata.mint, cloned.mint);
        assert_eq!(metadata.name, cloned.name);
        assert_eq!(metadata.symbol, cloned.symbol);
        assert_eq!(metadata.decimals, cloned.decimals);
        assert_eq!(metadata.logo_uri, cloned.logo_uri);
        assert_eq!(metadata.description, cloned.description);
    }

    #[test]
    fn test_price_data_clone() {
        let price = PriceData {
            mint: Pubkey::default(),
            usd_price: Some(1.5),
            sol_price: Some(0.02),
            volume_24h: Some(50000.0),
            market_cap: Some(1_000_000.0),
            fetched_at: Instant::now(),
        };
        let cloned = price.clone();
        assert_eq!(price.mint, cloned.mint);
        assert_eq!(price.usd_price, cloned.usd_price);
        assert_eq!(price.sol_price, cloned.sol_price);
        assert_eq!(price.volume_24h, cloned.volume_24h);
        assert_eq!(price.market_cap, cloned.market_cap);
    }

    #[test]
    fn test_transaction_context_clone() {
        let context = TransactionContext {
            signature: "test_signature".to_string(),
            block_height: Some(12345),
            slot: 67890,
            block_time: Some(1_234_567_890),
            fee: Some(5000),
            compute_units_consumed: Some(200_000),
        };
        let cloned = context.clone();
        assert_eq!(context.signature, cloned.signature);
        assert_eq!(context.block_height, cloned.block_height);
        assert_eq!(context.slot, cloned.slot);
        assert_eq!(context.block_time, cloned.block_time);
        assert_eq!(context.fee, cloned.fee);
        assert_eq!(
            context.compute_units_consumed,
            cloned.compute_units_consumed
        );
    }

    #[test]
    fn test_cache_entry_new_and_data_access() {
        let test_data = "test_data".to_string();
        let ttl = Duration::from_secs(60);
        let entry = CacheEntry::new(test_data.clone(), ttl);
        assert_eq!(entry.data, test_data);
        assert!(!entry.is_expired());
    }

    #[test]
    fn test_cache_entry_clone() {
        let test_data = "test_data".to_string();
        let entry = CacheEntry::new(test_data, Duration::from_secs(60));
        let cloned = entry.clone();
        assert_eq!(entry.data, cloned.data);
    }

    #[test]
    fn test_cache_stats_clone() {
        let stats = CacheStats {
            token_cache_size: 10,
            price_cache_size: 5,
            token_cache_hit_rate: 0.8,
            price_cache_hit_rate: 0.9,
        };
        let cloned = stats.clone();
        assert_eq!(stats.token_cache_size, cloned.token_cache_size);
        assert_eq!(stats.price_cache_size, cloned.price_cache_size);
        assert!((stats.token_cache_hit_rate - cloned.token_cache_hit_rate).abs() < f64::EPSILON);
        assert!((stats.price_cache_hit_rate - cloned.price_cache_hit_rate).abs() < f64::EPSILON);
    }

    #[test]
    fn test_enrichment_error_display() {
        // Create a mock serialization error
        let invalid_json = r#"{"invalid": json syntax"#;
        match serde_json::from_str::<serde_json::Value>(invalid_json) {
            Err(parse_error) => {
                let serialization_error = Error::SerializationError(parse_error);
                assert!(format!("{serialization_error}").contains("Serialization error"));
            }
            _ => panic!("Expected JSON parsing to fail"),
        }

        let task_error = Error::TaskError("test error".to_string());
        assert_eq!(format!("{task_error}"), "Task execution error: test error");

        let cache_error = Error::CacheError("cache failure".to_string());
        assert_eq!(format!("{cache_error}"), "Cache error: cache failure");

        let rate_limit_error = Error::RateLimitExceeded;
        assert_eq!(format!("{rate_limit_error}"), "API rate limit exceeded");
    }

    #[tokio::test]
    async fn test_event_enricher_clone() {
        let config = Config::default();
        let enricher = EventEnricher::new(config).expect("Failed to create enricher");
        let cloned = enricher.clone();

        // Both should have same cache stats initially
        let stats1 = enricher.get_cache_stats();
        let stats2 = cloned.get_cache_stats();
        assert_eq!(stats1.token_cache_size, stats2.token_cache_size);
        assert_eq!(stats1.price_cache_size, stats2.price_cache_size);
    }

    #[tokio::test]
    async fn test_extract_pubkeys_from_json_string() {
        let config = Config::default();
        let _enricher = EventEnricher::new(config).expect("Failed to create enricher");

        let mut addresses = Vec::new();
        let json_value = Value::String("So11111111111111111111111111111111111111112".to_string());
        EventEnricher::extract_pubkeys_from_json(&json_value, &mut addresses);
        assert_eq!(addresses.len(), 1);
    }

    #[tokio::test]
    async fn test_extract_pubkeys_from_json_invalid_string() {
        let config = Config::default();
        let _enricher = EventEnricher::new(config).expect("Failed to create enricher");

        let mut addresses = Vec::new();
        let json_value = Value::String("invalid_pubkey".to_string());
        EventEnricher::extract_pubkeys_from_json(&json_value, &mut addresses);
        assert_eq!(addresses.len(), 0);
    }

    #[tokio::test]
    async fn test_extract_pubkeys_from_json_object() {
        let config = Config::default();
        let _enricher = EventEnricher::new(config).expect("Failed to create enricher");

        let mut addresses = Vec::new();
        let json_obj = serde_json::json!({
            "mint_address": "So11111111111111111111111111111111111111112",
            "other_field": "value",
            "token_mint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"
        });
        EventEnricher::extract_pubkeys_from_json(&json_obj, &mut addresses);
        assert_eq!(addresses.len(), 2);
    }

    #[tokio::test]
    async fn test_extract_pubkeys_from_json_array() {
        let config = Config::default();
        let _enricher = EventEnricher::new(config).expect("Failed to create enricher");

        let mut addresses = Vec::new();
        let json_array = Value::Array(vec![
            Value::String("So11111111111111111111111111111111111111112".to_string()),
            Value::String("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string()),
        ]);
        EventEnricher::extract_pubkeys_from_json(&json_array, &mut addresses);
        assert_eq!(addresses.len(), 2);
    }

    #[tokio::test]
    async fn test_extract_pubkeys_from_json_other_types() {
        let config = Config::default();
        let _enricher = EventEnricher::new(config).expect("Failed to create enricher");

        let mut addresses = Vec::new();

        // Test with number
        EventEnricher::extract_pubkeys_from_json(
            &Value::Number(serde_json::Number::from(123)),
            &mut addresses,
        );
        assert_eq!(addresses.len(), 0);

        // Test with boolean
        EventEnricher::extract_pubkeys_from_json(&Value::Bool(true), &mut addresses);
        assert_eq!(addresses.len(), 0);

        // Test with null
        EventEnricher::extract_pubkeys_from_json(&Value::Null, &mut addresses);
        assert_eq!(addresses.len(), 0);
    }

    #[tokio::test]
    async fn test_extract_token_addresses_with_jupiter_protocol() {
        let config = Config::default();
        let _enricher = EventEnricher::new(config).expect("Failed to create enricher");

        let core = riglr_events_core::EventMetadata::new(
            "test".to_string(),
            riglr_events_core::EventKind::Transaction,
            "test".to_string(),
        );
        let metadata = SolanaEventMetadata::new(
            "test_sig".to_string(),
            100,
            EventType::Swap,
            ProtocolType::Jupiter,
            "0".to_string(),
            0,
            core,
        );
        let mut event = ZeroCopyEvent::new_owned(metadata, vec![]);

        let json = serde_json::json!({
            "input_mint": "So11111111111111111111111111111111111111112"
        });
        event.set_json_data(json);

        let addresses = EventEnricher::extract_token_addresses(&event);
        assert_eq!(addresses.len(), 1);
    }

    #[tokio::test]
    async fn test_extract_token_addresses_with_raydium_protocol() {
        let config = Config::default();
        let _enricher = EventEnricher::new(config).expect("Failed to create enricher");

        let core = riglr_events_core::EventMetadata::new(
            "test".to_string(),
            riglr_events_core::EventKind::Transaction,
            "test".to_string(),
        );
        let metadata = SolanaEventMetadata::new(
            "test_sig".to_string(),
            100,
            EventType::Swap,
            ProtocolType::RaydiumAmmV4,
            "0".to_string(),
            0,
            core,
        );
        let mut event = ZeroCopyEvent::new_owned(metadata, vec![]);

        let json = serde_json::json!({
            "pool_mint": "So11111111111111111111111111111111111111112"
        });
        event.set_json_data(json);

        let addresses = EventEnricher::extract_token_addresses(&event);
        assert_eq!(addresses.len(), 1);
    }

    #[tokio::test]
    async fn test_extract_token_addresses_with_other_protocol() {
        let config = Config::default();
        let _enricher = EventEnricher::new(config).expect("Failed to create enricher");

        let core = riglr_events_core::EventMetadata::new(
            "test".to_string(),
            riglr_events_core::EventKind::Transaction,
            "test".to_string(),
        );
        let metadata = SolanaEventMetadata::new(
            "test_sig".to_string(),
            100,
            EventType::Swap,
            ProtocolType::OrcaWhirlpool,
            "0".to_string(),
            0,
            core,
        );
        let mut event = ZeroCopyEvent::new_owned(metadata, vec![]);

        let json = serde_json::json!({
            "token_mint": "So11111111111111111111111111111111111111112"
        });
        event.set_json_data(json);

        let addresses = EventEnricher::extract_token_addresses(&event);
        assert_eq!(addresses.len(), 1);
    }

    #[tokio::test]
    async fn test_extract_token_addresses_no_json_data() {
        let config = Config::default();
        let _enricher = EventEnricher::new(config).expect("Failed to create enricher");

        let metadata = SolanaEventMetadata::default();
        let event = ZeroCopyEvent::new_owned(metadata, vec![]);

        let addresses = EventEnricher::extract_token_addresses(&event);
        assert_eq!(addresses.len(), 0);
    }

    #[tokio::test]
    async fn test_enrich_event_with_all_features_disabled() {
        let config = Config {
            enable_token_metadata: false,
            enable_price_data: false,
            enable_transaction_context: false,
            ..Default::default()
        };
        let enricher = EventEnricher::new(config).expect("Failed to create enricher");

        let metadata = SolanaEventMetadata::default();
        let mut event = ZeroCopyEvent::new_owned(metadata, vec![]);

        let json = serde_json::json!({
            "test": "value"
        });
        event.set_json_data(json);

        let result = enricher.enrich_event(event);
        assert!(result.is_ok());

        let enriched_event = result.expect("Failed to enrich event");
        if let Some(json_data) = enriched_event.get_json_data() {
            assert!(json_data.get("enrichment_time_ms").is_some());
        }
    }

    #[tokio::test]
    async fn test_enrich_event_with_no_token_addresses() {
        let config = Config::default();
        let enricher = EventEnricher::new(config).expect("Failed to create enricher");

        let metadata = SolanaEventMetadata::default();
        let mut event = ZeroCopyEvent::new_owned(metadata, vec![]);

        let json = serde_json::json!({
            "no_tokens": "here"
        });
        event.set_json_data(json);

        let result = enricher.enrich_event(event);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_enrich_event_with_no_json_data() {
        let config = Config::default();
        let enricher = EventEnricher::new(config).expect("Failed to create enricher");

        let metadata = SolanaEventMetadata::default();
        let event = ZeroCopyEvent::new_owned(metadata, vec![]);

        let result = enricher.enrich_event(event);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_enrich_events_empty_list() {
        let config = Config::default();
        let enricher = EventEnricher::new(config).expect("Failed to create enricher");

        let result = enricher.enrich_events(vec![]);
        assert!(result.is_ok());
        assert_eq!(result.expect("Failed to enrich events").len(), 0);
    }

    #[tokio::test]
    async fn test_enrich_events_multiple() {
        let config = Config::default();
        let enricher = EventEnricher::new(config).expect("Failed to create enricher");

        let metadata1 = SolanaEventMetadata::default();
        let event1 = ZeroCopyEvent::new_owned(metadata1, vec![]);

        let metadata2 = SolanaEventMetadata::default();
        let event2 = ZeroCopyEvent::new_owned(metadata2, vec![]);

        let result = enricher.enrich_events(vec![event1, event2]);
        assert!(result.is_ok());
        assert_eq!(result.expect("Failed to enrich events").len(), 2);
    }

    #[tokio::test]
    async fn test_enrich_with_token_metadata_no_tokens() {
        let config = Config::default();
        let enricher = EventEnricher::new(config).expect("Failed to create enricher");

        let metadata = SolanaEventMetadata::default();
        let mut event = ZeroCopyEvent::new_owned(metadata, vec![]);

        let json = serde_json::json!({
            "test": "value"
        });
        event.set_json_data(json);

        enricher.enrich_with_token_metadata(&mut event, &[]);
    }

    #[tokio::test]
    async fn test_enrich_with_token_metadata_no_json() {
        let config = Config::default();
        let enricher = EventEnricher::new(config).expect("Failed to create enricher");

        let metadata = SolanaEventMetadata::default();
        let mut event = ZeroCopyEvent::new_owned(metadata, vec![]);

        let token_addresses = vec![Pubkey::default()];
        enricher.enrich_with_token_metadata(&mut event, &token_addresses);
    }

    #[tokio::test]
    async fn test_enrich_with_price_data_no_tokens() {
        let config = Config::default();
        let enricher = EventEnricher::new(config).expect("Failed to create enricher");

        let metadata = SolanaEventMetadata::default();
        let mut event = ZeroCopyEvent::new_owned(metadata, vec![]);

        let json = serde_json::json!({
            "test": "value"
        });
        event.set_json_data(json);

        enricher.enrich_with_price_data(&mut event, &[]);
    }

    #[tokio::test]
    async fn test_enrich_with_price_data_no_json() {
        let config = Config::default();
        let enricher = EventEnricher::new(config).expect("Failed to create enricher");

        let metadata = SolanaEventMetadata::default();
        let mut event = ZeroCopyEvent::new_owned(metadata, vec![]);

        let token_addresses = vec![Pubkey::default()];
        enricher.enrich_with_price_data(&mut event, &token_addresses);
    }

    #[tokio::test]
    async fn test_enrich_with_transaction_context_no_json() {
        let config = Config::default();
        let _enricher = EventEnricher::new(config).expect("Failed to create enricher");

        let metadata = SolanaEventMetadata::default();
        let mut event = ZeroCopyEvent::new_owned(metadata, vec![]);

        EventEnricher::enrich_with_transaction_context(&mut event);
        // Function now returns () instead of Result
    }

    #[tokio::test]
    async fn test_get_token_metadata_cache_hit() {
        let config = Config::default();
        let enricher = EventEnricher::new(config).expect("Failed to create enricher");

        let mint = Pubkey::default();

        // First call should fetch from API and cache
        let result1 = enricher.get_token_metadata(mint);
        assert!(result1.is_some());

        // Second call should hit cache
        let result2 = enricher.get_token_metadata(mint);
        assert!(result2.is_some());
    }

    #[tokio::test]
    async fn test_get_price_data_cache_hit() {
        let config = Config::default();
        let enricher = EventEnricher::new(config).expect("Failed to create enricher");

        let mint = Pubkey::default();

        // First call should fetch from API and cache
        let result1 = enricher.get_price_data(mint);
        assert!(result1.is_some());

        // Second call should hit cache
        let result2 = enricher.get_price_data(mint);
        assert!(result2.is_some());
    }

    #[tokio::test]
    async fn test_cache_cleanup_when_full() {
        let config = Config {
            max_cache_size: 1, // Very small cache
            ..Default::default()
        };
        let enricher = EventEnricher::new(config).expect("Failed to create enricher");

        let mint1 = Pubkey::default();
        let mint2 = Pubkey::new_unique();

        // Fill cache with first mint
        let _ = enricher.get_token_metadata(mint1);

        // Add second mint (should trigger cleanup)
        let _ = enricher.get_token_metadata(mint2);

        let stats = enricher.get_cache_stats();
        assert!(stats.token_cache_size <= 1);
    }

    #[test]
    fn test_fetch_token_metadata_from_api() {
        let config = Config::default();
        let _enricher = EventEnricher::new(config).expect("Failed to create enricher");

        let mint = Pubkey::default();
        let meta = EventEnricher::fetch_token_metadata_from_api(mint);
        assert_eq!(meta.mint, mint);
        assert!(meta.name.is_some());
        assert!(meta.symbol.is_some());
        assert_eq!(meta.decimals, 9);
    }

    #[test]
    fn test_fetch_price_data_from_api() {
        let config = Config::default();
        let _enricher = EventEnricher::new(config).expect("Failed to create enricher");

        let mint = Pubkey::default();
        let price = EventEnricher::fetch_price_data_from_api(mint);
        assert_eq!(price.mint, mint);
        assert!(price.usd_price.is_some());
        assert!(price.sol_price.is_some());
        assert!(price.volume_24h.is_some());
        assert!(price.market_cap.is_some());
    }
}
