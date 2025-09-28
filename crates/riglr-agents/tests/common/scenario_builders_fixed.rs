/// Simple scenario builder implementation for Phase 4 E2E tests.
///
/// This is a simplified version to avoid compilation issues with the complex
/// scenario builder. It provides basic functionality for E2E tests.
use crate::common::BlockchainTestHarness;
use core::{error::Error as StdError, result::Result};
use solana_sdk::signer::Signer;

/// Simple scenario builder for E2E testing.
#[derive(Debug)]
pub struct SimpleScenarioBuilder {
    /// The name of the scenario being built.
    pub name: String,
}

impl SimpleScenarioBuilder {
    /// Creates a new scenario builder with the given name.
    #[must_use]
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }

    /// Create a simple SOL transfer scenario for testing.
    ///
    /// # Errors
    ///
    /// Returns an error if the SOL transfer fails or if the keypairs cannot be retrieved.
    pub fn execute_sol_transfer_test(
        harness: &BlockchainTestHarness,
        from_index: usize,
        to_index: usize,
        amount_sol: f64,
    ) -> Result<String, Box<dyn StdError + Send + Sync>> {
        let to_keypair = harness
            .get_funded_keypair(to_index)
            .ok_or("Failed to get recipient keypair")?;

        let to_pubkey = to_keypair.pubkey();

        #[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let amount_lamports = (amount_sol.max(0.0) * 1_000_000_000.0) as u64;

        let tx_info = harness.transfer_sol(from_index, &to_pubkey, amount_lamports)?;

        Ok(tx_info.signature.to_string())
    }
}
