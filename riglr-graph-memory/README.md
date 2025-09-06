# riglr-graph-memory

Advanced graph-based memory system for riglr agents, providing persistent knowledge graphs and semantic search capabilities through Neo4j integration.

## Features

- 🧠 **Knowledge Graph Storage**: Neo4j backend for complex entity relationships
- 🔍 **Hybrid Search**: Combines vector similarity with graph traversal
- 🤖 **Entity Extraction**: Automatic extraction of blockchain entities from text
- 🔗 **rig Integration**: Implements `rig::VectorStore` for seamless agent memory
- 📊 **Rich Queries**: Complex pattern matching and multi-hop traversals
- ⚡ **High Performance**: Optimized indexing and caching strategies
- 🛡️ **Production Ready**: Connection pooling, retry logic, and error handling

## Architecture

riglr-graph-memory provides a sophisticated graph-based memory system for AI agents, combining knowledge graphs with vector embeddings for enhanced contextual retrieval.

### Design Principles

- **Graph-First**: Relationships are first-class citizens, not afterthoughts
- **Entity-Centric**: Automatic extraction and linking of blockchain entities
- **Hybrid Retrieval**: Combines graph traversal with vector similarity search
- **Blockchain-Aware**: Built-in understanding of wallets, tokens, protocols, and transactions
- **Extensible**: Custom entity patterns and relationship types can be added
- **VectorStore Compatible**: Implements rig's VectorStore trait for agent integration

### Core Components

1. **GraphMemory**: High-level interface for document storage and retrieval
2. **KnowledgeGraph**: Core graph operations and entity relationship management
3. **EntityExtractor**: Pattern-based extraction of blockchain entities from text
4. **GraphRetriever**: VectorStore implementation combining embeddings with graph structure
5. **Neo4jClient**: Low-level Neo4j database interactions with connection pooling

### Data Model

```
Document ──has_entity──> Entity
    │                      │
    └──similar_to──>       └──relates_to──>
        Document               Entity

Entity Types:
- Wallet (addresses)
- Token (symbols, contracts)
- Protocol (DeFi protocols)
- Block (block numbers)
- Transaction (hashes, types)
```

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
riglr-graph-memory = "0.1.0"
```

## Prerequisites

### Neo4j Setup

1. **Using Docker (Recommended)**:
```bash
docker run -d \
  --name neo4j \
  -p 7474:7474 -p 7687:7687 \
  -e NEO4J_AUTH=neo4j/password \
  -e NEO4JLABS_PLUGINS='["apoc","graph-data-science"]' \
  neo4j:5.13.0
```

2. **Using Neo4j Desktop**:
   - Download from [neo4j.com](https://neo4j.com/download/)
   - Create a new project and database
   - Install APOC and GDS plugins

3. **Configuration**:
```bash
export NEO4J_URL=neo4j://localhost:7687
export NEO4J_USERNAME=neo4j
export NEO4J_PASSWORD=password
```

## Quick Start

### Basic Usage

```rust
use riglr_graph_memory::{GraphMemory, GraphMemoryConfig, RawTextDocument};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize graph memory
    let config = GraphMemoryConfig {
        neo4j_url: "neo4j://localhost:7687".to_string(),
        username: Some("neo4j".to_string()),
        password: Some("password".to_string()),
        ..Default::default()
    };
    
    let mut memory = GraphMemory::new(config).await?;
    
    // Add documents to build the knowledge graph
    let doc = RawTextDocument::new(
        "Wallet 0x742d35Cc6634C0532925a3b844Bc9e7595f0eA4B transferred 100 ETH to Uniswap V3"
    );
    
    let doc_id = memory.add_document(doc).await?;
    println!("Document added with ID: {}", doc_id);
    
    // Query the graph
    let related = memory.find_related_documents("Uniswap", Some(5)).await?;
    println!("Found {} related documents", related.len());
    
    Ok(())
}
```

### Integration with rig Agents

```rust
use riglr_graph_memory::{GraphMemory, GraphMemoryConfig};
use rig::agents::Agent;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create graph memory instance
    let config = GraphMemoryConfig::from_env()?;
    let memory = GraphMemory::new(config).await?;
    
    // Create an agent with graph memory
    let agent = Agent::builder()
        .preamble("You are a blockchain analyst with access to transaction history.")
        .dynamic_context(10, memory) // Use graph as vector store
        .build();
    
    // The agent can now retrieve relevant context from the graph
    let response = agent
        .prompt("What are the recent activities of wallet 0x742d35Cc?")
        .await?;
    
    println!("Analysis: {}", response);
    
    Ok(())
}
```

## Core Components

### GraphMemory

The main interface for interacting with the knowledge graph.

```rust
use riglr_graph_memory::{GraphMemory, GraphMemoryConfig};

let mut memory = GraphMemory::new(config).await?;

// Add documents
memory.add_document(doc).await?;
memory.add_documents(vec![doc1, doc2, doc3]).await?;

// Query operations
let docs = memory.find_related_documents("query", Some(10)).await?;
let history = memory.get_wallet_history("0x...").await?;
let stats = memory.get_statistics().await?;

// Advanced queries
let results = memory.execute_cypher(
    "MATCH (w:Wallet)-[:HOLDS]->(t:Token) 
     WHERE t.symbol = $symbol 
     RETURN w.address",
    json!({"symbol": "USDC"})
).await?;
```

### Document Types

```rust
use riglr_graph_memory::{RawTextDocument, DocumentMetadata, DocumentSource};

// Simple document
let doc = RawTextDocument::new("Transaction data here");

// Document with metadata
let mut metadata = DocumentMetadata::default();
metadata.source = Some(DocumentSource::Blockchain {
    chain: "ethereum".to_string(),
    block_number: Some(18500000),
    transaction_hash: Some("0xabc...".to_string()),
});
metadata.tags = vec!["defi".to_string(), "swap".to_string()];

let doc = RawTextDocument::with_metadata("Content", metadata);
```

### Entity Extraction

The system automatically extracts entities from text:

```rust
use riglr_graph_memory::EntityExtractor;

let extractor = EntityExtractor::new();
let entities = extractor.extract(
    "Wallet 0x742d35Cc swapped 100 USDC for ETH on Uniswap at block 18500000"
);

// Extracted entities:
// - Wallet: 0x742d35Cc
// - Tokens: USDC, ETH
// - Protocol: Uniswap
// - Block: 18500000
// - Transaction type: swap
```

### Knowledge Graph

Build and query a knowledge graph:

```rust
use riglr_graph_memory::graph::KnowledgeGraph;

let mut graph = KnowledgeGraph::new(client);

// Add documents to build the graph
let doc1 = RawTextDocument::new(
    "Vitalik's wallet 0x123... holds 1000 ETH"
);
let doc2 = RawTextDocument::new(
    "0x123... transferred 100 ETH to Uniswap"
);

graph.add_document(doc1).await?;
graph.add_document(doc2).await?;

// Query the graph
let wallet_history = graph.get_wallet_history("0x123...").await?;
let token_holders = graph.get_token_holders("ETH").await?;
let protocol_users = graph.get_protocol_interactions("Uniswap").await?;

// Find related documents
let related = graph.find_related_documents(
    "Uniswap",
    Some(10), // max results
).await?;

// Get graph statistics
let stats = graph.get_statistics().await?;
```

### Vector Store Integration

Use with rig's VectorStore trait:

```rust
use riglr_graph_memory::vector_store::{
    GraphRetriever,
    GraphRetrieverConfig,
};

let config = GraphRetrieverConfig {
    similarity_threshold: 0.7,
    max_graph_hops: 2,
    embedding_dimension: 1536,
    index_name: "embedding_index".to_string(),
};

let retriever = GraphRetriever::new(client, config).await?;

// Store document with embedding
let doc = RawTextDocument::new("Content...");
let embedding = vec![0.1; 1536]; // From embedding model

retriever.store_document(&doc, embedding).await?;

// Search by similarity
let query_embedding = vec![0.2; 1536]; // Query embedding
let results = retriever.search_similar(
    query_embedding,
    Some(10), // limit
    Some(0.7), // threshold
).await?;

for (document, score) in results {
    println!("Document: {} (similarity: {})", document.id, score);
}
```

## Advanced Features

### Custom Entity Patterns

Add custom patterns for entity extraction:

```rust
let mut extractor = EntityExtractor::new();

// Add custom protocol patterns
extractor.add_protocol_pattern(
    "MyProtocol",
    vec!["myproto", "MyProto", "MYPROTO"],
);

// Add custom token patterns
extractor.add_token_pattern(
    "MYTOKEN",
    vec!["$MYTOKEN", "MYTOKEN", "MyToken"],
);
```

### Graph Traversal

Perform complex graph queries:

```rust
// Find all paths between two wallets
let paths = graph.find_paths(
    "0x123...",
    "0x456...",
    Some(3), // max depth
).await?;

// Get transaction flow
let flow = graph.get_transaction_flow(
    "0x123...",
    Some(chrono::Duration::days(7)), // time window
).await?;

// Analyze protocol connections
let connections = graph.analyze_protocol_connections(
    "Uniswap",
    Some(2), // depth
).await?;
```

### Batch Processing

Process multiple documents efficiently:

```rust
let documents = vec![
    RawTextDocument::new("Doc 1"),
    RawTextDocument::new("Doc 2"),
    RawTextDocument::new("Doc 3"),
];

let results = graph.add_documents_batch(documents).await?;
println!("Added {} documents", results.len());
```

## Performance Optimization

### Indexing

Create appropriate indexes for your use case:

```rust
// Create vector similarity index
client.execute_query(
    r#"
    CREATE VECTOR INDEX embedding_index IF NOT EXISTS
    FOR (n:Document) ON (n.embedding)
    OPTIONS {
        indexConfig: {
            `vector.dimensions`: 1536,
            `vector.similarity_function`: 'cosine'
        }
    }
    "#,
    None,
).await?;

// Create composite indexes
client.execute_query(
    "CREATE INDEX wallet_chain_idx FOR (n:Wallet) ON (n.address, n.chain)",
    None,
).await?;
```

### Connection Pooling

Use connection pooling for high throughput:

```rust
let client = Neo4jClient::new_with_pool(
    "http://localhost:7474",
    Some("neo4j".to_string()),
    Some("password".to_string()),
    None,
    10, // max connections
).await?;
```

## Testing

Run tests with Docker:

```bash
# Start Neo4j container
docker run -d --name neo4j-test \
  -p 7474:7474 -p 7687:7687 \
  -e NEO4J_AUTH=neo4j/testpass \
  neo4j:5.13.0

# Run tests
cargo test -p riglr-graph-memory

# Run integration tests
cargo test -p riglr-graph-memory --test integration_tests -- --ignored
```

## Security

- Never expose Neo4j credentials in code
- Use environment variables for configuration
- Enable SSL/TLS for production deployments
- Implement query timeouts to prevent DoS
- Sanitize user input to prevent Cypher injection

## License

MIT

## Contributing

Contributions are welcome! Please ensure all tests pass and add new tests for any new functionality.