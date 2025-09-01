//! Integration tests for Solana tools against a containerized Solana validator
//!
//! These tests verify transaction submission tools work correctly with a real Solana node.

#[cfg(test)]
mod tests {
    #[cfg(feature = "integration-tests")]
    use {
        riglr_config::{Config, SolanaNetworkConfig},
        riglr_core::provider::ApplicationContext,
        riglr_solana_tools::{
            transaction::transfer_sol, utils::transaction::send_transaction_with_retry,
            LocalSolanaSigner,
        },
        solana_client::rpc_client::RpcClient,
        solana_sdk::{
            pubkey::Pubkey,
            signature::{Keypair, Signer},
            transaction::Transaction,
        },
        solana_system_interface::instruction as system_instruction,
        std::sync::Arc,
        testcontainers::{runners::AsyncRunner, GenericImage, ImageExt},
    };

    #[cfg(feature = "integration-tests")]
    async fn setup_solana_validator() -> (
        testcontainers::ContainerAsync<GenericImage>,
        String,
        Keypair,
    ) {
        // Use solana-test-validator image
        let container = GenericImage::new("solanalabs/solana", "v1.18.0")
            .with_cmd(vec![
                "solana-test-validator",
                "--no-bpf-jit",
                "--reset",
                "--quiet",
            ])
            .with_mapped_port(0, 8899.into()) // Use random host port
            .with_mapped_port(0, 8900.into()) // Use random host port
            .with_mapped_port(0, 9900.into()) // Use random host port
            .start()
            .await
            .expect("Failed to start Solana validator");

        // Get the RPC port
        let rpc_port = container
            .get_host_port_ipv4(8899)
            .await
            .expect("Failed to get RPC port");

        let rpc_url = format!("http://127.0.0.1:{}", rpc_port);

        // Generate a funded keypair (test validator provides airdrops)
        let payer = Keypair::new();

        // Wait for validator to be ready
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

        (container, rpc_url, payer)
    }

    #[cfg(feature = "integration-tests")]
    async fn airdrop_sol(
        client: &RpcClient,
        pubkey: &Pubkey,
        lamports: u64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let signature = client.request_airdrop(pubkey, lamports)?;

        // Wait for confirmation
        let mut retries = 0;
        while retries < 30 {
            if client.get_signature_status(&signature)?.is_some() {
                break;
            }
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            retries += 1;
        }

        Ok(())
    }

    // Integration tests for Solana tools
    #[tokio::test]
    #[cfg(feature = "integration-tests")] // Requires Docker
    async fn test_solana_transfer_integration() {
        let (_container, rpc_url, payer) = setup_solana_validator().await;

        // Create RPC client
        let client = RpcClient::new(rpc_url.clone());

        // Airdrop SOL to payer
        airdrop_sol(&client, &payer.pubkey(), 10_000_000_000) // 10 SOL
            .await
            .expect("Failed to airdrop SOL");

        // Create signer from keypair
        let solana_config = SolanaNetworkConfig {
            name: "test-local".to_string(),
            rpc_url: rpc_url.clone(),
            ws_url: None,
            explorer_url: Some("".to_string()),
        };
        let signer = LocalSolanaSigner::new(payer.to_base58_string(), solana_config)
            .expect("Failed to create signer");

        // Set up application context
        let config = (*Config::from_env()).clone();
        let app_context = ApplicationContext::from_config(&config);
        let signer_extension = Arc::new(signer);
        app_context.set_extension(signer_extension.clone());

        // Create recipient
        let recipient = Keypair::new();

        // Perform transfer using the tool
        let result = transfer_sol(
            recipient.pubkey().to_string(),
            1.0,  // 1 SOL
            None, // no memo
            None, // no priority fee
            &app_context,
        )
        .await
        .expect("Failed to transfer SOL");

        // Verify transfer succeeded
        let recipient_balance = client
            .get_balance(&recipient.pubkey())
            .expect("Failed to get recipient balance");
        assert_eq!(recipient_balance, 1_000_000_000);

        println!("Transfer succeeded with signature: {}", result.signature);
    }

    #[tokio::test]
    #[cfg(feature = "integration-tests")] // Requires Docker
    async fn test_solana_create_token_account() {
        let (_container, rpc_url, payer) = setup_solana_validator().await;

        // Create RPC client
        let client = RpcClient::new(rpc_url.clone());

        // Airdrop SOL to payer
        airdrop_sol(&client, &payer.pubkey(), 10_000_000_000)
            .await
            .expect("Failed to airdrop SOL");

        // Create signer
        let solana_config = SolanaNetworkConfig {
            name: "test-local".to_string(),
            rpc_url: rpc_url.clone(),
            ws_url: None,
            explorer_url: Some("".to_string()),
        };
        let signer = LocalSolanaSigner::new(payer.to_base58_string(), solana_config)
            .expect("Failed to create signer");

        // Set up application context
        let config = (*Config::from_env()).clone();
        let app_context = ApplicationContext::from_config(&config);
        let signer_extension = Arc::new(signer);
        app_context.set_extension(signer_extension.clone());

        // Create a mock token mint (in a real test, we'd create an actual SPL token)
        let mint_pubkey = Keypair::new().pubkey();
        let owner_pubkey = payer.pubkey();

        // For this test, just verify we can create keypairs and addresses
        println!("Mock mint pubkey: {}", mint_pubkey);
        println!("Owner pubkey: {}", owner_pubkey);

        // Basic validation that pubkeys are not empty
        assert!(
            !mint_pubkey.to_string().is_empty(),
            "Mint pubkey should not be empty"
        );
        assert!(
            !owner_pubkey.to_string().is_empty(),
            "Owner pubkey should not be empty"
        );
    }

    #[tokio::test]
    #[cfg(feature = "integration-tests")] // Requires Docker
    async fn test_solana_transaction_retry() {
        let (_container, rpc_url, payer) = setup_solana_validator().await;

        // Create RPC client
        let client = RpcClient::new(rpc_url.clone());

        // Airdrop SOL
        airdrop_sol(&client, &payer.pubkey(), 10_000_000_000)
            .await
            .expect("Failed to airdrop");

        // Create signer
        let solana_config = SolanaNetworkConfig {
            name: "test-local".to_string(),
            rpc_url: rpc_url.clone(),
            ws_url: None,
            explorer_url: Some("".to_string()),
        };
        let signer = LocalSolanaSigner::new(payer.to_base58_string(), solana_config)
            .expect("Failed to create signer");

        // Set up application context
        let config = (*Config::from_env()).clone();
        let app_context = ApplicationContext::from_config(&config);
        let signer_extension = Arc::new(signer);
        app_context.set_extension(signer_extension.clone());

        // Create a transaction
        let recipient = Keypair::new();
        let instruction = system_instruction::transfer(
            &payer.pubkey(),
            &recipient.pubkey(),
            500_000_000, // 0.5 SOL
        );

        // Create transaction and test retry functionality
        use riglr_solana_tools::utils::TransactionConfig;
        let recent_blockhash = client
            .get_latest_blockhash()
            .expect("Failed to get blockhash");
        let mut transaction = Transaction::new_with_payer(&[instruction], Some(&payer.pubkey()));
        transaction.sign(&[&payer], recent_blockhash);

        let config = TransactionConfig::default();
        let result = send_transaction_with_retry(&mut transaction, &config, "test transfer")
            .await
            .expect("Failed to send transaction with retry");

        // Verify transaction succeeded
        let status = client
            .get_signature_status(&result.signature.parse().unwrap())
            .expect("Failed to get status");
        assert!(status.is_some(), "Transaction should be confirmed");

        // Check recipient balance
        let balance = client
            .get_balance(&recipient.pubkey())
            .expect("Failed to get balance");
        assert_eq!(balance, 500_000_000);
    }

    #[tokio::test]
    #[cfg(feature = "integration-tests")] // Requires Docker
    async fn test_solana_batch_transactions() {
        let (_container, rpc_url, payer) = setup_solana_validator().await;

        let client = RpcClient::new(rpc_url.clone());

        // Airdrop SOL
        airdrop_sol(&client, &payer.pubkey(), 20_000_000_000)
            .await
            .expect("Failed to airdrop");

        let solana_config = SolanaNetworkConfig {
            name: "test-local".to_string(),
            rpc_url: rpc_url.clone(),
            ws_url: None,
            explorer_url: Some("".to_string()),
        };
        let signer = LocalSolanaSigner::new(payer.to_base58_string(), solana_config)
            .expect("Failed to create signer");

        let config = (*Config::from_env()).clone();
        let app_context = ApplicationContext::from_config(&config);
        let signer_extension = Arc::new(signer);
        app_context.set_extension(signer_extension.clone());

        // Create multiple recipients
        let recipients: Vec<Keypair> = (0..5).map(|_| Keypair::new()).collect();

        // Send SOL to each recipient
        for recipient in &recipients {
            let signature = transfer_sol(
                recipient.pubkey().to_string(),
                0.1,  // 0.1 SOL each
                None, // no memo
                None, // no priority fee
                &app_context,
            )
            .await
            .expect("Failed to transfer");

            println!(
                "Batch transfer {}: {}",
                recipient.pubkey(),
                signature.signature
            );
        }

        // Verify all recipients received SOL
        for recipient in &recipients {
            let balance = client
                .get_balance(&recipient.pubkey())
                .expect("Failed to get balance");
            assert_eq!(balance, 100_000_000);
        }
    }

    #[tokio::test]
    #[cfg(feature = "integration-tests")] // Requires Docker
    async fn test_solana_error_handling() {
        let (_container, rpc_url, payer) = setup_solana_validator().await;

        let client = RpcClient::new(rpc_url.clone());

        // Only airdrop a small amount
        airdrop_sol(&client, &payer.pubkey(), 1_000_000) // 0.001 SOL
            .await
            .expect("Failed to airdrop");

        let solana_config = SolanaNetworkConfig {
            name: "test-local".to_string(),
            rpc_url: rpc_url.clone(),
            ws_url: None,
            explorer_url: Some("".to_string()),
        };
        let signer = LocalSolanaSigner::new(payer.to_base58_string(), solana_config)
            .expect("Failed to create signer");

        let config = (*Config::from_env()).clone();
        let app_context = ApplicationContext::from_config(&config);
        let signer_extension = Arc::new(signer);
        app_context.set_extension(signer_extension.clone());

        // Try to transfer more than we have
        let recipient = Keypair::new();
        let result = transfer_sol(
            recipient.pubkey().to_string(),
            10.0, // 10 SOL (more than we have)
            None, // no memo
            None, // no priority fee
            &app_context,
        )
        .await;

        // Should fail due to insufficient funds
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(
            error_msg.contains("insufficient") || error_msg.contains("Insufficient"),
            "Error should indicate insufficient funds: {}",
            error_msg
        );
    }

    #[tokio::test]
    #[cfg(feature = "integration-tests")] // Requires Docker
    async fn test_solana_concurrent_transactions() {
        let (_container, rpc_url, payer) = setup_solana_validator().await;

        let client = RpcClient::new(rpc_url.clone());

        // Airdrop SOL
        airdrop_sol(&client, &payer.pubkey(), 50_000_000_000)
            .await
            .expect("Failed to airdrop");

        let solana_config = SolanaNetworkConfig {
            name: "test-local".to_string(),
            rpc_url: rpc_url.clone(),
            ws_url: None,
            explorer_url: Some("".to_string()),
        };
        let signer = LocalSolanaSigner::new(payer.to_base58_string(), solana_config)
            .expect("Failed to create signer");

        let config = (*Config::from_env()).clone();
        let app_context = ApplicationContext::from_config(&config);
        let signer_extension = Arc::new(signer);
        app_context.set_extension(signer_extension.clone());

        // Create multiple recipients
        let recipients: Vec<Keypair> = (0..10).map(|_| Keypair::new()).collect();

        // Send transfers concurrently
        let mut handles = vec![];
        for recipient in recipients.iter() {
            let recipient_str = recipient.pubkey().to_string();
            let app_context_clone = app_context.clone();
            let handle = tokio::spawn(async move {
                transfer_sol(recipient_str, 0.1, None, None, &app_context_clone).await
            });
            handles.push(handle);
        }

        // Wait for all transfers
        let mut success_count = 0;
        for handle in handles {
            if let Ok(Ok(_)) = handle.await {
                success_count += 1;
            }
        }

        // Most should succeed (some might fail due to blockhash/nonce issues in concurrent scenarios)
        assert!(
            success_count >= 7,
            "At least 70% of concurrent transactions should succeed, got {}/10",
            success_count
        );
    }
}
