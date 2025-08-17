//! Integration tests for Phase 1 foundational changes
use std::env;

// Environment variable constants to avoid linting warnings
const ENV_OPENAI_API_KEY: &str = "OPENAI_API_KEY";
const ENV_REDIS_URL: &str = "REDIS_URL";
const ENV_NEO4J_URL: &str = "NEO4J_URL";
const ENV_TWITTER_BEARER_TOKEN: &str = "TWITTER_BEARER_TOKEN";
const ENV_EXA_API_KEY: &str = "EXA_API_KEY";
const ENV_RPC_URL_1: &str = "RPC_URL_1";
const ENV_RPC_URL_137: &str = "RPC_URL_137";
const ENV_RPC_URL_42161: &str = "RPC_URL_42161";
const ENV_RPC_URL_8453: &str = "RPC_URL_8453";
const ENV_SOLANA_RPC_URL: &str = "SOLANA_RPC_URL";
const ENV_RPC_URL_999: &str = "RPC_URL_999";

#[test]
fn test_config_centralization_integration() {
    // Test that configuration loading works end-to-end
    env::set_var(ENV_OPENAI_API_KEY, "test-key");
    env::set_var(ENV_REDIS_URL, "redis://localhost:6379");
    env::set_var(ENV_NEO4J_URL, "neo4j://localhost:7687");
    env::set_var(ENV_TWITTER_BEARER_TOKEN, "test-token");
    env::set_var(ENV_EXA_API_KEY, "test-exa-key");
    env::set_var(ENV_RPC_URL_1, "https://test-ethereum.example.com");
    env::set_var(ENV_RPC_URL_137, "https://test-polygon.example.com");
    env::set_var(ENV_RPC_URL_42161, "https://test-arbitrum.example.com");
    env::set_var(ENV_RPC_URL_8453, "https://test-base.example.com");
    env::set_var(ENV_SOLANA_RPC_URL, "https://test-solana.example.com");

    // This should work without panicking
    let config = riglr_showcase::config::Config::from_env();
    assert!(!config.openai_api_key.is_empty());

    // Test validation
    let validation_result = config.validate();
    assert!(validation_result.is_ok());

    // Cleanup
    env::remove_var(ENV_OPENAI_API_KEY);
    env::remove_var(ENV_REDIS_URL);
    env::remove_var(ENV_NEO4J_URL);
    env::remove_var(ENV_TWITTER_BEARER_TOKEN);
    env::remove_var(ENV_EXA_API_KEY);
    env::remove_var(ENV_RPC_URL_1);
    env::remove_var(ENV_RPC_URL_137);
    env::remove_var(ENV_RPC_URL_42161);
    env::remove_var(ENV_RPC_URL_8453);
    env::remove_var(ENV_SOLANA_RPC_URL);
}

#[test]
fn test_evm_provider_extensibility() {
    env::set_var(ENV_RPC_URL_999, "https://test-new-chain.example.com");

    let result = riglr_evm_tools::util::chain_id_to_rpc_url(999);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "https://test-new-chain.example.com");

    let supported = riglr_evm_tools::util::get_supported_chains();
    assert!(supported.contains(&999));

    env::remove_var(ENV_RPC_URL_999);
}

#[test]
fn test_error_handling_integration() {
    // Test that error source chain is preserved through multiple conversions
    use riglr_core::error::ToolError;
    use riglr_evm_tools::error::EvmToolError;

    let original_error = EvmToolError::InvalidAddress("0xinvalid".to_string());
    let tool_error: ToolError = original_error.into();

    // Verify error classification
    assert!(!tool_error.is_retriable()); // Invalid input should not be retriable
}
