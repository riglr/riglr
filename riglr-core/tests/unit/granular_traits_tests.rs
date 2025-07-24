use async_trait::async_trait;
use riglr_core::signer::granular_traits::{
    Chain, EvmSigner, MultiChainSigner, SignerBase, SolanaSigner,
    UnifiedSigner, LegacySignerAdapter,
};
use riglr_core::signer::{SignerError, TransactionSigner};
use std::sync::Arc;
use std::any::Any;
use alloy::rpc::types::TransactionRequest;
use solana_sdk::transaction::Transaction;

// Mock implementations for testing
#[derive(Debug)]
struct MockSolanaSigner {
    address: String,
}

impl SignerBase for MockSolanaSigner {
    fn locale(&self) -> String {
        "en".to_string()
    }
    
    fn user_id(&self) -> Option<String> {
        Some("user123".to_string())
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[async_trait]
impl SolanaSigner for MockSolanaSigner {
    fn address(&self) -> String {
        self.address.clone()
    }
    
    fn pubkey(&self) -> String {
        self.address.clone() // For testing purposes
    }
    
    async fn sign_and_send_transaction(
        &self,
        _tx: &mut Transaction,
    ) -> Result<String, SignerError> {
        Ok("solana_tx_hash".to_string())
    }
    
    fn client(&self) -> Arc<solana_client::rpc_client::RpcClient> {
        Arc::new(solana_client::rpc_client::RpcClient::new("https://api.devnet.solana.com".to_string()))
    }
}

#[derive(Debug)]
struct MockEvmSigner {
    chain_id: u64,
    address: String,
}

impl SignerBase for MockEvmSigner {
    fn locale(&self) -> String {
        "fr".to_string()
    }
    
    fn user_id(&self) -> Option<String> {
        None
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[async_trait]
impl EvmSigner for MockEvmSigner {
    fn chain_id(&self) -> u64 {
        self.chain_id
    }
    
    fn address(&self) -> String {
        self.address.clone()
    }
    
    async fn sign_and_send_transaction(
        &self,
        _tx: TransactionRequest,
    ) -> Result<String, SignerError> {
        Ok("0xevmtxhash".to_string())
    }
    
    fn client(&self) -> Result<Arc<dyn riglr_core::signer::EvmClient>, SignerError> {
        Err(SignerError::UnsupportedOperation("Mock client".to_string()))
    }
}

#[derive(Debug)]
struct MockMultiChainSigner {
    primary: Chain,
}

impl SignerBase for MockMultiChainSigner {
    fn locale(&self) -> String {
        "ja".to_string()
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[async_trait]
impl SolanaSigner for MockMultiChainSigner {
    fn address(&self) -> String {
        "SolanaAddr123".to_string()
    }
    
    fn pubkey(&self) -> String {
        "SolanaPub123".to_string()
    }
    
    async fn sign_and_send_transaction(
        &self,
        _tx: &mut Transaction,
    ) -> Result<String, SignerError> {
        Ok("sol_multi_tx".to_string())
    }
    
    fn client(&self) -> Arc<solana_client::rpc_client::RpcClient> {
        Arc::new(solana_client::rpc_client::RpcClient::new("https://api.devnet.solana.com".to_string()))
    }
}

#[async_trait]
impl EvmSigner for MockMultiChainSigner {
    fn chain_id(&self) -> u64 {
        1
    }
    
    fn address(&self) -> String {
        "0xEvmAddr123".to_string()
    }
    
    async fn sign_and_send_transaction(
        &self,
        _tx: TransactionRequest,
    ) -> Result<String, SignerError> {
        Ok("0xevmmultitx".to_string())
    }
    
    fn client(&self) -> Result<Arc<dyn riglr_core::signer::EvmClient>, SignerError> {
        Err(SignerError::UnsupportedOperation("Mock client".to_string()))
    }
}

#[async_trait]
impl MultiChainSigner for MockMultiChainSigner {
    fn primary_chain(&self) -> Chain {
        self.primary.clone()
    }
    
    fn supports_chain(&self, chain: &Chain) -> bool {
        match chain {
            Chain::Solana => true,
            Chain::Evm { chain_id } => *chain_id == 1,
        }
    }
    
    fn as_solana(&self) -> Option<&dyn SolanaSigner> {
        if matches!(self.primary, Chain::Solana) {
            Some(self)
        } else {
            None
        }
    }
    
    fn as_evm(&self) -> Option<&dyn EvmSigner> {
        if matches!(self.primary, Chain::Evm { .. }) {
            Some(self)
        } else {
            None
        }
    }
}

// Mock TransactionSigner for adapter testing
#[derive(Debug)]
struct MockTransactionSigner;

#[async_trait]
impl TransactionSigner for MockTransactionSigner {
    async fn sign_and_send_solana_transaction(
        &self,
        _tx: &mut Transaction,
    ) -> Result<String, SignerError> {
        Ok("tx_sig".to_string())
    }
    
    async fn sign_and_send_evm_transaction(
        &self,
        _tx: TransactionRequest,
    ) -> Result<String, SignerError> {
        Ok("0xtxhash".to_string())
    }
    
    fn solana_client(&self) -> Option<Arc<solana_client::rpc_client::RpcClient>> {
        Some(Arc::new(solana_client::rpc_client::RpcClient::new("https://api.devnet.solana.com".to_string())))
    }
    
    fn evm_client(&self) -> Result<Arc<dyn riglr_core::signer::EvmClient>, SignerError> {
        Err(SignerError::UnsupportedOperation("Mock".to_string()))
    }
    
    fn pubkey(&self) -> Option<String> {
        Some("MockPubkey".to_string())
    }
    
    fn chain_id(&self) -> Option<u64> {
        None
    }
}

#[test]
fn test_signer_base_defaults() {
    let signer = MockSolanaSigner {
        address: "test".to_string(),
    };
    
    // Test default locale
    assert_eq!(signer.locale(), "en");
    
    // Test overridden user_id
    assert_eq!(signer.user_id(), Some("user123".to_string()));
    
    // Test another signer with different values
    let evm_signer = MockEvmSigner {
        chain_id: 1,
        address: "0x123".to_string(),
    };
    
    assert_eq!(evm_signer.locale(), "fr");
    assert_eq!(evm_signer.user_id(), None);
}

#[tokio::test]
async fn test_solana_signer() {
    let signer = MockSolanaSigner {
        address: "SolanaAddress123".to_string(),
    };
    
    assert_eq!(signer.address(), "SolanaAddress123");
    assert_eq!(signer.pubkey(), "SolanaAddress123");
    
    let mut tx = Transaction::default();
    let result = signer.sign_and_send_transaction(&mut tx).await;
    assert_eq!(result.unwrap(), "solana_tx_hash");
    
    // Test default retry implementation
    let retry_result = signer.sign_and_send_with_retry(&mut tx).await;
    assert_eq!(retry_result.unwrap(), "solana_tx_hash");
    
    // Test client
    let _client = signer.client();
}

#[tokio::test]
async fn test_evm_signer() {
    let signer = MockEvmSigner {
        chain_id: 137,
        address: "0xEvmAddress456".to_string(),
    };
    
    assert_eq!(signer.chain_id(), 137);
    assert_eq!(signer.address(), "0xEvmAddress456");
    
    let tx = TransactionRequest::default();
    let result = signer.sign_and_send_transaction(tx.clone()).await;
    assert_eq!(result.unwrap(), "0xevmtxhash");
    
    // Test default retry implementation
    let retry_result = signer.sign_and_send_with_retry(tx).await;
    assert_eq!(retry_result.unwrap(), "0xevmtxhash");
    
    // Test client error
    let client_result = signer.client();
    assert!(matches!(client_result, Err(SignerError::UnsupportedOperation(_))));
}

#[tokio::test]
async fn test_multi_chain_signer() {
    let signer = MockMultiChainSigner {
        primary: Chain::Solana,
    };
    
    // Test MultiChainSigner specific methods
    assert_eq!(signer.primary_chain(), Chain::Solana);
    assert!(signer.supports_chain(&Chain::Solana));
    assert!(signer.supports_chain(&Chain::Evm { chain_id: 1 }));
    assert!(!signer.supports_chain(&Chain::Evm { chain_id: 137 }));
    
    // Test SolanaSigner methods  
    let solana_addr = <MockMultiChainSigner as SolanaSigner>::address(&signer);
    assert_eq!(solana_addr, "SolanaAddr123");
    let solana_pubkey = <MockMultiChainSigner as SolanaSigner>::pubkey(&signer);
    assert_eq!(solana_pubkey, "SolanaPub123");
    
    let mut sol_tx = Transaction::default();
    let sol_result = <MockMultiChainSigner as SolanaSigner>::sign_and_send_transaction(&signer, &mut sol_tx).await;
    assert_eq!(sol_result.unwrap(), "sol_multi_tx");
    
    // Test EvmSigner methods through MultiChainSigner
    let evm_signer: &dyn EvmSigner = &signer;
    assert_eq!(evm_signer.chain_id(), 1);
    let evm_addr = <MockMultiChainSigner as EvmSigner>::address(&signer);
    assert_eq!(evm_addr, "0xEvmAddr123");
    
    let evm_tx = TransactionRequest::default();
    let evm_result = evm_signer.sign_and_send_transaction(evm_tx).await;
    assert_eq!(evm_result.unwrap(), "0xevmmultitx");
}

#[test]
fn test_chain_enum() {
    let solana_chain = Chain::Solana;
    let evm_chain = Chain::Evm { chain_id: 42161 };
    
    // Test Debug trait
    let debug_str = format!("{:?}", solana_chain);
    assert!(debug_str.contains("Solana"));
    
    let debug_str = format!("{:?}", evm_chain);
    assert!(debug_str.contains("Evm"));
    assert!(debug_str.contains("42161"));
    
    // Test Clone trait
    let cloned_solana = solana_chain.clone();
    assert_eq!(cloned_solana, Chain::Solana);
    
    let cloned_evm = evm_chain.clone();
    assert_eq!(cloned_evm, Chain::Evm { chain_id: 42161 });
    
    // Test PartialEq and Eq
    assert_eq!(Chain::Solana, Chain::Solana);
    assert_ne!(Chain::Solana, Chain::Evm { chain_id: 1 });
    assert_eq!(Chain::Evm { chain_id: 1 }, Chain::Evm { chain_id: 1 });
    assert_ne!(Chain::Evm { chain_id: 1 }, Chain::Evm { chain_id: 2 });
}

#[test]
fn test_legacy_signer_adapter() {
    let mock_signer = Arc::new(MockTransactionSigner);
    let adapter = LegacySignerAdapter::new(mock_signer.clone());
    
    // Test SignerBase implementation
    assert_eq!(adapter.locale(), "en");
    assert_eq!(adapter.user_id(), None);
    
    // Test Debug trait
    let debug_str = format!("{:?}", adapter);
    assert!(debug_str.contains("MockTransactionSigner"));
}

#[test]
fn test_unified_signer_with_legacy_adapter() {
    let signer = Arc::new(MockTransactionSigner);
    let adapter = LegacySignerAdapter::new(signer);
    
    // Test UnifiedSigner implementation
    // MockTransactionSigner has pubkey() returning Some, so it supports Solana
    assert!(adapter.supports_solana());
    // MockTransactionSigner has no chain_id, so it doesn't support EVM  
    assert!(!adapter.supports_evm());
    
    // Test as_solana and as_evm
    // Legacy adapters return None for these (as they don't implement granular traits directly)
    assert!(adapter.as_solana().is_none());
    assert!(adapter.as_evm().is_none());
}

#[test]
fn test_multi_chain_with_evm_primary() {
    let signer = MockMultiChainSigner {
        primary: Chain::Evm { chain_id: 1 },
    };
    
    assert_eq!(signer.primary_chain(), Chain::Evm { chain_id: 1 });
    assert!(signer.supports_chain(&Chain::Solana));
    assert!(signer.supports_chain(&Chain::Evm { chain_id: 1 }));
}

#[tokio::test]
async fn test_solana_signer_with_retry() {
    // Create a custom signer that implements retry logic
    #[derive(Debug)]
    struct RetrySolanaSigner {
        retry_count: std::sync::atomic::AtomicUsize,
    }
    
    impl SignerBase for RetrySolanaSigner {
        fn as_any(&self) -> &dyn Any {
            self
        }
    }
    
    #[async_trait]
    impl SolanaSigner for RetrySolanaSigner {
        fn address(&self) -> String {
            "retry_addr".to_string()
        }
        
        fn pubkey(&self) -> String {
            "retry_pub".to_string()
        }
        
        async fn sign_and_send_transaction(
            &self,
            _tx: &mut Transaction,
        ) -> Result<String, SignerError> {
            self.retry_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            Ok("regular_tx".to_string())
        }
        
        async fn sign_and_send_with_retry(
            &self,
            tx: &mut Transaction,
        ) -> Result<String, SignerError> {
            // Custom retry logic
            for _ in 0..3 {
                if let Ok(result) = self.sign_and_send_transaction(tx).await {
                    return Ok(format!("retry_{}", result));
                }
            }
            Err(SignerError::TransactionFailed("Failed after retries".to_string()))
        }
        
        fn client(&self) -> Arc<solana_client::rpc_client::RpcClient> {
            Arc::new(solana_client::rpc_client::RpcClient::new("https://api.devnet.solana.com".to_string()))
        }
    }
    
    let signer = RetrySolanaSigner {
        retry_count: std::sync::atomic::AtomicUsize::new(0),
    };
    
    let mut tx = Transaction::default();
    let result = signer.sign_and_send_with_retry(&mut tx).await;
    assert_eq!(result.unwrap(), "retry_regular_tx");
}

#[tokio::test]
async fn test_evm_signer_with_retry() {
    // Create a custom signer that implements retry logic
    #[derive(Debug)]
    struct RetryEvmSigner {
        attempt_count: std::sync::atomic::AtomicUsize,
    }
    
    impl SignerBase for RetryEvmSigner {
        fn as_any(&self) -> &dyn Any {
            self
        }
    }
    
    #[async_trait]
    impl EvmSigner for RetryEvmSigner {
        fn chain_id(&self) -> u64 {
            1
        }
        
        fn address(&self) -> String {
            "0xretry".to_string()
        }
        
        async fn sign_and_send_transaction(
            &self,
            _tx: TransactionRequest,
        ) -> Result<String, SignerError> {
            let count = self.attempt_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            if count < 2 {
                Err(SignerError::TransactionFailed("Temporary failure".to_string()))
            } else {
                Ok("0xsuccess".to_string())
            }
        }
        
        async fn sign_and_send_with_retry(
            &self,
            tx: TransactionRequest,
        ) -> Result<String, SignerError> {
            // Custom retry logic with exponential backoff simulation
            for i in 0..3 {
                match self.sign_and_send_transaction(tx.clone()).await {
                    Ok(result) => return Ok(format!("retry_{}_{}", i, result)),
                    Err(_) if i < 2 => continue,
                    Err(e) => return Err(e),
                }
            }
            unreachable!()
        }
        
        fn client(&self) -> Result<Arc<dyn riglr_core::signer::EvmClient>, SignerError> {
            Err(SignerError::UnsupportedOperation("Mock".to_string()))
        }
    }
    
    let signer = RetryEvmSigner {
        attempt_count: std::sync::atomic::AtomicUsize::new(0),
    };
    
    let tx = TransactionRequest::default();
    let result = signer.sign_and_send_with_retry(tx).await;
    assert_eq!(result.unwrap(), "retry_2_0xsuccess");
}