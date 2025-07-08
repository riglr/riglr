//! # riglr-graph-memory
//!
//! Advanced graph-based memory system for riglr agents with rig::VectorStore implementation.
//!
//! This crate provides a sophisticated knowledge graph backend that can store and query
//! complex relationships between on-chain entities, enabling agents to build rich,
//! contextual understanding of blockchain ecosystems.
//!
//! ## Features
//!
//! - **Graph Database Backend**: Neo4j integration for storing entity relationships
//! - **Vector Search**: Hybrid vector + graph search capabilities
//! - **Entity Extraction**: Automatic entity and relationship extraction from text
//! - **rig Integration**: Implements `rig::VectorStore` for seamless agent integration
//! - **Rich Queries**: Complex graph traversal and pattern matching
//! - **Scalable**: Designed for production workloads with proper indexing
//!
//! ## Architecture
//!
//! The graph memory system uses a hybrid approach:
//! 1. **Entity Storage**: Nodes represent blockchain entities (wallets, tokens, protocols)
//! 2. **Relationship Mapping**: Edges capture interactions and dependencies
//! 3. **Vector Indexing**: Text embeddings for semantic search
//! 4. **Query Engine**: Cypher-based queries with vector similarity
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use riglr_graph_memory::{GraphMemory, RawTextDocument};
//! use rig_core::agents::Agent;
//!
//! # async fn example() -> anyhow::Result<()> {
//! // Initialize graph memory with Neo4j connection
//! let memory = GraphMemory::new("neo4j://localhost:7687").await?;
//!
//! // Create an agent with graph memory
//! let agent = Agent::builder()
//!     .preamble("You are a blockchain analyst with access to transaction history.")
//!     .dynamic_context(2, memory) // Use graph as vector store
//!     .build();
//!
//! // Add some transaction data to the graph
//! let doc = RawTextDocument::new("Wallet 0xABC123 swapped 100 SOL for USDC on Jupiter");
//! memory.add_documents(vec![doc]).await?;
//!
//! let response = agent.prompt("What protocols has wallet 0xABC123 used?").await?;
//! println!("Agent response: {}", response);
//! # Ok(())
//! # }
//! ```
//!
//! ## Data Model
//!
//! The graph uses a standardized schema:
//!
//! - `(Wallet)` - Blockchain addresses/accounts
//! - `(Token)` - Fungible and non-fungible tokens  
//! - `(Protocol)` - DeFi protocols and applications
//! - `(Transaction)` - On-chain transactions
//! - `(Block)` - Blockchain blocks
//!
//! Relationships include:
//! - `(Wallet)-[:PERFORMED]->(Transaction)`
//! - `(Transaction)-[:INVOLVED]->(Token)`
//! - `(Transaction)-[:USED]->(Protocol)`
//! - `(Wallet)-[:HOLDS]->(Token)`

pub mod client;
pub mod document;
pub mod error;
pub mod extractor;
pub mod graph;
pub mod vector_store;

// rig integration - experimental implementation
pub mod rig_vector_store;

// Re-export main types
pub use client::Neo4jClient;
pub use document::RawTextDocument;
pub use error::{GraphMemoryError, Result};
pub use extractor::EntityExtractor;
pub use graph::GraphMemory;
pub use vector_store::GraphRetriever;

// Export rig integration components when feature is enabled
pub use rig_vector_store::{GraphVectorStore, RigDocument};

/// Current version of riglr-graph-memory  
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }
}
