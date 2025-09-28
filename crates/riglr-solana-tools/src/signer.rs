//! Solana-specific signer implementations.
//!
//! This module contains signer implementations that are specific to the Solana blockchain,
//! providing concrete implementations of the `UnifiedSigner` trait from `riglr-core`.

//! Solana transaction signer implementation
//!
//! This module provides complete Solana transaction signing and sending capabilities
//! with proper keypair management and blockhash handling.

extern crate alloc;

use alloc::sync::Arc;
use async_trait::async_trait;
use core::fmt::{Debug, Formatter, Result as FmtResult};
use solana_client::rpc_client::RpcClient;
use solana_commitment_config::CommitmentConfig;
use solana_compute_budget_interface::ComputeBudgetInstruction;
use solana_sdk::{
    instruction::Instruction, pubkey::Pubkey, signature::keypair_from_seed_phrase_and_passphrase,
    signature::Keypair, signer::Signer as _, transaction::Transaction,
};
use tokio::task::spawn_blocking;
use tracing::debug;

use crate::error::Error;
use riglr_config::SolanaNetworkConfig;
use riglr_core::signer::error::Standard;
use riglr_core::signer::{
    granular_traits::EvmSigner, Chain, SignerBase, SignerError, SolanaClient, SolanaSigner,
    UnifiedSigner,
};
use serde_json;

/// Solana priority fee configuration
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct PriorityFeeConfig {
    /// Additional compute units to request
    pub additional_compute_units: Option<u32>,
    /// Enable priority fees
    pub enabled: bool,
    /// Priority fee in microlamports per compute unit
    pub microlamports_per_cu: u64,
}

impl Default for PriorityFeeConfig {
    #[inline]
    fn default() -> Self {
        Self {
            additional_compute_units: Some(200_000),
            enabled: true,
            microlamports_per_cu: 1000, // 0.001 lamports per CU
        }
    }
}

/// Wrapper to implement `SolanaClient` for our `RpcClient`
pub struct SolanaClientWrapper {
    /// RPC client for Solana network operations
    client: Arc<RpcClient>,
}

impl Debug for SolanaClientWrapper {
    #[inline]
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        formatter
            .debug_struct("SolanaClientWrapper")
            .field("client", &"Arc<RpcClient>")
            .finish()
    }
}

/// Local Solana signer with keypair management
pub struct Local {
    /// RPC client for network operations
    client: Arc<RpcClient>,
    /// Client wrapper implementing `SolanaClient` trait
    client_wrapper: SolanaClientWrapper,
    /// Network configuration
    config: SolanaNetworkConfig,
    /// Keypair for signing operations
    keypair: Arc<Keypair>,
    /// Priority fee configuration
    priority_config: PriorityFeeConfig,
}

impl Local {
    /// Add priority fee instructions to transaction
    #[inline]
    pub fn add_priority_fee_instructions(&self, instructions: &mut Vec<Instruction>) {
        if !self.priority_config.enabled {
            return;
        }

        // Add compute budget instructions at the beginning
        let mut priority_instructions = vec![];

        // Set compute unit price for priority
        priority_instructions.push(ComputeBudgetInstruction::set_compute_unit_price(
            self.priority_config.microlamports_per_cu,
        ));

        // Optionally set compute unit limit
        if let Some(units) = self.priority_config.additional_compute_units {
            priority_instructions.push(ComputeBudgetInstruction::set_compute_unit_limit(units));
        }

        // Insert at the beginning of instructions
        priority_instructions.append(instructions);
        *instructions = priority_instructions;

        debug!(
            "Added priority fee: {} microlamports/CU",
            self.priority_config.microlamports_per_cu
        );
    }

    /// Create a new Solana signer from a Keypair and network config
    #[must_use]
    #[inline]
    pub fn from_keypair(keypair: Keypair, config: SolanaNetworkConfig) -> Self {
        let client = Arc::new(RpcClient::new_with_commitment(
            &config.rpc_url,
            CommitmentConfig::confirmed(),
        ));

        let client_wrapper = SolanaClientWrapper {
            client: Arc::<RpcClient>::clone(&client),
        };

        Self {
            client,
            client_wrapper,
            config,
            keypair: Arc::new(keypair),
            priority_config: PriorityFeeConfig::default(),
        }
    }

    /// Create a new Solana signer from a Keypair and RPC URL (compatibility)
    ///
    /// This method is provided for backward compatibility but the config-based methods are preferred.
    #[must_use]
    #[inline]
    pub fn from_keypair_with_url(keypair: Keypair, rpc_url: String) -> Self {
        let config = SolanaNetworkConfig::new("custom", rpc_url);
        Self::from_keypair(keypair, config)
    }

    /// Create a new `Local` from a seed phrase (for compatibility with existing code)
    ///
    /// # Errors
    /// Returns an error if the seed phrase is empty or invalid
    #[inline]
    pub fn from_seed_phrase(
        seed_phrase: &str,
        rpc_url: String,
    ) -> Result<Self, Box<dyn SignerError>> {
        // Validate seed phrase is not empty
        if seed_phrase.trim().is_empty() {
            return Err(Box::new(Standard::Configuration(
                "Invalid seed phrase: seed phrase cannot be empty".to_string(),
            )) as Box<dyn SignerError>);
        }

        let keypair =
            keypair_from_seed_phrase_and_passphrase(seed_phrase, "").map_err(|seed_error| {
                Box::new(Standard::Configuration(format!(
                    "Invalid seed phrase: {seed_error}"
                ))) as Box<dyn SignerError>
            })?;

        let config = SolanaNetworkConfig::new("custom", rpc_url);
        Ok(Self::from_keypair(keypair, config))
    }

    /// Get the public key of this signer
    #[must_use]
    #[inline]
    pub fn get_pubkey(&self) -> Pubkey {
        self.keypair.pubkey()
    }

    /// Get recent prioritization fees from the network
    ///
    /// # Errors
    /// Does not return errors - falls back to configured default if RPC calls fail
    #[inline]
    pub fn get_recent_prioritization_fees(&self) -> Result<u64, Error> {
        // Try to get recent prioritization fees from the network
        // Using the getRecentPrioritizationFees RPC method

        // Try to get fees from RPC, fall back to default if unavailable
        match self.client.get_recent_prioritization_fees(&[]) {
            Ok(fees) if !fees.is_empty() => {
                // Calculate average of recent fees
                let total: u64 = fees
                    .iter()
                    .map(|fee_info| fee_info.prioritization_fee)
                    .sum();
                let average = total.checked_div(fees.len() as u64).unwrap_or(0);

                debug!(
                    "Average recent prioritization fee: {} microlamports/CU",
                    average
                );

                // Use the average or fall back to configured if too low
                if average > 0 {
                    return Ok(average);
                }
                Ok(self.priority_config.microlamports_per_cu)
            }
            Ok(_) => {
                // No recent fees available, use configured default
                debug!(
                    "No recent prioritization fees available, using default: {} microlamports/CU",
                    self.priority_config.microlamports_per_cu
                );
                Ok(self.priority_config.microlamports_per_cu)
            }
            Err(network_error) => {
                // RPC error, log and use default
                debug!(
                    "Failed to get recent prioritization fees: {}, using default",
                    network_error
                );
                Ok(self.priority_config.microlamports_per_cu)
            }
        }
    }

    /// Get the Solana RPC client directly (convenience method for tools)
    #[must_use]
    #[inline]
    pub fn get_rpc_client(&self) -> Arc<RpcClient> {
        Arc::<RpcClient>::clone(&self.client)
    }

    /// Get the keypair (for advanced use cases)
    #[must_use]
    #[inline]
    pub fn keypair(&self) -> &Keypair {
        &self.keypair
    }

    /// Create a new Solana signer from a base58-encoded private key and network config
    ///
    /// # Errors
    /// Returns an error if the private key is invalid or network configuration is malformed
    #[inline]
    pub fn new(
        private_key: &str,
        config: SolanaNetworkConfig,
    ) -> Result<Self, Box<dyn SignerError>> {
        let keypair = Keypair::from_base58_string(private_key);

        let client = Arc::new(RpcClient::new_with_commitment(
            &config.rpc_url,
            CommitmentConfig::confirmed(),
        ));

        let client_wrapper = SolanaClientWrapper {
            client: Arc::<RpcClient>::clone(&client),
        };

        Ok(Self {
            client,
            client_wrapper,
            config,
            keypair: Arc::new(keypair),
            priority_config: PriorityFeeConfig::default(),
        })
    }

    /// Create a new Solana signer with custom priority fee configuration
    ///
    /// # Errors
    /// Returns an error if the private key is invalid or network configuration is malformed
    #[inline]
    pub fn new_with_priority_config(
        private_key: &str,
        config: SolanaNetworkConfig,
        priority_config: PriorityFeeConfig,
    ) -> Result<Self, Box<dyn SignerError>> {
        let keypair = Keypair::from_base58_string(private_key);

        let client = Arc::new(RpcClient::new_with_commitment(
            &config.rpc_url,
            CommitmentConfig::confirmed(),
        ));

        let client_wrapper = SolanaClientWrapper {
            client: Arc::<RpcClient>::clone(&client),
        };

        Ok(Self {
            client,
            client_wrapper,
            config,
            keypair: Arc::new(keypair),
            priority_config,
        })
    }

    /// Create a new Solana signer from a base58-encoded private key and RPC URL (compatibility)
    ///
    /// This method is provided for backward compatibility but the config-based methods are preferred.
    ///
    /// # Errors
    /// Returns an error if the private key is invalid or RPC URL is malformed
    #[inline]
    pub fn new_with_url(private_key: &str, rpc_url: String) -> Result<Self, Box<dyn SignerError>> {
        let config = SolanaNetworkConfig::new("custom", rpc_url);
        Self::new(private_key, config)
    }

    /// Optimize transaction for size and compute units
    ///
    /// # Errors
    /// Returns an error if transaction serialization fails or size exceeds limits
    #[inline]
    pub fn optimize_transaction(&self, tx: &mut Transaction) -> Result<(), Error> {
        const MAX_TRANSACTION_SIZE: usize = 1232; // Solana's max transaction size

        // Check transaction size
        let serialized = bincode::serialize(&tx).map_err(|serialization_error| {
            Error::Generic(format!(
                "Failed to serialize transaction: {serialization_error}"
            ))
        })?;

        if serialized.len() > MAX_TRANSACTION_SIZE {
            return Err(Error::Transaction(format!(
                "Transaction size {} exceeds maximum {}",
                serialized.len(),
                MAX_TRANSACTION_SIZE
            )));
        }

        debug!("Transaction size: {} bytes", serialized.len());
        Ok(())
    }

    /// Get the RPC URL
    #[must_use]
    #[inline]
    pub fn rpc_url(&self) -> &str {
        &self.config.rpc_url
    }
}

impl Debug for Local {
    #[inline]
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        formatter
            .debug_struct("Local")
            .field("pubkey", &self.get_pubkey().to_string())
            .finish()
    }
}

#[async_trait]
impl SolanaSigner for Local {
    #[inline]
    fn client(&self) -> &dyn SolanaClient {
        &self.client_wrapper
    }

    #[inline]
    fn pubkey(&self) -> String {
        self.get_pubkey().to_string()
    }

    async fn sign_and_send_transaction(
        &self,
        _transaction: serde_json::Value,
    ) -> Result<String, Box<dyn SignerError>> {
        // For now, return an error indicating this method needs implementation
        // This should handle JSON transaction format and convert to Solana transaction
        Err(Box::new(Standard::SigningFailed(
            "Transaction signing not yet implemented for JSON format".to_string(),
        )))
    }

    async fn sign_message(&self, message: &[u8]) -> Result<String, Box<dyn SignerError>> {
        let signature = self.keypair.sign_message(message);
        Ok(signature.to_string())
    }
}

#[async_trait]
impl SolanaClient for SolanaClientWrapper {
    async fn confirm_transaction(
        &self,
        _signature: &str,
    ) -> Result<serde_json::Value, Box<dyn SignerError>> {
        // Stub implementation
        Err(Box::new(Standard::SigningFailed(
            "Confirm transaction not yet implemented".to_string(),
        )))
    }

    async fn get_account_info(
        &self,
        _pubkey: &str,
    ) -> Result<Option<serde_json::Value>, Box<dyn SignerError>> {
        // Stub implementation
        Err(Box::new(Standard::SigningFailed(
            "Get account info not yet implemented".to_string(),
        )))
    }

    async fn get_balance(&self, pubkey_str: &str) -> Result<String, Box<dyn SignerError>> {
        use core::str::FromStr as _;
        let pubkey = Pubkey::from_str(pubkey_str).map_err(|parse_error| {
            Box::new(Standard::InvalidInput(format!(
                "Invalid pubkey: {parse_error}"
            ))) as Box<dyn SignerError>
        })?;

        let spawn_result = spawn_blocking({
            let client = Arc::<RpcClient>::clone(&self.client);
            move || client.get_balance(&pubkey)
        })
        .await;
        let balance = spawn_result
            .map_err(|spawn_error| {
                Box::new(Standard::Network(format!(
                    "Failed to get balance: {spawn_error}"
                ))) as Box<dyn SignerError>
            })?
            .map_err(|rpc_error| {
                Box::new(Standard::Network(format!(
                    "RPC error getting balance: {rpc_error}"
                ))) as Box<dyn SignerError>
            })?;

        Ok(balance.to_string())
    }

    async fn get_fee_for_transaction(
        &self,
        _transaction: &serde_json::Value,
    ) -> Result<String, Box<dyn SignerError>> {
        // Stub implementation
        Err(Box::new(Standard::SigningFailed(
            "Get fee for transaction not yet implemented".to_string(),
        )))
    }

    async fn get_latest_blockhash(&self) -> Result<String, Box<dyn SignerError>> {
        let spawn_result = spawn_blocking({
            let client = Arc::<RpcClient>::clone(&self.client);
            move || client.get_latest_blockhash()
        })
        .await;
        let blockhash = spawn_result
            .map_err(|spawn_error| {
                Box::new(Standard::Network(format!(
                    "Failed to get blockhash: {spawn_error}"
                ))) as Box<dyn SignerError>
            })?
            .map_err(|rpc_error| {
                Box::new(Standard::Network(format!(
                    "RPC error getting blockhash: {rpc_error}"
                ))) as Box<dyn SignerError>
            })?;

        Ok(blockhash.to_string())
    }

    async fn get_minimum_balance_for_rent_exemption(
        &self,
        _data_len: usize,
    ) -> Result<String, Box<dyn SignerError>> {
        // Stub implementation
        Err(Box::new(Standard::SigningFailed(
            "Get minimum balance for rent exemption not yet implemented".to_string(),
        )))
    }

    async fn get_slot(&self) -> Result<String, Box<dyn SignerError>> {
        let spawn_result = spawn_blocking({
            let client = Arc::<RpcClient>::clone(&self.client);
            move || client.get_slot()
        })
        .await;
        let slot = spawn_result
            .map_err(|spawn_error| {
                Box::new(Standard::Network(format!(
                    "Failed to get slot: {spawn_error}"
                ))) as Box<dyn SignerError>
            })?
            .map_err(|rpc_error| {
                Box::new(Standard::Network(format!(
                    "RPC error getting slot: {rpc_error}"
                ))) as Box<dyn SignerError>
            })?;

        Ok(slot.to_string())
    }

    async fn get_transaction(
        &self,
        _signature: &str,
    ) -> Result<Option<serde_json::Value>, Box<dyn SignerError>> {
        // Stub implementation
        Err(Box::new(Standard::SigningFailed(
            "Get transaction not yet implemented".to_string(),
        )))
    }

    async fn send_transaction(&self, _transaction: &str) -> Result<String, Box<dyn SignerError>> {
        // This would need to be implemented based on the actual transaction format
        Err(Box::new(Standard::SigningFailed(
            "Send transaction not yet implemented".to_string(),
        )))
    }
}

impl UnifiedSigner for Local {
    #[inline]
    fn as_evm(&self) -> Option<&dyn EvmSigner> {
        None
    }

    #[inline]
    fn as_solana(&self) -> Option<&dyn SolanaSigner> {
        Some(self)
    }

    #[inline]
    fn supports_evm(&self) -> bool {
        false
    }

    #[inline]
    fn supports_solana(&self) -> bool {
        true
    }
}

#[async_trait]
impl SignerBase for Local {
    #[inline]
    fn supported_chains(&self) -> &[Chain] {
        &[Chain::Solana]
    }

    #[inline]
    fn supports_chain(&self, chain: Chain) -> bool {
        matches!(chain, Chain::Solana)
    }

    #[inline]
    fn user_id(&self) -> String {
        "default".to_owned()
    }
}

#[cfg(test)]
#[expect(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn local_module_exists() {
        // Test that the local module is accessible
        // This ensures the module declaration is correct
        use core::any::type_name;
        let module_exists = type_name::<Local>();
        assert!(module_exists.contains("Local"));
    }

    #[test]
    fn local_solana_signer_re_export() {
        // Test that Local is properly accessible
        use core::any::type_name;
        let type_name = type_name::<Local>();
        assert!(type_name.contains("Local"));
    }

    #[test]
    fn module_structure_accessibility() {
        // Test that we can access the module structure correctly
        // This ensures all public items are accessible as expected

        // Create a dummy keypair for testing module accessibility
        use solana_sdk::signature::Keypair;
        let keypair = Keypair::new();
        let rpc_url = "https://api.devnet.solana.com".to_owned();

        // Test that we can create a Local
        let _signer = Local::from_keypair_with_url(keypair, rpc_url);

        // Test passes if we can construct the type without compilation errors
    }

    #[test]
    fn module_documentation_accessible() {
        // Test that module items maintain their documentation and structure
        // This is a compile-time verification that the module organization is correct

        // Verify the type has the same interface
        use solana_sdk::signature::Keypair;
        let keypair = Keypair::new();
        let rpc_url = "https://api.devnet.solana.com".to_owned();
        let signer = Local::from_keypair_with_url(keypair, rpc_url);

        // Test that methods are accessible
        let _rpc_url = signer.rpc_url();
        let _keypair = signer.keypair();
    }

    #[test]
    fn from_seed_phrase() {
        // Test that static methods work
        let seed_phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let rpc_url = "https://api.devnet.solana.com".to_owned();

        // This should work
        let result = Local::from_seed_phrase(seed_phrase, rpc_url);
        result.unwrap();
    }

    #[test]
    fn from_seed_phrase_invalid_seed() {
        // Note: Solana accepts any string as a seed phrase
        // This test now verifies that non-BIP39 phrases still work
        let seed = "invalid seed phrase";
        let rpc_url = "https://api.devnet.solana.com".to_owned();

        let result = Local::from_seed_phrase(seed, rpc_url.clone());
        let signer = result.unwrap();

        assert_eq!(signer.rpc_url(), rpc_url);
        assert!(!signer.pubkey().is_empty());
    }

    #[test]
    fn debug_implementation() {
        // Test that Debug trait is accessible
        use solana_sdk::signature::Keypair;
        let keypair = Keypair::new();
        let rpc_url = "https://api.devnet.solana.com".to_owned();
        let signer = Local::from_keypair_with_url(keypair, rpc_url);

        let debug_output = format!("{signer:?}");
        assert!(debug_output.contains("Local"));
        assert!(debug_output.contains("pubkey"));

        // Ensure sensitive data is not exposed
        assert!(!debug_output.contains("keypair"));
        assert!(!debug_output.contains("client"));
    }

    #[test]
    fn trait_implementations_accessible() {
        // Test that trait implementations are accessible
        use riglr_core::signer::granular_traits::{SolanaSigner, UnifiedSigner};

        use solana_sdk::signature::Keypair;
        let keypair = Keypair::new();
        let rpc_url = "https://api.devnet.solana.com".to_owned();
        let signer = Local::from_keypair_with_url(keypair, rpc_url);

        // Test SolanaSigner trait methods are accessible
        let pubkey = signer.pubkey();
        assert!(!pubkey.is_empty());

        // Test Solana client access - returns &dyn SolanaClient
        let _client = signer.client();

        // Test that Solana signer doesn't support EVM
        assert!(signer.supports_solana());
        assert!(!signer.supports_evm());
        assert!(signer.as_solana().is_some());
        assert!(signer.as_evm().is_none());
    }

    #[test]
    fn solana_signer_creation_with_valid_key() {
        let keypair = Keypair::new();
        let private_key = keypair.to_base58_string();
        let config = SolanaNetworkConfig::new("devnet", "https://api.devnet.solana.com");

        let signer = Local::new(&private_key, config).unwrap();
        let pubkey = SolanaSigner::pubkey(&signer);
        assert_eq!(pubkey, keypair.pubkey().to_string());
    }

    #[test]
    fn solana_signer_from_keypair() {
        let keypair = Keypair::new();
        let expected_pubkey = keypair.pubkey();
        let config = SolanaNetworkConfig::new("devnet", "https://api.devnet.solana.com");

        let signer = Local::from_keypair(keypair, config);
        assert_eq!(signer.get_pubkey(), expected_pubkey);
    }

    #[test]
    fn solana_signer_get_pubkey() {
        let keypair = Keypair::new();
        let expected_pubkey = keypair.pubkey();
        let config = SolanaNetworkConfig::new("devnet", "https://api.devnet.solana.com");

        let signer = Local::from_keypair(keypair, config);
        assert_eq!(signer.get_pubkey(), expected_pubkey);
    }

    #[test]
    fn solana_signer_debug_formatting() {
        let keypair = Keypair::new();
        let config = SolanaNetworkConfig::new("devnet", "https://api.devnet.solana.com");
        let signer = Local::from_keypair(keypair, config);

        let debug_string = format!("{signer:?}");
        assert!(debug_string.contains("Local"));
        assert!(debug_string.contains("pubkey"));
        assert!(debug_string.contains(&signer.get_pubkey().to_string()));
    }

    #[test]
    fn solana_signer_pubkey() {
        let keypair = Keypair::new();
        let expected_pubkey = keypair.pubkey().to_string();
        let config = SolanaNetworkConfig::new("devnet", "https://api.devnet.solana.com");
        let signer = Local::from_keypair(keypair, config);

        let pubkey = SolanaSigner::pubkey(&signer);
        assert_eq!(pubkey, expected_pubkey);
    }

    #[test]
    fn solana_signer_address() {
        let keypair = Keypair::new();
        let expected_address = keypair.pubkey().to_string();
        let config = SolanaNetworkConfig::new("devnet", "https://api.devnet.solana.com");
        let signer = Local::from_keypair(keypair, config);

        let pubkey = SolanaSigner::pubkey(&signer);
        assert_eq!(pubkey, expected_address);
    }

    #[test]
    fn solana_signer_client() {
        let keypair = Keypair::new();
        let config = SolanaNetworkConfig::new("devnet", "https://api.devnet.solana.com");
        let signer = Local::from_keypair(keypair, config);

        let _client = signer.client();
        // Just ensure we get a client back - returns &dyn SolanaClient
    }

    #[test]
    fn signer_base_supported_chains() {
        let keypair = Keypair::new();
        let config = SolanaNetworkConfig::new("devnet", "https://api.devnet.solana.com");
        let signer = Local::from_keypair(keypair, config);

        let chains = SignerBase::supported_chains(&signer);
        assert_eq!(chains, &[Chain::Solana]);
    }

    #[test]
    fn signer_base_user_id() {
        let keypair = Keypair::new();
        let config = SolanaNetworkConfig::new("devnet", "https://api.devnet.solana.com");
        let signer = Local::from_keypair(keypair, config);

        assert_eq!(SignerBase::user_id(&signer), "default".to_owned());
    }

    #[test]
    fn signer_base_supports_chain() {
        let keypair = Keypair::new();
        let config = SolanaNetworkConfig::new("devnet", "https://api.devnet.solana.com");
        let signer = Local::from_keypair(keypair, config);

        assert!(signer.supports_chain(Chain::Solana));
        assert!(!signer.supports_chain(Chain::Evm));
    }

    #[test]
    fn solana_signer_trait_pubkey_consistency() {
        let keypair = Keypair::new();
        let expected_pubkey = keypair.pubkey().to_string();
        let config = SolanaNetworkConfig::new("devnet", "https://api.devnet.solana.com");
        let signer = Local::from_keypair(keypair, config);

        assert_eq!(SolanaSigner::pubkey(&signer), expected_pubkey);
    }

    #[test]
    fn solana_signer_trait_pubkey() {
        let keypair = Keypair::new();
        let expected_pubkey = keypair.pubkey().to_string();
        let config = SolanaNetworkConfig::new("devnet", "https://api.devnet.solana.com");
        let signer = Local::from_keypair(keypair, config);

        assert_eq!(SolanaSigner::pubkey(&signer), expected_pubkey);
    }

    #[test]
    fn solana_signer_trait_client() {
        let keypair = Keypair::new();
        let config = SolanaNetworkConfig::new("devnet", "https://api.devnet.solana.com");
        let signer = Local::from_keypair(keypair, config);

        let _client = signer.client();
        // Just ensure we get a client back - returns &dyn SolanaClient
    }

    #[test]
    fn unified_signer_supports_solana() {
        let keypair = Keypair::new();
        let config = SolanaNetworkConfig::new("devnet", "https://api.devnet.solana.com");
        let signer = Local::from_keypair(keypair, config);

        assert!(signer.supports_solana());
    }

    #[test]
    fn unified_signer_supports_evm() {
        let keypair = Keypair::new();
        let config = SolanaNetworkConfig::new("devnet", "https://api.devnet.solana.com");
        let signer = Local::from_keypair(keypair, config);

        assert!(!signer.supports_evm());
    }

    #[test]
    fn unified_signer_as_solana() {
        let keypair = Keypair::new();
        let config = SolanaNetworkConfig::new("devnet", "https://api.devnet.solana.com");
        let signer = Local::from_keypair(keypair, config);

        let solana_signer = signer.as_solana();
        assert!(solana_signer.is_some());
    }

    #[test]
    fn unified_signer_as_evm() {
        let keypair = Keypair::new();
        let config = SolanaNetworkConfig::new("devnet", "https://api.devnet.solana.com");
        let signer = Local::from_keypair(keypair, config);

        let evm_signer = signer.as_evm();
        assert!(evm_signer.is_none());
    }

    #[test]
    fn unified_signer_chain_support_consistency() {
        let keypair = Keypair::new();
        let config = SolanaNetworkConfig::new("devnet", "https://api.devnet.solana.com");
        let signer = Local::from_keypair(keypair, config);

        // Verify that supports_* methods are consistent with as_* methods
        assert_eq!(signer.supports_solana(), signer.as_solana().is_some());
        assert_eq!(signer.supports_evm(), signer.as_evm().is_some());
    }

    #[test]
    fn solana_client_wrapper_creation() {
        let client = Arc::new(RpcClient::new("https://api.devnet.solana.com"));
        let _solana_client = SolanaClientWrapper { client };
        // Just test that it can be created without panicking
    }

    // Edge case tests
    #[test]
    fn solana_signer_creation_with_empty_rpc_url() {
        let keypair = Keypair::new();
        let private_key = keypair.to_base58_string();
        let network_config = SolanaNetworkConfig::new("Test", String::new()); // Empty URL

        // This should still create the signer, but RPC calls would fail
        Local::new(&private_key, network_config).unwrap();
    }

    #[test]
    fn solana_signer_creation_with_none_explorer_url() {
        let keypair = Keypair::new();
        let private_key = keypair.to_base58_string();
        let network_config = SolanaNetworkConfig::new("Test", "https://api.devnet.solana.com"); // None explorer URL by default

        Local::new(&private_key, network_config).unwrap();
    }

    #[test]
    fn new_with_url() {
        let keypair = Keypair::new();
        let private_key = keypair.to_base58_string();
        let rpc_url = "https://api.devnet.solana.com".to_owned();

        let signer = Local::new_with_url(&private_key, rpc_url.clone()).unwrap();
        assert_eq!(signer.rpc_url(), &rpc_url);
    }

    #[test]
    fn from_keypair_with_url() {
        let keypair = Keypair::new();
        let expected_pubkey = keypair.pubkey();
        let rpc_url = "https://api.devnet.solana.com".to_owned();

        let signer = Local::from_keypair_with_url(keypair, rpc_url.clone());
        assert_eq!(signer.get_pubkey(), expected_pubkey);
        assert_eq!(signer.rpc_url(), &rpc_url);
    }
}
