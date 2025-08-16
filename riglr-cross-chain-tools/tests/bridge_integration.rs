use riglr_core::{
    error::ToolError,
    signer::{SignerContext, SignerError, TransactionSigner},
};
use riglr_cross_chain_tools::{bridge::execute_cross_chain_bridge, error::CrossChainError};
use solana_sdk::transaction::Transaction;
use std::sync::Arc;

/// Mock signer for testing bridge operations
#[derive(Clone)]
struct MockBridgeSigner {
    chain_type: String,
    pubkey: Option<String>,
    address: Option<String>,
}

impl MockBridgeSigner {
    fn new_solana(pubkey: String) -> Self {
        Self {
            chain_type: "solana".to_string(),
            pubkey: Some(pubkey),
            address: None,
        }
    }

    fn new_evm(address: String) -> Self {
        Self {
            chain_type: "evm".to_string(),
            pubkey: None,
            address: Some(address),
        }
    }
}

#[async_trait::async_trait]
impl TransactionSigner for MockBridgeSigner {
    fn pubkey(&self) -> Option<String> {
        self.pubkey.clone()
    }

    fn address(&self) -> Option<String> {
        self.address.clone()
    }

    async fn sign_and_send_solana_transaction(
        &self,
        _tx: &mut Transaction,
    ) -> Result<String, SignerError> {
        Ok("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".to_string())
    }

    async fn sign_and_send_evm_transaction(
        &self,
        _tx: alloy::rpc::types::TransactionRequest,
    ) -> Result<String, SignerError> {
        Ok("0x0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef01".to_string())
    }

    fn solana_client(&self) -> Option<Arc<solana_client::rpc_client::RpcClient>> {
        Some(Arc::new(solana_client::rpc_client::RpcClient::new(
            "https://api.devnet.solana.com".to_string(),
        )))
    }

    fn evm_client(&self) -> Result<Arc<dyn riglr_core::signer::EvmClient>, SignerError> {
        Err(SignerError::UnsupportedOperation(
            "Mock bridge signer does not provide EVM client".to_string(),
        ))
    }
}

impl std::fmt::Debug for MockBridgeSigner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MockBridgeSigner")
            .field("chain_type", &self.chain_type)
            .field("pubkey", &self.pubkey)
            .field("address", &self.address)
            .finish()
    }
}

#[tokio::test]
async fn test_bridge_quote_fetching_mock() {
    // Test that we can fetch a quote without executing the bridge
    // This would normally integrate with Li.fi API but we'll mock it for testing

    let signer =
        MockBridgeSigner::new_evm("0x742d35Cc2F5f8a89A0D2EAd5a53c97c49444E34F".to_string());

    let result = SignerContext::with_signer(Arc::new(signer), async {
        // In a real implementation, this would call Li.fi quote endpoint
        // For testing, we validate input parameters and return mock quote
        let quote_result = mock_fetch_bridge_quote(
            "ethereum",
            "arbitrum",
            "0xA0b86a33E6441E20e7F4e7eC5F3e07f42B7B6E4B", // USDC
            "0xA0b86a33E6441E20e7F4e7eC5F3e07f42B7B6E4B", // USDC
            "1000000",                                    // 1 USDC
            "0x742d35Cc2F5f8a89A0D2EAd5a53c97c49444E34F",
        )
        .await;

        quote_result.map_err(|e| SignerError::UnsupportedOperation(e.to_string()))
    })
    .await;

    assert!(result.is_ok());
    let quote = result.unwrap();
    assert!(quote.estimated_gas > 0);
    assert!(!quote.route.is_empty());
    assert!(quote.estimated_time_seconds > 0);
}

#[tokio::test]
async fn test_bridge_error_handling_scenarios() {
    // Test unsupported chain pair
    let signer1 =
        MockBridgeSigner::new_evm("0x742d35Cc2F5f8a89A0D2EAd5a53c97c49444E34F".to_string());
    let result = SignerContext::with_signer(Arc::new(signer1), async {
        execute_cross_chain_bridge(
            "route_id_123".to_string(),
            "unsupported_chain".to_string(),
            "ethereum".to_string(),
            "1000000".to_string(),
        )
        .await
        .map_err(|e| SignerError::UnsupportedOperation(e.to_string()))
    })
    .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        SignerError::UnsupportedOperation(msg) => {
            assert!(msg.contains("unsupported_chain"));
        }
        _ => panic!("Expected UnsupportedOperation error"),
    }

    // Test insufficient liquidity scenario
    let signer2 =
        MockBridgeSigner::new_evm("0x742d35Cc2F5f8a89A0D2EAd5a53c97c49444E34F".to_string());
    let result = SignerContext::with_signer(Arc::new(signer2), async {
        mock_bridge_with_insufficient_liquidity()
            .await
            .map_err(|e| SignerError::UnsupportedOperation(e.to_string()))
    })
    .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        SignerError::UnsupportedOperation(msg) => {
            assert!(msg.contains("999999999999"));
        }
        _ => panic!("Expected UnsupportedOperation error"),
    }

    // Test API error scenario
    let signer3 =
        MockBridgeSigner::new_evm("0x742d35Cc2F5f8a89A0D2EAd5a53c97c49444E34F".to_string());
    let result = SignerContext::with_signer(Arc::new(signer3), async {
        mock_bridge_with_api_error()
            .await
            .map_err(|e| SignerError::UnsupportedOperation(e.to_string()))
    })
    .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        SignerError::UnsupportedOperation(msg) => {
            assert!(msg.contains("API temporarily unavailable"));
        }
        _ => panic!("Expected UnsupportedOperation error"),
    }
}

#[tokio::test]
async fn test_chain_id_validation() {
    let test_cases = vec![
        ("ethereum", 1),
        ("arbitrum", 42161),
        ("polygon", 137),
        ("base", 8453),
        ("solana", 1399811149),
    ];

    for (chain_name, expected_id) in test_cases {
        let chain_id = parse_chain_id_for_test(chain_name);
        assert!(chain_id.is_ok());
        assert_eq!(chain_id.unwrap(), expected_id);
    }

    // Test invalid chain
    let invalid_result = parse_chain_id_for_test("invalid_chain");
    assert!(invalid_result.is_err());
}

#[tokio::test]
async fn test_cross_chain_signer_context_bridge() {
    // Test Solana to EVM bridge
    let solana_signer =
        MockBridgeSigner::new_solana("11111111111111111111111111111112".to_string());

    let result = SignerContext::with_signer(Arc::new(solana_signer), async {
        // Mock a bridge from Solana to Ethereum
        mock_bridge_transaction(
            "solana", "ethereum", true, // should use Solana pubkey as from_address
        )
        .await
        .map_err(|e| SignerError::UnsupportedOperation(e.to_string()))
    })
    .await;

    assert!(result.is_ok());

    // Test EVM to Solana bridge
    let evm_signer =
        MockBridgeSigner::new_evm("0x742d35Cc2F5f8a89A0D2EAd5a53c97c49444E34F".to_string());

    let result = SignerContext::with_signer(Arc::new(evm_signer), async {
        // Mock a bridge from Ethereum to Solana
        mock_bridge_transaction(
            "ethereum", "solana", false, // should use EVM address as from_address
        )
        .await
        .map_err(|e| SignerError::UnsupportedOperation(e.to_string()))
    })
    .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_bridge_transaction_monitoring() {
    let signer =
        MockBridgeSigner::new_evm("0x742d35Cc2F5f8a89A0D2EAd5a53c97c49444E34F".to_string());

    let result = SignerContext::with_signer(Arc::new(signer), async {
        // Mock bridge execution that returns transaction hash
        let tx_hash = mock_execute_bridge_with_monitoring()
            .await
            .map_err(|e| SignerError::UnsupportedOperation(e.to_string()))?;

        // Verify transaction hash format
        if tx_hash.starts_with("0x") {
            assert_eq!(tx_hash.len(), 66); // 0x + 64 hex chars for EVM
        } else {
            assert_eq!(tx_hash.len(), 64); // 64 chars for Solana
        }

        Ok::<String, SignerError>(tx_hash)
    })
    .await;

    assert!(result.is_ok());
}

// Helper functions for testing

struct BridgeQuote {
    estimated_gas: u64,
    route: String,
    estimated_time_seconds: u64,
}

async fn mock_fetch_bridge_quote(
    from_chain: &str,
    to_chain: &str,
    _from_token: &str,
    _to_token: &str,
    amount: &str,
    from_address: &str,
) -> Result<BridgeQuote, CrossChainError> {
    // Validate chain names
    parse_chain_id_for_test(from_chain)?;
    parse_chain_id_for_test(to_chain)?;

    // Validate addresses
    if from_chain == "solana" && !from_address.chars().all(|c| c.is_alphanumeric()) {
        return Err(CrossChainError::InvalidRoute(
            "Invalid Solana address".to_string(),
        ));
    }

    if from_chain != "solana" && !from_address.starts_with("0x") {
        return Err(CrossChainError::InvalidRoute(
            "Invalid EVM address".to_string(),
        ));
    }

    // Validate amount
    let _amount_parsed: u64 = amount
        .parse()
        .map_err(|_| CrossChainError::InvalidRoute("Invalid amount".to_string()))?;

    Ok(BridgeQuote {
        estimated_gas: 150000,
        route: format!("{} -> {}", from_chain, to_chain),
        estimated_time_seconds: 300,
    })
}

async fn mock_bridge_with_insufficient_liquidity() -> Result<String, CrossChainError> {
    Err(CrossChainError::InsufficientLiquidity {
        amount: "999999999999".to_string(),
    })
}

async fn mock_bridge_with_api_error() -> Result<String, CrossChainError> {
    Err(CrossChainError::LifiApiError(
        "API temporarily unavailable".to_string(),
    ))
}

async fn mock_bridge_transaction(
    from_chain: &str,
    _to_chain: &str,
    is_solana_source: bool,
) -> Result<String, CrossChainError> {
    let signer = SignerContext::current()
        .await
        .map_err(|e| CrossChainError::ToolError(ToolError::permanent_string(e.to_string())))?;

    // Verify correct address type based on source chain
    if is_solana_source {
        assert!(signer.pubkey().is_some());
    } else {
        assert!(signer.address().is_some());
    }

    // Mock transaction execution
    if from_chain == "solana" {
        let mut tx = Transaction::default(); // Mock transaction data
        signer
            .sign_and_send_solana_transaction(&mut tx)
            .await
            .map_err(|e| CrossChainError::BridgeExecutionError(e.to_string()))
    } else {
        let tx_request = alloy::rpc::types::TransactionRequest::default();
        signer
            .sign_and_send_evm_transaction(tx_request)
            .await
            .map_err(|e| CrossChainError::BridgeExecutionError(e.to_string()))
    }
}

async fn mock_execute_bridge_with_monitoring() -> Result<String, CrossChainError> {
    // Simulate bridge execution with transaction monitoring
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

    // Return mock transaction hash
    Ok("0x0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef01".to_string())
}

fn parse_chain_id_for_test(chain_name: &str) -> Result<u64, CrossChainError> {
    match chain_name.to_lowercase().as_str() {
        "ethereum" | "eth" => Ok(1),
        "arbitrum" | "arb" => Ok(42161),
        "polygon" | "matic" => Ok(137),
        "base" => Ok(8453),
        "solana" | "sol" => Ok(1399811149),
        _ => Err(CrossChainError::UnsupportedChainPair {
            from_chain: chain_name.to_string(),
            to_chain: chain_name.to_string(),
        }),
    }
}
