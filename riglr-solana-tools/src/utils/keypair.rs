//! Keypair generation utilities for Solana
//!
//! This module provides utilities for generating keypairs for various purposes,
//! such as mint accounts, program derived addresses, etc.

use solana_sdk::signature::Keypair;

/// Generates new mint keypair for token creation
///
/// Creates a new randomly generated keypair suitable for use as a mint account
/// in SPL token creation operations.
///
/// # Returns
///
/// Returns a new `Keypair` with a randomly generated public/private key pair.
///
/// # Examples
///
/// ```rust,ignore
/// use riglr_solana_tools::utils::keypair::generate_mint_keypair;
///
/// let mint_keypair = generate_mint_keypair();
/// println!("New mint pubkey: {}", mint_keypair.pubkey());
/// ```
///
/// # Security Notes
///
/// - Each call generates a completely new keypair
/// - The private key should be handled securely
/// - For production use, consider proper key management practices
pub fn generate_mint_keypair() -> Keypair {
    Keypair::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    use solana_sdk::{pubkey::Pubkey, signer::Signer};

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
