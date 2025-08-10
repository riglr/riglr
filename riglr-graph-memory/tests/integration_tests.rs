//! Integration tests for riglr-graph-memory
//!
//! These tests verify the functionality of the graph memory system
//! with a real or mocked Neo4j instance.

use riglr_graph_memory::{
    client::Neo4jClient,
    document::{DocumentMetadata, DocumentSource, RawTextDocument},
    extractor::EntityExtractor,
    graph::{GraphMemory, GraphMemoryConfig},
};
use testcontainers::{runners::AsyncRunner, Container, GenericImage};

/// Helper to start a Neo4j test container
async fn start_neo4j_container() -> Container<GenericImage> {
    let neo4j_image = GenericImage::new("neo4j", "5.13.0")
        .with_env_var("NEO4J_AUTH", "neo4j/testpassword")
        .with_env_var("NEO4JLABS_PLUGINS", "[\"apoc\",\"graph-data-science\"]")
        .with_exposed_port(7474)
        .with_exposed_port(7687);

    neo4j_image.start().await.expect("Failed to start Neo4j container")
}

/// Helper to create a test Neo4j client
async fn create_test_client(container: &Container<GenericImage>) -> Neo4jClient {
    let http_port = container.get_host_port_ipv4(7474).await.expect("Failed to get port");
    let url = format!("http://localhost:{}", http_port);

    // Wait for Neo4j to be ready
    tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;

    Neo4jClient::new(
        url,
        Some("neo4j".to_string()),
        Some("testpassword".to_string()),
        None,
    )
    .await
    .expect("Failed to create Neo4j client")
}

/// Helper to create test GraphMemoryConfig
async fn create_test_config(container: &Container<GenericImage>) -> GraphMemoryConfig {
    let http_port = container.get_host_port_ipv4(7474).await.expect("Failed to get port");
    let url = format!("http://localhost:{}", http_port);

    GraphMemoryConfig {
        neo4j_url: url,
        username: Some("neo4j".to_string()),
        password: Some("testpassword".to_string()),
        database: Some("neo4j".to_string()),
        ..Default::default()
    }
}

#[cfg(test)]
mod client_tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires Docker
    async fn test_neo4j_connection() {
        let container = start_neo4j_container().await;
        let client = create_test_client(&container).await;

        // Test basic query
        let result = client
            .query("RETURN 1 as number", serde_json::json!({}))
            .await;

        assert!(result.is_ok());
        let records = result.unwrap();
        assert_eq!(records.len(), 1);
    }

    #[tokio::test]
    #[ignore] // Requires Docker
    async fn test_create_and_query_node() {
        let container = start_neo4j_container().await;
        let client = create_test_client(&container).await;

        // Create a test node
        let create_result = client
            .query(
                "CREATE (n:TestNode {name: $name, value: $value}) RETURN n",
                serde_json::json!({
                    "name": "test_node",
                    "value": 42
                }),
            )
            .await;

        assert!(create_result.is_ok());

        // Query for the node
        let query_result = client
            .query(
                "MATCH (n:TestNode {name: $name}) RETURN n.value as value",
                serde_json::json!({
                    "name": "test_node"
                }),
            )
            .await;

        assert!(query_result.is_ok());
        let records = query_result.unwrap();
        assert_eq!(records.len(), 1);
    }

    #[tokio::test]
    #[ignore] // Requires Docker
    async fn test_create_relationship() {
        let container = start_neo4j_container().await;
        let client = create_test_client(&container).await;

        // Create nodes and relationship
        let result = client
            .query(
                r#"
                CREATE (a:Wallet {address: $addr1})
                CREATE (b:Token {symbol: $symbol})
                CREATE (a)-[r:HOLDS {amount: $amount}]->(b)
                RETURN r
                "#,
                serde_json::json!({
                    "addr1": "0x123...",
                    "symbol": "USDC",
                    "amount": 1000
                }),
            )
            .await;

        assert!(result.is_ok());
    }
}

#[cfg(test)]
mod entity_extractor_tests {
    use super::*;

    #[test]
    fn test_extract_wallet_addresses() {
        let extractor = EntityExtractor::new();
        let text = "Wallet 0x742d35Cc6634C0532925a3b844Bc9e7595f0eA4B transferred 100 ETH";

        let entities = extractor.extract(text);

        assert!(!entities.wallets.is_empty());
        assert_eq!(
            entities.wallets[0].address,
            "0x742d35Cc6634C0532925a3b844Bc9e7595f0eA4B"
        );
    }

    #[test]
    fn test_extract_tokens() {
        let extractor = EntityExtractor::new();
        let text = "Swapped 100 USDC for 0.05 ETH on Uniswap";

        let entities = extractor.extract(text);

        assert!(entities.tokens.len() >= 2);
        assert!(entities
            .tokens
            .iter()
            .any(|t| t.symbol == "USDC" || t.symbol == "ETH"));
    }

    #[test]
    fn test_extract_protocols() {
        let extractor = EntityExtractor::new();
        let text = "Used Uniswap to swap tokens, then deposited on Aave for lending";

        let entities = extractor.extract(text);

        assert!(entities.protocols.len() >= 2);
        assert!(entities
            .protocols
            .iter()
            .any(|p| p.name.to_lowercase().contains("uniswap")));
        assert!(entities
            .protocols
            .iter()
            .any(|p| p.name.to_lowercase().contains("aave")));
    }

    #[test]
    fn test_extract_transactions() {
        let extractor = EntityExtractor::new();
        let text = "Transaction 0xabc123def456 confirmed: swapped 100 USDC for ETH";

        let entities = extractor.extract(text);

        assert!(!entities.transactions.is_empty());
        assert!(entities.transactions[0]
            .hash
            .as_ref()
            .unwrap()
            .contains("0xabc123"));
    }

    #[test]
    fn test_extract_relationships() {
        let extractor = EntityExtractor::new();
        let text =
            "Wallet 0x742d35Cc6634C0532925a3b844Bc9e7595f0eA4B holds 1000 USDC and staked on Aave";

        let entities = extractor.extract(text);

        assert!(!entities.relationships.is_empty());
        // Should detect "holds" and "staked" relationships
        assert!(entities.relationships.len() >= 1);
    }
}

#[cfg(test)]
mod document_tests {
    use super::*;

    #[test]
    fn test_raw_document_creation() {
        let doc = RawTextDocument::new("Test content");

        assert_eq!(doc.content, "Test content");
        assert!(!doc.id.is_empty());
        assert!(doc.metadata.is_none());
    }

    #[test]
    fn test_document_with_metadata() {
        let mut metadata = DocumentMetadata::default();
        metadata.source = Some(DocumentSource::Api {
            endpoint: "https://api.example.com".to_string(),
            method: "GET".to_string(),
        });
        metadata.tags = vec!["test".to_string(), "example".to_string()];

        let doc = RawTextDocument::with_metadata("Test content", metadata.clone());

        assert_eq!(doc.content, "Test content");
        assert!(doc.metadata.is_some());
        assert_eq!(doc.metadata.unwrap().tags.len(), 2);
    }
}

#[cfg(test)]
mod knowledge_graph_tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires Docker
    async fn test_knowledge_graph_creation() {
        let container = start_neo4j_container().await;
        let config = create_test_config(&container).await;

        // Wait for Neo4j to be ready
        tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;

        let graph = GraphMemory::new(config).await;
        assert!(graph.is_ok());
    }

    #[tokio::test]
    #[ignore] // Requires Docker
    async fn test_knowledge_graph_add_document() {
        let container = start_neo4j_container().await;
        let config = create_test_config(&container).await;

        // Wait for Neo4j to be ready
        tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;

        let mut graph = GraphMemory::new(config).await.unwrap();

        let doc = RawTextDocument::new(
            "Wallet 0x742d35Cc6634C0532925a3b844Bc9e7595f0eA4B holds 1000 USDC",
        );

        let result = graph.add_documents(vec![doc]).await;
        assert!(result.is_ok());

        let doc_ids = result.unwrap();
        assert!(!doc_ids.is_empty());
        assert!(!doc_ids[0].is_empty());
    }

    #[tokio::test]
    #[ignore] // Requires Docker
    async fn test_knowledge_graph_find_related() {
        let container = start_neo4j_container().await;
        let config = create_test_config(&container).await;

        // Wait for Neo4j to be ready
        tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;

        let mut graph = GraphMemory::new(config).await.unwrap();

        // Add multiple related documents
        let doc1 = RawTextDocument::new(
            "Wallet 0x742d35Cc6634C0532925a3b844Bc9e7595f0eA4B holds 1000 USDC",
        );
        let doc2 = RawTextDocument::new(
            "Wallet 0x742d35Cc6634C0532925a3b844Bc9e7595f0eA4B swapped USDC for ETH on Uniswap",
        );

        graph.add_documents(vec![doc1]).await.unwrap();
        graph.add_documents(vec![doc2]).await.unwrap();

        // Search for related documents
        let related = graph
            .search("0x742d35Cc6634C0532925a3b844Bc9e7595f0eA4B", Some(2))
            .await
            .unwrap();

        assert!(related.documents.len() >= 1);
    }

    #[tokio::test]
    #[ignore] // Requires Docker
    async fn test_knowledge_graph_wallet_history() {
        let container = start_neo4j_container().await;
        let config = create_test_config(&container).await;

        // Wait for Neo4j to be ready
        tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;

        let mut graph = GraphMemory::new(config).await.unwrap();

        let doc = RawTextDocument::new(
            "Wallet 0x742d35Cc6634C0532925a3b844Bc9e7595f0eA4B performed 3 transactions today",
        );

        graph.add_documents(vec![doc]).await.unwrap();

        let history = graph
            .search("0x742d35Cc6634C0532925a3b844Bc9e7595f0eA4B", None)
            .await
            .unwrap();

        assert!(!history.documents.is_empty());
    }

    #[tokio::test]
    #[ignore] // Requires Docker
    async fn test_knowledge_graph_complex_query() {
        let container = start_neo4j_container().await;
        let config = create_test_config(&container).await;

        // Wait for Neo4j to be ready
        tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;

        let mut graph = GraphMemory::new(config).await.unwrap();

        // Add complex transaction data
        let docs = vec![
            RawTextDocument::new(
                "Wallet 0xAAA holds 1000 USDC and 5 ETH, actively trading on Uniswap",
            ),
            RawTextDocument::new("Wallet 0xBBB swapped 100 USDC for DAI on Curve"),
            RawTextDocument::new("Wallet 0xAAA sent 500 USDC to wallet 0xBBB"),
        ];

        graph.add_documents(docs).await.unwrap();

        let stats = graph.get_stats().await.unwrap();
        assert!(stats.document_count >= 3);
        assert!(stats.entity_count > 0);

        let related = graph.search("USDC", Some(5)).await.unwrap();
        assert!(!related.documents.is_empty());
    }
}
