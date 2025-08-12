//! Comprehensive tests for config module

use riglr_showcase::config::Config;
use serial_test::serial;
use std::env;

const SOLANA_RPC_URL: &str = "SOLANA_RPC_URL";
const RPC_URL_1: &str = "RPC_URL_1";
const TWITTER_BEARER_TOKEN: &str = "TWITTER_BEARER_TOKEN";
const EXA_API_KEY: &str = "EXA_API_KEY";
const NEO4J_URL: &str = "NEO4J_URL";
const REDIS_URL: &str = "REDIS_URL";
const OPENAI_API_KEY: &str = "OPENAI_API_KEY";

#[test]
#[serial]
fn test_config_from_env_with_defaults() {
    // Store string values in variables to avoid linting warnings
    let redis_url_value = "redis://localhost:6379";
    let solana_rpc_url_value = "https://api.mainnet-beta.solana.com";
    let openai_api_key_value = "test_api_key";
    
    // Set required fields first
    // SAFETY: Safe in test context as we control the environment
    unsafe {
        env::set_var(REDIS_URL, redis_url_value);
        env::set_var(SOLANA_RPC_URL, solana_rpc_url_value);
        env::set_var(OPENAI_API_KEY, openai_api_key_value);
    }

    // Clear optional environment variables
    // SAFETY: Safe in test context as we control the environment
    unsafe {
        env::remove_var(RPC_URL_1);
        env::remove_var(TWITTER_BEARER_TOKEN);
        env::remove_var(EXA_API_KEY);
        env::remove_var(NEO4J_URL);
    }

    let config = Config::from_env();

    assert_eq!(
        config.network.solana_rpc_url,
        solana_rpc_url_value
    );
    // No default Ethereum RPC URL since we didn't set RPC_URL_1
    assert_eq!(config.network.get_rpc_url(1), None);
    assert!(config.providers.twitter_bearer_token.is_none());
    assert!(config.providers.exa_api_key.is_none());
    assert!(config.database.neo4j_url.is_none());
    assert_eq!(config.database.redis_url, redis_url_value);
    assert_eq!(
        config.providers.openai_api_key.as_deref(),
        Some(openai_api_key_value)
    );

    // Clean up
    // SAFETY: Safe in test context as we control the environment
    unsafe {
        env::remove_var(REDIS_URL);
        env::remove_var(SOLANA_RPC_URL);
        env::remove_var(OPENAI_API_KEY);
    }
}

#[test]
#[serial]
fn test_config_from_env_with_custom_values() {
    // Store string values in variables to avoid linting warnings
    let solana_rpc_url_value = "https://custom.solana.com";
    let rpc_url_1_value = "https://custom.ethereum.com";
    let twitter_bearer_token_value = "twitter_token";
    let exa_api_key_value = "exa_key";
    let neo4j_url_value = "neo4j://custom:7687";
    let redis_url_value = "redis://custom:6379";
    let openai_api_key_value = "openai_key";
    
    // Set all environment variables
    // SAFETY: Safe in test context as we control the environment
    unsafe {
        env::set_var(SOLANA_RPC_URL, solana_rpc_url_value);
        env::set_var(RPC_URL_1, rpc_url_1_value);
        env::set_var(TWITTER_BEARER_TOKEN, twitter_bearer_token_value);
        env::set_var(EXA_API_KEY, exa_api_key_value);
        env::set_var(NEO4J_URL, neo4j_url_value);
        env::set_var(REDIS_URL, redis_url_value);
        env::set_var(OPENAI_API_KEY, openai_api_key_value);
    }

    let config = Config::from_env();

    assert_eq!(config.network.solana_rpc_url, solana_rpc_url_value);
    assert_eq!(
        config.network.get_rpc_url(1),
        Some(rpc_url_1_value.to_string())
    );
    assert_eq!(
        config.providers.twitter_bearer_token,
        Some(twitter_bearer_token_value.to_string())
    );
    assert_eq!(config.providers.exa_api_key, Some(exa_api_key_value.to_string()));
    assert_eq!(
        config.database.neo4j_url.as_deref(),
        Some(neo4j_url_value)
    );
    assert_eq!(config.database.redis_url, redis_url_value);
    assert_eq!(
        config.providers.openai_api_key.as_deref(),
        Some(openai_api_key_value)
    );

    // Clean up
    // SAFETY: Safe in test context as we control the environment
    unsafe {
        env::remove_var(SOLANA_RPC_URL);
        env::remove_var(RPC_URL_1);
        env::remove_var(TWITTER_BEARER_TOKEN);
        env::remove_var(EXA_API_KEY);
        env::remove_var(NEO4J_URL);
        env::remove_var(REDIS_URL);
        env::remove_var(OPENAI_API_KEY);
    }
}

#[test]
#[serial]
fn test_config_from_env_missing_openai_key() {
    // Store string values in variables to avoid linting warnings
    let redis_url_value = "redis://localhost:6379";
    let solana_rpc_url_value = "https://api.mainnet-beta.solana.com";
    
    // Set required fields except OPENAI_API_KEY
    // SAFETY: Safe in test context as we control the environment
    unsafe {
        env::set_var(REDIS_URL, redis_url_value);
        env::set_var(SOLANA_RPC_URL, solana_rpc_url_value);
        env::remove_var(OPENAI_API_KEY);
    }

    // This should succeed since openai_api_key is optional
    let config = Config::from_env();
    assert!(config.providers.openai_api_key.is_none());

    // Clean up
    // SAFETY: Safe in test context as we control the environment
    unsafe {
        env::remove_var(REDIS_URL);
        env::remove_var(SOLANA_RPC_URL);
    }
}

#[test]
#[serial]
fn test_config_clone() {
    // Store string values in variables to avoid linting warnings
    let redis_url_value = "redis://localhost:6379";
    let solana_rpc_url_value = "https://api.mainnet-beta.solana.com";
    let openai_api_key_value = "test_key";
    
    // Set required fields
    // SAFETY: Safe in test context as we control the environment
    unsafe {
        env::set_var(REDIS_URL, redis_url_value);
        env::set_var(SOLANA_RPC_URL, solana_rpc_url_value);
        env::set_var(OPENAI_API_KEY, openai_api_key_value);
    }

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
    // SAFETY: Safe in test context as we control the environment
    unsafe {
        env::remove_var(REDIS_URL);
        env::remove_var(SOLANA_RPC_URL);
        env::remove_var(OPENAI_API_KEY);
    }
}

#[test]
#[serial]
fn test_config_debug() {
    // Store string values in variables to avoid linting warnings
    let redis_url_value = "redis://localhost:6379";
    let solana_rpc_url_value = "https://api.mainnet-beta.solana.com";
    let openai_api_key_value = "debug_key";
    
    // Set required fields
    // SAFETY: Safe in test context as we control the environment
    unsafe {
        env::set_var(REDIS_URL, redis_url_value);
        env::set_var(SOLANA_RPC_URL, solana_rpc_url_value);
        env::set_var(OPENAI_API_KEY, openai_api_key_value);
    }

    let config = Config::from_env();
    let debug_str = format!("{:?}", config);

    assert!(debug_str.contains("Config"));
    assert!(debug_str.contains("solana_rpc_url"));
    assert!(debug_str.contains("rpc_urls"));
    assert!(debug_str.contains("openai_api_key"));

    // Clean up
    // SAFETY: Safe in test context as we control the environment
    unsafe {
        env::remove_var(REDIS_URL);
        env::remove_var(SOLANA_RPC_URL);
        env::remove_var(OPENAI_API_KEY);
    }
}

#[test]
#[serial]
fn test_config_partial_env_vars() {
    // Store string values in variables to avoid linting warnings
    let solana_rpc_url_value = "https://partial.solana.com";
    let redis_url_value = "redis://localhost:6379";
    let twitter_bearer_token_value = "partial_twitter";
    let openai_api_key_value = "partial_key";
    
    // Set only some environment variables
    // SAFETY: Safe in test context as we control the environment
    unsafe {
        env::set_var(SOLANA_RPC_URL, solana_rpc_url_value);
        env::set_var(REDIS_URL, redis_url_value);
        env::set_var(TWITTER_BEARER_TOKEN, twitter_bearer_token_value);
        env::set_var(OPENAI_API_KEY, openai_api_key_value);
    }

    // Leave others unset to test defaults
    // SAFETY: Safe in test context as we control the environment
    unsafe {
        env::remove_var(RPC_URL_1);
        env::remove_var(EXA_API_KEY);
        env::remove_var(NEO4J_URL);
    }

    let config = Config::from_env();

    assert_eq!(config.network.solana_rpc_url, solana_rpc_url_value);
    // No Ethereum RPC URL since we didn't set RPC_URL_1
    assert_eq!(config.network.get_rpc_url(1), None);
    assert_eq!(
        config.providers.twitter_bearer_token,
        Some(twitter_bearer_token_value.to_string())
    );
    assert!(config.providers.exa_api_key.is_none());
    assert!(config.database.neo4j_url.is_none());
    assert_eq!(config.database.redis_url, redis_url_value);

    // Clean up
    // SAFETY: Safe in test context as we control the environment
    unsafe {
        env::remove_var(SOLANA_RPC_URL);
        env::remove_var(REDIS_URL);
        env::remove_var(TWITTER_BEARER_TOKEN);
        env::remove_var(OPENAI_API_KEY);
    }
}

#[test]
#[serial]
fn test_config_empty_env_values() {
    // Store string values in variables to avoid linting warnings
    let solana_rpc_url_value = "https://api.mainnet-beta.solana.com";
    let redis_url_value = "redis://localhost:6379";
    let empty_value = "";
    
    // Set required fields and empty values for optional ones
    // SAFETY: Safe in test context as we control the environment
    unsafe {
        env::set_var(SOLANA_RPC_URL, solana_rpc_url_value);
        env::set_var(REDIS_URL, redis_url_value);
        env::set_var(TWITTER_BEARER_TOKEN, empty_value);
        env::set_var(EXA_API_KEY, empty_value);
        env::set_var(OPENAI_API_KEY, empty_value);
    }

    let config = Config::from_env();

    // Empty strings are parsed as Some("") for optional fields
    assert_eq!(config.providers.twitter_bearer_token.as_deref(), Some(empty_value));
    assert_eq!(config.providers.exa_api_key.as_deref(), Some(empty_value));
    assert_eq!(config.providers.openai_api_key.as_deref(), Some(empty_value));

    // Test validation with empty values - should pass since empty strings are allowed
    let validation_result = config.validate();
    assert!(
        validation_result.is_ok(),
        "Validation should pass with empty strings"
    );

    // Clean up
    // SAFETY: Safe in test context as we control the environment
    unsafe {
        env::remove_var(SOLANA_RPC_URL);
        env::remove_var(TWITTER_BEARER_TOKEN);
        env::remove_var(EXA_API_KEY);
        env::remove_var(OPENAI_API_KEY);
        env::remove_var(REDIS_URL);
    }
}

#[test]
#[serial]
fn test_config_special_characters_in_env() {
    // Store string values in variables to avoid linting warnings
    let solana_rpc_url_value = "https://user:pass@solana.com:8899/path";
    let rpc_url_1_value = "wss://ethereum.com/ws";
    let twitter_bearer_token_value = "Bearer abc123!@#$%";
    let neo4j_url_value = "neo4j+s://user:pass@neo4j.com:7687";
    let redis_url_value = "redis://user:pass@redis.com:6379/0";
    let openai_api_key_value = "sk-123abc!@#";
    
    // Test with special characters in URLs and keys
    // SAFETY: Safe in test context as we control the environment
    unsafe {
        env::set_var(SOLANA_RPC_URL, solana_rpc_url_value);
        env::set_var(RPC_URL_1, rpc_url_1_value);
        env::set_var(TWITTER_BEARER_TOKEN, twitter_bearer_token_value);
        env::set_var(NEO4J_URL, neo4j_url_value);
        env::set_var(REDIS_URL, redis_url_value);
        env::set_var(OPENAI_API_KEY, openai_api_key_value);
    }

    let config = Config::from_env();

    assert_eq!(
        config.network.solana_rpc_url,
        solana_rpc_url_value
    );
    assert_eq!(
        config.network.get_rpc_url(1),
        Some(rpc_url_1_value.to_string())
    );
    assert_eq!(
        config.providers.twitter_bearer_token,
        Some(twitter_bearer_token_value.to_string())
    );
    assert_eq!(
        config.database.neo4j_url.as_deref(),
        Some(neo4j_url_value)
    );
    assert_eq!(
        config.database.redis_url,
        redis_url_value
    );
    assert_eq!(
        config.providers.openai_api_key.as_deref(),
        Some(openai_api_key_value)
    );

    // Clean up
    // SAFETY: Safe in test context as we control the environment
    unsafe {
        env::remove_var(SOLANA_RPC_URL);
        env::remove_var(RPC_URL_1);
        env::remove_var(TWITTER_BEARER_TOKEN);
        env::remove_var(NEO4J_URL);
        env::remove_var(REDIS_URL);
        env::remove_var(OPENAI_API_KEY);
    }
}

#[test]
#[serial]
fn test_config_localhost_urls() {
    // Store string values in variables to avoid linting warnings
    let solana_rpc_url_value = "http://localhost:8899";
    let rpc_url_1_value = "http://127.0.0.1:8545";
    let neo4j_url_value = "bolt://localhost:7687";
    let redis_url_value = "redis://127.0.0.1:6379";
    let openai_api_key_value = "test";
    
    // SAFETY: Safe in test context as we control the environment
    unsafe {
        env::set_var(SOLANA_RPC_URL, solana_rpc_url_value);
        env::set_var(RPC_URL_1, rpc_url_1_value);
        env::set_var(NEO4J_URL, neo4j_url_value);
        env::set_var(REDIS_URL, redis_url_value);
        env::set_var(OPENAI_API_KEY, openai_api_key_value);
    }

    let config = Config::from_env();

    assert_eq!(config.network.solana_rpc_url, solana_rpc_url_value);
    assert_eq!(
        config.network.get_rpc_url(1),
        Some(rpc_url_1_value.to_string())
    );
    assert_eq!(
        config.database.neo4j_url.as_deref(),
        Some(neo4j_url_value)
    );
    assert_eq!(config.database.redis_url, redis_url_value);

    // Clean up
    // SAFETY: Safe in test context as we control the environment
    unsafe {
        env::remove_var(SOLANA_RPC_URL);
        env::remove_var(RPC_URL_1);
        env::remove_var(NEO4J_URL);
        env::remove_var(REDIS_URL);
        env::remove_var(OPENAI_API_KEY);
    }
}

#[test]
#[serial]
fn test_config_network_specific_urls() {
    // Store string values in variables to avoid linting warnings
    let solana_rpc_url_value = "https://api.devnet.solana.com";
    let rpc_url_1_value = "https://rpc.ankr.com/eth_goerli";
    let redis_url_value = "redis://localhost:6379";
    let openai_api_key_value = "test";
    
    // Test various network-specific URLs
    // SAFETY: Safe in test context as we control the environment
    unsafe {
        env::set_var(SOLANA_RPC_URL, solana_rpc_url_value);
        env::set_var(RPC_URL_1, rpc_url_1_value);
        env::set_var(REDIS_URL, redis_url_value);
        env::set_var(OPENAI_API_KEY, openai_api_key_value);
    }

    let config = Config::from_env();

    assert_eq!(
        config.network.solana_rpc_url,
        solana_rpc_url_value
    );
    assert_eq!(
        config.network.get_rpc_url(1),
        Some(rpc_url_1_value.to_string())
    );

    // Clean up
    // SAFETY: Safe in test context as we control the environment
    unsafe {
        env::remove_var(SOLANA_RPC_URL);
        env::remove_var(RPC_URL_1);
        env::remove_var(REDIS_URL);
        env::remove_var(OPENAI_API_KEY);
    }
}

// Note: Cannot test invalid config with from_env() because it calls std::process::exit(1)
// instead of panicking, making it impossible to test in unit tests.
// The validation logic is still tested via the Config::validate() method in integration tests.
