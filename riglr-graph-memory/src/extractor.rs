//! Entity extraction and relationship mining from text documents.
//!
//! This module provides production-grade entity extraction capabilities for blockchain-related text,
//! identifying wallets, tokens, protocols, amounts, and relationships between entities.

use crate::document::{
    AmountMention, AmountType, EntityMention, EntityType, ExtractedEntities,
    RelationshipMention, RelationshipType,
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

/// Solana address regex pattern
static SOL_ADDRESS_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"[1-9A-HJ-NP-Za-km-z]{32,44}").expect("Invalid Solana address regex"));

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
    /// Create a new entity extractor with predefined patterns
    pub fn new() -> Self {
        let mut extractor = Self {
            protocol_patterns: HashMap::new(),
            token_patterns: HashMap::new(),
            chain_patterns: HashMap::new(),
            regex_cache: HashMap::new(),
        };

        extractor.initialize_patterns();
        extractor
    }

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
        let relationships = self
            .extract_relationships(text, &wallets, &tokens, &protocols);

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
            if let Ok(regex) = Regex::new(pattern) {
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
        let pattern_lower = pattern.to_lowercase();

        let mut start = 0;
        while let Some(pos) = text[start..].find(&pattern_lower) {
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
                .all(|c| "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz".contains(c))
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
        let cleaned = value_str.replace("$", "").replace(",", "");

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
            || amount_lower.contains("$")
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
        Self::new()
    }
}
