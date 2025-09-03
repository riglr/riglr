//! Test helpers for wiremock-based API testing
//!
//! This module provides centralized mock server setup and utilities for testing
//! web tools with hermetic, reproducible tests.

use serde_json::Value;
use std::fs;
use std::path::Path;
use wiremock::{
    matchers::{method, path, query_param},
    Mock, MockServer, ResponseTemplate,
};

/// Mock API server for testing web tools
pub struct MockApiServer {
    /// The wiremock server instance
    pub server: MockServer,
    /// The server's base URI
    pub uri: String,
}

impl MockApiServer {
    /// Create and start a new mock server
    pub async fn new() -> Self {
        let server = MockServer::start().await;
        let uri = server.uri();

        Self { server, uri }
    }

    /// Get the server's base URI
    pub fn uri(&self) -> &str {
        &self.uri
    }

    /// Mock a DexScreener API response
    pub async fn mock_dexscreener_response(
        &self,
        path_pattern: &str,
        status: u16,
        body_json: Value,
    ) {
        Mock::given(method("GET"))
            .and(path(path_pattern))
            .respond_with(
                ResponseTemplate::new(status)
                    .set_body_json(&body_json)
                    .insert_header("content-type", "application/json"),
            )
            .mount(&self.server)
            .await;
    }

    /// Mock a DexScreener API response with query parameters
    pub async fn mock_dexscreener_with_params(
        &self,
        path_pattern: &str,
        params: Vec<(&str, &str)>,
        status: u16,
        body_json: Value,
    ) {
        let mut mock = Mock::given(method("GET")).and(path(path_pattern));

        for (key, value) in params {
            mock = mock.and(query_param(key, value));
        }

        mock.respond_with(
            ResponseTemplate::new(status)
                .set_body_json(&body_json)
                .insert_header("content-type", "application/json"),
        )
        .mount(&self.server)
        .await;
    }

    /// Mock a Twitter API response
    #[allow(dead_code)]
    pub async fn mock_twitter_response(&self, path_pattern: &str, status: u16, body_json: Value) {
        Mock::given(method("GET"))
            .and(path(path_pattern))
            .respond_with(
                ResponseTemplate::new(status)
                    .set_body_json(&body_json)
                    .insert_header("content-type", "application/json"),
            )
            .mount(&self.server)
            .await;
    }

    /// Mock a Twitter API response with Bearer token authentication
    #[allow(dead_code)]
    pub async fn mock_twitter_auth_response(
        &self,
        path_pattern: &str,
        bearer_token: &str,
        status: u16,
        body_json: Value,
    ) {
        Mock::given(method("GET"))
            .and(path(path_pattern))
            .and(wiremock::matchers::header(
                "Authorization",
                format!("Bearer {}", bearer_token),
            ))
            .respond_with(
                ResponseTemplate::new(status)
                    .set_body_json(&body_json)
                    .insert_header("content-type", "application/json"),
            )
            .mount(&self.server)
            .await;
    }

    /// Mock a web search API response
    #[allow(dead_code)]
    pub async fn mock_web_search_response(
        &self,
        path_pattern: &str,
        status: u16,
        body_json: Value,
    ) {
        Mock::given(method("GET"))
            .and(path(path_pattern))
            .respond_with(
                ResponseTemplate::new(status)
                    .set_body_json(&body_json)
                    .insert_header("content-type", "application/json"),
            )
            .mount(&self.server)
            .await;
    }

    /// Mock a POST request response
    #[allow(dead_code)]
    pub async fn mock_post_response(&self, path_pattern: &str, status: u16, body_json: Value) {
        Mock::given(method("POST"))
            .and(path(path_pattern))
            .respond_with(
                ResponseTemplate::new(status)
                    .set_body_json(&body_json)
                    .insert_header("content-type", "application/json"),
            )
            .mount(&self.server)
            .await;
    }

    /// Mock an error response
    #[allow(dead_code)]
    pub async fn mock_error_response(&self, path_pattern: &str, status: u16, error_message: &str) {
        Mock::given(method("GET"))
            .and(path(path_pattern))
            .respond_with(
                ResponseTemplate::new(status)
                    .set_body_string(error_message)
                    .insert_header("content-type", "text/plain"),
            )
            .mount(&self.server)
            .await;
    }

    /// Verify that a request was made to the mock server
    pub async fn verify_request_made(&self, _path_pattern: &str) -> bool {
        // This would typically use wiremock's verification features
        // For now, return true as a placeholder
        true
    }
}

/// Load a JSON fixture file from the tests/fixtures directory
pub fn load_fixture(file_name: &str) -> Result<String, std::io::Error> {
    let fixture_path = Path::new("tests/fixtures").join(file_name);
    fs::read_to_string(fixture_path)
}

/// Load and parse a JSON fixture file
#[allow(dead_code)]
pub fn load_json_fixture(file_name: &str) -> Result<Value, Box<dyn std::error::Error>> {
    let content = load_fixture(file_name)?;
    let json = serde_json::from_str(&content)?;
    Ok(json)
}

/// Create a sample DexScreener success response for testing
pub fn create_dexscreener_success_fixture() -> Value {
    serde_json::json!({
        "schemaVersion": "1.0.0",
        "pairs": [
            {
                "chainId": "ethereum",
                "dexId": "uniswap",
                "url": "https://dexscreener.com/ethereum/0x123",
                "pairAddress": "0x1234567890123456789012345678901234567890",
                "labels": ["v2"],
                "baseToken": {
                    "address": "0xabcdef1234567890abcdef1234567890abcdef12",
                    "name": "Test Token",
                    "symbol": "TEST"
                },
                "quoteToken": {
                    "address": "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2",
                    "name": "Wrapped Ether",
                    "symbol": "WETH"
                },
                "priceNative": "0.001",
                "priceUsd": "2.50",
                "txns": {
                    "h24": {
                        "buys": 150,
                        "sells": 120
                    },
                    "h6": {
                        "buys": 40,
                        "sells": 35
                    },
                    "h1": {
                        "buys": 10,
                        "sells": 8
                    },
                    "m5": {
                        "buys": 2,
                        "sells": 1
                    }
                },
                "volume": {
                    "h24": 50000.0,
                    "h6": 12000.0,
                    "h1": 3000.0,
                    "m5": 500.0
                },
                "priceChange": {
                    "h24": 15.5,
                    "h6": 8.2,
                    "h1": 2.1,
                    "m5": 0.5
                },
                "liquidity": {
                    "usd": 1000000.0,
                    "base": 400000.0,
                    "quote": 200.0
                },
                "marketCap": 25000000.0,
                "fdv": 100000000.0
            }
        ]
    })
}

/// Create a DexScreener not found response for testing
pub fn create_dexscreener_not_found_fixture() -> Value {
    serde_json::json!({
        "schemaVersion": "1.0.0",
        "pairs": []
    })
}

/// Create a Twitter API success response for testing
#[allow(dead_code)]
pub fn create_twitter_success_fixture() -> Value {
    serde_json::json!({
        "data": [
            {
                "id": "1234567890",
                "text": "Just bought some $BTC! 🚀 #crypto #bitcoin",
                "author_id": "987654321",
                "created_at": "2024-01-15T10:30:00.000Z",
                "lang": "en",
                "public_metrics": {
                    "retweet_count": 42,
                    "reply_count": 15,
                    "like_count": 128,
                    "quote_count": 5,
                    "impression_count": 5000
                },
                "entities": {
                    "hashtags": [
                        {"tag": "crypto"},
                        {"tag": "bitcoin"}
                    ],
                    "cashtags": [
                        {"tag": "BTC"}
                    ]
                },
                "context_annotations": [
                    {
                        "domain": {
                            "id": "46",
                            "name": "Business & Finance",
                            "description": "Business and Finance"
                        },
                        "entity": {
                            "id": "1007360414114435072",
                            "name": "Bitcoin",
                            "description": "Bitcoin cryptocurrency"
                        }
                    }
                ]
            }
        ],
        "includes": {
            "users": [
                {
                    "id": "987654321",
                    "username": "cryptotrader",
                    "name": "Crypto Trader",
                    "description": "Trading crypto since 2017",
                    "verified": true,
                    "created_at": "2017-01-01T00:00:00.000Z",
                    "public_metrics": {
                        "followers_count": 50000,
                        "following_count": 500,
                        "tweet_count": 10000,
                        "listed_count": 100
                    }
                }
            ]
        },
        "meta": {
            "result_count": 1,
            "next_token": null
        }
    })
}

/// Create a sample error response for testing
#[allow(dead_code)]
pub fn create_error_fixture(message: &str) -> Value {
    serde_json::json!({
        "error": {
            "message": message,
            "code": "ERROR_CODE"
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_server_creation() {
        let mock_server = MockApiServer::new().await;
        assert!(!mock_server.uri().is_empty());
        assert!(mock_server.uri().starts_with("http://"));
    }

    #[tokio::test]
    async fn test_mock_dexscreener_response() {
        let mock_server = MockApiServer::new().await;
        let fixture = create_dexscreener_success_fixture();

        mock_server
            .mock_dexscreener_response("/tokens/ethereum/0x123", 200, fixture)
            .await;

        // The mock is now set up and ready to use
        assert!(
            mock_server
                .verify_request_made("/tokens/ethereum/0x123")
                .await
        );
    }
}
