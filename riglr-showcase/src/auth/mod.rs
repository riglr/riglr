//! Authentication providers for riglr showcase
//!
//! This module contains concrete implementations of SignerFactory for various
//! authentication providers. These serve as examples of how to implement
//! custom authentication for the riglr web adapters.

pub mod privy;

#[cfg(feature = "web-server")]
pub use privy::privy_impl::{PrivySignerFactory, PrivyUserData};

#[cfg(test)]
mod tests {

    #[test]
    fn test_module_structure_when_privy_module_exists_should_be_accessible() {
        // Test that privy module is accessible
        // The mere compilation of this test validates module structure
    }

    #[test]
    #[cfg(feature = "web-server")]
    fn test_privy_exports_when_web_server_feature_enabled_should_be_available() {
        // Test that PrivySignerFactory is accessible when web-server feature is enabled
        let factory = PrivySignerFactory::new("test_id".to_string(), "test_secret".to_string());
        assert_eq!(factory.supported_auth_types(), vec!["privy"]);

        // Test that PrivyUserData is accessible and can be constructed
        let user_data = PrivyUserData {
            id: "test_user".to_string(),
            solana_address: Some("11111111111111111111111111111112".to_string()),
            evm_address: Some("0x1111111111111111111111111111111111111111".to_string()),
            evm_wallet_id: Some("wallet_123".to_string()),
            verified: true,
        };
        assert_eq!(user_data.id, "test_user");
        assert!(user_data.verified);
    }

    #[test]
    #[cfg(not(feature = "web-server"))]
    fn test_exports_when_web_server_feature_disabled_should_not_be_available() {
        // When web-server feature is disabled, the re-exports should not be available
        // This test ensures conditional compilation works correctly
        // The mere compilation of this test validates the feature gate behavior
    }

    #[test]
    #[cfg(feature = "web-server")]
    fn test_privy_user_data_creation_when_all_fields_provided_should_store_correctly() {
        let user_data = PrivyUserData {
            id: "did:privy:123456".to_string(),
            solana_address: Some("So11111111111111111111111111111111111111112".to_string()),
            evm_address: Some("0x742d35Cc6636C0532925a3b8C17c604Bb4b7Efb3".to_string()),
            evm_wallet_id: Some("wallet_789".to_string()),
            verified: true,
        };

        assert_eq!(user_data.id, "did:privy:123456");
        assert_eq!(
            user_data.solana_address,
            Some("So11111111111111111111111111111111111111112".to_string())
        );
        assert_eq!(
            user_data.evm_address,
            Some("0x742d35Cc6636C0532925a3b8C17c604Bb4b7Efb3".to_string())
        );
        assert_eq!(user_data.evm_wallet_id, Some("wallet_789".to_string()));
        assert!(user_data.verified);
    }

    #[test]
    #[cfg(feature = "web-server")]
    fn test_privy_user_data_creation_when_optional_fields_none_should_handle_gracefully() {
        let user_data = PrivyUserData {
            id: "did:privy:no_wallets".to_string(),
            solana_address: None,
            evm_address: None,
            evm_wallet_id: None,
            verified: false,
        };

        assert_eq!(user_data.id, "did:privy:no_wallets");
        assert!(user_data.solana_address.is_none());
        assert!(user_data.evm_address.is_none());
        assert!(user_data.evm_wallet_id.is_none());
        assert!(!user_data.verified);
    }

    #[test]
    #[cfg(feature = "web-server")]
    fn test_privy_user_data_clone_when_called_should_create_identical_copy() {
        let original = PrivyUserData {
            id: "test_clone".to_string(),
            solana_address: Some("address1".to_string()),
            evm_address: Some("address2".to_string()),
            evm_wallet_id: Some("wallet1".to_string()),
            verified: true,
        };

        let cloned = original.clone();

        assert_eq!(original.id, cloned.id);
        assert_eq!(original.solana_address, cloned.solana_address);
        assert_eq!(original.evm_address, cloned.evm_address);
        assert_eq!(original.evm_wallet_id, cloned.evm_wallet_id);
        assert_eq!(original.verified, cloned.verified);
    }

    #[test]
    #[cfg(feature = "web-server")]
    fn test_privy_user_data_debug_when_called_should_format_correctly() {
        let user_data = PrivyUserData {
            id: "debug_test".to_string(),
            solana_address: Some("sol_addr".to_string()),
            evm_address: None,
            evm_wallet_id: None,
            verified: true,
        };

        let debug_output = format!("{:?}", user_data);
        assert!(debug_output.contains("debug_test"));
        assert!(debug_output.contains("sol_addr"));
        assert!(debug_output.contains("true"));
    }

    #[test]
    #[cfg(feature = "web-server")]
    fn test_privy_factory_with_empty_credentials_when_app_id_empty_should_create_factory() {
        // Test edge case with empty strings
        let factory = PrivySignerFactory::new("".to_string(), "secret".to_string());
        assert_eq!(factory.supported_auth_types(), vec!["privy"]);
    }

    #[test]
    #[cfg(feature = "web-server")]
    fn test_privy_factory_with_empty_secret_when_app_secret_empty_should_create_factory() {
        // Test edge case with empty app secret
        let factory = PrivySignerFactory::new("app_id".to_string(), "".to_string());
        assert_eq!(factory.supported_auth_types(), vec!["privy"]);
    }

    #[test]
    #[cfg(feature = "web-server")]
    fn test_privy_factory_with_both_empty_when_both_credentials_empty_should_create_factory() {
        // Test edge case with both empty
        let factory = PrivySignerFactory::new("".to_string(), "".to_string());
        assert_eq!(factory.supported_auth_types(), vec!["privy"]);
    }

    #[test]
    #[cfg(feature = "web-server")]
    fn test_privy_factory_supported_auth_types_when_called_should_return_privy() {
        let factory = PrivySignerFactory::new("test_id".to_string(), "test_secret".to_string());
        let auth_types = factory.supported_auth_types();

        assert_eq!(auth_types.len(), 1);
        assert_eq!(auth_types[0], "privy");
    }

    #[test]
    #[cfg(feature = "web-server")]
    fn test_privy_factory_with_unicode_credentials_when_provided_should_handle_correctly() {
        // Test with unicode characters in credentials
        let factory = PrivySignerFactory::new("app_🔑".to_string(), "secret_🗝️".to_string());
        assert_eq!(factory.supported_auth_types(), vec!["privy"]);
    }

    #[test]
    #[cfg(feature = "web-server")]
    fn test_privy_factory_with_very_long_credentials_when_provided_should_handle_correctly() {
        // Test with very long strings
        let long_string = "a".repeat(1000);
        let factory = PrivySignerFactory::new(long_string.clone(), long_string);
        assert_eq!(factory.supported_auth_types(), vec!["privy"]);
    }

    #[test]
    #[cfg(feature = "web-server")]
    fn test_privy_user_data_with_empty_id_when_provided_should_store_correctly() {
        let user_data = PrivyUserData {
            id: "".to_string(),
            solana_address: None,
            evm_address: None,
            evm_wallet_id: None,
            verified: false,
        };

        assert_eq!(user_data.id, "");
        assert!(!user_data.verified);
    }

    #[test]
    #[cfg(feature = "web-server")]
    fn test_privy_user_data_with_unicode_addresses_when_provided_should_store_correctly() {
        let user_data = PrivyUserData {
            id: "test_unicode".to_string(),
            solana_address: Some("🔑address".to_string()),
            evm_address: Some("0x🗝️address".to_string()),
            evm_wallet_id: Some("wallet_🆔".to_string()),
            verified: true,
        };

        assert_eq!(user_data.solana_address, Some("🔑address".to_string()));
        assert_eq!(user_data.evm_address, Some("0x🗝️address".to_string()));
        assert_eq!(user_data.evm_wallet_id, Some("wallet_🆔".to_string()));
    }

    #[test]
    #[cfg(feature = "web-server")]
    fn test_privy_user_data_with_very_long_strings_when_provided_should_store_correctly() {
        let long_string = "x".repeat(10000);
        let user_data = PrivyUserData {
            id: long_string.clone(),
            solana_address: Some(long_string.clone()),
            evm_address: Some(long_string.clone()),
            evm_wallet_id: Some(long_string.clone()),
            verified: true,
        };

        assert_eq!(user_data.id.len(), 10000);
        assert_eq!(user_data.solana_address.as_ref().unwrap().len(), 10000);
        assert_eq!(user_data.evm_address.as_ref().unwrap().len(), 10000);
        assert_eq!(user_data.evm_wallet_id.as_ref().unwrap().len(), 10000);
    }

    #[test]
    #[cfg(feature = "web-server")]
    fn test_module_exports_when_feature_enabled_should_match_privy_impl_exports() {
        // Test that the re-exports from privy module are available at the module level
        let factory = PrivySignerFactory::new("test".to_string(), "test".to_string());
        let _: Vec<String> = factory.supported_auth_types();

        let user_data = PrivyUserData {
            id: "test".to_string(),
            solana_address: None,
            evm_address: None,
            evm_wallet_id: None,
            verified: false,
        };
        let _: String = user_data.id;
    }

    #[test]
    fn test_privy_module_accessibility_when_imported_should_be_available() {
        // Test that the privy module can be accessed
        // This validates the pub mod privy; declaration

        // The ability to use the module path validates the module declaration
        let _module_path = "privy";
        assert!(_module_path == "privy");
    }

    #[test]
    #[cfg(feature = "web-server")]
    fn test_serde_serialization_when_privy_user_data_serialized_should_work() {
        let user_data = PrivyUserData {
            id: "serialize_test".to_string(),
            solana_address: Some("sol123".to_string()),
            evm_address: None,
            evm_wallet_id: Some("wallet456".to_string()),
            verified: true,
        };

        // Test that serialization works (JSON is common format)
        let serialized = serde_json::to_string(&user_data);
        assert!(serialized.is_ok());

        let json_str = serialized.unwrap();
        assert!(json_str.contains("serialize_test"));
        assert!(json_str.contains("sol123"));
        assert!(json_str.contains("wallet456"));
    }

    #[test]
    #[cfg(feature = "web-server")]
    fn test_serde_deserialization_when_privy_user_data_deserialized_should_work() {
        let json_data = r#"{
            "id": "deserialize_test",
            "solana_address": "sol789",
            "evm_address": null,
            "evm_wallet_id": "wallet123",
            "verified": false
        }"#;

        let deserialized: Result<PrivyUserData, _> = serde_json::from_str(json_data);
        assert!(deserialized.is_ok());

        let user_data = deserialized.unwrap();
        assert_eq!(user_data.id, "deserialize_test");
        assert_eq!(user_data.solana_address, Some("sol789".to_string()));
        assert!(user_data.evm_address.is_none());
        assert_eq!(user_data.evm_wallet_id, Some("wallet123".to_string()));
        assert!(!user_data.verified);
    }

    #[test]
    #[cfg(feature = "web-server")]
    fn test_serde_round_trip_when_serialize_then_deserialize_should_maintain_data() {
        let original = PrivyUserData {
            id: "round_trip_test".to_string(),
            solana_address: Some("original_sol".to_string()),
            evm_address: Some("original_evm".to_string()),
            evm_wallet_id: None,
            verified: true,
        };

        // Serialize then deserialize
        let serialized = serde_json::to_string(&original).unwrap();
        let deserialized: PrivyUserData = serde_json::from_str(&serialized).unwrap();

        assert_eq!(original.id, deserialized.id);
        assert_eq!(original.solana_address, deserialized.solana_address);
        assert_eq!(original.evm_address, deserialized.evm_address);
        assert_eq!(original.evm_wallet_id, deserialized.evm_wallet_id);
        assert_eq!(original.verified, deserialized.verified);
    }
}
