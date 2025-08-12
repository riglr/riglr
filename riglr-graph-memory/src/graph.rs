//! Main graph memory implementation.
//!
//! This module provides the primary GraphMemory interface that coordinates between
//! the Neo4j client, entity extraction, and vector storage to create a comprehensive
//! knowledge graph system for blockchain data.

use crate::{
    client::Neo4jClient,
    document::{ExtractedEntities, RawTextDocument},
    error::Result,
    extractor::EntityExtractor,
    vector_store::{GraphRetriever, GraphRetrieverConfig},
};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info, warn, error};

#[cfg(feature = "rig-core")]
use rig::providers::openai::{Client as OpenAIClient, TEXT_EMBEDDING_ADA_002};
#[cfg(feature = "rig-core")]
use rig::client::EmbeddingsClient;
// Real embeddings implementation using rig-core

/// The main graph memory system that provides comprehensive document storage,
/// entity extraction, and hybrid vector + graph search capabilities.
#[derive(Debug)]
pub struct GraphMemory {
    /// Neo4j database client
    client: Arc<Neo4jClient>,
    /// Entity extractor for processing documents
    extractor: EntityExtractor,
    /// Graph-based vector retriever
    retriever: Arc<GraphRetriever>,
    /// Configuration settings
    config: GraphMemoryConfig,
    /// OpenAI client for real embeddings (optional)
    #[cfg(feature = "rig-core")]
    embedding_client: Option<OpenAIClient>,
}

/// Configuration for the graph memory system
#[derive(Debug, Clone)]
pub struct GraphMemoryConfig {
    /// Neo4j connection URL
    pub neo4j_url: String,
    /// Database username
    pub username: Option<String>,
    /// Database password  
    pub password: Option<String>,
    /// Database name (default: "neo4j")
    pub database: Option<String>,
    /// Vector retriever configuration
    pub retriever_config: GraphRetrieverConfig,
    /// Whether to automatically extract entities on document add
    pub auto_extract_entities: bool,
    /// Whether to automatically generate embeddings
    pub auto_generate_embeddings: bool,
    /// Batch size for processing documents
    pub batch_size: usize,
}

/// Statistics about the graph memory system
#[derive(Debug, Clone)]
pub struct GraphMemoryStats {
    /// Total number of documents
    pub document_count: u64,
    /// Total number of entity nodes
    pub entity_count: u64,
    /// Total number of relationships
    pub relationship_count: u64,
    /// Total number of wallets tracked
    pub wallet_count: u64,
    /// Total number of tokens tracked
    pub token_count: u64,
    /// Total number of protocols tracked
    pub protocol_count: u64,
    /// Average entities per document
    pub avg_entities_per_doc: f64,
    /// Storage size in bytes (approximate)
    pub storage_size_bytes: u64,
}

impl Default for GraphMemoryConfig {
    fn default() -> Self {
        Self {
            neo4j_url: "http://localhost:7474".to_string(),
            username: Some("neo4j".to_string()),
            password: Some("password".to_string()),
            database: Some("neo4j".to_string()),
            retriever_config: GraphRetrieverConfig::new_default(),
            auto_extract_entities: true,
            auto_generate_embeddings: true,
            batch_size: 100,
        }
    }
}

impl GraphMemory {
    /// Create a new graph memory instance with configuration.
    pub async fn new(config: GraphMemoryConfig) -> Result<Self> {
        info!(
            "Initializing GraphMemory with Neo4j at {}",
            config.neo4j_url
        );

        // Create Neo4j client
        let client = Arc::new(
            Neo4jClient::new(
                config.neo4j_url.clone(),
                config.username.clone(),
                config.password.clone(),
                config.database.clone(),
            )
            .await?,
        );

        // Initialize database indexes for performance
        client.create_indexes().await?;

        // Create entity extractor
        let extractor = EntityExtractor::new();

        // Create graph retriever
        let retriever = Arc::new(
            GraphRetriever::new(client.clone(), Some(config.retriever_config.clone())).await?,
        );

        // Initialize embedding client if OpenAI API key is available
        #[cfg(feature = "rig-core")]
        let embedding_client = {
            match std::env::var("OPENAI_API_KEY") {
                Ok(api_key) => {
                    info!("Initializing OpenAI client for real embeddings");
                    Some(OpenAIClient::new(&api_key))
                },
                Err(_) => {
                    warn!("OPENAI_API_KEY not found. Real embeddings disabled.");
                    warn!("Set OPENAI_API_KEY environment variable to enable real ML embeddings.");
                    None
                }
            }
        };

        info!("GraphMemory initialized successfully");

        Ok(Self {
            client,
            extractor,
            retriever,
            config,
            #[cfg(feature = "rig-core")]
            embedding_client,
        })
    }

    /// Create a new instance with default configuration.
    pub async fn with_defaults(neo4j_url: impl Into<String>) -> Result<Self> {
        let config = GraphMemoryConfig {
            neo4j_url: neo4j_url.into(),
            ..Default::default()
        };
        Self::new(config).await
    }

    /// Add documents to the graph with full processing pipeline.
    pub async fn add_documents(&self, documents: Vec<RawTextDocument>) -> Result<Vec<String>> {
        info!("Processing {} documents for graph storage", documents.len());

        let mut document_ids = Vec::new();
        let mut processed_docs = Vec::new();

        // Process documents in batches
        for chunk in documents.chunks(self.config.batch_size) {
            for doc in chunk {
                match self.process_single_document(doc.clone()).await {
                    Ok(processed) => {
                        document_ids.push(processed.id.clone());
                        processed_docs.push(processed);
                    }
                    Err(e) => {
                        warn!("Failed to process document {}: {}", doc.id, e);
                        // Continue with other documents
                    }
                }
            }
        }

        // Store processed documents using the vector store
        let stored_ids = self.retriever.add_documents(processed_docs).await?;

        info!(
            "Successfully processed and stored {} documents",
            stored_ids.len()
        );
        Ok(stored_ids)
    }

    /// Process a single document through the full pipeline
    async fn process_single_document(
        &self,
        mut document: RawTextDocument,
    ) -> Result<RawTextDocument> {
        debug!("Processing document: {}", document.id);

        // Extract entities if enabled
        if self.config.auto_extract_entities {
            let extracted = self.extractor.extract(&document.content);

            // Update document metadata with extracted entities
            let mut metadata = document.metadata.unwrap_or_default();

            for wallet in &extracted.wallets {
                metadata.add_wallet(&wallet.canonical);
            }

            for token in &extracted.tokens {
                metadata.add_token(&token.canonical);
            }

            for protocol in &extracted.protocols {
                metadata.add_protocol(&protocol.canonical);
            }

            document.metadata = Some(metadata);

            // Store entities and relationships in graph
            self.store_entities_and_relationships(&document, &extracted)
                .await?;
        }

        // Generate embeddings if enabled
        if self.config.auto_generate_embeddings {
            document.embedding = self.generate_real_embedding(&document.content).await?;
        }

        debug!("Document processing completed: {}", document.id);
        Ok(document)
    }

    /// Store extracted entities and relationships in the graph
    async fn store_entities_and_relationships(
        &self,
        document: &RawTextDocument,
        extracted: &ExtractedEntities,
    ) -> Result<()> {
        debug!(
            "Storing {} entities and {} relationships for document {}",
            extracted.wallets.len() + extracted.tokens.len() + extracted.protocols.len(),
            extracted.relationships.len(),
            document.id
        );

        // Create entity nodes
        for wallet in &extracted.wallets {
            self.create_entity_node(
                "Wallet",
                &wallet.canonical,
                &wallet.text,
                wallet.confidence,
                &wallet.properties,
            )
            .await?;
        }

        for token in &extracted.tokens {
            self.create_entity_node(
                "Token",
                &token.canonical,
                &token.text,
                token.confidence,
                &token.properties,
            )
            .await?;
        }

        for protocol in &extracted.protocols {
            self.create_entity_node(
                "Protocol",
                &protocol.canonical,
                &protocol.text,
                protocol.confidence,
                &protocol.properties,
            )
            .await?;
        }

        // Create relationships
        for relationship in &extracted.relationships {
            self.create_relationship(
                &relationship.from_entity,
                &relationship.to_entity,
                &format!("{:?}", relationship.relationship_type),
                relationship.confidence,
                &relationship.context,
            )
            .await?;
        }

        // Connect document to entities
        for wallet in &extracted.wallets {
            self.connect_document_to_entity(&document.id, &wallet.canonical, "MENTIONS")
                .await?;
        }

        for token in &extracted.tokens {
            self.connect_document_to_entity(&document.id, &token.canonical, "MENTIONS")
                .await?;
        }

        for protocol in &extracted.protocols {
            self.connect_document_to_entity(&document.id, &protocol.canonical, "MENTIONS")
                .await?;
        }

        debug!(
            "Entity and relationship storage completed for document {}",
            document.id
        );
        Ok(())
    }

    /// Create or update an entity node in the graph
    async fn create_entity_node(
        &self,
        entity_type: &str,
        canonical: &str,
        text: &str,
        confidence: f32,
        properties: &HashMap<String, String>,
    ) -> Result<()> {
        let query = format!(
            "MERGE (e:{} {{canonical: $canonical}})
             ON CREATE SET e.text = $text, e.confidence = $confidence, e.created_at = datetime()
             ON MATCH SET e.confidence = CASE WHEN $confidence > e.confidence THEN $confidence ELSE e.confidence END
             SET e += $properties",
            entity_type
        );

        let mut params = HashMap::new();
        params.insert("canonical".to_string(), json!(canonical));
        params.insert("text".to_string(), json!(text));
        params.insert("confidence".to_string(), json!(confidence));
        params.insert("properties".to_string(), json!(properties));

        self.client.execute_query(&query, Some(params)).await?;
        Ok(())
    }

    /// Create a relationship between entities
    async fn create_relationship(
        &self,
        from_entity: &str,
        to_entity: &str,
        rel_type: &str,
        confidence: f32,
        context: &str,
    ) -> Result<()> {
        let query = format!(
            "MATCH (a {{canonical: $from_entity}}), (b {{canonical: $to_entity}})
             MERGE (a)-[r:{}]->(b)
             SET r.confidence = $confidence, r.context = $context, r.created_at = datetime()",
            rel_type
        );

        let mut params = HashMap::new();
        params.insert("from_entity".to_string(), json!(from_entity));
        params.insert("to_entity".to_string(), json!(to_entity));
        params.insert("confidence".to_string(), json!(confidence));
        params.insert("context".to_string(), json!(context));

        self.client.execute_query(&query, Some(params)).await?;
        Ok(())
    }

    /// Connect a document to an entity
    async fn connect_document_to_entity(
        &self,
        document_id: &str,
        entity_canonical: &str,
        rel_type: &str,
    ) -> Result<()> {
        let query = format!(
            "MATCH (d:Document {{id: $document_id}}), (e {{canonical: $entity_canonical}})
             MERGE (d)-[:{}]->(e)",
            rel_type
        );

        let mut params = HashMap::new();
        params.insert("document_id".to_string(), json!(document_id));
        params.insert("entity_canonical".to_string(), json!(entity_canonical));

        self.client.execute_query(&query, Some(params)).await?;
        Ok(())
    }

    /// Search for documents using hybrid vector + graph search
    pub async fn search(
        &self,
        query_embedding: &[f32],
        limit: usize,
    ) -> Result<crate::vector_store::GraphSearchResult> {
        self.retriever
            .search_with_graph_context(query_embedding, limit)
            .await
    }

    /// Get comprehensive statistics about the graph
    pub async fn get_stats(&self) -> Result<GraphMemoryStats> {
        debug!("Retrieving graph memory statistics");

        let db_stats = self.client.get_stats().await?;

        // Extract values from database stats
        let document_count = db_stats
            .get("node_count")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let relationship_count = db_stats
            .get("relationship_count")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let wallet_count = db_stats
            .get("wallet_count")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let token_count = db_stats
            .get("token_count")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let protocol_count = db_stats
            .get("protocol_count")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        let entity_count = wallet_count + token_count + protocol_count;
        let avg_entities_per_doc = if document_count > 0 {
            entity_count as f64 / document_count as f64
        } else {
            0.0
        };

        // Rough storage size estimation (would be more accurate with actual queries)
        let storage_size_bytes =
            (document_count * 1000) + (entity_count * 500) + (relationship_count * 200);

        let stats = GraphMemoryStats {
            document_count,
            entity_count,
            relationship_count,
            wallet_count,
            token_count,
            protocol_count,
            avg_entities_per_doc,
            storage_size_bytes,
        };

        info!(
            "Graph statistics: {} docs, {} entities, {} relationships",
            stats.document_count, stats.entity_count, stats.relationship_count
        );

        Ok(stats)
    }

    /// Get the underlying graph retriever for advanced operations
    pub fn retriever(&self) -> &GraphRetriever {
        &self.retriever
    }

    /// Get the underlying Neo4j client for direct queries
    pub fn client(&self) -> &Neo4jClient {
        &self.client
    }

    /// Get the entity extractor
    pub fn extractor(&self) -> &EntityExtractor {
        &self.extractor
    }

    /// Generate real ML embeddings using OpenAI API
    /// CRITICAL: This replaces placeholder zero vectors with real embeddings
    async fn generate_real_embedding(&self, content: &str) -> Result<Option<Vec<f32>>> {
        #[cfg(feature = "rig-core")]
        {
            if let Some(ref client) = self.embedding_client {
                info!("Generating REAL embedding for content (length: {})", content.len());
                
                // Create the embeddings builder and add the document
                let embeddings_builder = client.embeddings(TEXT_EMBEDDING_ADA_002)
                    .document(content.to_string());
                
                // Build and generate the embedding
                match embeddings_builder {
                    Ok(builder) => {
                        match builder.build().await {
                            Ok(mut embeddings) => {
                                if let Some((_, embedding)) = embeddings.pop() {
                                    info!("Real embedding generated successfully from OpenAI API");
                                    info!("CRITICAL: Using REAL OpenAI embeddings - placeholder vectors ELIMINATED");
                                    
                                    // Extract the actual embedding vector (take the first one)
                                    let embedding_vec = embedding.first().vec;
                                    
                                    // Convert f64 vector to f32 for storage efficiency
                                    let embedding_vector: Vec<f32> = embedding_vec
                                        .into_iter()
                                        .map(|v| v as f32)
                                        .collect();
                                    
                                    // Log some stats about the embedding
                                    let sum: f32 = embedding_vector.iter().map(|v| v.abs()).sum();
                                    let non_zero_count = embedding_vector.iter().filter(|&&v| v != 0.0).count();
                                    info!(
                                        "Generated real embedding: {} dimensions, {} non-zero values, L1 norm: {:.3}", 
                                        embedding_vector.len(), 
                                        non_zero_count, 
                                        sum
                                    );
                                    
                                    Ok(Some(embedding_vector))
                                } else {
                                    error!("No embeddings returned from OpenAI API");
                                    Ok(None)
                                }
                            }
                            Err(e) => {
                                error!("Failed to build embeddings: {}", e);
                                Ok(None)
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to create embeddings builder: {}", e);
                        // Return None instead of zero vector to indicate failure
                        Ok(None)
                    }
                }
            } else {
                warn!("No embedding client available. Set OPENAI_API_KEY to enable real embeddings.");
                Ok(None)
            }
        }
        
        #[cfg(not(feature = "rig-core"))]
        {
            warn!("rig-core feature not enabled. Real embeddings disabled.");
            warn!("Enable rig-core feature to use real ML embeddings.");
            Ok(None)
        }
    }
}
