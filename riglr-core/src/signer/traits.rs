//! Core client traits for blockchain network interactions
//!
//! This module defines client traits used by the signer system to provide
//! unified interfaces for blockchain network interactions across different networks.

use super::error::SignerError;
use alloy::primitives::{Bytes, TxHash, U256};
use alloy::rpc::types::TransactionRequest;
use async_trait::async_trait;
use solana_sdk::{pubkey::Pubkey, signature::Signature, transaction::Transaction};

/// Type-safe EVM client interface
#[async_trait]
pub trait EvmClient: Send + Sync + std::fmt::Debug {
    /// Get the balance of an address in wei
    async fn get_balance(&self, address: &str) -> Result<U256, SignerError>;

    /// Send a transaction to the network and return the transaction hash
    async fn send_transaction(&self, tx: &TransactionRequest) -> Result<TxHash, SignerError>;

    /// Execute a call against the network without creating a transaction
    async fn call(&self, tx: &TransactionRequest) -> Result<Bytes, SignerError>;
}

/// Type-safe Solana client interface
#[async_trait]
pub trait SolanaClient: Send + Sync {
    /// Get the balance of a Solana account in lamports
    async fn get_balance(&self, pubkey: &Pubkey) -> Result<u64, SignerError>;

    /// Send a transaction to the Solana network and return the signature
    async fn send_transaction(&self, tx: &Transaction) -> Result<Signature, SignerError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock EvmClient implementation
    #[derive(Debug)]
    struct MockEvmClient {
        pub balance_result: Result<U256, SignerError>,
        pub send_transaction_result: Result<TxHash, SignerError>,
        pub call_result: Result<Bytes, SignerError>,
    }

    impl MockEvmClient {
        fn new() -> Self {
            Self {
                balance_result: Ok(U256::from(1000)),
                send_transaction_result: Ok(TxHash::default()),
                call_result: Ok(Bytes::default()),
            }
        }

        fn with_balance_error(mut self, error: SignerError) -> Self {
            self.balance_result = Err(error);
            self
        }

        fn with_send_transaction_error(mut self, error: SignerError) -> Self {
            self.send_transaction_result = Err(error);
            self
        }

        fn with_call_error(mut self, error: SignerError) -> Self {
            self.call_result = Err(error);
            self
        }
    }

    #[async_trait]
    impl EvmClient for MockEvmClient {
        async fn get_balance(&self, _address: &str) -> Result<U256, SignerError> {
            self.balance_result.clone()
        }

        async fn send_transaction(&self, _tx: &TransactionRequest) -> Result<TxHash, SignerError> {
            self.send_transaction_result.clone()
        }

        async fn call(&self, _tx: &TransactionRequest) -> Result<Bytes, SignerError> {
            self.call_result.clone()
        }
    }

    // Mock SolanaClient implementation
    #[derive(Debug)]
    struct MockSolanaClient {
        pub balance_result: Result<u64, SignerError>,
        pub send_transaction_result: Result<Signature, SignerError>,
    }

    impl MockSolanaClient {
        fn new() -> Self {
            Self {
                balance_result: Ok(1000000),
                send_transaction_result: Ok(Signature::default()),
            }
        }

        fn with_balance_error(mut self, error: SignerError) -> Self {
            self.balance_result = Err(error);
            self
        }

        fn with_send_transaction_error(mut self, error: SignerError) -> Self {
            self.send_transaction_result = Err(error);
            self
        }
    }

    #[async_trait]
    impl SolanaClient for MockSolanaClient {
        async fn get_balance(&self, _pubkey: &Pubkey) -> Result<u64, SignerError> {
            self.balance_result.clone()
        }

        async fn send_transaction(&self, _tx: &Transaction) -> Result<Signature, SignerError> {
            self.send_transaction_result.clone()
        }
    }

    // Tests for EvmClient trait functionality
    #[tokio::test]
    async fn test_evm_client_get_balance_when_success_should_return_balance() {
        let client = MockEvmClient::new();
        let result = client.get_balance("0x123").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), U256::from(1000));
    }

    #[tokio::test]
    async fn test_evm_client_get_balance_when_error_should_return_error() {
        let client = MockEvmClient::new()
            .with_balance_error(SignerError::ProviderError("Network error".to_string()));
        let result = client.get_balance("0x123").await;
        assert!(result.is_err());
        match result.unwrap_err() {
            SignerError::ProviderError(msg) => assert_eq!(msg, "Network error"),
            _ => panic!("Expected ProviderError"),
        }
    }

    #[tokio::test]
    async fn test_evm_client_send_transaction_when_success_should_return_hash() {
        let client = MockEvmClient::new();
        let tx = TransactionRequest::default();
        let result = client.send_transaction(&tx).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), TxHash::default());
    }

    #[tokio::test]
    async fn test_evm_client_send_transaction_when_error_should_return_error() {
        let client = MockEvmClient::new().with_send_transaction_error(
            SignerError::TransactionFailed("Transaction failed".to_string()),
        );
        let tx = TransactionRequest::default();
        let result = client.send_transaction(&tx).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            SignerError::TransactionFailed(msg) => assert_eq!(msg, "Transaction failed"),
            _ => panic!("Expected TransactionFailed error"),
        }
    }

    #[tokio::test]
    async fn test_evm_client_call_when_success_should_return_bytes() {
        let client = MockEvmClient::new();
        let tx = TransactionRequest::default();
        let result = client.call(&tx).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Bytes::default());
    }

    #[tokio::test]
    async fn test_evm_client_call_when_error_should_return_error() {
        let client = MockEvmClient::new()
            .with_call_error(SignerError::TransactionFailed("Call failed".to_string()));
        let tx = TransactionRequest::default();
        let result = client.call(&tx).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            SignerError::TransactionFailed(msg) => assert_eq!(msg, "Call failed"),
            _ => panic!("Expected TransactionFailed error"),
        }
    }

    // Tests for SolanaClient trait functionality
    #[tokio::test]
    async fn test_solana_client_get_balance_when_success_should_return_balance() {
        let client = MockSolanaClient::new();
        let pubkey = Pubkey::default();
        let result = client.get_balance(&pubkey).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1000000);
    }

    #[tokio::test]
    async fn test_solana_client_get_balance_when_error_should_return_error() {
        let client = MockSolanaClient::new().with_balance_error(SignerError::ProviderError(
            "Solana network error".to_string(),
        ));
        let pubkey = Pubkey::default();
        let result = client.get_balance(&pubkey).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            SignerError::ProviderError(msg) => assert_eq!(msg, "Solana network error"),
            _ => panic!("Expected ProviderError"),
        }
    }

    #[tokio::test]
    async fn test_solana_client_send_transaction_when_success_should_return_signature() {
        let client = MockSolanaClient::new();
        let tx = Transaction::default();
        let result = client.send_transaction(&tx).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Signature::default());
    }

    #[tokio::test]
    async fn test_solana_client_send_transaction_when_error_should_return_error() {
        let client = MockSolanaClient::new().with_send_transaction_error(
            SignerError::TransactionFailed("Solana transaction failed".to_string()),
        );
        let tx = Transaction::default();
        let result = client.send_transaction(&tx).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            SignerError::TransactionFailed(msg) => assert_eq!(msg, "Solana transaction failed"),
            _ => panic!("Expected TransactionFailed error"),
        }
    }

    #[tokio::test]
    async fn test_evm_client_as_trait_object_should_work() {
        let client: Box<dyn EvmClient> = Box::new(MockEvmClient::new());
        let balance_result = client.get_balance("0x123").await;
        assert!(balance_result.is_ok());

        let tx = TransactionRequest::default();
        let send_result = client.send_transaction(&tx).await;
        assert!(send_result.is_ok());

        let call_result = client.call(&tx).await;
        assert!(call_result.is_ok());
    }

    #[tokio::test]
    async fn test_solana_client_as_trait_object_should_work() {
        let client: Box<dyn SolanaClient> = Box::new(MockSolanaClient::new());
        let pubkey = Pubkey::default();
        let balance_result = client.get_balance(&pubkey).await;
        assert!(balance_result.is_ok());

        let tx = Transaction::default();
        let send_result = client.send_transaction(&tx).await;
        assert!(send_result.is_ok());
    }
}
