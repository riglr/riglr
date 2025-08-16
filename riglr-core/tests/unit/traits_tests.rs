use async_trait::async_trait;
use riglr_core::signer::{SignerError, TransactionSigner, EvmClient, SolanaClient};
use std::sync::Arc;
use alloy::primitives::{U256, Bytes, TxHash};
use alloy::rpc::types::TransactionRequest;
use solana_sdk::{pubkey::Pubkey, transaction::Transaction, signature::Signature};
use std::str::FromStr;

// Mock TransactionSigner implementation for testing
#[derive(Debug)]
struct MockTransactionSigner {
    locale: String,
    user_id: Option<String>,
    chain_id: Option<u64>,
    address: Option<String>,
    pubkey: Option<String>,
    should_fail_solana: bool,
    should_fail_evm: bool,
}

impl Default for MockTransactionSigner {
    fn default() -> Self {
        Self {
            locale: "en".to_string(),
            user_id: None,
            chain_id: None,
            address: None,
            pubkey: None,
            should_fail_solana: false,
            should_fail_evm: false,
        }
    }
}

#[async_trait]
impl TransactionSigner for MockTransactionSigner {
    fn locale(&self) -> String {
        self.locale.clone()
    }

    fn user_id(&self) -> Option<String> {
        self.user_id.clone()
    }

    fn chain_id(&self) -> Option<u64> {
        self.chain_id
    }

    fn address(&self) -> Option<String> {
        self.address.clone()
    }

    fn pubkey(&self) -> Option<String> {
        self.pubkey.clone()
    }

    async fn sign_and_send_solana_transaction(
        &self,
        _tx: &mut Transaction,
    ) -> Result<String, SignerError> {
        if self.should_fail_solana {
            Err(SignerError::TransactionFailed("Solana tx failed".to_string()))
        } else {
            Ok("solana_signature_123".to_string())
        }
    }

    async fn sign_and_send_evm_transaction(
        &self,
        _tx: TransactionRequest,
    ) -> Result<String, SignerError> {
        if self.should_fail_evm {
            Err(SignerError::TransactionFailed("EVM tx failed".to_string()))
        } else {
            Ok("0xevm_tx_hash_456".to_string())
        }
    }

    fn solana_client(&self) -> Option<Arc<solana_client::rpc_client::RpcClient>> {
        if self.pubkey.is_some() {
            Some(Arc::new(solana_client::rpc_client::RpcClient::new(
                "https://api.devnet.solana.com".to_string()
            )))
        } else {
            None
        }
    }

    fn evm_client(&self) -> Result<Arc<dyn EvmClient>, SignerError> {
        if self.chain_id.is_some() {
            Ok(Arc::new(MockEvmClient))
        } else {
            Err(SignerError::UnsupportedOperation("No EVM support".to_string()))
        }
    }
}

// Mock EVM client implementation
#[derive(Debug)]
struct MockEvmClient;

#[async_trait]
impl EvmClient for MockEvmClient {
    async fn get_balance(&self, _address: &str) -> Result<U256, SignerError> {
        Ok(U256::from(1000000000000000000u64)) // 1 ETH in wei
    }

    async fn send_transaction(&self, _tx: &TransactionRequest) -> Result<TxHash, SignerError> {
        Ok(TxHash::from_str("0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef").unwrap())
    }

    async fn call(&self, _tx: &TransactionRequest) -> Result<Bytes, SignerError> {
        Ok(Bytes::from(vec![0x01, 0x02, 0x03, 0x04]))
    }
}

// Mock Solana client implementation
#[derive(Debug)]
struct MockSolanaClient;

#[async_trait]
impl SolanaClient for MockSolanaClient {
    async fn get_balance(&self, _pubkey: &Pubkey) -> Result<u64, SignerError> {
        Ok(1000000000) // 1 SOL in lamports
    }

    async fn send_transaction(&self, _tx: &Transaction) -> Result<Signature, SignerError> {
        Ok(Signature::default())
    }
}

#[test]
fn test_transaction_signer_defaults() {
    let signer = MockTransactionSigner::default();

    // Test default locale
    assert_eq!(signer.locale(), "en");

    // Test default user_id
    assert_eq!(signer.user_id(), None);

    // Test default chain_id
    assert_eq!(signer.chain_id(), None);

    // Test default address
    assert_eq!(signer.address(), None);

    // Test default pubkey
    assert_eq!(signer.pubkey(), None);
}

#[test]
fn test_transaction_signer_with_custom_values() {
    let signer = MockTransactionSigner {
        locale: "fr".to_string(),
        user_id: Some("user_123".to_string()),
        chain_id: Some(1),
        address: Some("0xAddress123".to_string()),
        pubkey: Some("SolanaPubkey456".to_string()),
        ..Default::default()
    };

    assert_eq!(signer.locale(), "fr");
    assert_eq!(signer.user_id(), Some("user_123".to_string()));
    assert_eq!(signer.chain_id(), Some(1));
    assert_eq!(signer.address(), Some("0xAddress123".to_string()));
    assert_eq!(signer.pubkey(), Some("SolanaPubkey456".to_string()));
}

#[tokio::test]
async fn test_sign_and_send_solana_transaction() {
    let signer = MockTransactionSigner::default();
    let mut tx = Transaction::default();

    let result = signer.sign_and_send_solana_transaction(&mut tx).await;
    assert_eq!(result.unwrap(), "solana_signature_123");
}

#[tokio::test]
async fn test_sign_and_send_solana_transaction_failure() {
    let signer = MockTransactionSigner {
        should_fail_solana: true,
        ..Default::default()
    };
    let mut tx = Transaction::default();

    let result = signer.sign_and_send_solana_transaction(&mut tx).await;
    assert!(matches!(result, Err(SignerError::TransactionFailed(_))));
}

#[tokio::test]
async fn test_sign_and_send_evm_transaction() {
    let signer = MockTransactionSigner::default();
    let tx = TransactionRequest::default();

    let result = signer.sign_and_send_evm_transaction(tx).await;
    assert_eq!(result.unwrap(), "0xevm_tx_hash_456");
}

#[tokio::test]
async fn test_sign_and_send_evm_transaction_failure() {
    let signer = MockTransactionSigner {
        should_fail_evm: true,
        ..Default::default()
    };
    let tx = TransactionRequest::default();

    let result = signer.sign_and_send_evm_transaction(tx).await;
    assert!(matches!(result, Err(SignerError::TransactionFailed(_))));
}

#[tokio::test]
async fn test_sign_and_send_solana_with_retry_default() {
    let signer = MockTransactionSigner::default();
    let mut tx = Transaction::default();

    // Test default implementation (should just call regular method)
    let result = signer.sign_and_send_solana_with_retry(&mut tx).await;
    assert_eq!(result.unwrap(), "solana_signature_123");
}

#[tokio::test]
async fn test_sign_and_send_evm_with_retry_default() {
    let signer = MockTransactionSigner::default();
    let tx = TransactionRequest::default();

    // Test default implementation (should just call regular method)
    let result = signer.sign_and_send_evm_with_retry(tx).await;
    assert_eq!(result.unwrap(), "0xevm_tx_hash_456");
}

#[test]
fn test_solana_client_presence() {
    // Test with Solana support
    let signer_with_solana = MockTransactionSigner {
        pubkey: Some("SolanaPubkey".to_string()),
        ..Default::default()
    };
    assert!(signer_with_solana.solana_client().is_some());

    // Test without Solana support
    let signer_without_solana = MockTransactionSigner::default();
    assert!(signer_without_solana.solana_client().is_none());
}

#[test]
fn test_evm_client_presence() {
    // Test with EVM support
    let signer_with_evm = MockTransactionSigner {
        chain_id: Some(1),
        ..Default::default()
    };
    assert!(signer_with_evm.evm_client().is_ok());

    // Test without EVM support
    let signer_without_evm = MockTransactionSigner::default();
    let result = signer_without_evm.evm_client();
    assert!(matches!(result, Err(SignerError::UnsupportedOperation(_))));
}

#[tokio::test]
async fn test_evm_client_get_balance() {
    let client = MockEvmClient;
    let balance = client.get_balance("0xAddress").await.unwrap();
    assert_eq!(balance, U256::from(1000000000000000000u64));
}

#[tokio::test]
async fn test_evm_client_send_transaction() {
    let client = MockEvmClient;
    let tx = TransactionRequest::default();
    let tx_hash = client.send_transaction(&tx).await.unwrap();
    assert_eq!(
        tx_hash.to_string(),
        "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
    );
}

#[tokio::test]
async fn test_evm_client_call() {
    let client = MockEvmClient;
    let tx = TransactionRequest::default();
    let result = client.call(&tx).await.unwrap();
    assert_eq!(result.to_vec(), vec![0x01, 0x02, 0x03, 0x04]);
}

#[tokio::test]
async fn test_solana_client_get_balance() {
    let client = MockSolanaClient;
    let pubkey = Pubkey::default();
    let balance = client.get_balance(&pubkey).await.unwrap();
    assert_eq!(balance, 1000000000);
}

#[tokio::test]
async fn test_solana_client_send_transaction() {
    let client = MockSolanaClient;
    let tx = Transaction::default();
    let signature = client.send_transaction(&tx).await.unwrap();
    assert_eq!(signature, Signature::default());
}

#[test]
fn test_transaction_signer_debug_trait() {
    let signer = MockTransactionSigner {
        locale: "ja".to_string(),
        user_id: Some("debug_user".to_string()),
        ..Default::default()
    };

    let debug_str = format!("{:?}", signer);
    assert!(debug_str.contains("MockTransactionSigner"));
    assert!(debug_str.contains("ja"));
    assert!(debug_str.contains("debug_user"));
}

// Test custom retry implementation
#[derive(Debug)]
struct RetryTransactionSigner {
    retry_count: std::sync::atomic::AtomicUsize,
}

#[async_trait]
impl TransactionSigner for RetryTransactionSigner {
    async fn sign_and_send_solana_transaction(
        &self,
        _tx: &mut Transaction,
    ) -> Result<String, SignerError> {
        self.retry_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        Ok("base_solana_sig".to_string())
    }

    async fn sign_and_send_evm_transaction(
        &self,
        _tx: TransactionRequest,
    ) -> Result<String, SignerError> {
        self.retry_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        Ok("0xbase_evm_tx".to_string())
    }

    async fn sign_and_send_solana_with_retry(
        &self,
        tx: &mut Transaction,
    ) -> Result<String, SignerError> {
        // Custom retry implementation
        for i in 0..3 {
            match self.sign_and_send_solana_transaction(tx).await {
                Ok(sig) => return Ok(format!("retry_{}__{}", i, sig)),
                Err(_) if i < 2 => continue,
                Err(e) => return Err(e),
            }
        }
        unreachable!()
    }

    async fn sign_and_send_evm_with_retry(
        &self,
        tx: TransactionRequest,
    ) -> Result<String, SignerError> {
        // Custom retry implementation
        for i in 0..3 {
            match self.sign_and_send_evm_transaction(tx.clone()).await {
                Ok(hash) => return Ok(format!("retry_{}__{}", i, hash)),
                Err(_) if i < 2 => continue,
                Err(e) => return Err(e),
            }
        }
        unreachable!()
    }

    fn solana_client(&self) -> Option<Arc<solana_client::rpc_client::RpcClient>> {
        None
    }

    fn evm_client(&self) -> Result<Arc<dyn EvmClient>, SignerError> {
        Err(SignerError::UnsupportedOperation("No client".to_string()))
    }
}

#[tokio::test]
async fn test_custom_retry_implementation() {
    let signer = RetryTransactionSigner {
        retry_count: std::sync::atomic::AtomicUsize::new(0),
    };

    // Test Solana retry
    let mut sol_tx = Transaction::default();
    let sol_result = signer.sign_and_send_solana_with_retry(&mut sol_tx).await;
    assert_eq!(sol_result.unwrap(), "retry_0__base_solana_sig");

    // Test EVM retry
    let evm_tx = TransactionRequest::default();
    let evm_result = signer.sign_and_send_evm_with_retry(evm_tx).await;
    assert_eq!(evm_result.unwrap(), "retry_0__0xbase_evm_tx");

    // Verify retry was called
    assert!(signer.retry_count.load(std::sync::atomic::Ordering::SeqCst) >= 2);
}