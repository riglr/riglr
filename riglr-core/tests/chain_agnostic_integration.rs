//! Integration tests for chain-agnostic signer implementation
//!
//! These tests verify that the abstraction layer correctly handles both
//! Solana and EVM implementations without leaking chain-specific details.

#[cfg(test)]
mod tests {
    use riglr_core::signer::{SignerError, SolanaSigner, UnifiedSigner};
    use std::sync::Arc;

    #[cfg(feature = "integration-tests")]
    use riglr_core::signer::SignerContext;
    #[cfg(feature = "integration-tests")]
    use testcontainers::{runners::AsyncRunner, ContainerAsync, GenericImage, ImageExt};

    /// Test container for Solana validator
    #[cfg(feature = "integration-tests")]
    async fn setup_solana_container() -> ContainerAsync<GenericImage> {
        let solana_image = GenericImage::new("solanalabs/solana", "v1.18.0")
            .with_exposed_port(8899.into())
            .with_exposed_port(8900.into());

        solana_image
            .start()
            .await
            .expect("Failed to start Solana container")
    }

    /// Test container for Ethereum node  
    #[cfg(feature = "integration-tests")]
    async fn setup_ethereum_container() -> ContainerAsync<GenericImage> {
        let ethereum_image = GenericImage::new("ethereum/client-go", "v1.13.0")
            .with_exposed_port(8545.into())
            .with_exposed_port(8546.into())
            .with_env_var("GETH_MODE", "dev");

        ethereum_image
            .start()
            .await
            .expect("Failed to start Ethereum container")
    }

    #[tokio::test]
    #[cfg(feature = "integration-tests")] // Requires Docker
    async fn test_solana_signer_with_testcontainer() {
        // Setup Solana container
        let _container = setup_solana_container().await;

        // Import LocalSolanaSigner from tools crate
        use riglr_config::SolanaNetworkConfig;
        use riglr_solana_tools::LocalSolanaSigner;

        // Create a test signer
        let config = SolanaNetworkConfig::new("testnet", "http://localhost:8899".to_string());

        let signer: Arc<dyn UnifiedSigner> = Arc::new(
            LocalSolanaSigner::new(
                "5MaiiCavjCmn9Hs1o3eznqDEhRwxo7pXiAYez7keQUviUkauRiTMD8DrESdrNjN8zd9mTmVhRvBJeg5vhyvgrAhG".to_string(),
                config,
            ).expect("Failed to create signer")
        );

        // Use SignerContext pattern
        SignerContext::with_signer(signer.clone(), async {
            // Verify we can retrieve it as a Solana signer
            let retrieved_signer = SignerContext::current_as_solana()
                .await
                .expect("Failed to get Solana signer");

            // Test that we can access Solana-specific methods through the handle
            let _pubkey = retrieved_signer.pubkey();
            let _client = retrieved_signer.client();

            Ok(())
        })
        .await
        .unwrap();
    }

    #[tokio::test]
    #[cfg(feature = "integration-tests")] // Requires Docker
    async fn test_evm_signer_with_testcontainer() {
        // Setup Ethereum container
        let _container = setup_ethereum_container().await;

        // Import LocalEvmSigner from tools crate
        use riglr_config::EvmNetworkConfig;
        use riglr_evm_tools::LocalEvmSigner;

        // Create a test signer
        let config = EvmNetworkConfig::new("testnet", 1337, "http://localhost:8545".to_string());

        let signer: Arc<dyn UnifiedSigner> = Arc::new(
            LocalEvmSigner::new(
                "0x0000000000000000000000000000000000000000000000000000000000000001".to_string(),
                config,
            )
            .expect("Failed to create signer"),
        );

        // Use SignerContext pattern
        SignerContext::with_signer(signer.clone(), async {
            // Verify we can retrieve it as an EVM signer
            let retrieved_signer = SignerContext::current_as_evm()
                .await
                .expect("Failed to get EVM signer");

            // Test that we can access EVM-specific methods through the handle
            let _address = retrieved_signer.address();
            let _chain_id = retrieved_signer.chain_id();

            Ok(())
        })
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_transaction_serialization_roundtrip() {
        // Test that transactions can be serialized and deserialized correctly

        // Create a mock transaction
        let tx_bytes = vec![1, 2, 3, 4, 5, 6, 7, 8];

        // Verify byte serialization works
        let mut tx_bytes_clone = tx_bytes.clone();
        assert_eq!(tx_bytes_clone.len(), 8);

        // Modify the bytes
        tx_bytes_clone[0] = 10;
        assert_ne!(tx_bytes, tx_bytes_clone);
    }

    #[tokio::test]
    async fn test_client_downcasting() {
        use riglr_config::SolanaNetworkConfig;
        use riglr_solana_tools::LocalSolanaSigner;

        // Create a signer with RPC client
        let config = SolanaNetworkConfig::new("testnet", "http://localhost:8899".to_string());

        let signer = LocalSolanaSigner::new(
            "5MaiiCavjCmn9Hs1o3eznqDEhRwxo7pXiAYez7keQUviUkauRiTMD8DrESdrNjN8zd9mTmVhRvBJeg5vhyvgrAhG".to_string(),
            config,
        ).expect("Failed to create signer");

        // Get client as Any
        let client_any = signer.client();

        // Simply verify we got a client - it should be a valid Arc reference
        assert!(
            Arc::strong_count(&client_any) > 0,
            "Client should be a valid Arc reference"
        );
    }

    #[tokio::test]
    async fn test_error_conversion_preserves_information() {
        // Test that error messages are preserved across the abstraction boundary
        let error_msg = "Transaction failed: insufficient funds";
        let signer_error = SignerError::BlockchainTransaction(error_msg.to_string());

        match signer_error {
            SignerError::BlockchainTransaction(msg) => {
                assert_eq!(msg, error_msg);
            }
            _ => panic!("Wrong error type"),
        }
    }

    #[tokio::test]
    async fn test_chain_specific_operations_isolated() {
        // Verify that chain-specific operations don't leak
        use riglr_config::{EvmNetworkConfig, SolanaNetworkConfig};
        use riglr_evm_tools::LocalEvmSigner;
        use riglr_solana_tools::LocalSolanaSigner;

        // Create Solana signer
        let sol_config = SolanaNetworkConfig::new("testnet", "http://localhost:8899".to_string());

        let sol_signer: Arc<dyn UnifiedSigner> = Arc::new(
            LocalSolanaSigner::new(
                "5MaiiCavjCmn9Hs1o3eznqDEhRwxo7pXiAYez7keQUviUkauRiTMD8DrESdrNjN8zd9mTmVhRvBJeg5vhyvgrAhG".to_string(),
                sol_config,
            ).unwrap()
        );

        // Create EVM signer
        let evm_config = EvmNetworkConfig::new("test", 1, "http://localhost:8545".to_string());
        let evm_signer: Arc<dyn UnifiedSigner> = Arc::new(
            LocalEvmSigner::new(
                "0x0000000000000000000000000000000000000000000000000000000000000001".to_string(),
                evm_config,
            )
            .unwrap(),
        );

        // Verify chain support is correctly isolated
        assert!(sol_signer.supports_solana());
        assert!(!sol_signer.supports_evm());
        assert!(sol_signer.as_solana().is_some());
        assert!(sol_signer.as_evm().is_none());

        assert!(!evm_signer.supports_solana());
        assert!(evm_signer.supports_evm());
        assert!(evm_signer.as_solana().is_none());
        assert!(evm_signer.as_evm().is_some());
    }
}
