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
pub struct RawTextDocument {
    /// Unique document identifier
    pub id: String,
    /// Raw text content to be processed
    pub content: String,
    /// Optional document metadata
    pub metadata: Option<DocumentMetadata>,
    /// Vector embedding (populated during processing)
    pub embedding: Option<Vec<f32>>,
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Document source information
    pub source: DocumentSource,
}

/// Metadata associated with a document
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
pub struct DocumentMetadata {
    /// Title or summary of the document
    pub title: Option<String>,
    /// Tags or categories
    pub tags: Vec<String>,
    /// Blockchain network if relevant (e.g., "ethereum", "solana")
    pub chain: Option<String>,
    /// Block number if transaction-related
    pub block_number: Option<u64>,
    /// Transaction hash if applicable
    pub transaction_hash: Option<String>,
    /// Wallet addresses mentioned
    pub wallet_addresses: Vec<String>,
    /// Token addresses mentioned
    pub token_addresses: Vec<String>,
    /// Protocol names mentioned
    pub protocols: Vec<String>,
    /// Confidence score for extracted entities (0.0 to 1.0)
    pub extraction_confidence: Option<f32>,
    /// Additional custom fields
    pub custom_fields: HashMap<String, serde_json::Value>,
}

/// Source of the document
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum DocumentSource {
    /// User-provided text input
    UserInput,
    /// On-chain transaction data
    OnChain {
        /// Blockchain network name (e.g., "ethereum", "solana")
        chain: String,
        /// Transaction hash or ID
        transaction_hash: String,
    },
    /// Social media post (Twitter, Discord, etc.)
    Social {
        /// Social media platform name
        platform: String,
        /// Post or message ID
        post_id: String,
        /// Author username or handle
        author: Option<String>,
    },
    /// News article or blog post
    News {
        /// Article URL
        url: String,
        /// Publication or website name
        publication: Option<String>,
    },
    /// API response or structured data
    ApiResponse {
        /// API endpoint URL or identifier
        endpoint: String,
        /// When the data was retrieved
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    /// Other sources
    Other(String),
}

/// Extracted entities from a document
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ExtractedEntities {
    /// Wallet addresses found in the document
    pub wallets: Vec<EntityMention>,
    /// Token contracts and symbols
    pub tokens: Vec<EntityMention>,
    /// DeFi protocols and applications
    pub protocols: Vec<EntityMention>,
    /// Blockchain networks mentioned
    pub chains: Vec<EntityMention>,
    /// Numerical amounts (prices, balances, etc.)
    pub amounts: Vec<AmountMention>,
    /// Relationships between entities
    pub relationships: Vec<RelationshipMention>,
}

/// An entity mention in the document
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EntityMention {
    /// The entity text as it appears in the document
    pub text: String,
    /// Normalized/canonical form (e.g., lowercase address)
    pub canonical: String,
    /// Entity type
    pub entity_type: EntityType,
    /// Confidence score (0.0 to 1.0)
    pub confidence: f32,
    /// Character positions in the original text
    pub span: (usize, usize),
    /// Additional properties
    pub properties: HashMap<String, String>,
}

/// Type of entity
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub enum EntityType {
    /// Cryptocurrency wallet address
    Wallet,
    /// Token contract or symbol
    Token,
    /// DeFi protocol or dApp
    Protocol,
    /// Blockchain network
    Chain,
    /// Other entity type
    Other(String),
}

/// A numerical amount mentioned in the document
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AmountMention {
    /// Raw text of the amount
    pub text: String,
    /// Parsed numerical value
    pub value: f64,
    /// Associated unit (ETH, USDC, USD, etc.)
    pub unit: Option<String>,
    /// Amount type (balance, price, fee, etc.)
    pub amount_type: AmountType,
    /// Character positions in the original text
    pub span: (usize, usize),
}

/// Type of amount
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub enum AmountType {
    /// Account or wallet balance
    Balance,
    /// Token or asset price
    Price,
    /// Transaction or gas fee
    Fee,
    /// Trading volume
    Volume,
    /// Market capitalization
    MarketCap,
    /// Other amount type
    Other(String),
}

/// A relationship between entities
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RelationshipMention {
    /// Source entity
    pub from_entity: String,
    /// Target entity
    pub to_entity: String,
    /// Relationship type
    pub relationship_type: RelationshipType,
    /// Confidence score
    pub confidence: f32,
    /// Supporting text snippet
    pub context: String,
}

/// Type of relationship
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum RelationshipType {
    /// One wallet transferred to another
    Transferred,
    /// Wallet interacted with protocol
    Interacted,
    /// Entity holds or owns another entity
    Holds,
    /// Token is part of protocol
    PartOf,
    /// Protocol deployed on chain
    DeployedOn,
    /// Generic relationship
    Related,
}

impl RawTextDocument {
    /// Create a new raw text document with automatic ID generation.
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            content: content.into(),
            metadata: None,
            embedding: None,
            created_at: chrono::Utc::now(),
            source: DocumentSource::UserInput,
        }
    }

    /// Create a document with metadata.
    pub fn with_metadata(content: impl Into<String>, metadata: DocumentMetadata) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            content: content.into(),
            metadata: Some(metadata),
            embedding: None,
            created_at: chrono::Utc::now(),
            source: DocumentSource::UserInput,
        }
    }

    /// Create a document with a specific source.
    pub fn with_source(content: impl Into<String>, source: DocumentSource) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            content: content.into(),
            metadata: None,
            embedding: None,
            created_at: chrono::Utc::now(),
            source,
        }
    }

    /// Create a document for on-chain transaction data.
    pub fn from_transaction(
        content: impl Into<String>,
        chain: impl Into<String>,
        tx_hash: impl Into<String>,
    ) -> Self {
        let chain = chain.into();
        let tx_hash = tx_hash.into();

        let source = DocumentSource::OnChain {
            chain: chain.clone(),
            transaction_hash: tx_hash.clone(),
        };

        let metadata = DocumentMetadata {
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
    pub fn is_processed(&self) -> bool {
        self.embedding.is_some()
    }

    /// Get document word count
    pub fn word_count(&self) -> usize {
        self.content.split_whitespace().count()
    }

    /// Get character count
    pub fn char_count(&self) -> usize {
        self.content.len()
    }
}

impl DocumentMetadata {
    /// Add a tag to the document
    pub fn add_tag(&mut self, tag: impl Into<String>) {
        self.tags.push(tag.into());
    }

    /// Add a wallet address mention
    pub fn add_wallet(&mut self, address: impl Into<String>) {
        self.wallet_addresses.push(address.into());
    }

    /// Add a token address mention
    pub fn add_token(&mut self, address: impl Into<String>) {
        self.token_addresses.push(address.into());
    }

    /// Add a protocol name mention
    pub fn add_protocol(&mut self, name: impl Into<String>) {
        self.protocols.push(name.into());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use serde_json;
    use std::collections::HashMap;

    #[test]
    fn test_raw_text_document_new_should_create_document_with_defaults() {
        let content = "Test document content";
        let doc = RawTextDocument::new(content);

        assert_eq!(doc.content, content);
        assert!(doc.id.len() > 0); // UUID should be generated
        assert!(doc.metadata.is_none());
        assert!(doc.embedding.is_none());
        assert!(matches!(doc.source, DocumentSource::UserInput));
        // created_at should be recent (within last few seconds)
        let now = Utc::now();
        let diff = now.signed_duration_since(doc.created_at);
        assert!(diff.num_seconds() < 5);
    }

    #[test]
    fn test_raw_text_document_new_when_empty_content_should_work() {
        let doc = RawTextDocument::new("");
        assert_eq!(doc.content, "");
        assert!(doc.id.len() > 0);
    }

    #[test]
    fn test_raw_text_document_with_metadata_should_include_metadata() {
        let content = "Test content";
        let metadata = DocumentMetadata {
            title: Some("Test Title".to_string()),
            tags: vec!["test".to_string(), "document".to_string()],
            chain: Some("ethereum".to_string()),
            ..Default::default()
        };

        let doc = RawTextDocument::with_metadata(content, metadata.clone());

        assert_eq!(doc.content, content);
        assert!(doc.metadata.is_some());
        let doc_metadata = doc.metadata.unwrap();
        assert_eq!(doc_metadata.title, Some("Test Title".to_string()));
        assert_eq!(doc_metadata.tags, vec!["test", "document"]);
        assert_eq!(doc_metadata.chain, Some("ethereum".to_string()));
        assert!(matches!(doc.source, DocumentSource::UserInput));
    }

    #[test]
    fn test_raw_text_document_with_source_should_use_provided_source() {
        let content = "Test content";
        let source = DocumentSource::OnChain {
            chain: "ethereum".to_string(),
            transaction_hash: "0x123".to_string(),
        };

        let doc = RawTextDocument::with_source(content, source.clone());

        assert_eq!(doc.content, content);
        assert!(doc.metadata.is_none());
        if let DocumentSource::OnChain {
            chain,
            transaction_hash,
        } = doc.source
        {
            assert_eq!(chain, "ethereum");
            assert_eq!(transaction_hash, "0x123");
        } else {
            panic!("Expected OnChain source");
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

        let metadata = doc.metadata.unwrap();
        assert_eq!(metadata.chain, Some("ethereum".to_string()));
        assert_eq!(
            metadata.transaction_hash,
            Some("0xabcdef123456".to_string())
        );

        if let DocumentSource::OnChain {
            chain: src_chain,
            transaction_hash: src_hash,
        } = doc.source
        {
            assert_eq!(src_chain, "ethereum");
            assert_eq!(src_hash, "0xabcdef123456");
        } else {
            panic!("Expected OnChain source");
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
        let metadata = DocumentMetadata::default();

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
        let mut metadata = DocumentMetadata::default();
        metadata.add_tag("test");
        metadata.add_tag("blockchain");

        assert_eq!(metadata.tags, vec!["test", "blockchain"]);
    }

    #[test]
    fn test_document_metadata_add_wallet_should_add_to_wallet_addresses() {
        let mut metadata = DocumentMetadata::default();
        metadata.add_wallet("0x1234567890abcdef");
        metadata.add_wallet("0xfedcba0987654321");

        assert_eq!(
            metadata.wallet_addresses,
            vec!["0x1234567890abcdef", "0xfedcba0987654321"]
        );
    }

    #[test]
    fn test_document_metadata_add_token_should_add_to_token_addresses() {
        let mut metadata = DocumentMetadata::default();
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
        let mut metadata = DocumentMetadata::default();
        metadata.add_protocol("Uniswap");
        metadata.add_protocol("Compound");

        assert_eq!(metadata.protocols, vec!["Uniswap", "Compound"]);
    }

    #[test]
    fn test_document_source_variants_should_serialize_and_deserialize() {
        // Test UserInput
        let user_input = DocumentSource::UserInput;
        let json = serde_json::to_string(&user_input).unwrap();
        let deserialized: DocumentSource = serde_json::from_str(&json).unwrap();
        assert!(matches!(deserialized, DocumentSource::UserInput));

        // Test OnChain
        let on_chain = DocumentSource::OnChain {
            chain: "ethereum".to_string(),
            transaction_hash: "0x123".to_string(),
        };
        let json = serde_json::to_string(&on_chain).unwrap();
        let deserialized: DocumentSource = serde_json::from_str(&json).unwrap();
        if let DocumentSource::OnChain {
            chain,
            transaction_hash,
        } = deserialized
        {
            assert_eq!(chain, "ethereum");
            assert_eq!(transaction_hash, "0x123");
        } else {
            panic!("Expected OnChain variant");
        }

        // Test Social
        let social = DocumentSource::Social {
            platform: "twitter".to_string(),
            post_id: "12345".to_string(),
            author: Some("user123".to_string()),
        };
        let json = serde_json::to_string(&social).unwrap();
        let deserialized: DocumentSource = serde_json::from_str(&json).unwrap();
        if let DocumentSource::Social {
            platform,
            post_id,
            author,
        } = deserialized
        {
            assert_eq!(platform, "twitter");
            assert_eq!(post_id, "12345");
            assert_eq!(author, Some("user123".to_string()));
        } else {
            panic!("Expected Social variant");
        }

        // Test Social with None author
        let social_no_author = DocumentSource::Social {
            platform: "discord".to_string(),
            post_id: "67890".to_string(),
            author: None,
        };
        let json = serde_json::to_string(&social_no_author).unwrap();
        let deserialized: DocumentSource = serde_json::from_str(&json).unwrap();
        if let DocumentSource::Social {
            platform,
            post_id,
            author,
        } = deserialized
        {
            assert_eq!(platform, "discord");
            assert_eq!(post_id, "67890");
            assert!(author.is_none());
        } else {
            panic!("Expected Social variant");
        }

        // Test News
        let news = DocumentSource::News {
            url: "https://example.com/article".to_string(),
            publication: Some("Example News".to_string()),
        };
        let json = serde_json::to_string(&news).unwrap();
        let deserialized: DocumentSource = serde_json::from_str(&json).unwrap();
        if let DocumentSource::News { url, publication } = deserialized {
            assert_eq!(url, "https://example.com/article");
            assert_eq!(publication, Some("Example News".to_string()));
        } else {
            panic!("Expected News variant");
        }

        // Test News with None publication
        let news_no_pub = DocumentSource::News {
            url: "https://blog.example.com".to_string(),
            publication: None,
        };
        let json = serde_json::to_string(&news_no_pub).unwrap();
        let deserialized: DocumentSource = serde_json::from_str(&json).unwrap();
        if let DocumentSource::News { url, publication } = deserialized {
            assert_eq!(url, "https://blog.example.com");
            assert!(publication.is_none());
        } else {
            panic!("Expected News variant");
        }

        // Test ApiResponse
        let timestamp = Utc::now();
        let api_response = DocumentSource::ApiResponse {
            endpoint: "/api/v1/data".to_string(),
            timestamp,
        };
        let json = serde_json::to_string(&api_response).unwrap();
        let deserialized: DocumentSource = serde_json::from_str(&json).unwrap();
        if let DocumentSource::ApiResponse {
            endpoint,
            timestamp: ts,
        } = deserialized
        {
            assert_eq!(endpoint, "/api/v1/data");
            assert_eq!(ts, timestamp);
        } else {
            panic!("Expected ApiResponse variant");
        }

        // Test Other
        let other = DocumentSource::Other("custom_source".to_string());
        let json = serde_json::to_string(&other).unwrap();
        let deserialized: DocumentSource = serde_json::from_str(&json).unwrap();
        if let DocumentSource::Other(source) = deserialized {
            assert_eq!(source, "custom_source");
        } else {
            panic!("Expected Other variant");
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
            let json = serde_json::to_string(&variant).unwrap();
            let deserialized: EntityType = serde_json::from_str(&json).unwrap();
            match (&variant, &deserialized) {
                (EntityType::Wallet, EntityType::Wallet) => (),
                (EntityType::Token, EntityType::Token) => (),
                (EntityType::Protocol, EntityType::Protocol) => (),
                (EntityType::Chain, EntityType::Chain) => (),
                (EntityType::Other(a), EntityType::Other(b)) => assert_eq!(a, b),
                _ => panic!("Variants don't match"),
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
            let json = serde_json::to_string(&variant).unwrap();
            let deserialized: AmountType = serde_json::from_str(&json).unwrap();
            match (&variant, &deserialized) {
                (AmountType::Balance, AmountType::Balance) => (),
                (AmountType::Price, AmountType::Price) => (),
                (AmountType::Fee, AmountType::Fee) => (),
                (AmountType::Volume, AmountType::Volume) => (),
                (AmountType::MarketCap, AmountType::MarketCap) => (),
                (AmountType::Other(a), AmountType::Other(b)) => assert_eq!(a, b),
                _ => panic!("Variants don't match"),
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
            let json = serde_json::to_string(&variant).unwrap();
            let deserialized: RelationshipType = serde_json::from_str(&json).unwrap();
            match (&variant, &deserialized) {
                (RelationshipType::Transferred, RelationshipType::Transferred) => (),
                (RelationshipType::Interacted, RelationshipType::Interacted) => (),
                (RelationshipType::Holds, RelationshipType::Holds) => (),
                (RelationshipType::PartOf, RelationshipType::PartOf) => (),
                (RelationshipType::DeployedOn, RelationshipType::DeployedOn) => (),
                (RelationshipType::Related, RelationshipType::Related) => (),
                _ => panic!("Variants don't match"),
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

        let json = serde_json::to_string(&mention).unwrap();
        let deserialized: EntityMention = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.text, "0x1234...");
        assert_eq!(deserialized.canonical, "0x1234567890abcdef");
        assert!(matches!(deserialized.entity_type, EntityType::Wallet));
        assert_eq!(deserialized.confidence, 0.95);
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

        let json = serde_json::to_string(&mention).unwrap();
        let deserialized: AmountMention = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.text, "1.5 ETH");
        assert_eq!(deserialized.value, 1.5);
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

        let json = serde_json::to_string(&mention).unwrap();
        let deserialized: AmountMention = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.text, "100");
        assert_eq!(deserialized.value, 100.0);
        assert!(deserialized.unit.is_none());
        if let AmountType::Other(ref s) = deserialized.amount_type {
            assert_eq!(s, "count");
        } else {
            panic!("Expected Other amount type");
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

        let json = serde_json::to_string(&mention).unwrap();
        let deserialized: RelationshipMention = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.from_entity, "wallet1");
        assert_eq!(deserialized.to_entity, "wallet2");
        assert!(matches!(
            deserialized.relationship_type,
            RelationshipType::Transferred
        ));
        assert_eq!(deserialized.confidence, 0.9);
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

        let json = serde_json::to_string(&entities).unwrap();
        let deserialized: ExtractedEntities = serde_json::from_str(&json).unwrap();

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

        let metadata = DocumentMetadata {
            title: Some("Test Document".to_string()),
            tags: vec!["important".to_string()],
            chain: Some("polygon".to_string()),
            block_number: Some(12345678),
            transaction_hash: Some("0xabcdef".to_string()),
            wallet_addresses: vec!["0x123".to_string()],
            token_addresses: vec!["0x456".to_string()],
            protocols: vec!["Aave".to_string()],
            extraction_confidence: Some(0.87),
            custom_fields,
        };

        let json = serde_json::to_string(&metadata).unwrap();
        let deserialized: DocumentMetadata = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.title, Some("Test Document".to_string()));
        assert_eq!(deserialized.tags, vec!["important"]);
        assert_eq!(deserialized.chain, Some("polygon".to_string()));
        assert_eq!(deserialized.block_number, Some(12345678));
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
            metadata: Some(DocumentMetadata {
                title: Some("Complete Test".to_string()),
                ..Default::default()
            }),
            embedding: Some(vec![0.1, 0.2, 0.3, 0.4]),
            created_at: Utc::now(),
            source: DocumentSource::UserInput,
        };

        let json = serde_json::to_string(&doc).unwrap();
        let deserialized: RawTextDocument = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.id, "test-id");
        assert_eq!(deserialized.content, "Complete test document");
        assert!(deserialized.metadata.is_some());
        assert!(deserialized.embedding.is_some());
        assert_eq!(deserialized.embedding.unwrap(), vec![0.1, 0.2, 0.3, 0.4]);
        assert!(matches!(deserialized.source, DocumentSource::UserInput));
    }
}
