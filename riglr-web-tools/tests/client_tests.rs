//! Comprehensive tests for client module

use riglr_web_tools::client::WebClient;

#[test]
fn test_web_client_new() {
    let client = WebClient::new().expect("Failed to create client");

    assert!(client.api_keys.is_empty());
    assert!(client.config.is_empty());
}

#[test]
fn test_web_client_with_api_key() {
    let client = WebClient::new().expect("Failed to create client")
        .with_api_key("service1", "key1")
        .with_api_key("service2", "key2");

    assert_eq!(client.api_keys.get("service1"), Some(&"key1".to_string()));
    assert_eq!(client.api_keys.get("service2"), Some(&"key2".to_string()));
}

#[test]
fn test_web_client_with_twitter_token() {
    let client = WebClient::new().expect("Failed to create client").with_twitter_token("bearer_token_123");

    assert_eq!(
        client.api_keys.get("twitter"),
        Some(&"bearer_token_123".to_string())
    );
}

#[test]
fn test_web_client_with_exa_key() {
    let client = WebClient::new().expect("Failed to create client").with_exa_key("exa_api_key_456");

    assert_eq!(
        client.api_keys.get("exa"),
        Some(&"exa_api_key_456".to_string())
    );
}

#[test]
fn test_web_client_with_dexscreener_key() {
    let client = WebClient::new().expect("Failed to create client").with_dexscreener_key("dex_key_789");

    assert_eq!(
        client.api_keys.get("dexscreener"),
        Some(&"dex_key_789".to_string())
    );
}

#[test]
fn test_web_client_with_config() {
    let mut client = WebClient::new().expect("Failed to create client");
    client.set_config("timeout", "30");
    client.set_config("retry_count", "3");

    assert_eq!(client.config.get("timeout"), Some(&"30".to_string()));
    assert_eq!(client.config.get("retry_count"), Some(&"3".to_string()));
}

#[test]
fn test_web_client_chaining() {
    let mut client = WebClient::new().expect("Failed to create client")
        .with_api_key("service1", "key1")
        .with_twitter_token("twitter_token")
        .with_exa_key("exa_key")
        .with_dexscreener_key("dex_key");
    client.set_config("option1", "value1");
    client.set_config("option2", "value2");

    assert_eq!(client.api_keys.len(), 4);
    assert_eq!(client.config.len(), 2);
}

#[test]
fn test_web_client_overwrite_api_key() {
    let client = WebClient::new().expect("Failed to create client")
        .with_api_key("service", "old_key")
        .with_api_key("service", "new_key");

    assert_eq!(client.api_keys.get("service"), Some(&"new_key".to_string()));
}

#[test]
fn test_web_client_get_api_key() {
    let client = WebClient::new().expect("Failed to create client").with_api_key("test", "test_key");

    let key = client.get_api_key("test");
    assert!(key.is_some());
    assert_eq!(key.unwrap(), "test_key");

    let missing = client.get_api_key("nonexistent");
    assert!(missing.is_none());
}

#[test]
fn test_web_client_get_config() {
    let mut client = WebClient::new().expect("Failed to create client");
    client.set_config("setting", "value");

    let config = client.get_config("setting");
    assert!(config.is_some());
    assert_eq!(config.unwrap(), "value");

    let missing = client.get_config("nonexistent");
    assert!(missing.is_none());
}

#[test]
fn test_web_client_clone() {
    let mut client = WebClient::new().expect("Failed to create client")
        .with_api_key("service", "key");
    client.set_config("option", "value");

    let cloned = client.clone();

    assert_eq!(cloned.api_keys.get("service"), Some(&"key".to_string()));
    assert_eq!(cloned.config.get("option"), Some(&"value".to_string()));
}

#[test]
fn test_web_client_debug() {
    let client = WebClient::new().expect("Failed to create client").with_api_key("test", "key");

    let debug_str = format!("{:?}", client);
    assert!(debug_str.contains("WebClient"));
    assert!(debug_str.contains("api_keys"));
}

#[test]
fn test_web_client_default() {
    let client = WebClient::default();

    assert!(client.api_keys.is_empty());
    assert!(client.config.is_empty());
}

#[test]
fn test_web_client_empty_strings() {
    let client = WebClient::new().expect("Failed to create client").with_api_key("", "");

    assert_eq!(client.api_keys.get(""), Some(&"".to_string()));
}

#[test]
fn test_web_client_special_characters() {
    let mut client = WebClient::new().expect("Failed to create client")
        .with_api_key("service@123", "key!@#$%");
    client.set_config("config-key", "value/with/slashes");

    assert_eq!(
        client.api_keys.get("service@123"),
        Some(&"key!@#$%".to_string())
    );
    assert_eq!(
        client.config.get("config-key"),
        Some(&"value/with/slashes".to_string())
    );
}

#[test]
fn test_web_client_multiple_services() {
    let client = WebClient::new().expect("Failed to create client")
        .with_twitter_token("twitter_token")
        .with_exa_key("exa_key")
        .with_dexscreener_key("dex_key")
        .with_api_key("custom", "custom_key");

    assert_eq!(client.api_keys.len(), 4);
    assert!(client.api_keys.contains_key("twitter"));
    assert!(client.api_keys.contains_key("exa"));
    assert!(client.api_keys.contains_key("dexscreener"));
    assert!(client.api_keys.contains_key("custom"));
}

#[test]
fn test_web_client_builder_pattern() {
    let mut client = WebClient::new().expect("Failed to create client");

    // Test that the builder pattern works correctly
    client = client.with_api_key("key1", "value1");
    client.set_config("config1", "value1");

    assert_eq!(client.api_keys.len(), 1);
    assert_eq!(client.config.len(), 1);
}

#[test]
fn test_web_client_http_client_exists() {
    let client = WebClient::new().expect("Failed to create client");

    // Just verify that http_client field exists and can be accessed
    let _ = &client.http_client;
    // If we get here without panicking, the field exists and test passes
}

#[test]
fn test_web_client_hashmap_operations() {
    let mut client = WebClient::new().expect("Failed to create client");

    // Direct HashMap operations
    client
        .api_keys
        .insert("direct".to_string(), "value".to_string());
    client
        .config
        .insert("direct_config".to_string(), "config_value".to_string());

    assert_eq!(client.api_keys.get("direct"), Some(&"value".to_string()));
    assert_eq!(
        client.config.get("direct_config"),
        Some(&"config_value".to_string())
    );
}
