//! Integration tests for riglr-solana-common

use riglr_solana_common::{
    create_solana_client, default_solana_config, format_balance, format_solana_address,
    lamports_to_sol, sol_to_lamports, string_to_pubkey, validate_solana_address, SolanaAccount,
    SolanaConfig,
};

#[test]
fn test_solana_config_creation() {
    let config = SolanaConfig::default();
    assert_eq!(config.rpc_url, "https://api.mainnet-beta.solana.com");
    assert_eq!(config.commitment, "confirmed");
    assert_eq!(config.timeout_seconds, 30);
}

#[test]
fn test_default_solana_config_from_env() {
    // This test ensures default config works when no env vars are set
    let config = default_solana_config();
    assert!(!config.rpc_url.is_empty());
    assert!(!config.commitment.is_empty());
    assert!(config.timeout_seconds > 0);
}

#[test]
fn test_solana_account_creation() {
    // Test with valid Solana address
    let account = SolanaAccount::new("So11111111111111111111111111111111111111112", true, false);
    assert!(account.is_ok());

    let account = account.unwrap();
    assert_eq!(
        account.pubkey,
        "So11111111111111111111111111111111111111112"
    );
    assert!(account.is_signer);
    assert!(!account.is_writable);

    // Test conversion back to pubkey
    let pubkey = account.to_pubkey();
    assert!(pubkey.is_ok());
}

#[test]
fn test_solana_account_invalid_address() {
    let account = SolanaAccount::new("invalid", true, false);
    assert!(account.is_err());
}

#[test]
fn test_address_validation() {
    // Valid Solana address
    let result = validate_solana_address("So11111111111111111111111111111111111111112");
    assert!(result.is_ok());

    // Invalid address
    let result = validate_solana_address("invalid");
    assert!(result.is_err());
}

#[test]
fn test_string_to_pubkey() {
    let result = string_to_pubkey("So11111111111111111111111111111111111111112");
    assert!(result.is_ok());

    let result = string_to_pubkey("invalid");
    assert!(result.is_err());
}

#[test]
fn test_address_formatting() {
    let pubkey = string_to_pubkey("So11111111111111111111111111111111111111112").unwrap();
    let formatted = format_solana_address(&pubkey);
    assert_eq!(formatted, "So11111111111111111111111111111111111111112");
}

#[test]
fn test_solana_client_creation() {
    let config = SolanaConfig::default();
    let client = create_solana_client(&config);
    // Just ensure client creation doesn't panic
    // We can't test actual RPC calls without a live connection
    assert_eq!(client.url(), config.rpc_url);
}

#[test]
fn test_balance_conversions() {
    // Test lamports to SOL conversion
    assert_eq!(lamports_to_sol(1_000_000_000), 1.0);
    assert_eq!(lamports_to_sol(500_000_000), 0.5);
    assert_eq!(lamports_to_sol(1), 0.000000001);

    // Test SOL to lamports conversion
    assert_eq!(sol_to_lamports(1.0), 1_000_000_000);
    assert_eq!(sol_to_lamports(0.5), 500_000_000);
    assert_eq!(sol_to_lamports(0.000000001), 1);
}

#[test]
fn test_balance_formatting() {
    // Test large balance formatting
    let formatted = format_balance(1_000_000_000); // 1 SOL
    assert!(formatted.contains("SOL"));
    assert!(formatted.contains("1."));

    // Test small balance formatting (lamports)
    let formatted = format_balance(500);
    assert!(formatted.contains("lamports"));
    assert!(formatted.contains("500"));

    // Test medium balance formatting
    let formatted = format_balance(10_000_000); // 0.01 SOL
    assert!(formatted.contains("SOL"));
}
