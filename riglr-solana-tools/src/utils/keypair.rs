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

    #[test]
    fn test_generate_mint_keypair_returns_valid_keypair() {
        let keypair = generate_mint_keypair();

        // Test that pubkey has correct length (32 bytes)
        assert_eq!(keypair.pubkey().to_bytes().len(), 32);

        // Test that secret key has correct length (64 bytes for ed25519)
        assert_eq!(keypair.to_bytes().len(), 64);
    }

    #[test]
    fn test_generate_mint_keypair_multiple_calls_unique() {
        // Generate multiple keypairs to ensure randomness
        let mut pubkeys = std::collections::HashSet::new();

        for _ in 0..100 {
            let keypair = generate_mint_keypair();
            let pubkey = keypair.pubkey();

            // Each pubkey should be unique
            assert!(pubkeys.insert(pubkey), "Duplicate pubkey found: {}", pubkey);
        }

        // Should have 100 unique pubkeys
        assert_eq!(pubkeys.len(), 100);
    }

    #[test]
    fn test_generate_mint_keypair_signature_verification() {
        let keypair = generate_mint_keypair();

        // Test signing and verification with different message types
        let empty_message = b"";
        let short_message = b"a";
        let long_message = b"this is a longer message that should still work perfectly fine";

        // Test empty message
        let sig1 = keypair.sign_message(empty_message);
        assert!(sig1.verify(keypair.pubkey().as_ref(), empty_message));

        // Test short message
        let sig2 = keypair.sign_message(short_message);
        assert!(sig2.verify(keypair.pubkey().as_ref(), short_message));

        // Test long message
        let sig3 = keypair.sign_message(long_message);
        assert!(sig3.verify(keypair.pubkey().as_ref(), long_message));

        // Test that signatures are different for different messages
        assert_ne!(sig1, sig2);
        assert_ne!(sig2, sig3);
        assert_ne!(sig1, sig3);
    }

    #[test]
    fn test_generate_mint_keypair_consistent_behavior() {
        // Test that the function always returns a valid Keypair type
        let keypair = generate_mint_keypair();

        // Should be able to convert to bytes and back
        let keypair_bytes = keypair.to_bytes();
        let reconstructed =
            Keypair::try_from(&keypair_bytes[..]).expect("Should reconstruct keypair");

        // Reconstructed keypair should have same pubkey
        assert_eq!(keypair.pubkey(), reconstructed.pubkey());
    }
}
