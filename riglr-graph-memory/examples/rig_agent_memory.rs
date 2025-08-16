//! Example: rig Agent with GraphVectorStore Memory
//!
//! This example demonstrates how to use riglr-graph-memory as a vector store
//! for rig agents, providing graph-based persistent memory capabilities.

use std::collections::HashMap;
use std::sync::Arc;
use tracing::{info, warn};

const NEO4J_URL: &str = "NEO4J_URL";
const NEO4J_USER: &str = "NEO4J_USER";
const NEO4J_PASSWORD: &str = "NEO4J_PASSWORD";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    info!("🚀 Starting rig agent with graph memory example");

    // For this demonstration, we'll show the basic workflow using the GraphVectorStore
    {
        use riglr_graph_memory::{GraphVectorStore, RigDocument};
        use serde_json::json;
        // Setup Neo4j connection
        let neo4j_url =
            std::env::var(NEO4J_URL).unwrap_or_else(|_| "bolt://localhost:7687".to_string());
        let _neo4j_user = std::env::var(NEO4J_USER).unwrap_or_else(|_| "neo4j".to_string());
        let _neo4j_password =
            std::env::var(NEO4J_PASSWORD).unwrap_or_else(|_| "password".to_string());

        info!("Connecting to Neo4j at {}", neo4j_url);

        // For now, we'll create a mock Neo4j client since the actual implementation
        // would require a working Neo4j connection
        let neo4j_client = create_mock_neo4j_client().await?;
        let graph_vector_store = Arc::new(GraphVectorStore::new(
            neo4j_client,
            "agent_memory".to_string(),
        ));

        // Demonstrate adding documents to the vector store
        info!("Adding sample documents to graph memory");

        let sample_documents = vec![
            RigDocument {
                id: "crypto_001".to_string(),
                content: "Bitcoin reached a new all-time high of $73,000, driven by institutional adoption and ETF approvals.".to_string(),
                embedding: generate_mock_embedding(),
                metadata: {
                    let mut meta = HashMap::new();
                    meta.insert("topic".to_string(), json!("cryptocurrency"));
                    meta.insert("sentiment".to_string(), json!("positive"));
                    meta.insert("date".to_string(), json!("2024-03-14"));
                    meta
                },
            },
            RigDocument {
                id: "defi_001".to_string(),
                content: "Uniswap v4 introduces hooks functionality, allowing developers to customize pool behavior and create innovative DeFi products.".to_string(),
                embedding: generate_mock_embedding(),
                metadata: {
                    let mut meta = HashMap::new();
                    meta.insert("topic".to_string(), json!("defi"));
                    meta.insert("protocol".to_string(), json!("uniswap"));
                    meta.insert("version".to_string(), json!("v4"));
                    meta
                },
            },
            RigDocument {
                id: "nft_001".to_string(),
                content: "The NFT market shows signs of recovery with increased trading volume and new innovative use cases in gaming and virtual worlds.".to_string(),
                embedding: generate_mock_embedding(),
                metadata: {
                    let mut meta = HashMap::new();
                    meta.insert("topic".to_string(), json!("nft"));
                    meta.insert("trend".to_string(), json!("recovery"));
                    meta.insert("sector".to_string(), json!("gaming"));
                    meta
                },
            },
        ];

        match graph_vector_store.add_documents(sample_documents).await {
            Ok(ids) => info!("Successfully added {} documents to vector store", ids.len()),
            Err(e) => warn!("Failed to add documents: {}", e),
        }

        // Demonstrate vector search
        info!("Performing vector search for DeFi-related content");
        let query_embedding = generate_mock_embedding(); // In real usage, this would be from an embedding model

        match graph_vector_store.search(query_embedding, 2).await {
            Ok(results) => {
                info!("Vector search returned {} results:", results.len());
                for (doc, score) in results {
                    info!("  📄 Document: {} (score: {:.3})", doc.id, score);
                    info!("     Content: {}", truncate_string(&doc.content, 100));
                    if let Some(topic) = doc.metadata.get("topic") {
                        info!("     Topic: {}", topic);
                    }
                }
            }
            Err(e) => warn!("Vector search failed: {}", e),
        }

        // Demonstrate document retrieval
        info!("Retrieving specific documents by ID");
        let doc_ids = vec!["crypto_001".to_string(), "defi_001".to_string()];

        match graph_vector_store.get_documents(doc_ids).await {
            Ok(docs) => {
                info!("Retrieved {} documents:", docs.len());
                for doc in docs {
                    info!("  📄 {}: {}", doc.id, truncate_string(&doc.content, 80));
                }
            }
            Err(e) => warn!("Document retrieval failed: {}", e),
        }

        // Note: In a real application, you would now create a rig agent like this:
        /*
        use rig::agent::AgentBuilder;

        let agent = AgentBuilder::new("gpt-4")
            .preamble("You are a cryptocurrency and DeFi expert with access to a graph-based knowledge store.")
            .vector_store(graph_vector_store)
            .build();

        let response = agent.prompt("What are the latest trends in DeFi?").await?;
        info!("Agent response: {}", response);
        */

        info!("✅ Graph memory example completed successfully!");
        info!("💡 To use with a real rig agent, uncomment the agent creation code above.");
    }

    Ok(())
}

// Mock Neo4j client for demonstration purposes
async fn create_mock_neo4j_client(
) -> Result<Arc<riglr_graph_memory::Neo4jClient>, Box<dyn std::error::Error>> {
    // In a real implementation, this would connect to Neo4j:
    // let client = Neo4jClient::new(&neo4j_url, &neo4j_user, &neo4j_password).await?;

    // For the example, we'll create a mock client that simulates Neo4j responses
    info!("ℹ️  Using mock Neo4j client for demonstration");
    info!("   In production, ensure Neo4j is running and accessible");

    // This is a placeholder - the actual implementation would use the real Neo4j client
    let mock_client = riglr_graph_memory::Neo4jClient::default();
    Ok(Arc::new(mock_client))
}

// Generate a mock embedding vector for demonstration
fn generate_mock_embedding() -> Vec<f32> {
    // In a real application, this would come from an embedding model like OpenAI's text-embedding-ada-002
    (0..1536).map(|i| (i as f32 * 0.001) % 1.0).collect()
}

// Utility function to truncate strings for display
fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_embedding_generation() {
        let embedding = generate_mock_embedding();
        assert_eq!(embedding.len(), 1536);
        assert!(embedding.iter().all(|&x| x >= 0.0 && x <= 1.0));
    }

    #[test]
    fn test_string_truncation() {
        let short_text = "Short text";
        let long_text =
            "This is a very long text that should be truncated when it exceeds the maximum length";

        assert_eq!(truncate_string(short_text, 50), short_text);
        assert_eq!(truncate_string(long_text, 20), "This is a very long ...");
    }
}
