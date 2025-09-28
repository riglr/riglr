//! Document types and processing for graph memory.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// A raw text document that can be added to the graph memory system.
///
/// This document type supports blockchain-specific metadata and automatic
/// entity extraction to populate the knowledge graph.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RawText {
    /// Raw text content to be processed
    pub content: String,
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Vector embedding (populated during processing)
    pub embedding: Option<Vec<f32>>,
    /// Unique document identifier
    pub id: String,
    /// Optional document metadata
    pub metadata: Option<Metadata>,
    /// Document source information
    pub source: Source,
}

/// Metadata associated with a document
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
pub struct Metadata {
    /// Block number if transaction-related
    pub block_number: Option<u64>,
    /// Blockchain network if relevant (e.g., "ethereum", "solana")
    pub chain: Option<String>,
    /// Additional custom fields
    pub custom_fields: HashMap<String, serde_json::Value>,
    /// Confidence score for extracted entities (0.0 to 1.0)
    pub extraction_confidence: Option<f32>,
    /// Protocol names mentioned
    pub protocols: Vec<String>,
    /// Tags or categories
    pub tags: Vec<String>,
    /// Title or summary of the document
    pub title: Option<String>,
    /// Token addresses mentioned
    pub token_addresses: Vec<String>,
    /// Transaction hash if applicable
    pub transaction_hash: Option<String>,
    /// Wallet addresses mentioned
    pub wallet_addresses: Vec<String>,
}

/// Source of the document
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum Source {
    /// API response or structured data
    ApiResponse {
        /// API endpoint URL or identifier
        endpoint: String,
        /// When the data was retrieved
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    /// News article or blog post
    News {
        /// Article URL
        url: String,
        /// Publication or website name
        publication: Option<String>,
    },
    /// On-chain transaction data
    OnChain {
        /// Blockchain network name (e.g., "ethereum", "solana")
        chain: String,
        /// Transaction hash or ID
        transaction_hash: String,
    },
    /// Other sources
    Other(String),
    /// Social media post (Twitter, Discord, etc.)
    Social {
        /// Social media platform name
        platform: String,
        /// Post or message ID
        post_id: String,
        /// Author username or handle
        author: Option<String>,
    },
    /// User-provided text input
    UserInput,
}

/// Extracted entities from a document
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ExtractedEntities {
    /// Numerical amounts (prices, balances, etc.)
    pub amounts: Vec<AmountMention>,
    /// Blockchain networks mentioned
    pub chains: Vec<EntityMention>,
    /// `DeFi` protocols and applications
    pub protocols: Vec<EntityMention>,
    /// Relationships between entities
    pub relationships: Vec<RelationshipMention>,
    /// Token contracts and symbols
    pub tokens: Vec<EntityMention>,
    /// Wallet addresses found in the document
    pub wallets: Vec<EntityMention>,
}

/// An entity mention in the document
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EntityMention {
    /// Normalized/canonical form (e.g., lowercase address)
    pub canonical: String,
    /// Confidence score (0.0 to 1.0)
    pub confidence: f32,
    /// Entity type
    pub entity_type: EntityType,
    /// Additional properties
    pub properties: HashMap<String, String>,
    /// Character positions in the original text
    pub span: (usize, usize),
    /// The entity text as it appears in the document
    pub text: String,
}

/// Type of entity
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub enum EntityType {
    /// Blockchain network
    Chain,
    /// Other entity type
    Other(String),
    /// `DeFi` protocol or dApp
    Protocol,
    /// Token contract or symbol
    Token,
    /// Cryptocurrency wallet address
    Wallet,
}

/// A numerical amount mentioned in the document
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AmountMention {
    /// Amount type (balance, price, fee, etc.)
    pub amount_type: AmountType,
    /// Character positions in the original text
    pub span: (usize, usize),
    /// Raw text of the amount
    pub text: String,
    /// Associated unit (ETH, USDC, USD, etc.)
    pub unit: Option<String>,
    /// Parsed numerical value
    pub value: f64,
}

/// Type of amount
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub enum AmountType {
    /// Account or wallet balance
    Balance,
    /// Transaction or gas fee
    Fee,
    /// Market capitalization
    MarketCap,
    /// Other amount type
    Other(String),
    /// Token or asset price
    Price,
    /// Trading volume
    Volume,
}

/// A relationship between entities
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RelationshipMention {
    /// Confidence score
    pub confidence: f32,
    /// Supporting text snippet
    pub context: String,
    /// Source entity
    pub from_entity: String,
    /// Relationship type
    pub relationship_type: RelationshipType,
    /// Target entity
    pub to_entity: String,
}

/// Type of relationship
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum RelationshipType {
    /// Protocol deployed on chain
    DeployedOn,
    /// Entity holds or owns another entity
    Holds,
    /// Wallet interacted with protocol
    Interacted,
    /// Token is part of protocol
    PartOf,
    /// Generic relationship
    Related,
    /// One wallet transferred to another
    Transferred,
}

impl RawText {
    /// Get character count
    #[must_use]
    pub const fn char_count(&self) -> usize {
        self.content.len()
    }

    /// Create a document for on-chain transaction data.
    pub fn from_transaction(
        content: impl Into<String>,
        chain: impl Into<String>,
        tx_hash: impl Into<String>,
    ) -> Self {
        let chain = chain.into();
        let tx_hash = tx_hash.into();

        let source = Source::OnChain {
            chain: chain.clone(),
            transaction_hash: tx_hash.clone(),
        };

        let metadata = Metadata {
            chain: Some(chain),
            transaction_hash: Some(tx_hash),
            ..Default::default()
        };

        Self {
            id: Uuid::new_v4().to_string(),
            content: content.into(),
            metadata: Some(metadata),
            embedding: None,
            created_at: chrono::Utc::now(),
            source,
        }
    }

    /// Check if document has been processed (has embedding)
    #[must_use]
    pub const fn is_processed(&self) -> bool {
        self.embedding.is_some()
    }

    /// Create a new raw text document with automatic ID generation.
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            content: content.into(),
            metadata: None,
            embedding: None,
            created_at: chrono::Utc::now(),
            source: Source::UserInput,
        }
    }

    /// Create a document with metadata.
    pub fn with_metadata(content: impl Into<String>, metadata: Metadata) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            content: content.into(),
            metadata: Some(metadata),
            embedding: None,
            created_at: chrono::Utc::now(),
            source: Source::UserInput,
        }
    }

    /// Create a document with a specific source.
    pub fn with_source(content: impl Into<String>, source: Source) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            content: content.into(),
            metadata: None,
            embedding: None,
            created_at: chrono::Utc::now(),
            source,
        }
    }

    /// Get document word count
    #[must_use]
    pub fn word_count(&self) -> usize {
        self.content.split_whitespace().count()
    }
}

impl Metadata {
    /// Add a protocol name mention
    pub fn add_protocol(&mut self, name: impl Into<String>) {
        self.protocols.push(name.into());
    }

    /// Add a tag to the document
    pub fn add_tag(&mut self, tag: impl Into<String>) {
        self.tags.push(tag.into());
    }

    /// Add a token address mention
    pub fn add_token(&mut self, address: impl Into<String>) {
        self.token_addresses.push(address.into());
    }

    /// Add a wallet address mention
    pub fn add_wallet(&mut self, address: impl Into<String>) {
        self.wallet_addresses.push(address.into());
    }
}

#[cfg(test)]
#[expect(clippy::expect_used)]
mod tests {
    use super::*;
    use crate::RawTextDocument;
    use chrono::Utc;
    use serde_json;
    use std::collections::HashMap;

    #[test]
    fn test_raw_text_document_new_should_create_document_with_defaults() {
        let content = "Test document content";
        let doc = RawTextDocument::new(content);

        assert_eq!(doc.content, content);
        assert!(!doc.id.is_empty()); // UUID should be generated
        assert!(doc.metadata.is_none());
        assert!(doc.embedding.is_none());
        assert!(matches!(doc.source, Source::UserInput));
        // created_at should be recent (within last few seconds)
        let now = Utc::now();
        let diff = now.signed_duration_since(doc.created_at);
        assert!(diff.num_seconds() < 5);
    }

    #[test]
    fn test_raw_text_document_new_when_empty_content_should_work() {
        let doc = RawTextDocument::new("");
        assert_eq!(doc.content, "");
        assert!(!doc.id.is_empty());
    }

    #[test]
    fn test_raw_text_document_with_metadata_should_include_metadata() {
        let content = "Test content";
        let metadata = Metadata {
            title: Some("Test Title".to_string()),
            tags: vec!["test".to_string(), "document".to_string()],
            chain: Some("ethereum".to_string()),
            ..Default::default()
        };

        let doc = RawTextDocument::with_metadata(content, metadata);

        assert_eq!(doc.content, content);
        assert!(doc.metadata.is_some());
        let doc_metadata = doc.metadata.expect("Expected metadata to be Some");
        assert_eq!(doc_metadata.title, Some("Test Title".to_string()));
        assert_eq!(doc_metadata.tags, vec!["test", "document"]);
        assert_eq!(doc_metadata.chain, Some("ethereum".to_string()));
        assert!(matches!(doc.source, Source::UserInput));
    }

    #[test]
    fn test_raw_text_document_with_source_should_use_provided_source() {
        let content = "Test content";
        let source = Source::OnChain {
            chain: "ethereum".to_string(),
            transaction_hash: "0x123".to_string(),
        };

        let doc = RawTextDocument::with_source(content, source);

        assert_eq!(doc.content, content);
        assert!(doc.metadata.is_none());
        if let Source::OnChain {
            chain,
            transaction_hash,
        } = doc.source
        {
            assert_eq!(chain, "ethereum");
            assert_eq!(transaction_hash, "0x123");
        } else {
            unreachable!("Expected OnChain source");
        }
    }

    #[test]
    fn test_raw_text_document_from_transaction_should_create_with_metadata_and_source() {
        let content = "Transaction data";
        let chain = "ethereum";
        let tx_hash = "0xabcdef123456";

        let doc = RawTextDocument::from_transaction(content, chain, tx_hash);

        assert_eq!(doc.content, content);
        assert!(doc.metadata.is_some());

        let metadata = doc.metadata.expect("Expected metadata to be Some");
        assert_eq!(metadata.chain, Some("ethereum".to_string()));
        assert_eq!(
            metadata.transaction_hash,
            Some("0xabcdef123456".to_string())
        );

        if let Source::OnChain {
            chain: src_chain,
            transaction_hash: src_hash,
        } = doc.source
        {
            assert_eq!(src_chain, "ethereum");
            assert_eq!(src_hash, "0xabcdef123456");
        } else {
            unreachable!("Expected OnChain source");
        }
    }

    #[test]
    fn test_raw_text_document_is_processed_when_no_embedding_should_return_false() {
        let doc = RawTextDocument::new("test");
        assert!(!doc.is_processed());
    }

    #[test]
    fn test_raw_text_document_is_processed_when_has_embedding_should_return_true() {
        let mut doc = RawTextDocument::new("test");
        doc.embedding = Some(vec![0.1, 0.2, 0.3]);
        assert!(doc.is_processed());
    }

    #[test]
    fn test_raw_text_document_word_count_should_count_words() {
        let doc = RawTextDocument::new("hello world test document");
        assert_eq!(doc.word_count(), 4);
    }

    #[test]
    fn test_raw_text_document_word_count_when_empty_should_return_zero() {
        let doc = RawTextDocument::new("");
        assert_eq!(doc.word_count(), 0);
    }

    #[test]
    fn test_raw_text_document_word_count_with_multiple_spaces_should_handle_correctly() {
        let doc = RawTextDocument::new("hello    world   test");
        assert_eq!(doc.word_count(), 3);
    }

    #[test]
    fn test_raw_text_document_char_count_should_count_characters() {
        let doc = RawTextDocument::new("hello");
        assert_eq!(doc.char_count(), 5);
    }

    #[test]
    fn test_raw_text_document_char_count_when_empty_should_return_zero() {
        let doc = RawTextDocument::new("");
        assert_eq!(doc.char_count(), 0);
    }

    #[test]
    fn test_raw_text_document_char_count_with_unicode_should_count_bytes() {
        let doc = RawTextDocument::new("héllo");
        assert_eq!(doc.char_count(), 6); // UTF-8 byte count
    }

    #[test]
    fn test_document_metadata_default_should_create_empty_metadata() {
        let metadata = Metadata::default();

        assert!(metadata.title.is_none());
        assert!(metadata.tags.is_empty());
        assert!(metadata.chain.is_none());
        assert!(metadata.block_number.is_none());
        assert!(metadata.transaction_hash.is_none());
        assert!(metadata.wallet_addresses.is_empty());
        assert!(metadata.token_addresses.is_empty());
        assert!(metadata.protocols.is_empty());
        assert!(metadata.extraction_confidence.is_none());
        assert!(metadata.custom_fields.is_empty());
    }

    #[test]
    fn test_document_metadata_add_tag_should_add_to_tags_list() {
        let mut metadata = Metadata::default();
        metadata.add_tag("test");
        metadata.add_tag("blockchain");

        assert_eq!(metadata.tags, vec!["test", "blockchain"]);
    }

    #[test]
    fn test_document_metadata_add_wallet_should_add_to_wallet_addresses() {
        let mut metadata = Metadata::default();
        metadata.add_wallet("0x1234567890abcdef");
        metadata.add_wallet("0xfedcba0987654321");

        assert_eq!(
            metadata.wallet_addresses,
            vec!["0x1234567890abcdef", "0xfedcba0987654321"]
        );
    }

    #[test]
    fn test_document_metadata_add_token_should_add_to_token_addresses() {
        let mut metadata = Metadata::default();
        metadata.add_token("0xA0b86a33E6128c4a80c7B73F8C4a5c85f4b4c4d7");
        metadata.add_token("0xB0c86b33F6128d4b80d8C73G8D4b5d85g4c4d4e8");

        assert_eq!(
            metadata.token_addresses,
            vec![
                "0xA0b86a33E6128c4a80c7B73F8C4a5c85f4b4c4d7",
                "0xB0c86b33F6128d4b80d8C73G8D4b5d85g4c4d4e8"
            ]
        );
    }

    #[test]
    fn test_document_metadata_add_protocol_should_add_to_protocols() {
        let mut metadata = Metadata::default();
        metadata.add_protocol("Uniswap");
        metadata.add_protocol("Compound");

        assert_eq!(metadata.protocols, vec!["Uniswap", "Compound"]);
    }

    #[test]
    #[expect(clippy::too_many_lines)] // Comprehensive test for all enum variants
    fn test_document_source_variants_should_serialize_and_deserialize() {
        // Test UserInput
        let user_input = Source::UserInput;
        let json = serde_json::to_string(&user_input).expect("Serialization should not fail");
        let deserialized: Source =
            serde_json::from_str(&json).expect("Deserialization should not fail");
        assert!(matches!(deserialized, Source::UserInput));

        // Test OnChain
        let on_chain = Source::OnChain {
            chain: "ethereum".to_string(),
            transaction_hash: "0x123".to_string(),
        };
        let json = serde_json::to_string(&on_chain).expect("Serialization should not fail");
        let deserialized: Source =
            serde_json::from_str(&json).expect("Deserialization should not fail");
        if let Source::OnChain {
            chain,
            transaction_hash,
        } = deserialized
        {
            assert_eq!(chain, "ethereum");
            assert_eq!(transaction_hash, "0x123");
        } else {
            unreachable!("Expected OnChain variant");
        }

        // Test Social
        let social = Source::Social {
            platform: "twitter".to_string(),
            post_id: "12345".to_string(),
            author: Some("user123".to_string()),
        };
        let json = serde_json::to_string(&social).expect("Serialization should not fail");
        let deserialized: Source =
            serde_json::from_str(&json).expect("Deserialization should not fail");
        if let Source::Social {
            platform,
            post_id,
            author,
        } = deserialized
        {
            assert_eq!(platform, "twitter");
            assert_eq!(post_id, "12345");
            assert_eq!(author, Some("user123".to_string()));
        } else {
            unreachable!("Expected Social variant");
        }

        // Test Social with None author
        let social_no_author = Source::Social {
            platform: "discord".to_string(),
            post_id: "67890".to_string(),
            author: None,
        };
        let json = serde_json::to_string(&social_no_author).expect("Serialization should not fail");
        let deserialized: Source =
            serde_json::from_str(&json).expect("Deserialization should not fail");
        if let Source::Social {
            platform,
            post_id,
            author,
        } = deserialized
        {
            assert_eq!(platform, "discord");
            assert_eq!(post_id, "67890");
            assert!(author.is_none());
        } else {
            unreachable!("Expected Social variant");
        }

        // Test News
        let news = Source::News {
            url: "https://example.com/article".to_string(),
            publication: Some("Example News".to_string()),
        };
        let json = serde_json::to_string(&news).expect("Serialization should not fail");
        let deserialized: Source =
            serde_json::from_str(&json).expect("Deserialization should not fail");
        if let Source::News { url, publication } = deserialized {
            assert_eq!(url, "https://example.com/article");
            assert_eq!(publication, Some("Example News".to_string()));
        } else {
            unreachable!("Expected News variant");
        }

        // Test News with None publication
        let news_no_pub = Source::News {
            url: "https://blog.example.com".to_string(),
            publication: None,
        };
        let json = serde_json::to_string(&news_no_pub).expect("Serialization should not fail");
        let deserialized: Source =
            serde_json::from_str(&json).expect("Deserialization should not fail");
        if let Source::News { url, publication } = deserialized {
            assert_eq!(url, "https://blog.example.com");
            assert!(publication.is_none());
        } else {
            unreachable!("Expected News variant");
        }

        // Test ApiResponse
        let timestamp = Utc::now();
        let api_response = Source::ApiResponse {
            endpoint: "/api/v1/data".to_string(),
            timestamp,
        };
        let json = serde_json::to_string(&api_response).expect("Serialization should not fail");
        let deserialized: Source =
            serde_json::from_str(&json).expect("Deserialization should not fail");
        if let Source::ApiResponse {
            endpoint,
            timestamp: ts,
        } = deserialized
        {
            assert_eq!(endpoint, "/api/v1/data");
            assert_eq!(ts, timestamp);
        } else {
            unreachable!("Expected ApiResponse variant");
        }

        // Test Other
        let other = Source::Other("custom_source".to_string());
        let json = serde_json::to_string(&other).expect("Serialization should not fail");
        let deserialized: Source =
            serde_json::from_str(&json).expect("Deserialization should not fail");
        if let Source::Other(source) = deserialized {
            assert_eq!(source, "custom_source");
        } else {
            unreachable!("Expected Other variant");
        }
    }

    #[test]
    fn test_entity_type_variants_should_serialize_and_deserialize() {
        // Test all enum variants
        let variants = vec![
            EntityType::Wallet,
            EntityType::Token,
            EntityType::Protocol,
            EntityType::Chain,
            EntityType::Other("custom".to_string()),
        ];

        for variant in variants {
            let json = serde_json::to_string(&variant).expect("Serialization should not fail");
            let deserialized: EntityType =
                serde_json::from_str(&json).expect("Deserialization should not fail");
            match (variant, deserialized) {
                (EntityType::Wallet, EntityType::Wallet)
                | (EntityType::Token, EntityType::Token)
                | (EntityType::Protocol, EntityType::Protocol)
                | (EntityType::Chain, EntityType::Chain) => (),
                (EntityType::Other(a), EntityType::Other(b)) => assert_eq!(a, b),
                _ => {
                    unreachable!("Variants don't match");
                }
            }
        }
    }

    #[test]
    fn test_amount_type_variants_should_serialize_and_deserialize() {
        let variants = vec![
            AmountType::Balance,
            AmountType::Price,
            AmountType::Fee,
            AmountType::Volume,
            AmountType::MarketCap,
            AmountType::Other("custom_amount".to_string()),
        ];

        for variant in variants {
            let json = serde_json::to_string(&variant).expect("Serialization should not fail");
            let deserialized: AmountType =
                serde_json::from_str(&json).expect("Deserialization should not fail");
            match (variant, deserialized) {
                (AmountType::Balance, AmountType::Balance)
                | (AmountType::Price, AmountType::Price)
                | (AmountType::Fee, AmountType::Fee)
                | (AmountType::Volume, AmountType::Volume)
                | (AmountType::MarketCap, AmountType::MarketCap) => (),
                (AmountType::Other(a), AmountType::Other(b)) => assert_eq!(a, b),
                _ => {
                    unreachable!("Variants don't match");
                }
            }
        }
    }

    #[test]
    fn test_relationship_type_variants_should_serialize_and_deserialize() {
        let variants = vec![
            RelationshipType::Transferred,
            RelationshipType::Interacted,
            RelationshipType::Holds,
            RelationshipType::PartOf,
            RelationshipType::DeployedOn,
            RelationshipType::Related,
        ];

        for variant in variants {
            let json = serde_json::to_string(&variant).expect("Serialization should not fail");
            let deserialized: RelationshipType =
                serde_json::from_str(&json).expect("Deserialization should not fail");
            match (variant, deserialized) {
                (RelationshipType::Transferred, RelationshipType::Transferred)
                | (RelationshipType::Interacted, RelationshipType::Interacted)
                | (RelationshipType::Holds, RelationshipType::Holds)
                | (RelationshipType::PartOf, RelationshipType::PartOf)
                | (RelationshipType::DeployedOn, RelationshipType::DeployedOn)
                | (RelationshipType::Related, RelationshipType::Related) => (),
                _ => {
                    unreachable!("Variants don't match");
                }
            }
        }
    }

    #[test]
    fn test_entity_mention_serialization() {
        let mut properties = HashMap::new();
        properties.insert("network".to_string(), "ethereum".to_string());

        let mention = EntityMention {
            text: "0x1234...".to_string(),
            canonical: "0x1234567890abcdef".to_string(),
            entity_type: EntityType::Wallet,
            confidence: 0.95,
            span: (10, 20),
            properties,
        };

        let json = serde_json::to_string(&mention).expect("Serialization should not fail");
        let deserialized: EntityMention =
            serde_json::from_str(&json).expect("Deserialization should not fail");

        assert_eq!(deserialized.text, "0x1234...");
        assert_eq!(deserialized.canonical, "0x1234567890abcdef");
        assert!(matches!(deserialized.entity_type, EntityType::Wallet));
        {
            assert!(
                (deserialized.confidence - 0.95).abs() < f32::EPSILON,
                "Expected confidence ~0.95, got {}",
                deserialized.confidence
            );
        }
        assert_eq!(deserialized.span, (10, 20));
        assert_eq!(
            deserialized.properties.get("network"),
            Some(&"ethereum".to_string())
        );
    }

    #[test]
    fn test_amount_mention_serialization() {
        let mention = AmountMention {
            text: "1.5 ETH".to_string(),
            value: 1.5,
            unit: Some("ETH".to_string()),
            amount_type: AmountType::Balance,
            span: (5, 12),
        };

        let json = serde_json::to_string(&mention).expect("Serialization should not fail");
        let deserialized: AmountMention =
            serde_json::from_str(&json).expect("Deserialization should not fail");

        assert_eq!(deserialized.text, "1.5 ETH");
        {
            assert!(
                (deserialized.value - 1.5).abs() < f64::EPSILON,
                "Expected value ~1.5, got {}",
                deserialized.value
            );
        }
        assert_eq!(deserialized.unit, Some("ETH".to_string()));
        assert!(matches!(deserialized.amount_type, AmountType::Balance));
        assert_eq!(deserialized.span, (5, 12));
    }

    #[test]
    fn test_amount_mention_without_unit() {
        let mention = AmountMention {
            text: "100".to_string(),
            value: 100.0,
            unit: None,
            amount_type: AmountType::Other("count".to_string()),
            span: (0, 3),
        };

        let json = serde_json::to_string(&mention).expect("Serialization should not fail");
        let deserialized: AmountMention =
            serde_json::from_str(&json).expect("Deserialization should not fail");

        assert_eq!(deserialized.text, "100");
        {
            assert!(
                (deserialized.value - 100.0).abs() < f64::EPSILON,
                "Expected value ~100.0, got {}",
                deserialized.value
            );
        }
        assert!(deserialized.unit.is_none());
        if let AmountType::Other(ref s) = deserialized.amount_type {
            assert_eq!(s, "count");
        } else {
            unreachable!("Expected Other amount type");
        }
    }

    #[test]
    fn test_relationship_mention_serialization() {
        let mention = RelationshipMention {
            from_entity: "wallet1".to_string(),
            to_entity: "wallet2".to_string(),
            relationship_type: RelationshipType::Transferred,
            confidence: 0.9,
            context: "sent 5 ETH to".to_string(),
        };

        let json = serde_json::to_string(&mention).expect("Serialization should not fail");
        let deserialized: RelationshipMention =
            serde_json::from_str(&json).expect("Deserialization should not fail");

        assert_eq!(deserialized.from_entity, "wallet1");
        assert_eq!(deserialized.to_entity, "wallet2");
        assert!(matches!(
            deserialized.relationship_type,
            RelationshipType::Transferred
        ));
        {
            assert!(
                (deserialized.confidence - 0.9).abs() < f32::EPSILON,
                "Expected confidence ~0.9, got {}",
                deserialized.confidence
            );
        }
        assert_eq!(deserialized.context, "sent 5 ETH to");
    }

    #[test]
    fn test_extracted_entities_serialization() {
        let entities = ExtractedEntities {
            wallets: vec![EntityMention {
                text: "wallet1".to_string(),
                canonical: "0x123".to_string(),
                entity_type: EntityType::Wallet,
                confidence: 0.95,
                span: (0, 7),
                properties: HashMap::new(),
            }],
            tokens: vec![],
            protocols: vec![],
            chains: vec![],
            amounts: vec![],
            relationships: vec![],
        };

        let json = serde_json::to_string(&entities).expect("Serialization should not fail");
        let deserialized: ExtractedEntities =
            serde_json::from_str(&json).expect("Deserialization should not fail");

        assert_eq!(deserialized.wallets.len(), 1);
        assert_eq!(deserialized.tokens.len(), 0);
        assert_eq!(deserialized.protocols.len(), 0);
        assert_eq!(deserialized.chains.len(), 0);
        assert_eq!(deserialized.amounts.len(), 0);
        assert_eq!(deserialized.relationships.len(), 0);
    }

    #[test]
    fn test_document_metadata_with_custom_fields() {
        let mut custom_fields = HashMap::new();
        custom_fields.insert(
            "priority".to_string(),
            serde_json::Value::String("high".to_string()),
        );
        custom_fields.insert(
            "score".to_string(),
            serde_json::Value::Number(serde_json::Number::from(85)),
        );

        let metadata = Metadata {
            title: Some("Test Document".to_string()),
            tags: vec!["important".to_string()],
            chain: Some("polygon".to_string()),
            block_number: Some(12_345_678),
            transaction_hash: Some("0xabcdef".to_string()),
            wallet_addresses: vec!["0x123".to_string()],
            token_addresses: vec!["0x456".to_string()],
            protocols: vec!["Aave".to_string()],
            extraction_confidence: Some(0.87),
            custom_fields,
        };

        let json = serde_json::to_string(&metadata).expect("Serialization should not fail");
        let deserialized: Metadata =
            serde_json::from_str(&json).expect("Deserialization should not fail");

        assert_eq!(deserialized.title, Some("Test Document".to_string()));
        assert_eq!(deserialized.tags, vec!["important"]);
        assert_eq!(deserialized.chain, Some("polygon".to_string()));
        assert_eq!(deserialized.block_number, Some(12_345_678));
        assert_eq!(deserialized.transaction_hash, Some("0xabcdef".to_string()));
        assert_eq!(deserialized.wallet_addresses, vec!["0x123"]);
        assert_eq!(deserialized.token_addresses, vec!["0x456"]);
        assert_eq!(deserialized.protocols, vec!["Aave"]);
        assert_eq!(deserialized.extraction_confidence, Some(0.87));
        assert_eq!(deserialized.custom_fields.len(), 2);
    }

    #[test]
    fn test_complete_raw_text_document_serialization() {
        let doc = RawTextDocument {
            id: "test-id".to_string(),
            content: "Complete test document".to_string(),
            metadata: Some(Metadata {
                title: Some("Complete Test".to_string()),
                ..Default::default()
            }),
            embedding: Some(vec![0.1, 0.2, 0.3, 0.4]),
            created_at: Utc::now(),
            source: Source::UserInput,
        };

        let json = serde_json::to_string(&doc).expect("Serialization should not fail");
        let deserialized: RawTextDocument =
            serde_json::from_str(&json).expect("Deserialization should not fail");

        assert_eq!(deserialized.id, "test-id");
        assert_eq!(deserialized.content, "Complete test document");
        assert!(deserialized.metadata.is_some());
        assert!(deserialized.embedding.is_some());
        assert_eq!(
            deserialized
                .embedding
                .expect("Expected embedding to be Some"),
            vec![0.1, 0.2, 0.3, 0.4]
        );
        assert!(matches!(deserialized.source, Source::UserInput));
    }
}
