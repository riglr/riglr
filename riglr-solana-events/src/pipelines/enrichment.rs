//! Event enrichment pipeline for metadata augmentation
//!
//! This module provides functionality to enrich parsed events with additional
//! metadata such as token information, price data, and transaction context.

use crate::types::ProtocolType;
use crate::zero_copy::ZeroCopyEvent;
use dashmap::DashMap;
use serde_json::Value;
use solana_sdk::pubkey::Pubkey;
use std::sync::Arc;
use std::time::{Duration, Instant};
// UnifiedEvent trait has been removed

/// Configuration for event enrichment
#[derive(Debug, Clone)]
pub struct EnrichmentConfig {
    /// Enable token metadata enrichment
    pub enable_token_metadata: bool,
    /// Enable price data enrichment
    pub enable_price_data: bool,
    /// Enable transaction context enrichment
    pub enable_transaction_context: bool,
    /// Cache TTL for metadata
    pub metadata_cache_ttl: Duration,
    /// Maximum cache size
    pub max_cache_size: usize,
    /// Timeout for external API calls
    pub api_timeout: Duration,
}

impl Default for EnrichmentConfig {
    fn default() -> Self {
        Self {
            enable_token_metadata: true,
            enable_price_data: true,
            enable_transaction_context: true,
            metadata_cache_ttl: Duration::from_secs(300), // 5 minutes
            max_cache_size: 10000,
            api_timeout: Duration::from_secs(5),
        }
    }
}

/// Token metadata information
#[derive(Debug, Clone)]
pub struct TokenMetadata {
    /// Token mint address
    pub mint: Pubkey,
    /// Token name
    pub name: Option<String>,
    /// Token symbol
    pub symbol: Option<String>,
    /// Token decimals
    pub decimals: u8,
    /// Token logo URI
    pub logo_uri: Option<String>,
    /// Token description
    pub description: Option<String>,
    /// When this metadata was fetched
    pub fetched_at: Instant,
}

/// Price information for a token
#[derive(Debug, Clone)]
pub struct PriceData {
    /// Token mint address
    pub mint: Pubkey,
    /// Price in USD
    pub usd_price: Option<f64>,
    /// Price in SOL
    pub sol_price: Option<f64>,
    /// 24h volume in USD
    pub volume_24h: Option<f64>,
    /// Market cap in USD
    pub market_cap: Option<f64>,
    /// When this price data was fetched
    pub fetched_at: Instant,
}

/// Transaction context information
#[derive(Debug, Clone)]
pub struct TransactionContext {
    /// Transaction signature
    pub signature: String,
    /// Block height
    pub block_height: Option<u64>,
    /// Slot number
    pub slot: u64,
    /// Block time
    pub block_time: Option<i64>,
    /// Fee paid for the transaction
    pub fee: Option<u64>,
    /// Compute units consumed
    pub compute_units_consumed: Option<u64>,
}

/// Cache entry for metadata with TTL
#[derive(Debug, Clone)]
struct CacheEntry<T> {
    data: T,
    expires_at: Instant,
}

impl<T> CacheEntry<T> {
    fn new(data: T, ttl: Duration) -> Self {
        Self {
            data,
            expires_at: Instant::now() + ttl,
        }
    }

    fn is_expired(&self) -> bool {
        Instant::now() > self.expires_at
    }
}

/// Event enricher with caching and external API integration
pub struct EventEnricher {
    /// Enrichment configuration
    config: EnrichmentConfig,
    /// Token metadata cache
    token_cache: Arc<DashMap<Pubkey, CacheEntry<TokenMetadata>>>,
    /// Price data cache
    price_cache: Arc<DashMap<Pubkey, CacheEntry<PriceData>>>,
    /// HTTP client for external API calls
    http_client: reqwest::Client,
}

impl EventEnricher {
    /// Create a new event enricher
    pub fn new(config: EnrichmentConfig) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(config.api_timeout)
            .build()
            .expect("Failed to create HTTP client");

        Self {
            config,
            token_cache: Arc::new(DashMap::new()),
            price_cache: Arc::new(DashMap::new()),
            http_client,
        }
    }

    /// Enrich a single event with additional metadata
    pub async fn enrich_event(
        &self,
        mut event: ZeroCopyEvent<'static>,
    ) -> Result<ZeroCopyEvent<'static>, EnrichmentError> {
        let start_time = Instant::now();

        // Extract relevant token addresses from the event
        let token_addresses = self.extract_token_addresses(&event);

        // Enrich with token metadata
        if self.config.enable_token_metadata && !token_addresses.is_empty() {
            self.enrich_with_token_metadata(&mut event, &token_addresses)
                .await?;
        }

        // Enrich with price data
        if self.config.enable_price_data && !token_addresses.is_empty() {
            self.enrich_with_price_data(&mut event, &token_addresses)
                .await?;
        }

        // Enrich with transaction context
        if self.config.enable_transaction_context {
            self.enrich_with_transaction_context(&mut event).await?;
        }

        // Add enrichment timing
        if let Some(json_data) = event.get_json_data() {
            let mut json_map = json_data.as_object().unwrap().clone();
            json_map.insert(
                "enrichment_time_ms".to_string(),
                Value::Number(serde_json::Number::from(
                    start_time.elapsed().as_millis() as u64
                )),
            );
            event.set_json_data(Value::Object(json_map));
        }

        Ok(event)
    }

    /// Enrich multiple events in parallel
    pub async fn enrich_events(
        &self,
        events: Vec<ZeroCopyEvent<'static>>,
    ) -> Result<Vec<ZeroCopyEvent<'static>>, EnrichmentError> {
        let mut tasks = Vec::new();

        for event in events {
            let enricher = self.clone();
            tasks.push(tokio::spawn(
                async move { enricher.enrich_event(event).await },
            ));
        }

        let mut enriched_events = Vec::new();
        for task in tasks {
            match task.await {
                Ok(Ok(event)) => enriched_events.push(event),
                Ok(Err(e)) => return Err(e),
                Err(e) => return Err(EnrichmentError::TaskError(e.to_string())),
            }
        }

        Ok(enriched_events)
    }

    /// Extract token addresses from an event
    fn extract_token_addresses(&self, event: &ZeroCopyEvent) -> Vec<Pubkey> {
        let mut addresses = Vec::new();

        // Look for token addresses in the JSON data
        if let Some(json_data) = event.get_json_data() {
            self.extract_pubkeys_from_json(json_data, &mut addresses);
        }

        // Protocol-specific token extraction
        match event.protocol_type() {
            ProtocolType::Jupiter => {
                // Jupiter events might have input/output mints
                // This would be implemented based on actual data structure
            }
            ProtocolType::RaydiumAmmV4 => {
                // Raydium events have pool tokens
                // This would be implemented based on actual data structure
            }
            _ => {}
        }

        addresses
    }

    /// Recursively extract Pubkey-like strings from JSON
    #[allow(clippy::only_used_in_recursion)]
    fn extract_pubkeys_from_json(&self, value: &Value, addresses: &mut Vec<Pubkey>) {
        match value {
            Value::String(s) => {
                if let Ok(pubkey) = s.parse::<Pubkey>() {
                    addresses.push(pubkey);
                }
            }
            Value::Object(map) => {
                for (key, val) in map {
                    if key.contains("mint") || key.contains("token") || key.contains("address") {
                        self.extract_pubkeys_from_json(val, addresses);
                    }
                }
            }
            Value::Array(arr) => {
                for val in arr {
                    self.extract_pubkeys_from_json(val, addresses);
                }
            }
            _ => {}
        }
    }

    /// Enrich event with token metadata
    async fn enrich_with_token_metadata(
        &self,
        event: &mut ZeroCopyEvent<'static>,
        token_addresses: &[Pubkey],
    ) -> Result<(), EnrichmentError> {
        let mut token_metadata = Vec::new();

        for &token_address in token_addresses {
            if let Some(metadata) = self.get_token_metadata(token_address).await? {
                token_metadata.push(metadata);
            }
        }

        if !token_metadata.is_empty() {
            // Add token metadata to JSON
            if let Some(json_data) = event.get_json_data() {
                let mut json_map = json_data.as_object().unwrap().clone();
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

                json_map.insert("token_metadata".to_string(), Value::Array(metadata_json));
                event.set_json_data(Value::Object(json_map));
            }
        }

        Ok(())
    }

    /// Enrich event with price data
    async fn enrich_with_price_data(
        &self,
        event: &mut ZeroCopyEvent<'static>,
        token_addresses: &[Pubkey],
    ) -> Result<(), EnrichmentError> {
        let mut price_data = Vec::new();

        for &token_address in token_addresses {
            if let Some(price) = self.get_price_data(token_address).await? {
                price_data.push(price);
            }
        }

        if !price_data.is_empty() {
            // Add price data to JSON
            if let Some(json_data) = event.get_json_data() {
                let mut json_map = json_data.as_object().unwrap().clone();
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

                json_map.insert("price_data".to_string(), Value::Array(price_json));
                event.set_json_data(Value::Object(json_map));
            }
        }

        Ok(())
    }

    /// Enrich event with transaction context
    async fn enrich_with_transaction_context(
        &self,
        event: &mut ZeroCopyEvent<'static>,
    ) -> Result<(), EnrichmentError> {
        // In a real implementation, this would fetch additional transaction details
        // For now, we'll use the existing metadata

        if let Some(json_data) = event.get_json_data() {
            let mut json_map = json_data.as_object().unwrap().clone();

            json_map.insert(
                "transaction_context".to_string(),
                serde_json::json!({
                    "signature": event.signature(),
                    "slot": event.slot(),
                    "block_time": event.block_number(),
                    "enriched": true
                }),
            );

            event.set_json_data(Value::Object(json_map));
        }

        Ok(())
    }

    /// Get token metadata with caching
    async fn get_token_metadata(
        &self,
        mint: Pubkey,
    ) -> Result<Option<TokenMetadata>, EnrichmentError> {
        // Check cache first
        if let Some(entry) = self.token_cache.get(&mint) {
            if !entry.is_expired() {
                return Ok(Some(entry.data.clone()));
            }
        }

        // Fetch from external API (placeholder implementation)
        let metadata = self.fetch_token_metadata_from_api(mint).await?;

        // Update cache
        if let Some(ref meta) = metadata {
            // Cleanup expired entries if cache is full
            if self.token_cache.len() >= self.config.max_cache_size {
                self.token_cache.retain(|_, entry| !entry.is_expired());
            }

            self.token_cache.insert(
                mint,
                CacheEntry::new(meta.clone(), self.config.metadata_cache_ttl),
            );
        }

        Ok(metadata)
    }

    /// Get price data with caching
    async fn get_price_data(&self, mint: Pubkey) -> Result<Option<PriceData>, EnrichmentError> {
        // Check cache first
        if let Some(entry) = self.price_cache.get(&mint) {
            if !entry.is_expired() {
                return Ok(Some(entry.data.clone()));
            }
        }

        // Fetch from external API (placeholder implementation)
        let price_data = self.fetch_price_data_from_api(mint).await?;

        // Update cache
        if let Some(ref price) = price_data {
            // Cleanup expired entries if cache is full
            if self.price_cache.len() >= self.config.max_cache_size {
                self.price_cache.retain(|_, entry| !entry.is_expired());
            }

            self.price_cache.insert(
                mint,
                CacheEntry::new(price.clone(), self.config.metadata_cache_ttl),
            );
        }

        Ok(price_data)
    }

    /// Fetch token metadata from external API
    async fn fetch_token_metadata_from_api(
        &self,
        mint: Pubkey,
    ) -> Result<Option<TokenMetadata>, EnrichmentError> {
        // Placeholder implementation - would integrate with Jupiter API, Solscan, etc.
        // For now, return mock data for testing
        Ok(Some(TokenMetadata {
            mint,
            name: Some(format!("Token {}", mint)),
            symbol: Some("TKN".to_string()),
            decimals: 9,
            logo_uri: None,
            description: Some("Mock token metadata".to_string()),
            fetched_at: Instant::now(),
        }))
    }

    /// Fetch price data from external API
    async fn fetch_price_data_from_api(
        &self,
        mint: Pubkey,
    ) -> Result<Option<PriceData>, EnrichmentError> {
        // Placeholder implementation - would integrate with CoinGecko, Jupiter Price API, etc.
        // For now, return mock data for testing
        Ok(Some(PriceData {
            mint,
            usd_price: Some(1.0),
            sol_price: Some(0.01),
            volume_24h: Some(1000000.0),
            market_cap: Some(10000000.0),
            fetched_at: Instant::now(),
        }))
    }

    /// Get cache statistics
    pub fn get_cache_stats(&self) -> CacheStats {
        CacheStats {
            token_cache_size: self.token_cache.len(),
            price_cache_size: self.price_cache.len(),
            token_cache_hit_rate: 0.0, // Would be tracked in real implementation
            price_cache_hit_rate: 0.0, // Would be tracked in real implementation
        }
    }
}

impl Clone for EventEnricher {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            token_cache: self.token_cache.clone(),
            price_cache: self.price_cache.clone(),
            http_client: self.http_client.clone(),
        }
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    /// Number of entries in token metadata cache
    pub token_cache_size: usize,
    /// Number of entries in price data cache
    pub price_cache_size: usize,
    /// Token cache hit rate (0.0 to 1.0)
    pub token_cache_hit_rate: f64,
    /// Price cache hit rate (0.0 to 1.0)
    pub price_cache_hit_rate: f64,
}

/// Error type for enrichment operations
#[derive(Debug, thiserror::Error)]
pub enum EnrichmentError {
    /// HTTP request failed
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    /// Task execution error
    #[error("Task execution error: {0}")]
    TaskError(String),

    /// Cache error
    #[error("Cache error: {0}")]
    CacheError(String),

    /// API rate limit exceeded
    #[error("API rate limit exceeded")]
    RateLimitExceeded,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::EventMetadata;

    #[tokio::test]
    async fn test_enricher_creation() {
        let config = EnrichmentConfig::default();
        let enricher = EventEnricher::new(config);

        let stats = enricher.get_cache_stats();
        assert_eq!(stats.token_cache_size, 0);
        assert_eq!(stats.price_cache_size, 0);
    }

    #[tokio::test]
    async fn test_token_address_extraction() {
        let config = EnrichmentConfig::default();
        let enricher = EventEnricher::new(config);

        let metadata = EventMetadata::default();
        let mut event = ZeroCopyEvent::new_owned(metadata, vec![]);

        // Add JSON data with token addresses
        let json = serde_json::json!({
            "input_mint": "So11111111111111111111111111111111111111112",
            "output_mint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"
        });
        event.set_json_data(json);

        let addresses = enricher.extract_token_addresses(&event);
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

        std::thread::sleep(Duration::from_millis(2));
        assert!(entry.is_expired());
    }
}
