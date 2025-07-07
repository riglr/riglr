//! Neo4j client for graph database operations.

use crate::error::{GraphMemoryError, Result};
use reqwest::{Client, Response};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use tracing::{debug, info, warn};

/// Neo4j database client using HTTP REST API.
///
/// This client provides production-grade connectivity to Neo4j databases
/// with proper error handling, authentication, and query optimization.
#[derive(Debug, Clone)]
pub struct Neo4jClient {
    /// HTTP client for API requests
    client: Client,
    /// Base URL for Neo4j HTTP API (e.g., http://localhost:7474)
    base_url: String,
    /// Database name (default: "neo4j")
    database: String,
    /// Authentication credentials
    auth: Option<(String, String)>,
}

/// Neo4j query request structure
#[derive(Debug, Serialize)]
struct QueryRequest {
    statement: String,
    parameters: Option<HashMap<String, Value>>,
}

/// Neo4j query response structure
#[derive(Debug, Deserialize)]
struct QueryResponse {
    results: Vec<QueryResult>,
    errors: Vec<QueryError>,
}

/// Individual query result
#[derive(Debug, Deserialize)]
struct QueryResult {
    columns: Vec<String>,
    data: Vec<QueryRow>,
}

/// Query result row
#[derive(Debug, Deserialize)]
struct QueryRow {
    row: Vec<Value>,
    meta: Option<Value>,
}

/// Query error structure
#[derive(Debug, Deserialize)]
struct QueryError {
    code: String,
    message: String,
}

impl Neo4jClient {
    /// Create a new Neo4j client with HTTP endpoint.
    ///
    /// # Arguments
    ///
    /// * `base_url` - Neo4j HTTP endpoint (e.g., "http://localhost:7474")
    /// * `username` - Database username (optional)
    /// * `password` - Database password (optional)
    /// * `database` - Database name (optional, defaults to "neo4j")
    pub async fn new(
        base_url: impl Into<String>,
        username: Option<String>,
        password: Option<String>,
        database: Option<String>,
    ) -> Result<Self> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| {
                GraphMemoryError::Database(format!("Failed to create HTTP client: {}", e))
            })?;

        let base_url = base_url.into();
        let auth = match (username, password) {
            (Some(u), Some(p)) => Some((u, p)),
            _ => None,
        };

        let instance = Self {
            client,
            base_url,
            database: database.unwrap_or_else(|| "neo4j".to_string()),
            auth,
        };

        // Test connectivity
        instance.test_connection().await?;

        info!(
            "Neo4j client connected successfully to {}",
            instance.base_url
        );
        Ok(instance)
    }

    /// Test database connectivity
    async fn test_connection(&self) -> Result<()> {
        debug!("Testing Neo4j connection to {}", self.base_url);

        let query = "RETURN 1 as test";
        let result = self.execute_query(query, None).await?;

        if result["results"].as_array().is_some() {
            debug!("Neo4j connection test successful");
            Ok(())
        } else {
            Err(GraphMemoryError::Database(
                "Connection test failed".to_string(),
            ))
        }
    }

    /// Execute a Cypher query with optional parameters.
    ///
    /// # Arguments
    ///
    /// * `query` - Cypher query string
    /// * `parameters` - Optional query parameters
    ///
    /// # Returns
    ///
    /// Raw JSON response from Neo4j
    pub async fn execute_query(
        &self,
        query: &str,
        parameters: Option<HashMap<String, Value>>,
    ) -> Result<Value> {
        debug!("Executing Cypher query: {}", query);

        let url = format!("{}/db/{}/tx/commit", self.base_url, self.database);

        let request = QueryRequest {
            statement: query.to_string(),
            parameters,
        };

        let statements = vec![request];
        let body = json!({ "statements": statements });

        let mut req_builder = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .json(&body);

        // Add authentication if configured
        if let Some((username, password)) = &self.auth {
            req_builder = req_builder.basic_auth(username, Some(password));
        }

        let response = req_builder
            .send()
            .await
            .map_err(|e| GraphMemoryError::Database(format!("HTTP request failed: {}", e)))?;

        self.handle_response(response).await
    }

    /// Handle HTTP response and extract query results
    async fn handle_response(&self, response: Response) -> Result<Value> {
        let status = response.status();
        let response_text = response
            .text()
            .await
            .map_err(|e| GraphMemoryError::Database(format!("Failed to read response: {}", e)))?;

        if !status.is_success() {
            warn!(
                "Neo4j query failed with status {}: {}",
                status, response_text
            );
            return Err(GraphMemoryError::Query(format!(
                "Query failed with status {}: {}",
                status, response_text
            )));
        }

        let json_response: Value =
            serde_json::from_str(&response_text).map_err(GraphMemoryError::Serialization)?;

        // Check for Neo4j errors
        if let Some(errors) = json_response["errors"].as_array() {
            if !errors.is_empty() {
                let error_messages: Vec<String> = errors
                    .iter()
                    .filter_map(|e| e["message"].as_str())
                    .map(|s| s.to_string())
                    .collect();

                return Err(GraphMemoryError::Query(format!(
                    "Neo4j errors: {}",
                    error_messages.join(", ")
                )));
            }
        }

        debug!("Query executed successfully");
        Ok(json_response)
    }

    /// Execute a simple read query and return the first column of results
    pub async fn simple_query(&self, query: &str) -> Result<Vec<Value>> {
        let response = self.execute_query(query, None).await?;

        let mut results = Vec::new();

        if let Some(query_results) = response["results"].as_array() {
            for result in query_results {
                if let Some(rows) = result["data"].as_array() {
                    for row_data in rows {
                        if let Some(row) = row_data["row"].as_array() {
                            if let Some(first_value) = row.first() {
                                results.push(first_value.clone());
                            }
                        }
                    }
                }
            }
        }

        Ok(results)
    }

    /// Create database indexes for optimal performance
    pub async fn create_indexes(&self) -> Result<()> {
        info!("Creating Neo4j indexes for optimal performance");

        let indexes = vec![
            // Vector similarity index for embeddings
            "CREATE VECTOR INDEX IF NOT EXISTS embedding_index FOR (n:Document) ON (n.embedding) OPTIONS {indexConfig: {`vector.dimensions`: 1536, `vector.similarity_function`: 'cosine'}}",

            // Standard indexes for common lookups
            "CREATE INDEX IF NOT EXISTS wallet_address_index FOR (n:Wallet) ON (n.address)",
            "CREATE INDEX IF NOT EXISTS token_address_index FOR (n:Token) ON (n.address)",
            "CREATE INDEX IF NOT EXISTS token_symbol_index FOR (n:Token) ON (n.symbol)",
            "CREATE INDEX IF NOT EXISTS protocol_name_index FOR (n:Protocol) ON (n.name)",
            "CREATE INDEX IF NOT EXISTS transaction_hash_index FOR (n:Transaction) ON (n.hash)",
            "CREATE INDEX IF NOT EXISTS block_number_index FOR (n:Block) ON (n.number)",

            // Composite indexes for common query patterns
            "CREATE INDEX IF NOT EXISTS wallet_token_index FOR (n:Wallet) ON (n.address, n.chain)",
            "CREATE INDEX IF NOT EXISTS transaction_block_index FOR (n:Transaction) ON (n.block_number, n.chain)",
        ];

        for index_query in indexes {
            match self.execute_query(index_query, None).await {
                Ok(_) => debug!("Created index successfully: {}", index_query),
                Err(e) => {
                    warn!("Failed to create index '{}': {}", index_query, e);
                    // Continue with other indexes even if one fails
                }
            }
        }

        info!("Index creation completed");
        Ok(())
    }

    /// Get database statistics
    pub async fn get_stats(&self) -> Result<HashMap<String, Value>> {
        debug!("Retrieving Neo4j database statistics");

        let queries = vec![
            ("node_count", "MATCH (n) RETURN count(n) as count"),
            (
                "relationship_count",
                "MATCH ()-[r]->() RETURN count(r) as count",
            ),
            ("wallet_count", "MATCH (n:Wallet) RETURN count(n) as count"),
            ("token_count", "MATCH (n:Token) RETURN count(n) as count"),
            (
                "transaction_count",
                "MATCH (n:Transaction) RETURN count(n) as count",
            ),
            (
                "protocol_count",
                "MATCH (n:Protocol) RETURN count(n) as count",
            ),
        ];

        let mut stats = HashMap::new();

        for (stat_name, query) in queries {
            match self.simple_query(query).await {
                Ok(results) => {
                    if let Some(value) = results.first() {
                        stats.insert(stat_name.to_string(), value.clone());
                    }
                }
                Err(e) => {
                    warn!("Failed to get stat '{}': {}", stat_name, e);
                    stats.insert(stat_name.to_string(), Value::Null);
                }
            }
        }

        info!("Retrieved database statistics: {} entries", stats.len());
        Ok(stats)
    }
}
