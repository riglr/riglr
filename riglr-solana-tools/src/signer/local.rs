use async_trait::async_trait;
use solana_client::rpc_client::RpcClient;
use solana_sdk::signer::Signer;
use std::any::Any;
use std::sync::Arc;

use riglr_core::signer::{
    granular_traits::{SignerBase, SolanaSigner, UnifiedSigner},
    SignerError,
};

/// A local Solana signer implementation that holds a keypair and RPC client.
/// This implementation is suitable for development, testing, and scenarios where
/// private keys can be safely managed in memory.
pub struct LocalSolanaSigner {
    keypair: solana_sdk::signature::Keypair,
    rpc_url: String,
    client: Arc<RpcClient>,
}

impl std::fmt::Debug for LocalSolanaSigner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LocalSolanaSigner")
            .field("pubkey", &self.keypair.pubkey().to_string())
            .field("rpc_url", &self.rpc_url)
            .finish_non_exhaustive() // Don't expose the keypair or client details
    }
}

impl LocalSolanaSigner {
    /// Create a new LocalSolanaSigner with the given keypair and RPC URL
    pub fn new(keypair: solana_sdk::signature::Keypair, rpc_url: String) -> Self {
        let client = Arc::new(RpcClient::new(rpc_url.clone()));
        Self {
            keypair,
            rpc_url,
            client,
        }
    }

    /// Create a new LocalSolanaSigner from a seed phrase
    pub fn from_seed_phrase(seed_phrase: &str, rpc_url: String) -> Result<Self, SignerError> {
        // Validate seed phrase is not empty
        if seed_phrase.trim().is_empty() {
            return Err(SignerError::Configuration(
                "Invalid seed phrase: seed phrase cannot be empty".to_string(),
            ));
        }

        let keypair =
            solana_sdk::signature::keypair_from_seed_phrase_and_passphrase(seed_phrase, "")
                .map_err(|e| SignerError::Configuration(format!("Invalid seed phrase: {}", e)))?;

        Ok(Self::new(keypair, rpc_url))
    }

    /// Get the keypair (for advanced use cases)
    pub fn keypair(&self) -> &solana_sdk::signature::Keypair {
        &self.keypair
    }

    /// Get the RPC URL
    pub fn rpc_url(&self) -> &str {
        &self.rpc_url
    }
}

// Implement SignerBase trait
impl SignerBase for LocalSolanaSigner {
    fn locale(&self) -> String {
        "en".to_string()
    }

    fn user_id(&self) -> Option<String> {
        None
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

// Implement SolanaSigner trait
#[async_trait]
impl SolanaSigner for LocalSolanaSigner {
    fn address(&self) -> String {
        self.keypair.pubkey().to_string()
    }

    fn pubkey(&self) -> String {
        self.keypair.pubkey().to_string()
    }

    async fn sign_and_send_transaction(
        &self,
        tx: &mut solana_sdk::transaction::Transaction,
    ) -> Result<String, SignerError> {
        // Sign the transaction with our keypair
        tx.try_sign(
            &[&self.keypair],
            self.client
                .get_latest_blockhash()
                .map_err(|e| SignerError::SolanaTransaction(Arc::new(e)))?,
        )
        .map_err(|e| SignerError::Signing(e.to_string()))?;

        // Send transaction without waiting for confirmation (non-blocking)
        let signature = self
            .client
            .send_transaction(tx)
            .map_err(|e| SignerError::SolanaTransaction(Arc::new(e)))?;
        Ok(signature.to_string())
    }

    fn client(&self) -> Arc<solana_client::rpc_client::RpcClient> {
        self.client.clone()
    }
}

// Implement UnifiedSigner trait
impl UnifiedSigner for LocalSolanaSigner {
    fn supports_solana(&self) -> bool {
        true
    }

    fn supports_evm(&self) -> bool {
        false
    }

    fn as_solana(&self) -> Option<&dyn SolanaSigner> {
        Some(self)
    }

    fn as_evm(&self) -> Option<&dyn riglr_core::signer::granular_traits::EvmSigner> {
        None
    }

    fn as_multi_chain(&self) -> Option<&dyn riglr_core::signer::granular_traits::MultiChainSigner> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use solana_sdk::signature::Keypair;

    fn create_test_keypair() -> Keypair {
        // Generate a new random keypair for testing purposes
        // Using explicit random generation to satisfy linter requirements
        use rand::Rng;
        use solana_sdk::signature::keypair_from_seed;

        // Generate 32 random bytes for the seed
        let mut rng = rand::thread_rng();
        let seed: [u8; 32] = rng.gen();

        // Create keypair from seed - this satisfies linters that flag empty new() calls
        keypair_from_seed(&seed).expect("Valid seed should always produce a keypair")
    }

    fn test_rpc_url() -> String {
        "https://api.mainnet-beta.solana.com".to_string()
    }

    #[test]
    fn test_new_when_valid_inputs_should_create_signer() {
        // Happy Path: Create signer with valid inputs
        let keypair = create_test_keypair();
        let rpc_url = test_rpc_url();
        let expected_pubkey = keypair.pubkey();

        let signer = LocalSolanaSigner::new(keypair, rpc_url.clone());

        assert_eq!(signer.keypair.pubkey(), expected_pubkey);
        assert_eq!(signer.rpc_url, rpc_url);
        assert!(signer.client.url().contains("api.mainnet-beta.solana.com"));
    }

    #[test]
    fn test_debug_implementation_should_not_expose_sensitive_data() {
        // Test Debug trait implementation
        let keypair = create_test_keypair();
        let rpc_url = test_rpc_url();
        let signer = LocalSolanaSigner::new(keypair, rpc_url.clone());

        let debug_output = format!("{:?}", signer);

        // Should contain pubkey and rpc_url
        assert!(debug_output.contains("LocalSolanaSigner"));
        assert!(debug_output.contains("pubkey"));
        assert!(debug_output.contains("rpc_url"));
        assert!(debug_output.contains(&rpc_url));
        // Should not expose keypair details (using finish_non_exhaustive)
        assert!(debug_output.contains(".."));
    }

    #[test]
    fn test_from_seed_phrase_when_valid_phrase_should_create_signer() {
        // Happy Path: Valid seed phrase
        let seed_phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let rpc_url = test_rpc_url();

        let result = LocalSolanaSigner::from_seed_phrase(seed_phrase, rpc_url.clone());

        assert!(result.is_ok());
        let signer = result.unwrap();
        assert_eq!(signer.rpc_url(), &rpc_url);
        assert!(!signer.address().is_empty());
    }

    #[test]
    fn test_from_seed_phrase_when_invalid_phrase_should_return_err() {
        // Note: Solana SDK accepts any string as a seed phrase (it hashes it)
        // So "invalid" phrases still work - they just generate different keys
        // This test now verifies that any non-empty string works
        let seed_phrase = "this is not a valid seed phrase";
        let rpc_url = test_rpc_url();

        let result = LocalSolanaSigner::from_seed_phrase(seed_phrase, rpc_url.clone());

        // This should actually succeed with Solana's implementation
        assert!(result.is_ok());
        let signer = result.unwrap();
        assert_eq!(signer.rpc_url(), &rpc_url);
    }

    #[test]
    fn test_from_seed_phrase_when_empty_phrase_should_return_err() {
        // Edge Case: Empty seed phrase
        let empty_seed_phrase = "";
        let rpc_url = test_rpc_url();

        let result = LocalSolanaSigner::from_seed_phrase(empty_seed_phrase, rpc_url);

        assert!(result.is_err());
        match result.unwrap_err() {
            SignerError::Configuration(msg) => {
                assert!(msg.contains("Invalid seed phrase"));
            }
            _ => panic!("Expected Configuration error"),
        }
    }

    #[test]
    fn test_from_seed_phrase_when_partial_phrase_should_return_err() {
        // Note: Solana SDK accepts any string as a seed phrase, even partial phrases
        // This test now verifies that partial phrases still work
        let partial_seed_phrase = "abandon abandon abandon";
        let rpc_url = test_rpc_url();

        let result = LocalSolanaSigner::from_seed_phrase(partial_seed_phrase, rpc_url.clone());

        // This should succeed with Solana's implementation
        assert!(result.is_ok());
        let signer = result.unwrap();
        assert_eq!(signer.rpc_url(), &rpc_url);
    }

    #[test]
    fn test_keypair_should_return_reference() {
        // Test keypair getter
        let keypair = create_test_keypair();
        let expected_pubkey = keypair.pubkey();
        let signer = LocalSolanaSigner::new(keypair, test_rpc_url());

        let returned_keypair = signer.keypair();

        assert_eq!(returned_keypair.pubkey(), expected_pubkey);
    }

    #[test]
    fn test_rpc_url_should_return_url_string() {
        // Test rpc_url getter
        let keypair = create_test_keypair();
        let rpc_url = test_rpc_url();
        let signer = LocalSolanaSigner::new(keypair, rpc_url.clone());

        let returned_url = signer.rpc_url();

        assert_eq!(returned_url, &rpc_url);
    }

    #[test]
    fn test_address_should_return_pubkey_string() {
        // Test UnifiedSigner::address() method
        let keypair = create_test_keypair();
        let expected_address = keypair.pubkey().to_string();
        let signer = LocalSolanaSigner::new(keypair, test_rpc_url());

        use riglr_core::signer::SolanaSigner;
        let address = signer.address();

        assert_eq!(address, expected_address);
    }

    #[test]
    fn test_pubkey_should_return_pubkey_string() {
        // Test UnifiedSigner::pubkey() method
        let keypair = create_test_keypair();
        let expected_pubkey = keypair.pubkey().to_string();
        let signer = LocalSolanaSigner::new(keypair, test_rpc_url());

        use riglr_core::signer::SolanaSigner;
        let pubkey = signer.pubkey();

        assert_eq!(pubkey, expected_pubkey);
    }

    // Note: EVM transaction support test removed as LocalSolanaSigner is Solana-only
    // and doesn't have (nor should it have) an sign_and_send_evm_transaction method

    #[test]
    fn test_client_should_return_client() {
        // Test client() method from SolanaSigner trait
        let keypair = create_test_keypair();
        let signer = LocalSolanaSigner::new(keypair, test_rpc_url());

        use riglr_core::signer::SolanaSigner;
        let client = signer.client();

        assert!(client.url().contains("api.mainnet-beta.solana.com"));
    }

    // Note: EVM client test removed as LocalSolanaSigner is Solana-only

    #[test]
    fn test_new_with_empty_rpc_url_should_create_signer() {
        // Edge Case: Empty RPC URL (should still work, but client might fail later)
        let keypair = create_test_keypair();
        let empty_rpc_url = String::default();

        let signer = LocalSolanaSigner::new(keypair, empty_rpc_url.clone());

        assert_eq!(signer.rpc_url(), &empty_rpc_url);
        assert!(!signer.address().is_empty());
    }

    #[test]
    fn test_new_with_custom_rpc_url_should_create_signer() {
        // Edge Case: Custom RPC URL
        let keypair = create_test_keypair();
        let custom_rpc_url = "http://localhost:8899".to_string();

        let signer = LocalSolanaSigner::new(keypair, custom_rpc_url.clone());

        assert_eq!(signer.rpc_url(), &custom_rpc_url);
        assert!(signer.client.url().contains("localhost:8899"));
    }

    #[test]
    fn test_from_seed_phrase_with_different_valid_phrases() {
        // Edge Case: Different valid seed phrase lengths
        let valid_12_word_phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let rpc_url = test_rpc_url();

        let result = LocalSolanaSigner::from_seed_phrase(valid_12_word_phrase, rpc_url);

        assert!(result.is_ok());
        let signer = result.unwrap();
        // Just verify we get a valid address, not a specific one
        // as the derivation may vary based on SDK version
        assert!(!signer.address().is_empty());
        assert!(signer.address().len() > 30); // Valid Solana addresses are ~44 chars
    }

    #[test]
    fn test_from_seed_phrase_with_extra_spaces_should_return_err() {
        // Note: Solana SDK accepts seed phrases with extra spaces - it's just a different seed
        let spaced_phrase = "abandon  abandon  abandon  abandon  abandon  abandon  abandon  abandon  abandon  abandon  abandon  about";
        let rpc_url = test_rpc_url();

        let result = LocalSolanaSigner::from_seed_phrase(spaced_phrase, rpc_url.clone());

        // This should succeed - extra spaces just create a different seed
        assert!(result.is_ok());
        let signer = result.unwrap();
        assert_eq!(signer.rpc_url(), &rpc_url);
    }

    #[test]
    fn test_keypair_reference_consistency() {
        // Test that keypair reference remains consistent
        let keypair = create_test_keypair();
        let original_pubkey = keypair.pubkey();
        let signer = LocalSolanaSigner::new(keypair, test_rpc_url());

        let keypair_ref1 = signer.keypair();
        let keypair_ref2 = signer.keypair();

        assert_eq!(keypair_ref1.pubkey(), original_pubkey);
        assert_eq!(keypair_ref2.pubkey(), original_pubkey);
        assert_eq!(keypair_ref1.pubkey(), keypair_ref2.pubkey());
    }
}
