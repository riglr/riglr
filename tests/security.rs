//! Security-focused tests for production deployment
use std::env;

// Environment variable constants to avoid string literals
const OPENAI_API_KEY: &str = "OPENAI_API_KEY";
const REDIS_URL: &str = "REDIS_URL";
const NEO4J_URL: &str = "NEO4J_URL";
const TWITTER_BEARER_TOKEN: &str = "TWITTER_BEARER_TOKEN";
const EXA_API_KEY: &str = "EXA_API_KEY";
const RPC_URL_1: &str = "RPC_URL_1";
const RPC_URL_137: &str = "RPC_URL_137";
const RPC_URL_42161: &str = "RPC_URL_42161";
const RPC_URL_8453: &str = "RPC_URL_8453";
const SOLANA_RPC_URL: &str = "SOLANA_RPC_URL";
const RPC_URL_999: &str = "RPC_URL_999";

#[test]
fn test_no_hardcoded_secrets() {
    // Ensure no API keys or secrets are hardcoded
    use std::process::Command;

    let patterns = [
        "sk-[a-zA-Z0-9]+", // OpenAI-style API keys
        "[a-f0-9]{32,64}", // Hex keys (but allow short hashes)
        "-----BEGIN.*PRIVATE KEY-----", // Private keys
        "xoxb-[0-9]+-[0-9]+-[0-9]+-[a-f0-9]+", // Slack tokens
    ];

    for pattern in &patterns {
        let output = Command::new("rg")
            .args(&[
                pattern,
                "--type", "rust",
                "--glob", "!.env.example",
                "--glob", "!tests/*",
                "--glob", "!*.md",
                "."
            ])
            .current_dir("/mnt/storage/projects/riglr")
            .output()
            .expect("Failed to run security scan");

        let results = String::from_utf8_lossy(&output.stdout);

        // Filter out acceptable patterns (test keys, documentation examples)
        let concerning_results: Vec<&str> = results
            .lines()
            .filter(|line| {
                !line.contains("test-key") &&
                !line.contains("your-key") &&
                !line.contains("{{") &&
                !line.contains("example") &&
                !line.contains("placeholder")
            })
            .collect();

        assert!(concerning_results.is_empty(),
               "Found potential hardcoded secrets with pattern {}: {:#?}",
               pattern, concerning_results);
    }
}

#[test]
fn test_configuration_validation() {
    // Test that configuration validation catches common issues

    // Test empty API key validation
    env::set_var(OPENAI_API_KEY, "");
    env::set_var(REDIS_URL, "redis://localhost:6379");
    env::set_var(NEO4J_URL, "neo4j://localhost:7687");
    env::set_var(TWITTER_BEARER_TOKEN, "test-key");
    env::set_var(EXA_API_KEY, "test-key");
    env::set_var(RPC_URL_1, "https://test.example.com");
    env::set_var(RPC_URL_137, "https://test.example.com");
    env::set_var(RPC_URL_42161, "https://test.example.com");
    env::set_var(RPC_URL_8453, "https://test.example.com");
    env::set_var(SOLANA_RPC_URL, "https://test.example.com");

    let config = riglr_showcase::config::Config::from_env();
    let validation_result = config.validate();

    assert!(validation_result.is_err(), "Should reject empty API key");

    // Test invalid URL formats
    env::set_var(REDIS_URL, "invalid-url");
    let config = riglr_showcase::config::Config::from_env();
    let validation_result = config.validate();
    assert!(validation_result.is_err(), "Should reject invalid Redis URL format");

    // Clean up
    env::remove_var(OPENAI_API_KEY);
    env::remove_var(REDIS_URL);
    env::remove_var(NEO4J_URL);
    env::remove_var(TWITTER_BEARER_TOKEN);
    env::remove_var(EXA_API_KEY);
    env::remove_var(RPC_URL_1);
    env::remove_var(RPC_URL_137);
    env::remove_var(RPC_URL_42161);
    env::remove_var(RPC_URL_8453);
    env::remove_var(SOLANA_RPC_URL);
}

#[tokio::test]
async fn test_error_information_disclosure() {
    // Test that errors don't leak sensitive information
    use riglr_core::error::ToolError;

    let sensitive_data = "sk-1234567890abcdef"; // Mock API key
    let error_with_sensitive = std::io::Error::new(
        std::io::ErrorKind::Other,
        format!("Failed to authenticate with key: {}", sensitive_data)
    );

    let tool_error = ToolError::permanent_string(error_with_sensitive, "Authentication failed".to_string());
    let error_display = format!("{}", tool_error);

    // Error should not leak the sensitive data in the display
    assert!(!error_display.contains("sk-1234567890abcdef"),
           "Error should not leak sensitive information in display: {}", error_display);

    // But source should be available for debugging (in secure environments)
    assert!(tool_error.source().is_some());
}

#[test]
fn test_safe_error_propagation() {
    // Test that error conversions preserve safety
    use riglr_core::error::ToolError;
    use riglr_evm_tools::error::EvmToolError;

    // Test that we don't lose error classification through conversions
    let network_error = EvmToolError::ProviderError("Network timeout".to_string());
    let tool_error: ToolError = network_error.into();

    // Network errors should be retriable
    assert!(tool_error.is_retriable(), "Network errors should be retriable");

    // Test that permanent errors stay permanent
    let balance_error = EvmToolError::InsufficientBalance;
    let tool_error: ToolError = balance_error.into();

    // Balance errors should not be retriable
    assert!(!tool_error.is_retriable(), "Insufficient balance should not be retriable");
}

#[test]
fn test_rpc_url_validation() {
    // Test that RPC URL format validation works
    use riglr_evm_tools::util::chain_id_to_rpc_url;

    // Test invalid URL format
    env::set_var(RPC_URL_999, "invalid-url");
    let result = chain_id_to_rpc_url(999);
    assert!(result.is_err(), "Should reject invalid URL format");

    // Test valid URL format
    env::set_var(RPC_URL_999, "https://valid-url.example.com");
    let result = chain_id_to_rpc_url(999);
    assert!(result.is_ok(), "Should accept valid URL format");

    // Test empty URL
    env::set_var(RPC_URL_999, "");
    let result = chain_id_to_rpc_url(999);
    assert!(result.is_err(), "Should reject empty URL");

    // Clean up
    env::remove_var(RPC_URL_999);
}

#[test]
fn test_no_debug_info_leaks() {
    // Ensure debug information doesn't leak in production builds
    use std::process::Command;

    let output = Command::new("rg")
        .args(&[
            "dbg!|println!",
            "--type", "rust",
            "--glob", "!tests/*",
            "--glob", "!examples/*",
            "src/"
        ])
        .current_dir("/mnt/storage/projects/riglr")
        .output()
        .expect("Failed to run debug leak scan");

    let results = String::from_utf8_lossy(&output.stdout);

    // Filter out acceptable debug usage (in development utilities, etc.)
    let debug_leaks: Vec<&str> = results
        .lines()
        .filter(|line| {
            !line.contains("#[cfg(test)]") &&
            !line.contains("// Debug:") &&
            !line.contains("debug_assert!")
        })
        .collect();

    assert!(debug_leaks.is_empty(),
           "Found debug information leaks in production code: {:#?}", debug_leaks);
}

#[test]
fn test_unsafe_code_usage() {
    // Ensure unsafe code is properly justified and minimal
    use std::process::Command;

    let output = Command::new("rg")
        .args(&[
            "unsafe",
            "--type", "rust",
            "--glob", "!tests/*",
            "src/"
        ])
        .current_dir("/mnt/storage/projects/riglr")
        .output()
        .expect("Failed to run unsafe code scan");

    let results = String::from_utf8_lossy(&output.stdout);

    // Each unsafe usage should be documented with a safety comment
    for line in results.lines() {
        if line.contains("unsafe") && !line.contains("// Safety:") {
            panic!("Unsafe code usage without safety documentation: {}", line);
        }
    }
}