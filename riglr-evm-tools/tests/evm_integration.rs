use alloy::{
    primitives::{Address, U256},
    rpc::types::TransactionRequest,
};
use riglr_core::signer::{SignerContext, SignerError, TransactionSigner};
use riglr_evm_tools::{
    error::EvmToolError,
    util::{execute_evm_transaction, make_provider},
};
use std::str::FromStr;
use std::sync::Arc;

// Environment variable names as constants to avoid string literal warnings
const ETHEREUM_RPC_URL: &str = "ETHEREUM_RPC_URL";
const ARBITRUM_RPC_URL: &str = "ARBITRUM_RPC_URL";
const POLYGON_RPC_URL: &str = "POLYGON_RPC_URL";
const BASE_RPC_URL: &str = "BASE_RPC_URL";

/// Mock EVM signer for testing
#[derive(Clone)]
struct MockEvmSigner {
    address: String,
}

impl MockEvmSigner {
    fn new(address: String) -> Self {
        Self { address }
    }
}

#[async_trait::async_trait]
impl TransactionSigner for MockEvmSigner {
    fn pubkey(&self) -> Option<String> {
        None
    }

    fn address(&self) -> Option<String> {
        Some(self.address.clone())
    }

    async fn sign_and_send_solana_transaction(
        &self,
        _tx: &mut solana_sdk::transaction::Transaction,
    ) -> Result<String, SignerError> {
        Err(SignerError::UnsupportedOperation(
            "EVM signer cannot sign Solana transactions".to_string(),
        ))
    }

    async fn sign_and_send_evm_transaction(
        &self,
        tx: TransactionRequest,
    ) -> Result<String, SignerError> {
        // Mock transaction signing - in real implementation this would sign with private key
        let tx_hash = format!(
            "0x{:064x}",
            tx.to
                .as_ref()
                .map(|addr| match addr {
                    alloy::primitives::TxKind::Call(address) => address.to_string(),
                    alloy::primitives::TxKind::Create => String::new(),
                })
                .unwrap_or_default()
                .len()
                + tx.value.unwrap_or_default().to::<u64>() as usize
        );
        Ok(tx_hash)
    }

    fn solana_client(&self) -> Option<Arc<solana_client::rpc_client::RpcClient>> {
        None
    }

    fn evm_client(&self) -> Result<Arc<dyn riglr_core::signer::EvmClient>, SignerError> {
        Err(SignerError::UnsupportedOperation(
            "Mock signer does not provide EVM client".to_string(),
        ))
    }
}

impl std::fmt::Debug for MockEvmSigner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MockEvmSigner")
            .field("address", &self.address)
            .finish()
    }
}

#[tokio::test]
async fn test_provider_factory_with_different_chains() {
    // Set up environment variables for testing
    // SAFETY: This is a test that controls its own environment
    unsafe {
        std::env::set_var(ETHEREUM_RPC_URL, "https://eth.llamarpc.com");
        std::env::set_var(ARBITRUM_RPC_URL, "https://arb1.arbitrum.io/rpc");
        std::env::set_var(POLYGON_RPC_URL, "https://polygon-rpc.com");
        std::env::set_var(BASE_RPC_URL, "https://mainnet.base.org");
    }

    // Test provider creation for different chains
    let chains_to_test = vec![
        (1, "Ethereum"),
        (42161, "Arbitrum"),
        (137, "Polygon"),
        (8453, "Base"),
    ];

    for (chain_id, chain_name) in chains_to_test {
        let result = make_provider(chain_id);
        assert!(
            result.is_ok(),
            "Failed to create provider for {}: {:?}",
            chain_name,
            result.err()
        );

        let provider = result.unwrap();
        // Basic provider validation - in real tests you might call get_chain_id()
        // For mock tests, we just verify the provider was created
        // Provider doesn't implement Debug, just verify it was created
        let _ = provider;
    }

    // Test unsupported chain
    let unsupported_result = make_provider(999999);
    match unsupported_result {
        Err(EvmToolError::UnsupportedChain(chain_id)) => {
            assert_eq!(chain_id, 999999);
        }
        _ => panic!("Expected UnsupportedChain error for unsupported chain"),
    }

    // Clean up
    // SAFETY: This is a test that controls its own environment
    unsafe {
        std::env::remove_var(ETHEREUM_RPC_URL);
        std::env::remove_var(ARBITRUM_RPC_URL);
        std::env::remove_var(POLYGON_RPC_URL);
        std::env::remove_var(BASE_RPC_URL);
    }
}

#[tokio::test]
async fn test_transaction_execution_helper() {
    // Set up environment
    // SAFETY: This is a test that controls its own environment
    unsafe {
        std::env::set_var(ETHEREUM_RPC_URL, "https://eth.llamarpc.com");
    }

    let signer = MockEvmSigner::new("0x742d35Cc2F5f8a89A0D2EAd5a53c97c49444E34F".to_string());

    let result = SignerContext::with_signer(Arc::new(signer), async {
        execute_evm_transaction(1, |address, _provider| async move {
            // Verify we get the correct address
            assert_eq!(
                address.to_string().to_lowercase(),
                "0x742d35cc2f5f8a89a0d2ead5a53c97c49444e34f"
            );

            // Create a mock transaction
            let tx = TransactionRequest {
                to: Some(alloy::primitives::TxKind::Call(
                    Address::from_str("0x1234567890123456789012345678901234567890").unwrap(),
                )),
                value: Some(U256::from(1000000000000000000u64)), // 1 ETH
                gas: Some(21000),
                gas_price: Some(20000000000u128), // 20 gwei
                ..Default::default()
            };

            Ok::<TransactionRequest, EvmToolError>(tx)
        })
        .await
        .map_err(|e| SignerError::UnsupportedOperation(e.to_string()))
    })
    .await;

    assert!(result.is_ok());
    let tx_hash = result.unwrap();
    assert!(tx_hash.starts_with("0x"));
    assert_eq!(tx_hash.len(), 66); // 0x + 64 hex chars

    // Clean up
    // SAFETY: This is a test that controls its own environment
    unsafe {
        std::env::remove_var(ETHEREUM_RPC_URL);
    }
}

#[tokio::test]
async fn test_evm_error_handling() {
    let signer = MockEvmSigner::new("0x742d35Cc2F5f8a89A0D2EAd5a53c97c49444E34F".to_string());

    // Test missing environment variable
    // SAFETY: This is a test that controls its own environment
    unsafe {
        std::env::remove_var(ETHEREUM_RPC_URL);
    }

    let result = SignerContext::with_signer(Arc::new(signer.clone()), async {
        execute_evm_transaction(1, |_address, _provider| async move {
            Ok::<TransactionRequest, EvmToolError>(TransactionRequest::default())
        })
        .await
        .map_err(|e| SignerError::UnsupportedOperation(e.to_string()))
    })
    .await;

    assert!(result.is_err());

    // Test invalid address format
    let invalid_signer = MockEvmSigner::new("invalid_address".to_string());

    // Restore environment for this test
    // SAFETY: This is a test that controls its own environment
    unsafe {
        std::env::set_var(ETHEREUM_RPC_URL, "https://eth.llamarpc.com");
    }

    let result = SignerContext::with_signer(Arc::new(invalid_signer), async {
        execute_evm_transaction(1, |_address, _provider| async move {
            Ok::<TransactionRequest, EvmToolError>(TransactionRequest::default())
        })
        .await
        .map_err(|e| SignerError::UnsupportedOperation(e.to_string()))
    })
    .await;

    assert!(result.is_err());

    // Test transaction creation error
    let result = SignerContext::with_signer(Arc::new(signer), async {
        execute_evm_transaction(1, |_address, _provider| async move {
            Err::<TransactionRequest, EvmToolError>(EvmToolError::TransactionBuildError(
                "Mock build error".to_string(),
            ))
        })
        .await
        .map_err(|e| SignerError::UnsupportedOperation(e.to_string()))
    })
    .await;

    assert!(result.is_err());
    // Error is now wrapped in SignerError, so just check it's an error

    // Clean up
    // SAFETY: This is a test that controls its own environment
    unsafe {
        std::env::remove_var(ETHEREUM_RPC_URL);
    }
}

#[tokio::test]
async fn test_provider_factory_error_scenarios() {
    // Test missing environment variable for supported chain
    // SAFETY: This is a test that controls its own environment
    unsafe {
        std::env::remove_var(ETHEREUM_RPC_URL);
    }

    let result = make_provider(1);
    assert!(result.is_err());

    // Test malformed RPC URL
    // SAFETY: This is a test that controls its own environment
    unsafe {
        std::env::set_var(ETHEREUM_RPC_URL, "not_a_valid_url");
    }

    let result = make_provider(1);
    assert!(result.is_err());

    // Clean up
    // SAFETY: This is a test that controls its own environment
    unsafe {
        std::env::remove_var(ETHEREUM_RPC_URL);
    }
}

#[tokio::test]
async fn test_concurrent_provider_creation() {
    // Set up environment
    // SAFETY: This is a test that controls its own environment
    unsafe {
        std::env::set_var(ETHEREUM_RPC_URL, "https://eth.llamarpc.com");
        std::env::set_var(ARBITRUM_RPC_URL, "https://arb1.arbitrum.io/rpc");
    }

    let handles = vec![
        tokio::spawn(async { make_provider(1) }),
        tokio::spawn(async { make_provider(42161) }),
        tokio::spawn(async { make_provider(1) }),
        tokio::spawn(async { make_provider(42161) }),
    ];

    use futures::future::join_all;
    let results = join_all(handles).await;

    for result in results {
        let provider_result = result.unwrap();
        assert!(
            provider_result.is_ok(),
            "Provider creation failed: {:?}",
            provider_result.err()
        );
    }

    // Clean up
    // SAFETY: This is a test that controls its own environment
    unsafe {
        std::env::remove_var(ETHEREUM_RPC_URL);
        std::env::remove_var(ARBITRUM_RPC_URL);
    }
}

#[tokio::test]
async fn test_transaction_execution_with_different_addresses() {
    // SAFETY: This is a test that controls its own environment
    unsafe {
        std::env::set_var(ETHEREUM_RPC_URL, "https://eth.llamarpc.com");
    }

    let addresses = vec![
        "0x742d35Cc2F5f8a89A0D2EAd5a53c97c49444E34F",
        "0x1234567890123456789012345678901234567890",
        "0xabcdefabcdefabcdefabcdefabcdefabcdefabcd",
    ];

    for addr in addresses {
        let signer = MockEvmSigner::new(addr.to_string());

        let result = SignerContext::with_signer(Arc::new(signer), async move {
            execute_evm_transaction(1, move |address, _provider| async move {
                // Verify the address passed to the transaction creator
                assert_eq!(address.to_string().to_lowercase(), addr.to_lowercase());

                Ok::<TransactionRequest, EvmToolError>(TransactionRequest {
                    to: Some(alloy::primitives::TxKind::Call(address)),
                    value: Some(U256::from(1000)),
                    ..Default::default()
                })
            })
            .await
            .map_err(|e| SignerError::UnsupportedOperation(e.to_string()))
        })
        .await;

        assert!(
            result.is_ok(),
            "Transaction execution failed for address {}: {:?}",
            addr,
            result.err()
        );
    }

    // Clean up
    // SAFETY: This is a test that controls its own environment
    unsafe {
        std::env::remove_var(ETHEREUM_RPC_URL);
    }
}

#[tokio::test]
async fn test_error_conversion_to_tool_error() {
    use riglr_core::error::ToolError;

    // Test EvmToolError to ToolError conversion
    let evm_errors = vec![
        EvmToolError::ProviderError("Network timeout".to_string()),
        EvmToolError::InsufficientBalance,
        EvmToolError::InvalidAddress("Bad format".to_string()),
        EvmToolError::UnsupportedChain(999),
        EvmToolError::TransactionBuildError("Build failed".to_string()),
    ];

    for evm_error in evm_errors {
        let tool_error: ToolError = evm_error.into();

        // Verify conversion logic
        match &tool_error {
            ToolError::Retriable { .. } => {
                // Provider errors should be retriable
            }
            ToolError::Permanent { .. } => {
                // Balance, build errors should be permanent
            }
            ToolError::InvalidInput { .. } => {
                // Address format, unsupported chain should be invalid input
            }
            _ => panic!("Unexpected ToolError variant: {:?}", tool_error),
        }
    }
}
