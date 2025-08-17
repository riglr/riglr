//! Integration tests for web tools functionality

use riglr_web_tools::{client::WebClient, dexscreener::*, news::*, price::*, web_search::*};
use serde_json::json;
use wiremock::{
    matchers::{method, path, query_param},
    Mock, MockServer, ResponseTemplate,
};

// Constants for environment variable names
const DEXSCREENER_BASE_URL: &str = "DEXSCREENER_BASE_URL";
const DEXSCREENER_API_URL: &str = "DEXSCREENER_API_URL";
const EXA_API_URL: &str = "EXA_API_URL";
const EXA_API_KEY: &str = "EXA_API_KEY";
const NEWSAPI_URL: &str = "NEWSAPI_URL";
const NEWSAPI_KEY: &str = "NEWSAPI_KEY";
const CRYPTOPANIC_URL: &str = "CRYPTOPANIC_URL";
const CRYPTOPANIC_KEY: &str = "CRYPTOPANIC_KEY";

/// Helper to create a WebClient that points to our mock server
#[allow(dead_code)]
async fn create_mock_client(server: &MockServer) -> WebClient {
    let mut client = WebClient::new().unwrap();
    // Override the base URL to point to our mock server
    client.set_config("base_url", &server.uri());
    client
}

#[tokio::test]
async fn test_dexscreener_token_info_success() {
    // Initialize logging
    let _ = tracing_subscriber::fmt::try_init();

    // Start a mock server
    let mock_server = MockServer::start().await;

    // Create a mock response that matches DexScreener's actual API format
    let mock_response = json!({
        "schemaVersion": "1.0.0",
        "pairs": [
            {
                "chainId": "ethereum",
                "dexId": "uniswap_v3",
                "url": "https://dexscreener.com/ethereum/0x88e6a0c2ddd26feeb64f039a2c41296fcb3f5640",
                "pairAddress": "0x88e6a0c2ddd26feeb64f039a2c41296fcb3f5640",
                "baseToken": {
                    "address": "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
                    "name": "USD Coin",
                    "symbol": "USDC"
                },
                "quoteToken": {
                    "address": "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2",
                    "name": "Wrapped Ether",
                    "symbol": "WETH"
                },
                "priceNative": "0.00038462",
                "priceUsd": "1.00001",
                "txns": {
                    "h24": {
                        "buys": 5432,
                        "sells": 4891
                    }
                },
                "volume": {
                    "h24": 25000000.0
                },
                "priceChange": {
                    "h24": 0.05,
                    "h1": -0.01,
                    "m5": 0.001
                },
                "liquidity": {
                    "usd": 50000000.0,
                    "base": 25000000,
                    "quote": 9615.38
                },
                "fdv": 5200000000i64,
                "marketCap": 5200000000i64
            }
        ]
    });

    // Configure the mock to respond to token info requests
    Mock::given(method("GET"))
        .and(path(
            "/dex/tokens/0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(&mock_response))
        .mount(&mock_server)
        .await;

    // Override the DexScreener base URL to use our mock server
    unsafe {
        // SAFETY: Safe in test environment where we control thread access
        std::env::set_var(DEXSCREENER_BASE_URL, mock_server.uri());
    }

    // Call the actual function
    let result = get_token_info(
        "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string(),
        Some("ethereum".to_string()),
        Some(true),
        None,
    )
    .await;

    // Verify the result
    assert!(result.is_ok());
    let token_info = result.unwrap();
    println!("Token info: {:?}", token_info);
    println!("Price USD: {:?}", token_info.price_usd);
    assert_eq!(token_info.symbol, "USDC");
    assert_eq!(token_info.name, "USD Coin");
    // Price should be parsed from the string "1.00001"
    assert!(token_info.price_usd.is_some());
    let price = token_info.price_usd.unwrap();
    println!("Actual price: {}, Expected: 1.00001", price);
    assert!((price - 1.00001).abs() < 0.00001);
    assert_eq!(token_info.market_cap, Some(5200000000.0));
    assert_eq!(token_info.volume_24h, Some(25000000.0));
    assert_eq!(token_info.pairs.len(), 1);

    // Clean up
    unsafe {
        // SAFETY: Safe in test environment where we control thread access
        std::env::remove_var(DEXSCREENER_BASE_URL);
    }
}

#[tokio::test]
async fn test_dexscreener_token_not_found() {
    let mock_server = MockServer::start().await;

    // Mock a response with no pairs (token not found)
    let mock_response = json!({
        "schemaVersion": "1.0.0",
        "pairs": []
    });

    Mock::given(method("GET"))
        .and(path("/dex/tokens/0xInvalidAddress"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&mock_response))
        .mount(&mock_server)
        .await;

    unsafe {
        // SAFETY: Safe in test environment where we control thread access
        std::env::set_var(DEXSCREENER_BASE_URL, mock_server.uri());
    }

    let result = get_token_info(
        "0xInvalidAddress".to_string(),
        Some("ethereum".to_string()),
        Some(true),
        None,
    )
    .await;

    // Should return an error when no pairs are found
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("No pairs found"));

    unsafe {
        // SAFETY: Safe in test environment where we control thread access
        std::env::remove_var(DEXSCREENER_BASE_URL);
    }
}

#[tokio::test]
async fn test_dexscreener_api_error_handling() {
    let mock_server = MockServer::start().await;

    // Mock different error scenarios
    // 1. 401 Unauthorized
    Mock::given(method("GET"))
        .and(path("/dex/tokens/unauthorized"))
        .respond_with(ResponseTemplate::new(401).set_body_string("Unauthorized"))
        .mount(&mock_server)
        .await;

    // 2. 429 Rate Limited
    Mock::given(method("GET"))
        .and(path("/dex/tokens/rate_limited"))
        .respond_with(ResponseTemplate::new(429).set_body_string("Rate limit exceeded"))
        .mount(&mock_server)
        .await;

    // 3. 500 Server Error (should retry)
    Mock::given(method("GET"))
        .and(path("/dex/tokens/server_error"))
        .respond_with(ResponseTemplate::new(500).set_body_string("Internal server error"))
        .up_to_n_times(2)
        .mount(&mock_server)
        .await;

    // After retries, return success
    Mock::given(method("GET"))
        .and(path("/dex/tokens/server_error"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "schemaVersion": "1.0.0",
            "pairs": [{
                "baseToken": {"address": "0x123", "name": "Test", "symbol": "TEST"},
                "quoteToken": {"address": "0x456", "name": "WETH", "symbol": "WETH"},
                "priceUsd": "1.0",
                "priceNative": "0.001",
                "pairAddress": "0x789",
                "dexId": "uniswap_v3",
                "url": "https://test.com"
            }]
        })))
        .mount(&mock_server)
        .await;

    unsafe {
        // SAFETY: Safe in test environment where we control thread access
        std::env::set_var(DEXSCREENER_BASE_URL, mock_server.uri());
    }

    // Test 401 - should not retry
    let result = get_token_info("unauthorized".to_string(), None, None, None).await;
    assert!(result.is_err());

    // Test 429 - should not retry
    let result = get_token_info("rate_limited".to_string(), None, None, None).await;
    assert!(result.is_err());

    // Test 500 - should retry and eventually succeed
    let result = get_token_info("server_error".to_string(), None, None, None).await;
    assert!(result.is_ok());

    unsafe {
        // SAFETY: Safe in test environment where we control thread access
        std::env::remove_var(DEXSCREENER_BASE_URL);
    }
}

#[tokio::test]
async fn test_price_fetch_with_multiple_pairs() {
    let mock_server = MockServer::start().await;

    // Mock response with multiple pairs to test aggregation
    let mock_response = json!({
        "pairs": [
            {
                "priceUsd": "2500.50",
                "liquidity": {"usd": 1000000.0},
                "baseToken": {"address": "0xETH", "symbol": "ETH"},
                "dexId": "uniswap_v3",
                "pairAddress": "0xpair1"
            },
            {
                "priceUsd": "2499.75",
                "liquidity": {"usd": 500000.0},
                "baseToken": {"address": "0xETH", "symbol": "ETH"},
                "dexId": "sushiswap",
                "pairAddress": "0xpair2"
            },
            {
                "priceUsd": "2501.00",
                "liquidity": {"usd": 2000000.0},  // Highest liquidity
                "baseToken": {"address": "0xETH", "symbol": "ETH"},
                "dexId": "curve",
                "pairAddress": "0xpair3"
            }
        ]
    });

    Mock::given(method("GET"))
        .and(path("/latest/dex/search/"))
        .and(query_param("q", "ethereum:0xETH"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&mock_response))
        .mount(&mock_server)
        .await;

    // Override base URL for DexScreener API
    unsafe {
        // SAFETY: Safe in test environment where we control thread access
        std::env::set_var(DEXSCREENER_API_URL, format!("{}/latest", mock_server.uri()));
    }

    // The price module should select the pair with highest liquidity
    let result = get_token_price("0xETH".to_string(), Some("ethereum".to_string())).await;

    assert!(result.is_ok());
    let price_result = result.unwrap();
    assert_eq!(price_result.token_symbol, Some("ETH".to_string()));
    assert_eq!(price_result.price_usd, "2501.00"); // From highest liquidity pair
    assert_eq!(price_result.source_dex, Some("curve".to_string()));
    assert_eq!(price_result.source_liquidity_usd, Some(2000000.0));

    unsafe {
        // SAFETY: Safe in test environment where we control thread access
        std::env::remove_var(DEXSCREENER_API_URL);
    }
}

#[tokio::test]
async fn test_search_tokens_with_filters() {
    let mock_server = MockServer::start().await;

    let mock_response = json!({
        "schemaVersion": "1.0.0",
        "pairs": [
            {
                "chainId": "ethereum",
                "dexId": "uniswap_v3",
                "baseToken": {
                    "address": "0xPEPE",
                    "name": "Pepe",
                    "symbol": "PEPE"
                },
                "quoteToken": {
                    "address": "0xWETH",
                    "name": "Wrapped Ether",
                    "symbol": "WETH"
                },
                "priceUsd": "0.00001234",
                "priceNative": "0.000000005",
                "pairAddress": "0xPEPEPair",
                "url": "https://dexscreener.com/ethereum/0xPEPEPair",
                "liquidity": {"usd": 5000000.0},
                "marketCap": 1000000000i64,
                "volume": {"h24": 10000000.0}
            }
        ]
    });

    Mock::given(method("GET"))
        .and(path("/dex/search"))
        .and(query_param("q", "pepe"))
        .and(query_param("chain", "ethereum"))
        .and(query_param("min_market_cap", "1000000"))
        .and(query_param("min_liquidity", "100000"))
        .and(query_param("limit", "10"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&mock_response))
        .mount(&mock_server)
        .await;

    unsafe {
        // SAFETY: Safe in test environment where we control thread access
        std::env::set_var(DEXSCREENER_BASE_URL, mock_server.uri());
    }

    let result = search_tokens(
        "pepe".to_string(),
        Some("ethereum".to_string()),
        Some(1000000.0),
        Some(100000.0),
        Some(10),
    )
    .await;

    assert!(result.is_ok());
    let search_result = result.unwrap();
    assert_eq!(search_result.query, "pepe");
    assert_eq!(search_result.tokens.len(), 1);
    assert_eq!(search_result.tokens[0].symbol, "PEPE");
    assert_eq!(search_result.metadata.result_count, 1);

    unsafe {
        // SAFETY: Safe in test environment where we control thread access
        std::env::remove_var(DEXSCREENER_BASE_URL);
    }
}

#[tokio::test]
async fn test_web_search_with_exa_api() {
    let mock_server = MockServer::start().await;

    // Mock Exa API response
    let mock_response = json!({
        "results": [
            {
                "url": "https://example.com/article1",
                "title": "Rust Programming Best Practices",
                "snippet": "Learn about the best practices in Rust programming...",
                "publishedDate": "2024-01-15T10:00:00Z",
                "score": 0.95
            },
            {
                "url": "https://example.com/article2",
                "title": "Advanced Rust Techniques",
                "snippet": "Explore advanced techniques and patterns in Rust...",
                "publishedDate": "2024-01-14T15:30:00Z",
                "score": 0.89
            }
        ],
        "autopromptString": "rust programming language"
    });

    Mock::given(method("POST"))
        .and(path("/search"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&mock_response))
        .mount(&mock_server)
        .await;

    // Set mock server as Exa API URL
    unsafe {
        // SAFETY: Safe in test environment where we control thread access
        std::env::set_var(EXA_API_URL, mock_server.uri());
        std::env::set_var(EXA_API_KEY, "test_key");
    }

    let result = search_web(
        "rust programming".to_string(),
        Some(10),
        Some(false),
        None,
        None,
        None,
    )
    .await;

    assert!(result.is_ok());
    let search_result = result.unwrap();
    assert_eq!(search_result.query, "rust programming");
    assert_eq!(search_result.results.len(), 2);
    assert_eq!(
        search_result.results[0].title,
        "Rust Programming Best Practices"
    );
    assert!(search_result.results[0].relevance_score > 0.9);

    unsafe {
        // SAFETY: Safe in test environment where we control thread access
        std::env::remove_var(EXA_API_URL);
        std::env::remove_var(EXA_API_KEY);
    }
}

#[tokio::test]
async fn test_news_aggregation_with_mock_apis() {
    let mock_server = MockServer::start().await;

    // Mock NewsAPI response
    let newsapi_response = json!({
        "status": "ok",
        "totalResults": 2,
        "articles": [
            {
                "source": {
                    "id": "techcrunch",
                    "name": "TechCrunch"
                },
                "author": "John Doe",
                "title": "Bitcoin Hits New All-Time High",
                "description": "Bitcoin reaches unprecedented levels...",
                "url": "https://techcrunch.com/bitcoin-ath",
                "urlToImage": "https://techcrunch.com/image.jpg",
                "publishedAt": "2024-01-20T12:00:00Z",
                "content": "Full article content about Bitcoin's rise..."
            }
        ]
    });

    Mock::given(method("GET"))
        .and(path("/v2/everything"))
        .and(query_param("q", "bitcoin"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&newsapi_response))
        .mount(&mock_server)
        .await;

    // Mock CryptoPanic response
    let cryptopanic_response = json!({
        "count": 1,
        "results": [
            {
                "id": 12345,
                "title": "Ethereum Upgrade Successful",
                "published_at": "2024-01-20T10:00:00Z",
                "url": "https://cryptopanic.com/news/ethereum",
                "source": {
                    "title": "CryptoPanic",
                    "domain": "cryptopanic.com"
                },
                "kind": "news",
                "votes": {
                    "positive": 150,
                    "negative": 10,
                    "important": 50
                }
            }
        ]
    });

    Mock::given(method("GET"))
        .and(path("/api/v1/posts/"))
        .and(query_param("filter", "bitcoin"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&cryptopanic_response))
        .mount(&mock_server)
        .await;

    unsafe {
        // SAFETY: Safe in test environment where we control thread access
        std::env::set_var(NEWSAPI_URL, format!("{}/v2", mock_server.uri()));
        std::env::set_var(NEWSAPI_KEY, "test_newsapi_key");
        std::env::set_var(CRYPTOPANIC_URL, format!("{}/api/v1", mock_server.uri()));
        std::env::set_var(CRYPTOPANIC_KEY, "test_cryptopanic_key");
    }

    let result = get_crypto_news(
        "bitcoin".to_string(),
        Some("24h".to_string()),
        None,
        Some(60),
        Some(true),
    )
    .await;

    // Even with mocked APIs, the function creates sample articles
    // This tests the aggregation logic
    assert!(result.is_ok());
    let news_result = result.unwrap();
    assert_eq!(news_result.topic, "bitcoin");
    assert!(!news_result.articles.is_empty());
    assert!(!news_result.metadata.sources_queried.is_empty());

    unsafe {
        // SAFETY: Safe in test environment where we control thread access
        std::env::remove_var(NEWSAPI_URL);
        std::env::remove_var(NEWSAPI_KEY);
        std::env::remove_var(CRYPTOPANIC_URL);
        std::env::remove_var(CRYPTOPANIC_KEY);
    }
}

#[tokio::test]
async fn test_client_retry_logic() {
    let mock_server = MockServer::start().await;

    // First two attempts return 503, third succeeds
    Mock::given(method("GET"))
        .and(path("/retry-test"))
        .respond_with(ResponseTemplate::new(503).set_body_string("Service unavailable"))
        .up_to_n_times(2)
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/retry-test"))
        .respond_with(ResponseTemplate::new(200).set_body_string("Success after retries"))
        .mount(&mock_server)
        .await;

    let client = WebClient::new().unwrap();
    let url = format!("{}/retry-test", mock_server.uri());

    let result = client.get(&url).await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Success after retries");
}

#[tokio::test]
async fn test_client_no_retry_on_client_errors() {
    let mock_server = MockServer::start().await;

    // 400 Bad Request should not retry
    Mock::given(method("GET"))
        .and(path("/bad-request"))
        .respond_with(ResponseTemplate::new(400).set_body_string("Bad request"))
        .expect(1) // Should only be called once
        .mount(&mock_server)
        .await;

    let client = WebClient::new().unwrap();
    let url = format!("{}/bad-request", mock_server.uri());

    let result = client.get(&url).await;

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("400"));
}

#[tokio::test]
async fn test_malformed_json_response() {
    let mock_server = MockServer::start().await;

    // Return invalid JSON
    Mock::given(method("GET"))
        .and(path("/dex/tokens/malformed"))
        .respond_with(ResponseTemplate::new(200).set_body_string("{ invalid json }"))
        .mount(&mock_server)
        .await;

    unsafe {
        // SAFETY: Safe in test environment where we control thread access
        std::env::set_var(DEXSCREENER_BASE_URL, mock_server.uri());
    }

    let result = get_token_info("malformed".to_string(), None, None, None).await;

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("parse"));

    unsafe {
        // SAFETY: Safe in test environment where we control thread access
        std::env::remove_var(DEXSCREENER_BASE_URL);
    }
}

#[tokio::test]
async fn test_concurrent_requests() {
    let mock_server = MockServer::start().await;

    // Set up multiple endpoints
    for i in 1..=5 {
        let path_str = format!("/token{}", i);
        Mock::given(method("GET"))
            .and(path(path_str.as_str()))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "id": i,
                "data": format!("Token {}", i)
            })))
            .mount(&mock_server)
            .await;
    }

    let client = WebClient::new().unwrap();
    let base_url = mock_server.uri();

    // Make concurrent requests
    let futures: Vec<_> = (1..=5)
        .map(|i| {
            let url = format!("{}/token{}", base_url, i);
            let client = client.clone();
            async move { client.get(&url).await }
        })
        .collect();

    let results = futures::future::join_all(futures).await;

    // All requests should succeed
    assert_eq!(results.len(), 5);
    for result in results {
        assert!(result.is_ok());
    }
}
