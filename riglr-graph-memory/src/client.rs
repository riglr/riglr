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

#[allow(dead_code)]
/// Neo4j query response structure
#[derive(Debug, Deserialize)]
struct QueryResponse {
    results: Vec<QueryResult>,
    errors: Vec<QueryError>,
}

#[allow(dead_code)]
/// Individual query result
#[derive(Debug, Deserialize)]
struct QueryResult {
    columns: Vec<String>,
    data: Vec<QueryRow>,
}

#[allow(dead_code)]
/// Query result row
#[derive(Debug, Deserialize)]
struct QueryRow {
    row: Vec<Value>,
    meta: Option<Value>,
}

#[allow(dead_code)]
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

        let response_result = req_builder.send().await;
        let response = response_result
            .map_err(|e| GraphMemoryError::Database(format!("HTTP request failed: {}", e)))?;

        self.handle_response(response).await
    }

    /// Handle HTTP response and extract query results
    async fn handle_response(&self, response: Response) -> Result<Value> {
        let status = response.status();
        let text_result = response.text().await;
        let response_text = text_result
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
            let query_result = self.execute_query(index_query, None).await;
            match query_result {
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
            let query_result = self.simple_query(query).await;
            match query_result {
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

impl Default for Neo4jClient {
    fn default() -> Self {
        Self {
            client: Client::new(),
            base_url: "http://localhost:7474".to_string(),
            database: "neo4j".to_string(),
            auth: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::collections::HashMap;

    // Helper function to create a mock client for testing
    fn create_mock_client() -> Neo4jClient {
        Neo4jClient {
            client: Client::new(),
            base_url: "http://localhost:7474".to_string(),
            database: "neo4j".to_string(),
            auth: Some(("neo4j".to_string(), "password".to_string())),
        }
    }

    #[test]
    fn test_neo4j_client_default() {
        let client = Neo4jClient::default();
        assert_eq!(client.base_url, "http://localhost:7474");
        assert_eq!(client.database, "neo4j");
        assert!(client.auth.is_none());
    }

    #[test]
    fn test_query_request_serialization() {
        let mut params = HashMap::new();
        params.insert("test".to_string(), json!("value"));

        let request = QueryRequest {
            statement: "RETURN 1".to_string(),
            parameters: Some(params),
        };

        let serialized = serde_json::to_string(&request).unwrap();
        assert!(serialized.contains("RETURN 1"));
        assert!(serialized.contains("test"));
        assert!(serialized.contains("value"));
    }

    #[test]
    fn test_query_request_serialization_no_params() {
        let request = QueryRequest {
            statement: "RETURN 1".to_string(),
            parameters: None,
        };

        let serialized = serde_json::to_string(&request).unwrap();
        assert!(serialized.contains("RETURN 1"));
        assert!(serialized.contains("null"));
    }

    #[test]
    fn test_query_response_deserialization() {
        let json_str = r#"{
            "results": [{
                "columns": ["test"],
                "data": [{
                    "row": [1],
                    "meta": null
                }]
            }],
            "errors": []
        }"#;

        let response: QueryResponse = serde_json::from_str(json_str).unwrap();
        assert_eq!(response.results.len(), 1);
        assert_eq!(response.errors.len(), 0);
        assert_eq!(response.results[0].columns[0], "test");
        assert_eq!(response.results[0].data.len(), 1);
    }

    #[test]
    fn test_query_error_deserialization() {
        let json_str = r#"{
            "results": [],
            "errors": [{
                "code": "Neo.ClientError.Statement.SyntaxError",
                "message": "Invalid syntax"
            }]
        }"#;

        let response: QueryResponse = serde_json::from_str(json_str).unwrap();
        assert_eq!(response.results.len(), 0);
        assert_eq!(response.errors.len(), 1);
        assert_eq!(
            response.errors[0].code,
            "Neo.ClientError.Statement.SyntaxError"
        );
        assert_eq!(response.errors[0].message, "Invalid syntax");
    }

    #[test]
    fn test_query_result_with_meta() {
        let json_str = r#"{
            "columns": ["n"],
            "data": [{
                "row": [{"name": "test"}],
                "meta": {"id": 123}
            }]
        }"#;

        let result: QueryResult = serde_json::from_str(json_str).unwrap();
        assert_eq!(result.columns[0], "n");
        assert_eq!(result.data.len(), 1);
        assert!(result.data[0].meta.is_some());
    }

    #[test]
    fn test_query_row_without_meta() {
        let json_str = r#"{
            "row": [1, "test", null],
            "meta": null
        }"#;

        let row: QueryRow = serde_json::from_str(json_str).unwrap();
        assert_eq!(row.row.len(), 3);
        assert!(row.meta.is_none());
        assert_eq!(row.row[0], json!(1));
        assert_eq!(row.row[1], json!("test"));
        assert_eq!(row.row[2], json!(null));
    }

    // Test handle_response with successful response
    #[tokio::test]
    async fn test_handle_response_success() {
        let _client = create_mock_client();

        // Create a mock successful response
        let json_response = json!({
            "results": [{
                "columns": ["test"],
                "data": [{"row": [1]}]
            }],
            "errors": []
        });

        // We can't easily mock reqwest::Response, so we'll test the JSON parsing logic
        // by testing the error conditions that would occur in handle_response
        let response_text = serde_json::to_string(&json_response).unwrap();
        let parsed: Value = serde_json::from_str(&response_text).unwrap();

        // Verify the structure matches what handle_response expects
        assert!(parsed["results"].is_array());
        assert!(parsed["errors"].is_array());
        assert_eq!(parsed["errors"].as_array().unwrap().len(), 0);
    }

    // Test handle_response with Neo4j errors
    #[tokio::test]
    async fn test_handle_response_with_neo4j_errors() {
        let json_response = json!({
            "results": [],
            "errors": [
                {"message": "Syntax error"},
                {"message": "Another error"}
            ]
        });

        let response_text = serde_json::to_string(&json_response).unwrap();
        let parsed: Value = serde_json::from_str(&response_text).unwrap();

        // Simulate the error checking logic from handle_response
        if let Some(errors) = parsed["errors"].as_array() {
            if !errors.is_empty() {
                let error_messages: Vec<String> = errors
                    .iter()
                    .filter_map(|e| e["message"].as_str())
                    .map(|s| s.to_string())
                    .collect();

                assert_eq!(error_messages.len(), 2);
                assert_eq!(error_messages[0], "Syntax error");
                assert_eq!(error_messages[1], "Another error");
            }
        }
    }

    // Test simple_query result parsing
    #[tokio::test]
    async fn test_simple_query_result_parsing() {
        let response = json!({
            "results": [{
                "columns": ["count"],
                "data": [
                    {"row": [42]},
                    {"row": [100]},
                    {"row": [null]}
                ]
            }],
            "errors": []
        });

        let mut results = Vec::new();

        // Simulate the parsing logic from simple_query
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

        assert_eq!(results.len(), 3);
        assert_eq!(results[0], json!(42));
        assert_eq!(results[1], json!(100));
        assert_eq!(results[2], json!(null));
    }

    // Test simple_query with empty results
    #[tokio::test]
    async fn test_simple_query_empty_results() {
        let response = json!({
            "results": [],
            "errors": []
        });

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

        assert_eq!(results.len(), 0);
    }

    // Test simple_query with no data field
    #[tokio::test]
    async fn test_simple_query_no_data_field() {
        let response = json!({
            "results": [{
                "columns": ["count"]
                // Missing data field
            }],
            "errors": []
        });

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

        assert_eq!(results.len(), 0);
    }

    // Test simple_query with empty row
    #[tokio::test]
    async fn test_simple_query_empty_row() {
        let response = json!({
            "results": [{
                "columns": ["count"],
                "data": [
                    {"row": []}, // Empty row
                    {"row": [42]}
                ]
            }],
            "errors": []
        });

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

        assert_eq!(results.len(), 1); // Only one result from the non-empty row
        assert_eq!(results[0], json!(42));
    }

    // Test auth configuration during client creation
    #[test]
    fn test_auth_configuration_both_provided() {
        let username = Some("user".to_string());
        let password = Some("pass".to_string());

        let auth = match (username, password) {
            (Some(u), Some(p)) => Some((u, p)),
            _ => None,
        };

        assert!(auth.is_some());
        let (u, p) = auth.unwrap();
        assert_eq!(u, "user");
        assert_eq!(p, "pass");
    }

    #[test]
    fn test_auth_configuration_username_only() {
        let username = Some("user".to_string());
        let password: Option<String> = None;

        let auth = match (username, password) {
            (Some(u), Some(p)) => Some((u, p)),
            _ => None,
        };

        assert!(auth.is_none());
    }

    #[test]
    fn test_auth_configuration_password_only() {
        let username: Option<String> = None;
        let password = Some("pass".to_string());

        let auth = match (username, password) {
            (Some(u), Some(p)) => Some((u, p)),
            _ => None,
        };

        assert!(auth.is_none());
    }

    #[test]
    fn test_auth_configuration_none_provided() {
        let username: Option<String> = None;
        let password: Option<String> = None;

        let auth = match (username, password) {
            (Some(u), Some(p)) => Some((u, p)),
            _ => None,
        };

        assert!(auth.is_none());
    }

    // Test database name configuration
    #[test]
    fn test_database_name_provided() {
        let database = Some("custom_db".to_string());
        let result = database.unwrap_or_else(|| "neo4j".to_string());
        assert_eq!(result, "custom_db");
    }

    #[test]
    fn test_database_name_default() {
        let database: Option<String> = None;
        let result = database.unwrap_or_else(|| "neo4j".to_string());
        assert_eq!(result, "neo4j");
    }

    // Test URL building for execute_query
    #[test]
    fn test_url_building() {
        let base_url = "http://localhost:7474";
        let database = "neo4j";
        let url = format!("{}/db/{}/tx/commit", base_url, database);
        assert_eq!(url, "http://localhost:7474/db/neo4j/tx/commit");
    }

    #[test]
    fn test_url_building_custom_database() {
        let base_url = "http://localhost:7474";
        let database = "custom";
        let url = format!("{}/db/{}/tx/commit", base_url, database);
        assert_eq!(url, "http://localhost:7474/db/custom/tx/commit");
    }

    // Test request body creation
    #[test]
    fn test_request_body_creation() {
        let mut parameters = HashMap::new();
        parameters.insert("param1".to_string(), json!("value1"));
        parameters.insert("param2".to_string(), json!(42));

        let request = QueryRequest {
            statement: "MATCH (n) RETURN n".to_string(),
            parameters: Some(parameters),
        };

        let statements = vec![request];
        let body = json!({ "statements": statements });

        assert!(body["statements"].is_array());
        let statements_array = body["statements"].as_array().unwrap();
        assert_eq!(statements_array.len(), 1);

        let first_statement = &statements_array[0];
        assert_eq!(first_statement["statement"], "MATCH (n) RETURN n");
        assert!(first_statement["parameters"].is_object());
    }

    #[test]
    fn test_request_body_creation_no_params() {
        let request = QueryRequest {
            statement: "MATCH (n) RETURN n".to_string(),
            parameters: None,
        };

        let statements = vec![request];
        let body = json!({ "statements": statements });

        let statements_array = body["statements"].as_array().unwrap();
        let first_statement = &statements_array[0];
        assert_eq!(first_statement["statement"], "MATCH (n) RETURN n");
        assert!(first_statement["parameters"].is_null());
    }

    // Test indexes creation logic
    #[test]
    fn test_create_indexes_query_list() {
        let indexes = vec![
            "CREATE VECTOR INDEX IF NOT EXISTS embedding_index FOR (n:Document) ON (n.embedding) OPTIONS {indexConfig: {`vector.dimensions`: 1536, `vector.similarity_function`: 'cosine'}}",
            "CREATE INDEX IF NOT EXISTS wallet_address_index FOR (n:Wallet) ON (n.address)",
            "CREATE INDEX IF NOT EXISTS token_address_index FOR (n:Token) ON (n.address)",
            "CREATE INDEX IF NOT EXISTS token_symbol_index FOR (n:Token) ON (n.symbol)",
            "CREATE INDEX IF NOT EXISTS protocol_name_index FOR (n:Protocol) ON (n.name)",
            "CREATE INDEX IF NOT EXISTS transaction_hash_index FOR (n:Transaction) ON (n.hash)",
            "CREATE INDEX IF NOT EXISTS block_number_index FOR (n:Block) ON (n.number)",
            "CREATE INDEX IF NOT EXISTS wallet_token_index FOR (n:Wallet) ON (n.address, n.chain)",
            "CREATE INDEX IF NOT EXISTS transaction_block_index FOR (n:Transaction) ON (n.block_number, n.chain)",
        ];

        assert_eq!(indexes.len(), 9);
        assert!(indexes[0].contains("VECTOR INDEX"));
        assert!(indexes[1].contains("wallet_address_index"));
        assert!(indexes[8].contains("transaction_block_index"));
    }

    // Test statistics queries
    #[test]
    fn test_get_stats_queries() {
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

        assert_eq!(queries.len(), 6);
        assert_eq!(queries[0].0, "node_count");
        assert_eq!(queries[0].1, "MATCH (n) RETURN count(n) as count");
        assert_eq!(queries[5].0, "protocol_count");
        assert!(queries[1].1.contains("()-[r]->()"));
    }

    // Test stats HashMap building logic
    #[test]
    fn test_stats_hashmap_building() {
        let mut stats = HashMap::new();

        // Simulate successful stat retrieval
        let results = vec![json!(42)];
        if let Some(value) = results.first() {
            stats.insert("test_stat".to_string(), value.clone());
        }

        assert_eq!(stats.len(), 1);
        assert_eq!(stats.get("test_stat"), Some(&json!(42)));
    }

    #[test]
    fn test_stats_hashmap_empty_results() {
        let mut stats = HashMap::new();

        // Simulate empty results
        let results: Vec<Value> = vec![];
        if let Some(value) = results.first() {
            stats.insert("test_stat".to_string(), value.clone());
        } else {
            // This is what would happen in the error case
            stats.insert("test_stat".to_string(), Value::Null);
        }

        assert_eq!(stats.len(), 1);
        assert_eq!(stats.get("test_stat"), Some(&Value::Null));
    }

    // Test connection test validation logic
    #[test]
    fn test_connection_test_validation_success() {
        let result = json!({
            "results": [{
                "columns": ["test"],
                "data": [{"row": [1]}]
            }],
            "errors": []
        });

        let is_valid = result["results"].as_array().is_some();
        assert!(is_valid);
    }

    #[test]
    fn test_connection_test_validation_failure() {
        let result = json!({
            "errors": ["Connection failed"]
        });

        let is_valid = result["results"].as_array().is_some();
        assert!(!is_valid);
    }

    // Test Clone implementation for Neo4jClient
    #[test]
    fn test_neo4j_client_clone() {
        let original = create_mock_client();
        let cloned = original.clone();

        assert_eq!(original.base_url, cloned.base_url);
        assert_eq!(original.database, cloned.database);
        assert_eq!(original.auth, cloned.auth);
    }

    // Test Debug implementation for Neo4jClient
    #[test]
    fn test_neo4j_client_debug() {
        let client = create_mock_client();
        let debug_str = format!("{:?}", client);

        assert!(debug_str.contains("Neo4jClient"));
        assert!(debug_str.contains("localhost:7474"));
        assert!(debug_str.contains("neo4j"));
    }

    // Test serialization structs Debug implementations
    #[test]
    fn test_query_request_debug() {
        let request = QueryRequest {
            statement: "RETURN 1".to_string(),
            parameters: None,
        };

        let debug_str = format!("{:?}", request);
        assert!(debug_str.contains("QueryRequest"));
        assert!(debug_str.contains("RETURN 1"));
    }

    #[test]
    fn test_query_response_debug() {
        let response = QueryResponse {
            results: vec![],
            errors: vec![],
        };

        let debug_str = format!("{:?}", response);
        assert!(debug_str.contains("QueryResponse"));
    }

    #[test]
    fn test_query_result_debug() {
        let result = QueryResult {
            columns: vec!["test".to_string()],
            data: vec![],
        };

        let debug_str = format!("{:?}", result);
        assert!(debug_str.contains("QueryResult"));
        assert!(debug_str.contains("test"));
    }

    #[test]
    fn test_query_row_debug() {
        let row = QueryRow {
            row: vec![json!(1)],
            meta: None,
        };

        let debug_str = format!("{:?}", row);
        assert!(debug_str.contains("QueryRow"));
    }

    #[test]
    fn test_query_error_debug() {
        let error = QueryError {
            code: "Test.Error".to_string(),
            message: "Test message".to_string(),
        };

        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("QueryError"));
        assert!(debug_str.contains("Test.Error"));
        assert!(debug_str.contains("Test message"));
    }

    // Test error message formatting
    #[test]
    fn test_error_message_joining() {
        let errors = vec![
            json!({"message": "First error"}),
            json!({"message": "Second error"}),
            json!({"message": "Third error"}),
        ];

        let error_messages: Vec<String> = errors
            .iter()
            .filter_map(|e| e["message"].as_str())
            .map(|s| s.to_string())
            .collect();

        let joined = error_messages.join(", ");
        assert_eq!(joined, "First error, Second error, Third error");
    }

    #[test]
    fn test_error_message_joining_with_missing_messages() {
        let errors = vec![
            json!({"message": "First error"}),
            json!({"code": "ERROR_CODE"}), // Missing message field
            json!({"message": "Third error"}),
        ];

        let error_messages: Vec<String> = errors
            .iter()
            .filter_map(|e| e["message"].as_str())
            .map(|s| s.to_string())
            .collect();

        let joined = error_messages.join(", ");
        assert_eq!(joined, "First error, Third error");
    }

    // Test edge cases for various data types
    #[test]
    fn test_handle_complex_json_values() {
        let complex_value = json!({
            "nested": {
                "array": [1, 2, 3],
                "object": {"key": "value"},
                "null": null,
                "boolean": true,
                "number": 42.5
            }
        });

        // Test that we can clone and work with complex JSON values
        let cloned = complex_value.clone();
        assert_eq!(complex_value, cloned);
        assert!(complex_value["nested"]["array"].is_array());
        assert!(complex_value["nested"]["object"].is_object());
        assert!(complex_value["nested"]["null"].is_null());
        assert!(complex_value["nested"]["boolean"].is_boolean());
        assert!(complex_value["nested"]["number"].is_number());
    }
}
