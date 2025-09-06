//! Chain-agnostic client traits for blockchain network interactions
//!
//! This module defines abstract client interfaces that enable riglr-core
//! to interact with blockchain networks without depending on their SDKs.
//!
//! # Architecture
//!
//! These traits serve as abstraction boundaries between riglr-core and
//! blockchain-specific implementations. Concrete implementations live in
//! the respective tools crates (riglr-solana-tools, riglr-evm-tools).
//!
//! # Chain-Agnostic Design
//!
//! - All data is passed in serialized form (JSON for EVM, bytes for Solana)
//! - No blockchain SDK types appear in the trait signatures
//! - Implementations handle serialization/deserialization internally

use super::error::SignerError;
use async_trait::async_trait;

/// Chain-agnostic interface for EVM-compatible blockchain clients.
///
/// This trait abstracts EVM network operations without exposing
/// implementation details from alloy, ethers-rs, or other EVM libraries.
///
/// # Implementation Notes
///
/// Concrete implementations should wrap actual EVM clients (e.g., alloy Provider)
/// and handle JSON serialization/deserialization internally.
#[async_trait]
pub trait EvmClient: Send + Sync + std::fmt::Debug {
    /// Returns the balance of an address in wei as a string.
    ///
    /// # Parameters
    /// * `address` - Ethereum address in checksummed hex format (0x-prefixed)
    ///
    /// # Returns
    /// Balance in wei as a decimal string (e.g., "1000000000000000000" for 1 ETH)
    async fn get_balance(&self, address: &str) -> Result<String, SignerError>;

    /// Sends a transaction to the network and returns the transaction hash.
    ///
    /// # Parameters
    /// * `tx` - Transaction as a JSON object conforming to Ethereum RPC spec
    ///
    /// # Returns
    /// Transaction hash as a 0x-prefixed hex string
    async fn send_transaction(&self, tx: &serde_json::Value) -> Result<String, SignerError>;

    /// Executes a call against the network without creating a transaction.
    ///
    /// Used for read-only contract interactions and gas estimation.
    ///
    /// # Parameters
    /// * `tx` - Call parameters as a JSON object
    ///
    /// # Returns
    /// Call result as a 0x-prefixed hex string
    async fn call(&self, tx: &serde_json::Value) -> Result<String, SignerError>;
}

/// Chain-agnostic interface for Solana blockchain clients.
///
/// This trait abstracts Solana network operations without exposing
/// types from solana-sdk or solana-client.
///
/// # Implementation Notes
///
/// Concrete implementations should wrap actual Solana RPC clients
/// and handle transaction serialization with bincode internally.
#[async_trait]
pub trait SolanaClient: Send + Sync {
    /// Returns the balance of a Solana account in lamports.
    ///
    /// # Parameters
    /// * `pubkey` - Solana public key in base58 encoding
    ///
    /// # Returns
    /// Balance in lamports (1 SOL = 1_000_000_000 lamports)
    async fn get_balance(&self, pubkey: &str) -> Result<u64, SignerError>;

    /// Sends a transaction to the Solana network and returns the signature.
    ///
    /// # Parameters
    /// * `tx` - Serialized transaction bytes (typically using bincode)
    ///
    /// # Returns
    /// Transaction signature as a base58-encoded string
    async fn send_transaction(&self, tx: &[u8]) -> Result<String, SignerError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock EvmClient implementation
    #[derive(Debug)]
    struct MockEvmClient {
        pub balance_result: Result<String, SignerError>,
        pub send_transaction_result: Result<String, SignerError>,
        pub call_result: Result<String, SignerError>,
    }

    impl MockEvmClient {
        fn new() -> Self {
            Self {
                balance_result: Ok("1000".to_string()),
                send_transaction_result: Ok("0xhash".to_string()),
                call_result: Ok("0xdata".to_string()),
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
        async fn get_balance(&self, _address: &str) -> Result<String, SignerError> {
            self.balance_result.clone()
        }

        async fn send_transaction(&self, _tx: &serde_json::Value) -> Result<String, SignerError> {
            self.send_transaction_result.clone()
        }

        async fn call(&self, _tx: &serde_json::Value) -> Result<String, SignerError> {
            self.call_result.clone()
        }
    }

    // Mock SolanaClient implementation
    #[derive(Debug)]
    struct MockSolanaClient {
        pub balance_result: Result<u64, SignerError>,
        pub send_transaction_result: Result<String, SignerError>,
    }

    impl MockSolanaClient {
        fn new() -> Self {
            Self {
                balance_result: Ok(1000000),
                send_transaction_result: Ok("signature".to_string()),
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
        async fn get_balance(&self, _pubkey: &str) -> Result<u64, SignerError> {
            self.balance_result.clone()
        }

        async fn send_transaction(&self, _tx: &[u8]) -> Result<String, SignerError> {
            self.send_transaction_result.clone()
        }
    }

    // Tests for EvmClient trait functionality
    #[tokio::test]
    async fn test_evm_client_get_balance_when_success_should_return_balance() {
        let client = MockEvmClient::new();
        let result = client.get_balance("0x123").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "1000");
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
        let tx = serde_json::json!({"to": "0x123", "value": "0x0"});
        let result = client.send_transaction(&tx).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "0xhash");
    }

    #[tokio::test]
    async fn test_evm_client_send_transaction_when_error_should_return_error() {
        let client = MockEvmClient::new().with_send_transaction_error(
            SignerError::TransactionFailed("Transaction failed".to_string()),
        );
        let tx = serde_json::json!({"to": "0x123", "value": "0x0"});
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
        let tx = serde_json::json!({"to": "0x123", "data": "0x"});
        let result = client.call(&tx).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "0xdata");
    }

    #[tokio::test]
    async fn test_evm_client_call_when_error_should_return_error() {
        let client = MockEvmClient::new()
            .with_call_error(SignerError::TransactionFailed("Call failed".to_string()));
        let tx = serde_json::json!({"to": "0x123", "data": "0x"});
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
        let pubkey = "11111111111111111111111111111112";
        let result = client.get_balance(pubkey).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1000000);
    }

    #[tokio::test]
    async fn test_solana_client_get_balance_when_error_should_return_error() {
        let client = MockSolanaClient::new().with_balance_error(SignerError::ProviderError(
            "Solana network error".to_string(),
        ));
        let pubkey = "11111111111111111111111111111112";
        let result = client.get_balance(pubkey).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            SignerError::ProviderError(msg) => assert_eq!(msg, "Solana network error"),
            _ => panic!("Expected ProviderError"),
        }
    }

    #[tokio::test]
    async fn test_solana_client_send_transaction_when_success_should_return_signature() {
        let client = MockSolanaClient::new();
        let tx = vec![0u8; 64]; // Mock transaction bytes
        let result = client.send_transaction(&tx).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "signature");
    }

    #[tokio::test]
    async fn test_solana_client_send_transaction_when_error_should_return_error() {
        let client = MockSolanaClient::new().with_send_transaction_error(
            SignerError::TransactionFailed("Solana transaction failed".to_string()),
        );
        let tx = vec![0u8; 64]; // Mock transaction bytes
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

        let tx = serde_json::json!({"to": "0x123", "value": "0x0"});
        let send_result = client.send_transaction(&tx).await;
        assert!(send_result.is_ok());

        let call_result = client.call(&tx).await;
        assert!(call_result.is_ok());
    }

    #[tokio::test]
    async fn test_solana_client_as_trait_object_should_work() {
        let client: Box<dyn SolanaClient> = Box::new(MockSolanaClient::new());
        let pubkey = "11111111111111111111111111111112";
        let balance_result = client.get_balance(pubkey).await;
        assert!(balance_result.is_ok());

        let tx = vec![0u8; 64]; // Mock transaction bytes
        let send_result = client.send_transaction(&tx).await;
        assert!(send_result.is_ok());
    }
}
