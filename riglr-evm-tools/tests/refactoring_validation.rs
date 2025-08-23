//! Comprehensive test suite to validate riglr-evm-tools refactoring
//!
//! This test module validates that all functionality from riglr-evm-common
//! is now accessible through riglr-evm-tools after the refactoring.

use riglr_evm_tools::{
    chain_id_to_name,
    // Chain mapping
    chain_name_to_id,
    ensure_0x_prefix,
    format_address,
    format_address_string,
    get_chain_info,

    is_checksummed,
    known_addresses,

    // Address utilities
    parse_evm_address,
    strip_0x_prefix,
    validate_evm_address,
    EvmAccount,
    // Error types
    EvmCommonError,
    // Core types
    EvmConfig,
    EvmResult,

    EvmToken,
    EvmTransactionData,
};

#[cfg(test)]
mod tests {
    use super::*;

    /// Test that address utilities work correctly
    #[test]
    fn test_address_utilities() {
        let test_address = "0x742d35Cc6634C0532925a3b8D94E07F644b71Dc2";

        // Test parsing
        let parsed = parse_evm_address(test_address).unwrap();
        assert!(!parsed.is_zero());

        // Test validation
        assert!(validate_evm_address(test_address).is_ok());

        // Test formatting
        let formatted = format_address(&parsed);
        assert!(formatted.starts_with("0x"));
        assert_eq!(formatted.len(), 42);

        // Test string formatting
        let formatted_string =
            format_address_string("742d35cc6634c0532925a3b8d94e07f644b71dc2").unwrap();
        assert!(formatted_string.starts_with("0x"));
        assert_eq!(formatted_string.len(), 42);

        // Test checksumming
        let is_check = is_checksummed(test_address);
        assert!(is_check); // This specific address is checksummed

        // Test prefix operations
        let without_prefix = strip_0x_prefix(test_address);
        assert_eq!(without_prefix.len(), 40);
        assert!(!without_prefix.starts_with("0x"));

        let with_prefix = ensure_0x_prefix(without_prefix);
        assert!(with_prefix.starts_with("0x"));
        assert_eq!(with_prefix.len(), 42);

        // Test known addresses
        assert_eq!(
            known_addresses::ZERO_ADDRESS,
            "0x0000000000000000000000000000000000000000"
        );
        assert!(validate_evm_address(known_addresses::ZERO_ADDRESS).is_ok());
    }

    /// Test that chain utilities work correctly
    #[test]
    fn test_chain_utilities() {
        // Test chain name to ID mapping
        let eth_id = chain_name_to_id("ethereum").unwrap();
        assert_eq!(eth_id, 1);

        // Test case insensitive
        let eth_id_caps = chain_name_to_id("ETHEREUM").unwrap();
        assert_eq!(eth_id_caps, 1);

        // Test alias
        let eth_id_alias = chain_name_to_id("eth").unwrap();
        assert_eq!(eth_id_alias, 1);

        // Test reverse mapping
        let eth_name = chain_id_to_name(1).unwrap();
        assert_eq!(eth_name.to_lowercase(), "ethereum");

        // Test chain info
        let chain_info = get_chain_info(1).unwrap();
        assert_eq!(chain_info.chain_id, 1);
        assert_eq!(chain_info.name, "Ethereum");
        assert_eq!(chain_info.symbol, "ETH");
        assert!(chain_info.block_explorer.is_some());
        assert!(chain_info.default_rpc.is_some());

        // Test unsupported chain
        let unsupported = get_chain_info(99999);
        assert!(unsupported.is_none());

        // Test invalid chain name
        let invalid_name = chain_name_to_id("nonexistent");
        assert!(invalid_name.is_err());
    }

    /// Test that error types work correctly
    #[test]
    fn test_error_types() {
        // Test invalid address error
        let invalid_result: EvmResult<()> =
            Err(EvmCommonError::InvalidAddress("test error".to_string()));

        match invalid_result {
            Err(EvmCommonError::InvalidAddress(msg)) => {
                assert_eq!(msg, "test error");
            }
            _ => panic!("Expected InvalidAddress error"),
        }

        // Test that actual invalid addresses produce errors
        let parse_result = parse_evm_address("invalid_address");
        assert!(parse_result.is_err());

        match parse_result {
            Err(EvmCommonError::InvalidAddress(_)) => {
                // Expected
            }
            _ => panic!("Expected InvalidAddress error for invalid address"),
        }
    }

    /// Test that core types are accessible and work correctly
    #[test]
    fn test_core_types() {
        let test_address = "0x742d35Cc6634C0532925a3b8D94E07F644b71Dc2";

        // Test EvmConfig
        let config = EvmConfig {
            rpc_url: "https://eth.llamarpc.com".to_string(),
            chain_id: 1,
            timeout_seconds: 30,
            max_gas_limit: None,
            gas_price_multiplier: Some(1.1),
        };
        assert_eq!(config.chain_id, 1);
        assert_eq!(config.timeout_seconds, 30);
        assert!(config.validate().is_ok());

        // Test default config
        let default_config = EvmConfig::default();
        assert_eq!(default_config.chain_id, 1);
        assert!(default_config.gas_price_multiplier.is_some());

        // Test EvmAccount
        let account = EvmAccount {
            name: Some("test_account".to_string()),
            address: test_address.to_string(),
            chain_id: 1,
        };
        assert_eq!(account.chain_id, 1);
        assert_eq!(account.address, test_address);

        // Test EvmToken
        let token = EvmToken {
            address: test_address.to_string(),
            symbol: "TEST".to_string(),
            decimals: 18,
            name: "Test Token".to_string(),
            chain_id: 1,
        };
        assert_eq!(token.decimals, 18);
        assert_eq!(token.symbol, "TEST");

        // Test EvmTransactionData
        let tx_data = EvmTransactionData {
            to: test_address.to_string(),
            value: "1000000000000000000".to_string(), // 1 ETH in wei
            data: "0x".to_string(),
            gas_limit: "21000".to_string(),
            gas_price: "20000000000".to_string(), // 20 gwei
            chain_id: 1,
        };
        assert_eq!(tx_data.to, test_address);
        assert_eq!(tx_data.value, "1000000000000000000");
        assert_eq!(tx_data.chain_id, 1);
    }

    /// Integration test to verify that all functions work together
    #[test]
    fn test_integration() {
        // Test a complete workflow using multiple functions

        // 1. Parse an address
        let addr_str = "742d35cc6634c0532925a3b8d94e07f644b71dc2"; // lowercase without prefix
        let addr = parse_evm_address(addr_str).unwrap();

        // 2. Format it properly
        let formatted = format_address(&addr);

        // 3. Validate it's now checksummed
        assert!(is_checksummed(&formatted));

        // 4. Get chain info for Ethereum
        let chain_id = chain_name_to_id("ethereum").unwrap();
        let chain_info = get_chain_info(chain_id).unwrap();

        // 5. Create a transaction data structure
        let _tx_data = EvmTransactionData {
            to: formatted,
            value: "1000000000000000000".to_string(),
            data: "0x".to_string(),
            gas_limit: "21000".to_string(),
            gas_price: "20000000000".to_string(), // 20 gwei
            chain_id: chain_info.chain_id,
        };

        // If we get here, all the APIs work together correctly
    }
}
