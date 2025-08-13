//! Utility functions for Solana operations

use crate::error::{Result, SolanaToolError};
use riglr_core::SignerContext;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::{CommitmentConfig, CommitmentLevel},
    pubkey::Pubkey,
    signature::Keypair,
    transaction::Transaction,
    instruction::Instruction,
};
#[cfg(test)]
use solana_sdk::signer::Signer;
use std::future::Future;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

pub type SolanaProvider = Arc<RpcClient>;

/// Get RPC URL from environment or use default
pub fn get_rpc_url() -> Result<String> {
    match std::env::var("SOLANA_RPC_URL") {
        Ok(url) if !url.trim().is_empty() => {
            // Validate URL format
            if !url.starts_with("http://") && !url.starts_with("https://") && !url.starts_with("wss://") {
                return Err(SolanaToolError::Generic(format!(
                    "Invalid RPC URL format: {}. Must start with http://, https://, or wss://",
                    url
                )));
            }
            tracing::debug!("Found Solana RPC URL: {}", &url[..std::cmp::min(50, url.len())]);
            Ok(url)
        }
        _ => {
            // Default to mainnet-beta
            Ok("https://api.mainnet-beta.solana.com".to_string())
        }
    }
}

/// Factory function for creating Solana RPC clients
/// Centralizes client creation and ensures consistent configuration
pub fn make_client() -> Result<SolanaProvider> {
    let rpc_url = get_rpc_url()?;
    
    let client = RpcClient::new_with_timeout_and_commitment(
        rpc_url,
        Duration::from_secs(30),
        CommitmentConfig {
            commitment: CommitmentLevel::Confirmed,
        },
    );
    
    Ok(Arc::new(client))
}

/// Create client with specific commitment level
pub fn make_client_with_commitment(commitment: CommitmentLevel) -> Result<SolanaProvider> {
    let rpc_url = get_rpc_url()?;
    
    let client = RpcClient::new_with_timeout_and_commitment(
        rpc_url,
        Duration::from_secs(30),
        CommitmentConfig { commitment },
    );
    
    Ok(Arc::new(client))
}

/// Higher-order function to execute Solana transactions
/// Abstracts signer context retrieval and transaction signing
pub async fn execute_solana_transaction<F, Fut>(
    tx_creator: F,
) -> Result<String>
where
    F: FnOnce(Pubkey, SolanaProvider) -> Fut + Send + 'static,
    Fut: Future<Output = Result<Transaction>> + Send + 'static,
{
    // Get signer from context
    let signer = SignerContext::current()
        .await
        .map_err(|e| SolanaToolError::SignerError(e))?;
    
    // Get Solana pubkey
    let pubkey_str = signer
        .pubkey()
        .ok_or_else(|| SolanaToolError::Generic("No Solana pubkey in signer context".to_string()))?;
    let pubkey = Pubkey::from_str(&pubkey_str)
        .map_err(|e| SolanaToolError::InvalidAddress(format!("Invalid pubkey format: {}", e)))?;
    
    // Create client
    let client = make_client()?;
    
    // Execute transaction creator
    let mut tx = tx_creator(pubkey, client).await?;
    
    // Sign and send via signer context
    signer
        .sign_and_send_solana_transaction(&mut tx)
        .await
        .map_err(|e| SolanaToolError::SignerError(e))
}

/// Check if a Solana address is valid
pub fn validate_address(address: &str) -> Result<Pubkey> {
    Pubkey::from_str(address)
        .map_err(|e| SolanaToolError::InvalidAddress(format!("Invalid address: {}", e)))
}

/// Creates properly signed Solana transaction with mint keypair
pub async fn create_token_with_mint_keypair(
    instructions: Vec<Instruction>,
    mint_keypair: &Keypair,
) -> Result<String> {
    let signer = SignerContext::current().await
        .map_err(|e| SolanaToolError::SignerError(e.into()))?;
    let payer_pubkey = signer.pubkey()
        .ok_or_else(|| SolanaToolError::InvalidKey("No Solana pubkey in signer context".to_string()))?
        .parse()
        .map_err(|e| SolanaToolError::InvalidKey(format!("Invalid pubkey format: {}", e)))?;
    
    let mut tx = Transaction::new_with_payer(&instructions, Some(&payer_pubkey));
    
    // Get recent blockhash using stateless client
    let client = make_client()?;
    let recent_blockhash = client.get_latest_blockhash()
        .map_err(|e| SolanaToolError::SolanaClient(Box::new(e)))?;
    
    tx.partial_sign(&[mint_keypair], recent_blockhash);
    
    // Sign and send transaction via signer context
    let signature = signer.sign_and_send_solana_transaction(&mut tx).await
        .map_err(|e| SolanaToolError::SignerError(e.into()))?;
    
    Ok(signature)
}

/// Generates new mint keypair for token creation
pub fn generate_mint_keypair() -> Keypair {
    Keypair::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    use solana_sdk::pubkey::Pubkey;

    #[test]
    fn test_get_rpc_url() {
        // Test with env var
        std::env::set_var("SOLANA_RPC_URL", "https://custom.solana.com");
        let result = get_rpc_url();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "https://custom.solana.com");
        std::env::remove_var("SOLANA_RPC_URL");
        
        // Test default
        let result = get_rpc_url();
        assert!(result.is_ok());
        assert!(result.unwrap().contains("mainnet"));
    }
    
    #[test]
    fn test_validate_address() {
        // Valid address
        let valid = "11111111111111111111111111111111";
        assert!(validate_address(valid).is_ok());
        
        // Invalid address
        let invalid = "invalid";
        assert!(validate_address(invalid).is_err());
    }

    #[test]
    fn test_generate_mint_keypair() {
        let keypair1 = generate_mint_keypair();
        let keypair2 = generate_mint_keypair();
        
        // Ensure different keypairs are generated
        assert_ne!(keypair1.pubkey(), keypair2.pubkey());
    }

    #[test]
    fn test_keypair_properties() {
        let keypair = generate_mint_keypair();
        
        // Ensure pubkey is valid
        assert_ne!(keypair.pubkey(), Pubkey::default());
        
        // Ensure we can sign with the keypair
        let message = b"test message";
        let signature = keypair.sign_message(message);
        assert!(signature.verify(keypair.pubkey().as_ref(), message));
    }
}