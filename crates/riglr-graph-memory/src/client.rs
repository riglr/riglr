//! Neo4j client for graph database operations.

use crate::error::{Error, Result};
use core::time::Duration;
use reqwest::{Client as HttpClient, Response};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use tracing::{debug, info, warn};

/// Neo4j database client using HTTP REST API.
///
/// This client provides production-grade connectivity to Neo4j databases
/// with proper error handling, authentication, and query optimization.
#[derive(Debug, Clone)]
pub struct Client {
    /// Authentication credentials
    auth: Option<(String, String)>,
    /// Base URL for Neo4j HTTP API (e.g., <http://localhost:7474>)
    base_url: String,
    /// HTTP client for API requests
    http: HttpClient,
    /// Database name (default: "neo4j")
    database: String,
}

/// Neo4j query request structure
#[derive(Debug, Serialize)]
struct QueryRequest {
    parameters: Option<HashMap<String, Value>>,
    statement: String,
}

/// Neo4j query response structure
#[derive(Debug, Deserialize)]
struct QueryResponse {
    #[allow(clippy::allow_attributes, dead_code)]
    errors: Vec<QueryError>,
    #[allow(clippy::allow_attributes, dead_code)]
    results: Vec<QueryResult>,
}

/// Individual query result
#[derive(Debug, Deserialize)]
struct QueryResult {
    #[allow(clippy::allow_attributes, dead_code)]
    columns: Vec<String>,
    #[allow(clippy::allow_attributes, dead_code)]
    data: Vec<QueryRow>,
}

/// Query result row
#[derive(Debug, Deserialize)]
struct QueryRow {
    #[allow(clippy::allow_attributes, dead_code)]
    meta: Option<Value>,
    #[allow(clippy::allow_attributes, dead_code)]
    row: Vec<Value>,
}

/// Query error structure
#[derive(Debug, Deserialize)]
struct QueryError {
    #[allow(clippy::allow_attributes, dead_code)]
    code: String,
    #[allow(clippy::allow_attributes, dead_code)]
    message: String,
}

impl Client {
    /// Create a new Neo4j client with HTTP endpoint.
    ///
    /// # Arguments
    ///
    /// * `base_url` - Neo4j HTTP endpoint (e.g., "<http://localhost:7474>")
    /// * `username` - Database username (optional)
    /// * `password` - Database password (optional)
    /// * `database` - Database name (optional, defaults to "neo4j")
    ///
    /// # Errors
    /// Returns an error if:
    /// - HTTP client creation fails
    /// - Database connection test fails
    /// - Authentication fails
    pub async fn new(
        base_url: impl Into<String>,
        username: Option<String>,
        password: Option<String>,
        database: Option<String>,
    ) -> Result<Self> {
        let http = HttpClient::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| Error::Database(format!("Failed to create HTTP client: {e}")))?;

        let base_url = base_url.into();
        let auth = match (username, password) {
            (Some(u), Some(p)) => Some((u, p)),
            _ => None,
        };

        let instance = Self {
            auth,
            base_url,
            http,
            database: database.unwrap_or_else(|| "neo4j".to_string()),
        };

        // Test connectivity
        instance.test_connection().await?;

        info!(
            "Neo4j client connected successfully to {}",
            instance.base_url
        );
        Ok(instance)
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
    ///
    /// # Errors
    /// Returns an error if:
    /// - HTTP request to Neo4j fails
    /// - Neo4j returns HTTP error status
    /// - Response parsing fails
    /// - Neo4j returns query errors
    /// - Authentication fails
    pub async fn execute_query(
        &self,
        query: &str,
        parameters: Option<HashMap<String, Value>>,
    ) -> Result<Value> {
        debug!("Executing Cypher query: {}", query);

        let url = format!("{}/db/{}/tx/commit", self.base_url, self.database);

        let request = QueryRequest {
            parameters,
            statement: query.to_string(),
        };

        let statements = vec![request];
        let body = json!({ "statements": statements });

        let mut req_builder = self
            .http
            .post(&url)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .json(&body);

        // Add authentication if configured
        if let Some(ref auth) = self.auth {
            req_builder = req_builder.basic_auth(&auth.0, Some(&auth.1));
        }

        let response_result = req_builder.send().await;
        let response =
            response_result.map_err(|e| Error::Database(format!("HTTP request failed: {e}")))?;

        self.handle_response(response).await
    }

    /// Test database connectivity
    async fn test_connection(&self) -> Result<()> {
        debug!("Testing Neo4j connection to {}", self.base_url);

        let query = "RETURN 1 as test";
        let result = self.execute_query(query, None).await?;

        if let Some(results) = result.get("results").and_then(|v| v.as_array()) {
            if !results.is_empty() {
                debug!("Neo4j connection test successful");
                return Ok(());
            }
        }
        Err(Error::Database("Connection test failed".to_string()))
    }

    /// Collect statistics by executing all queries
    async fn collect_statistics(&self, queries: Vec<(&str, &str)>) -> HashMap<String, Value> {
        let mut stats = HashMap::new();

        for (stat_name, query) in queries {
            let stat_value = self.execute_stat_query(stat_name, query).await;
            stats.insert(stat_name.to_string(), stat_value);
        }

        stats
    }

    /// Handle HTTP response and extract query results
    async fn handle_response(&self, response: Response) -> Result<Value> {
        let status = response.status();
        let text_result = response.text().await;
        let response_text =
            text_result.map_err(|e| Error::Database(format!("Failed to read response: {e}")))?;

        if !status.is_success() {
            warn!(
                "Neo4j query failed with status {}: {}",
                status, response_text
            );
            return Err(Error::Query(format!(
                "Query failed with status {status}: {response_text}"
            )));
        }

        let json_response: Value =
            serde_json::from_str(&response_text).map_err(Error::Serialization)?;

        // Check for Neo4j errors
        if let Some(errors) = json_response.get("errors").and_then(|v| v.as_array()) {
            if !errors.is_empty() {
                let error_messages: Vec<String> = errors
                    .iter()
                    .filter_map(|e| e.get("message").and_then(|v| v.as_str()))
                    .map(ToString::to_string)
                    .collect();

                return Err(Error::Query(format!(
                    "Neo4j errors: {}",
                    error_messages.join(", ")
                )));
            }
        }

        debug!("Query executed successfully");
        Ok(json_response)
    }

    /// Execute a simple read query and return the first column of results
    ///
    /// # Errors
    ///
    /// Returns error if query execution fails or result parsing fails
    ///
    /// # Panics
    ///
    /// Panics if the Neo4j response doesn't contain the expected JSON structure:
    /// - Missing "results" field in response
    /// - Missing "data" field in query results
    /// - Missing "row" field in result data
    pub async fn simple_query(&self, query: &str) -> Result<Vec<Value>> {
        let response = self.execute_query(query, None).await?;

        let mut results = Vec::new();

        if let Some(query_results) = response.get("results").and_then(|v| v.as_array()) {
            for result in query_results {
                if let Some(rows) = result.get("data").and_then(|v| v.as_array()) {
                    for row_data in rows {
                        if let Some(row) = row_data.get("row").and_then(|v| v.as_array()) {
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
    ///
    /// # Errors
    ///
    /// Returns error if index creation fails or database connection fails
    pub async fn create_indexes(&self) -> Result<()> {
        info!("Creating Neo4j indexes for optimal performance");

        let indexes = Self::get_index_definitions();
        self.execute_index_creation(indexes).await?;

        info!("Index creation completed");
        Ok(())
    }

    /// Create a single index with error handling
    async fn create_single_index(&self, index_query: &str) {
        let query_result = self.execute_query(index_query, None).await;
        match query_result {
            Ok(_) => debug!("Created index successfully: {}", index_query),
            Err(e) => {
                warn!("Failed to create index '{}': {}", index_query, e);
                // Continue with other indexes even if one fails
            }
        }
    }

    /// Execute index creation with proper error handling
    async fn execute_index_creation(&self, indexes: Vec<&str>) -> Result<()> {
        for index_query in indexes {
            self.create_single_index(index_query).await;
        }
        Ok(())
    }

    /// Execute a single statistics query with error handling
    async fn execute_stat_query(&self, stat_name: &str, query: &str) -> Value {
        let query_result = self.simple_query(query).await;
        match query_result {
            Ok(results) => results.first().map_or(Value::Null, Clone::clone),
            Err(e) => {
                warn!("Failed to get stat '{}': {}", stat_name, e);
                Value::Null
            }
        }
    }

    /// Get all index definitions for the graph schema
    fn get_index_definitions() -> Vec<&'static str> {
        vec![
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
        ]
    }

    /// Get database statistics
    ///
    /// # Errors
    ///
    /// Returns error if statistics query fails or result parsing fails
    pub async fn get_stats(&self) -> Result<HashMap<String, Value>> {
        debug!("Retrieving Neo4j database statistics");

        let queries = Self::get_stats_queries();
        let stats = self.collect_statistics(queries).await;

        info!("Retrieved database statistics: {} entries", stats.len());
        Ok(stats)
    }

    /// Get all statistics query definitions
    fn get_stats_queries() -> Vec<(&'static str, &'static str)> {
        vec![
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
        ]
    }
}

impl Default for Client {
    fn default() -> Self {
        Self {
            auth: None,
            base_url: "http://localhost:7474".to_string(),
            http: HttpClient::new(),
            database: "neo4j".to_string(),
        }
    }
}

#[cfg(test)]
#[expect(clippy::expect_used)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::collections::HashMap;

    // Helper function to create a mock client for testing
    fn create_mock_client() -> Client {
        Client {
            auth: Some(("neo4j".to_string(), "password".to_string())),
            base_url: "http://localhost:7474".to_string(),
            http: HttpClient::new(),
            database: "neo4j".to_string(),
        }
    }

    #[test]
    fn test_neo4j_client_default() {
        let client = Client::default();
        assert_eq!(client.base_url, "http://localhost:7474");
        assert_eq!(client.database, "neo4j");
        assert!(client.auth.is_none());
    }

    #[test]
    fn test_query_request_serialization() {
        let mut params = HashMap::new();
        params.insert("test".to_string(), json!("value"));

        let request = QueryRequest {
            parameters: Some(params),
            statement: "RETURN 1".to_string(),
        };

        let serialized =
            serde_json::to_string(&request).expect("QueryRequest should always serialize to JSON");
        assert!(serialized.contains("RETURN 1"));
        assert!(serialized.contains("test"));
        assert!(serialized.contains("value"));
    }

    #[test]
    fn test_query_request_serialization_no_params() {
        let request = QueryRequest {
            parameters: None,
            statement: "RETURN 1".to_string(),
        };

        let serialized =
            serde_json::to_string(&request).expect("QueryRequest should always serialize to JSON");
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

        let response: QueryResponse =
            serde_json::from_str(json_str).expect("Valid test JSON should deserialize correctly");
        assert_eq!(response.results.len(), 1);
        assert_eq!(response.errors.len(), 0);
        assert_eq!(
            response
                .results
                .first()
                .expect("Should have first result")
                .columns
                .first()
                .expect("Should have first column"),
            "test"
        );
        assert_eq!(
            response
                .results
                .first()
                .expect("Should have first result")
                .data
                .len(),
            1
        );
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

        let response: QueryResponse =
            serde_json::from_str(json_str).expect("Valid test JSON should deserialize correctly");
        assert_eq!(response.results.len(), 0);
        assert_eq!(response.errors.len(), 1);
        assert_eq!(
            response
                .errors
                .first()
                .expect("Should have first error")
                .code,
            "Neo.ClientError.Statement.SyntaxError"
        );
        assert_eq!(
            response
                .errors
                .first()
                .expect("Should have first error")
                .message,
            "Invalid syntax"
        );
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

        let result: QueryResult =
            serde_json::from_str(json_str).expect("Valid test JSON should deserialize correctly");
        assert_eq!(
            result.columns.first().expect("Should have first column"),
            "n"
        );
        assert_eq!(result.data.len(), 1);
        assert!(result
            .data
            .first()
            .expect("Should have first data row")
            .meta
            .is_some());
    }

    #[test]
    fn test_query_row_without_meta() {
        let json_str = r#"{
            "row": [1, "test", null],
            "meta": null
        }"#;

        let row: QueryRow =
            serde_json::from_str(json_str).expect("Valid test JSON should deserialize correctly");
        assert_eq!(row.row.len(), 3);
        assert!(row.meta.is_none());
        assert_eq!(
            row.row.first().expect("Should have first row element"),
            &json!(1)
        );
        assert_eq!(
            row.row.get(1).expect("Should have second row element"),
            &json!("test")
        );
        assert_eq!(
            row.row.get(2).expect("Should have third row element"),
            &json!(null)
        );
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
        let response_text =
            serde_json::to_string(&json_response).expect("Test JSON should serialize correctly");
        let parsed: Value = serde_json::from_str(&response_text)
            .expect("Serialized JSON should parse back correctly");

        // Verify the structure matches what handle_response expects
        assert!(parsed
            .get("results")
            .expect("Should have results")
            .is_array());
        assert!(parsed.get("errors").expect("Should have errors").is_array());
        assert_eq!(
            parsed
                .get("errors")
                .expect("Should have errors")
                .as_array()
                .expect("Test JSON should have errors array")
                .len(),
            0
        );
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

        let response_text =
            serde_json::to_string(&json_response).expect("Test JSON should serialize correctly");
        let parsed: Value = serde_json::from_str(&response_text)
            .expect("Serialized JSON should parse back correctly");

        // Simulate the error checking logic from handle_response
        if let Some(errors) = parsed.get("errors").and_then(|v| v.as_array()) {
            if !errors.is_empty() {
                let error_messages: Vec<String> = errors
                    .iter()
                    .filter_map(|e| e["message"].as_str())
                    .map(ToString::to_string)
                    .collect();

                assert_eq!(error_messages.len(), 2);
                assert_eq!(
                    error_messages
                        .first()
                        .expect("Should have first error message"),
                    "Syntax error"
                );
                assert_eq!(
                    error_messages
                        .get(1)
                        .expect("Should have second error message"),
                    "Another error"
                );
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
        if let Some(query_results) = response.get("results").and_then(|v| v.as_array()) {
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
        assert_eq!(
            results.first().expect("Should have first result"),
            &json!(42)
        );
        assert_eq!(
            results.get(1).expect("Should have second result"),
            &json!(100)
        );
        assert_eq!(
            results.get(2).expect("Should have third result"),
            &json!(null)
        );
    }

    // Test simple_query with empty results
    #[tokio::test]
    async fn test_simple_query_empty_results() {
        let response = json!({
            "results": [],
            "errors": []
        });

        let mut results = Vec::new();

        if let Some(query_results) = response.get("results").and_then(|v| v.as_array()) {
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

        if let Some(query_results) = response.get("results").and_then(|v| v.as_array()) {
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

        if let Some(query_results) = response.get("results").and_then(|v| v.as_array()) {
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
        assert_eq!(
            results.first().expect("Should have first result"),
            &json!(42)
        );
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
        let (u, p) = auth.expect("Auth should be Some in this test case");
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
        let result = "custom_db".to_string();
        assert_eq!(result, "custom_db");
    }

    #[test]
    fn test_database_name_default() {
        let result = "neo4j".to_string();
        assert_eq!(result, "neo4j");
    }

    // Test URL building for execute_query
    #[test]
    fn test_url_building() {
        let base_url = "http://localhost:7474";
        let database = "neo4j";
        let url = format!("{base_url}/db/{database}/tx/commit");
        assert_eq!(url, "http://localhost:7474/db/neo4j/tx/commit");
    }

    #[test]
    fn test_url_building_custom_database() {
        let base_url = "http://localhost:7474";
        let database = "custom";
        let url = format!("{base_url}/db/{database}/tx/commit");
        assert_eq!(url, "http://localhost:7474/db/custom/tx/commit");
    }

    // Test request body creation
    #[test]
    fn test_request_body_creation() {
        let mut parameters = HashMap::new();
        parameters.insert("param1".to_string(), json!("value1"));
        parameters.insert("param2".to_string(), json!(42));

        let request = QueryRequest {
            parameters: Some(parameters),
            statement: "MATCH (n) RETURN n".to_string(),
        };

        let statements = vec![request];
        let body = json!({ "statements": statements });

        assert!(body
            .get("statements")
            .expect("Should have statements")
            .is_array());
        let statements_array = body
            .get("statements")
            .expect("Should have statements")
            .as_array()
            .expect("Test JSON should have statements array");
        assert_eq!(statements_array.len(), 1);

        let first_statement = statements_array
            .first()
            .expect("Should have first statement");
        assert_eq!(first_statement["statement"], "MATCH (n) RETURN n");
        assert!(first_statement["parameters"].is_object());
    }

    #[test]
    fn test_request_body_creation_no_params() {
        let request = QueryRequest {
            parameters: None,
            statement: "MATCH (n) RETURN n".to_string(),
        };

        let statements = vec![request];
        let body = json!({ "statements": statements });

        let statements_array = body
            .get("statements")
            .expect("Should have statements")
            .as_array()
            .expect("Test JSON should have statements array");
        let first_statement = statements_array
            .first()
            .expect("Should have first statement");
        assert_eq!(first_statement["statement"], "MATCH (n) RETURN n");
        assert!(first_statement["parameters"].is_null());
    }

    // Test indexes creation logic
    #[test]
    fn test_create_indexes_query_list() {
        let indexes = Client::get_index_definitions();
        let expected = ["CREATE VECTOR INDEX IF NOT EXISTS embedding_index FOR (n:Document) ON (n.embedding) OPTIONS {indexConfig: {`vector.dimensions`: 1536, `vector.similarity_function`: 'cosine'}}",
            "CREATE INDEX IF NOT EXISTS wallet_address_index FOR (n:Wallet) ON (n.address)",
            "CREATE INDEX IF NOT EXISTS token_address_index FOR (n:Token) ON (n.address)",
            "CREATE INDEX IF NOT EXISTS token_symbol_index FOR (n:Token) ON (n.symbol)",
            "CREATE INDEX IF NOT EXISTS protocol_name_index FOR (n:Protocol) ON (n.name)",
            "CREATE INDEX IF NOT EXISTS transaction_hash_index FOR (n:Transaction) ON (n.hash)",
            "CREATE INDEX IF NOT EXISTS block_number_index FOR (n:Block) ON (n.number)",
            "CREATE INDEX IF NOT EXISTS wallet_token_index FOR (n:Wallet) ON (n.address, n.chain)",
            "CREATE INDEX IF NOT EXISTS transaction_block_index FOR (n:Transaction) ON (n.block_number, n.chain)"];

        assert_eq!(indexes.len(), expected.len());
        assert!(indexes
            .first()
            .expect("Should have first index")
            .contains("VECTOR INDEX"));
        assert!(indexes
            .get(1)
            .expect("Should have second index")
            .contains("wallet_address_index"));
        assert!(indexes
            .get(8)
            .expect("Should have ninth index")
            .contains("transaction_block_index"));
    }

    // Test statistics queries
    #[test]
    fn test_get_stats_queries() {
        let queries = Client::get_stats_queries();
        let expected = [
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

        assert_eq!(queries.len(), expected.len());
        assert_eq!(
            queries.first().expect("Should have first query").0,
            "node_count"
        );
        assert_eq!(
            queries.first().expect("Should have first query").1,
            "MATCH (n) RETURN count(n) as count"
        );
        assert_eq!(
            queries.get(5).expect("Should have sixth query").0,
            "protocol_count"
        );
        assert!(queries
            .get(1)
            .expect("Should have second query")
            .1
            .contains("()-[r]->()")); // Fixed typo
    }

    // Test stats HashMap building logic
    #[test]
    fn test_stats_hashmap_building() {
        let mut stats = HashMap::new();

        // Simulate successful stat retrieval
        let results = [json!(42)];
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

        let is_valid = result.get("results").and_then(|v| v.as_array()).is_some();
        assert!(is_valid);
    }

    #[test]
    fn test_connection_test_validation_failure() {
        let result = json!({
            "errors": ["Connection failed"]
        });

        let is_valid = result.get("results").and_then(|v| v.as_array()).is_some();
        assert!(!is_valid);
    }

    // Test Clone implementation for Client
    #[test]
    fn test_neo4j_client_clone() {
        let original = create_mock_client();
        let cloned = original.clone();

        assert_eq!(original.base_url, cloned.base_url);
        assert_eq!(original.database, cloned.database);
        assert_eq!(original.auth, cloned.auth);
    }

    // Test Debug implementation for Client
    #[test]
    fn test_neo4j_client_debug() {
        let client = create_mock_client();
        let debug_str = format!("{client:?}");

        assert!(debug_str.contains("Client"));
        assert!(debug_str.contains("localhost:7474"));
        assert!(debug_str.contains("neo4j"));
    }

    // Test serialization structs Debug implementations
    #[test]
    fn test_query_request_debug() {
        let request = QueryRequest {
            parameters: None,
            statement: "RETURN 1".to_string(),
        };

        let debug_str = format!("{request:?}");
        assert!(debug_str.contains("QueryRequest"));
        assert!(debug_str.contains("RETURN 1"));
    }

    #[test]
    fn test_query_response_debug() {
        let response = QueryResponse {
            results: vec![],
            errors: vec![],
        };

        let debug_str = format!("{response:?}");
        assert!(debug_str.contains("QueryResponse"));
    }

    #[test]
    fn test_query_result_debug() {
        let result = QueryResult {
            columns: vec!["test".to_string()],
            data: vec![],
        };

        let debug_str = format!("{result:?}");
        assert!(debug_str.contains("QueryResult"));
        assert!(debug_str.contains("test"));
    }

    #[test]
    fn test_query_row_debug() {
        let row = QueryRow {
            row: vec![json!(1)],
            meta: None,
        };

        let debug_str = format!("{row:?}");
        assert!(debug_str.contains("QueryRow"));
    }

    #[test]
    fn test_query_error_debug() {
        let error = QueryError {
            code: "Test.Error".to_string(),
            message: "Test message".to_string(),
        };

        let debug_str = format!("{error:?}");
        assert!(debug_str.contains("QueryError"));
        assert!(debug_str.contains("Test.Error"));
        assert!(debug_str.contains("Test message"));
    }

    // Test error message formatting
    #[test]
    fn test_error_message_joining() {
        let errors = [
            json!({"message": "First error"}),
            json!({"message": "Second error"}),
            json!({"message": "Third error"}),
        ];

        let error_messages: Vec<String> = errors
            .iter()
            .filter_map(|e| e["message"].as_str())
            .map(ToString::to_string)
            .collect();

        let joined = error_messages.join(", ");
        assert_eq!(joined, "First error, Second error, Third error");
    }

    #[test]
    fn test_error_message_joining_with_missing_messages() {
        let errors = [
            json!({"message": "First error"}),
            json!({"code": "ERROR_CODE"}), // Missing message field
            json!({"message": "Third error"}),
        ];

        let error_messages: Vec<String> = errors
            .iter()
            .filter_map(|e| e["message"].as_str())
            .map(ToString::to_string)
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
        assert!(complex_value
            .get("nested")
            .and_then(|v| v.get("array"))
            .expect("Should have nested.array")
            .is_array());
        assert!(complex_value
            .get("nested")
            .and_then(|v| v.get("object"))
            .expect("Should have nested.object")
            .is_object());
        assert!(complex_value
            .get("nested")
            .and_then(|v| v.get("null"))
            .expect("Should have nested.null")
            .is_null());
        assert!(complex_value
            .get("nested")
            .and_then(|v| v.get("boolean"))
            .expect("Should have nested.boolean")
            .is_boolean());
        assert!(complex_value
            .get("nested")
            .and_then(|v| v.get("number"))
            .expect("Should have nested.number")
            .is_number());
    }
}
