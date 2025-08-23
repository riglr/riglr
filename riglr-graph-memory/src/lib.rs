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

/// Rig integration module providing VectorStore implementation
///
/// This module provides experimental integration with the rig framework,
/// implementing `VectorStore` trait to enable seamless use of graph memory
/// with rig-based AI agents. The implementation bridges between rig's
/// document model and our graph-based storage system.
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
        assert!(
            VERSION.starts_with("0.") || VERSION.starts_with("1."),
            "VERSION should be a valid semver"
        );
    }

    #[test]
    fn test_version_when_valid_semver_should_pass() {
        // Happy path: Test that VERSION is not empty and contains valid characters
        assert!(!VERSION.is_empty(), "VERSION should not be empty");
        assert!(
            VERSION.contains('.'),
            "VERSION should contain dot separators"
        );

        // Verify it follows basic semver format (at least X.Y pattern)
        let parts: Vec<&str> = VERSION.split('.').collect();
        assert!(
            parts.len() >= 2,
            "VERSION should have at least major.minor format"
        );

        // Verify major version is numeric
        assert!(
            parts[0].parse::<u32>().is_ok(),
            "Major version should be numeric"
        );

        // Verify minor version is numeric (may contain additional text for pre-release)
        let minor_numeric = parts[1].split('-').next().unwrap_or(parts[1]);
        assert!(
            minor_numeric.parse::<u32>().is_ok(),
            "Minor version should start with numeric"
        );
    }

    #[test]
    fn test_version_when_checked_for_length_should_be_reasonable() {
        // Edge case: VERSION should not be unreasonably long
        assert!(VERSION.len() < 100, "VERSION should be reasonable length");
        assert!(VERSION.len() > 0, "VERSION should not be empty");
    }

    #[test]
    fn test_version_when_checked_for_ascii_should_be_valid() {
        // Edge case: VERSION should only contain valid ASCII characters
        assert!(VERSION.is_ascii(), "VERSION should be ASCII");

        // Should not contain whitespace
        assert!(!VERSION.contains(' '), "VERSION should not contain spaces");
        assert!(!VERSION.contains('\t'), "VERSION should not contain tabs");
        assert!(
            !VERSION.contains('\n'),
            "VERSION should not contain newlines"
        );
    }

    #[test]
    fn test_version_when_compared_to_common_patterns_should_match() {
        // Test various version patterns that should be valid
        let valid_patterns = [
            VERSION.starts_with("0."),
            VERSION.starts_with("1."),
            VERSION.starts_with("2."),
            VERSION.starts_with("3."),
            VERSION.starts_with("4."),
            VERSION.starts_with("5."),
            VERSION.starts_with("6."),
            VERSION.starts_with("7."),
            VERSION.starts_with("8."),
            VERSION.starts_with("9."),
        ];

        // At least one pattern should match
        assert!(
            valid_patterns.iter().any(|&x| x),
            "VERSION should start with a valid digit"
        );
    }

    #[test]
    fn test_version_constant_accessibility() {
        // Test that VERSION constant is accessible and has expected properties
        let version_str = VERSION;
        assert_eq!(version_str, VERSION, "VERSION should be consistent");

        // Test that it can be used in string operations
        let version_string = format!("Version: {}", VERSION);
        assert!(
            version_string.contains("Version: "),
            "VERSION should be usable in string formatting"
        );
        assert!(
            version_string.len() > "Version: ".len(),
            "Formatted string should contain actual version"
        );
    }

    #[test]
    fn test_module_re_exports_accessibility() {
        // Test that re-exported types are accessible (compilation test)
        // This ensures the module structure and re-exports are working correctly

        // These should compile without error, testing the re-export paths
        let _client_type = std::marker::PhantomData::<Neo4jClient>;
        let _document_type = std::marker::PhantomData::<RawTextDocument>;
        let _error_type = std::marker::PhantomData::<GraphMemoryError>;
        let _result_type = std::marker::PhantomData::<Result<()>>;
        let _extractor_type = std::marker::PhantomData::<EntityExtractor>;
        let _graph_type = std::marker::PhantomData::<GraphMemory>;
        let _retriever_type = std::marker::PhantomData::<GraphRetriever>;
        let _vector_store_type = std::marker::PhantomData::<GraphVectorStore>;
        let _rig_doc_type = std::marker::PhantomData::<RigDocument>;

        // If this test compiles and runs, all re-exports are working
    }

    #[test]
    fn test_version_edge_cases() {
        // Edge case: Test VERSION against potential edge case values
        assert!(!VERSION.is_empty(), "VERSION should never be empty");

        // Should not be just dots
        assert_ne!(VERSION, ".", "VERSION should not be just a dot");
        assert_ne!(VERSION, "..", "VERSION should not be just dots");
        assert_ne!(VERSION, "...", "VERSION should not be just dots");

        // Should not start or end with dot
        assert!(
            !VERSION.starts_with('.'),
            "VERSION should not start with dot"
        );
        assert!(!VERSION.ends_with('.'), "VERSION should not end with dot");

        // Should contain actual content between dots
        let parts: Vec<&str> = VERSION.split('.').collect();
        for part in parts.iter() {
            assert!(
                !part.is_empty(),
                "VERSION parts should not be empty between dots"
            );
        }
    }

    #[test]
    fn test_crate_metadata_consistency() {
        // Test that VERSION matches expected crate metadata patterns
        // This is a comprehensive test for the env!("CARGO_PKG_VERSION") macro

        // The version should be the same as what Cargo would report
        let version = VERSION;

        // Basic validation that this is a proper Cargo package version
        assert!(
            version.chars().next().unwrap().is_ascii_digit(),
            "VERSION should start with a digit"
        );

        // Should have at least one dot (major.minor minimum)
        assert!(
            version.matches('.').count() >= 1,
            "VERSION should have at least major.minor format"
        );

        // Each component before pre-release markers should be numeric
        let base_version = version.split('-').next().unwrap_or(version);
        let base_version = base_version.split('+').next().unwrap_or(base_version);

        for component in base_version.split('.') {
            assert!(
                component.parse::<u32>().is_ok(),
                "VERSION component '{}' should be numeric",
                component
            );
        }
    }
}
