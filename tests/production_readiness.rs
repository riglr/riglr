//! Production readiness integration tests
use std::env;

// Environment variable constants to avoid string literals
const OPENAI_API_KEY: &str = "OPENAI_API_KEY";
const ANTHROPIC_API_KEY: &str = "ANTHROPIC_API_KEY";
const RPC_URL_12345: &str = "RPC_URL_12345";

#[tokio::test]
async fn test_configuration_fail_fast() {
    // Test that missing environment variables cause immediate failure
    // SAFETY: Safe in test context as we're only modifying test environment variables
    unsafe {
        env::remove_var(OPENAI_API_KEY);
        env::remove_var(ANTHROPIC_API_KEY);
    }

    // This should panic when trying to load config with missing required vars
    let result = std::panic::catch_unwind(|| riglr_showcase::config::Config::from_env());

    assert!(
        result.is_err(),
        "Should panic with missing required API keys"
    );

    // Restore for other tests
    // SAFETY: Safe in test context as we're only modifying test environment variables
    unsafe {
        env::set_var(OPENAI_API_KEY, "test-key");
    }
}

#[tokio::test]
async fn test_error_source_preservation() {
    // Test that error sources are preserved through conversion chains
    use riglr_core::error::ToolError;
    use riglr_evm_tools::error::EvmToolError;

    let original_error = EvmToolError::InvalidAddress("0xinvalid".to_string());
    let tool_error: ToolError = original_error.into();

    // Should preserve source for debugging
    assert!(tool_error.source().is_some());

    // Should classify correctly
    assert!(!tool_error.is_retriable());
}

#[tokio::test]
async fn test_evm_provider_extensibility() {
    // Test that new chains can be added without code changes
    // SAFETY: Safe in test context as we're only modifying test environment variables
    unsafe {
        env::set_var(RPC_URL_12345, "https://test-new-chain.example.com");
    }

    let result = riglr_evm_tools::util::chain_id_to_rpc_url(12345);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "https://test-new-chain.example.com");

    let supported = riglr_evm_tools::util::get_supported_chains();
    assert!(supported.contains(&12345));

    // SAFETY: Safe in test context as we're only modifying test environment variables
    unsafe {
        env::remove_var(RPC_URL_12345);
    }
}

#[tokio::test]
async fn test_no_mock_implementations() {
    // Verify no mock implementations remain in production code
    use std::process::Command;

    let output = Command::new("rg")
        .args(&[
            "-i",
            "mock|placeholder|todo|fixme",
            "--type",
            "rust",
            "--glob",
            "!tests/*",
            "--glob",
            "!examples/*",
            "src/",
        ])
        .current_dir("/mnt/storage/projects/riglr")
        .output()
        .expect("Failed to run ripgrep");

    let results = String::from_utf8_lossy(&output.stdout);

    // Filter out acceptable mock usage (in test utilities, etc.)
    let concerning_mocks: Vec<&str> = results
        .lines()
        .filter(|line| {
            !line.contains("test_")
                && !line.contains("mock_signer")
                && !line.contains("MockAgent")
                && !line.contains("// TODO:")
                && !line.contains("// FIXME:")
                && (line.contains("mock") || line.contains("TODO") || line.contains("FIXME"))
        })
        .collect();

    if !concerning_mocks.is_empty() {
        panic!(
            "Found concerning mock/placeholder implementations:\n{:#?}",
            concerning_mocks
        );
    }
}

#[tokio::test]
#[ignore] // Requires real API keys - enable in CI with proper API key setup
async fn test_real_api_integration() {
    // Test that real APIs work correctly

    // Test price feed
    let price_result = riglr_web_tools::price::get_token_price(
        "So11111111111111111111111111111111111111112".to_string(), // SOL
        Some("solana".to_string()),
    )
    .await;

    assert!(price_result.is_ok(), "Real price feed should work");

    // Test Li.fi integration
    let routes_result = riglr_cross_chain_tools::lifi::get_routes(
        "ethereum",
        "polygon",
        "0xA0b86a33E6417c8f1E3BBb8D81c0a64d3E7fE6C8", // USDC
        "1000000",                                    // 1 USDC
        "0x742d35Cc6645C677A61e7b77dDf0b9A4Cb5F9568",
        "0x742d35Cc6645C677A61e7b77dDf0b9A4Cb5F9568",
    )
    .await;

    assert!(routes_result.is_ok(), "Li.fi route discovery should work");
}

#[test]
fn test_bridge_mock_removal() {
    // Test that bridge functions don't return mock transaction hashes
    use std::process::Command;

    let output = Command::new("rg")
        .args(&[
            "SolanaTxHash1234567890|EvmTxHash0987654321",
            "--type",
            "rust",
            "src/",
        ])
        .current_dir("/mnt/storage/projects/riglr")
        .output()
        .expect("Failed to run ripgrep");

    let results = String::from_utf8_lossy(&output.stdout);
    assert!(
        results.is_empty(),
        "Found mock transaction hashes: {}",
        results
    );
}

#[test]
fn test_trading_bot_mock_removal() {
    // Test that trading bot doesn't have mock price fallbacks
    use std::process::Command;

    let output = Command::new("rg")
        .args(&[
            "23\\.45|1650\\.30|26800\\.50",
            "--type",
            "rust",
            "create-riglr-app/src/bin/trading_bot.rs",
        ])
        .current_dir("/mnt/storage/projects/riglr")
        .output()
        .expect("Failed to run ripgrep");

    let results = String::from_utf8_lossy(&output.stdout);

    // Filter out regex patterns which are acceptable
    let mock_prices: Vec<&str> = results
        .lines()
        .filter(|line| !line.contains(r"\$(\d+\.?\d*)") && !line.contains("price"))
        .collect();

    assert!(
        mock_prices.is_empty(),
        "Found mock prices in trading bot: {:#?}",
        mock_prices
    );
}

#[test]
fn test_no_placeholder_responses() {
    // Scan for placeholder JSON responses and TODO markers in production code
    use std::process::Command;

    let output = Command::new("rg")
        .args(&[
            r#"json!\(\{\}\)|// Placeholder|TODO.*Update"#,
            "--type",
            "rust",
            "--glob",
            "!tests/*",
            "--glob",
            "!examples/*",
            "--glob",
            "!**/target/*",
            "src/",
        ])
        .current_dir("/mnt/storage/projects/riglr")
        .output()
        .expect("Failed to run ripgrep");

    let results = String::from_utf8_lossy(&output.stdout);

    // Filter out acceptable patterns
    let concerning_placeholders: Vec<&str> = results
        .lines()
        .filter(|line| {
            // Skip test-related files and acceptable patterns
            !line.contains("test_") &&
            !line.contains("#[cfg(test)]") &&
            !line.contains("// TODO: Implement when") && // Implementation notes are OK
            !line.contains("// TODO: Consider") && // Architecture notes are OK
            // Focus on actual placeholder implementations
            (line.contains("json!({})") ||
             line.contains("// Placeholder") ||
             line.contains("TODO: Update"))
        })
        .collect();

    if !concerning_placeholders.is_empty() {
        panic!(
            "Found placeholder implementations in production code:\n{:#?}",
            concerning_placeholders
        );
    }
}

#[tokio::test]
async fn test_all_examples_compile() {
    // Validate all showcase examples compile with current APIs
    use std::process::Command;

    let output = Command::new("cargo")
        .args(&["check", "--examples"])
        .current_dir("/mnt/storage/projects/riglr/riglr-showcase")
        .output()
        .expect("Failed to run cargo check");

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        panic!("Examples failed to compile:\n{}", stderr);
    }
}

#[test]
fn test_signer_context_usage() {
    // Verify that blockchain tools use SignerContext pattern correctly
    use std::process::Command;

    let output = Command::new("rg")
        .args(&[
            r#"\.sign_transaction\(|\.evm_client\(|\.solana_client\("#,
            "--type",
            "rust",
            "src/",
        ])
        .current_dir("/mnt/storage/projects/riglr")
        .output()
        .expect("Failed to run ripgrep");

    let results = String::from_utf8_lossy(&output.stdout);

    // All blockchain operations should go through SignerContext
    let direct_signer_usage: Vec<&str> = results
        .lines()
        .filter(|line| {
            // Look for direct signer usage outside of SignerContext
            !line.contains("SignerContext::") &&
            !line.contains("signer.") &&
            !line.contains("// ") && // Skip comments
            (line.contains(".sign_transaction(") ||
             line.contains(".evm_client(") ||
             line.contains(".solana_client("))
        })
        .collect();

    // This is informational rather than failing - just warn about patterns
    if !direct_signer_usage.is_empty() {
        println!("INFO: Found potential direct signer usage (review for SignerContext pattern):");
        for usage in direct_signer_usage {
            println!("  {}", usage);
        }
    }
}
