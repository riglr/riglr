//! Comprehensive tests for config module

use riglr_showcase::config::Config;
use std::env;

#[test]
fn test_config_from_env_with_defaults() {
    // Clear environment variables
    env::remove_var("SOLANA_RPC_URL");
    env::remove_var("RPC_URL_1");
    env::remove_var("TWITTER_BEARER_TOKEN");
    env::remove_var("EXA_API_KEY");
    env::remove_var("NEO4J_URL");
    env::remove_var("REDIS_URL");

    // Set required fields
    env::set_var("OPENAI_API_KEY", "test_api_key");
    env::set_var("REDIS_URL", "redis://localhost:6379");
    env::set_var("SOLANA_RPC_URL", "https://api.mainnet-beta.solana.com");

    let config = Config::from_env();

    assert_eq!(
        config.network.solana_rpc_url,
        "https://api.mainnet-beta.solana.com"
    );
    // No default Ethereum RPC URL since we didn't set RPC_URL_1
    assert_eq!(config.network.get_rpc_url(1), None);
    assert!(config.providers.twitter_bearer_token.is_none());
    assert!(config.providers.exa_api_key.is_none());
    assert!(config.database.neo4j_url.is_none());
    assert_eq!(config.database.redis_url, "redis://localhost:6379");
    assert_eq!(
        config.providers.openai_api_key.as_deref(),
        Some("test_api_key")
    );

    // Clean up
    env::remove_var("OPENAI_API_KEY");
    env::remove_var("REDIS_URL");
    env::remove_var("SOLANA_RPC_URL");
}

#[test]
fn test_config_from_env_with_custom_values() {
    // Set all environment variables
    env::set_var("SOLANA_RPC_URL", "https://custom.solana.com");
    env::set_var("RPC_URL_1", "https://custom.ethereum.com");
    env::set_var("TWITTER_BEARER_TOKEN", "twitter_token");
    env::set_var("EXA_API_KEY", "exa_key");
    env::set_var("NEO4J_URL", "neo4j://custom:7687");
    env::set_var("REDIS_URL", "redis://custom:6379");
    env::set_var("OPENAI_API_KEY", "openai_key");

    let config = Config::from_env();

    assert_eq!(config.network.solana_rpc_url, "https://custom.solana.com");
    assert_eq!(
        config.network.get_rpc_url(1),
        Some("https://custom.ethereum.com".to_string())
    );
    assert_eq!(
        config.providers.twitter_bearer_token,
        Some("twitter_token".to_string())
    );
    assert_eq!(config.providers.exa_api_key, Some("exa_key".to_string()));
    assert_eq!(
        config.database.neo4j_url.as_deref(),
        Some("neo4j://custom:7687")
    );
    assert_eq!(config.database.redis_url, "redis://custom:6379");
    assert_eq!(
        config.providers.openai_api_key.as_deref(),
        Some("openai_key")
    );

    // Clean up
    env::remove_var("SOLANA_RPC_URL");
    env::remove_var("RPC_URL_1");
    env::remove_var("TWITTER_BEARER_TOKEN");
    env::remove_var("EXA_API_KEY");
    env::remove_var("NEO4J_URL");
    env::remove_var("REDIS_URL");
    env::remove_var("OPENAI_API_KEY");
}

#[test]
fn test_config_from_env_missing_openai_key() {
    // Set required fields except OPENAI_API_KEY
    env::set_var("REDIS_URL", "redis://localhost:6379");
    env::set_var("SOLANA_RPC_URL", "https://api.mainnet-beta.solana.com");

    // Clear OPENAI_API_KEY
    env::remove_var("OPENAI_API_KEY");

    // This should succeed since openai_api_key is optional
    let config = Config::from_env();
    assert!(config.providers.openai_api_key.is_none());

    // Clean up
    env::remove_var("REDIS_URL");
    env::remove_var("SOLANA_RPC_URL");
}

#[test]
fn test_config_clone() {
    // Set required fields
    env::set_var("REDIS_URL", "redis://localhost:6379");
    env::set_var("SOLANA_RPC_URL", "https://api.mainnet-beta.solana.com");
    env::set_var("OPENAI_API_KEY", "test_key");

    let config = Config::from_env();
    let cloned = config.clone();

    assert_eq!(cloned.network.solana_rpc_url, config.network.solana_rpc_url);
    // assert_eq!(cloned.network.rpc_urls, config.network.rpc_urls);
    assert_eq!(
        cloned.providers.twitter_bearer_token,
        config.providers.twitter_bearer_token
    );
    assert_eq!(cloned.providers.exa_api_key, config.providers.exa_api_key);
    assert_eq!(cloned.database.neo4j_url, config.database.neo4j_url);
    assert_eq!(cloned.database.redis_url, config.database.redis_url);
    assert_eq!(
        cloned.providers.openai_api_key,
        config.providers.openai_api_key
    );

    // Clean up
    env::remove_var("REDIS_URL");
    env::remove_var("SOLANA_RPC_URL");
    env::remove_var("OPENAI_API_KEY");
}

#[test]
fn test_config_debug() {
    // Set required fields
    env::set_var("REDIS_URL", "redis://localhost:6379");
    env::set_var("SOLANA_RPC_URL", "https://api.mainnet-beta.solana.com");
    env::set_var("OPENAI_API_KEY", "debug_key");

    let config = Config::from_env();
    let debug_str = format!("{:?}", config);

    assert!(debug_str.contains("Config"));
    assert!(debug_str.contains("solana_rpc_url"));
    assert!(debug_str.contains("rpc_urls"));
    assert!(debug_str.contains("openai_api_key"));

    // Clean up
    env::remove_var("REDIS_URL");
    env::remove_var("SOLANA_RPC_URL");
    env::remove_var("OPENAI_API_KEY");
}

#[test]
fn test_config_partial_env_vars() {
    // Set only some environment variables
    env::set_var("SOLANA_RPC_URL", "https://partial.solana.com");
    env::set_var("REDIS_URL", "redis://localhost:6379");
    env::set_var("TWITTER_BEARER_TOKEN", "partial_twitter");
    env::set_var("OPENAI_API_KEY", "partial_key");

    // Leave others unset to test defaults
    env::remove_var("RPC_URL_1");
    env::remove_var("EXA_API_KEY");
    env::remove_var("NEO4J_URL");

    let config = Config::from_env();

    assert_eq!(config.network.solana_rpc_url, "https://partial.solana.com");
    // No Ethereum RPC URL since we didn't set RPC_URL_1
    assert_eq!(config.network.get_rpc_url(1), None);
    assert_eq!(
        config.providers.twitter_bearer_token,
        Some("partial_twitter".to_string())
    );
    assert!(config.providers.exa_api_key.is_none());
    assert!(config.database.neo4j_url.is_none());
    assert_eq!(config.database.redis_url, "redis://localhost:6379");

    // Clean up
    env::remove_var("SOLANA_RPC_URL");
    env::remove_var("REDIS_URL");
    env::remove_var("TWITTER_BEARER_TOKEN");
    env::remove_var("OPENAI_API_KEY");
}

#[test]
fn test_config_empty_env_values() {
    // Set required fields and empty values for optional ones
    env::set_var("SOLANA_RPC_URL", "https://api.mainnet-beta.solana.com");
    env::set_var("REDIS_URL", "redis://localhost:6379");
    env::set_var("TWITTER_BEARER_TOKEN", "");
    env::set_var("EXA_API_KEY", "");
    env::set_var("OPENAI_API_KEY", "");

    let config = Config::from_env();

    // Empty strings are parsed as Some("") for optional fields
    assert_eq!(config.providers.twitter_bearer_token.as_deref(), Some(""));
    assert_eq!(config.providers.exa_api_key.as_deref(), Some(""));
    assert_eq!(config.providers.openai_api_key.as_deref(), Some(""));

    // Test validation with empty values - should pass since empty strings are allowed
    let validation_result = config.validate();
    assert!(
        validation_result.is_ok(),
        "Validation should pass with empty strings"
    );

    // Clean up
    env::remove_var("SOLANA_RPC_URL");
    env::remove_var("TWITTER_BEARER_TOKEN");
    env::remove_var("EXA_API_KEY");
    env::remove_var("OPENAI_API_KEY");
    env::remove_var("REDIS_URL");
}

#[test]
fn test_config_special_characters_in_env() {
    // Test with special characters in URLs and keys
    env::set_var("SOLANA_RPC_URL", "https://user:pass@solana.com:8899/path");
    env::set_var("RPC_URL_1", "wss://ethereum.com/ws");
    env::set_var("TWITTER_BEARER_TOKEN", "Bearer abc123!@#$%");
    env::set_var("NEO4J_URL", "neo4j+s://user:pass@neo4j.com:7687");
    env::set_var("REDIS_URL", "redis://user:pass@redis.com:6379/0");
    env::set_var("OPENAI_API_KEY", "sk-123abc!@#");

    let config = Config::from_env();

    assert_eq!(
        config.network.solana_rpc_url,
        "https://user:pass@solana.com:8899/path"
    );
    assert_eq!(
        config.network.get_rpc_url(1),
        Some("wss://ethereum.com/ws".to_string())
    );
    assert_eq!(
        config.providers.twitter_bearer_token,
        Some("Bearer abc123!@#$%".to_string())
    );
    assert_eq!(
        config.database.neo4j_url.as_deref(),
        Some("neo4j+s://user:pass@neo4j.com:7687")
    );
    assert_eq!(
        config.database.redis_url,
        "redis://user:pass@redis.com:6379/0"
    );
    assert_eq!(
        config.providers.openai_api_key.as_deref(),
        Some("sk-123abc!@#")
    );

    // Clean up
    env::remove_var("SOLANA_RPC_URL");
    env::remove_var("RPC_URL_1");
    env::remove_var("TWITTER_BEARER_TOKEN");
    env::remove_var("NEO4J_URL");
    env::remove_var("REDIS_URL");
    env::remove_var("OPENAI_API_KEY");
}

#[test]
fn test_config_localhost_urls() {
    env::set_var("SOLANA_RPC_URL", "http://localhost:8899");
    env::set_var("RPC_URL_1", "http://127.0.0.1:8545");
    env::set_var("NEO4J_URL", "bolt://localhost:7687");
    env::set_var("REDIS_URL", "redis://127.0.0.1:6379");
    env::set_var("OPENAI_API_KEY", "test");

    let config = Config::from_env();

    assert_eq!(config.network.solana_rpc_url, "http://localhost:8899");
    assert_eq!(
        config.network.get_rpc_url(1),
        Some("http://127.0.0.1:8545".to_string())
    );
    assert_eq!(
        config.database.neo4j_url.as_deref(),
        Some("bolt://localhost:7687")
    );
    assert_eq!(config.database.redis_url, "redis://127.0.0.1:6379");

    // Clean up
    env::remove_var("SOLANA_RPC_URL");
    env::remove_var("RPC_URL_1");
    env::remove_var("NEO4J_URL");
    env::remove_var("REDIS_URL");
    env::remove_var("OPENAI_API_KEY");
}

#[test]
fn test_config_network_specific_urls() {
    // Test various network-specific URLs
    env::set_var("SOLANA_RPC_URL", "https://api.devnet.solana.com");
    env::set_var("RPC_URL_1", "https://rpc.ankr.com/eth_goerli");
    env::set_var("REDIS_URL", "redis://localhost:6379");
    env::set_var("OPENAI_API_KEY", "test");

    let config = Config::from_env();

    assert_eq!(
        config.network.solana_rpc_url,
        "https://api.devnet.solana.com"
    );
    assert_eq!(
        config.network.get_rpc_url(1),
        Some("https://rpc.ankr.com/eth_goerli".to_string())
    );

    // Clean up
    env::remove_var("SOLANA_RPC_URL");
    env::remove_var("RPC_URL_1");
    env::remove_var("REDIS_URL");
    env::remove_var("OPENAI_API_KEY");
}

// Note: Cannot test invalid config with from_env() because it calls std::process::exit(1)
// instead of panicking, making it impossible to test in unit tests.
// The validation logic is still tested via the Config::validate() method in integration tests.
