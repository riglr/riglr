# riglr-graph-memory API Reference

Comprehensive API documentation for the `riglr-graph-memory` crate.

## Table of Contents

### Structs

- [`AmountMention`](#amountmention)
- [`DocumentMetadata`](#documentmetadata)
- [`EntityExtractor`](#entityextractor)
- [`EntityMention`](#entitymention)
- [`ExtractedEntities`](#extractedentities)
- [`GraphDocument`](#graphdocument)
- [`GraphMemory`](#graphmemory)
- [`GraphMemoryConfig`](#graphmemoryconfig)
- [`GraphMemoryStats`](#graphmemorystats)
- [`GraphRetriever`](#graphretriever)
- [`GraphRetrieverConfig`](#graphretrieverconfig)
- [`GraphSearchResult`](#graphsearchresult)
- [`GraphVectorStore`](#graphvectorstore)
- [`Neo4jClient`](#neo4jclient)
- [`RawTextDocument`](#rawtextdocument)
- [`RelationshipMention`](#relationshipmention)
- [`RigDocument`](#rigdocument)
- [`SearchMetrics`](#searchmetrics)

### Functions

- [`add_documents`](#add_documents)
- [`add_documents`](#add_documents)
- [`add_documents`](#add_documents)
- [`add_protocol`](#add_protocol)
- [`add_tag`](#add_tag)
- [`add_token`](#add_token)
- [`add_wallet`](#add_wallet)
- [`broad_context`](#broad_context)
- [`char_count`](#char_count)
- [`client`](#client)
- [`create_indexes`](#create_indexes)
- [`delete_documents`](#delete_documents)
- [`execute_query`](#execute_query)
- [`extract`](#extract)
- [`extractor`](#extractor)
- [`from_transaction`](#from_transaction)
- [`get_documents`](#get_documents)
- [`get_stats`](#get_stats)
- [`get_stats`](#get_stats)
- [`high_precision`](#high_precision)
- [`is_processed`](#is_processed)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new`](#new)
- [`new_default`](#new_default)
- [`retriever`](#retriever)
- [`search`](#search)
- [`search`](#search)
- [`search_with_graph_context`](#search_with_graph_context)
- [`simple_query`](#simple_query)
- [`top_n_ids`](#top_n_ids)
- [`with_defaults`](#with_defaults)
- [`with_dimension`](#with_dimension)
- [`with_metadata`](#with_metadata)
- [`with_source`](#with_source)
- [`word_count`](#word_count)

### Enums

- [`AmountType`](#amounttype)
- [`DocumentSource`](#documentsource)
- [`EntityType`](#entitytype)
- [`GraphMemoryError`](#graphmemoryerror)
- [`RelationshipType`](#relationshiptype)

### Traits

- [`VectorStore`](#vectorstore)

### Constants

- [`VERSION`](#version)

## Structs

### AmountMention

**Source**: `src/document.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
```

```rust
pub struct AmountMention { /// Raw text of the amount pub text: String, /// Parsed numerical value pub value: f64, /// Associated unit (ETH, USDC, USD, etc.)
```

A numerical amount mentioned in the document

---

### DocumentMetadata

**Source**: `src/document.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
```

```rust
pub struct DocumentMetadata { /// Title or summary of the document pub title: Option<String>, /// Tags or categories pub tags: Vec<String>, /// Blockchain network if relevant (e.g., "ethereum", "solana")
```

Metadata associated with a document

---

### EntityExtractor

**Source**: `src/extractor.rs`

**Attributes**:
```rust
#[derive(Debug)]
```

```rust
pub struct EntityExtractor { /// Known protocol names for recognition protocol_patterns: HashMap<String, Vec<String>>, /// Token symbol patterns token_patterns: HashMap<String, Vec<String>>, /// Blockchain network patterns chain_patterns: HashMap<String, Vec<String>>, /// Compiled regex patterns for performance #[allow(dead_code)]
```

Production-grade entity extractor for blockchain text analysis

---

### EntityMention

**Source**: `src/document.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
```

```rust
pub struct EntityMention { /// The entity text as it appears in the document pub text: String, /// Normalized/canonical form (e.g., lowercase address)
```

An entity mention in the document

---

### ExtractedEntities

**Source**: `src/document.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
```

```rust
pub struct ExtractedEntities { /// Wallet addresses found in the document pub wallets: Vec<EntityMention>, /// Token contracts and symbols pub tokens: Vec<EntityMention>, /// DeFi protocols and applications pub protocols: Vec<EntityMention>, /// Blockchain networks mentioned pub chains: Vec<EntityMention>, /// Numerical amounts (prices, balances, etc.)
```

Extracted entities from a document

---

### GraphDocument

**Source**: `src/vector_store.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct GraphDocument { /// Document unique identifier pub id: String, /// Document content pub content: String, /// Vector embedding pub embedding: Vec<f32>, /// Metadata extracted from the document pub metadata: HashMap<String, Value>, /// Entities extracted from this document pub entities: Vec<String>, /// Graph relationships pub relationships: Vec<String>, /// Similarity score (populated during search)
```

A document stored in the graph with vector embeddings

---

### GraphMemory

**Source**: `src/graph.rs`

**Attributes**:
```rust
#[derive(Debug)]
```

```rust
pub struct GraphMemory { /// Neo4j database client client: Arc<Neo4jClient>, /// Entity extractor for processing documents extractor: EntityExtractor, /// Graph-based vector retriever retriever: Arc<GraphRetriever>, /// Configuration settings config: GraphMemoryConfig, /// OpenAI client for real embeddings (optional)
```

The main graph memory system that provides comprehensive document storage,
entity extraction, and hybrid vector + graph search capabilities.

---

### GraphMemoryConfig

**Source**: `src/graph.rs`

**Attributes**:
```rust
#[derive(Debug, Clone)]
```

```rust
pub struct GraphMemoryConfig { /// Neo4j connection URL pub neo4j_url: String, /// Database username pub username: Option<String>, /// Database password pub password: Option<String>, /// Database name (default: "neo4j")
```

Configuration for the graph memory system

---

### GraphMemoryStats

**Source**: `src/graph.rs`

**Attributes**:
```rust
#[derive(Debug, Clone)]
```

```rust
pub struct GraphMemoryStats { /// Total number of documents pub document_count: u64, /// Total number of entity nodes pub entity_count: u64, /// Total number of relationships pub relationship_count: u64, /// Total number of wallets tracked pub wallet_count: u64, /// Total number of tokens tracked pub token_count: u64, /// Total number of protocols tracked pub protocol_count: u64, /// Average entities per document pub avg_entities_per_doc: f64, /// Storage size in bytes (approximate)
```

Statistics about the graph memory system

---

### GraphRetriever

**Source**: `src/vector_store.rs`

**Attributes**:
```rust
#[derive(Debug)]
```

```rust
pub struct GraphRetriever { /// Neo4j client for database operations client: Arc<Neo4jClient>, /// Vector index name in Neo4j index_name: String, /// Minimum similarity threshold for vector search similarity_threshold: f32, /// Maximum number of graph hops for relationship traversal max_graph_hops: u32, /// Embedding dimension (default 1536 for OpenAI)
```

A retriever that combines graph and vector search for enhanced context.

This implementation provides sophisticated document retrieval by leveraging both
vector similarity search and graph relationships to find the most relevant context
for agent queries.

---

### GraphRetrieverConfig

**Source**: `src/vector_store.rs`

**Attributes**:
```rust
#[derive(Debug, Clone)]
```

```rust
pub struct GraphRetrieverConfig { /// Minimum similarity threshold (0.0 to 1.0)
```

Configuration for graph-based vector retrieval

---

### GraphSearchResult

**Source**: `src/vector_store.rs`

**Attributes**:
```rust
#[derive(Debug, Clone)]
```

```rust
pub struct GraphSearchResult { /// Retrieved documents pub documents: Vec<GraphDocument>, /// Related entities found through graph traversal pub related_entities: Vec<String>, /// Query performance metrics pub metrics: SearchMetrics, }
```

Search result from graph vector store

---

### GraphVectorStore

**Source**: `src/rig_vector_store.rs`

```rust
pub struct GraphVectorStore { client: Arc<Neo4jClient>, index_name: String, embedding_dimension: usize, }
```

Graph-based vector store implementation for rig

This implementation leverages Neo4j's vector capabilities combined with graph traversal
to provide enhanced contextual search for rig agents.

---

### Neo4jClient

**Source**: `src/client.rs`

**Attributes**:
```rust
#[derive(Debug, Clone)]
```

```rust
pub struct Neo4jClient { /// HTTP client for API requests client: Client, /// Base URL for Neo4j HTTP API (e.g., http://localhost:7474)
```

Neo4j database client using HTTP REST API.

This client provides production-grade connectivity to Neo4j databases
with proper error handling, authentication, and query optimization.

---

### RawTextDocument

**Source**: `src/document.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
```

```rust
pub struct RawTextDocument { /// Unique document identifier pub id: String, /// Raw text content to be processed pub content: String, /// Optional document metadata pub metadata: Option<DocumentMetadata>, /// Vector embedding (populated during processing)
```

A raw text document that can be added to the graph memory system.

This document type supports blockchain-specific metadata and automatic
entity extraction to populate the knowledge graph.

---

### RelationshipMention

**Source**: `src/document.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
```

```rust
pub struct RelationshipMention { /// Source entity pub from_entity: String, /// Target entity pub to_entity: String, /// Relationship type pub relationship_type: RelationshipType, /// Confidence score pub confidence: f32, /// Supporting text snippet pub context: String, }
```

A relationship between entities

---

### RigDocument

**Source**: `src/rig_vector_store.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

```rust
pub struct RigDocument { /// Unique document identifier pub id: String, /// Main text content of the document pub content: String, /// Vector embedding representation of the document pub embedding: Vec<f32>, /// Additional metadata key-value pairs for the document pub metadata: HashMap<String, Value>, }
```

Document type that bridges between rig and our graph memory system

---

### SearchMetrics

**Source**: `src/vector_store.rs`

**Attributes**:
```rust
#[derive(Debug, Clone)]
```

```rust
pub struct SearchMetrics { /// Vector search time in milliseconds pub vector_search_time_ms: u64, /// Graph traversal time in milliseconds pub graph_traversal_time_ms: u64, /// Total query time in milliseconds pub total_time_ms: u64, /// Number of nodes examined pub nodes_examined: u32, /// Number of relationships traversed pub relationships_traversed: u32, }
```

Performance metrics for graph search operations

---

## Functions

### add_documents

**Source**: `src/rig_vector_store.rs`

```rust
pub async fn add_documents(&self, documents: Vec<RigDocument>) -> Result<Vec<String>>
```

Add documents to the graph vector store (direct implementation)

---

### add_documents

**Source**: `src/vector_store.rs`

```rust
pub async fn add_documents(&self, documents: Vec<RawTextDocument>) -> Result<Vec<String>>
```

Add documents to the graph vector store
This is the core functionality that would be exposed through rig::VectorStore

---

### add_documents

**Source**: `src/graph.rs`

```rust
pub async fn add_documents(&self, documents: Vec<RawTextDocument>) -> Result<Vec<String>>
```

Add documents to the graph with full processing pipeline.

---

### add_protocol

**Source**: `src/document.rs`

```rust
pub fn add_protocol(&mut self, name: impl Into<String>)
```

Add a protocol name mention

---

### add_tag

**Source**: `src/document.rs`

```rust
pub fn add_tag(&mut self, tag: impl Into<String>)
```

Add a tag to the document

---

### add_token

**Source**: `src/document.rs`

```rust
pub fn add_token(&mut self, address: impl Into<String>)
```

Add a token address mention

---

### add_wallet

**Source**: `src/document.rs`

```rust
pub fn add_wallet(&mut self, address: impl Into<String>)
```

Add a wallet address mention

---

### broad_context

**Source**: `src/vector_store.rs`

```rust
pub fn broad_context() -> Self
```

Create configuration for broad contextual search

---

### char_count

**Source**: `src/document.rs`

```rust
pub fn char_count(&self) -> usize
```

Get character count

---

### client

**Source**: `src/graph.rs`

```rust
pub fn client(&self) -> &Neo4jClient
```

Get the underlying Neo4j client for direct queries

---

### create_indexes

**Source**: `src/client.rs`

```rust
pub async fn create_indexes(&self) -> Result<()>
```

Create database indexes for optimal performance

---

### delete_documents

**Source**: `src/rig_vector_store.rs`

```rust
pub async fn delete_documents(&self, ids: Vec<String>) -> Result<Vec<String>>
```

Delete documents by their IDs

---

### execute_query

**Source**: `src/client.rs`

```rust
pub async fn execute_query( &self, query: &str, parameters: Option<HashMap<String, Value>>, ) -> Result<Value>
```

Execute a Cypher query with optional parameters.

# Arguments

* `query` - Cypher query string
* `parameters` - Optional query parameters

# Returns

Raw JSON response from Neo4j

---

### extract

**Source**: `src/extractor.rs`

```rust
pub fn extract(&self, text: &str) -> ExtractedEntities
```

Extract all entities and relationships from a text document

---

### extractor

**Source**: `src/graph.rs`

```rust
pub fn extractor(&self) -> &EntityExtractor
```

Get the entity extractor

---

### from_transaction

**Source**: `src/document.rs`

```rust
pub fn from_transaction( content: impl Into<String>, chain: impl Into<String>, tx_hash: impl Into<String>, ) -> Self
```

Create a document for on-chain transaction data.

---

### get_documents

**Source**: `src/rig_vector_store.rs`

```rust
pub async fn get_documents(&self, ids: Vec<String>) -> Result<Vec<RigDocument>>
```

Get documents by their IDs

---

### get_stats

**Source**: `src/client.rs`

```rust
pub async fn get_stats(&self) -> Result<HashMap<String, Value>>
```

Get database statistics

---

### get_stats

**Source**: `src/graph.rs`

```rust
pub async fn get_stats(&self) -> Result<GraphMemoryStats>
```

Get comprehensive statistics about the graph

---

### high_precision

**Source**: `src/vector_store.rs`

```rust
pub fn high_precision() -> Self
```

Create configuration for high-precision search

---

### is_processed

**Source**: `src/document.rs`

```rust
pub fn is_processed(&self) -> bool
```

Check if document has been processed (has embedding)

---

### new

**Source**: `src/client.rs`

```rust
pub async fn new( base_url: impl Into<String>, username: Option<String>, password: Option<String>, database: Option<String>, ) -> Result<Self>
```

Create a new Neo4j client with HTTP endpoint.

# Arguments

* `base_url` - Neo4j HTTP endpoint (e.g., "http://localhost:7474")
* `username` - Database username (optional)
* `password` - Database password (optional)
* `database` - Database name (optional, defaults to "neo4j")

---

### new

**Source**: `src/rig_vector_store.rs`

```rust
pub fn new(client: Arc<Neo4jClient>, index_name: String) -> Self
```

Create a new GraphVectorStore with Neo4j client

---

### new

**Source**: `src/vector_store.rs`

```rust
pub async fn new( client: Arc<Neo4jClient>, config: Option<GraphRetrieverConfig>, ) -> Result<Self>
```

Create a new graph retriever with Neo4j client

---

### new

**Source**: `src/document.rs`

```rust
pub fn new(content: impl Into<String>) -> Self
```

Create a new raw text document with automatic ID generation.

---

### new

**Source**: `src/graph.rs`

```rust
pub async fn new(config: GraphMemoryConfig) -> Result<Self>
```

Create a new graph memory instance with configuration.

---

### new_default

**Source**: `src/vector_store.rs`

```rust
pub fn new_default() -> Self
```

Create default configuration

---

### retriever

**Source**: `src/graph.rs`

```rust
pub fn retriever(&self) -> &GraphRetriever
```

Get the underlying graph retriever for advanced operations

---

### search

**Source**: `src/rig_vector_store.rs`

```rust
pub async fn search( &self, query_embedding: Vec<f32>, limit: usize, ) -> Result<Vec<(RigDocument, f32)>>
```

Search for similar documents using vector similarity

---

### search

**Source**: `src/graph.rs`

```rust
pub async fn search( &self, query_embedding: &[f32], limit: usize, ) -> Result<crate::vector_store::GraphSearchResult>
```

Search for documents using hybrid vector + graph search

---

### search_with_graph_context

**Source**: `src/vector_store.rs`

```rust
pub async fn search_with_graph_context( &self, query_embedding: &[f32], limit: usize, ) -> Result<GraphSearchResult>
```

Perform hybrid vector + graph search

---

### simple_query

**Source**: `src/client.rs`

```rust
pub async fn simple_query(&self, query: &str) -> Result<Vec<Value>>
```

Execute a simple read query and return the first column of results

---

### top_n_ids

**Source**: `src/vector_store.rs`

```rust
pub async fn top_n_ids(&self, query_embedding: &[f32], n: usize) -> Result<Vec<String>>
```

Get top N document IDs for a query embedding

---

### with_defaults

**Source**: `src/graph.rs`

```rust
pub async fn with_defaults(neo4j_url: impl Into<String>) -> Result<Self>
```

Create a new instance with default configuration.

---

### with_dimension

**Source**: `src/rig_vector_store.rs`

```rust
pub fn with_dimension(client: Arc<Neo4jClient>, index_name: String, dimension: usize) -> Self
```

Create GraphVectorStore with custom embedding dimension

---

### with_metadata

**Source**: `src/document.rs`

```rust
pub fn with_metadata(content: impl Into<String>, metadata: DocumentMetadata) -> Self
```

Create a document with metadata.

---

### with_source

**Source**: `src/document.rs`

```rust
pub fn with_source(content: impl Into<String>, source: DocumentSource) -> Self
```

Create a document with a specific source.

---

### word_count

**Source**: `src/document.rs`

```rust
pub fn word_count(&self) -> usize
```

Get document word count

---

## Enums

### AmountType

**Source**: `src/document.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
```

```rust
pub enum AmountType { /// Account or wallet balance Balance, /// Token or asset price Price, /// Transaction or gas fee Fee, /// Trading volume Volume, /// Market capitalization MarketCap, /// Other amount type Other(String), }
```

Type of amount

**Variants**:

- `Balance`
- `Price`
- `Fee`
- `Volume`
- `MarketCap`
- `Other(String)`

---

### DocumentSource

**Source**: `src/document.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
```

```rust
pub enum DocumentSource { /// User-provided text input UserInput, /// On-chain transaction data OnChain { /// Blockchain network name (e.g., "ethereum", "solana") chain: String, /// Transaction hash or ID transaction_hash: String, }, /// Social media post (Twitter, Discord, etc.) Social { /// Social media platform name platform: String, /// Post or message ID post_id: String, /// Author username or handle author: Option<String>, }, /// News article or blog post News { /// Article URL url: String, /// Publication or website name publication: Option<String>, }, /// API response or structured data ApiResponse { /// API endpoint URL or identifier endpoint: String, /// When the data was retrieved timestamp: chrono::DateTime<chrono::Utc>, }, /// Other sources Other(String), }
```

Source of the document

**Variants**:

- `UserInput`
- `OnChain`
- `chain`
- `transaction_hash`
- `Social`
- `platform`
- `post_id`
- `author`
- `News`
- `url`
- `publication`
- `ApiResponse`
- `endpoint`
- `timestamp`
- `Other(String)`

---

### EntityType

**Source**: `src/document.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
```

```rust
pub enum EntityType { /// Cryptocurrency wallet address Wallet, /// Token contract or symbol Token, /// DeFi protocol or dApp Protocol, /// Blockchain network Chain, /// Other entity type Other(String), }
```

Type of entity

**Variants**:

- `Wallet`
- `Token`
- `Protocol`
- `Chain`
- `Other(String)`

---

### GraphMemoryError

**Source**: `src/error.rs`

**Attributes**:
```rust
#[derive(Error, Debug)]
```

```rust
pub enum GraphMemoryError { /// Database connection error #[error("Database error: {0}")] Database(String), /// Query execution failed #[error("Query error: {0}")] Query(String), /// Entity extraction failed #[error("Entity extraction error: {0}")] EntityExtraction(String), /// Vector embedding failed #[error("Embedding error: {0}")] Embedding(String), /// HTTP request error #[error("HTTP error: {0}")] Http(#[from] reqwest::Error), /// Serialization error #[error("Serialization error: {0}")] Serialization(#[from] serde_json::Error), /// Core riglr error #[error("Core error: {0}")] Core(#[from] riglr_core::CoreError), /// Generic error #[error("Graph memory error: {0}")] Generic(String), }
```

Main error type for graph memory operations.

**Variants**:

- `Database(String)`
- `Query(String)`
- `EntityExtraction(String)`
- `Embedding(String)`
- `Http(#[from] reqwest::Error)`
- `Serialization(#[from] serde_json::Error)`
- `Core(#[from] riglr_core::CoreError)`
- `Generic(String)`

---

### RelationshipType

**Source**: `src/document.rs`

**Attributes**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
```

```rust
pub enum RelationshipType { /// One wallet transferred to another Transferred, /// Wallet interacted with protocol Interacted, /// Entity holds or owns another entity Holds, /// Token is part of protocol PartOf, /// Protocol deployed on chain DeployedOn, /// Generic relationship Related, }
```

Type of relationship

**Variants**:

- `Transferred`
- `Interacted`
- `Holds`
- `PartOf`
- `DeployedOn`
- `Related`

---

## Traits

### VectorStore

**Source**: `src/rig_vector_store.rs`

**Attributes**:
```rust
#[cfg(feature = "rig")]
#[async_trait]
```

```rust
pub trait VectorStore: Send + Sync { ... }
```

Trait for vector stores that can be used with rig agents
This trait defines the interface for storing and retrieving documents with embeddings

**Methods**:

#### `add_documents`

```rust
async fn add_documents( &self, documents: Vec<Self::Document>, ) -> std::result::Result<Vec<String>, Self::Error>;
```

#### `search`

```rust
async fn search( &self, query_embedding: Vec<f32>, limit: usize, ) -> std::result::Result<Vec<(Self::Document, f32)>, Self::Error>;
```

#### `get_documents`

```rust
async fn get_documents( &self, ids: Vec<String>, ) -> std::result::Result<Vec<Self::Document>, Self::Error>;
```

#### `delete_documents`

```rust
async fn delete_documents( &self, ids: Vec<String>, ) -> std::result::Result<Vec<String>, Self::Error>;
```

---

## Constants

### VERSION

**Source**: `src/lib.rs`

```rust
const VERSION: &str
```

Current version of riglr-graph-memory

---


---

*This documentation was automatically generated from the source code.*