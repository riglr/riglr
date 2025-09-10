# riglr-graph-memory

{{#include ../../../riglr-graph-memory/README.md}}

## API Reference

### Contents

- [Structs](#structs)
- [Enums](#enums)
- [Traits](#traits)
- [Type Aliases](#type-aliases)
- [Constants](#constants)

### Structs

> Core data structures and types.

#### `AmountMention`

A numerical amount mentioned in the document

---

#### `DocumentMetadata`

Metadata associated with a document

---

#### `EntityExtractor`

Production-grade entity extractor for blockchain text analysis

---

#### `EntityMention`

An entity mention in the document

---

#### `ExtractedEntities`

Extracted entities from a document

---

#### `GraphDocument`

A document stored in the graph with vector embeddings

---

#### `GraphMemory`

The main graph memory system that provides comprehensive document storage,
entity extraction, and hybrid vector + graph search capabilities.

---

#### `GraphMemoryConfig`

Configuration for the graph memory system

---

#### `GraphMemoryStats`

Statistics about the graph memory system

---

#### `GraphRetriever`

A retriever that combines graph and vector search for enhanced context.

This implementation provides sophisticated document retrieval by leveraging both
vector similarity search and graph relationships to find the most relevant context
for agent queries.

---

#### `GraphRetrieverConfig`

Configuration for graph-based vector retrieval

---

#### `GraphSearchResult`

Search result from graph vector store

---

#### `GraphVectorStore`

Graph-based vector store implementation for rig

This implementation leverages Neo4j's vector capabilities combined with graph traversal
to provide enhanced contextual search for rig agents.

---

#### `Neo4jClient`

Neo4j database client using HTTP REST API.

This client provides production-grade connectivity to Neo4j databases
with proper error handling, authentication, and query optimization.

---

#### `RawTextDocument`

A raw text document that can be added to the graph memory system.

This document type supports blockchain-specific metadata and automatic
entity extraction to populate the knowledge graph.

---

#### `RelationshipMention`

A relationship between entities

---

#### `RigDocument`

Document type that bridges between rig and our graph memory system

---

#### `SearchMetrics`

Performance metrics for graph search operations

---

### Enums

> Enumeration types for representing variants.

#### `AmountType`

Type of amount

**Variants:**

- `Balance`
  - Account or wallet balance
- `Price`
  - Token or asset price
- `Fee`
  - Transaction or gas fee
- `Volume`
  - Trading volume
- `MarketCap`
  - Market capitalization
- `Other`
  - Other amount type

---

#### `DocumentSource`

Source of the document

**Variants:**

- `UserInput`
  - User-provided text input
- `OnChain`
  - On-chain transaction data
- `Social`
  - Social media post (Twitter, Discord, etc.)
- `News`
  - News article or blog post
- `ApiResponse`
  - API response or structured data
- `Other`
  - Other sources

---

#### `EntityType`

Type of entity

**Variants:**

- `Wallet`
  - Cryptocurrency wallet address
- `Token`
  - Token contract or symbol
- `Protocol`
  - DeFi protocol or dApp
- `Chain`
  - Blockchain network
- `Other`
  - Other entity type

---

#### `GraphMemoryError`

Main error type for graph memory operations.

**Variants:**

- `Database`
  - Database connection error
- `Query`
  - Query execution failed
- `EntityExtraction`
  - Entity extraction failed
- `Embedding`
  - Vector embedding failed
- `Http`
  - HTTP request error
- `Serialization`
  - Serialization error
- `Core`
  - Core riglr error
- `Generic`
  - Generic error

---

#### `RelationshipType`

Type of relationship

**Variants:**

- `Transferred`
  - One wallet transferred to another
- `Interacted`
  - Wallet interacted with protocol
- `Holds`
  - Entity holds or owns another entity
- `PartOf`
  - Token is part of protocol
- `DeployedOn`
  - Protocol deployed on chain
- `Related`
  - Generic relationship

---

### Traits

> Trait definitions for implementing common behaviors.

#### `VectorStore`

Trait for vector stores that can be used with rig agents
This trait defines the interface for storing and retrieving documents with embeddings

**Methods:**

- `add_documents()`
  - Add documents to the vector store
- `search()`
  - Search for similar documents using vector similarity
- `get_documents()`
  - Get documents by their IDs
- `delete_documents()`
  - Delete documents by their IDs

---

### Type Aliases

#### `Result`

Result type alias for graph memory operations.

**Type:** `<T, >`

---

### Constants

#### `VERSION`

Current version of riglr-graph-memory

**Type:** `&str`

---
