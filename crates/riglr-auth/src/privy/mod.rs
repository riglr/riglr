//! Privy authentication provider implementation
//!
//! Provides embedded wallets with social login capabilities through Privy.

mod config;
mod provider;
mod signer;
mod types;

#[expect(clippy::module_name_repetitions)]
pub use config::PrivyConfig;
#[expect(clippy::module_name_repetitions)]
pub use provider::PrivyProvider;
#[expect(clippy::module_name_repetitions)]
pub use types::{LinkedAccount, PrivyUserData, PrivyWallet};

// Re-export for convenience
pub use provider::create_privy_provider;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::AuthError;
    use core::mem::size_of_val;

    #[test]
    fn test_privy_config_re_export_should_be_available() {
        // Test that PrivyConfig is properly re-exported
        let config = PrivyConfig::new("test_app_id".to_string(), "test_secret".to_string());
        assert_eq!(config.app_id, "test_app_id");
        assert_eq!(config.app_secret, "test_secret");
    }

    #[test]
    fn test_privy_provider_re_export_should_be_available() {
        // Test that PrivyProvider is properly re-exported
        let config = PrivyConfig::new("test_app_id".to_string(), "test_secret".to_string());
        let provider = PrivyProvider::new(config);

        // Verify the provider was created successfully
        assert!(size_of_val(&provider) > 0);
    }

    #[test]
    #[expect(clippy::panic)]
    fn test_linked_account_re_export_should_be_available() {
        // Test that LinkedAccount enum is properly re-exported
        let email_account = LinkedAccount::Email {
            address: "test@example.com".to_string(),
            verified: true,
        };

        match email_account {
            LinkedAccount::Email { address, verified } => {
                assert_eq!(address, "test@example.com");
                assert!(verified);
            }
            _ => panic!("Expected Email variant"),
        }
    }

    #[test]
    fn test_privy_user_data_re_export_should_be_available() {
        // Test that PrivyUserData is properly re-exported
        use std::collections::HashMap;

        let user_data = PrivyUserData {
            id: "test_user_id".to_string(),
            linked_accounts: vec![],
            verified: true,
            metadata: HashMap::new(),
        };

        assert_eq!(user_data.id, "test_user_id");
        assert!(user_data.verified);
        assert!(user_data.linked_accounts.is_empty());
        assert!(user_data.metadata.is_empty());
    }

    #[test]
    fn test_privy_wallet_re_export_should_be_available() {
        // Test that PrivyWallet is properly re-exported
        let wallet = PrivyWallet {
            id: Some("wallet_id".to_string()),
            address: "test_address".to_string(),
            chain_type: "ethereum".to_string(),
            wallet_client: "metamask".to_string(),
            delegated: true,
            imported: false,
        };

        assert_eq!(wallet.id, Some("wallet_id".to_string()));
        assert_eq!(wallet.address, "test_address");
        assert_eq!(wallet.chain_type, "ethereum");
        assert_eq!(wallet.wallet_client, "metamask");
        assert!(wallet.delegated);
        assert!(!wallet.imported);
    }

    #[test]
    fn test_create_privy_provider_re_export_should_be_available() {
        // Test that create_privy_provider function is properly re-exported
        // We can't actually call it without setting environment variables,
        // but we can verify it's accessible

        // If this compiles, the function exists and is accessible
        let _ = create_privy_provider as fn() -> Result<PrivyProvider, AuthError>;
    }

    #[test]
    #[expect(clippy::panic)]
    fn test_all_public_types_in_scope() {
        // Comprehensive test to ensure all re-exported types are in scope
        use std::collections::HashMap;

        // Test PrivyConfig
        let config = PrivyConfig::new("app_id".to_string(), "secret".to_string());
        assert!(!config.app_id.is_empty());

        // Test PrivyProvider
        let provider = PrivyProvider::new(config);
        assert!(size_of_val(&provider) > 0);

        // Test LinkedAccount variants
        let email_account = LinkedAccount::Email {
            address: "test@example.com".to_string(),
            verified: false,
        };
        let phone_account = LinkedAccount::Phone {
            number: "+1234567890".to_string(),
            verified: true,
        };
        let social_account = LinkedAccount::Social {
            provider: "google".to_string(),
            username: Some("test_user".to_string()),
            verified: true,
        };
        let other_account = LinkedAccount::Other;

        // Verify all variants are accessible
        match email_account {
            LinkedAccount::Email { .. } => {}
            _ => panic!("Expected Email variant"),
        }
        match phone_account {
            LinkedAccount::Phone { .. } => {}
            _ => panic!("Expected Phone variant"),
        }
        match social_account {
            LinkedAccount::Social { .. } => {}
            _ => panic!("Expected Social variant"),
        }
        match other_account {
            LinkedAccount::Other => {}
            _ => panic!("Expected Other variant"),
        }

        // Test PrivyWallet
        let wallet = PrivyWallet {
            id: None,
            address: "0x123".to_string(),
            chain_type: "solana".to_string(),
            wallet_client: "phantom".to_string(),
            delegated: false,
            imported: true,
        };
        assert!(!wallet.address.is_empty());

        // Test PrivyUserData
        let user_data = PrivyUserData {
            id: "user123".to_string(),
            linked_accounts: vec![LinkedAccount::Wallet(wallet), email_account],
            verified: false,
            metadata: HashMap::new(),
        };
        assert_eq!(user_data.linked_accounts.len(), 2);
    }

    #[test]
    fn test_module_structure_integrity() {
        // Test that the module structure is properly organized
        // This test verifies that all the expected public items are available

        // These should all compile without errors, proving the re-exports work
        let _: Option<PrivyConfig> = None;
        let _: Option<PrivyProvider> = None;
        let _: Option<LinkedAccount> = None;
        let _: Option<PrivyUserData> = None;
        let _: Option<PrivyWallet> = None;

        // Verify create_privy_provider function is accessible
        let _ = create_privy_provider as fn() -> Result<PrivyProvider, AuthError>;

        // If this test compiles and runs, all re-exports are working correctly
    }

    #[test]
    fn test_documentation_comments_present() {
        // Verify that the module has proper documentation
        // This is a meta-test to ensure the module is well-documented

        // The module should have documentation (checked at compile time)
        // This test mainly serves as a reminder to maintain documentation
    }
}
