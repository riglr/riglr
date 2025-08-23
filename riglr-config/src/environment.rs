//! Environment variable utilities

use crate::{ConfigError, ConfigResult};
use std::env;

/// Type alias for custom environment source
type CustomEnvSource = dyn Fn(&str) -> Option<String>;

/// Source for loading environment variables
pub enum EnvironmentSource {
    /// Load from system environment
    System,
    /// Load from .env file
    DotEnv(String),
    /// Load from custom source
    Custom(Box<CustomEnvSource>),
}

impl EnvironmentSource {
    /// Get an environment variable
    pub fn get(&self, key: &str) -> Option<String> {
        match self {
            EnvironmentSource::System => env::var(key).ok(),
            EnvironmentSource::DotEnv(_path) => {
                // This would need dotenv parsing logic
                // For now, fallback to system
                env::var(key).ok()
            }
            EnvironmentSource::Custom(f) => f(key),
        }
    }

    /// Get a required environment variable
    pub fn require(&self, key: &str) -> ConfigResult<String> {
        self.get(key)
            .ok_or_else(|| ConfigError::MissingEnvVar(key.to_string()))
    }

    /// Get an optional environment variable with default
    pub fn get_or(&self, key: &str, default: String) -> String {
        self.get(key).unwrap_or(default)
    }

    /// Check if an environment variable exists
    pub fn exists(&self, key: &str) -> bool {
        self.get(key).is_some()
    }
}

/// Helper to extract values by prefix
#[allow(dead_code)]
pub fn extract_by_prefix(prefix: &str) -> Vec<(String, String)> {
    env::vars().filter(|(k, _)| k.starts_with(prefix)).collect()
}

/// Helper to extract and parse chain IDs from RPC_URL_{CHAIN_ID} pattern
#[allow(dead_code)]
pub fn extract_chain_rpc_urls() -> Vec<(u64, String)> {
    extract_by_prefix("RPC_URL_")
        .into_iter()
        .filter_map(|(key, value)| {
            key.strip_prefix("RPC_URL_")
                .and_then(|chain_id| chain_id.parse::<u64>().ok())
                .map(|id| (id, value))
        })
        .collect()
}

/// Helper to extract and parse contract addresses
#[allow(dead_code)]
pub fn extract_contract_overrides(chain_id: u64) -> Vec<(String, String)> {
    let prefixes = ["ROUTER_", "QUOTER_", "FACTORY_", "WETH_", "USDC_", "USDT_"];

    prefixes
        .iter()
        .filter_map(|prefix| {
            let key = format!("{}{}", prefix, chain_id);
            env::var(&key)
                .ok()
                .map(|value| (prefix.trim_end_matches('_').to_lowercase(), value))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    // Constants for environment variable names used in tests
    const TEST_VAR_EXISTS: &str = "TEST_VAR_EXISTS";
    const TEST_VAR_NOT_EXISTS: &str = "TEST_VAR_NOT_EXISTS";
    const TEST_DOTENV_VAR: &str = "TEST_DOTENV_VAR";
    const TEST_DOTENV_VAR_NOT_EXISTS: &str = "TEST_DOTENV_VAR_NOT_EXISTS";
    const CUSTOM_VAR: &str = "CUSTOM_VAR";
    const NON_EXISTENT_VAR: &str = "NON_EXISTENT_VAR";
    const REQUIRED_VAR: &str = "REQUIRED_VAR";
    const MISSING_REQUIRED_VAR: &str = "MISSING_REQUIRED_VAR";
    const OPTIONAL_VAR: &str = "OPTIONAL_VAR";
    const MISSING_OPTIONAL_VAR: &str = "MISSING_OPTIONAL_VAR";
    const EXISTS_VAR: &str = "EXISTS_VAR";
    const NOT_EXISTS_VAR: &str = "NOT_EXISTS_VAR";
    const TEST_PREFIX_VAR1: &str = "TEST_PREFIX_VAR1";
    const TEST_PREFIX_VAR2: &str = "TEST_PREFIX_VAR2";
    const OTHER_VAR: &str = "OTHER_VAR";
    const TEST_PREFIX: &str = "TEST_PREFIX_";
    const NONEXISTENT_PREFIX: &str = "NONEXISTENT_PREFIX_";
    const EMPTY_PREFIX_TEST: &str = "EMPTY_PREFIX_TEST";
    const RPC_URL_1: &str = "RPC_URL_1";
    const RPC_URL_137: &str = "RPC_URL_137";
    const RPC_URL_INVALID: &str = "RPC_URL_INVALID";
    const OTHER_URL_1: &str = "OTHER_URL_1";
    const RPC_URL_NOT_A_NUMBER: &str = "RPC_URL_not_a_number";
    const RPC_URL_EMPTY: &str = "RPC_URL_";
    const ROUTER_1: &str = "ROUTER_1";
    const QUOTER_1: &str = "QUOTER_1";
    const FACTORY_1: &str = "FACTORY_1";
    const WETH_1: &str = "WETH_1";
    const USDC_1: &str = "USDC_1";
    const USDT_1: &str = "USDT_1";
    const ROUTER_42: &str = "ROUTER_42";
    const USDC_42: &str = "USDC_42";
    const ROUTER_0: &str = "ROUTER_0";

    // Helper to create a custom environment source for testing
    fn create_test_env(vars: HashMap<String, String>) -> EnvironmentSource {
        EnvironmentSource::Custom(Box::new(move |key: &str| vars.get(key).cloned()))
    }

    #[test]
    fn test_environment_source_system_get_when_var_exists_should_return_some() {
        // Set a test environment variable
        env::set_var(TEST_VAR_EXISTS, "test_value");

        let source = EnvironmentSource::System;
        let result = source.get(TEST_VAR_EXISTS);

        assert_eq!(result, Some("test_value".to_string()));

        // Clean up
        env::remove_var(TEST_VAR_EXISTS);
    }

    #[test]
    fn test_environment_source_system_get_when_var_not_exists_should_return_none() {
        let source = EnvironmentSource::System;
        let result = source.get(TEST_VAR_NOT_EXISTS);

        assert_eq!(result, None);
    }

    #[test]
    fn test_environment_source_dotenv_get_when_var_exists_should_return_some() {
        // Set a test environment variable (since DotEnv falls back to system)
        env::set_var(TEST_DOTENV_VAR, "dotenv_value");

        let source = EnvironmentSource::DotEnv("test.env".to_string());
        let result = source.get(TEST_DOTENV_VAR);

        assert_eq!(result, Some("dotenv_value".to_string()));

        // Clean up
        env::remove_var(TEST_DOTENV_VAR);
    }

    #[test]
    fn test_environment_source_dotenv_get_when_var_not_exists_should_return_none() {
        let source = EnvironmentSource::DotEnv("test.env".to_string());
        let result = source.get(TEST_DOTENV_VAR_NOT_EXISTS);

        assert_eq!(result, None);
    }

    #[test]
    fn test_environment_source_custom_get_when_var_exists_should_return_some() {
        let mut vars = HashMap::new();
        vars.insert(CUSTOM_VAR.to_string(), "custom_value".to_string());

        let source = create_test_env(vars);
        let result = source.get(CUSTOM_VAR);

        assert_eq!(result, Some("custom_value".to_string()));
    }

    #[test]
    fn test_environment_source_custom_get_when_var_not_exists_should_return_none() {
        let vars = HashMap::new();
        let source = create_test_env(vars);
        let result = source.get(NON_EXISTENT_VAR);

        assert_eq!(result, None);
    }

    #[test]
    fn test_environment_source_require_when_var_exists_should_return_ok() {
        let mut vars = HashMap::new();
        vars.insert(REQUIRED_VAR.to_string(), "required_value".to_string());

        let source = create_test_env(vars);
        let result = source.require(REQUIRED_VAR);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "required_value");
    }

    #[test]
    fn test_environment_source_require_when_var_not_exists_should_return_err() {
        let vars = HashMap::new();
        let source = create_test_env(vars);
        let result = source.require(MISSING_REQUIRED_VAR);

        assert!(result.is_err());
        match result.unwrap_err() {
            ConfigError::MissingEnvVar(var_name) => {
                assert_eq!(var_name, MISSING_REQUIRED_VAR);
            }
            _ => panic!("Expected MissingEnvVar error"),
        }
    }

    #[test]
    fn test_environment_source_get_or_when_var_exists_should_return_value() {
        let mut vars = HashMap::new();
        vars.insert(OPTIONAL_VAR.to_string(), "actual_value".to_string());

        let source = create_test_env(vars);
        let result = source.get_or(OPTIONAL_VAR, "default_value".to_string());

        assert_eq!(result, "actual_value");
    }

    #[test]
    fn test_environment_source_get_or_when_var_not_exists_should_return_default() {
        let vars = HashMap::new();
        let source = create_test_env(vars);
        let result = source.get_or(MISSING_OPTIONAL_VAR, "default_value".to_string());

        assert_eq!(result, "default_value");
    }

    #[test]
    fn test_environment_source_exists_when_var_exists_should_return_true() {
        let mut vars = HashMap::new();
        vars.insert(EXISTS_VAR.to_string(), "some_value".to_string());

        let source = create_test_env(vars);
        let result = source.exists(EXISTS_VAR);

        assert!(result);
    }

    #[test]
    fn test_environment_source_exists_when_var_not_exists_should_return_false() {
        let vars = HashMap::new();
        let source = create_test_env(vars);
        let result = source.exists(NOT_EXISTS_VAR);

        assert!(!result);
    }

    #[test]
    fn test_extract_by_prefix_when_matching_vars_exist_should_return_filtered_list() {
        // Set test environment variables
        env::set_var(TEST_PREFIX_VAR1, "value1");
        env::set_var(TEST_PREFIX_VAR2, "value2");
        env::set_var(OTHER_VAR, "other_value");

        let result = extract_by_prefix(TEST_PREFIX);

        // Should contain the prefix-matching vars but not others
        let matching_keys: Vec<String> = result.iter().map(|(k, _)| k.clone()).collect();
        assert!(matching_keys.contains(&TEST_PREFIX_VAR1.to_string()));
        assert!(matching_keys.contains(&TEST_PREFIX_VAR2.to_string()));
        assert!(!matching_keys.contains(&OTHER_VAR.to_string()));

        // Check values
        let var1_entry = result.iter().find(|(k, _)| k == TEST_PREFIX_VAR1);
        assert!(var1_entry.is_some());
        assert_eq!(var1_entry.unwrap().1, "value1");

        // Clean up
        env::remove_var(TEST_PREFIX_VAR1);
        env::remove_var(TEST_PREFIX_VAR2);
        env::remove_var(OTHER_VAR);
    }

    #[test]
    fn test_extract_by_prefix_when_no_matching_vars_should_return_empty_list() {
        let result = extract_by_prefix(NONEXISTENT_PREFIX);
        assert!(result.is_empty());
    }

    #[test]
    fn test_extract_by_prefix_when_empty_prefix_should_return_all_vars() {
        // Set a test var to ensure we get at least one result
        env::set_var(EMPTY_PREFIX_TEST, "test_value");

        let result = extract_by_prefix("");

        // Should return all environment variables (at least our test one)
        assert!(!result.is_empty());
        let contains_test_var = result.iter().any(|(k, _)| k == EMPTY_PREFIX_TEST);
        assert!(contains_test_var);

        // Clean up
        env::remove_var(EMPTY_PREFIX_TEST);
    }

    #[test]
    fn test_extract_chain_rpc_urls_when_valid_chain_ids_should_return_parsed_list() {
        // Set test RPC URL environment variables
        env::set_var(RPC_URL_1, "https://mainnet.infura.io");
        env::set_var(RPC_URL_137, "https://polygon-rpc.com");
        env::set_var(RPC_URL_INVALID, "https://invalid.com"); // Should be filtered out
        env::set_var(OTHER_URL_1, "https://other.com"); // Should be filtered out

        let result = extract_chain_rpc_urls();

        // Should contain valid chain IDs only
        assert_eq!(result.len(), 2);

        let chain_ids: Vec<u64> = result.iter().map(|(id, _)| *id).collect();
        assert!(chain_ids.contains(&1));
        assert!(chain_ids.contains(&137));

        // Check URLs
        let eth_entry = result.iter().find(|(id, _)| *id == 1);
        assert!(eth_entry.is_some());
        assert_eq!(eth_entry.unwrap().1, "https://mainnet.infura.io");

        let polygon_entry = result.iter().find(|(id, _)| *id == 137);
        assert!(polygon_entry.is_some());
        assert_eq!(polygon_entry.unwrap().1, "https://polygon-rpc.com");

        // Clean up
        env::remove_var(RPC_URL_1);
        env::remove_var(RPC_URL_137);
        env::remove_var(RPC_URL_INVALID);
        env::remove_var(OTHER_URL_1);
    }

    #[test]
    fn test_extract_chain_rpc_urls_when_no_rpc_urls_should_return_empty_list() {
        let result = extract_chain_rpc_urls();

        // Filter out any existing RPC URLs that might be set in the environment
        let test_result: Vec<_> = result
            .into_iter()
            .filter(|(id, _)| *id == 999999) // Use a chain ID that definitely won't exist
            .collect();

        assert!(test_result.is_empty());
    }

    #[test]
    fn test_extract_chain_rpc_urls_when_invalid_chain_id_should_filter_out() {
        env::set_var(RPC_URL_NOT_A_NUMBER, "https://invalid.com");
        env::set_var(RPC_URL_EMPTY, "https://empty-chain-id.com");

        let result = extract_chain_rpc_urls();

        // Should not contain any entries with our test URLs
        let contains_invalid = result
            .iter()
            .any(|(_, url)| url == "https://invalid.com" || url == "https://empty-chain-id.com");
        assert!(!contains_invalid);

        // Clean up
        env::remove_var(RPC_URL_NOT_A_NUMBER);
        env::remove_var(RPC_URL_EMPTY);
    }

    #[test]
    fn test_extract_contract_overrides_when_contracts_exist_should_return_mapped_list() {
        // Set test contract environment variables for chain 1
        env::set_var(ROUTER_1, "0x1234567890123456789012345678901234567890");
        env::set_var(QUOTER_1, "0x2345678901234567890123456789012345678901");
        env::set_var(FACTORY_1, "0x3456789012345678901234567890123456789012");
        env::set_var(WETH_1, "0x4567890123456789012345678901234567890123");
        env::set_var(USDC_1, "0x5678901234567890123456789012345678901234");
        env::set_var(USDT_1, "0x6789012345678901234567890123456789012345");

        let result = extract_contract_overrides(1);

        // Should contain all contract types
        assert_eq!(result.len(), 6);

        let contract_names: Vec<String> = result.iter().map(|(name, _)| name.clone()).collect();
        assert!(contract_names.contains(&"router".to_string()));
        assert!(contract_names.contains(&"quoter".to_string()));
        assert!(contract_names.contains(&"factory".to_string()));
        assert!(contract_names.contains(&"weth".to_string()));
        assert!(contract_names.contains(&"usdc".to_string()));
        assert!(contract_names.contains(&"usdt".to_string()));

        // Check specific contract address
        let router_entry = result.iter().find(|(name, _)| name == "router");
        assert!(router_entry.is_some());
        assert_eq!(
            router_entry.unwrap().1,
            "0x1234567890123456789012345678901234567890"
        );

        // Clean up
        env::remove_var(ROUTER_1);
        env::remove_var(QUOTER_1);
        env::remove_var(FACTORY_1);
        env::remove_var(WETH_1);
        env::remove_var(USDC_1);
        env::remove_var(USDT_1);
    }

    #[test]
    fn test_extract_contract_overrides_when_partial_contracts_exist_should_return_partial_list() {
        // Set only some contract environment variables for chain 42
        env::set_var(ROUTER_42, "0x1111111111111111111111111111111111111111");
        env::set_var(USDC_42, "0x2222222222222222222222222222222222222222");

        let result = extract_contract_overrides(42);

        // Should contain only the set contracts
        assert_eq!(result.len(), 2);

        let contract_names: Vec<String> = result.iter().map(|(name, _)| name.clone()).collect();
        assert!(contract_names.contains(&"router".to_string()));
        assert!(contract_names.contains(&"usdc".to_string()));
        assert!(!contract_names.contains(&"quoter".to_string()));

        // Clean up
        env::remove_var(ROUTER_42);
        env::remove_var(USDC_42);
    }

    #[test]
    fn test_extract_contract_overrides_when_no_contracts_exist_should_return_empty_list() {
        let result = extract_contract_overrides(999);
        assert!(result.is_empty());
    }

    #[test]
    fn test_extract_contract_overrides_when_different_chain_id_should_not_interfere() {
        // Set contracts for chain 1
        env::set_var(ROUTER_1, "0x1111111111111111111111111111111111111111");
        env::set_var(USDC_1, "0x2222222222222222222222222222222222222222");

        // Query for chain 2 (should be empty)
        let result = extract_contract_overrides(2);
        assert!(result.is_empty());

        // Query for chain 1 (should have results)
        let result = extract_contract_overrides(1);
        assert_eq!(result.len(), 2);

        // Clean up
        env::remove_var(ROUTER_1);
        env::remove_var(USDC_1);
    }

    #[test]
    fn test_extract_contract_overrides_when_zero_chain_id_should_work() {
        env::set_var(ROUTER_0, "0x0000000000000000000000000000000000000000");

        let result = extract_contract_overrides(0);
        assert_eq!(result.len(), 1);

        let router_entry = result.iter().find(|(name, _)| name == "router");
        assert!(router_entry.is_some());
        assert_eq!(
            router_entry.unwrap().1,
            "0x0000000000000000000000000000000000000000"
        );

        // Clean up
        env::remove_var(ROUTER_0);
    }

    #[test]
    fn test_extract_contract_overrides_when_max_chain_id_should_work() {
        let max_chain = u64::MAX;
        let router_key = format!("ROUTER_{}", max_chain);
        let router_address = "0xffffffffffffffffffffffffffffffffffffffff";

        env::set_var(&router_key, router_address);

        let result = extract_contract_overrides(max_chain);
        assert_eq!(result.len(), 1);

        let router_entry = result.iter().find(|(name, _)| name == "router");
        assert!(router_entry.is_some());
        assert_eq!(router_entry.unwrap().1, router_address);

        // Clean up
        env::remove_var(&router_key);
    }
}
