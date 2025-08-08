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
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
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
        chain: String,
        transaction_hash: String,
    },
    /// Social media post (Twitter, Discord, etc.)
    Social {
        platform: String,
        post_id: String,
        author: Option<String>,
    },
    /// News article or blog post
    News {
        url: String,
        publication: Option<String>,
    },
    /// API response or structured data
    ApiResponse {
        endpoint: String,
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
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum EntityType {
    Wallet,
    Token,
    Protocol,
    Chain,
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
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum AmountType {
    Balance,
    Price,
    Fee,
    Volume,
    MarketCap,
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
    /// Wallet holds token
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

        let mut metadata = DocumentMetadata::default();
        metadata.chain = Some(chain);
        metadata.transaction_hash = Some(tx_hash);

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
    /// Create empty metadata
    pub fn new() -> Self {
        Self::default()
    }

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

impl Default for DocumentMetadata {
    fn default() -> Self {
        Self {
            title: None,
            tags: Vec::new(),
            chain: None,
            block_number: None,
            transaction_hash: None,
            wallet_addresses: Vec::new(),
            token_addresses: Vec::new(),
            protocols: Vec::new(),
            extraction_confidence: None,
            custom_fields: HashMap::new(),
        }
    }
}
