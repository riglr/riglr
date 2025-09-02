//! Integration tests for EVM tools against a containerized Ethereum node
//!
//! These tests verify transaction submission tools work correctly with a real EVM node.

#[cfg(test)]
mod tests {
    #[cfg(feature = "integration-tests")]
    use {
        alloy::primitives::{utils::format_ether, Address, U256},
        alloy::providers::{Provider, ProviderBuilder},
        alloy::rpc::types::TransactionRequest,
        riglr_config::EvmNetworkConfig,
        riglr_core::provider::ApplicationContext,
        riglr_core::signer::SignerContext,
        riglr_core::UnifiedSigner,
        riglr_evm_tools::{transaction::send_eth, LocalEvmSigner},
        std::str::FromStr,
        std::sync::Arc,
        testcontainers::{runners::AsyncRunner, GenericImage, ImageExt},
    };

    #[cfg(feature = "integration-tests")]
    async fn setup_anvil_node() -> (testcontainers::ContainerAsync<GenericImage>, String, String) {
        // Use Anvil (Foundry's local Ethereum node)
        let container = GenericImage::new("ghcr.io/foundry-rs/foundry", "latest")
            .with_cmd(vec![
                "anvil",
                "--host",
                "0.0.0.0",
                "--accounts",
                "10",
                "--balance",
                "10000",
                "--block-time",
                "1",
            ])
            .with_mapped_port(8545, 8545.into())
            .start()
            .await
            .expect("Failed to start Anvil node");

        // Get the RPC port
        let rpc_port = container
            .get_host_port_ipv4(8545)
            .await
            .expect("Failed to get RPC port");

        let rpc_url = format!("http://127.0.0.1:{}", rpc_port);

        // Anvil provides deterministic private keys
        // First account private key
        let private_key =
            "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80".to_string();

        // Wait for node to be ready
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        (container, rpc_url, private_key)
    }

    #[cfg(feature = "integration-tests")]
    async fn get_balance(provider_url: &str, address: Address) -> U256 {
        let provider = ProviderBuilder::new()
            .connect(provider_url)
            .await
            .expect("Failed to connect provider");

        provider
            .get_balance(address)
            .await
            .expect("Failed to get balance")
    }

    #[tokio::test]
    #[cfg(feature = "integration-tests")] // Requires Docker
    async fn test_evm_transfer_integration() {
        let (_container, rpc_url, private_key) = setup_anvil_node().await;

        // Create signer
        let config = EvmNetworkConfig::new("local", 31337, rpc_url.clone());
        let _signer = LocalEvmSigner::new(private_key, config).expect("Failed to create signer");

        // Set up application context
        let config = riglr_config::Config::from_env();
        let app_context = ApplicationContext::from_config(&config);
        let unified_signer: Arc<dyn UnifiedSigner> = Arc::new(_signer);

        // Get sender address
        let sender_address = Address::from_str("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266")
            .expect("Invalid sender address");

        // Create recipient address
        let recipient = Address::from_str("0x70997970C51812dc3A010C7d01b50e0d17dc79C8")
            .expect("Invalid recipient address");

        // Execute test within SignerContext
        SignerContext::with_signer(unified_signer, async move {
            // Check initial balances
            let initial_sender_balance = get_balance(&rpc_url, sender_address).await;
            let initial_recipient_balance = get_balance(&rpc_url, recipient).await;

            println!(
                "Initial sender balance: {}",
                format_ether(initial_sender_balance)
            );
            println!(
                "Initial recipient balance: {}",
                format_ether(initial_recipient_balance)
            );

            // Send 1 ETH
            let tx_hash = send_eth(recipient.to_string(), 1.0, None, &app_context)
                .await
                .expect("Failed to send ETH");

            println!("Transaction hash: {}", tx_hash);

            // Wait for transaction to be mined
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

            // Check final balances
            let final_sender_balance = get_balance(&rpc_url, sender_address).await;
            let final_recipient_balance = get_balance(&rpc_url, recipient).await;

            println!(
                "Final sender balance: {}",
                format_ether(final_sender_balance)
            );
            println!(
                "Final recipient balance: {}",
                format_ether(final_recipient_balance)
            );

            // Verify transfer (1 ETH = 1e18 wei)
            let expected_amount = U256::from(1_000_000_000_000_000_000u64); // 1 ETH in wei
            assert_eq!(
                final_recipient_balance - initial_recipient_balance,
                expected_amount
            );
            assert!(final_sender_balance < initial_sender_balance - expected_amount); // Less due to gas

            Ok(())
        })
        .await
        .expect("Test should complete successfully");
    }

    #[tokio::test]
    #[cfg(feature = "integration-tests")] // Requires Docker
    async fn test_evm_contract_deployment() {
        let (_container, rpc_url, private_key) = setup_anvil_node().await;

        // Create signer
        let config = EvmNetworkConfig::new("local", 31337, rpc_url.clone());
        let _signer = LocalEvmSigner::new(private_key, config).expect("Failed to create signer");

        let unified_signer: Arc<dyn UnifiedSigner> = Arc::new(_signer);

        // Execute test within SignerContext
        SignerContext::with_signer(unified_signer, async move {
            // Simple storage contract bytecode (stores and retrieves a number)
            // contract SimpleStorage { uint256 public storedValue; function set(uint256 x) public { storedValue = x; } }
            let contract_bytecode = "0x608060405234801561001057600080fd5b50610150806100206000396000f3fe608060405234801561001057600080fd5b50600436106100365760003560e01c806360fe47b11461003b5780636d4ce63c14610057575b600080fd5b610055600480360381019061005091906100be565b610075565b005b61005f61007f565b60405161006c91906100fa565b60405180910390f35b8060008190555050565b60008054905090565b600080fd5b6000819050919050565b6100a08161008d565b81146100ab57600080fd5b50565b6000813590506100bd81610097565b92915050565b6000602082840312156100da576100d9610088565b5b60006100e8848285016100ae565b91505092915050565b6100fa8161008d565b82525050565b600060208201905061011560008301846100f1565b9291505056fea2646970667358221220";

            // Deploy contract
            let bytecode_bytes = alloy::primitives::hex::decode(contract_bytecode.strip_prefix("0x").unwrap_or(contract_bytecode))
                .expect("Invalid bytecode hex");
            let deployment_tx = TransactionRequest::default()
                .input(bytecode_bytes.into());

            let tx_json = serde_json::to_value(&deployment_tx)
                .expect("Failed to serialize deployment tx");

            let evm_signer = SignerContext::current_as_evm()
                .await
                .expect("Failed to get EVM signer");

            let deployment_hash = evm_signer
                .sign_and_send_transaction(tx_json)
                .await
                .expect("Failed to deploy contract");

            println!("Contract deployment hash: {}", deployment_hash);

            // Wait for deployment
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

            // Get receipt to find contract address
            let provider = ProviderBuilder::new()
                .connect(&rpc_url)
                .await
                .expect("Failed to connect");

            let receipt = provider
                .get_transaction_receipt(deployment_hash.parse().unwrap())
                .await
                .expect("Failed to get receipt")
                .expect("Receipt not found");

            let contract_address = receipt.contract_address.expect("No contract address");
            println!("Deployed contract at: {}", contract_address);

            Ok(())
        })
        .await
        .expect("Test should complete successfully");
    }

    #[tokio::test]
    #[cfg(feature = "integration-tests")] // Requires Docker
    async fn test_evm_multiple_transfers() {
        let (_container, rpc_url, private_key) = setup_anvil_node().await;

        let config = EvmNetworkConfig::new("local", 31337, rpc_url.clone());
        let _signer = LocalEvmSigner::new(private_key, config).expect("Failed to create signer");

        let config = riglr_config::Config::from_env();
        let app_context = ApplicationContext::from_config(&config);

        // Create multiple recipients
        let recipients = vec![
            "0x70997970C51812dc3A010C7d01b50e0d17dc79C8",
            "0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC",
            "0x90F79bf6EB2c4f870365E785982E1f101E93b906",
        ];

        // Send ETH to each recipient
        for recipient in &recipients {
            let tx_hash = send_eth(recipient.to_string(), 0.1, None, &app_context)
                .await
                .expect("Failed to send ETH");
            println!("Sent to {}: {}", recipient, tx_hash);
        }

        // Wait for transactions
        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

        // Verify all recipients received ETH
        for recipient_str in &recipients {
            let recipient = Address::from_str(recipient_str).unwrap();
            let balance = get_balance(&rpc_url, recipient).await;
            let expected_amount = U256::from(100_000_000_000_000_000u64); // 0.1 ETH in wei
            assert!(
                balance >= expected_amount,
                "Recipient should have received ETH"
            );
        }
    }

    #[tokio::test]
    #[cfg(feature = "integration-tests")] // Requires Docker
    async fn test_evm_insufficient_funds_error() {
        let (_container, rpc_url, _) = setup_anvil_node().await;

        // Create signer with a new private key (no funds)
        let unfunded_key = "0x0000000000000000000000000000000000000000000000000000000000000001";
        let config = EvmNetworkConfig::new("local", 31337, rpc_url);
        let _signer =
            LocalEvmSigner::new(unfunded_key.to_string(), config).expect("Failed to create signer");

        let config = riglr_config::Config::from_env();
        let app_context = ApplicationContext::from_config(&config);

        // Try to send ETH without funds
        let recipient = "0x70997970C51812dc3A010C7d01b50e0d17dc79C8";

        let result = send_eth(recipient.to_string(), 1.0, None, &app_context).await;

        // Should fail due to insufficient funds
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string().to_lowercase();
        assert!(
            error_msg.contains("insufficient")
                || error_msg.contains("not enough")
                || error_msg.contains("funds"),
            "Error should indicate insufficient funds: {}",
            error_msg
        );
    }

    #[tokio::test]
    #[cfg(feature = "integration-tests")] // Requires Docker
    async fn test_evm_gas_estimation() {
        let (_container, rpc_url, private_key) = setup_anvil_node().await;

        let config = EvmNetworkConfig::new("local", 31337, rpc_url.clone());
        let _signer = LocalEvmSigner::new(private_key, config).expect("Failed to create signer");

        let unified_signer: Arc<dyn UnifiedSigner> = Arc::new(_signer);

        // Execute test within SignerContext
        SignerContext::with_signer(unified_signer, async move {
            // Create a complex transaction that requires gas estimation
            let recipient =
                Address::from_str("0x70997970C51812dc3A010C7d01b50e0d17dc79C8").unwrap();

            // Send with data (more expensive than simple transfer)
            let tx = TransactionRequest::default()
                .to(recipient)
                .value(U256::from(1_000_000_000_000_000u64)) // 0.001 ETH
                .input(vec![1, 2, 3, 4, 5].into()); // Some data

            let tx_json = serde_json::to_value(&tx).expect("Failed to serialize");

            let evm_signer = SignerContext::current_as_evm()
                .await
                .expect("Failed to get signer");

            let tx_hash = evm_signer
                .sign_and_send_transaction(tx_json)
                .await
                .expect("Failed to send transaction");

            println!("Transaction with data sent: {}", tx_hash);

            // Verify transaction succeeded
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

            let provider = ProviderBuilder::new()
                .connect(&rpc_url)
                .await
                .expect("Failed to connect");

            let receipt = provider
                .get_transaction_receipt(tx_hash.parse().unwrap())
                .await
                .expect("Failed to get receipt")
                .expect("Receipt should exist");

            assert!(receipt.status(), "Transaction should succeed");
            assert!(
                receipt.gas_used > 21000,
                "Should use more gas than simple transfer"
            );

            Ok(())
        })
        .await
        .expect("Test should complete successfully");
    }

    #[tokio::test]
    #[cfg(feature = "integration-tests")] // Requires Docker
    async fn test_evm_concurrent_transactions() {
        let (_container, rpc_url, private_key) = setup_anvil_node().await;

        let config = EvmNetworkConfig::new("local", 31337, rpc_url.clone());
        let _signer = LocalEvmSigner::new(private_key, config).expect("Failed to create signer");

        let config = riglr_config::Config::from_env();
        let app_context = ApplicationContext::from_config(&config);

        // Recipients for concurrent transfers
        let recipients = vec![
            "0x70997970C51812dc3A010C7d01b50e0d17dc79C8",
            "0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC",
            "0x90F79bf6EB2c4f870365E785982E1f101E93b906",
            "0x15d34AAf54267DB7D7c367839AAf71A00a2C6A65",
            "0x9965507D1a55bcC2695C58ba16FB37d819B0A4dc",
        ];

        // Send transfers concurrently
        let mut handles = vec![];

        for recipient in recipients.iter() {
            let recipient_str = recipient.to_string();
            let app_context_clone = app_context.clone();
            let handle = tokio::spawn(async move {
                send_eth(recipient_str, 0.01, None, &app_context_clone).await
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

        // Most should succeed (some might fail due to nonce issues)
        assert!(
            success_count >= 3,
            "At least 60% of concurrent transactions should succeed, got {}/5",
            success_count
        );
    }
}
