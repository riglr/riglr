//! Entity extraction and relationship mining from text documents.
//!
//! This module provides production-grade entity extraction capabilities for blockchain-related text,
//! identifying wallets, tokens, protocols, amounts, and relationships between entities.

use crate::document::{
    AmountMention, AmountType, EntityMention, EntityType, ExtractedEntities, RelationshipMention,
    RelationshipType,
};
use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::{HashMap, HashSet};
use tracing::{debug, info};

/// Production-grade entity extractor for blockchain text analysis
#[derive(Debug)]
pub struct EntityExtractor {
    /// Known protocol names for recognition
    protocol_patterns: HashMap<String, Vec<String>>,
    /// Token symbol patterns
    token_patterns: HashMap<String, Vec<String>>,
    /// Blockchain network patterns
    chain_patterns: HashMap<String, Vec<String>>,
    /// Compiled regex patterns for performance
    #[allow(dead_code)]
    regex_cache: HashMap<String, Regex>,
}

/// Ethereum address regex pattern (exactly 40 hex chars, not part of longer hash)
static ETH_ADDRESS_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"0x[a-fA-F0-9]{40}\b").expect("Invalid Ethereum address regex"));

/// Solana address regex pattern (Base58 without 0, O, I, l)
static SOL_ADDRESS_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\b[123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz]{32,44}\b")
        .expect("Invalid Solana address regex")
});

/// Amount pattern (e.g., "123.45 ETH", "$1,234.56", "1K USDC", "$1.2B")
static AMOUNT_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\$?[0-9]+(?:[.,][0-9]+)*[KMBkmb]?(?:\s+[A-Z]{2,10})?")
        .expect("Invalid amount regex")
});

/// Transaction hash patterns
#[allow(dead_code)]
static TX_HASH_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"0x[a-fA-F0-9]{64}").expect("Invalid transaction hash regex"));

impl EntityExtractor {
    /// Initialize known patterns for entity recognition
    fn initialize_patterns(&mut self) {
        // DeFi Protocol patterns
        self.protocol_patterns.insert(
            "uniswap".to_string(),
            vec![
                "uniswap".to_string(),
                "uni".to_string(),
                "uniswap v2".to_string(),
                "uniswap v3".to_string(),
            ],
        );

        self.protocol_patterns.insert(
            "aave".to_string(),
            vec![
                "aave".to_string(),
                "aave protocol".to_string(),
                "aave lending".to_string(),
            ],
        );

        self.protocol_patterns.insert(
            "compound".to_string(),
            vec!["compound".to_string(), "compound finance".to_string()],
        );

        self.protocol_patterns.insert(
            "jupiter".to_string(),
            vec![
                "jupiter".to_string(),
                "jupiter aggregator".to_string(),
                "jup".to_string(),
            ],
        );

        self.protocol_patterns.insert(
            "solend".to_string(),
            vec!["solend".to_string(), "solend protocol".to_string()],
        );

        // Token patterns
        self.token_patterns.insert(
            "ethereum".to_string(),
            vec![
                "eth".to_string(),
                "ethereum".to_string(),
                "ether".to_string(),
            ],
        );

        self.token_patterns.insert(
            "bitcoin".to_string(),
            vec!["btc".to_string(), "bitcoin".to_string()],
        );

        self.token_patterns.insert(
            "usdc".to_string(),
            vec!["usdc".to_string(), "usd coin".to_string()],
        );

        self.token_patterns.insert(
            "usdt".to_string(),
            vec!["usdt".to_string(), "tether".to_string()],
        );

        self.token_patterns.insert(
            "solana".to_string(),
            vec!["sol".to_string(), "solana".to_string()],
        );

        // Chain patterns
        self.chain_patterns.insert(
            "ethereum".to_string(),
            vec![
                "ethereum".to_string(),
                "eth mainnet".to_string(),
                "ethereum mainnet".to_string(),
            ],
        );

        self.chain_patterns.insert(
            "solana".to_string(),
            vec!["solana".to_string(), "solana mainnet".to_string()],
        );

        self.chain_patterns.insert(
            "polygon".to_string(),
            vec![
                "polygon".to_string(),
                "matic".to_string(),
                "polygon pos".to_string(),
            ],
        );

        self.chain_patterns.insert(
            "arbitrum".to_string(),
            vec![
                "arbitrum".to_string(),
                "arbitrum one".to_string(),
                "arb".to_string(),
            ],
        );

        debug!(
            "Initialized entity extraction patterns for {} protocols, {} tokens, {} chains",
            self.protocol_patterns.len(),
            self.token_patterns.len(),
            self.chain_patterns.len()
        );
    }

    /// Extract all entities and relationships from a text document
    pub fn extract(&self, text: &str) -> ExtractedEntities {
        debug!("Extracting entities from text ({} chars)", text.len());

        let text_lower = text.to_lowercase();

        // Extract different entity types
        let wallets = self.extract_wallet_addresses(text);
        let tokens = self.extract_tokens(&text_lower);
        let protocols = self.extract_protocols(&text_lower);
        let chains = self.extract_chains(&text_lower);
        let amounts = self.extract_amounts(text);
        let relationships = self.extract_relationships(text, &wallets, &tokens, &protocols);

        info!("Extracted {} wallets, {} tokens, {} protocols, {} chains, {} amounts, {} relationships",
              wallets.len(), tokens.len(), protocols.len(), chains.len(), amounts.len(), relationships.len());

        ExtractedEntities {
            wallets,
            tokens,
            protocols,
            chains,
            amounts,
            relationships,
        }
    }

    /// Extract wallet addresses from text
    fn extract_wallet_addresses(&self, text: &str) -> Vec<EntityMention> {
        let mut wallets = Vec::new();

        // Extract Ethereum addresses
        for mat in ETH_ADDRESS_REGEX.find_iter(text) {
            let address = mat.as_str().to_string();
            let canonical = address.to_lowercase();

            wallets.push(EntityMention {
                text: address.clone(),
                canonical,
                entity_type: EntityType::Wallet,
                confidence: 0.95, // High confidence for valid address format
                span: (mat.start(), mat.end()),
                properties: {
                    let mut props = HashMap::new();
                    props.insert("chain".to_string(), "ethereum".to_string());
                    props.insert("format".to_string(), "ethereum".to_string());
                    props
                },
            });
        }

        // Extract Solana addresses (more complex validation needed)
        for mat in SOL_ADDRESS_REGEX.find_iter(text) {
            let address = mat.as_str().to_string();

            // Basic validation for Solana addresses
            if self.is_likely_solana_address(&address) {
                wallets.push(EntityMention {
                    text: address.clone(),
                    canonical: address.clone(),
                    entity_type: EntityType::Wallet,
                    confidence: 0.85, // Slightly lower confidence due to format ambiguity
                    span: (mat.start(), mat.end()),
                    properties: {
                        let mut props = HashMap::new();
                        props.insert("chain".to_string(), "solana".to_string());
                        props.insert("format".to_string(), "base58".to_string());
                        props
                    },
                });
            }
        }

        debug!("Extracted {} wallet addresses", wallets.len());
        wallets
    }

    fn extract_tokens(&self, text: &str) -> Vec<EntityMention> {
        let mut tokens = Vec::new();
        let mut seen = HashSet::new();

        for (canonical_name, patterns) in &self.token_patterns {
            for pattern in patterns {
                let positions = self.find_pattern_positions(text, pattern);
                for (start, end) in positions {
                    if seen.insert(canonical_name.clone()) {
                        tokens.push(EntityMention {
                            text: text[start..end].to_string(),
                            canonical: canonical_name.clone(),
                            entity_type: EntityType::Token,
                            confidence: 0.90,
                            span: (start, end),
                            properties: {
                                let mut props = HashMap::new();
                                props.insert("symbol".to_string(), canonical_name.to_uppercase());
                                props
                            },
                        });
                    }
                }
            }
        }

        debug!("Extracted {} token mentions", tokens.len());
        tokens
    }

    /// Extract protocol mentions from text
    fn extract_protocols(&self, text: &str) -> Vec<EntityMention> {
        let mut protocols = Vec::new();
        let mut seen = HashSet::new();

        for (canonical_name, patterns) in &self.protocol_patterns {
            for pattern in patterns {
                let positions = self.find_pattern_positions(text, pattern);
                for (start, end) in positions {
                    if seen.insert(canonical_name.clone()) {
                        protocols.push(EntityMention {
                            text: text[start..end].to_string(),
                            canonical: canonical_name.clone(),
                            entity_type: EntityType::Protocol,
                            confidence: 0.88,
                            span: (start, end),
                            properties: {
                                let mut props = HashMap::new();
                                props.insert("category".to_string(), "defi".to_string());
                                props
                            },
                        });
                    }
                }
            }
        }

        debug!("Extracted {} protocol mentions", protocols.len());
        protocols
    }

    /// Extract blockchain network mentions
    fn extract_chains(&self, text: &str) -> Vec<EntityMention> {
        let mut chains = Vec::new();
        let mut seen = HashSet::new();

        for (canonical_name, patterns) in &self.chain_patterns {
            for pattern in patterns {
                let positions = self.find_pattern_positions(text, pattern);
                for (start, end) in positions {
                    if seen.insert(canonical_name.clone()) {
                        chains.push(EntityMention {
                            text: text[start..end].to_string(),
                            canonical: canonical_name.clone(),
                            entity_type: EntityType::Chain,
                            confidence: 0.92,
                            span: (start, end),
                            properties: {
                                let mut props = HashMap::new();
                                props.insert("layer".to_string(), "l1".to_string());
                                props
                            },
                        });
                    }
                }
            }
        }

        debug!("Extracted {} chain mentions", chains.len());
        chains
    }

    /// Extract numerical amounts and values
    fn extract_amounts(&self, text: &str) -> Vec<AmountMention> {
        let mut amounts = Vec::new();

        for mat in AMOUNT_REGEX.find_iter(text) {
            let full_match = mat.as_str();
            let (value_str, unit) = self.parse_amount_match(full_match);

            if let Some(value) = self.parse_numeric_value(&value_str) {
                let amount_type = self.classify_amount_type(full_match, text);

                amounts.push(AmountMention {
                    text: full_match.to_string(),
                    value,
                    unit,
                    amount_type,
                    span: (mat.start(), mat.end()),
                });
            }
        }

        debug!("Extracted {} amount mentions", amounts.len());
        amounts
    }

    /// Extract relationships between entities
    fn extract_relationships(
        &self,
        text: &str,
        wallets: &[EntityMention],
        tokens: &[EntityMention],
        protocols: &[EntityMention],
    ) -> Vec<RelationshipMention> {
        let mut relationships = Vec::new();
        let text_lower = text.to_lowercase();

        // Look for common relationship patterns
        let relationship_patterns = vec![
            (
                r"(\w+)\s+(swapped|traded|exchanged)\s+.*\s+(for|to)\s+(\w+)",
                RelationshipType::Transferred,
            ),
            (
                r"(\w+)\s+(used|interacted with|called)\s+(\w+)",
                RelationshipType::Interacted,
            ),
            (r"(\w+)\s+(holds|owns|has)\s+(\w+)", RelationshipType::Holds),
            (
                r"(\w+)\s+(deployed on|built on|runs on)\s+(\w+)",
                RelationshipType::DeployedOn,
            ),
        ];

        for (pattern, rel_type) in relationship_patterns {
            let regex_result = Regex::new(pattern);
            if let Ok(regex) = regex_result {
                for mat in regex.find_iter(&text_lower) {
                    let context = mat.as_str().to_string();

                    // This is a simplified relationship extraction
                    // In production, you'd use more sophisticated NLP
                    if let Some(from_entity) =
                        self.find_nearby_entity(&context, wallets, tokens, protocols)
                    {
                        if let Some(to_entity) =
                            self.find_nearby_entity(&context, tokens, protocols, &[])
                        {
                            relationships.push(RelationshipMention {
                                from_entity: from_entity.canonical.clone(),
                                to_entity: to_entity.canonical.clone(),
                                relationship_type: rel_type.clone(),
                                confidence: 0.75,
                                context: context.clone(),
                            });
                        }
                    }
                }
            }
        }

        debug!("Extracted {} relationships", relationships.len());
        relationships
    }

    /// Helper function to find pattern positions in text
    fn find_pattern_positions(&self, text: &str, pattern: &str) -> Vec<(usize, usize)> {
        let mut positions = Vec::new();

        // Early return for empty pattern to avoid infinite loop
        if pattern.is_empty() {
            return positions;
        }

        let pattern_lower = pattern.to_lowercase();
        let text_lower = text.to_lowercase();

        let mut start = 0;
        while let Some(pos) = text_lower[start..].find(&pattern_lower) {
            let actual_start = start + pos;
            let actual_end = actual_start + pattern.len();
            positions.push((actual_start, actual_end));
            start = actual_end;
        }

        positions
    }

    /// Check if a string is likely a Solana address
    fn is_likely_solana_address(&self, address: &str) -> bool {
        // Basic checks for Solana address format
        address.len() >= 32
            && address.len() <= 44
            && !address.contains("0x")
            && address
                .chars()
                .all(|c| c.is_ascii_alphanumeric() && c != '0' && c != 'O' && c != 'I' && c != 'l')
    }

    /// Parse amount text into value and unit
    fn parse_amount_match(&self, text: &str) -> (String, Option<String>) {
        // Check if there's a space separating value and unit
        let parts: Vec<&str> = text.split_whitespace().collect();
        if parts.len() >= 2 {
            (parts[0].to_string(), Some(parts[1].to_string()))
        } else {
            // No space, so the whole thing is the value (e.g., "$1.2B")
            (text.to_string(), None)
        }
    }

    /// Parse numeric value from text (handling K, M, B suffixes)
    fn parse_numeric_value(&self, value_str: &str) -> Option<f64> {
        let cleaned = value_str.replace(['$', ','], "");

        if let Some(last_char) = cleaned.chars().last() {
            let (num_part, multiplier) = match last_char {
                'K' | 'k' => (&cleaned[..cleaned.len() - 1], 1000.0),
                'M' | 'm' => (&cleaned[..cleaned.len() - 1], 1_000_000.0),
                'B' | 'b' => (&cleaned[..cleaned.len() - 1], 1_000_000_000.0),
                _ => (cleaned.as_str(), 1.0),
            };

            if let Ok(base_value) = num_part.parse::<f64>() {
                Some(base_value * multiplier)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Classify the type of amount based on context
    fn classify_amount_type(&self, amount_text: &str, context: &str) -> AmountType {
        let context_lower = context.to_lowercase();
        let amount_lower = amount_text.to_lowercase();

        // Check context-specific keywords first, before generic $ check
        if context_lower.contains("balance") || context_lower.contains("holds") {
            AmountType::Balance
        } else if context_lower.contains("fee") || context_lower.contains("gas") {
            AmountType::Fee
        } else if context_lower.contains("volume") || context_lower.contains("trading") {
            AmountType::Volume
        } else if context_lower.contains("market cap") || context_lower.contains("mcap") {
            AmountType::MarketCap
        } else if context_lower.contains("price")
            || context_lower.contains("worth")
            || amount_lower.contains('$')
        {
            AmountType::Price
        } else {
            AmountType::Other("unknown".to_string())
        }
    }

    /// Find nearby entity mentions in context
    fn find_nearby_entity<'a>(
        &self,
        context: &str,
        entities: &'a [EntityMention],
        alt1: &'a [EntityMention],
        alt2: &'a [EntityMention],
    ) -> Option<&'a EntityMention> {
        // Simple implementation - find first entity that appears in context
        entities
            .iter()
            .chain(alt1.iter())
            .chain(alt2.iter())
            .find(|&entity| context.to_lowercase().contains(&entity.canonical))
            .map(|v| v as _)
    }
}

impl Default for EntityExtractor {
    fn default() -> Self {
        let mut extractor = Self {
            protocol_patterns: HashMap::new(),
            token_patterns: HashMap::new(),
            chain_patterns: HashMap::new(),
            regex_cache: HashMap::new(),
        };
        extractor.initialize_patterns();
        extractor
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::{AmountType, EntityType};

    fn create_test_extractor() -> EntityExtractor {
        EntityExtractor::default()
    }

    #[test]
    fn test_default_creates_extractor_with_patterns() {
        let extractor = EntityExtractor::default();
        assert!(!extractor.protocol_patterns.is_empty());
        assert!(!extractor.token_patterns.is_empty());
        assert!(!extractor.chain_patterns.is_empty());
    }

    #[test]
    fn test_initialize_patterns_populates_all_maps() {
        let mut extractor = EntityExtractor {
            protocol_patterns: HashMap::new(),
            token_patterns: HashMap::new(),
            chain_patterns: HashMap::new(),
            regex_cache: HashMap::new(),
        };

        assert!(extractor.protocol_patterns.is_empty());
        assert!(extractor.token_patterns.is_empty());
        assert!(extractor.chain_patterns.is_empty());

        extractor.initialize_patterns();

        assert!(!extractor.protocol_patterns.is_empty());
        assert!(!extractor.token_patterns.is_empty());
        assert!(!extractor.chain_patterns.is_empty());

        // Verify specific patterns exist
        assert!(extractor.protocol_patterns.contains_key("uniswap"));
        assert!(extractor.protocol_patterns.contains_key("aave"));
        assert!(extractor.protocol_patterns.contains_key("compound"));
        assert!(extractor.protocol_patterns.contains_key("jupiter"));
        assert!(extractor.protocol_patterns.contains_key("solend"));

        assert!(extractor.token_patterns.contains_key("ethereum"));
        assert!(extractor.token_patterns.contains_key("bitcoin"));
        assert!(extractor.token_patterns.contains_key("usdc"));
        assert!(extractor.token_patterns.contains_key("usdt"));
        assert!(extractor.token_patterns.contains_key("solana"));

        assert!(extractor.chain_patterns.contains_key("ethereum"));
        assert!(extractor.chain_patterns.contains_key("solana"));
        assert!(extractor.chain_patterns.contains_key("polygon"));
        assert!(extractor.chain_patterns.contains_key("arbitrum"));
    }

    #[test]
    fn test_extract_empty_text_returns_empty_entities() {
        let extractor = create_test_extractor();
        let result = extractor.extract("");

        assert!(result.wallets.is_empty());
        assert!(result.tokens.is_empty());
        assert!(result.protocols.is_empty());
        assert!(result.chains.is_empty());
        assert!(result.amounts.is_empty());
        assert!(result.relationships.is_empty());
    }

    #[test]
    fn test_extract_with_valid_ethereum_address() {
        let extractor = create_test_extractor();
        let text = "Send ETH to 0x742d35cc6645c0532eb5d65c8092a7b8ab27d1de";
        let result = extractor.extract(text);

        assert_eq!(result.wallets.len(), 1);
        let wallet = &result.wallets[0];
        assert_eq!(wallet.text, "0x742d35cc6645c0532eb5d65c8092a7b8ab27d1de");
        assert_eq!(
            wallet.canonical,
            "0x742d35cc6645c0532eb5d65c8092a7b8ab27d1de"
        );
        assert_eq!(wallet.entity_type, EntityType::Wallet);
        assert_eq!(wallet.confidence, 0.95);
        assert_eq!(
            wallet.properties.get("chain"),
            Some(&"ethereum".to_string())
        );
        assert_eq!(
            wallet.properties.get("format"),
            Some(&"ethereum".to_string())
        );
    }

    #[test]
    fn test_extract_with_valid_solana_address() {
        let extractor = create_test_extractor();
        let text = "Transfer SOL to 9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM";
        let result = extractor.extract(text);

        assert_eq!(result.wallets.len(), 1);
        let wallet = &result.wallets[0];
        assert_eq!(wallet.text, "9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM");
        assert_eq!(
            wallet.canonical,
            "9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM"
        );
        assert_eq!(wallet.entity_type, EntityType::Wallet);
        assert_eq!(wallet.confidence, 0.85);
        assert_eq!(wallet.properties.get("chain"), Some(&"solana".to_string()));
        assert_eq!(wallet.properties.get("format"), Some(&"base58".to_string()));
    }

    #[test]
    fn test_extract_tokens_finds_known_tokens() {
        let extractor = create_test_extractor();
        let text = "I swapped ETH for USDC on ethereum";
        let result = extractor.extract(text);

        assert!(!result.tokens.is_empty());
        let token_names: Vec<&str> = result.tokens.iter().map(|t| t.canonical.as_str()).collect();
        assert!(token_names.contains(&"ethereum"));
        assert!(token_names.contains(&"usdc"));

        for token in &result.tokens {
            assert_eq!(token.entity_type, EntityType::Token);
            assert_eq!(token.confidence, 0.90);
            assert!(token.properties.contains_key("symbol"));
        }
    }

    #[test]
    fn test_extract_protocols_finds_known_protocols() {
        let extractor = create_test_extractor();
        let text = "I used Uniswap and Aave for DeFi trading";
        let result = extractor.extract(text);

        assert!(!result.protocols.is_empty());
        let protocol_names: Vec<&str> = result
            .protocols
            .iter()
            .map(|p| p.canonical.as_str())
            .collect();
        assert!(protocol_names.contains(&"uniswap"));
        assert!(protocol_names.contains(&"aave"));

        for protocol in &result.protocols {
            assert_eq!(protocol.entity_type, EntityType::Protocol);
            assert_eq!(protocol.confidence, 0.88);
            assert_eq!(
                protocol.properties.get("category"),
                Some(&"defi".to_string())
            );
        }
    }

    #[test]
    fn test_extract_chains_finds_known_chains() {
        let extractor = create_test_extractor();
        let text = "Deploy on Ethereum and Solana networks";
        let result = extractor.extract(text);

        assert!(!result.chains.is_empty());
        let chain_names: Vec<&str> = result.chains.iter().map(|c| c.canonical.as_str()).collect();
        assert!(chain_names.contains(&"ethereum"));
        assert!(chain_names.contains(&"solana"));

        for chain in &result.chains {
            assert_eq!(chain.entity_type, EntityType::Chain);
            assert_eq!(chain.confidence, 0.92);
            assert_eq!(chain.properties.get("layer"), Some(&"l1".to_string()));
        }
    }

    #[test]
    fn test_extract_amounts_with_various_formats() {
        let extractor = create_test_extractor();
        let text = "Price is $1,234.56 and volume is 1.5K ETH, market cap 2.1B";
        let result = extractor.extract(text);

        assert!(!result.amounts.is_empty());

        // Should find multiple amounts
        let amount_texts: Vec<&str> = result.amounts.iter().map(|a| a.text.as_str()).collect();
        assert!(amount_texts.iter().any(|&text| text.contains("1,234.56")));
        assert!(amount_texts.iter().any(|&text| text.contains("1.5K")));
        assert!(amount_texts.iter().any(|&text| text.contains("2.1B")));
    }

    #[test]
    fn test_is_likely_solana_address_valid_cases() {
        let extractor = create_test_extractor();

        // Valid Solana addresses
        assert!(extractor.is_likely_solana_address("9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM"));
        assert!(extractor.is_likely_solana_address("SoLEao8wTzSfqhuou8dpYXGcoyGVDEBWwgEBpAEpDWe"));

        // Edge cases - minimum length
        assert!(extractor.is_likely_solana_address("ABCDEFGHJKMNPQRSTUVWXYZabcdefghjk"));

        // Maximum length (44 chars)
        assert!(extractor.is_likely_solana_address("ABCDEFGHJKMNPQRSTUVWXYZabcdefghjkmnpqrstuv"));
    }

    #[test]
    fn test_is_likely_solana_address_invalid_cases() {
        let extractor = create_test_extractor();

        // Too short
        assert!(!extractor.is_likely_solana_address("short"));
        assert!(!extractor.is_likely_solana_address("ABCDEFGHJKMNPQRSTUVWXYZabcdefgh")); // 31 chars, too short

        // Too long (45 chars)
        assert!(
            !extractor.is_likely_solana_address("ABCDEFGHJKMNPQRSTUVWXYZabcdefghjkmnpqrstuvwxy")
        );

        // Contains 0x (Ethereum-like)
        assert!(!extractor.is_likely_solana_address("0x742d35cc6645c0532eb5d65c8092a7b8ab27d1de"));

        // Contains excluded characters (0, O, I, l)
        assert!(!extractor.is_likely_solana_address("9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYt0WWM"));
        assert!(!extractor.is_likely_solana_address("9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVLOzYtAWWM"));
        assert!(!extractor.is_likely_solana_address("9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVLIzYtAWWM"));
        assert!(!extractor.is_likely_solana_address("9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVLlzYtAWWM"));

        // Non-alphanumeric characters
        assert!(!extractor.is_likely_solana_address("9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL-zYtAWWM"));
    }

    #[test]
    fn test_find_pattern_positions_multiple_matches() {
        let extractor = create_test_extractor();
        let text = "eth trading with eth and more eth";
        let positions = extractor.find_pattern_positions(text, "eth");

        assert_eq!(positions.len(), 3);
        assert_eq!(positions[0], (0, 3));
        assert_eq!(positions[1], (17, 20));
        assert_eq!(positions[2], (30, 33));
    }

    #[test]
    fn test_find_pattern_positions_no_matches() {
        let extractor = create_test_extractor();
        let text = "bitcoin only here";
        let positions = extractor.find_pattern_positions(text, "ethereum");

        assert!(positions.is_empty());
    }

    #[test]
    fn test_find_pattern_positions_case_insensitive() {
        let extractor = create_test_extractor();
        let text = "ETH and Eth and eth";
        let positions = extractor.find_pattern_positions(text, "eth");

        assert_eq!(positions.len(), 3);
    }

    #[test]
    fn test_parse_amount_match_with_space() {
        let extractor = create_test_extractor();
        let (value, unit) = extractor.parse_amount_match("123.45 ETH");

        assert_eq!(value, "123.45");
        assert_eq!(unit, Some("ETH".to_string()));
    }

    #[test]
    fn test_parse_amount_match_without_space() {
        let extractor = create_test_extractor();
        let (value, unit) = extractor.parse_amount_match("$1,234.56");

        assert_eq!(value, "$1,234.56");
        assert_eq!(unit, None);
    }

    #[test]
    fn test_parse_amount_match_multiple_spaces() {
        let extractor = create_test_extractor();
        let (value, unit) = extractor.parse_amount_match("100   USD");

        assert_eq!(value, "100");
        assert_eq!(unit, Some("USD".to_string()));
    }

    #[test]
    fn test_parse_numeric_value_basic_numbers() {
        let extractor = create_test_extractor();

        assert_eq!(extractor.parse_numeric_value("123"), Some(123.0));
        assert_eq!(extractor.parse_numeric_value("123.45"), Some(123.45));
        assert_eq!(extractor.parse_numeric_value("0"), Some(0.0));
    }

    #[test]
    fn test_parse_numeric_value_with_k_suffix() {
        let extractor = create_test_extractor();

        assert_eq!(extractor.parse_numeric_value("1K"), Some(1000.0));
        assert_eq!(extractor.parse_numeric_value("1k"), Some(1000.0));
        assert_eq!(extractor.parse_numeric_value("1.5K"), Some(1500.0));
        assert_eq!(extractor.parse_numeric_value("0.5k"), Some(500.0));
    }

    #[test]
    fn test_parse_numeric_value_with_m_suffix() {
        let extractor = create_test_extractor();

        assert_eq!(extractor.parse_numeric_value("1M"), Some(1_000_000.0));
        assert_eq!(extractor.parse_numeric_value("1m"), Some(1_000_000.0));
        assert_eq!(extractor.parse_numeric_value("2.5M"), Some(2_500_000.0));
    }

    #[test]
    fn test_parse_numeric_value_with_b_suffix() {
        let extractor = create_test_extractor();

        assert_eq!(extractor.parse_numeric_value("1B"), Some(1_000_000_000.0));
        assert_eq!(extractor.parse_numeric_value("1b"), Some(1_000_000_000.0));
        assert_eq!(extractor.parse_numeric_value("1.2B"), Some(1_200_000_000.0));
    }

    #[test]
    fn test_parse_numeric_value_with_dollar_and_comma() {
        let extractor = create_test_extractor();

        assert_eq!(extractor.parse_numeric_value("$1,234"), Some(1234.0));
        assert_eq!(extractor.parse_numeric_value("$1,234.56"), Some(1234.56));
        assert_eq!(
            extractor.parse_numeric_value("$1,000,000"),
            Some(1_000_000.0)
        );
    }

    #[test]
    fn test_parse_numeric_value_invalid_input() {
        let extractor = create_test_extractor();

        assert_eq!(extractor.parse_numeric_value("abc"), None);
        assert_eq!(extractor.parse_numeric_value(""), None);
        assert_eq!(extractor.parse_numeric_value("$"), None);
        assert_eq!(extractor.parse_numeric_value("1.2.3"), None);
        assert_eq!(extractor.parse_numeric_value("K"), None);
    }

    #[test]
    fn test_classify_amount_type_balance() {
        let extractor = create_test_extractor();

        assert_eq!(
            extractor.classify_amount_type("100 ETH", "My balance is 100 ETH"),
            AmountType::Balance
        );
        assert_eq!(
            extractor.classify_amount_type("50 USDC", "Account holds 50 USDC"),
            AmountType::Balance
        );
    }

    #[test]
    fn test_classify_amount_type_fee() {
        let extractor = create_test_extractor();

        assert_eq!(
            extractor.classify_amount_type("0.001 ETH", "Transaction fee 0.001 ETH"),
            AmountType::Fee
        );
        assert_eq!(
            extractor.classify_amount_type("20 gwei", "Gas price is 20 gwei"),
            AmountType::Fee
        );
    }

    #[test]
    fn test_classify_amount_type_volume() {
        let extractor = create_test_extractor();

        assert_eq!(
            extractor.classify_amount_type("1M USDC", "Daily volume 1M USDC"),
            AmountType::Volume
        );
        assert_eq!(
            extractor.classify_amount_type("500K", "Trading volume reached 500K"),
            AmountType::Volume
        );
    }

    #[test]
    fn test_classify_amount_type_market_cap() {
        let extractor = create_test_extractor();

        assert_eq!(
            extractor.classify_amount_type("1B", "Market cap is 1B"),
            AmountType::MarketCap
        );
        assert_eq!(
            extractor.classify_amount_type("500M", "The mcap reached 500M"),
            AmountType::MarketCap
        );
    }

    #[test]
    fn test_classify_amount_type_price() {
        let extractor = create_test_extractor();

        assert_eq!(
            extractor.classify_amount_type("$50", "ETH price is $50"),
            AmountType::Price
        );
        assert_eq!(
            extractor.classify_amount_type("$1000", "Token worth $1000"),
            AmountType::Price
        );
        assert_eq!(
            extractor.classify_amount_type("$100", "Cost $100"),
            AmountType::Price
        );
    }

    #[test]
    fn test_classify_amount_type_other() {
        let extractor = create_test_extractor();

        assert_eq!(
            extractor.classify_amount_type("100", "Random number 100"),
            AmountType::Other("unknown".to_string())
        );
        assert_eq!(
            extractor.classify_amount_type("42", "The answer is 42"),
            AmountType::Other("unknown".to_string())
        );
    }

    #[test]
    fn test_find_nearby_entity_found_in_first_list() {
        let extractor = create_test_extractor();

        let entity1 = EntityMention {
            text: "ETH".to_string(),
            canonical: "ethereum".to_string(),
            entity_type: EntityType::Token,
            confidence: 0.9,
            span: (0, 3),
            properties: HashMap::new(),
        };

        let entities = vec![entity1];
        let result = extractor.find_nearby_entity("trading ethereum today", &entities, &[], &[]);

        assert!(result.is_some());
        assert_eq!(result.unwrap().canonical, "ethereum");
    }

    #[test]
    fn test_find_nearby_entity_found_in_alt_lists() {
        let extractor = create_test_extractor();

        let entity1 = EntityMention {
            text: "USDC".to_string(),
            canonical: "usdc".to_string(),
            entity_type: EntityType::Token,
            confidence: 0.9,
            span: (0, 4),
            properties: HashMap::new(),
        };

        let alt1 = vec![entity1];
        let result = extractor.find_nearby_entity("swapping for usdc", &[], &alt1, &[]);

        assert!(result.is_some());
        assert_eq!(result.unwrap().canonical, "usdc");
    }

    #[test]
    fn test_find_nearby_entity_not_found() {
        let extractor = create_test_extractor();

        let entity1 = EntityMention {
            text: "BTC".to_string(),
            canonical: "bitcoin".to_string(),
            entity_type: EntityType::Token,
            confidence: 0.9,
            span: (0, 3),
            properties: HashMap::new(),
        };

        let entities = vec![entity1];
        let result = extractor.find_nearby_entity("trading ethereum today", &entities, &[], &[]);

        assert!(result.is_none());
    }

    #[test]
    fn test_extract_wallet_addresses_empty_text() {
        let extractor = create_test_extractor();
        let wallets = extractor.extract_wallet_addresses("");
        assert!(wallets.is_empty());
    }

    #[test]
    fn test_extract_wallet_addresses_no_addresses() {
        let extractor = create_test_extractor();
        let wallets = extractor.extract_wallet_addresses("Just some random text without addresses");
        assert!(wallets.is_empty());
    }

    #[test]
    fn test_extract_wallet_addresses_mixed_addresses() {
        let extractor = create_test_extractor();
        let text = "Send ETH to 0x742d35cc6645c0532eb5d65c8092a7b8ab27d1de and SOL to 9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM";
        let wallets = extractor.extract_wallet_addresses(text);

        assert_eq!(wallets.len(), 2);

        // Find Ethereum address
        let eth_wallet = wallets
            .iter()
            .find(|w| w.canonical.starts_with("0x"))
            .unwrap();
        assert_eq!(
            eth_wallet.properties.get("chain"),
            Some(&"ethereum".to_string())
        );

        // Find Solana address
        let sol_wallet = wallets
            .iter()
            .find(|w| !w.canonical.starts_with("0x"))
            .unwrap();
        assert_eq!(
            sol_wallet.properties.get("chain"),
            Some(&"solana".to_string())
        );
    }

    #[test]
    fn test_extract_wallet_addresses_invalid_solana() {
        let extractor = create_test_extractor();
        // This looks like Solana format but has excluded characters
        let text = "Invalid address: 9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL0zYtAWWM";
        let wallets = extractor.extract_wallet_addresses(text);

        assert!(wallets.is_empty());
    }

    #[test]
    fn test_extract_tokens_empty_text() {
        let extractor = create_test_extractor();
        let tokens = extractor.extract_tokens("");
        assert!(tokens.is_empty());
    }

    #[test]
    fn test_extract_tokens_no_known_tokens() {
        let extractor = create_test_extractor();
        let tokens = extractor.extract_tokens("trading some unknown coin");
        assert!(tokens.is_empty());
    }

    #[test]
    fn test_extract_tokens_duplicate_prevention() {
        let extractor = create_test_extractor();
        let tokens = extractor.extract_tokens("eth and ethereum and eth again");

        // Should only find one instance due to duplicate prevention
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].canonical, "ethereum");
    }

    #[test]
    fn test_extract_protocols_empty_text() {
        let extractor = create_test_extractor();
        let protocols = extractor.extract_protocols("");
        assert!(protocols.is_empty());
    }

    #[test]
    fn test_extract_protocols_no_known_protocols() {
        let extractor = create_test_extractor();
        let protocols = extractor.extract_protocols("using some unknown protocol");
        assert!(protocols.is_empty());
    }

    #[test]
    fn test_extract_protocols_duplicate_prevention() {
        let extractor = create_test_extractor();
        let protocols = extractor.extract_protocols("uniswap and uni and uniswap v2");

        // Should only find one instance due to duplicate prevention
        assert_eq!(protocols.len(), 1);
        assert_eq!(protocols[0].canonical, "uniswap");
    }

    #[test]
    fn test_extract_chains_empty_text() {
        let extractor = create_test_extractor();
        let chains = extractor.extract_chains("");
        assert!(chains.is_empty());
    }

    #[test]
    fn test_extract_chains_no_known_chains() {
        let extractor = create_test_extractor();
        let chains = extractor.extract_chains("deploying on unknown network");
        assert!(chains.is_empty());
    }

    #[test]
    fn test_extract_chains_duplicate_prevention() {
        let extractor = create_test_extractor();
        let chains = extractor.extract_chains("ethereum and ethereum mainnet and eth mainnet");

        // Should only find one instance due to duplicate prevention
        assert_eq!(chains.len(), 1);
        assert_eq!(chains[0].canonical, "ethereum");
    }

    #[test]
    fn test_extract_amounts_empty_text() {
        let extractor = create_test_extractor();
        let amounts = extractor.extract_amounts("");
        assert!(amounts.is_empty());
    }

    #[test]
    fn test_extract_amounts_no_amounts() {
        let extractor = create_test_extractor();
        let amounts = extractor.extract_amounts("Just some text without numbers");
        assert!(amounts.is_empty());
    }

    #[test]
    fn test_extract_amounts_invalid_numeric_values() {
        let extractor = create_test_extractor();
        // These should match the regex but fail numeric parsing
        let amounts = extractor.extract_amounts("Invalid amounts: abc.def xyz");
        assert!(amounts.is_empty());
    }

    #[test]
    fn test_extract_relationships_empty_inputs() {
        let extractor = create_test_extractor();
        let relationships = extractor.extract_relationships("", &[], &[], &[]);
        assert!(relationships.is_empty());
    }

    #[test]
    fn test_extract_relationships_no_patterns() {
        let extractor = create_test_extractor();
        let relationships = extractor.extract_relationships("just random text", &[], &[], &[]);
        assert!(relationships.is_empty());
    }

    #[test]
    fn test_static_regex_patterns() {
        // Test that static regex patterns compile and work
        assert!(ETH_ADDRESS_REGEX.is_match("0x742d35cc6645c0532eb5d65c8092a7b8ab27d1de"));
        assert!(!ETH_ADDRESS_REGEX.is_match("0x742d35cc6645c0532eb5d65c8092a7b8ab27d1def")); // Too long
        assert!(!ETH_ADDRESS_REGEX.is_match("742d35cc6645c0532eb5d65c8092a7b8ab27d1de")); // Missing 0x

        assert!(SOL_ADDRESS_REGEX.is_match("9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM"));
        assert!(!SOL_ADDRESS_REGEX.is_match("0x742d35cc6645c0532eb5d65c8092a7b8ab27d1de")); // Ethereum format

        assert!(AMOUNT_REGEX.is_match("123.45"));
        assert!(AMOUNT_REGEX.is_match("$1,234.56"));
        assert!(AMOUNT_REGEX.is_match("1K ETH"));
        assert!(AMOUNT_REGEX.is_match("1.2B"));

        assert!(TX_HASH_REGEX
            .is_match("0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"));
        assert!(!TX_HASH_REGEX.is_match("0x742d35cc6645c0532eb5d65c8092a7b8ab27d1de"));
        // Too short for tx hash
    }

    #[test]
    fn test_debug_implementation() {
        let extractor = create_test_extractor();
        let debug_str = format!("{:?}", extractor);
        assert!(debug_str.contains("EntityExtractor"));
    }

    #[test]
    fn test_edge_case_find_pattern_positions_empty_pattern() {
        let extractor = create_test_extractor();
        let positions = extractor.find_pattern_positions("some text", "");
        assert!(positions.is_empty());
    }

    #[test]
    fn test_edge_case_parse_amount_match_empty_string() {
        let extractor = create_test_extractor();
        let (value, unit) = extractor.parse_amount_match("");
        assert_eq!(value, "");
        assert_eq!(unit, None);
    }

    #[test]
    fn test_edge_case_parse_numeric_value_single_character() {
        let extractor = create_test_extractor();
        assert_eq!(extractor.parse_numeric_value("5"), Some(5.0));
        assert_eq!(extractor.parse_numeric_value("K"), None);
        assert_eq!(extractor.parse_numeric_value("$"), None);
    }
}
