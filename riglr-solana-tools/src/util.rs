//! Utility functions for Solana operations

use solana_sdk::{
    signature::Keypair,
    transaction::Transaction,
    instruction::Instruction,
};
#[cfg(test)]
use solana_sdk::signer::Signer;
use riglr_core::signer::SignerContext;
use crate::error::SolanaToolError;

/// Creates properly signed Solana transaction with mint keypair
pub async fn create_token_with_mint_keypair(
    instructions: Vec<Instruction>,
    mint_keypair: &Keypair,
) -> Result<String, SolanaToolError> {
    let signer = SignerContext::current().await
        .map_err(|e| SolanaToolError::SignerError(e.into()))?;
    let payer_pubkey = signer.pubkey()
        .ok_or_else(|| SolanaToolError::InvalidKey("No Solana pubkey in signer context".to_string()))?
        .parse()
        .map_err(|e| SolanaToolError::InvalidKey(format!("Invalid pubkey format: {}", e)))?;
    
    let mut tx = Transaction::new_with_payer(&instructions, Some(&payer_pubkey));
    
    // Get recent blockhash
    let rpc_client = signer.solana_client();
    let recent_blockhash = rpc_client.get_latest_blockhash()
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