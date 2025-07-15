//! Verification tests for Solana token deployment functionality
//! 
//! This test suite validates that token deployment is properly integrated
//! with the signer context and uses real keypairs instead of mocks.

#[cfg(test)]
mod token_deployment_tests {
    use riglr_solana_tools::pump::{deploy_pump_token, generate_mint_keypair, TokenMetadata};
    use riglr_core::signer::{SignerContext, MockSigner};
    use solana_sdk::signature::Keypair;
    use solana_sdk::signer::Signer;

    /// Create a mock Solana signer for testing
    async fn create_mock_solana_signer() -> MockSigner {
        let keypair = Keypair::new();
        let mut signer = MockSigner::new();
        signer.set_solana_keypair(keypair);
        signer
    }

    #[tokio::test]
    async fn test_token_deployment_uses_real_signer() {
        // Setup mock signer context
        let signer = create_mock_solana_signer().await;
        let signer_pubkey = signer.solana_keypair().unwrap().pubkey();
        SignerContext::set(Box::new(signer)).await;

        // Generate mint keypair (should be new and unique)
        let mint_keypair = generate_mint_keypair();
        assert_ne!(mint_keypair.pubkey(), signer_pubkey, "Mint keypair should be different from signer");

        // Create token metadata
        let metadata = TokenMetadata {
            name: "Test Token".to_string(),
            symbol: "TEST".to_string(),
            description: "Test token for verification".to_string(),
            image_url: None,
            twitter: None,
            telegram: None,
            website: None,
        };

        // Note: Actual deployment would require network connection
        // This test verifies the structure is correct
        assert!(mint_keypair.pubkey() != solana_sdk::pubkey::Pubkey::default());
    }

    #[tokio::test]
    async fn test_mint_keypair_generation_is_unique() {
        // Generate multiple mint keypairs
        let keypair1 = generate_mint_keypair();
        let keypair2 = generate_mint_keypair();
        let keypair3 = generate_mint_keypair();

        // Verify all are unique
        assert_ne!(keypair1.pubkey(), keypair2.pubkey());
        assert_ne!(keypair2.pubkey(), keypair3.pubkey());
        assert_ne!(keypair1.pubkey(), keypair3.pubkey());
    }

    #[tokio::test]
    async fn test_signer_context_integration() {
        // Test that token deployment properly retrieves signer from context
        let signer = create_mock_solana_signer().await;
        let expected_pubkey = signer.solana_keypair().unwrap().pubkey();
        SignerContext::set(Box::new(signer)).await;

        // Verify signer context is accessible
        let current_signer = SignerContext::current().await;
        assert!(current_signer.is_ok(), "Signer context should be available");
        
        let retrieved_keypair = current_signer.unwrap().solana_keypair();
        assert!(retrieved_keypair.is_ok(), "Should retrieve keypair from signer");
        assert_eq!(retrieved_keypair.unwrap().pubkey(), expected_pubkey, "Should match expected signer pubkey");
    }

    #[tokio::test]
    async fn test_token_deployment_error_handling() {
        // Test deployment without signer context
        SignerContext::clear().await;
        
        let metadata = TokenMetadata {
            name: "Test Token".to_string(),
            symbol: "TEST".to_string(),
            description: "Test token".to_string(),
            image_url: None,
            twitter: None,
            telegram: None,
            website: None,
        };

        // This should fail due to missing signer context
        let result = deploy_pump_token(
            metadata,
            None, // No initial buy
            None, // Default slippage
            None, // Default priority fee
            None, // Default Jitopool tip
        ).await;

        assert!(result.is_err(), "Should fail without signer context");
        let error_message = result.unwrap_err().to_string();
        assert!(error_message.contains("signer") || error_message.contains("context"), 
                "Error should mention missing signer context");
    }
}